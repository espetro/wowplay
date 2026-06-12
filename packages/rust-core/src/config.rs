//! Persisted application configuration.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::adapters::errors::LaunchError;

/// User-facing application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// Selected runner: "crossover", "whisky", or "moonshine".
    pub runner: String,
    /// Absolute path to the WoW 3.3.5a client directory.
    pub wow_dir: PathBuf,
    /// Wine/CrossOver bottle name.
    #[serde(default = "default_bottle")]
    pub bottle: String,
    /// Whether to deploy libSiliconPatch.dll.
    #[serde(default)]
    pub enable_lib_silicon: bool,
}

impl AppConfig {
    /// Default configuration before the user has run setup.
    pub fn default_values() -> Self {
        Self {
            runner: "crossover".into(),
            wow_dir: PathBuf::new(),
            bottle: default_bottle(),
            enable_lib_silicon: false,
        }
    }

    /// Validate that the configured paths and runner name are usable.
    pub fn validate(&self) -> Result<(), LaunchError> {
        if self.runner.is_empty() {
            return Err(LaunchError::SetupFailed(
                "runner is not configured — run `wowplay setup`".into(),
            ));
        }
        if self.wow_dir.as_os_str().is_empty() || !self.wow_dir.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "wow_dir {} does not exist — run `wowplay setup`",
                self.wow_dir.display()
            )));
        }
        Ok(())
    }
}

fn default_bottle() -> String {
    "Win10".into()
}

/// Abstraction over where config is stored.
pub trait ConfigStore: Send + Sync {
    /// Load the configuration from the store.
    fn load(&self) -> Result<AppConfig, LaunchError>;
    /// Save the configuration to the store.
    fn save(&self, config: &AppConfig) -> Result<(), LaunchError>;
    /// Returns the path where config is stored.
    fn path(&self) -> &Path;
}

/// Default TOML-backed config store at `~/.local/share/wowplay/config.toml`.
pub struct TomlConfigStore {
    path: PathBuf,
}

impl TomlConfigStore {
    /// Returns the default config path `~/.local/share/wowplay/config.toml`.
    pub fn default_path() -> PathBuf {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".local/share/wowplay/config.toml")
    }

    /// Creates a new store using the default path.
    pub fn new() -> Self {
        Self::from_path(Self::default_path())
    }

    /// Creates a store at an explicit path.
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Default for TomlConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigStore for TomlConfigStore {
    fn load(&self) -> Result<AppConfig, LaunchError> {
        if !self.path.exists() {
            return Ok(AppConfig::default_values());
        }
        let text = std::fs::read_to_string(&self.path).map_err(|e| {
            LaunchError::SetupFailed(format!("read config {}: {e}", self.path.display()))
        })?;
        toml::from_str(&text).map_err(|e| {
            LaunchError::SetupFailed(format!("parse config {}: {e}", self.path.display()))
        })
    }

    fn save(&self, config: &AppConfig) -> Result<(), LaunchError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LaunchError::SetupFailed(format!("mkdir {}: {e}", parent.display()))
            })?;
        }
        let text = toml::to_string_pretty(config)
            .map_err(|e| LaunchError::SetupFailed(format!("serialize config: {e}")))?;
        std::fs::write(&self.path, text).map_err(|e| {
            LaunchError::SetupFailed(format!("write config {}: {e}", self.path.display()))
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_roundtrip_save_load() {
        let tmp = TempDir::new().unwrap();
        let store = TomlConfigStore::from_path(tmp.path().join("config.toml"));
        let config = AppConfig {
            runner: "whisky".into(),
            wow_dir: PathBuf::from("/tmp/wow"),
            bottle: "Win10".into(),
            enable_lib_silicon: true,
        };
        store.save(&config).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn test_validate_rejects_missing_wow_dir() {
        let config = AppConfig {
            runner: "whisky".into(),
            wow_dir: PathBuf::from("/does/not/exist"),
            bottle: "Win10".into(),
            enable_lib_silicon: false,
        };
        assert!(config.validate().is_err());
    }
}
