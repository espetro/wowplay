//! WowLauncher orchestrator — runner-agnostic WoW launch sequence.
//!
//! Delegates runner-specific work (loader prep, env vars, spawn) to a [`RunnerPort`]
//! while owning the game-setup and session-management logic.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::adapters::errors::LaunchError;
use crate::ports::launcher::WowLauncherPort;
use crate::ports::runner::RunnerPort;

/// Orchestrates a full WoW 3.3.5a session via rosettax87 + a [`RunnerPort`].
///
/// Launch sequence:
/// 1. Apply game patch: stage D9VK, winerosetta (mods/ only), libSiliconPatch, rosettax87
/// 2. Prepare loader via runner (e.g. create `$CX_HOSTED/wineloader2`)
/// 3. Patch DivxDecoder.dll natively via Rust PE patcher (enables winerosetta injection)
/// 4. Ensure rosettax87 background service is running
/// 5. `rosettax87 $LOADER WoW.exe` (from WoW dir)
pub struct WowLauncher {
    runner: Arc<dyn RunnerPort>,
    wowsilicon_resources: PathBuf,
    /// Optional override for rosettax87 binary source directory.
    /// When set, `runtime_loader` and `libRuntimeRosettax87` are copied from here
    /// instead of from `wowsilicon_resources/rosettax87/`.
    rosettax87_bin_dir: Option<PathBuf>,
    bottle: String,
    use_sudo: bool,
    enable_lib_silicon: bool,
}

impl WowLauncher {
    /// Create a launcher with the given runner and resources.
    ///
    /// `enable_lib_silicon` defaults to `true` — set to `false` to skip libSiliconPatch.dll deployment.
    pub fn new(runner: Arc<dyn RunnerPort>, wowsilicon_resources: PathBuf, bottle: &str) -> Self {
        Self {
            runner,
            wowsilicon_resources,
            rosettax87_bin_dir: None,
            bottle: bottle.to_string(),
            use_sudo: false,
            enable_lib_silicon: true,
        }
    }

    /// Override the directory from which rosettax87 binaries are copied.
    ///
    /// When set, `runtime_loader` and `libRuntimeRosettax87` are sourced from this
    /// directory instead of the patching resources dir.
    pub fn with_rosettax87_bin_dir(mut self, dir: PathBuf) -> Self {
        self.rosettax87_bin_dir = Some(dir);
        self
    }

    /// Deprecated: sudo is no longer required; rosettax87 now installs its JIT hook
    /// via fork/ptrace without root. Kept for backward compatibility.
    pub fn with_sudo(mut self) -> Self {
        self.use_sudo = true;
        self
    }

    /// Enable or disable libSiliconPatch.dll deployment.
    ///
    /// Defaults to `true`. Set to `false` when running on Whisky where winerosetta's VEH
    /// handles x87 exception handling without the proprietary library.
    pub fn with_enable_lib_silicon(mut self, enable: bool) -> Self {
        self.enable_lib_silicon = enable;
        self
    }

    /// Launches WoW, optionally tee-ing stdout/stderr to a log file.
    pub fn launch_wow_logged(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
    ) -> Result<WowSession, LaunchError> {
        if !self.runner.is_available() {
            return Err(LaunchError::SetupFailed(format!(
                "{} is not available",
                self.runner.name()
            )));
        }

        Self::apply_game_patch(
            wow_dir,
            &self.wowsilicon_resources,
            self.rosettax87_bin_dir.as_deref(),
            self.enable_lib_silicon,
        )?;
        let loader = self.runner.prepare_loader()?;

        let runtime_loader = wow_dir.join("rosettax87/runtime_loader");
        Self::bootstrap_divx_decoder(wow_dir)?;
        Self::ensure_rosetta_service(&runtime_loader, self.use_sudo)?;

        let wow_exe = Self::find_wow_exe(wow_dir)?;
        let env_vars = self.runner.build_env(&self.bottle);

        let mut cmd = Command::new(&runtime_loader);
        cmd.arg(&loader).arg(&wow_exe).current_dir(wow_dir);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let (child, log_threads) = if let Some(log_path) = log_path {
            let log_file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .map_err(|e| {
                    LaunchError::SetupFailed(format!("open log {}: {e}", log_path.display()))
                })?;
            let log = Arc::new(Mutex::new(log_file));

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(LaunchError::SpawnFailed)?;
            let stdout = child.stdout.take().expect("stdout piped");
            let stderr = child.stderr.take().expect("stderr piped");

            let t1 = Self::tee_to_log(stdout, Arc::clone(&log));
            let t2 = Self::tee_to_log(stderr, log);
            (child, vec![t1, t2])
        } else {
            let child = cmd.spawn().map_err(LaunchError::SpawnFailed)?;
            (child, vec![])
        };

        Ok(WowSession { child, log_threads })
    }

