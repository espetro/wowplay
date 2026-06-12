//! Application state.

use std::sync::RwLock;
use tauri_plugin_shell::process::CommandChild;

/// Transient state (config is persisted to TomlConfigStore, not held here).
pub struct AppState {
    /// Handle to the spawned wowplay sidecar process, if any.
    pub wow_process: RwLock<Option<CommandChild>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wow_process: RwLock::new(None),
        }
    }
}
