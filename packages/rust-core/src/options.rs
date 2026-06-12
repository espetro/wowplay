//! Use-case DTOs — primary port contracts for the hexagonal architecture.
//!
//! These structs are the formal interface between primary adapters (CLI, GUI) and core logic.
//! Any new adapter that calls core creates one of these and passes it to the matching function.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Options for launching WoW via `WowLauncher::from_options`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    /// Path to the WoW 3.3.5a game directory.
    pub wow_dir: PathBuf,
    /// Runner to use (e.g. "crossover", "whisky", "moonshine").
    pub runner: String,
    /// Wine/CrossOver bottle name (e.g. "Win10").
    pub bottle: String,
    /// Explicit patching resources directory; auto-detected when `None`.
    pub patching_dir: Option<PathBuf>,
    /// Override directory for rosettax87 binaries; uses bundled resources when `None`.
    pub rosettax87_bin_dir: Option<PathBuf>,
    /// Enable libSiliconPatch.dll deployment (opt-in; off by default).
    pub enable_lib_silicon: bool,
    /// Explicit Whisky.app bundle path — CLI-only override, ignored by GUI.
    pub whisky_bundle: Option<PathBuf>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            wow_dir: PathBuf::default(),
            runner: "crossover".to_string(),
            bottle: "Win10".to_string(),
            patching_dir: None,
            rosettax87_bin_dir: None,
            enable_lib_silicon: false,
            whisky_bundle: None,
        }
    }
}

/// Options for running the one-time setup via `SetupOrchestrator::run`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetupOptions {
    /// Path to the WoW 3.3.5a game directory.
    pub wow_dir: PathBuf,
    /// Explicit patching resources directory; auto-detected when `None`.
    pub patching_dir: Option<PathBuf>,
    /// Enable libSiliconPatch.dll deployment (opt-in; off by default).
    pub enable_lib_silicon: bool,
}

/// Options for the diagnostics checklist via `run_checklist`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnoseOptions {
    /// WoW directory for DivxDecoder and wineloader2 checks; skips those checks when `None`.
    pub wow_dir: Option<PathBuf>,
    /// Explicit patching resources directory; auto-detected when `None`.
    pub patching_dir: Option<PathBuf>,
}
