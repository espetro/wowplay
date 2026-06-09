use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use wow_silicon_core::integration::crossover::{
    apply_game_patch, create_wineloader2, find_crossover, find_wowsilicon,
    is_rosetta_service_running, wineloader2_path, wowsilicon_resources, CrossoverLauncher,
};

#[derive(Parser)]
#[command(name = "wowplay", about = "Run WoW 3.3.5a on Apple Silicon")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Launch WoW via rosettax87 + CrossOver
    Run {
        /// Path to WoW 3.3.5a game directory
        #[arg(long)]
        wow_dir: Option<PathBuf>,
        /// CrossOver bottle name (default: Win10)
        #[arg(long, default_value = "Win10")]
        bottle: String,
        /// Path to a WoWSilicon Patching directory (skips app-bundle detection)
        #[arg(long)]
        patching_dir: Option<PathBuf>,
        /// Skip sudo for rosettax87 — manage the service manually
        #[arg(long)]
        no_sudo: bool,
        /// Print diagnostics then exit without launching
        #[arg(long)]
        diagnose: bool,
        /// Skip log file creation; raw stderr only
        #[arg(long)]
        no_log: bool,
    },
    /// One-time setup: stage DLLs and create wineloader2
    Setup {
        /// Path to WoW 3.3.5a game directory
        #[arg(long)]
        wow_dir: PathBuf,
        /// Path to a WoWSilicon Patching directory (skips app-bundle detection).
        /// Use vendor/wowsilicon/Sources/WoWSiliconSwift/Resources/Patching from the repo.
        #[arg(long)]
        patching_dir: Option<PathBuf>,
    },
    /// Print environment checklist and exit
    Diagnose {
        /// WoW directory for DivxDecoder and wineloader2 checks
        #[arg(long)]
        wow_dir: Option<PathBuf>,
        /// Path to a WoWSilicon Patching directory (skips app-bundle detection)
        #[arg(long)]
        patching_dir: Option<PathBuf>,
    },
}

fn info(msg: &str) {
    eprintln!("  \x1b[34m[info]\x1b[0m {msg}");
}

fn ok(msg: &str) {
    eprintln!("  \x1b[32m[ ok ]\x1b[0m {msg}");
}

fn warn(msg: &str) {
    eprintln!("  \x1b[33m[warn]\x1b[0m {msg}");
}

fn die(msg: &str) -> ! {
    eprintln!("  \x1b[31m[fail]\x1b[0m {msg}");
    process::exit(1);
}

fn resolve_patching_dir(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    match explicit {
        Some(p) => Ok(p),
        None => find_wowsilicon()
            .map(|app| wowsilicon_resources(&app))
            .map_err(|e| e.to_string()),
    }
}

fn run_diagnose(wow_dir: Option<&PathBuf>, patching_dir: Option<&PathBuf>) {
    info("Checking CrossOver…");
    let cx_opt = match find_crossover() {
        Ok(p) => {
            ok(&format!("CrossOver: {}", p.display()));
            Some(p)
        }
        Err(e) => {
            warn(&format!("CrossOver: {e}"));
            None
        }
    };

    info("Checking patching resources…");
    match resolve_patching_dir(patching_dir.cloned()) {
        Ok(res) => {
            if res.exists() {
                ok(&format!("Patching dir: {}", res.display()));
            } else {
                warn(&format!("Patching dir not found: {}", res.display()));
            }
        }
        Err(e) => warn(&format!(
            "Patching dir: {e} — pass --patching-dir or install WoWSilicon.app"
        )),
    }

    info("Checking rosettax87 service…");
    if is_rosetta_service_running() {
        ok("rosettax87 service running");
    } else {
        warn("rosettax87 not running — will start on launch");
    }

    if let Some(ref cx) = cx_opt {
        let wl2 = wineloader2_path(cx);
        info("Checking wineloader2…");
        if wl2.exists() {
            ok(&format!("wineloader2: {}", wl2.display()));
        } else {
            warn("wineloader2 not staged — run `wowplay setup --wow-dir <dir>`");
        }
    }

    if let Some(dir) = wow_dir {
        info("Checking DivxDecoder…");
        if dir.join("DivxDecoder.dll.bak").exists() {
            ok("DivxDecoder.dll patched");
        } else if dir.join("DivxDecoder.dll").exists() {
            warn("DivxDecoder.dll not yet patched — will patch on first launch");
        } else {
            warn("DivxDecoder.dll not found — reinstall WoW client if launch fails");
        }
    }
}

