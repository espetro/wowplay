//! CrossOver adapter — implements [`RunnerPort`] for CrossOver.app.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::adapters::errors::LaunchError;
use crate::ports::runner::RunnerPort;

const CX_HOSTED_REL: &str = "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application";

const WINELOADER_REL: &str =
    "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader";
const WINELOADER64_REL: &str =
    "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader64";

/// CrossOver-specific [`RunnerPort`] implementation.
pub struct CrossOverAdapter {
    bundle: PathBuf,
}

impl CrossOverAdapter {
    /// Create a new adapter from a CrossOver.app bundle path.
    pub fn new(bundle: PathBuf) -> Self {
        Self { bundle }
    }

    /// Discover CrossOver.app on this machine.
    pub fn find_bundle() -> Result<PathBuf, LaunchError> {
        let home_candidate = home_dir().map(|h| h.join("Applications/CrossOver.app"));
        if let Some(p) = home_candidate {
            if p.exists() {
                return Ok(p);
            }
        }
        let system = PathBuf::from("/Applications/CrossOver.app");
        if system.exists() {
            return Ok(system);
        }
        Err(LaunchError::CrossoverNotFound(
            "CrossOver.app not found; install from codeweavers.com".into(),
        ))
    }

    /// Returns the path to CrossOver's wineloader binary.
    ///
    /// Checks for `wineloader64` first (newer CrossOver), then falls back to `wineloader`.
    fn wineloader_path(&self) -> PathBuf {
        let loader64 = self.bundle.join(WINELOADER64_REL);
        if loader64.exists() {
            return loader64;
        }
        self.bundle.join(WINELOADER_REL)
    }

    /// Returns the expected path for the unsigned wineloader2 copy.
    fn wineloader2_path(&self) -> PathBuf {
        self.bundle.join(CX_HOSTED_REL).join("wineloader2")
    }
}

impl RunnerPort for CrossOverAdapter {
    fn name(&self) -> &str {
        "CrossOver"
    }

    fn is_available(&self) -> bool {
        self.bundle.exists() && self.wineloader_path().exists()
    }

    fn prepare_loader(&self) -> Result<PathBuf, LaunchError> {
        let src = self.wineloader_path();
        if !src.exists() {
            return Err(LaunchError::CrossoverNotFound(format!(
                "wineloader not found at {}",
                src.display()
            )));
        }

        let dst = self.wineloader2_path();
        fs::copy(&src, &dst).map_err(|e| LaunchError::SetupFailed(format!("copy wineloader: {e}")))?;

        Command::new("codesign")
            .args(["--remove-signature", &dst.display().to_string()])
            .status()
            .map_err(|e| LaunchError::CodesignFailed(e.to_string()))?;

        Ok(dst)
    }

    fn build_env(&self, bottle_name: &str) -> Vec<(String, String)> {
        let cx_root = self.bundle.join("Contents/SharedSupport/CrossOver");
        let cx_hosted = self.bundle.join(CX_HOSTED_REL);
        let wineprefix = home_dir().unwrap_or_default().join(format!(
            "Library/Application Support/CrossOver/Bottles/{bottle_name}"
        ));
        let wineloader2 = cx_hosted.join("wineloader2");

        vec![
            ("CX_ROOT".into(), cx_root.display().to_string()),
            ("CX_BOTTLE".into(), bottle_name.to_string()),
            ("WINEPREFIX".into(), wineprefix.display().to_string()),
            (
                "WINESERVER".into(),
                cx_hosted.join("wineserver").display().to_string(),
            ),
            ("WINELOADER".into(), wineloader2.display().to_string()),
            // d3d9=n,b: load D9VK. winerosetta loads via dlls.txt — no override needed.
            ("WINEDLLOVERRIDES".into(), "d3d9=n,b".into()),
            (
                "WINEDEBUG".into(),
                "warn+all,err+all,+loaddll,+module".into(),
            ),
            ("MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS".into(), "1".into()),
            ("DXVK_ASYNC".into(), "1".into()),
        ]
    }

    fn spawn(
        &self,
        program: &Path,
        args: &[&str],
        env: &[(String, String)],
        cwd: &Path,
    ) -> Result<Child, LaunchError> {
        let mut cmd = Command::new(program);
        cmd.args(args).current_dir(cwd);
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.spawn().map_err(LaunchError::SpawnFailed)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
