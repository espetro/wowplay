//! End-to-end PE patching tests.

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use wow_silicon_core::adapters::pe_import_patcher::patch_dll_imports;

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u16_le(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

#[test]
fn test_divxdecoder_patching_e2e() {
    let tmp = tempdir().unwrap();
    let wow_dir = tmp.path();

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rust-core/tests/fixtures/minimal.dll");
    let target = wow_dir.join("DivxDecoder.dll");
    fs::copy(&fixture, &target).expect("copy fixture");

    patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
        .expect("patch should succeed");

    assert!(
        wow_dir.join("DivxDecoder.dll.bak").exists(),
        "Backup should be created"
    );

    // Verify by inspecting headers
    let patched = fs::read(&target).unwrap();
    let e_lfanew = read_u32_le(&patched, 0x3C) as usize;
    let coff = e_lfanew + 4;
    let num_sections = read_u16_le(&patched, coff + 2);
    assert!(num_sections >= 1, "Should have sections");

    // Find .winep section
    let size_opt = read_u16_le(&patched, coff + 16) as usize;
    let optional = coff + 20;
    let sec_table = optional + size_opt;
    let mut found_winep = false;
    for i in 0..num_sections {
        let off = sec_table + i as usize * 40;
        let name = std::str::from_utf8(&patched[off..off + 8])
            .unwrap_or("")
            .trim_end_matches('\0');
        if name == ".winep" {
            found_winep = true;
            break;
        }
    }
    assert!(found_winep, "Should have .winep section");
}

#[test]
fn test_patching_idempotent_e2e() {
    let tmp = tempdir().unwrap();
    let wow_dir = tmp.path();

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rust-core/tests/fixtures/minimal.dll");
    let target = wow_dir.join("DivxDecoder.dll");
    fs::copy(&fixture, &target).expect("copy fixture");

    patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
        .expect("first patch should succeed");

    patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
        .expect("second patch should be idempotent");

    assert!(
        wow_dir.join("DivxDecoder.dll.bak").exists(),
        "Backup should exist after idempotent patches"
    );
}

#[test]
fn test_restore_backup_e2e() {
    let tmp = tempdir().unwrap();
    let wow_dir = tmp.path();

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rust-core/tests/fixtures/minimal.dll");
    let target = wow_dir.join("DivxDecoder.dll");
    fs::copy(&fixture, &target).expect("copy fixture");
    let original_bytes = fs::read(&target).unwrap();

    patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll").unwrap();

    wow_silicon_core::adapters::pe_import_patcher::restore_dll_backup(wow_dir, "DivxDecoder.dll")
        .expect("restore should succeed");

    let restored_bytes = fs::read(&target).unwrap();
    assert_eq!(
        original_bytes, restored_bytes,
        "Restored file should match original"
    );

    assert!(
        !wow_dir.join("DivxDecoder.dll.bak").exists(),
        "Backup should be removed after restore"
    );
}
