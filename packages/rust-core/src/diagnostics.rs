//! Diagnostics — structured environment checklist for WoW on Apple Silicon.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::adapters::errors::LaunchError;
use crate::adapters::whisky_adapter::{find_moonshine, WhiskyAdapter};
use crate::integration::crossover::{find_crossover, wineloader2_path};
use crate::resources::resolve_patching_dir;
use crate::setup::{RunnerCheck, SetupOrchestrator};

/// Structured report from the diagnostics checklist.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsReport {
    /// CrossOver bundle path if found.
    pub crossover: Option<String>,
    /// Whisky bundle path if found.
    pub whisky: Option<String>,
    /// Moonshine bundle path if found.
    pub moonshine: Option<String>,
    /// Patching resources directory if found.
    pub patching_dir: Option<String>,
    /// Whether the rosettax87 background service is running.
    pub rosetta_running: bool,
    /// Path to wineloader2 if found.
    pub wineloader2: Option<String>,
    /// Whether DivxDecoder.dll has been patched (if wow_dir provided).
    pub divxdecoder_patched: Option<bool>,
    /// Whether DivxDecoder.dll exists (if wow_dir provided).
    pub divxdecoder_found: Option<bool>,
    /// Per-runner availability status.
    pub runners: Vec<RunnerCheck>,
}

/// Runs the full diagnostics checklist and returns a structured report.
///
/// `wow_dir` is optional — when provided, DivxDecoder checks are included.
/// `patching_dir` is optional — when provided, skips auto-detection.
pub fn run_checklist(
    wow_dir: Option<&Path>,
    patching_dir: Option<PathBuf>,
) -> Result<DiagnosticsReport, LaunchError> {
    // CrossOver
    let crossover = match find_crossover() {
        Ok(p) => Some(p.display().to_string()),
        Err(_) => None,
    };

    // Whisky
    let whisky = match WhiskyAdapter::find_bundle() {
        Ok(p) => Some(p.display().to_string()),
        Err(_) => None,
    };

    // Moonshine
    let moonshine = match find_moonshine() {
        Ok(p) => Some(p.display().to_string()),
        Err(_) => None,
    };

    // Patching resources
    let patching_dir_result = match resolve_patching_dir(patching_dir) {
        Ok(res) => {
            if res.exists() {
                Some(res.display().to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    };

    // Rosetta service — removed; rosettax87 is no longer a daemon pre-launch
    let rosetta_running = false;

    // Wineloader2
    let wineloader2 = if let Some(ref cx_path) = crossover {
        let wl2 = wineloader2_path(&PathBuf::from(cx_path));
        if wl2.exists() {
            Some(wl2.display().to_string())
        } else {
            None
        }
    } else {
        None
    };

    // DivxDecoder (only if wow_dir provided)
    let (divxdecoder_patched, divxdecoder_found) = wow_dir
        .map(|dir| {
            let patched = dir.join("DivxDecoder.dll.bak").exists();
            let found = dir.join("DivxDecoder.dll").exists();
            (patched, found)
        })
        .map_or((None, None), |(p, f)| (Some(p), Some(f)));

    // Runner checks
    let runners = SetupOrchestrator::check_all_runners();

    Ok(DiagnosticsReport {
        crossover,
        whisky,
        moonshine,
        patching_dir: patching_dir_result,
        rosetta_running,
        wineloader2,
        divxdecoder_patched,
        divxdecoder_found,
        runners,
    })
}
