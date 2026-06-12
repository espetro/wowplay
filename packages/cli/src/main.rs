use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use wow_silicon_core::config::TomlConfigStore;
use wow_silicon_core::diagnostics::run_checklist;
use wow_silicon_core::reset::ResetOrchestrator;
use wow_silicon_core::setup::SetupOrchestrator;
use wow_silicon_core::commands::config::{list_config, set_config};
use wow_silicon_core::commands::run::{run_wow, RunOverrides};

mod prompt_adapter;
use prompt_adapter::{DialoguerPrompt, HeadlessPrompt};

#[derive(Parser)]
#[command(name = "wowplay", about = "Run WoW 3.3.5a on Apple Silicon")]
struct Cli {
    #[arg(long, short, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Launch WoW via rosettax87 + CrossOver (applies patches automatically on first launch)
    Run {
        /// Override configured runner
        #[arg(long)]
        runner: Option<String>,
        /// Override configured WoW directory
        #[arg(long)]
        wow_dir: Option<PathBuf>,
        /// Override configured bottle
        #[arg(long)]
        bottle: Option<String>,
        /// Path to the patching resources directory (staged, bundled, or explicit override)
        #[arg(long)]
        patching_dir: Option<PathBuf>,
        /// Deprecated: sudo is no longer required
        #[arg(long)]
        sudo: bool,
        /// Print diagnostics then exit without launching
        #[arg(long)]
        diagnose: bool,
        /// Skip log file creation; raw stderr only
        #[arg(long)]
        no_log: bool,
        /// Explicit path to Whisky.app (only used when overriding runner to whisky)
        #[arg(long)]
        whisky_bundle: Option<PathBuf>,
    },
    /// One-time setup: stage DLLs and create wineloader2
    Setup {
        /// Path to the patching resources directory (staged, bundled, or explicit override)
        #[arg(long)]
        patching_dir: Option<PathBuf>,
        /// Runner to use without prompting (crossover, whisky, moonshine)
        #[arg(long)]
        runner: Option<String>,
        /// Path to WoW 3.3.5a directory without prompting
        #[arg(long)]
        wow_dir: Option<PathBuf>,
        /// Enable libSiliconPatch without prompting
        #[arg(long)]
        enable_lib_silicon: bool,
    },
    /// Remove all wowplay patches and staged files (uninstall-like cleanup)
    Reset {
        /// Path to WoW 3.3.5a game directory
        #[arg(long)]
        wow_dir: PathBuf,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Print environment checklist and exit
    Diagnose {
        /// WoW directory for DivxDecoder and wineloader2 checks
        #[arg(long)]
        wow_dir: Option<PathBuf>,
        /// Path to the patching resources directory (staged, bundled, or explicit override)
        #[arg(long)]
        patching_dir: Option<PathBuf>,
    },
    /// Read and write configuration
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Print the current configuration
    List,
    /// Set a configuration value
    Set {
        /// Config key (runner, wow_dir, bottle, enable_lib_silicon)
        key: String,
        /// New value
        value: String,
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

fn print_runner_table() {
    info("Checking available runners…");
    for check in SetupOrchestrator::check_all_runners() {
        if check.available {
            ok(&format!("{}  ✅  {:?}", check.display_name, check.path));
        } else {
            warn(&format!("{}  ❌  not found", check.display_name));
        }
    }
}

fn run_diagnose(wow_dir: Option<&PathBuf>, patching_dir: Option<&PathBuf>) {
    let report = match run_checklist(wow_dir.map(|p| p.as_path()), patching_dir.cloned()) {
        Ok(r) => r,
        Err(e) => {
            die(&format!("diagnostics failed: {e}"));
        }
    };

    info("Checking CrossOver…");
    if let Some(ref p) = report.crossover {
        ok(&format!("CrossOver: {p}"));
    } else {
        warn("CrossOver: not found");
    }

    info("Checking Whisky…");
    if let Some(ref p) = report.whisky {
        ok(&format!("Whisky: {p}"));
    } else {
        warn(&format!("Whisky: not found"));
    }

    info("Checking patching resources…");
    if let Some(ref p) = report.patching_dir {
        ok(&format!("Patching dir: {p}"));
    } else {
        warn("Patching dir: not found");
    }

    info("Checking rosettax87 service…");
    if report.rosetta_running {
        ok("rosettax87 service running");
    } else {
        warn("rosettax87 not running — will start on launch");
    }

    if let Some(ref wl2) = report.wineloader2 {
        info("Checking wineloader2…");
        ok(&format!("wineloader2: {wl2}"));
    }

    if let (Some(patched), Some(found)) = (report.divxdecoder_patched, report.divxdecoder_found) {
        info("Checking DivxDecoder…");
        if patched {
            ok("DivxDecoder.dll patched (native Rust patcher)");
        } else if found {
            warn("DivxDecoder.dll not yet patched — will patch on first launch");
        } else {
            warn("DivxDecoder.dll not found — reinstall WoW client if launch fails");
        }
    }

    info("Available runners:");
    for runner in report.runners {
        let status = if runner.available {
            "available"
        } else {
            "not found"
        };
        info(&format!("  {}: {status}", runner.name));
    }
}

fn make_log_path() -> Result<PathBuf, String> {
    let log_dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".local/share/wowplay/logs"),
        Err(_) => std::env::temp_dir().join("wowplay/logs"),
    };
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
    let verbose = cli.verbose;

    match cli.command {
        Cmd::Diagnose {
            wow_dir,
            patching_dir,
        } => {
            run_diagnose(wow_dir.as_ref(), patching_dir.as_ref());
        }

        Cmd::Setup {
            patching_dir,
            runner,
            wow_dir,
            enable_lib_silicon,
        } => {
            let log_path = match make_log_path() {
                Ok(p) => {
                    eprintln!("Logs are saved to: {}", p.display());
                    Some(p)
                }
                Err(e) => {
                    warn(&format!("could not create log file: {e}"));
                    None
                }
            };

            print_runner_table();

            let store = TomlConfigStore::new();
            let messages = if runner.is_some() || wow_dir.is_some() || enable_lib_silicon {
                let prompt = HeadlessPrompt {
                    runner,
                    wow_dir,
                    enable_lib_silicon,
                };
                SetupOrchestrator::interactive_setup(&prompt, &store, patching_dir)
            } else {
                let prompt = DialoguerPrompt;
                SetupOrchestrator::interactive_setup(&prompt, &store, patching_dir)
            }
            .unwrap_or_else(|e| {
                if let Some(ref p) = log_path {
                    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p)
                    {
                        let _ = writeln!(f, "[fail] {e}");
                    }
                }
                die(&e.to_string())
            });

            if let Some(ref p) = log_path {
                if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
                    for msg in &messages {
                        let _ = writeln!(f, "[ ok ] {msg}");
                    }
                }
            }

            for msg in messages {
                ok(&msg);
            }
            ok("Configured successfully");
        }