    /// Applies the WoW game patch: copies DLLs, rosettax87 binaries, updates dlls.txt.
    ///
    /// `enable_lib_silicon` controls whether `libSiliconPatch.dll` is copied and registered in dlls.txt.
    /// When `false`, only `winerosetta.dll` is deployed — suitable for Whisky+Crossover where the VEH
    /// handles x87 exception handling without the proprietary patch library.
    pub fn apply_game_patch(
        wow_dir: &Path,
        resources: &Path,
        rosettax87_bin_dir: Option<&Path>,
        enable_lib_silicon: bool,
    ) -> Result<(), LaunchError> {
        if !wow_dir.exists() {
            return Err(LaunchError::WowDirNotFound(wow_dir.display().to_string()));
        }

        let copy = |rel_src: &str, rel_dst: &str| -> Result<(), LaunchError> {
            let src = resources.join(rel_src);
            let dst = wow_dir.join(rel_dst);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    LaunchError::SetupFailed(format!("mkdir {}: {e}", parent.display()))
                })?;
            }
            fs::copy(&src, &dst).map_err(|e| {
                LaunchError::SetupFailed(format!("copy {} → {}: {e}", src.display(), dst.display()))
            })?;
            Ok(())
        };

        // D9VK: DirectX9 → Vulkan → MoltenVK → Metal.
        copy("d9vk/d3d9.dll", "d3d9.dll")?;

        // winerosetta only in mods/; dlls.txt handles loading. No game-root copy.
        copy("winerosetta/winerosetta.dll", "mods/winerosetta.dll")?;
        if enable_lib_silicon {
            copy(
                "libSiliconPatch/wotlk/libSiliconPatch.dll",
                "mods/libSiliconPatch.dll",
            )?;
        }

        let (rosettax87_src, loader_name) = match rosettax87_bin_dir {
            Some(dir) => (dir.to_path_buf(), "runtime_loader"),
            None => (resources.join("rosettax87"), "rosettax87"),
        };
        let copy_rosetta = |src_name: &str, dst_rel: &str| -> Result<(), LaunchError> {
            let src = rosettax87_src.join(src_name);
            let dst = wow_dir.join(dst_rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    LaunchError::SetupFailed(format!("mkdir {}: {e}", parent.display()))
                })?;
            }
            fs::copy(&src, &dst).map_err(|e| {
                LaunchError::SetupFailed(format!("copy {} → {}: {e}", src.display(), dst.display()))
            })?;
            Ok(())
        };
        copy_rosetta(loader_name, "rosettax87/runtime_loader")?;
        copy_rosetta("libRuntimeRosettax87", "rosettax87/libRuntimeRosettax87")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for bin in [
                "rosettax87/runtime_loader",
                "rosettax87/libRuntimeRosettax87",
            ] {
                let p = wow_dir.join(bin);
                fs::set_permissions(&p, fs::Permissions::from_mode(0o755))
                    .map_err(|e| LaunchError::SetupFailed(format!("chmod {}: {e}", p.display())))?;
            }
            // Belt-and-suspenders: signed binaries from the zip/DMG should not need this,
            // but unsigned/ad-hoc builds still get quarantine-blocked without it.
            let rosettax87_dir = wow_dir.join("rosettax87");
            let status = Command::new("xattr")
                .args([
                    "-dr",
                    "com.apple.quarantine",
                    &rosettax87_dir.display().to_string(),
                ])
                .status();
            if status.map(|s| !s.success()).unwrap_or(true) {
                eprintln!(
                    "  \x1b[33m[warn]\x1b[0m quarantine clear on rosettax87 dir had non-zero exit \
                     — if launch fails, run: xattr -dr com.apple.quarantine {:?}",
                    rosettax87_dir
                );
            }
        }

        Self::update_dlls_txt(
            wow_dir,
            if enable_lib_silicon {
                &["mods/winerosetta.dll", "mods/libSiliconPatch.dll"]
            } else {
                &["mods/winerosetta.dll"]
            },
        )?;

        Ok(())
    }

    fn update_dlls_txt(wow_dir: &Path, entries: &[&str]) -> Result<(), LaunchError> {
        let path = wow_dir.join("dlls.txt");
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let mut out = existing.clone();
        for entry in entries {
            if !existing.to_lowercase().contains(&entry.to_lowercase()) {
                out.push_str(entry);
                out.push('\n');
            }
        }
        fs::write(&path, out)
            .map_err(|e| LaunchError::SetupFailed(format!("write dlls.txt: {e}")))?;
        Ok(())
    }

    /// Patches DivxDecoder.dll (and DivxTac.dll if present) to import winerosetta.
    /// Native Rust implementation — no Wine dependency.
    fn bootstrap_divx_decoder(wow_dir: &Path) -> Result<(), LaunchError> {
        use crate::adapters::pe_import_patcher::patch_dll_imports;

        let dlls_to_patch = ["DivxDecoder.dll", "DivxTac.dll"];

        for dll_name in &dlls_to_patch {
            let dll_path = wow_dir.join(dll_name);
            if !dll_path.exists() {
                continue; // Some clients don't have DivxTac.dll
            }

            patch_dll_imports(wow_dir, dll_name, "mods/winerosetta.dll").map_err(|e| {
                LaunchError::SetupFailed(format!("Failed to patch {}: {}", dll_name, e))
            })?;
        }

        Ok(())
    }

    /// Returns true if the rosettax87 JIT service is already running.
    pub fn is_rosetta_service_running() -> bool {
        std::path::Path::new("/var/run/rosetta_helper.sock").exists()
            || Command::new("pgrep")
                .args(["-x", "runtime_loader"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
    }

    fn ensure_rosetta_service(rosettax87: &Path, use_sudo: bool) -> Result<(), LaunchError> {
        if Self::is_rosetta_service_running() {
            return Ok(());
        }
        if use_sudo {
            // Deprecated path: was required before rosettax87 used fork/ptrace.
            eprintln!(
                "  \x1b[33m[warn]\x1b[0m --sudo is deprecated; rosettax87 no longer requires root"
            );
            let sudo_cached = Command::new("sudo")
                .args(["-n", "true"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !sudo_cached {
                eprintln!("  \x1b[34m[info]\x1b[0m Enter your password when prompted.");
                Command::new("sudo")
                    .arg("-v")
                    .status()
                    .map_err(|e| match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            LaunchError::SetupFailed("sudo not found on this system".into())
                        }
                        std::io::ErrorKind::PermissionDenied => {
                            LaunchError::SetupFailed("sudo authentication failed".into())
                        }
                        _ => LaunchError::SpawnFailed(e),
                    })?;
            }
            Command::new("sudo")
                .args(["-n", &rosettax87.display().to_string()])
                .spawn()
                .map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        LaunchError::SetupFailed("sudo not found on this system".into())
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        LaunchError::SetupFailed("sudo authentication failed".into())
                    }
                    _ => LaunchError::SpawnFailed(e),
                })?;
        } else {
            Command::new(rosettax87)
                .spawn()
                .map_err(LaunchError::SpawnFailed)?;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        if !Self::is_rosetta_service_running() {
            return Err(LaunchError::SetupFailed(
                "rosettax87 failed to start — if the binary is quarantined, run: \
                 xattr -dr com.apple.quarantine <wow_dir>/rosettax87"
                    .into(),
            ));
        }
        Ok(())
    }

    fn tee_to_log(
        reader: impl Read + Send + 'static,
        log: Arc<Mutex<fs::File>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut reader = reader;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if let Ok(mut f) = log.lock() {
                            let _ = f.write_all(&buf[..n]);
                        }
                    }
                }
            }
        })
    }

    fn find_wow_exe(wow_dir: &Path) -> Result<PathBuf, LaunchError> {
        ["WoW.exe", "wow.exe", "Wow.exe"]
            .iter()
            .map(|name| wow_dir.join(name))
            .find(|p| p.exists())
            .ok_or_else(|| {
                LaunchError::WowDirNotFound(format!("WoW.exe not found in {}", wow_dir.display()))
            })
    }
}

