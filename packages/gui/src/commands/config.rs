//! Config commands backed by TomlConfigStore (shared with CLI).

use wow_silicon_core::commands::config::{list_config as core_list, set_config as core_set};
use wow_silicon_core::config::{AppConfig, ConfigStore, TomlConfigStore};

use crate::error::CommandError;

/// Gets the current configuration from TomlConfigStore.
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, CommandError> {
    let store = TomlConfigStore::new();
    store.load().map_err(|e| CommandError::from(e.to_string()))
}

/// Sets a single config key (runner, wow_dir, bottle, enable_lib_silicon).
/// Mirrors `wowplay config set <key> <value>`.
#[tauri::command]
pub async fn set_config(key: String, value: String) -> Result<String, CommandError> {
    let store = TomlConfigStore::new();
    core_set(&store, &key, &value).map_err(|e| CommandError::from(e.to_string()))
}

/// Lists the current configuration as a formatted string.
/// Mirrors `wowplay config list`.
#[tauri::command]
pub async fn list_config() -> Result<String, CommandError> {
    let store = TomlConfigStore::new();
    core_list(&store).map_err(|e| CommandError::from(e.to_string()))
}
