use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::error::CommandError;
use crate::state::app_state::{AppConfig, AppState};

/// Gets the current application configuration.
#[tauri::command]
#[specta::specta]
pub async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, CommandError> {
    let config = state
        .config
        .read()
        .map_err(|e| CommandError::from(e.to_string()))?;
    Ok(config.clone())
}

/// Sets the application configuration and persists it to disk.
#[tauri::command]
#[specta::specta]
pub async fn set_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
    app_handle: AppHandle,
) -> Result<(), CommandError> {
    let mut state_config = state
        .config
        .write()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *state_config = config.clone();

    let store = app_handle
        .store_builder("config.json")
        .build()
        .map_err(|e| CommandError::from(e.to_string()))?;
    store.set(
        "config",
        serde_json::to_value(&config).map_err(|e| CommandError::from(e.to_string()))?,
    );
    store
        .save()
        .map_err(|e| CommandError::from(e.to_string()))?;
    Ok(())
}

/// Resets the application configuration to defaults and clears the store.
#[tauri::command]
#[specta::specta]
pub async fn reset_config(
    state: tauri::State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), CommandError> {
    let mut config = state
        .config
        .write()
        .map_err(|e| CommandError::from(e.to_string()))?;
    *config = AppConfig::default();

    let store = app_handle
        .store_builder("config.json")
        .build()
        .map_err(|e| CommandError::from(e.to_string()))?;
    store.delete("config");
    store
        .save()
        .map_err(|e| CommandError::from(e.to_string()))?;
    Ok(())
}