fn make_log_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let log_dir = PathBuf::from(home).join(".local/share/wowplay/logs");
    fs::create_dir_all(&log_dir).map_err(|e| format!("mkdir logs: {e}"))?;
    prune_old_logs(&log_dir);
    let ts = timestamp_now();
    Ok(log_dir.join(format!("{ts}.log")))
}

fn prune_old_logs(log_dir: &std::path::Path) {
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(7 * 24 * 60 * 60))
        .unwrap_or(std::time::UNIX_EPOCH);
    let Ok(entries) = fs::read_dir(log_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.modified().map(|t| t < cutoff).unwrap_or(false) {
            let _ = fs::remove_file(&path);
        }
    }
}

fn timestamp_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ss = secs % 60;
    let mins = secs / 60;
    let mm = mins % 60;
    let hours = mins / 60;
    let hh = hours % 24;
    let (year, month, day) = days_to_ymd((hours / 24) as u32);
    format!("{year:04}{month:02}{day:02}T{hh:02}{mm:02}{ss:02}")
}

// Hinnant civil_from_days — converts Unix day count to (year, month, day).
fn days_to_ymd(days: u32) -> (u32, u32, u32) {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m, d)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Diagnose { wow_dir, patching_dir } => {
            run_diagnose(wow_dir.as_ref(), patching_dir.as_ref());
        }

        Cmd::Setup { wow_dir, patching_dir } => {
            info("Setting up wineloader2…");
            let crossover = find_crossover().unwrap_or_else(|e| die(&e.to_string()));
            create_wineloader2(&crossover).unwrap_or_else(|e| die(&e.to_string()));
            ok("wineloader2 staged");

            info("Applying game patch…");
            let resources = match patching_dir {
                Some(p) => p,
                None => {
                    let wowsilicon = find_wowsilicon().unwrap_or_else(|e| die(&e.to_string()));
                    wowsilicon_resources(&wowsilicon)
                }
            };
            apply_game_patch(&wow_dir, &resources).unwrap_or_else(|e| die(&e.to_string()));
            ok("game patch applied");
        }

        Cmd::Run {
            wow_dir,
            bottle,
            patching_dir,
            no_sudo,
            diagnose,
            no_log,
        } => {
            if diagnose {
                run_diagnose(wow_dir.as_ref(), patching_dir.as_ref());
                return;
            }

            let resources =
                resolve_patching_dir(patching_dir).unwrap_or_else(|e| die(&e.to_string()));
            let mut launcher = CrossoverLauncher::from_patching_dir(&bottle, resources)
                .unwrap_or_else(|e| die(&e.to_string()));
            if no_sudo {
                launcher = launcher.no_sudo();
            }

            let wow_dir = wow_dir.unwrap_or_else(|| {
                die("--wow-dir is required; e.g. wowplay run --wow-dir ~/WoW");
            });

            let log_path = if no_log {
                None
            } else {
                match make_log_path() {
                    Ok(p) => Some(p),
                    Err(e) => {
                        warn(&format!("could not create log file: {e}"));
                        None
                    }
                }
            };

            info(&format!("Launching WoW from {}…", wow_dir.display()));
            let session = launcher
                .launch_wow_logged(&wow_dir, log_path.as_deref())
                .unwrap_or_else(|e| die(&e.to_string()));

            ok(&format!("WoW started (pid {})", session.pid()));
            session.wait().unwrap_or_else(|e| die(&e.to_string()));

            if let Some(ref p) = log_path {
                info(&format!("log: {}", p.display()));
            }
        }
    }
}
