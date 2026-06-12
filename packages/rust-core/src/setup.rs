//! Setup orchestrator — one-time interactive setup for WoW on Apple Silicon.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::adapters::errors::LaunchError;
use crate::config::{AppConfig, ConfigStore};
use crate::integration::crossover::{apply_game_patch, create_wineloader2, find_crossover};
use crate::integration::wow_launcher::WowLauncher;
use crate::ports::prompt::{PromptItem, PromptPort};
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
    /// Interactive setup: prompts the user, saves config, applies patches.
    pub fn interactive_setup(
        prompt: &dyn PromptPort,
        store: &dyn ConfigStore,
        patching_dir: Option<PathBuf>,
    ) -> Result<Vec<String>, LaunchError> {
        let existing = store.load().unwrap_or_else(|_| AppConfig::default_values());

        // 1. Select runner
        let runner_items = vec![
            PromptItem::new("CrossOver").with_detail("recommended; requires subscription"),
            PromptItem::new("Whisky").with_detail("free, based on Wine"),
            PromptItem::new("Moonshine").with_detail("Whisky fork"),
        ];
        let default_runner_idx = RunnerRegistry::available_runners()
            .iter()
            .position(|n| n == &existing.runner)
            .unwrap_or(0);
        let runner_idx =
            prompt.select_one("Select a runner", &runner_items, Some(default_runner_idx))?;
        let runner = RunnerRegistry::available_runners()[runner_idx].to_string();

        // 2. Path to WoW
        let wow_dir = prompt.input_path(
            "Path to your WoW 3.3.5a directory",
            true,
            existing
                .wow_dir
                .exists()
                .then_some(existing.wow_dir.as_path()),
        )?;

        // 3. Options
        let option_items = vec![PromptItem::new("Enable libSiliconPatch")
            .with_detail("closed-source patch library; optional")];
        let defaults = vec![existing.enable_lib_silicon];
        let selections = prompt.select_many(
            "Select options (Space to toggle, Enter to confirm)",
            &option_items,
            &defaults,
        )?;
        let enable_lib_silicon = selections.contains(&0);

        let config = AppConfig {
            runner,
            wow_dir: wow_dir.clone(),
            bottle: existing.bottle,
            enable_lib_silicon,
        };
        store.save(&config)?;

        // Apply patches using the selected runner
        let messages = Self::apply(&wow_dir, patching_dir, &config)?;
        Ok(messages)
    }

    /// Low-level apply step. Used by interactive setup and by tests.
    pub fn apply(
        wow_dir: &Path,
        patching_dir: Option<PathBuf>,
        config: &AppConfig,
    ) -> Result<Vec<String>, LaunchError> {
        let mut messages = Vec::new();

        if let Some(staged) = stage_bundled_resources()? {
            messages.push(format!("patching resources staged to {}", staged.display()));
        }

        // wineloader2 only for CrossOver
        if config.runner == "crossover" {
            match find_crossover() {
                Ok(crossover) => {
                    create_wineloader2(&crossover).map_err(|e| {
                        LaunchError::SetupFailed(format!(
                            "failed to create CrossOver wineloader2: {e}. \
                             If CrossOver is installed system-wide, move it to ~/Applications \
                             or run with a different runner."
                        ))
                    })?;
                    messages.push("wineloader2 staged".to_string());
                }
                Err(e) => {
                    messages.push(format!("wineloader2 skipped (CrossOver not found: {e})"));
                }
            }
        } else {
            messages.push(format!(
                "wineloader2 skipped (runner '{}' does not require CrossOver's wineloader2)",
                config.runner
            ));
        }

        let resources = resolve_patching_dir(patching_dir)?;
        apply_game_patch(wow_dir, &resources, config.enable_lib_silicon)?;
        messages.push("game patch applied".to_string());

        WowLauncher::bootstrap_divx_decoder(wow_dir)?;
        messages.push("DivxDecoder.dll patched".to_string());

        Ok(messages)
    }

    /// Runs the full setup sequence (deprecated: use `interactive_setup`).
    ///
    /// 1. Stage bundled patching resources to `~/.local/share/wowplay/patching`
    /// 2. Create wineloader2 (CrossOver unsigned loader copy)
    /// 3. Apply game patch (DLLs, rosettax87 binaries, dlls.txt)
    /// 4. Patch DivxDecoder.dll to import winerosetta
    #[deprecated(since = "0.5.0", note = "use interactive_setup instead")]
    pub fn run(
        wow_dir: &Path,
        patching_dir: Option<PathBuf>,
        enable_lib_silicon: bool,
    ) -> Result<Vec<String>, LaunchError> {
        let config = AppConfig {
            runner: "crossover".into(),
            wow_dir: wow_dir.to_path_buf(),
            bottle: "Win10".into(),
            enable_lib_silicon,
        };
        Self::apply(wow_dir, patching_dir, &config)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_whisky_skips_wineloader2() {
        let tmp = tempfile::tempdir().unwrap();
        let wow_dir = tmp.path();

        let config = AppConfig {
            runner: "whisky".into(),
            wow_dir: wow_dir.to_path_buf(),
            bottle: "Win10".into(),
            enable_lib_silicon: false,
        };

        // This will fail later because patching resources are missing, but we can
        // still observe that no wineloader2 message is produced before the failure.
        let result = SetupOrchestrator::apply(wow_dir, Some(wow_dir.to_path_buf()), &config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(!err.contains("wineloader2"));
    }
}
