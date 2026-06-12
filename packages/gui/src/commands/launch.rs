use std::path::PathBuf;

use wow_silicon_core::integration::wow_launcher::WowLauncher;
use wow_silicon_core::options::LaunchOptions;

use crate::error::CommandError;
use crate::logging::make_log_path;
use crate::state::app_state::AppState;

/// Launches WoW directly via rust-core and returns the process PID.
#[tauri::command]
#[specta::specta]
pub async fn launch_wow(
    wow_dir: String,
    runner: String,
    bottle: String,
    state: tauri::State<'_, AppState>,
) -> Result<u32, CommandError> {
    let log_path = make_log_path();
    let options = LaunchOptions {
        wow_dir: PathBuf::from(&wow_dir),
        runner,
        bottle,
        ..LaunchOptions::default()
    };

    let session = tokio::task::spawn_blocking(move || {
        let wow_dir_path = options.wow_dir.clone();
        let launcher = WowLauncher::from_options(options)?;
        launcher.launch_wow_logged(&wow_dir_path, log_path.as_deref())
    })
    .await
    .map_err(|e| CommandError::from(e.to_string()))?
    .map_err(|e| CommandError::from(e.to_string()))?;

    let pid = session.pid();

    // Detach a waiter so WoW can outlive this command handler.
    tokio::task::spawn_blocking(move || {
        let _ = session.wait();
    });

    let mut process = state
        .wow_process
        .lock()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *process = Some(pid);

    Ok(pid)
}