        Cmd::Reset { wow_dir, yes } => {
            if !yes {
                warn("This will remove all wowplay patches and staged files:");
                info("  - restore DivxDecoder.dll / DivxTac.dll from .bak");
                info("  - remove d3d9.dll, mods/, rosettax87/, dlls.txt");
                info("  - remove CrossOver wineloader2 copy");
                info("  - remove ~/.local/share/wowplay/patching");
                eprint!("  Are you sure? [y/N]: ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_err() {
                    die("could not read confirmation");
                }
                let trimmed = input.trim().to_lowercase();
                if trimmed != "y" && trimmed != "yes" {
                    die("reset aborted");
                }
            }

            let log_path = match make_log_path() {
                Ok(p) => {
                    eprintln!("Logs are saved to: {}", p.display());
                    Some(p)
                }
                Err(e) => {
                    warn(&format!("could not create log file: {e}"));
                    None
                }
            };

            let messages = ResetOrchestrator::run(&wow_dir).unwrap_or_else(|e| {
                if let Some(ref p) = log_path {
                    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
                        let _ = writeln!(f, "[fail] {e}");
                    }
                }
                die(&e.to_string())
            });

            if let Some(ref p) = log_path {
                if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
                    for msg in &messages {
                        let _ = writeln!(f, "[ ok ] {msg}");
                    }
                }
            }

            for msg in messages {
                ok(&msg);
            }
        }

        Cmd::Run {
            runner,
            wow_dir,
            bottle,
            patching_dir,
            sudo: _,
            diagnose,
            no_log,
            whisky_bundle,
        } => {
            if diagnose {
                run_diagnose(wow_dir.as_ref(), patching_dir.as_ref());
                return;
            }

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

            if let Some(ref p) = log_path {
                eprintln!("Logs are saved to: {}", p.display());
            }

            let store = TomlConfigStore::new();
            let overrides = RunOverrides {
                runner,
                wow_dir,
                bottle,
                patching_dir,
                whisky_bundle,
            };

            let session = run_wow(&store, overrides, log_path.as_deref(), verbose > 0)
                .unwrap_or_else(|e| {
                    if let Some(ref p) = log_path {
                        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p)
                        {
                            let _ = writeln!(f, "[fail] {e}");
                        }
                    }
                    die(&e.to_string())
                });

            session.wait().unwrap_or_else(|e| die(&e.to_string()));
        }

        Cmd::Config { cmd } => {
            let store = TomlConfigStore::new();
            match cmd {
                ConfigCmd::List => {
                    let out = list_config(&store).unwrap_or_else(|e| die(&e.to_string()));
                    eprintln!("{out}");
                }
                ConfigCmd::Set { key, value } => {
                    let out = set_config(&store, &key, &value)
                        .unwrap_or_else(|e| die(&e.to_string()));
                    ok(&out);
                }
            }
        }
    }
}
