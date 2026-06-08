//! CrossOver integration: finds CrossOver, stages WoWSilicon assets, launches WoW.
//!
//! Mirrors the patching logic from WoWSilicon v2.5.4:
//! - `d3d9.dll` (D9VK) at game root — WINEDLLOVERRIDES=d3d9=n,b
//! - `mods/winerosetta.dll` loaded via `dlls.txt`
//! - `mods/libSiliconPatch.dll` (WotLK-specific x87 optimizations) via `dlls.txt`
//! - `rosettax87/` arm64 JIT translator
//! - `/tmp/cx-bin/wineloader64` — unsigned copy keeping the exact filename

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::adapters::errors::LaunchError;
use crate::ports::launcher::WowLauncherPort;

const CX_HOSTED_REL: &str = "Contents/SharedSupport/CrossOver/CrossOver-Hosted Application";

// CrossOver may ship wineloader (x86_64) or wineloader64 depending on version.
// We check both at runtime.
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
/// Checks for `wineloader64` first (newer CrossOver), then falls back
/// to `wineloader` (older/other versions).
pub fn wineloader_path(crossover: &Path) -> PathBuf {
    let loader64 = crossover.join(WINELOADER64_REL);
    if loader64.exists() {
        return loader64;
    }
    crossover.join(WINELOADER_REL)
}

/// Creates an unsigned copy of CrossOver's wineloader at `/tmp/cx-bin/wineloader64`.
///
/// Wine re-execs itself by searching for the loader by its current filename.
/// We keep the name exactly `wineloader64` in a writable dir for compatibility.
pub fn create_wineloader2(crossover: &Path) -> Result<PathBuf, LaunchError> {
    let src = wineloader_path(crossover);
    if !src.exists() {
        return Err(LaunchError::CrossoverNotFound(format!(
            "wineloader not found at {}",
            src.display()
        )));
    }

    let dir = PathBuf::from("/tmp/cx-bin");
    fs::create_dir_all(&dir)
        .map_err(|e| LaunchError::SetupFailed(format!("mkdir /tmp/cx-bin: {e}")))?;

    let dst = dir.join("wineloader64");
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

    // D9VK: DirectX9 → Vulkan → MoltenVK → Metal. Required for rendering on Apple Silicon.
    copy("d9vk/d3d9.dll", "d3d9.dll")?;

    // winerosetta.dll at game root for WINEDLLOVERRIDES=winerosetta=n,b.
    // Also in mods/ for dlls.txt on newer CrossOver builds.
    copy("winerosetta/winerosetta.dll", "winerosetta.dll")?;
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
            // Remove macOS quarantine flag (copied from a quarantined .app bundle)
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

/// Appends entries to `dlls.txt` (Wine reads this to load extra DLLs at startup).
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

    vec![
        ("CX_ROOT".into(), cx_root.display().to_string()),
        ("CX_BOTTLE".into(), bottle_name.to_string()),
        ("WINEPREFIX".into(), wineprefix.display().to_string()),
        (
            "WINESERVER".into(),
            cx_hosted.join("wineserver").display().to_string(),
        ),
        ("WINELOADER".into(), "/tmp/cx-bin/wineloader64".into()),
        (
            "WINEDLLPATH".into(),
            format!(
                "{}/lib/wine:{}/lib64/wine",
                cx_root.display(),
                cx_root.display()
            ),
        ),
        // d3d9=n,b: load D9VK; winerosetta=n,b: load x87 VEH patcher
        ("WINEDLLOVERRIDES".into(), "d3d9=n,b;winerosetta=n,b".into()),
        (
            "DYLD_LIBRARY_PATH".into(),
            format!("{}:{}", cx_root.join("lib").display(), cx_hosted.display()),
        ),
        (
            "DYLD_FALLBACK_LIBRARY_PATH".into(),
            format!("{}/lib:/usr/lib", cx_root.display()),
        ),
        // D9VK / MoltenVK performance
        ("MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS".into(), "1".into()),
        ("DXVK_ASYNC".into(), "1".into()),
    ]
}

/// Orchestrates a full WoW 3.3.5a session via rosettax87 (from WoWSilicon) + CrossOver.
///
/// Launch sequence:
/// 1. Apply game patch: stage D9VK d3d9.dll, winerosetta, libSiliconPatch, rosettax87
/// 2. Create `/tmp/cx-bin/wineloader64` (unsigned copy, exact filename)
/// 3. Set up CrossOver/Wine + D9VK env vars
/// 4. `rosettax87 /tmp/cx-bin/wineloader64 WoW.exe` (from WoW dir)
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
        self.check_prerequisites()?;

        apply_game_patch(wow_dir, &self.wowsilicon_resources)?;
        let wineloader2 = create_wineloader2(&self.crossover)?;

        let wow_exe = ["WoW.exe", "wow.exe", "Wow.exe"]
            .iter()
            .map(|name| wow_dir.join(name))
            .find(|p| p.exists())
            .ok_or_else(|| {
                LaunchError::WowDirNotFound(format!("WoW.exe not found in {}", wow_dir.display()))
            })?;

        let rosettax87 = wow_dir.join("rosettax87/rosettax87");
        let env_vars = wine_env(&self.crossover, &self.bottle);

        let mut cmd = Command::new(&rosettax87);
        cmd.arg(&wineloader2).arg(&wow_exe).current_dir(wow_dir);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        cmd.spawn().map_err(LaunchError::SpawnFailed)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
