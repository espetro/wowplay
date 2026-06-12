//! Config command handlers.

use std::path::PathBuf;

use crate::adapters::errors::LaunchError;
use crate::config::ConfigStore;
use crate::runner_registry::RunnerRegistry;

/// Supported config keys for `wowplay config set`.
pub const CONFIG_KEYS: &[&str] = &["runner", "wow_dir", "bottle", "enable_lib_silicon"];

/// List the current configuration values.
pub fn list_config(store: &dyn ConfigStore) -> Result<String, LaunchError> {
    let config = store.load()?;
    Ok(format!(
        "runner = {}\nwow_dir = {}\nbottle = {}\nenable_lib_silicon = {}",
        config.runner,
        config.wow_dir.display(),
        config.bottle,
        config.enable_lib_silicon
    ))
}

/// Set a configuration value by key.
pub fn set_config(store: &dyn ConfigStore, key: &str, value: &str) -> Result<String, LaunchError> {
    let mut config = store.load()?;

    match key {
        "runner" => {
            if !RunnerRegistry::available_runners().contains(&value) {
                return Err(LaunchError::SetupFailed(format!(
                    "unknown runner '{}'; supported: {}",
                    value,
                    RunnerRegistry::available_runners().join(", ")
                )));
            }
            config.runner = value.into();
        }
        "wow_dir" => {
            let path = PathBuf::from(value);
            if !path.exists() {
                return Err(LaunchError::SetupFailed(format!(
                    "wow_dir {} does not exist",
                    path.display()
                )));
            }
            config.wow_dir = path;
        }
        "bottle" => {
            config.bottle = value.into();
        }
        "enable_lib_silicon" => {
            config.enable_lib_silicon = value.parse().map_err(|_| {
                LaunchError::SetupFailed("enable_lib_silicon must be 'true' or 'false'".into())
            })?;
        }
        _ => {
            return Err(LaunchError::SetupFailed(format!(
                "unknown config key '{}'; supported: {}",
                key,
                CONFIG_KEYS.join(", ")
            )));
        }
    }

    store.save(&config)?;
    Ok(format!("{key} set to {value}"))
}
