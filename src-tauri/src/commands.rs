//! The Tauri command surface — the single boundary between React and Rust.
//!
//! Mirrored by `src/ipc/` on the TypeScript side and documented in
//! `docs/IPC_CONTRACT.md`. Every command returns `Result<T, IpcError>`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};

use crate::error::{IpcError, code};
use crate::history::{self, HistoryFile};
use crate::launch::{self, LaunchRecord, LaunchStatus, Registry};
use crate::library::{self, LibraryFile};
use crate::model::{Availability, Diagnostics, Game, GameEntry};
use crate::paths;
use crate::provider::{self, ProviderInfo};
use crate::report;

/// Emitted when a launched game exits, with the finished `LaunchRecord`.
pub const LAUNCH_FINISHED_EVENT: &str = "reaplay://launch-finished";

/// Loaded once at startup and held for the life of the app.
pub struct AppState {
    pub library_path: PathBuf,
    pub library: Mutex<LibraryFile>,
    pub history_path: PathBuf,
    pub history: Arc<Mutex<HistoryFile>>,
    pub registry: Arc<Registry>,
}

impl AppState {
    pub fn new(
        library_path: PathBuf,
        library: LibraryFile,
        history_path: PathBuf,
        history: HistoryFile,
    ) -> Self {
        Self {
            library_path,
            library: Mutex::new(library),
            history_path,
            history: Arc::new(Mutex::new(history)),
            registry: Arc::new(Registry::new()),
        }
    }

    fn with_library<T>(
        &self,
        action: impl FnOnce(&mut LibraryFile) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        let mut guard = self
            .library
            .lock()
            .map_err(|_| IpcError::internal("library lock poisoned"))?;
        action(&mut guard)
    }

