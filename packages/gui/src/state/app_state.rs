use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

use serde::{Deserialize, Serialize};

/// Persisted application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct AppConfig {
    /// Selected runner name (e.g. "crossover", "whisky", "moonshine").
    pub runner: Option<String>,
    /// Path to the WoW 3.3.5a game directory.
    pub wow_dir: Option<PathBuf>,
    /// Whether to show info-level alerts in the UI.
    #[serde(default = "default_show_alerts")]
    pub show_alerts: bool,
    /// Wine/CrossOver bottle name (e.g. "Win10").
    #[serde(default)]
    pub bottle: Option<String>,
}

fn default_show_alerts() -> bool {
    true
}

/// Shared mutable state across Tauri commands.
pub struct AppState {
    /// Current application configuration.
    pub config: RwLock<AppConfig>,
    /// PID of the running WoW process, if any.
    pub wow_process: Mutex<Option<u32>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: RwLock::new(AppConfig::default()),
            wow_process: Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn from_config(config: AppConfig) -> Self {
        Self {
            config: RwLock::new(config),
            wow_process: Mutex::new(None),
        }
    }
}
