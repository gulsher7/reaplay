//! The Tauri command surface — the single boundary between React and Rust.
//!
//! Mirrored by `src/ipc/` on the TypeScript side and documented in
//! `docs/IPC_CONTRACT.md`. Every command returns `Result<T, IpcError>`.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::error::{IpcError, code};
use crate::library::{self, LibraryFile};
use crate::model::{Diagnostics, Game, GameEntry};
use crate::paths;

/// Loaded once at startup and held for the life of the app.
pub struct AppState {
    pub library_path: PathBuf,
    pub library: Mutex<LibraryFile>,
}

impl AppState {
    pub fn new(library_path: PathBuf, library: LibraryFile) -> Self {
        Self {
            library_path,
            library: Mutex::new(library),
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
