use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use wow_silicon_core::options::SetupOptions;
use wow_silicon_core::setup::SetupOrchestrator;

use crate::error::CommandError;
use crate::logging::make_log_path;

/// Result of running the setup sequence.
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct SetupResult {
    pub success: bool,
    pub messages: Vec<String>,
}

/// Runs the one-time setup sequence directly via rust-core.
#[tauri::command]
#[specta::specta]
pub async fn run_setup(
    wow_dir: String,
    runner: String,
) -> Result<SetupResult, CommandError> {
    let _ = runner; // kept for Tauri command ABI; frontend passes it
    let options = SetupOptions {
        wow_dir: PathBuf::from(&wow_dir),
        ..Default::default()
    };

    let messages = tokio::task::spawn_blocking(move || SetupOrchestrator::run(&options))
        .await
        .map_err(|e| CommandError::from(e.to_string()))?
        .map_err(|e| {
            if let Some(log_path) = make_log_path() {
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    let _ = writeln!(f, "[fail] {e}");
                }
            }
            CommandError::from(e.to_string())
        })?;

    if let Some(log_path) = make_log_path() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            for msg in &messages {
                let _ = writeln!(f, "[ ok ] {msg}");
            }
        }
    }

    Ok(SetupResult { success: true, messages })
}

/// Validates a WoW installation directory.
#[tauri::command]
#[specta::specta]
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
#[derive(Debug, Clone, Serialize, specta::Type)]
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
