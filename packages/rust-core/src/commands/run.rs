//! Run command handler.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::adapters::errors::LaunchError;
use crate::adapters::whisky_adapter::WhiskyAdapter;
use crate::config::ConfigStore;
use crate::integration::wow_launcher::WowLauncher;
use crate::ports::runner::RunnerPort;
use crate::resources::resolve_patching_dir;
use crate::runner_registry::RunnerRegistry;

/// Optional overrides for the run command.
pub struct RunOverrides {
    /// Override the configured runner.
    pub runner: Option<String>,
    /// Override the configured WoW directory.
    pub wow_dir: Option<PathBuf>,
    /// Override the configured bottle.
    pub bottle: Option<String>,
    /// Override the patching directory.
    pub patching_dir: Option<PathBuf>,
    /// Explicit path to Whisky.app bundle.
    pub whisky_bundle: Option<PathBuf>,
}

/// Run WoW using stored config with optional overrides.
pub fn run_wow(
    store: &dyn ConfigStore,
    overrides: RunOverrides,
    log_path: Option<&Path>,
    verbose: bool,
) -> Result<WowSession, LaunchError> {
    let mut config = store.load()?;
    config.validate()?;

    if let Some(runner) = overrides.runner {
        config.runner = runner;
    }
    if let Some(wow_dir) = overrides.wow_dir {
        config.wow_dir = wow_dir;
    }
    if let Some(bottle) = overrides.bottle {
        config.bottle = bottle;
    }

    let resources = resolve_patching_dir(overrides.patching_dir)?;
    let runner: Arc<dyn RunnerPort> = match config.runner.as_str() {
        "whisky" => {
            if let Some(bundle) = overrides.whisky_bundle {
                Arc::new(WhiskyAdapter::new(bundle))
            } else {
                RunnerRegistry::resolve(&config.runner)?
            }
        }
        _ => RunnerRegistry::resolve(&config.runner)?,
    };

    let mut launcher = WowLauncher::new(runner, resources, &config.bottle);
    if let Ok(bin_dir) = std::env::var("ROSETTAX87_BIN_DIR") {
        launcher = launcher.with_rosettax87_bin_dir(PathBuf::from(bin_dir));
    }
    launcher = launcher.with_enable_lib_silicon(config.enable_lib_silicon);

    launcher.launch_wow_logged(&config.wow_dir, log_path, verbose)
}

pub use crate::integration::wow_launcher::WowSession;
