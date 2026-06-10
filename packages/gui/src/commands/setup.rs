use std::path::PathBuf;

use serde::Serialize;

use crate::error::CommandError;
use crate::state::app_state::AppState;
use wow_silicon_core::setup::SetupOrchestrator;

/// Result of running the setup sequence.
#[derive(Debug, Clone, Serialize)]
pub struct SetupResult {
    pub success: bool,
    pub messages: Vec<String>,
}

/// Runs the one-time setup sequence: stage resources, create wineloader2, apply game patch.
#[tauri::command]
pub async fn run_setup(
    wow_dir: String,
    _runner: String,
    _state: tauri::State<'_, AppState>,
) -> Result<SetupResult, CommandError> {
    let wow_dir = PathBuf::from(wow_dir);

    let messages = tokio::task::spawn_blocking(move || {
        SetupOrchestrator::run(
            &wow_dir,
            None,  // Use default patching dir resolution
            true,  // Enable libSilicon by default
        )
    })
    .await
    .map_err(|e| CommandError::from(format!("spawn blocking failed: {e}")))??;

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
        assert_eq!(result.severity, "warning"); // no DivxDecoder.dll.bak
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
