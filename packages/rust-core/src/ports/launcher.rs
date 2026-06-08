//! Launcher port traits — abstractions for rosettax87 and WoW launch orchestration.

use std::path::Path;
use std::process::Child;

use crate::adapters::errors::LaunchError;

/// Port for launching a Windows x86 program through the rosettax87 JIT runtime.
///
/// The runtime hooks Rosetta 2's x87 handlers before exec-ing into the target,
/// giving 4–5× better FP performance for WoW 3.3.5a.
pub trait RosettaLauncherPort: Send + Sync {
    /// Spawn `program` (e.g. wineloader2) via the runtime_loader, forwarding `args`.
    fn launch(&self, program: &Path, args: &[&str]) -> Result<Child, LaunchError>;

    /// Returns true if the runtime_loader binary exists and is executable.
    fn is_available(&self) -> bool;

    /// Path to the underlying runtime_loader binary.
    fn runtime_path(&self) -> &Path;
}

/// Port for orchestrating a full WoW session.
pub trait WowLauncherPort: Send + Sync {
    /// Run the full launch sequence: setup, patch wineloader, exec WoW.
    fn launch_wow(&self, wow_dir: &Path) -> Result<Child, LaunchError>;

    /// Verify prerequisites (CrossOver present, winerosetta.dll staged, etc.).
    fn check_prerequisites(&self) -> Result<(), LaunchError>;
}
