use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use wow_silicon_core::integration::crossover::{
    apply_game_patch, create_wineloader2, find_crossover, find_wowsilicon, wowsilicon_resources,
};
use wow_silicon_core::ports::launcher::WowLauncherPort;
use wow_silicon_core::integration::crossover::CrossoverLauncher;

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
        /// Print diagnostics then exit without launching
        #[arg(long)]
        diagnose: bool,
    },
    /// One-time setup: stage DLLs and create wineloader2
    Setup {
        /// Path to WoW 3.3.5a game directory
        #[arg(long)]
        wow_dir: PathBuf,
    },
    /// Print environment checklist and exit
    Diagnose,
}

fn info(msg: &str) {
    eprintln!("  \x1b[34m[info]\x1b[0m {msg}");
}

fn ok(msg: &str) {
    eprintln!("  \x1b[32m[ ok ]\x1b[0m {msg}");
}

fn die(msg: &str) -> ! {
    eprintln!("  \x1b[31m[fail]\x1b[0m {msg}");
    process::exit(1);
}

fn run_diagnose() {
    info("Checking CrossOver…");
    match find_crossover() {
        Ok(p) => ok(&format!("CrossOver: {}", p.display())),
        Err(e) => eprintln!("  \x1b[33m[warn]\x1b[0m CrossOver: {e}"),
    }

    info("Checking WoWSilicon…");
    match find_wowsilicon() {
        Ok(p) => {
            ok(&format!("WoWSilicon: {}", p.display()));
            let res = wowsilicon_resources(&p);
            if res.exists() {
                ok(&format!("WoWSilicon resources: {}", res.display()));
            } else {
                eprintln!("  \x1b[33m[warn]\x1b[0m resources dir missing: {}", res.display());
            }
        }
        Err(e) => eprintln!("  \x1b[33m[warn]\x1b[0m WoWSilicon: {e}"),
    }

    let wineloader2 = PathBuf::from("/tmp/cx-bin/wineloader64");
    if wineloader2.exists() {
        ok("wineloader64 staged at /tmp/cx-bin/wineloader64");
    } else {
        eprintln!("  \x1b[33m[warn]\x1b[0m wineloader64 not staged — run `wowplay setup`");
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Diagnose => {
            run_diagnose();
        }

        Cmd::Setup { wow_dir } => {
            info("Setting up wineloader2…");
            let crossover = find_crossover().unwrap_or_else(|e| die(&e.to_string()));
            create_wineloader2(&crossover).unwrap_or_else(|e| die(&e.to_string()));
            ok("wineloader64 staged");

            info("Applying game patch…");
            let wowsilicon = find_wowsilicon().unwrap_or_else(|e| die(&e.to_string()));
            let resources = wowsilicon_resources(&wowsilicon);
            apply_game_patch(&wow_dir, &resources).unwrap_or_else(|e| die(&e.to_string()));
            ok("game patch applied");
        }

        Cmd::Run { wow_dir, bottle, diagnose } => {
            if diagnose {
                run_diagnose();
                return;
            }

            let launcher = CrossoverLauncher::with_bottle(&bottle)
                .unwrap_or_else(|e| die(&e.to_string()));

            let wow_dir = wow_dir.unwrap_or_else(|| {
                die("--wow-dir is required; e.g. wowplay run --wow-dir ~/WoW");
            });

            info(&format!("Launching WoW from {}…", wow_dir.display()));
            let mut child = launcher
                .launch_wow(&wow_dir)
                .unwrap_or_else(|e| die(&e.to_string()));

            ok(&format!("WoW started (pid {})", child.id()));
            child.wait().unwrap_or_else(|e| die(&e.to_string()));
        }
    }
}
