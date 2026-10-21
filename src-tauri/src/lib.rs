mod commands;
mod diagnostics;
mod error;
mod history;
mod launch;
mod library;
mod model;
mod paths;
mod provider;
mod store;

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

            let data_dir = data_dir(app.handle())?;
            let library_path = data_dir.join("library.json");
            let history_path = data_dir.join("launches.json");

            // Damaged stores recover to empty rather than blocking startup; an
            // unreachable data directory is fatal and worth failing loudly.
            let loaded = library::load(&library_path)?;
            let past = history::load(&history_path)?;
            log::info!(
                "library: {} game(s), {} past launch(es), from {}",
                loaded.games.len(),
                past.launches.len(),
                data_dir.display()
            );

            app.manage(AppState::new(library_path, loaded, history_path, past));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::import_game,
            commands::remove_game,
            commands::get_diagnostics,
            commands::list_providers,
            commands::launch_game,
            commands::terminate_launch,
            commands::list_launches,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

fn data_dir(handle: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = handle.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
