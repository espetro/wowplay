//! Resource resolution — finds WoWSilicon patching resources on disk.

use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::errors::LaunchError;
use crate::integration::crossover::{find_wowsilicon, wowsilicon_resources};

/// Resolves the patching resources directory.
///
/// Resolution order:
/// 1. Explicit path if provided
/// 2. Previously staged at `~/.local/share/wowplay/patching`
/// 3. Bundled next to the binary (release zip layout)
/// 4. WoWSilicon.app (legacy / developer path)
pub fn resolve_patching_dir(explicit: Option<PathBuf>) -> Result<PathBuf, LaunchError> {
    if let Some(p) = explicit {
        return Ok(p);
    }

    // env var — dev/CI escape hatch without needing --patching-dir flag
    if let Ok(p) = std::env::var("WOWPLAY_PATCHING_DIR") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Ok(p);
        }
    }

    // 1. Previously staged by wowplay setup
    let installed = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/wowplay/patching");
    if installed.exists() {
        return Ok(installed);
    }

    // 2. Bundled next to the binary (release zip layout)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("patching");
            if bundled.exists() {
                return Ok(bundled);
            }
            // App bundle layout (Tauri DMG): Contents/MacOS/../Resources/patching
            let bundle_resources = dir.join("../Resources/patching");
            if bundle_resources.exists() {
                return Ok(bundle_resources.canonicalize().unwrap_or(bundle_resources));
            }
        }
    }

    // 3. WoWSilicon.app (legacy / developer path)
    if let Ok(app) = find_wowsilicon() {
        return Ok(wowsilicon_resources(&app));
    }

    Err(LaunchError::SetupFailed(
        "patching resources not found — download the release zip and run wowplay setup".into(),
    ))
}

/// Recursively copies a directory tree.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), LaunchError> {
    fs::create_dir_all(dst)
        .map_err(|e| LaunchError::SetupFailed(format!("mkdir {}: {e}", dst.display())))?;
    for entry in fs::read_dir(src)
        .map_err(|e| LaunchError::SetupFailed(format!("read_dir {}: {e}", src.display())))?
    {
        let entry = entry.map_err(|e| LaunchError::SetupFailed(format!("dir entry: {e}")))?;
        let ty = entry
            .file_type()
            .map_err(|e| LaunchError::SetupFailed(format!("file type: {e}")))?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path).map_err(|e| {
                LaunchError::SetupFailed(format!("copy {}: {e}", entry.path().display()))
            })?;
        }
    }
    Ok(())
}

/// Stages bundled patching resources to `~/.local/share/wowplay/patching`.
///
/// Looks for a `patching/` directory next to the current executable and copies
/// it to the user's data directory if not already present.
pub fn stage_bundled_resources() -> Result<Option<PathBuf>, LaunchError> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("patching");
            let staged = PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".local/share/wowplay/patching");
            if bundled.exists() && !staged.exists() {
                copy_dir_recursive(&bundled, &staged)?;
                return Ok(Some(staged));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_recursive() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create a nested structure
        let subdir = src.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let mut file = fs::File::create(subdir.join("test.txt")).unwrap();
        writeln!(file, "hello").unwrap();

        copy_dir_recursive(src.path(), &dst.path().join("copied")).unwrap();

        assert!(dst.path().join("copied/subdir/test.txt").exists());
        let content = fs::read_to_string(dst.path().join("copied/subdir/test.txt")).unwrap();
        assert!(content.contains("hello"));
    }
}
