mod commands;
mod diagnostics;
mod error;
mod library;
mod model;
mod paths;

use std::path::PathBuf;

use tauri::Manager;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let library_path = library_path(app.handle())?;
            // A damaged library recovers to empty rather than blocking startup;
            // an unreadable data directory is fatal and worth failing loudly.
            let loaded = library::load(&library_path)?;
            log::info!(
                "library: {} game(s) from {}",
                loaded.games.len(),
                library_path.display()
            );
            app.manage(AppState::new(library_path, loaded));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::import_game,
            commands::remove_game,
            commands::get_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

fn library_path(handle: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = handle.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("library.json"))
}
