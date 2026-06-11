use std::io::Write;
use std::sync::{Arc, Mutex};

use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tokio::sync::oneshot;

use crate::error::CommandError;
use crate::logging::make_log_path;
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
    let args = vec![
        "run".to_string(),
        "--wow-dir".to_string(),
        wow_dir,
        "--runner".to_string(),
        runner.clone(),
        "--bottle".to_string(),
        bottle,
    ];
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
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = Arc::clone(&captured);
    let (fail_tx, fail_rx) = oneshot::channel::<String>();
    let drain_handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => {
                    let s = String::from_utf8_lossy(&line).to_string();
                    captured_clone.lock().unwrap().push(s);
                }
                CommandEvent::Terminated(payload) => {
                    if payload.code.map(|c| c != 0).unwrap_or(false) {
                        let msg = {
                            let buf = captured_clone.lock().unwrap();
                            if buf.is_empty() {
                                format!("wowplay exited with code {}", payload.code.unwrap_or(-1))
                            } else {
                                buf.join("\n")
                            }
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
            if let Some(log_path) = make_log_path() {
                if let Ok(mut f) =
                    std::fs::OpenOptions::new().create(true).append(true).open(&log_path)
                {
                    let _ = writeln!(f, "[fail] {err_msg}");
                }
            }
            return Err(CommandError::from(err_msg));
        }
        _ => {
            // Timeout (process still running) or clean exit — write initial output to log.
            if let Some(log_path) = make_log_path() {
                if let Ok(mut f) =
                    std::fs::OpenOptions::new().create(true).append(true).open(&log_path)
                {
                    for line in captured.lock().unwrap().iter() {
                        let _ = writeln!(f, "{line}");
                    }
                }
            }
        }
    }

    let mut process = state
        .wow_process
        .write()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *process = Some(child);

    Ok(pid)
}
