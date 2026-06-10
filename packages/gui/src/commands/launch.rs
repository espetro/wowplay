use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tokio::sync::oneshot;

use crate::error::CommandError;
use crate::state::app_state::AppState;

/// Launches WoW via the wowplay sidecar and returns the process PID.
#[tauri::command]
pub async fn launch_wow(
    app: tauri::AppHandle,
    wow_dir: String,
    runner: String,
    bottle: String,
    state: tauri::State<'_, AppState>,
) -> Result<u32, CommandError> {
    let mut args = vec![
        "run".to_string(),
        "--wow-dir".to_string(),
        wow_dir,
        "--runner".to_string(),
        runner.clone(),
        "--bottle".to_string(),
        bottle,
    ];
    if runner == "whisky" || runner == "moonshine" {
        args.push("--disable-lib-silicon".to_string());
    }

    let (mut rx, child) = app
        .shell()
        .sidecar("wowplay")
        .map_err(|e| CommandError::from(e.to_string()))?
        .args(args)
        .spawn()
        .map_err(|e| CommandError::from(e.to_string()))?;

    let pid = child.pid();

    // Drain output for ~1.5 s to surface fast failures (unknown runner, missing WoW.exe, etc.)
    // while not blocking on a successful long-running launch.
    let (fail_tx, fail_rx) = oneshot::channel::<String>();
    let drain_handle = tokio::spawn(async move {
        let mut stderr_buf: Vec<String> = Vec::new();
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stderr(line) => {
                    stderr_buf.push(String::from_utf8_lossy(&line).to_string());
                }
                CommandEvent::Terminated(payload) => {
                    if payload.code.map(|c| c != 0).unwrap_or(false) {
                        let msg = if stderr_buf.is_empty() {
                            format!("wowplay exited with code {}", payload.code.unwrap_or(-1))
                        } else {
                            stderr_buf.join("\n")
                        };
                        let _ = fail_tx.send(msg);
                    }
                    return;
                }
                _ => {}
            }
        }
    });

    match tokio::time::timeout(tokio::time::Duration::from_millis(1500), fail_rx).await {
        Ok(Ok(err_msg)) => {
            drain_handle.abort();
            return Err(CommandError::from(err_msg));
        }
        _ => {
            // Timeout or clean exit: process is running — drain task detaches naturally
        }
    }

    let mut process = state
        .wow_process
        .write()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *process = Some(child);

    Ok(pid)
}
