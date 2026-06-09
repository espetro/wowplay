//! Adapter wrapping CrossOver.app — finds the bundle, stages wineloader2, builds Wine env.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::errors::LaunchError;

const CX_SHARED: &str = "Contents/SharedSupport/CrossOver";
const CX_HOSTED: &str = "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application";

/// Adapter that wraps a CrossOver.app bundle.
pub struct CrossOverAdapter {
    crossover: PathBuf,
}

impl CrossOverAdapter {
    /// Create an adapter for the given CrossOver.app bundle path.
    pub fn new(crossover: PathBuf) -> Self {
        Self { crossover }
    }

    /// Search standard locations for CrossOver.app and return its path.
    pub fn find_bundle() -> Result<PathBuf, LaunchError> {
        let candidates = [
            PathBuf::from("/Applications/CrossOver.app"),
            home_dir()
                .map(|h| h.join("Applications/CrossOver.app"))
                .unwrap_or_default(),
        ];
        for p in &candidates {
            if p.exists() {
                return Ok(p.clone());
            }
        }
        Err(LaunchError::CrossoverNotFound(
            "CrossOver.app not found; install from codeweavers.com".into(),
        ))
    }

    /// Path to the CrossOver bundle root.
    pub fn crossover_path(&self) -> &Path {
        &self.crossover
    }

    /// Copy `wineloader` → `wineloader2` (unsigned) so Wine can exec it without SIP issues.
    ///
    /// Strips the ad-hoc codesign so the loader can be patched at runtime.
    pub fn prepare_loader(&self) -> Result<PathBuf, LaunchError> {
        let hosted = self.crossover.join(CX_HOSTED);
        let src = {
            let w64 = hosted.join("wineloader64");
            if w64.exists() {
                w64
            } else {
                hosted.join("wineloader")
            }
        };
        if !src.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "wineloader not found at {}",
                src.display()
            )));
        }
        let dst = hosted.join("wineloader2");
        std::fs::copy(&src, &dst)
            .map_err(|e| LaunchError::SetupFailed(format!("copy wineloader → wineloader2: {e}")))?;
        let status = Command::new("codesign")
            .args(["--remove-signature", dst.to_str().unwrap_or_default()])
            .status()
            .map_err(LaunchError::SpawnFailed)?;
        if !status.success() {
            return Err(LaunchError::CodesignFailed(format!(
                "codesign --remove-signature failed on {}",
                dst.display()
            )));
        }
        Ok(dst)
    }

    /// Build the Wine environment variables for launching inside `bottle_name`.
    pub fn build_env(&self, bottle_name: &str) -> Vec<(String, String)> {
        let bottles_root = home_dir()
            .map(|h| h.join("Library/Application Support/CrossOver/Bottles"))
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let wineprefix = bottles_root.join(bottle_name);

        let lib64 = self.crossover.join(CX_SHARED).join("lib64");
        let wine64 = self.crossover.join(CX_SHARED).join("wine64");
        let dllpath = wine64.join("lib/wine/x86_64-unix");

        let mut env = vec![
            ("WINEPREFIX".into(), wineprefix.display().to_string()),
            ("WINEDLLPATH".into(), dllpath.display().to_string()),
        ];
        if lib64.exists() {
            env.push(("DYLD_LIBRARY_PATH".into(), lib64.display().to_string()));
        }
        // Suppress the standard CrossOver GUI; we own the lifecycle.
        env.push(("CX_BOTTLE".into(), bottle_name.into()));
        env.push(("WINE_LARGE_ADDRESS_AWARE".into(), "1".into()));
        env
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
