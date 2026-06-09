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
/// 3. Patch DivxDecoder.dll once via libDllLdr (enables winerosetta injection)
/// 4. Ensure rosettax87 background service is running
/// 5. `rosettax87 $LOADER WoW.exe` (from WoW dir)
pub struct WowLauncher {
    runner: Arc<dyn RunnerPort>,
    wowsilicon_resources: PathBuf,
    bottle: String,
    use_sudo: bool,
}

impl WowLauncher {
    /// Create a launcher with the given runner and resources.
    pub fn new(runner: Arc<dyn RunnerPort>, wowsilicon_resources: PathBuf, bottle: &str) -> Self {
        Self {
            runner,
            wowsilicon_resources,
            bottle: bottle.to_string(),
            use_sudo: false,
        }
    }

    /// Deprecated: sudo is no longer required; rosettax87 now installs its JIT hook
    /// via fork/ptrace without root. Kept for backward compatibility.
    pub fn with_sudo(mut self) -> Self {
        self.use_sudo = true;
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

        Self::apply_game_patch(wow_dir, &self.wowsilicon_resources)?;
        let loader = self.runner.prepare_loader()?;

        let rosettax87 = wow_dir.join("rosettax87/rosettax87");
        Self::bootstrap_divx_decoder(wow_dir, &loader)?;
        Self::ensure_rosetta_service(&rosettax87, self.use_sudo)?;

        let wow_exe = Self::find_wow_exe(wow_dir)?;
        let env_vars = self.runner.build_env(&self.bottle);

        eprintln!(
            "  \x1b[34m[info]\x1b[0m rosettax87:  {}",
            rosettax87.display()
        );
        eprintln!("  \x1b[34m[info]\x1b[0m wineloader2: {}", loader.display());
        eprintln!("  \x1b[34m[info]\x1b[0m WoW:         {}", wow_exe.display());
        eprintln!("  \x1b[34m[info]\x1b[0m bottle:      {}", self.bottle);
        eprintln!(
            "  \x1b[34m[info]\x1b[0m log:         {}",
            log_path
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(none)".into())
        );

        let mut cmd = Command::new(&rosettax87);
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

            let t1 = Self::tee_to_log(stdout, std::io::stdout(), Arc::clone(&log));
            let t2 = Self::tee_to_log(stderr, std::io::stderr(), log);
            (child, vec![t1, t2])
        } else {
            let child = cmd.spawn().map_err(LaunchError::SpawnFailed)?;
            (child, vec![])
        };

        Ok(WowSession { child, log_threads })
    }

    /// Applies the WoW game patch: copies DLLs, rosettax87 binaries, updates dlls.txt.
    pub fn apply_game_patch(wow_dir: &Path, resources: &Path) -> Result<(), LaunchError> {
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
        copy(
            "libSiliconPatch/wotlk/libSiliconPatch.dll",
            "mods/libSiliconPatch.dll",
        )?;
        copy("winerosetta/libDllLdr.dll", "libDllLdr.dll")?;

        // rosettax87 JIT translator — arm64 binary that hooks Rosetta 2 for x87 FPU
        copy("rosettax87/rosettax87", "rosettax87/rosettax87")?;
        copy(
            "rosettax87/libRuntimeRosettax87",
            "rosettax87/libRuntimeRosettax87",
        )?;
        for bin in ["rosettax87/rosettax87", "rosettax87/libRuntimeRosettax87"] {
            let p = wow_dir.join(bin);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&p, fs::Permissions::from_mode(0o755))
                    .map_err(|e| LaunchError::SetupFailed(format!("chmod {}: {e}", p.display())))?;
                Command::new("xattr")
                    .args(["-d", "com.apple.quarantine", &p.display().to_string()])
                    .status()
                    .ok();
            }
        }

        Self::update_dlls_txt(
            wow_dir,
            &["mods/winerosetta.dll", "mods/libSiliconPatch.dll"],
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

    /// Patches DivxDecoder.dll once via libDllLdr — required for winerosetta to inject.
    fn bootstrap_divx_decoder(wow_dir: &Path, wineloader2: &Path) -> Result<(), LaunchError> {
        if wow_dir.join("DivxDecoder.dll.bak").exists() {
            return Ok(());
        }
        if !wow_dir.join("DivxDecoder.dll").exists() {
            return Err(LaunchError::SetupFailed(
                "DivxDecoder.dll not found — reinstall client".into(),
            ));
        }
        Command::new(wineloader2)
            .args(["rundll32", "libDllLdr.dll,PatchDivxDecoder"])
            .arg(wow_dir)
            .current_dir(wow_dir)
            .env(
                "WINEDLLOVERRIDES",
                "winemenubuilder.exe=d;mscoree=d;mshtml=d",
            )
            .env("WINEDEBUG", "-all")
            .status()
            .map_err(LaunchError::SpawnFailed)?;
        Ok(())
    }

    /// Returns true if the rosettax87 JIT service is already running.
    pub fn is_rosetta_service_running() -> bool {
        std::path::Path::new("/var/run/rosetta_helper.sock").exists()
            || Command::new("pgrep")
                .args(["-x", "rosettax87"])
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
                "rosettax87 failed to start — check that the binary is not quarantined".into(),
            ));
        }
        Ok(())
    }

    fn tee_to_log(
        reader: impl Read + Send + 'static,
        mut terminal: impl Write + Send + 'static,
        log: Arc<Mutex<fs::File>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut reader = reader;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = terminal.write_all(&buf[..n]);
                        let _ = terminal.flush();
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
