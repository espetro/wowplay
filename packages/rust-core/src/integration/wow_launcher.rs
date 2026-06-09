//! `WowLauncher` — orchestrates the full rosettax87 + CrossOver launch sequence.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use crate::adapters::crossover_adapter::CrossOverAdapter;
use crate::adapters::errors::LaunchError;
use crate::ports::launcher::WowLauncherPort;

/// A running WoW session.
pub struct WowSession {
    child: Child,
}

impl WowSession {
    /// PID of the WoW process.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Block until the WoW process exits.
    pub fn wait(mut self) -> Result<(), LaunchError> {
        self.child
            .wait()
            .map(|_| ())
            .map_err(LaunchError::SpawnFailed)
    }
}

/// Orchestrates starting WoW via rosettax87_jit + CrossOver.
pub struct WowLauncher {
    adapter: Arc<CrossOverAdapter>,
    resources: PathBuf,
    bottle: String,
    with_sudo: bool,
}

impl WowLauncher {
    /// Create a launcher.
    pub fn new(adapter: Arc<CrossOverAdapter>, resources: PathBuf, bottle: &str) -> Self {
        Self {
            adapter,
            resources,
            bottle: bottle.to_string(),
            with_sudo: false,
        }
    }

    /// Opt in to launching the rosettax87 service with sudo (no longer the default).
    pub fn with_sudo(mut self) -> Self {
        self.with_sudo = true;
        self
    }

    /// Copy DLLs and rosettax87 binaries from `resources` into the WoW game directory.
    pub fn apply_game_patch(wow_dir: &Path, resources: &Path) -> Result<(), LaunchError> {
        if !resources.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "patching resources not found: {}",
                resources.display()
            )));
        }
        for entry in fs::read_dir(resources).map_err(LaunchError::SpawnFailed)? {
            let entry = entry.map_err(LaunchError::SpawnFailed)?;
            let dst = wow_dir.join(entry.file_name());
            if entry.path().is_file() {
                fs::copy(entry.path(), &dst).map_err(|e| {
                    LaunchError::SetupFailed(format!(
                        "copy {} → {}: {e}",
                        entry.path().display(),
                        dst.display()
                    ))
                })?;
            }
        }
        Ok(())
    }

    /// Returns true if the rosettax87 `runtime_loader` service is already running.
    pub fn is_rosetta_service_running() -> bool {
        Command::new("pgrep")
            .arg("-x")
            .arg("runtime_loader")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Launch WoW, optionally writing output to `log_path`.
    pub fn launch_wow_logged(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
    ) -> Result<WowSession, LaunchError> {
        self.spawn_wow(wow_dir, log_path)
    }

    fn spawn_wow(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
    ) -> Result<WowSession, LaunchError> {
        if !wow_dir.exists() {
            return Err(LaunchError::WowDirNotFound(wow_dir.display().to_string()));
        }
        let cx_path = self.adapter.crossover_path();
        let wineloader2 = cx_path
            .join("Contents/SharedSupport/CrossOver/CrossOver-Hosted Application")
            .join("wineloader2");
        if !wineloader2.exists() {
            return Err(LaunchError::SetupFailed(
                "wineloader2 not staged; run `wowplay setup`".into(),
            ));
        }

        let runtime_loader = find_runtime_loader()?;
        let wow_exe = wow_dir.join("WoW.exe");

        let env = self.adapter.build_env(&self.bottle);
        let mut cmd = Command::new(&runtime_loader);
        cmd.arg(&wineloader2)
            .arg(&wow_exe)
            .current_dir(wow_dir)
            .envs(env);

        if let Some(log) = log_path {
            let file = fs::File::create(log).map_err(LaunchError::SpawnFailed)?;
            let stderr_file = file.try_clone().map_err(LaunchError::SpawnFailed)?;
            cmd.stdout(Stdio::from(file))
                .stderr(Stdio::from(stderr_file));
        }

        let child = cmd.spawn().map_err(LaunchError::SpawnFailed)?;
        Ok(WowSession { child })
    }
}

impl WowLauncherPort for WowLauncher {
    fn check_prerequisites(&self) -> Result<(), LaunchError> {
        let cx_path = self.adapter.crossover_path();
        let wineloader2 = cx_path
            .join("Contents/SharedSupport/CrossOver/CrossOver-Hosted Application")
            .join("wineloader2");
        if !wineloader2.exists() {
            return Err(LaunchError::SetupFailed(
                "wineloader2 not staged; run `wowplay setup`".into(),
            ));
        }
        if !self.resources.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "patching resources not found: {}",
                self.resources.display()
            )));
        }
        find_runtime_loader()?;
        Ok(())
    }

    fn launch_wow(&self, wow_dir: &Path) -> Result<Child, LaunchError> {
        let session = self.spawn_wow(wow_dir, None)?;
        Ok(session.child)
    }
}

fn find_runtime_loader() -> Result<PathBuf, LaunchError> {
    let candidates = [
        PathBuf::from("/usr/local/bin/runtime_loader"),
        PathBuf::from("vendor/rosettax87_jit/build/bin/runtime_loader"),
    ];
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    Err(LaunchError::RuntimeNotFound(
        "runtime_loader not found; run scripts/setup.sh".into(),
    ))
}