impl WowLauncherPort for WowLauncher {
    fn launch_wow(&self, wow_dir: &Path) -> Result<Child, LaunchError> {
        let session = self.launch_wow_logged(wow_dir, None)?;
        Ok(session.child)
    }

    fn check_prerequisites(&self) -> Result<(), LaunchError> {
        if !self.runner.is_available() {
            return Err(LaunchError::SetupFailed(format!(
                "{} is not available",
                self.runner.name()
            )));
        }
        let d3d9 = self.wowsilicon_resources.join("d9vk/d3d9.dll");
        if !d3d9.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "d3d9.dll not found at {} — run wowplay setup --patching-dir <patching-dir>",
                d3d9.display()
            )));
        }
        Ok(())
    }
}

/// A running WoW session, with optional background log-tee threads.
pub struct WowSession {
    /// The child process handle.
    pub child: Child,
    log_threads: Vec<std::thread::JoinHandle<()>>,
}

impl WowSession {
    /// Returns the OS PID of the WoW process.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Waits for WoW to exit and joins any log threads.
    pub fn wait(mut self) -> std::io::Result<std::process::ExitStatus> {
        let status = self.child.wait()?;
        for t in self.log_threads {
            let _ = t.join();
        }
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Child;

    struct MockRunner;

    impl RunnerPort for MockRunner {
        fn name(&self) -> &str {
            "mock"
        }
        fn is_available(&self) -> bool {
            true
        }
        fn prepare_loader(&self) -> Result<PathBuf, crate::adapters::errors::LaunchError> {
            Ok(PathBuf::from("/usr/bin/true"))
        }
        fn build_env(&self, _bottle: &str) -> Vec<(String, String)> {
            vec![]
        }
        fn spawn(
            &self,
            _program: &Path,
            _args: &[&str],
            _env: &[(String, String)],
            _cwd: &Path,
        ) -> Result<Child, crate::adapters::errors::LaunchError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_rosettax87_bin_dir_default_is_none() {
        let launcher = WowLauncher::new(
            std::sync::Arc::new(MockRunner),
            PathBuf::from("/tmp/resources"),
            "test",
        );
        assert!(launcher.rosettax87_bin_dir.is_none());
    }

    #[test]
    fn test_rosettax87_bin_dir_override_stores_path() {
        let launcher = WowLauncher::new(
            std::sync::Arc::new(MockRunner),
            PathBuf::from("/tmp/resources"),
            "test",
        )
        .with_rosettax87_bin_dir(PathBuf::from("/tmp/rtx87"));

        assert_eq!(
            launcher.rosettax87_bin_dir,
            Some(PathBuf::from("/tmp/rtx87"))
        );
    }
}
