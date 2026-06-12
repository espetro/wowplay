//! Reset orchestrator — reverts all side effects created by [`SetupOrchestrator::apply`](crate::setup::SetupOrchestrator)
//! and [`WowLauncher::apply_game_patch`](crate::integration::wow_launcher::WowLauncher).
//!
//! Every operation is idempotent: missing files, directories, and backups are silently skipped.

use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::errors::LaunchError;

/// Orchestrates a full uninstall-style reset of the WoW on Apple Silicon setup.
///
/// Reverts every side effect created by setup:
/// - Restores `DivxDecoder.dll` and `DivxTac.dll` from `.bak` backups
/// - Removes `d3d9.dll`
/// - Removes `mods/winerosetta.dll` and `mods/libSiliconPatch.dll`
/// - Removes the entire `rosettax87/` directory
/// - Cleans `dlls.txt` of mod entries (deletes the file if empty)
/// - Removes CrossOver's `wineloader2` copy (if CrossOver is found)
/// - Removes staged patching resources at `~/.local/share/wowplay/patching`
pub struct ResetOrchestrator;

impl ResetOrchestrator {
    /// Runs the full reset sequence against the given WoW directory.
    ///
    /// Returns a human-readable log of actions taken.
    /// All operations are idempotent — missing items are silently skipped.
    pub fn run(wow_dir: &Path) -> Result<Vec<String>, LaunchError> {
        if !wow_dir.exists() {
            return Err(LaunchError::WowDirNotFound(wow_dir.display().to_string()));
        }

        let mut log = Vec::new();

        Self::restore_dll_backups(wow_dir, &mut log)?;
        Self::remove_d3d9(wow_dir, &mut log)?;
        Self::remove_mod_dlls(wow_dir, &mut log)?;
        Self::remove_rosettax87(wow_dir, &mut log)?;
        Self::clean_dlls_txt(wow_dir, &mut log)?;
        Self::remove_wineloader2(&mut log)?;
        Self::remove_staged_resources(&mut log)?;

        Ok(log)
    }

    // ── Step 1: Restore patched DLLs from .bak backups ──

    fn restore_dll_backups(wow_dir: &Path, log: &mut Vec<String>) -> Result<(), LaunchError> {
        let dlls = ["DivxDecoder.dll", "DivxTac.dll"];

        for dll_name in &dlls {
            let dll_path = wow_dir.join(dll_name);
            let bak_path = wow_dir.join(format!("{}.bak", dll_name));

            if !bak_path.exists() {
                continue;
            }

            // Remove the patched version if it exists
            if dll_path.exists() {
                fs::remove_file(&dll_path).map_err(|e| {
                    LaunchError::SetupFailed(format!("remove patched {}: {e}", dll_path.display()))
                })?;
            }

            // Restore from backup
            fs::rename(&bak_path, &dll_path).map_err(|e| {
                LaunchError::SetupFailed(format!("restore {} from backup: {e}", dll_path.display()))
            })?;

            log.push(format!("restored {}", dll_name));
        }

        Ok(())
    }

    // ── Step 2: Remove d3d9.dll ──

