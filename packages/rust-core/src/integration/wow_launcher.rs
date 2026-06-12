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
/// Launch sequence (pure execution — no directory mutations):
/// 1. Verify setup is complete (all patched files present)
/// 2. Prepare loader via runner (e.g. create `$CX_HOSTED/wineloader2`)
/// 3. Ensure rosettax87 background service is running
/// 4. `rosettax87 $LOADER WoW.exe` (from WoW dir)
///
/// Directory mutations (`apply_game_patch`, `bootstrap_divx_decoder`) belong in
/// [`SetupOrchestrator::run`](crate::setup::SetupOrchestrator::run).
pub struct WowLauncher {
    runner: Arc<dyn RunnerPort>,
    patching_dir: PathBuf,
    /// Optional override for rosettax87 binary source directory.
    /// When set, `runtime_loader` and `libRuntimeRosettax87` are copied from here
    /// instead of from `patching_dir/rosettax87/`.
    rosettax87_bin_dir: Option<PathBuf>,
    bottle: String,
    use_sudo: bool,
    enable_lib_silicon: bool,
}

impl WowLauncher {
    /// Create a launcher with the given runner and resources.
    ///
    /// `enable_lib_silicon` defaults to `false` — set to `true` via `with_enable_lib_silicon` to
    /// opt in to libSiliconPatch.dll deployment.
    pub fn new(runner: Arc<dyn RunnerPort>, patching_dir: PathBuf, bottle: &str) -> Self {
        Self {
            runner,
            patching_dir,
            rosettax87_bin_dir: None,
            bottle: bottle.to_string(),
            use_sudo: false,
            enable_lib_silicon: false,
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
    /// Defaults to `false` (opt-in). Pass `true` to deploy the closed-source patch library.
    pub fn with_enable_lib_silicon(mut self, enable: bool) -> Self {
        self.enable_lib_silicon = enable;
        self
    }

    /// Launches WoW, optionally tee-ing stdout/stderr to a log file.
    ///
    /// Pure execution: verifies that setup has been completed (all patched files
    /// present) and then spawns the WoW process. Does **not** mutate the WoW
    /// directory — run `wowplay setup` first.
    pub fn launch_wow_logged(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
        verbose: bool,
    ) -> Result<WowSession, LaunchError> {
        if !self.runner.is_available() {
            return Err(LaunchError::SetupFailed(format!(
                "{} is not available",
                self.runner.name()
            )));
        }

        Self::check_setup_complete(wow_dir)?;
        let loader = self.runner.prepare_loader()?;

        let runtime_loader = wow_dir.join("rosettax87/runtime_loader");
        // runtime_loader is a wrapper that forks/exec's the Wine loader; it is not a daemon.

        let wow_exe = Self::find_wow_exe(wow_dir)?;
        let env_vars = self.runner.build_env(&self.bottle);

        if verbose {
            eprintln!("  [debug] runtime_loader: {}", runtime_loader.display());
            eprintln!("  [debug] loader: {}", loader.display());
            eprintln!("  [debug] wow_exe: {}", wow_exe.display());
            eprintln!(
                "  [debug] launch command: {} {} {}",
                runtime_loader.display(),
                loader.display(),
                wow_exe.display()
            );
        }

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
        } else {
            let stale = wow_dir.join("mods/libSiliconPatch.dll");
            if stale.exists() {
                fs::remove_file(&stale).map_err(|e| {
                    LaunchError::SetupFailed(format!("remove stale libSiliconPatch.dll: {e}"))
                })?;
            }
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
            if enable_lib_silicon {
                &[]
            } else {
                &["mods/libSiliconPatch.dll"]
            },
        )?;

        Ok(())
    }

    fn update_dlls_txt(
        wow_dir: &Path,
        to_add: &[&str],
        to_remove: &[&str],
    ) -> Result<(), LaunchError> {
        let path = wow_dir.join("dlls.txt");
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let mut lines: Vec<&str> = existing
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                !to_remove.iter().any(|r| lower == r.to_lowercase())
            })
            .collect();
        for entry in to_add {
            let lower = entry.to_lowercase();
            if !lines.iter().any(|l| l.to_lowercase() == lower) {
                lines.push(entry);
            }
        }
        let mut out = lines.join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        fs::write(&path, out)
            .map_err(|e| LaunchError::SetupFailed(format!("write dlls.txt: {e}")))?;
        Ok(())
    }

    /// Verifies all patched-state files exist in the WoW directory.
    ///
    /// Returns `LaunchError::SetupFailed` with a message telling the user to run
    /// `wowplay setup` when any required file is missing.
    fn check_setup_complete(wow_dir: &Path) -> Result<(), LaunchError> {
        let required = [
            "d3d9.dll",
            "mods/winerosetta.dll",
            "rosettax87/runtime_loader",
            "rosettax87/libRuntimeRosettax87",
        ];

        for rel in &required {
            let path = wow_dir.join(rel);
            if !path.exists() {
                return Err(LaunchError::SetupFailed(format!(
                    "required file {} not found — run `wowplay setup` first",
                    rel
                )));
            }
        }

        if !wow_dir.join("DivxDecoder.dll.bak").exists() {
            return Err(LaunchError::SetupFailed(
                "DivxDecoder.dll has not been patched — run `wowplay setup` first".into(),
            ));
        }

        Ok(())
    }

    /// Patches DivxDecoder.dll (and DivxTac.dll if present) to import winerosetta.
    /// Native Rust implementation — no Wine dependency.
    ///
    /// Idempotent: skips DLLs that already have a `.bak` backup (indicating a previous patch).
    pub fn bootstrap_divx_decoder(wow_dir: &Path) -> Result<(), LaunchError> {
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
        let session = self.launch_wow_logged(wow_dir, None, false)?;
        Ok(session.child)
    }

    fn check_prerequisites(&self) -> Result<(), LaunchError> {
        if !self.runner.is_available() {
            return Err(LaunchError::SetupFailed(format!(
                "{} is not available",
                self.runner.name()
            )));
        }
        let d3d9 = self.patching_dir.join("d9vk/d3d9.dll");
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
    fn test_update_dlls_txt_removes_stale_entry_when_lib_silicon_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let dlls = dir.path().join("dlls.txt");
        fs::write(&dlls, "mods/winerosetta.dll\nmods/libSiliconPatch.dll\n").unwrap();

        WowLauncher::update_dlls_txt(
            dir.path(),
            &["mods/winerosetta.dll"],
            &["mods/libSiliconPatch.dll"],
        )
        .unwrap();

        let contents = fs::read_to_string(&dlls).unwrap();
        assert!(!contents.to_lowercase().contains("libsiliconpatch"));
        assert!(contents.to_lowercase().contains("winerosetta"));
    }

    #[test]
    fn test_update_dlls_txt_keeps_entry_when_lib_silicon_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let dlls = dir.path().join("dlls.txt");
        fs::write(&dlls, "").unwrap();

        WowLauncher::update_dlls_txt(
            dir.path(),
            &["mods/winerosetta.dll", "mods/libSiliconPatch.dll"],
            &[],
        )
        .unwrap();

        let contents = fs::read_to_string(&dlls).unwrap();
        assert!(contents.to_lowercase().contains("libsiliconpatch"));
        assert!(contents.to_lowercase().contains("winerosetta"));
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

    fn write_file(dir: &Path, rel: &str, content: &[u8]) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    fn create_setup_complete_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let wow_dir = dir.path();
        write_file(wow_dir, "d3d9.dll", b"d3d9");
        write_file(wow_dir, "mods/winerosetta.dll", b"wine");
        write_file(wow_dir, "rosettax87/runtime_loader", b"loader");
        write_file(wow_dir, "rosettax87/libRuntimeRosettax87", b"lib");
        write_file(wow_dir, "DivxDecoder.dll.bak", b"original");
        dir
    }

    #[test]
    fn test_check_setup_complete_succeeds_when_all_files_present() {
        let dir = create_setup_complete_dir();
        assert!(WowLauncher::check_setup_complete(dir.path()).is_ok());
    }

    #[test]
    fn test_check_setup_complete_fails_when_d3d9_missing() {
        let dir = tempfile::tempdir().unwrap();
        let wow_dir = dir.path();
        write_file(wow_dir, "mods/winerosetta.dll", b"wine");
        write_file(wow_dir, "rosettax87/runtime_loader", b"loader");
        write_file(wow_dir, "rosettax87/libRuntimeRosettax87", b"lib");
        write_file(wow_dir, "DivxDecoder.dll.bak", b"original");

        let result = WowLauncher::check_setup_complete(wow_dir);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("d3d9.dll"),
            "error should mention d3d9.dll: {msg}"
        );
        assert!(
            msg.contains("wowplay setup"),
            "error should mention wowplay setup: {msg}"
        );
    }

    #[test]
    fn test_check_setup_complete_fails_when_winerosetta_missing() {
        let dir = tempfile::tempdir().unwrap();
        let wow_dir = dir.path();
        write_file(wow_dir, "d3d9.dll", b"d3d9");
        write_file(wow_dir, "rosettax87/runtime_loader", b"loader");
        write_file(wow_dir, "rosettax87/libRuntimeRosettax87", b"lib");
        write_file(wow_dir, "DivxDecoder.dll.bak", b"original");

        let result = WowLauncher::check_setup_complete(wow_dir);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("winerosetta"),
            "error should mention winerosetta: {msg}"
        );
    }

    #[test]
    fn test_check_setup_complete_fails_when_rosettax87_missing() {
        let dir = tempfile::tempdir().unwrap();
        let wow_dir = dir.path();
        write_file(wow_dir, "d3d9.dll", b"d3d9");
        write_file(wow_dir, "mods/winerosetta.dll", b"wine");
        write_file(wow_dir, "DivxDecoder.dll.bak", b"original");

        let result = WowLauncher::check_setup_complete(wow_dir);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("runtime_loader"),
            "error should mention runtime_loader: {msg}"
        );
    }

    #[test]
    fn test_check_setup_complete_fails_when_divxdecoder_not_patched() {
        let dir = tempfile::tempdir().unwrap();
        let wow_dir = dir.path();
        write_file(wow_dir, "d3d9.dll", b"d3d9");
        write_file(wow_dir, "mods/winerosetta.dll", b"wine");
        write_file(wow_dir, "rosettax87/runtime_loader", b"loader");
        write_file(wow_dir, "rosettax87/libRuntimeRosettax87", b"lib");

        let result = WowLauncher::check_setup_complete(wow_dir);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("DivxDecoder"),
            "error should mention DivxDecoder: {msg}"
        );
    }
}
