//! Runner port — abstraction over any Windows-on-macOS runtime.
//!
//! Enables plugging in alternative Wine runners (CrossOver, Whisky, vanilla Wine, etc.)
//! without changing the orchestration layer.

use std::path::Path;
use std::process::Child;

use crate::adapters::errors::LaunchError;

/// Abstraction over any Windows-on-macOS runtime (CrossOver, Whisky, Wine, etc.)
pub trait RunnerPort: Send + Sync {
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;

    /// Verify the runtime bundle exists and is usable.
    fn is_available(&self) -> bool;

    /// Prepare the Windows loader binary.
    ///
    /// For CrossOver: create unsigned `wineloader2` copy next to `wineloader`.
    fn prepare_loader(&self) -> Result<std::path::PathBuf, LaunchError>;

    /// Build environment variables for the Windows process.
    ///
    /// For CrossOver: CX_ROOT, CX_BOTTLE, WINEPREFIX, WINEDLLOVERRIDES, etc.
    fn build_env(&self, bottle: &str) -> Vec<(String, String)>;

    /// Spawn a process with the given environment and working directory.
    fn spawn(
        &self,
        program: &Path,
        args: &[&str],
        env: &[(String, String)],
        cwd: &Path,
    ) -> Result<Child, LaunchError>;
}