    fn with_history<T>(
        &self,
        action: impl FnOnce(&mut HistoryFile) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        let mut guard = self
            .history
            .lock()
            .map_err(|_| IpcError::internal("history lock poisoned"))?;
        action(&mut guard)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportGameRequest {
    pub executable_path: String,
    /// Optional override. When absent, a title is derived from the file name.
    #[serde(default)]
    pub title: Option<String>,
}

/// Every game in the library, each with freshly checked availability.
#[tauri::command]
pub fn list_games(state: State<AppState>) -> Result<Vec<GameEntry>, IpcError> {
    list_games_inner(&state)
}

pub fn list_games_inner(state: &AppState) -> Result<Vec<GameEntry>, IpcError> {
    state.with_library(|file| {
        Ok(file
            .games
            .iter()
            .cloned()
            .map(|game| {
                let availability = paths::availability_of(&game.executable_path);
                GameEntry { game, availability }
            })
            .collect())
    })
}

/// Validate an executable and add it to the library. Copies nothing, installs
/// nothing, and launches nothing.
#[tauri::command]
pub fn import_game(
    request: ImportGameRequest,
    state: State<AppState>,
) -> Result<GameEntry, IpcError> {
    import_game_inner(request, &state)
}

pub fn import_game_inner(
    request: ImportGameRequest,
    state: &AppState,
) -> Result<GameEntry, IpcError> {
    let canonical = paths::validate_executable(&request.executable_path)?;
    let executable_path = canonical.display().to_string();

    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| paths::title_from_path(&canonical));

    let install_directory = canonical
        .parent()
        .map(|parent| parent.display().to_string());

    let now = chrono::Utc::now().to_rfc3339();
    let game = Game {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        executable_path: executable_path.clone(),
        install_directory,
        provider_id: None,
        profile_id: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let path = state.library_path.clone();
    state.with_library(|file| {
        if let Some(existing) = file
            .games
            .iter()
            .find(|g| g.executable_path == executable_path)
        {
            return Err(IpcError::detailed(
                code::GAME_ALREADY_IMPORTED,
                format!("“{}” is already in your library.", existing.title),
                existing.id.clone(),
            ));
        }

        file.games.push(game.clone());
        // Persist before reporting success, so a failed write is not reported
        // as an import that happened.
        library::save(&path, file)?;
        Ok(GameEntry {
            game,
            availability: paths::availability_of(&executable_path),
        })
    })
}

/// Forget a game. Its files are left untouched on disk.
#[tauri::command]
pub fn remove_game(id: String, state: State<AppState>) -> Result<(), IpcError> {
    remove_game_inner(id, &state)
}

pub fn remove_game_inner(id: String, state: &AppState) -> Result<(), IpcError> {
    let path = state.library_path.clone();
    state.with_library(|file| {
        let before = file.games.len();
        file.games.retain(|game| game.id != id);
        if file.games.len() == before {
            return Err(IpcError::detailed(
                code::GAME_NOT_FOUND,
                "That game is no longer in your library.",
                id.clone(),
            ));
        }
        library::save(&path, file)
    })
}

/// Host information and detected compatibility tools.
#[tauri::command]
pub fn get_diagnostics(app: AppHandle, state: State<AppState>) -> Result<Diagnostics, IpcError> {
    Ok(Diagnostics {
        app_version: app.package_info().version.to_string(),
        os_version: crate::diagnostics::os_version(),
        arch: crate::diagnostics::arch(),
        capabilities: crate::diagnostics::detect_capabilities(),
        library_path: state.library_path.display().to_string(),
    })
}

/// A redacted diagnostic report, safe to paste into a bug report.
#[tauri::command]
pub fn export_diagnostics(app: AppHandle, state: State<AppState>) -> Result<String, IpcError> {
    export_diagnostics_inner(app.package_info().version.to_string(), &state)
}

pub fn export_diagnostics_inner(app_version: String, state: &AppState) -> Result<String, IpcError> {
    let diagnostics = Diagnostics {
        app_version,
        os_version: crate::diagnostics::os_version(),
        arch: crate::diagnostics::arch(),
        capabilities: crate::diagnostics::detect_capabilities(),
        library_path: state.library_path.display().to_string(),
    };
    let providers: Vec<ProviderInfo> = provider::all()
        .into_iter()
        .map(|provider| provider.info())
        .collect();
    let games = list_games_inner(state)?;
    let launches = list_launches_inner(state)?;

    Ok(report::build(&diagnostics, &providers, &games, &launches))
}

/// Compatibility providers and whether each is usable right now.
#[tauri::command]
pub fn list_providers() -> Vec<ProviderInfo> {
    provider::all()
        .into_iter()
        .map(|provider| provider.info())
        .collect()
}

/// Start a game. Returns as soon as the process is running; the finished
/// record arrives later on the `reaplay://launch-finished` event.
#[tauri::command]
pub fn launch_game(
    app: AppHandle,
    id: String,
    state: State<AppState>,
) -> Result<LaunchRecord, IpcError> {
    let handle = app.clone();
    launch_game_inner(id, &state, move |record| {
        if let Err(error) = handle.emit(LAUNCH_FINISHED_EVENT, record) {
            log::error!("could not emit launch outcome: {error}");
        }
    })
}

/// The launch logic, independent of Tauri so it can be tested directly.
///
/// `on_finish` is called once the process exits, after the outcome has been
/// written to history.
pub fn launch_game_inner<F>(
    id: String,
    state: &AppState,
    on_finish: F,
) -> Result<LaunchRecord, IpcError>
where
    F: FnOnce(LaunchRecord) + Send + 'static,
{
    let game = state.with_library(|file| {
        file.games
            .iter()
            .find(|game| game.id == id)
            .cloned()
            .ok_or_else(|| {
                IpcError::detailed(
                    code::GAME_NOT_FOUND,
                    "That game is no longer in your library.",
                    id.clone(),
                )
            })
    })?;

    // Re-check the files now rather than trusting what the list last reported:
    // a drive can be unplugged between listing and launching.
    match paths::availability_of(&game.executable_path) {
        Availability::Available => {}
        Availability::ExecutableMissing => {
            return Err(IpcError::detailed(
                code::GAME_UNAVAILABLE,
                format!("“{}” is missing from disk.", game.title),
                game.executable_path.clone(),
            ));
        }
        Availability::VolumeUnavailable => {
            return Err(IpcError::detailed(
                code::VOLUME_UNAVAILABLE,
                format!("The drive holding “{}” is not connected.", game.title),
                game.executable_path.clone(),
            ));
        }
    }

    let chosen = provider::resolve(game.provider_id.as_deref())?;
    let info = chosen.info();
    if !info.available {
        return Err(IpcError::detailed(
            code::PROVIDER_UNAVAILABLE,
            format!(
                "{} is not installed, so this game cannot be launched.",
                info.display_name
            ),
            info.detail.unwrap_or_default(),
        ));
    }

    let executable = PathBuf::from(&game.executable_path);
    let (program, args) = chosen.command_for(&executable)?;
    let working_directory =
        launch::working_directory(&executable, game.install_directory.as_deref());

    let record = LaunchRecord {
        id: uuid::Uuid::new_v4().to_string(),
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        provider_id: info.id.clone(),
        program: program.display().to_string(),
        args: args.clone(),
        started_at: chrono::Utc::now().to_rfc3339(),
        finished_at: None,
        status: LaunchStatus::Running,
        exit_code: None,
        stdout_tail: None,
        stderr_tail: None,
        error: None,
    };

    let history_path = state.history_path.clone();
    let started = record.clone();
    state.with_history(|file| {
        file.push(started);
        history::save(&history_path, file)
    })?;

    // Shared handles rather than an AppHandle, so the finish path works
    // identically under test.
    let history = state.history.clone();
    let finish_path = state.history_path.clone();
    let pending = record.clone();

    launch::start_tracked(
        state.registry.clone(),
        record.id.clone(),
        &program,
        &args,
        working_directory.as_deref(),
        move |outcome| {
            let mut finished = pending;
            finished.status = outcome.status;
            finished.exit_code = outcome.exit_code;
            finished.finished_at = Some(chrono::Utc::now().to_rfc3339());
            finished.stdout_tail = non_empty(outcome.stdout_tail);
            finished.stderr_tail = non_empty(outcome.stderr_tail);

            match history.lock() {
                Ok(mut file) => {
                    file.replace(finished.clone());
                    if let Err(error) = history::save(&finish_path, &file) {
                        log::error!("could not record launch outcome: {error}");
                    }
                }
                Err(_) => log::error!("history lock poisoned while recording a launch"),
            }

            on_finish(finished);
        },
    )?;

    Ok(record)
}

/// Ask a running game to stop.
#[tauri::command]
pub fn terminate_launch(launch_id: String, state: State<AppState>) -> Result<(), IpcError> {
    state.registry.terminate(&launch_id)
}

/// Recent launches, newest first, with whether each is still running.
#[tauri::command]
pub fn list_launches(state: State<AppState>) -> Result<Vec<LaunchRecord>, IpcError> {
    list_launches_inner(&state)
}

pub fn list_launches_inner(state: &AppState) -> Result<Vec<LaunchRecord>, IpcError> {
    let running = state.registry.running_ids();
    state.with_history(|file| {
        Ok(file
            .launches
            .iter()
            .cloned()
            .map(|mut record| {
                // A record left as Running by a crash should not look live.
                if record.status == LaunchStatus::Running && !running.contains(&record.id) {
                    record.status = LaunchStatus::Terminated;
                }
                record
            })
            .collect())
    })
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// A real `AppState` backed by temporary files: these tests exercise the
    /// same code paths the Tauri commands call, without a Tauri runtime.
    fn test_state() -> (AppState, PathBuf) {
        let dir = std::env::temp_dir().join(format!("reaplay-cmd-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("temp dir");
        let state = AppState::new(
            dir.join("library.json"),
            LibraryFile::default(),
            dir.join("launches.json"),
            HistoryFile::default(),
        );
        (state, dir)
    }

    fn stub_exe(dir: &Path, name: &str) -> String {
        let path = dir.join(name);
        fs::write(&path, b"MZ").expect("write stub exe");
        path.display().to_string()
    }

    fn import(state: &AppState, path: &str, title: Option<&str>) -> Result<GameEntry, IpcError> {
        import_game_inner(
            ImportGameRequest {
                executable_path: path.to_string(),
                title: title.map(str::to_string),
            },
            state,
        )
    }

    #[test]
    fn import_then_list_round_trips() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "Celeste.exe");

        let imported = import(&state, &exe, None).expect("import");
        assert_eq!(imported.game.title, "Celeste");
        assert!(matches!(imported.availability, Availability::Available));

        let listed = list_games_inner(&state).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].game.id, imported.game.id);
        assert_eq!(
            listed[0].game.install_directory.as_deref(),
            Some(
                dir.canonicalize()
                    .expect("canonical")
                    .display()
                    .to_string()
                    .as_str()
            )
        );
    }

