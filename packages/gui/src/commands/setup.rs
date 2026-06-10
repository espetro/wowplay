use std::path::PathBuf;

use serde::Serialize;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use crate::error::CommandError;

/// Result of running the setup sequence.
#[derive(Debug, Clone, Serialize)]
pub struct SetupResult {
    pub success: bool,
    pub messages: Vec<String>,
}

/// Runs the one-time setup sequence via the wowplay sidecar.
#[tauri::command]
pub async fn run_setup(
    app: tauri::AppHandle,
    wow_dir: String,
    runner: String,
) -> Result<SetupResult, CommandError> {
    let mut args = vec!["setup".to_string(), "--wow-dir".to_string(), wow_dir];
    if runner == "whisky" || runner == "moonshine" {
        args.push("--disable-lib-silicon".to_string());
    }

    let (mut rx, _child) = app
        .shell()
        .sidecar("wowplay")
        .map_err(|e| CommandError::from(e.to_string()))?
        .args(args)
        .spawn()
        .map_err(|e| CommandError::from(e.to_string()))?;

    let mut messages: Vec<String> = Vec::new();
    let mut exit_code: Option<i32> = None;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                messages.push(String::from_utf8_lossy(&line).to_string());
            }
            CommandEvent::Stderr(line) => {
                messages.push(String::from_utf8_lossy(&line).to_string());
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    if exit_code.map(|c| c != 0).unwrap_or(false) {
        let msg = messages.join("\n");
        return Err(CommandError::from(if msg.is_empty() {
            format!("setup failed with code {}", exit_code.unwrap_or(-1))
        } else {
            msg
        }));
    }

    Ok(SetupResult {
        success: true,
        messages,
    })
}

/// Validates a WoW installation directory.
#[tauri::command]
pub async fn validate_wow_dir(path: String) -> Result<ValidationResult, CommandError> {
    let path = PathBuf::from(path);
    let wow_exe = path.join("WoW.exe").exists() || path.join("wow.exe").exists();
    let divx_patched = path.join("DivxDecoder.dll.bak").exists();

    let (valid, message, severity) = if !wow_exe {
        (
            false,
            "WoW.exe not found in this folder".to_string(),
            "error".to_string(),
        )
    } else if !divx_patched {
        (
            true,
            "DivxDecoder.dll not yet patched — will patch on launch".to_string(),
            "warning".to_string(),
        )
    } else {
        (
            true,
            "WoW installation verified".to_string(),
            "info".to_string(),
        )
    };

    Ok(ValidationResult {
        valid,
        wow_exe_found: wow_exe,
        divxdecoder_patched: divx_patched,
        message,
        severity,
    })
}

/// Result of validating a WoW directory.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub wow_exe_found: bool,
    pub divxdecoder_patched: bool,
    pub message: String,
    pub severity: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_validate_wow_dir_finds_exe() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("WoW.exe")).unwrap();
        let result = validate_wow_dir_sync(dir.path().to_string_lossy().to_string());
        assert!(result.valid);
        assert!(result.wow_exe_found);
        assert_eq!(result.severity, "warning");
    }

    #[test]
    fn test_validate_wow_dir_finds_exe_and_patch() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("WoW.exe")).unwrap();
        File::create(dir.path().join("DivxDecoder.dll.bak")).unwrap();
        let result = validate_wow_dir_sync(dir.path().to_string_lossy().to_string());
        assert!(result.valid);
        assert!(result.wow_exe_found);
        assert!(result.divxdecoder_patched);
        assert_eq!(result.severity, "info");
    }

    #[test]
    fn test_validate_wow_dir_missing_exe() {
        let dir = TempDir::new().unwrap();
        let result = validate_wow_dir_sync(dir.path().to_string_lossy().to_string());
        assert!(!result.valid);
        assert!(!result.wow_exe_found);
        assert_eq!(result.severity, "error");
    }

    fn validate_wow_dir_sync(path: String) -> ValidationResult {
        let path = PathBuf::from(path);
        let wow_exe = path.join("WoW.exe").exists() || path.join("wow.exe").exists();
        let divx_patched = path.join("DivxDecoder.dll.bak").exists();

        let (valid, message, severity) = if !wow_exe {
            (
                false,
                "WoW.exe not found in this folder".to_string(),
                "error".to_string(),
            )
        } else if !divx_patched {
            (
                true,
                "DivxDecoder.dll not yet patched — will patch on launch".to_string(),
                "warning".to_string(),
            )
        } else {
            (
                true,
                "WoW installation verified".to_string(),
                "info".to_string(),
            )
        };

        ValidationResult {
            valid,
            wow_exe_found: wow_exe,
            divxdecoder_patched: divx_patched,
            message,
            severity,
        }
    }
}
