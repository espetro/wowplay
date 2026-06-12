//! CrossOver integration — backward-compatible wrappers around [`WowLauncher`] + [`CrossOverAdapter`].
//!
//! All free functions are preserved as thin wrappers so existing CLI and tests continue
//! to compile without changes.

use std::path::{Path, PathBuf};
use std::process::Child;

use crate::adapters::crossover_adapter::CrossOverAdapter;
use crate::adapters::errors::LaunchError;
use crate::integration::wow_launcher::{WowLauncher, WowSession};
use crate::ports::launcher::WowLauncherPort;
use crate::ports::runner::RunnerPort;

/// Finds the CrossOver.app bundle on this machine.
///
/// Thin wrapper around [`CrossOverAdapter::find_bundle`].
pub fn find_crossover() -> Result<PathBuf, LaunchError> {
    CrossOverAdapter::find_bundle()
}

/// Returns the path to CrossOver's wineloader binary.
///
/// Checks for `wineloader64` first (newer CrossOver), then falls back to `wineloader`.
pub fn wineloader_path(crossover: &Path) -> PathBuf {
    let loader64 = crossover
        .join("Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader64");
    if loader64.exists() {
        return loader64;
    }
    crossover.join("Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader")
}

/// Returns the expected path for the unsigned wineloader2 copy.
pub fn wineloader2_path(crossover: &Path) -> PathBuf {
    crossover
        .join("Contents/SharedSupport/CrossOver/CrossOver-Hosted Application")
        .join("wineloader2")
}

/// Creates an unsigned copy of CrossOver's wineloader at `$CX_HOSTED/wineloader2`.
///
/// Thin wrapper around [`CrossOverAdapter::prepare_loader`].
pub fn create_wineloader2(crossover: &Path) -> Result<PathBuf, LaunchError> {
    let adapter = CrossOverAdapter::new(crossover.to_path_buf());
    adapter.prepare_loader()
}

/// Applies the WoW game patch: copies DLLs, rosettax87 binaries, updates dlls.txt.
///
/// Thin wrapper around [`WowLauncher::apply_game_patch`].
/// When `enable_lib_silicon` is `false`, libSiliconPatch.dll is not copied or registered.
pub fn apply_game_patch(
    wow_dir: &Path,
    resources: &Path,
    enable_lib_silicon: bool,
) -> Result<(), LaunchError> {
    WowLauncher::apply_game_patch(wow_dir, resources, None, enable_lib_silicon)
}

/// Returns the Wine environment for a CrossOver bottle launch.
///
/// Thin wrapper around [`CrossOverAdapter::build_env`].
pub fn wine_env(crossover: &Path, bottle_name: &str) -> Vec<(String, String)> {
    let adapter = CrossOverAdapter::new(crossover.to_path_buf());
    adapter.build_env(bottle_name)
}

/// Orchestrates a full WoW 3.3.5a session via rosettax87 + CrossOver.
///
/// This struct is now a thin wrapper around [`WowLauncher`] that hardcodes the
/// CrossOver adapter. All constructors and methods keep their original signatures
/// for backward compatibility.
pub struct CrossoverLauncher {
    inner: WowLauncher,
}

impl CrossoverLauncher {
    /// Create a launcher. Discovers CrossOver and WoWSilicon automatically; defaults to `Win10` bottle.
    pub fn new() -> Result<Self, LaunchError> {
        Self::with_bottle("Win10")
    }

    /// Create a launcher targeting a specific CrossOver bottle.
    pub fn with_bottle(bottle: &str) -> Result<Self, LaunchError> {
        let crossover = find_crossover()?;
        let resources = crate::resources::resolve_patching_dir(None)?;
        let adapter = CrossOverAdapter::new(crossover);
        let mut inner = WowLauncher::new(std::sync::Arc::new(adapter), resources, bottle);
        if let Ok(bin_dir) = std::env::var("ROSETTAX87_BIN_DIR") {
            inner = inner.with_rosettax87_bin_dir(PathBuf::from(bin_dir));
        }
        Ok(Self { inner })
    }

    /// Create a launcher using an explicit patching directory (no WoWSilicon.app needed).
    pub fn from_patching_dir(bottle: &str, patching_dir: PathBuf) -> Result<Self, LaunchError> {
        let crossover = find_crossover()?;
        let adapter = CrossOverAdapter::new(crossover);
        let mut inner = WowLauncher::new(std::sync::Arc::new(adapter), patching_dir, bottle);
        if let Ok(bin_dir) = std::env::var("ROSETTAX87_BIN_DIR") {
            inner = inner.with_rosettax87_bin_dir(PathBuf::from(bin_dir));
        }
        Ok(Self { inner })
    }

    /// Launches WoW, optionally tee-ing stdout/stderr to a log file.
    pub fn launch_wow_logged(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
        verbose: bool,
    ) -> Result<WowSession, LaunchError> {
        self.inner.launch_wow_logged(wow_dir, log_path, verbose)
    }
}

impl WowLauncherPort for CrossoverLauncher {
    fn check_prerequisites(&self) -> Result<(), LaunchError> {
        self.inner.check_prerequisites()
    }

    fn launch_wow(&self, wow_dir: &Path) -> Result<Child, LaunchError> {
        self.inner.launch_wow(wow_dir)
    }
}
