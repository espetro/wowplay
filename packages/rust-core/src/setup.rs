//! Setup orchestrator — one-time setup for WoW on Apple Silicon.

use std::path::PathBuf;

use serde::Serialize;

use crate::adapters::errors::LaunchError;
use crate::integration::crossover::{apply_game_patch, create_wineloader2, find_crossover};
use crate::options::SetupOptions;
use crate::resources::{resolve_patching_dir, stage_bundled_resources};
use crate::runner_registry::RunnerRegistry;

/// Result of checking a runner's availability.
#[derive(Debug, Clone, Serialize)]
pub struct RunnerCheck {
    /// Internal runner name (e.g. "crossover", "whisky", "moonshine").
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Whether the runner is installed and available.
    pub available: bool,
    /// Path to the runner bundle, if available.
    pub path: Option<PathBuf>,
}

/// Orchestrates the one-time setup: stage resources, create wineloader2, apply game patch.
pub struct SetupOrchestrator;

impl SetupOrchestrator {
    /// Runs the full setup sequence.
    ///
    /// 1. Stage bundled patching resources to `~/.local/share/wowplay/patching`
    /// 2. Create wineloader2 (CrossOver unsigned loader copy)
    /// 3. Apply game patch (DLLs, rosettax87 binaries, dlls.txt)
    pub fn run(options: &SetupOptions) -> Result<Vec<String>, LaunchError> {
        let mut messages = Vec::new();

        // Stage bundled resources
        if let Some(staged) = stage_bundled_resources()? {
            messages.push(format!("patching resources staged to {}", staged.display()));
        }

        // Create wineloader2
        match find_crossover() {
            Ok(crossover) => {
                create_wineloader2(&crossover)?;
                messages.push("wineloader2 staged".to_string());
            }
            Err(e) => {
                messages.push(format!("wineloader2 skipped (CrossOver not found: {e})"));
            }
        }

        // Apply game patch
        let resources = resolve_patching_dir(options.patching_dir.clone())?;
        apply_game_patch(&options.wow_dir, &resources, options.enable_lib_silicon)?;
        messages.push("game patch applied".to_string());

        Ok(messages)
    }

    /// Checks all known runners and returns their availability status.
    pub fn check_all_runners() -> Vec<RunnerCheck> {
        let mut checks = Vec::new();

        for name in RunnerRegistry::available_runners() {
            let (available, path) = match RunnerRegistry::resolve(name) {
                Ok(adapter) => {
                    let path = adapter
                        .prepare_loader()
                        .ok()
                        .map(|p| p.parent().unwrap_or(&p).to_path_buf());
                    (true, path)
                }
                Err(_) => (false, None),
            };

            let display_name = match name {
                "crossover" => "CrossOver",
                "whisky" => "Whisky",
                "moonshine" => "Moonshine",
                _ => name,
            };

            checks.push(RunnerCheck {
                name: name.to_string(),
                display_name: display_name.to_string(),
                available,
                path,
            });
        }

        checks
    }
}
