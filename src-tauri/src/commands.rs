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
        chosen.stop_command(),
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

    /// The end-to-end tests share one CrossOver bottle, and stopping a game
    /// stops everything in that bottle, so two of them running at once would
    /// kill each other's processes. Cargo runs tests in parallel by default,
    /// so they take this lock rather than relying on `--test-threads=1`.
    static BOTTLE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Take the bottle lock, ignoring poisoning: a panic in one end-to-end
    /// test should not cascade into "every other test failed too".
    fn bottle_lock() -> std::sync::MutexGuard<'static, ()> {
        BOTTLE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

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

    /// End-to-end against the real CrossOver installed on this machine:
    /// import a genuine Windows executable and launch it through exactly the
    /// code path the UI calls.
    ///
    /// `cmd.exe` with no arguments exits as soon as it reads EOF, and the
    /// launcher gives every process a null stdin, so it terminates by itself.
    ///
    /// Ignored by default: it needs CrossOver and a bottle, which CI does not
    /// have. Run it with `cargo test --lib -- --ignored --nocapture`.
    #[test]
    #[ignore = "requires an installed CrossOver with a bottle"]
    fn a_real_windows_executable_runs_through_crossover() {
        let _guard = bottle_lock();
        let bottles = crate::diagnostics::crossover_bottles();
        let Some(bottle) = bottles.first() else {
            panic!("no CrossOver bottle found; create one before running this");
        };
        let cmd_exe = PathBuf::from(std::env::var_os("HOME").expect("HOME"))
            .join("Library/Application Support/CrossOver/Bottles")
            .join(bottle)
            .join("drive_c/windows/system32/cmd.exe");
        assert!(cmd_exe.is_file(), "no cmd.exe at {}", cmd_exe.display());

        let (state, _dir) = test_state();
        let imported = import(
            &state,
            &cmd_exe.display().to_string(),
            Some("Windows Command Prompt"),
        )
        .expect("import a real Windows executable");

        let (tx, rx) = std::sync::mpsc::channel();
        let started = launch_game_inner(imported.game.id.clone(), &state, move |finished| {
            let _ = tx.send(finished);
        })
        .expect("launch should start");
        assert_eq!(started.status, LaunchStatus::Running);
        assert_eq!(started.provider_id, "crossover");

        let finished = rx
            .recv_timeout(std::time::Duration::from_secs(180))
            .expect("the launch should finish");

        println!("program : {}", finished.program);
        println!("args    : {:?}", finished.args);
        println!("status  : {:?}", finished.status);
        println!("exit    : {:?}", finished.exit_code);
        println!(
            "stdout  : {}",
            finished.stdout_tail.as_deref().unwrap_or("")
        );
        println!(
            "stderr  : {}",
            finished.stderr_tail.as_deref().unwrap_or("")
        );

        assert_eq!(
            finished.status,
            LaunchStatus::Exited,
            "{:?}",
            finished.error
        );
        assert_eq!(finished.exit_code, Some(0));
        // Proof that real Windows code ran, not merely that a process started.
        assert!(
            finished
                .stdout_tail
                .as_deref()
                .unwrap_or("")
                .contains("Microsoft Windows"),
            "cmd.exe banner missing from captured stdout"
        );

        // The history must record the finished launch, not the running one.
        let history = list_launches_inner(&state).expect("history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, LaunchStatus::Exited);
    }

    /// The whole point of the project, end to end: import a real Windows game
    /// and run it through ReaPlay's own launch pipeline.
    ///
    /// A game does not exit on its own, so this also exercises the part that
    /// matters in practice — tracking a running process and terminating it.
    ///
    /// Ignored by default: it needs CrossOver, a bottle, and Anarch present.
    /// Run it with `cargo test --lib -- --ignored --nocapture`.
    #[test]
    #[ignore = "requires CrossOver and a local copy of Anarch"]
    fn a_real_windows_game_launches_and_can_be_stopped() {
        let _guard = bottle_lock();
        let game_exe = PathBuf::from(
            "/Users/gulsher/Desktop/lets-talk/games/anarch/Anarch_winshitxp_sdl_1-1.exe",
        );
        if !game_exe.is_file() {
            panic!("Anarch is not at {}", game_exe.display());
        }

        // A leftover game from an earlier run would satisfy every assertion
        // below without this run proving anything, so refuse to start dirty.
        assert!(
            !a_windows_process_is_running("Anarch_winshitxp_sdl_1-1.exe"),
            "an earlier Anarch is still running; stop it before running this"
        );

        let (state, _dir) = test_state();
        let imported = import(&state, &game_exe.display().to_string(), Some("Anarch"))
            .expect("import the game");

        let (tx, rx) = std::sync::mpsc::channel();
        let started = launch_game_inner(imported.game.id.clone(), &state, move |finished| {
            let _ = tx.send(finished);
        })
        .expect("launch should start");

        println!("program : {}", started.program);
        println!("args    : {:?}", started.args);
        assert_eq!(started.provider_id, "crossover");
        assert_eq!(started.status, LaunchStatus::Running);

        // While it runs, the library view must show it as running.
        let during = list_launches_inner(&state).expect("history during");
        assert_eq!(during[0].status, LaunchStatus::Running);

        // Let the game actually start before stopping it. Terminating in the
        // first milliseconds would test a race, not the real behaviour: a user
        // pressing Stop does it to a game that is up and running.
        let appeared = (0..100).any(|_| {
            if a_windows_process_is_running("Anarch_winshitxp_sdl_1-1.exe") {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
            false
        });
        assert!(appeared, "the game never started");

        // Exactly what the `terminate_launch` command does.
        state
            .registry
            .terminate(&started.id)
            .expect("terminate the game");

        let finished = rx
            .recv_timeout(std::time::Duration::from_secs(60))
            .expect("terminating should produce an outcome");
        println!("status  : {:?}", finished.status);
        println!("exit    : {:?}", finished.exit_code);
        println!(
            "stdout  : {}",
            finished.stdout_tail.as_deref().unwrap_or("")
        );
        println!(
            "stderr  : {}",
            finished.stderr_tail.as_deref().unwrap_or("")
        );

        // Terminated, not "exited cleanly": the distinction is the point.
        assert_ne!(finished.status, LaunchStatus::Running);
        let after = list_launches_inner(&state).expect("history after");
        assert_eq!(after.len(), 1);
        assert_ne!(after[0].status, LaunchStatus::Running);

        // The assertion that actually matters, and the one whose absence hid a
        // real bug: the *game* must be gone, not merely the process ReaPlay
        // spawned. Wine reparents the game to init, so an earlier version of
        // this code reported a clean stop while the game kept running.
        assert!(
            !a_windows_process_is_running("Anarch_winshitxp_sdl_1-1.exe"),
            "ReaPlay reported the game stopped, but it is still running"
        );
    }

    /// Whether the *game* is running, as opposed to the launcher that starts
    /// it.
    ///
    /// This matches on the Windows path separator, which matters: the `wine`
    /// wrapper ReaPlay spawns carries the game's **native** path in its own
    /// arguments (`/Users/.../game.exe`), so a search for the bare file name
    /// matches the wrapper the instant it starts, before the game exists. The
    /// Wine-side process is the one whose command line is a Windows path
    /// (`Y:\...\game.exe`), so requiring a preceding backslash finds the game
    /// and nothing else.
    #[cfg(test)]
    fn a_windows_process_is_running(exe_name: &str) -> bool {
        let pattern = format!(r"\\{exe_name}");
        let output = std::process::Command::new("/usr/bin/pgrep")
            .args(["-f", &pattern])
            .output()
            .expect("pgrep should run");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| !line.trim().is_empty())
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
