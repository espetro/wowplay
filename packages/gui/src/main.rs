mod commands;
mod error;
mod logging;
mod state;

use state::app_state::AppState;

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // AppState is now stateless — no config loading at startup
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::set_config,
            commands::config::list_config,
            commands::diagnostics::check_runners,
            commands::setup::run_setup,
            commands::setup::run_reset,
            commands::launch::launch_wow,
            commands::setup::validate_wow_dir,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
