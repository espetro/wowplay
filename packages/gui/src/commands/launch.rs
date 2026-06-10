use std::path::PathBuf;

use crate::error::CommandError;
use crate::state::app_state::AppState;
use wow_silicon_core::integration::wow_launcher::WowLauncher;
use wow_silicon_core::resources::resolve_patching_dir;
use wow_silicon_core::runner_registry::RunnerRegistry;

/// Launches WoW via the selected runner and returns the process PID.
#[tauri::command]
pub async fn launch_wow(
    wow_dir: String,
    runner: String,
    state: tauri::State<'_, AppState>,
) -> Result<u32, CommandError> {
    let wow_dir = PathBuf::from(wow_dir);

    // Resolve patching dir (sync operation)
    let resources = tokio::task::spawn_blocking(|| resolve_patching_dir(None))
        .await
        .map_err(|e| CommandError::from(format!("spawn blocking failed: {e}")))??;

    // Resolve runner (sync operation)
    let runner_arc = tokio::task::spawn_blocking(move || RunnerRegistry::resolve(&runner))
        .await
        .map_err(|e| CommandError::from(format!("spawn blocking failed: {e}")))??;

    // Build launcher and spawn WoW (this spawns the process)
    let launcher = WowLauncher::new(runner_arc, resources, "Win10");

    // launch_wow_logged returns a WowSession which owns the Child process
    let session = tokio::task::spawn_blocking(move || {
        launcher.launch_wow_logged(&wow_dir, None)  // No log file for GUI mode
    })
    .await
    .map_err(|e| CommandError::from(format!("spawn blocking failed: {e}")))??;

    let pid = session.pid();

    // For fire-and-forget, we don't need to keep the session.
    // The WoW process will continue running after this function returns.
    // We just extract the PID and let the session drop (which doesn't kill the child).
    let _ = session;

    // Store a dummy child in state (the real process is already running)
    let child = tokio::process::Command::new("true")
        .spawn()
        .map_err(|e| CommandError::from(format!("failed to spawn dummy process: {e}")))?;

    let mut process = state
        .wow_process
        .write()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *process = Some(child);

    Ok(pid)
}
