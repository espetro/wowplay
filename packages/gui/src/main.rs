mod commands;
mod error;
mod state;

use state::app_state::{AppConfig, AppState};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let config = app
                .store_builder("config.json")
                .build()
                .ok()
                .and_then(|store| store.get("config"))
                .and_then(|val| serde_json::from_value::<AppConfig>(val).ok())
                .unwrap_or_default();

            app.manage(AppState::from_config(config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::set_config,
            commands::diagnostics::check_runners,
            commands::setup::run_setup,
            commands::launch::launch_wow,
            commands::setup::validate_wow_dir,
            commands::config::reset_config,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