    #[test]
    fn an_explicit_title_wins_over_the_derived_one() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "game_final_2.exe");
        let imported = import(&state, &exe, Some("  My Game  ")).expect("import");
        assert_eq!(imported.game.title, "My Game");
    }

    #[test]
    fn a_blank_title_falls_back_to_the_derived_one() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "my_cool_game.exe");
        let imported = import(&state, &exe, Some("   ")).expect("import");
        assert_eq!(imported.game.title, "My cool game");
    }

    #[test]
    fn importing_the_same_executable_twice_is_refused() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "dup.exe");
        import(&state, &exe, None).expect("first import");
        let error = import(&state, &exe, None).expect_err("second import");
        assert_eq!(error.code, code::GAME_ALREADY_IMPORTED);
        assert_eq!(list_games_inner(&state).expect("list").len(), 1);
    }

    #[test]
    fn an_import_survives_a_restart() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "persisted.exe");
        import(&state, &exe, None).expect("import");

        // Reload from disk exactly as startup does.
        let reloaded = library::load(&state.library_path).expect("reload");
        assert_eq!(reloaded.games.len(), 1);
        assert_eq!(reloaded.games[0].title, "Persisted");
    }

    #[test]
    fn remove_forgets_the_game_but_leaves_the_file() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "removable.exe");
        let imported = import(&state, &exe, None).expect("import");

        remove_game_inner(imported.game.id.clone(), &state).expect("remove");
        assert!(list_games_inner(&state).expect("list").is_empty());
        assert!(
            Path::new(&exe).exists(),
            "the game file must not be deleted"
        );

        let error = remove_game_inner(imported.game.id, &state).expect_err("second remove");
        assert_eq!(error.code, code::GAME_NOT_FOUND);
    }

    #[test]
    fn a_game_whose_file_vanished_is_reported_unavailable() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "vanishing.exe");
        import(&state, &exe, None).expect("import");

        fs::remove_file(&exe).expect("delete the executable");

        let listed = list_games_inner(&state).expect("list");
        assert!(matches!(
            listed[0].availability,
            Availability::ExecutableMissing
        ));
    }

    #[test]
    fn launching_an_unknown_game_is_rejected() {
        let (state, _dir) = test_state();
        let error =
            launch_game_inner("no-such-game".into(), &state, |_| {}).expect_err("should reject");
        assert_eq!(error.code, code::GAME_NOT_FOUND);
    }

    #[test]
    fn launching_a_game_whose_file_vanished_is_rejected_before_any_process_starts() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "gone.exe");
        let imported = import(&state, &exe, None).expect("import");
        fs::remove_file(&exe).expect("delete");

        let error = launch_game_inner(imported.game.id, &state, |_| {}).expect_err("should reject");
        assert_eq!(error.code, code::GAME_UNAVAILABLE);
        // Nothing should have been recorded: no launch was attempted.
        assert!(list_launches_inner(&state).expect("list").is_empty());
    }

    #[test]
    fn launching_without_a_compatibility_provider_fails_cleanly() {
        if provider::all().iter().any(|p| p.info().available) {
            // A provider is installed on this host, so this path is not
            // reachable here. The companion assertion lives in the
            // provider tests.
            return;
        }
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "needs-wine.exe");
        let imported = import(&state, &exe, None).expect("import");

        let error = launch_game_inner(imported.game.id, &state, |_| {}).expect_err("should fail");
        assert_eq!(error.code, code::PROVIDER_UNAVAILABLE);
        // A refused launch must not leave a phantom entry in the history.
        assert!(list_launches_inner(&state).expect("list").is_empty());
    }

    #[test]
    fn an_unknown_provider_id_pinned_on_a_game_is_reported() {
        let (state, dir) = test_state();
        let exe = stub_exe(&dir, "pinned.exe");
        let imported = import(&state, &exe, None).expect("import");

        state
            .with_library(|file| {
                file.games[0].provider_id = Some("not-a-provider".into());
                Ok(())
            })
            .expect("pin provider");

        let error = launch_game_inner(imported.game.id, &state, |_| {}).expect_err("should fail");
        assert_eq!(error.code, code::PROVIDER_NOT_FOUND);
    }
}
