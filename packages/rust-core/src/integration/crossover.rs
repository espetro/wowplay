//! CrossOver integration: finds CrossOver, stages WoWSilicon assets, launches WoW.
//!
//! Mirrors the patching logic from WoWSilicon v2.5.4:
//! - `d3d9.dll` (D9VK) at game root — WINEDLLOVERRIDES=d3d9=n,b
//! - `mods/winerosetta.dll` loaded via `dlls.txt`
//! - `mods/libSiliconPatch.dll` (WotLK-specific x87 optimizations) via `dlls.txt`
//! - `rosettax87/` arm64 JIT translator
//! - `$CX_HOSTED/wineloader2` — unsigned copy placed alongside Wine siblings

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::adapters::errors::LaunchError;
use crate::ports::launcher::WowLauncherPort;

const CX_HOSTED_REL: &str = "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application";

const WINELOADER_REL: &str =
    "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader";
const WINELOADER64_REL: &str =
    "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wineloader64";

const WOWSILICON_BUNDLE: &str =
    "Contents/Resources/WoWSilicon-swift_WoWSiliconSwift.bundle/Patching";

/// Finds the CrossOver.app bundle on this machine.
pub fn find_crossover() -> Result<PathBuf, LaunchError> {
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

/// Finds the WoWSilicon.app bundle, which supplies D9VK, winerosetta, etc.
pub fn find_wowsilicon() -> Result<PathBuf, LaunchError> {
    let home_candidate = home_dir().map(|h| h.join("Applications/WoWSilicon.app"));
    if let Some(p) = home_candidate {
        if p.exists() {
            return Ok(p);
        }
    }
    let system = PathBuf::from("/Applications/WoWSilicon.app");
    if system.exists() {
        return Ok(system);
    }
    Err(LaunchError::SetupFailed(
        "WoWSilicon.app not found; download from github.com/WoWSilicon/WoWSilicon".into(),
    ))
}

/// Returns the `Patching/` resource directory inside WoWSilicon.app.
pub fn wowsilicon_resources(wowsilicon: &Path) -> PathBuf {
    wowsilicon.join(WOWSILICON_BUNDLE)
}

/// Returns the path to CrossOver's wineloader binary.
///
/// Checks for `wineloader64` first (newer CrossOver), then falls back to `wineloader`.
pub fn wineloader_path(crossover: &Path) -> PathBuf {
    let loader64 = crossover.join(WINELOADER64_REL);
    if loader64.exists() {
        return loader64;
    }
    crossover.join(WINELOADER_REL)
}

/// Returns the expected path for the unsigned wineloader2 copy.
pub fn wineloader2_path(crossover: &Path) -> PathBuf {
    crossover.join(CX_HOSTED_REL).join("wineloader2")
}

/// Creates an unsigned copy of CrossOver's wineloader at `$CX_HOSTED/wineloader2`.
///
/// Placing it alongside wineserver, wineboot, etc. lets Wine resolve its sibling
/// binaries by directory when re-execing child processes.
pub fn create_wineloader2(crossover: &Path) -> Result<PathBuf, LaunchError> {
    let src = wineloader_path(crossover);
    if !src.exists() {
        return Err(LaunchError::CrossoverNotFound(format!(
            "wineloader not found at {}",
            src.display()
        )));
    }

    let dst = wineloader2_path(crossover);
    fs::copy(&src, &dst).map_err(|e| LaunchError::SetupFailed(format!("copy wineloader: {e}")))?;

    Command::new("codesign")
        .args(["--remove-signature", &dst.display().to_string()])
        .status()
        .map_err(|e| LaunchError::CodesignFailed(e.to_string()))?;

    Ok(dst)
}

/// Applies the WoW game patch: copies DLLs, rosettax87 binaries, updates dlls.txt.
///
/// Mirrors WoWSilicon's `applyGamePatch()`.
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

    update_dlls_txt(
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
    fs::write(&path, out).map_err(|e| LaunchError::SetupFailed(format!("write dlls.txt: {e}")))?;
    Ok(())
}

/// Returns the Wine environment for a CrossOver bottle launch.
pub fn wine_env(crossover: &Path, bottle_name: &str) -> Vec<(String, String)> {
    let cx_root = crossover.join("Contents/SharedSupport/CrossOver");
    let cx_hosted = crossover.join(CX_HOSTED_REL);
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

fn ensure_rosetta_service(rosettax87: &Path) -> Result<(), LaunchError> {
    if is_rosetta_service_running() {
        return Ok(());
    }
    Command::new("sudo")
        .arg(rosettax87)
        .spawn()
        .map_err(LaunchError::SpawnFailed)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    if !is_rosetta_service_running() {
        return Err(LaunchError::SetupFailed(
            "rosettax87 failed to start — check sudo permissions".into(),
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

/// A running WoW session, with optional background log-tee threads.
pub struct WowSession {
    child: Child,
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

/// Orchestrates a full WoW 3.3.5a session via rosettax87 + CrossOver.
///
/// Launch sequence:
/// 1. Apply game patch: stage D9VK, winerosetta (mods/ only), libSiliconPatch, rosettax87
/// 2. Create `$CX_HOSTED/wineloader2` (unsigned copy, alongside Wine siblings)
/// 3. Patch DivxDecoder.dll once via libDllLdr (enables winerosetta injection)
/// 4. Ensure rosettax87 background service is running
/// 5. `rosettax87 $CX_HOSTED/wineloader2 WoW.exe` (from WoW dir)
pub struct CrossoverLauncher {
    crossover: PathBuf,
    wowsilicon_resources: PathBuf,
    bottle: String,
}

impl CrossoverLauncher {
    /// Create a launcher. Discovers CrossOver and WoWSilicon automatically; defaults to `Win10` bottle.
    pub fn new() -> Result<Self, LaunchError> {
        Self::with_bottle("Win10")
    }

    /// Create a launcher targeting a specific CrossOver bottle.
    pub fn with_bottle(bottle: &str) -> Result<Self, LaunchError> {
        let crossover = find_crossover()?;
        let wowsilicon = find_wowsilicon()?;
        let wowsilicon_resources = wowsilicon_resources(&wowsilicon);
        Ok(Self {
            crossover,
            wowsilicon_resources,
            bottle: bottle.to_string(),
        })
    }

    /// Launches WoW, optionally tee-ing stdout/stderr to a log file.
    pub fn launch_wow_logged(
        &self,
        wow_dir: &Path,
        log_path: Option<&Path>,
    ) -> Result<WowSession, LaunchError> {
        self.check_prerequisites()?;

        apply_game_patch(wow_dir, &self.wowsilicon_resources)?;
        let wineloader2 = create_wineloader2(&self.crossover)?;

        let rosettax87 = wow_dir.join("rosettax87/rosettax87");
        bootstrap_divx_decoder(wow_dir, &wineloader2)?;
        ensure_rosetta_service(&rosettax87)?;

        let wow_exe = ["WoW.exe", "wow.exe", "Wow.exe"]
            .iter()
            .map(|name| wow_dir.join(name))
            .find(|p| p.exists())
            .ok_or_else(|| {
                LaunchError::WowDirNotFound(format!("WoW.exe not found in {}", wow_dir.display()))
            })?;

        let env_vars = wine_env(&self.crossover, &self.bottle);

        eprintln!(
            "  \x1b[34m[info]\x1b[0m rosettax87:  {}",
            rosettax87.display()
        );
        eprintln!(
            "  \x1b[34m[info]\x1b[0m wineloader2: {}",
            wineloader2.display()
        );
        eprintln!("  \x1b[34m[info]\x1b[0m WoW:         {}", wow_exe.display());
        eprintln!("  \x1b[34m[info]\x1b[0m bottle:      {}", self.bottle);
        eprintln!(
            "  \x1b[34m[info]\x1b[0m log:         {}",
            log_path
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(none)".into())
        );

        let mut cmd = Command::new(&rosettax87);
        cmd.arg(&wineloader2).arg(&wow_exe).current_dir(wow_dir);
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

            let t1 = tee_to_log(stdout, std::io::stdout(), Arc::clone(&log));
            let t2 = tee_to_log(stderr, std::io::stderr(), log);
            (child, vec![t1, t2])
        } else {
            let child = cmd.spawn().map_err(LaunchError::SpawnFailed)?;
            (child, vec![])
        };

        Ok(WowSession { child, log_threads })
    }
}

impl WowLauncherPort for CrossoverLauncher {
    fn check_prerequisites(&self) -> Result<(), LaunchError> {
        let loader = wineloader_path(&self.crossover);
        if !loader.exists() {
            return Err(LaunchError::CrossoverNotFound(loader.display().to_string()));
        }
        let d3d9 = self.wowsilicon_resources.join("d9vk/d3d9.dll");
        if !d3d9.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "d3d9.dll not found at {} — is WoWSilicon.app installed?",
                d3d9.display()
            )));
        }
        Ok(())
    }

    fn launch_wow(&self, wow_dir: &Path) -> Result<Child, LaunchError> {
        let session = self.launch_wow_logged(wow_dir, None)?;
        Ok(session.child)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
