use std::path::PathBuf;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tauri_plugin_shell::process::CommandChild;

/// Persisted application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Selected runner name (e.g. "crossover", "whisky", "moonshine").
    pub runner: Option<String>,
    /// Path to the WoW 3.3.5a game directory.
    pub wow_dir: Option<PathBuf>,
    /// Whether to show info-level alerts in the UI.
    #[serde(default)]
    pub show_alerts: bool,
    /// Wine/CrossOver bottle name (e.g. "Win10").
    #[serde(default)]
    pub bottle: Option<String>,
}

/// Shared mutable state across Tauri commands.
pub struct AppState {
    /// Current application configuration.
    pub config: RwLock<AppConfig>,
    /// Handle to the spawned wowplay sidecar process, if any.
    pub wow_process: RwLock<Option<CommandChild>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: RwLock::new(AppConfig::default()),
            wow_process: RwLock::new(None),
        }
    }
}

impl AppState {
    pub fn from_config(config: AppConfig) -> Self {
        Self {
            config: RwLock::new(config),
            wow_process: RwLock::new(None),
        }
    }
}
