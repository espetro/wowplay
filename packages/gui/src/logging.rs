use std::fs;
use std::path::PathBuf;

/// Creates a timestamped log file path under `~/.local/share/wowplay/logs/`
/// (falls back to `$TMPDIR/wowplay/logs/` if HOME is unavailable).
pub fn make_log_path() -> Option<PathBuf> {
    let log_dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".local/share/wowplay/logs"),
        Err(_) => std::env::temp_dir().join("wowplay/logs"),
    };
    fs::create_dir_all(&log_dir).ok()?;
    prune_old_logs(&log_dir);
    Some(log_dir.join(format!("{}.log", timestamp_now())))
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
