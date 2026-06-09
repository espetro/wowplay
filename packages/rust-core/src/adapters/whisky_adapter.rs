//! Whisky adapter — implements [`RunnerPort`] for Whisky.app (legacy Isaac Marovitz version).
//!
//! Whisky is a macOS-native Wine wrapper that uses Wine 7.7 bundled in
//! `~/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/`.
//! It runs 32-bit Windows apps via Wine's WoW64 subsystem through the `wine64` binary.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::adapters::errors::LaunchError;
use crate::ports::runner::RunnerPort;

const WHISKY_APP_PATH: &str = "/Applications/Whisky.app";
const WHISKY_CMD_REL: &str = "Contents/Resources/WhiskyCmd";
const WINE_BIN_DIR: &str = "Libraries/Wine/bin";

/// Whisky-specific [`RunnerPort`] implementation.
pub struct WhiskyAdapter {
    bundle: PathBuf,
    wine_bin: PathBuf,
}

impl WhiskyAdapter {
    /// Create a new adapter from a Whisky.app bundle path.
    pub fn new(bundle: PathBuf) -> Self {
        let wine_bin = home_dir()
            .map(|h| h.join("Library/Application Support/com.isaacmarovitz.Whisky"))
            .unwrap_or_default()
            .join(WINE_BIN_DIR);
        Self { bundle, wine_bin }
    }

    /// Discover Whisky.app on this machine.
    pub fn find_bundle() -> Result<PathBuf, LaunchError> {
        let system = PathBuf::from(WHISKY_APP_PATH);
        if system.exists() {
            return Ok(system);
        }
        Err(LaunchError::SetupFailed(
            "Whisky.app not found; install from getwhisky.app or GitHub (Whisky-App/Whisky)".into(),
        ))
    }

    /// Returns the path to Whisky's `wine64` binary.
    fn wine64_path(&self) -> PathBuf {
        self.wine_bin.join("wine64")
    }

    /// Returns the path to Whisky's `wineserver` binary.
    fn wineserver_path(&self) -> PathBuf {
        self.wine_bin.join("wineserver")
    }

    /// Finds the bottle path by name using `whisky list`.
    fn find_bottle_path(&self, bottle_name: &str) -> Result<PathBuf, LaunchError> {
        let output = Command::new(self.bundle.join(WHISKY_CMD_REL))
            .arg("list")
            .output()
            .map_err(|e| LaunchError::SetupFailed(format!("whisky list failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(bottle_name) {
                // Parse path from table output
                // Format: | Name | Windows Version | Path |
                if let Some(path_start) = line.find("~/Library") {
                    let path_str = &line[path_start..];
                    let path_str = path_str.trim_end_matches('|').trim();
                    let expanded = path_str
                        .replace("~", &home_dir().unwrap_or_default().display().to_string());
                    return Ok(PathBuf::from(expanded));
                }
            }
        }
        Err(LaunchError::SetupFailed(format!(
            "Bottle '{}' not found in Whisky; create it with: whisky create {}",
            bottle_name, bottle_name
        )))
    }
}

impl RunnerPort for WhiskyAdapter {
    fn name(&self) -> &str {
        "Whisky"
    }

    fn is_available(&self) -> bool {
        self.bundle.exists() && self.wine64_path().exists()
    }

    fn prepare_loader(&self) -> Result<PathBuf, LaunchError> {
        let wine64 = self.wine64_path();
        if !wine64.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "wine64 not found at {} — reinstall Whisky",
                wine64.display()
            )));
        }
        Ok(wine64)
    }

    fn build_env(&self, bottle_name: &str) -> Vec<(String, String)> {
        let wine64 = self.wine64_path();
        let wineserver = self.wineserver_path();

        // Get bottle path, fallback to default if lookup fails
        let wineprefix = self.find_bottle_path(bottle_name).unwrap_or_else(|_| {
            home_dir()
                .unwrap_or_default()
                .join("Library/Containers/com.isaacmarovitz.Whisky/Bottles")
                .join(bottle_name)
        });

        vec![
            ("PATH".into(), format!("{}:$PATH", self.wine_bin.display())),
            ("WINE".into(), wine64.display().to_string()),
            ("WINESERVER".into(), wineserver.display().to_string()),
            ("WINELOADER".into(), wine64.display().to_string()),
            ("WINEPREFIX".into(), wineprefix.display().to_string()),
            // d3d9=n,b: load D9VK
            ("WINEDLLOVERRIDES".into(), "d3d9=n,b".into()),
            (
                "WINEDEBUG".into(),
                "warn+all,err+all,+loaddll,+module".into(),
            ),
            ("MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS".into(), "1".into()),
            ("DXVK_ASYNC".into(), "1".into()),
            ("WINEESYNC".into(), "1".into()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_bundle_when_whisky_not_installed() {
        // This test will fail if Whisky is installed, which is expected
        // In CI or clean environments, it validates the error message
        if !Path::new(WHISKY_APP_PATH).exists() {
            let result = WhiskyAdapter::find_bundle();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Whisky.app not found"));
        }
    }

    #[test]
    fn test_adapter_name() {
        let adapter = WhiskyAdapter::new(PathBuf::from(WHISKY_APP_PATH));
        assert_eq!(adapter.name(), "Whisky");
    }
}
