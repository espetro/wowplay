use std::path::PathBuf;

use wow_silicon_core::integration::crossover::{find_crossover, CrossoverLauncher};
use wow_silicon_core::ports::launcher::WowLauncherPort;

/// Full launch validation — requires:
/// 1. WoW 3.3.5a client at ~/Documents/ChromieCraft_3.3.5a (or set WOW_DIR env var)
/// 2. WoWSilicon.app installed in ~/Applications or /Applications
/// 3. CrossOver.app installed
#[test]
#[ignore]
fn test_wow_launches() {
    let wow_dir = std::env::var("WOW_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs_home().join("Documents/ChromieCraft_3.3.5a"));

    let launcher = CrossoverLauncher::new().expect("CrossOver or WoWSilicon not found");

    launcher
        .check_prerequisites()
        .expect("prerequisites not met");

    let mut child = launcher.launch_wow(&wow_dir).expect("launch failed");
    // Give WoW a moment to start, then check it's still alive
    std::thread::sleep(std::time::Duration::from_secs(5));
    let status = child.try_wait().expect("wait failed");
    assert!(status.is_none(), "WoW exited immediately: {:?}", status);
    child.kill().ok();
}

/// Verify prerequisites without actually launching the game
#[test]
#[ignore]
fn test_check_prerequisites() {
    let launcher = CrossoverLauncher::new().expect("CrossOver or WoWSilicon not found");
    launcher
        .check_prerequisites()
        .expect("prerequisites not met");
    println!("CrossOver found at: {:?}", find_crossover());
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_default()
}
