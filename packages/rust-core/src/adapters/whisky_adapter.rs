//! Whisky adapter — implements [`RunnerPort`] for Whisky.app (legacy Isaac Marovitz version).
//!
//! Whisky is a macOS-native Wine wrapper that uses Wine 7.7 bundled in
//! `~/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/`.
//! It runs 32-bit Windows apps via Wine's WoW64 subsystem through the `wine64` binary.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::adapters::errors::LaunchError;
use crate::ports::runner::RunnerPort;

const WHISKY_APP_PATHS: &[&str] = &["/Applications/Whisky.app", "~/Applications/Whisky.app"];
const MOONSHINE_APP_PATHS: &[&str] = &[
    "/Applications/Moonshine.app",
    "~/Applications/Moonshine.app",
];
const WHISKY_CMD_REL: &str = "Contents/Resources/WhiskyCmd";
/// Candidate Wine bin dirs relative to `~/Library/Application Support/com.isaacmarovitz.Whisky`.
/// Probed in order; first with an actual `wine64` binary wins.
const WINE_BIN_DIRS: &[&str] = &[
    "Libraries/Wine/bin", // primary (current Whisky layout)
    "Wine/bin",           // alternate flat layout seen in some builds
];

/// Whisky-specific [`RunnerPort`] implementation.
pub struct WhiskyAdapter {
    bundle: PathBuf,
    /// Path to the Wine `bin/` directory (e.g. `…/Libraries/Wine/bin`).
    wine_bin: PathBuf,
}

impl WhiskyAdapter {
    /// Create a new adapter from a Whisky.app bundle path.
    pub fn new(bundle: PathBuf) -> Self {
        let support_base = home_dir()
            .map(|h| h.join("Library/Application Support/com.isaacmarovitz.Whisky"))
            .unwrap_or_default();
        let wine_bin = WINE_BIN_DIRS
            .iter()
            .map(|d| support_base.join(d))
            .find(|p| p.join("wine64").exists())
            .unwrap_or_else(|| support_base.join(WINE_BIN_DIRS[0]));
        Self { bundle, wine_bin }
    }

    /// Discover Whisky.app on this machine.
    ///
    /// Checks user-local (`~/Applications`) first, then system-wide (`/Applications`).
    pub fn find_bundle() -> Result<PathBuf, LaunchError> {
        let home = home_dir().unwrap_or_default();

        for path in WHISKY_APP_PATHS {
            let expanded = if let Some(rest) = path.strip_prefix("~/") {
                home.join(rest)
            } else {
                PathBuf::from(path)
            };
            if expanded.exists() {
                return Ok(expanded);
            }
        }

        Err(LaunchError::SetupFailed(
            "Whisky.app not found in ~/Applications or /Applications; install from getwhisky.app or GitHub (Whisky-App/Whisky)".into(),
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
                    let home = home_dir().unwrap_or_default();
                    let expanded = path_str.replace("~", &home.display().to_string());
                    let path = PathBuf::from(expanded);

                    // Verify the path exists; if not, try the correct base path
                    // (Whisky reports "Containers/Whisky" but the actual bundle ID is
                    // "com.isaacmarovitz.Whisky")
                    if path.exists() {
                        return Ok(path);
                    }

                    // Try the correct path: Containers/com.isaacmarovitz.Whisky/Bottles/<uuid>
                    let home = home.display().to_string();
                    let corrected = path_str
                        .replace(
                            "~/Library/Containers/Whisky/",
                            &format!("{}/Library/Containers/com.isaacmarovitz.Whisky/", home),
                        )
                        .replace("~", &home);
                    let corrected_path = PathBuf::from(corrected);
                    if corrected_path.exists() {
                        return Ok(corrected_path);
                    }

                    // Return the original path even if it doesn't exist — wine will auto-create
                    return Ok(path);
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
                "wine64 not found at {} (also checked alternate layouts: {}) — reinstall Whisky",
                wine64.display(),
                WINE_BIN_DIRS.join(", ")
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

/// Discover Moonshine.app on this machine.
///
/// Moonshine is a Whisky fork — same Wine layout, same binary paths.
pub fn find_moonshine() -> Result<PathBuf, LaunchError> {
    let home = home_dir().unwrap_or_default();
    for path in MOONSHINE_APP_PATHS {
        let expanded = if let Some(rest) = path.strip_prefix("~/") {
            home.join(rest)
        } else {
            PathBuf::from(path)
        };
        if expanded.exists() {
            return Ok(expanded);
        }
    }
    Err(LaunchError::SetupFailed(
        "Moonshine.app not found in ~/Applications or /Applications; install from github.com/ybmeng/moonshine".into(),
    ))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_bundle_when_whisky_not_installed() {
        let any_installed = WHISKY_APP_PATHS.iter().any(|p| {
            let expanded = if let Some(rest) = p.strip_prefix("~/") {
                home_dir().unwrap_or_default().join(rest)
            } else {
                PathBuf::from(p)
            };
            expanded.exists()
        });
        if !any_installed {
            let result = WhiskyAdapter::find_bundle();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Whisky.app not found"));
        }
    }

    #[test]
    fn test_find_moonshine_when_not_installed() {
        let any_installed = MOONSHINE_APP_PATHS.iter().any(|p| {
            let expanded = if let Some(rest) = p.strip_prefix("~/") {
                home_dir().unwrap_or_default().join(rest)
            } else {
                PathBuf::from(p)
            };
            expanded.exists()
        });
        if !any_installed {
            let result = find_moonshine();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Moonshine.app not found"));
        }
    }

    #[test]
    fn test_adapter_name() {
        let adapter = WhiskyAdapter::new(PathBuf::from("/Applications/Whisky.app"));
        assert_eq!(adapter.name(), "Whisky");
    }
}