    fn remove_d3d9(wow_dir: &Path, log: &mut Vec<String>) -> Result<(), LaunchError> {
        let path = wow_dir.join("d3d9.dll");
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| LaunchError::SetupFailed(format!("remove d3d9.dll: {e}")))?;
            log.push("removed d3d9.dll".to_string());
        }
        Ok(())
    }

    // ── Step 3: Remove mod DLLs ──

    fn remove_mod_dlls(wow_dir: &Path, log: &mut Vec<String>) -> Result<(), LaunchError> {
        let mods = ["mods/winerosetta.dll", "mods/libSiliconPatch.dll"];

        for rel in &mods {
            let path = wow_dir.join(rel);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| {
                    LaunchError::SetupFailed(format!("remove {}: {e}", path.display()))
                })?;
                log.push(format!("removed {}", rel));
            }
        }

        // Clean up empty mods/ directory
        let mods_dir = wow_dir.join("mods");
        if mods_dir.is_dir() {
            let is_empty = fs::read_dir(&mods_dir)
                .map(|mut d| d.next().is_none())
                .unwrap_or(false);
            if is_empty {
                fs::remove_dir(&mods_dir)
                    .map_err(|e| LaunchError::SetupFailed(format!("remove empty mods/: {e}")))?;
                log.push("removed empty mods/".to_string());
            }
        }

        Ok(())
    }

    // ── Step 4: Remove rosettax87/ directory ──

    fn remove_rosettax87(wow_dir: &Path, log: &mut Vec<String>) -> Result<(), LaunchError> {
        let path = wow_dir.join("rosettax87");
        if path.exists() {
            fs::remove_dir_all(&path)
                .map_err(|e| LaunchError::SetupFailed(format!("remove rosettax87/: {e}")))?;
            log.push("removed rosettax87/".to_string());
        }
        Ok(())
    }

    // ── Step 5: Clean dlls.txt ──

    fn clean_dlls_txt(wow_dir: &Path, log: &mut Vec<String>) -> Result<(), LaunchError> {
        let path = wow_dir.join("dlls.txt");
        if !path.exists() {
            return Ok(());
        }

        let entries_to_remove = ["mods/winerosetta.dll", "mods/libSiliconPatch.dll"];

        let existing = fs::read_to_string(&path)
            .map_err(|e| LaunchError::SetupFailed(format!("read dlls.txt: {e}")))?;

        let remaining: Vec<&str> = existing
            .lines()
            .filter(|line| {
                let lower = line.trim().to_lowercase();
                !entries_to_remove.iter().any(|r| lower == r.to_lowercase())
            })
            .collect();

        if remaining.is_empty() {
            fs::remove_file(&path)
                .map_err(|e| LaunchError::SetupFailed(format!("remove empty dlls.txt: {e}")))?;
            log.push("removed dlls.txt (empty after cleanup)".to_string());
        } else if remaining.len() < existing.lines().count() {
            let mut out = remaining.join("\n");
            out.push('\n');
            fs::write(&path, &out)
                .map_err(|e| LaunchError::SetupFailed(format!("write dlls.txt: {e}")))?;
            log.push("cleaned dlls.txt (removed mod entries)".to_string());
        }

        Ok(())
    }

    // ── Step 6: Remove CrossOver's wineloader2 ──

    fn remove_wineloader2(log: &mut Vec<String>) -> Result<(), LaunchError> {
        let crossover = match crate::integration::crossover::find_crossover() {
            Ok(cx) => cx,
            Err(_) => return Ok(()), // CrossOver not found, silently skip
        };

        let loader2 = crate::integration::crossover::wineloader2_path(&crossover);
        if loader2.exists() {
            fs::remove_file(&loader2)
                .map_err(|e| LaunchError::SetupFailed(format!("remove wineloader2: {e}")))?;
            log.push(format!(
                "removed {}",
                loader2.file_name().unwrap_or_default().to_string_lossy()
            ));
        }

        Ok(())
    }

    // ── Step 7: Remove staged patching resources ──

    fn remove_staged_resources(log: &mut Vec<String>) -> Result<(), LaunchError> {
        let home = std::env::var("HOME").unwrap_or_default();
        let staging = PathBuf::from(home).join(".local/share/wowplay/patching");

        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|e| {
                LaunchError::SetupFailed(format!("remove {}: {e}", staging.display()))
            })?;
            log.push(format!("removed {}", staging.display()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::setup::SetupOrchestrator;
    use std::fs;
    use tempfile::TempDir;

    // ── Helper: create a fake wow directory ──

    fn create_fake_wow_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create minimal WoW-like structure
        fs::write(dir.path().join("WoW.exe"), b"fake").unwrap();
        dir
    }

    fn write_backup(wow_dir: &Path, dll_name: &str, content: &[u8]) {
        fs::write(wow_dir.join(format!("{}.bak", dll_name)), content).unwrap();
    }

    fn write_file(wow_dir: &Path, rel: &str, content: &[u8]) {
        let path = wow_dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    // ── Test: restore DLL backups ──

    #[test]
    fn test_restore_dll_backups_restores_divxdecoder() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        // Write a patched DLL and its backup
        write_file(wow_dir, "DivxDecoder.dll", b"patched content");
        write_backup(wow_dir, "DivxDecoder.dll", b"original content");

        let mut log = Vec::new();
        ResetOrchestrator::restore_dll_backups(wow_dir, &mut log).unwrap();

        assert_eq!(
            fs::read(wow_dir.join("DivxDecoder.dll")).unwrap(),
            b"original content"
        );
        assert!(!wow_dir.join("DivxDecoder.dll.bak").exists());
        assert!(log.contains(&"restored DivxDecoder.dll".to_string()));
    }

    #[test]
    fn test_restore_dll_backups_restores_both_dlls() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        write_file(wow_dir, "DivxDecoder.dll", b"patched decoder");
        write_backup(wow_dir, "DivxDecoder.dll", b"original decoder");
        write_file(wow_dir, "DivxTac.dll", b"patched tac");
        write_backup(wow_dir, "DivxTac.dll", b"original tac");

        let mut log = Vec::new();
        ResetOrchestrator::restore_dll_backups(wow_dir, &mut log).unwrap();

        assert_eq!(
            fs::read(wow_dir.join("DivxDecoder.dll")).unwrap(),
            b"original decoder"
        );
        assert_eq!(
            fs::read(wow_dir.join("DivxTac.dll")).unwrap(),
            b"original tac"
        );
        assert!(log.contains(&"restored DivxDecoder.dll".to_string()));
        assert!(log.contains(&"restored DivxTac.dll".to_string()));
    }

    #[test]
    fn test_restore_dll_backups_skips_missing_backup() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        // Only patched DLL, no backup
        write_file(wow_dir, "DivxDecoder.dll", b"no backup for this");

        let mut log = Vec::new();
        ResetOrchestrator::restore_dll_backups(wow_dir, &mut log).unwrap();

        // Should not touch the DLL
        assert_eq!(
            fs::read(wow_dir.join("DivxDecoder.dll")).unwrap(),
            b"no backup for this"
        );
        assert!(log.is_empty());
    }

    #[test]
    fn test_restore_dll_backups_restores_from_backup_when_dll_missing() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        // Only backup exists (DLL was deleted manually)
        write_backup(wow_dir, "DivxDecoder.dll", b"original");

        let mut log = Vec::new();
        ResetOrchestrator::restore_dll_backups(wow_dir, &mut log).unwrap();

        assert_eq!(
            fs::read(wow_dir.join("DivxDecoder.dll")).unwrap(),
            b"original"
        );
        assert!(!wow_dir.join("DivxDecoder.dll.bak").exists());
        assert!(log.contains(&"restored DivxDecoder.dll".to_string()));
    }

    // ── Test: remove d3d9.dll ──

    #[test]
    fn test_remove_d3d9_removes_file() {
        let dir = create_fake_wow_dir();
        write_file(dir.path(), "d3d9.dll", b"d3d9 binary");

        let mut log = Vec::new();
        ResetOrchestrator::remove_d3d9(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("d3d9.dll").exists());
        assert!(log.contains(&"removed d3d9.dll".to_string()));
    }

    #[test]
    fn test_remove_d3d9_skips_missing() {
        let dir = create_fake_wow_dir();

        let mut log = Vec::new();
        ResetOrchestrator::remove_d3d9(dir.path(), &mut log).unwrap();

        assert!(log.is_empty());
    }

    // ── Test: remove mod DLLs ──

    #[test]
    fn test_remove_mod_dlls_removes_both() {
        let dir = create_fake_wow_dir();
        write_file(dir.path(), "mods/winerosetta.dll", b"wine");
        write_file(dir.path(), "mods/libSiliconPatch.dll", b"silicon");

        let mut log = Vec::new();
        ResetOrchestrator::remove_mod_dlls(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("mods/winerosetta.dll").exists());
        assert!(!dir.path().join("mods/libSiliconPatch.dll").exists());
        assert!(log.contains(&"removed mods/winerosetta.dll".to_string()));
        assert!(log.contains(&"removed mods/libSiliconPatch.dll".to_string()));
    }

    #[test]
    fn test_remove_mod_dlls_removes_empty_mods_dir() {
        let dir = create_fake_wow_dir();
        // Only mod DLL, no other files in mods/
        write_file(dir.path(), "mods/winerosetta.dll", b"wine");

        let mut log = Vec::new();
        ResetOrchestrator::remove_mod_dlls(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("mods").exists());
        assert!(log.contains(&"removed empty mods/".to_string()));
    }

    #[test]
    fn test_remove_mod_dlls_keeps_mods_dir_with_other_files() {
        let dir = create_fake_wow_dir();
        write_file(dir.path(), "mods/winerosetta.dll", b"wine");
        write_file(dir.path(), "mods/other.dll", b"other");

        let mut log = Vec::new();
        ResetOrchestrator::remove_mod_dlls(dir.path(), &mut log).unwrap();

        assert!(dir.path().join("mods/other.dll").exists());
        assert!(dir.path().join("mods").is_dir());
        assert!(!log.iter().any(|m| m.contains("empty mods/")));
    }

    #[test]
    fn test_remove_mod_dlls_skips_missing() {
        let dir = create_fake_wow_dir();

        let mut log = Vec::new();
        ResetOrchestrator::remove_mod_dlls(dir.path(), &mut log).unwrap();

        assert!(log.is_empty());
    }

    // ── Test: remove rosettax87/ ──

    #[test]
    fn test_remove_rosettax87_removes_directory() {
        let dir = create_fake_wow_dir();
        write_file(dir.path(), "rosettax87/runtime_loader", b"loader");
        write_file(dir.path(), "rosettax87/libRuntimeRosettax87", b"lib");

        let mut log = Vec::new();
        ResetOrchestrator::remove_rosettax87(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("rosettax87").exists());
        assert!(log.contains(&"removed rosettax87/".to_string()));
    }

    #[test]
    fn test_remove_rosettax87_skips_missing() {
        let dir = create_fake_wow_dir();

        let mut log = Vec::new();
        ResetOrchestrator::remove_rosettax87(dir.path(), &mut log).unwrap();

        assert!(log.is_empty());
    }

    // ── Test: clean dlls.txt ──

    #[test]
    fn test_clean_dlls_txt_removes_mod_entries() {
        let dir = create_fake_wow_dir();
        fs::write(
            dir.path().join("dlls.txt"),
            "some/other.dll\nmods/winerosetta.dll\nmods/libSiliconPatch.dll\n",
        )
        .unwrap();

        let mut log = Vec::new();
        ResetOrchestrator::clean_dlls_txt(dir.path(), &mut log).unwrap();

        let contents = fs::read_to_string(dir.path().join("dlls.txt")).unwrap();
        assert!(contents.contains("some/other.dll"));
        assert!(!contents.to_lowercase().contains("winerosetta"));
        assert!(!contents.to_lowercase().contains("libsiliconpatch"));
        assert!(log.iter().any(|m| m.contains("cleaned dlls.txt")));
    }

    #[test]
    fn test_clean_dlls_txt_deletes_file_when_empty() {
        let dir = create_fake_wow_dir();
        fs::write(
            dir.path().join("dlls.txt"),
            "mods/winerosetta.dll\nmods/libSiliconPatch.dll\n",
        )
        .unwrap();

        let mut log = Vec::new();
        ResetOrchestrator::clean_dlls_txt(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("dlls.txt").exists());
        assert!(log.iter().any(|m| m.contains("removed dlls.txt")));
    }

    #[test]
    fn test_clean_dlls_txt_skips_nonexistent_file() {
        let dir = create_fake_wow_dir();

        let mut log = Vec::new();
        ResetOrchestrator::clean_dlls_txt(dir.path(), &mut log).unwrap();

        assert!(log.is_empty());
    }

    #[test]
    fn test_clean_dlls_txt_preserves_unrelated_entries() {
        let dir = create_fake_wow_dir();
        fs::write(dir.path().join("dlls.txt"), "d3d9.dll\nsome/other.dll\n").unwrap();

        let mut log = Vec::new();
        ResetOrchestrator::clean_dlls_txt(dir.path(), &mut log).unwrap();

        let contents = fs::read_to_string(dir.path().join("dlls.txt")).unwrap();
        assert_eq!(contents, "d3d9.dll\nsome/other.dll\n");
        // No entries were removed, so no log entry
        assert!(log.is_empty());
    }

    #[test]
    fn test_clean_dlls_txt_case_insensitive() {
        let dir = create_fake_wow_dir();
        fs::write(
            dir.path().join("dlls.txt"),
            "Mods/Winerosetta.dll\nMODS/LIBSILICONPATCH.DLL\n",
        )
        .unwrap();

        let mut log = Vec::new();
        ResetOrchestrator::clean_dlls_txt(dir.path(), &mut log).unwrap();

        assert!(!dir.path().join("dlls.txt").exists());
    }

    // ── Test: full run integration ──

    #[test]
    fn test_run_full_reset() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        // Set up a full patch state
        write_file(wow_dir, "DivxDecoder.dll", b"patched decoder");
        write_backup(wow_dir, "DivxDecoder.dll", b"original decoder");
        write_file(wow_dir, "DivxTac.dll", b"patched tac");
        write_backup(wow_dir, "DivxTac.dll", b"original tac");
        write_file(wow_dir, "d3d9.dll", b"d3d9");
        write_file(wow_dir, "mods/winerosetta.dll", b"wine");
        write_file(wow_dir, "mods/libSiliconPatch.dll", b"silicon");
        write_file(wow_dir, "rosettax87/runtime_loader", b"loader");
        write_file(wow_dir, "rosettax87/libRuntimeRosettax87", b"lib");
        fs::write(
            wow_dir.join("dlls.txt"),
            "mods/winerosetta.dll\nmods/libSiliconPatch.dll\n",
        )
        .unwrap();

        let log = ResetOrchestrator::run(wow_dir).unwrap();

        // Verify DLLs restored
        assert_eq!(
            fs::read(wow_dir.join("DivxDecoder.dll")).unwrap(),
            b"original decoder"
        );
        assert_eq!(
            fs::read(wow_dir.join("DivxTac.dll")).unwrap(),
            b"original tac"
        );
        assert!(!wow_dir.join("DivxDecoder.dll.bak").exists());
        assert!(!wow_dir.join("DivxTac.dll.bak").exists());

        // Verify files removed
        assert!(!wow_dir.join("d3d9.dll").exists());
        assert!(!wow_dir.join("mods/winerosetta.dll").exists());
        assert!(!wow_dir.join("mods/libSiliconPatch.dll").exists());
        assert!(!wow_dir.join("rosettax87").exists());
        assert!(!wow_dir.join("dlls.txt").exists());

        // Verify log contains key entries
        assert!(log.iter().any(|m| m.contains("restored DivxDecoder.dll")));
        assert!(log.iter().any(|m| m.contains("restored DivxTac.dll")));
        assert!(log.iter().any(|m| m.contains("removed d3d9.dll")));
        assert!(log.iter().any(|m| m.contains("removed rosettax87/")));
    }

    // ── Test: idempotency ──

    #[test]
    fn test_run_idempotent() {
        let dir = create_fake_wow_dir();
        let wow_dir = dir.path();

        // Set up and run reset
        write_file(wow_dir, "DivxDecoder.dll", b"patched");
        write_backup(wow_dir, "DivxDecoder.dll", b"original");
        write_file(wow_dir, "d3d9.dll", b"d3d9");

        let log1 = ResetOrchestrator::run(wow_dir).unwrap();
        assert!(!log1.is_empty());

        // Run again — nothing to reset
        let log2 = ResetOrchestrator::run(wow_dir).unwrap();
        assert!(log2.is_empty());
    }

    #[test]
    fn test_run_on_clean_dir_only_removes_wineloader2_and_staging() {
        let dir = create_fake_wow_dir();

        let log = ResetOrchestrator::run(dir.path()).unwrap();

        // On a clean dir, only wineloader2 and staging might produce entries
        // (and only if they actually exist on the system)
        // The key point: no errors, no crashes
        for entry in &log {
            // These are the only allowed entries on a clean wow dir
            assert!(
                entry.contains("wineloader2") || entry.contains("wowplay/patching"),
                "unexpected log entry on clean dir: {}",
                entry
            );
        }
    }

    #[test]
    fn test_run_fails_on_nonexistent_dir() {
        let result = ResetOrchestrator::run(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
        match result.unwrap_err() {
            LaunchError::WowDirNotFound(_) => {}
            other => panic!("expected WowDirNotFound, got {:?}", other),
        }
    }

    fn create_minimal_patching_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "d9vk/d3d9.dll", b"d3d9");
        write_file(dir.path(), "winerosetta/winerosetta.dll", b"wine");
        write_file(dir.path(), "rosettax87/rosettax87", b"rosettax87");
        write_file(
            dir.path(),
            "rosettax87/libRuntimeRosettax87",
            b"libRuntimeRosettax87",
        );
        dir
    }

    fn copy_minimal_dll_as_divx(wow_dir: &Path) {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal.dll");
        let target = wow_dir.join("DivxDecoder.dll");
        fs::copy(&fixture, &target).expect("copy minimal.dll fixture");
    }

    fn assert_wow_dir_is_pristine(wow_dir: &Path) {
        assert!(wow_dir.join("WoW.exe").exists());
        assert!(wow_dir.join("DivxDecoder.dll").exists());
        assert!(!wow_dir.join("DivxDecoder.dll.bak").exists());
        assert!(!wow_dir.join("d3d9.dll").exists());
        assert!(!wow_dir.join("mods").exists());
        assert!(!wow_dir.join("rosettax87").exists());
        assert!(!wow_dir.join("dlls.txt").exists());
    }

    #[test]
    fn test_setup_then_reset_is_identity() {
        let wow_dir = TempDir::new().unwrap();
        let patching_dir = create_minimal_patching_dir();

        fs::write(wow_dir.path().join("WoW.exe"), b"fake wow").unwrap();
        copy_minimal_dll_as_divx(wow_dir.path());

        SetupOrchestrator::apply(
            wow_dir.path(),
            Some(patching_dir.path().to_path_buf()),
            &AppConfig {
                runner: "crossover".into(),
                wow_dir: wow_dir.path().to_path_buf(),
                bottle: "Win10".into(),
                enable_lib_silicon: false,
            },
        )
        .expect("setup should succeed");
        ResetOrchestrator::run(wow_dir.path()).expect("reset should succeed");

        assert_wow_dir_is_pristine(wow_dir.path());
    }

    #[test]
    fn test_setup_is_idempotent() {
        let wow_dir = TempDir::new().unwrap();
        let patching_dir = create_minimal_patching_dir();

        fs::write(wow_dir.path().join("WoW.exe"), b"fake wow").unwrap();
        copy_minimal_dll_as_divx(wow_dir.path());

        SetupOrchestrator::apply(
            wow_dir.path(),
            Some(patching_dir.path().to_path_buf()),
            &AppConfig {
                runner: "crossover".into(),
                wow_dir: wow_dir.path().to_path_buf(),
                bottle: "Win10".into(),
                enable_lib_silicon: false,
            },
        )
        .expect("first setup should succeed");
        let mut entries_after_first: Vec<String> = fs::read_dir(wow_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        entries_after_first.sort();

        let log2 = SetupOrchestrator::apply(
            wow_dir.path(),
            Some(patching_dir.path().to_path_buf()),
            &AppConfig {
                runner: "crossover".into(),
                wow_dir: wow_dir.path().to_path_buf(),
                bottle: "Win10".into(),
                enable_lib_silicon: false,
            },
        )
        .expect("second setup should succeed");
        let mut entries_after_second: Vec<String> = fs::read_dir(wow_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        entries_after_second.sort();

        assert_eq!(entries_after_first, entries_after_second);
        assert!(log2.iter().any(|m| m.contains("game patch applied")));
        assert!(log2.iter().any(|m| m.contains("DivxDecoder.dll patched")));
    }
}
