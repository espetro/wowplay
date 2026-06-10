use std::path::PathBuf;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

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
}

/// Shared mutable state across Tauri commands.
pub struct AppState {
    /// Current application configuration.
    pub config: RwLock<AppConfig>,
    /// Handle to the spawned WoW process, if any.
    pub wow_process: RwLock<Option<tokio::process::Child>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: RwLock::new(AppConfig::default()),
            wow_process: RwLock::new(None),
        }
    }
}
