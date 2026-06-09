//! Runner registry — resolves runner names to [`RunnerPort`] implementations.
//!
//! Adding a new runner requires:
//! 1. Implement [`RunnerPort`] for the new adapter
//! 2. Add a branch in [`RunnerRegistry::resolve`]
//! 3. Add to [`RunnerRegistry::available_runners`]

use std::sync::Arc;

use crate::adapters::crossover_adapter::CrossOverAdapter;
use crate::adapters::errors::LaunchError;
use crate::adapters::whisky_adapter::{find_moonshine, WhiskyAdapter};
use crate::ports::runner::RunnerPort;

/// Registry for discovering and instantiating runners.
pub struct RunnerRegistry;

impl RunnerRegistry {
    /// Resolve a runner by name.
    ///
    /// Returns [`LaunchError::SetupFailed`] with "Unknown runner: {name}" if the name
    /// is not recognised.
    pub fn resolve(name: &str) -> Result<Arc<dyn RunnerPort>, LaunchError> {
        match name {
            "crossover" => {
                let bundle = CrossOverAdapter::find_bundle()?;
                Ok(Arc::new(CrossOverAdapter::new(bundle)))
            }
            "whisky" => {
                let bundle = WhiskyAdapter::find_bundle()?;
                Ok(Arc::new(WhiskyAdapter::new(bundle)))
            }
            "moonshine" => {
                let bundle = find_moonshine()?;
                Ok(Arc::new(WhiskyAdapter::new(bundle)))
            }
            _ => Err(LaunchError::SetupFailed(format!("Unknown runner: {name}"))),
        }
    }

    /// The default runner name used when none is specified on the CLI.
    pub fn default_runner() -> &'static str {
        "crossover"
    }

    /// All runner names known to the registry.
    pub fn available_runners() -> Vec<&'static str> {
        vec!["crossover", "whisky", "moonshine"]
    }
}
