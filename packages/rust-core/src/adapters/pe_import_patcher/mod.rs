//! PE Import Patcher - Native Rust PE import table manipulation.
//!
//! Uses an append-only strategy: adds a new section containing the import table
//! without modifying any existing section, preserving RVAs and relocation tables.

use std::fs;
use std::path::Path;

mod errors;
pub use errors::{PatchError, PatchResult};

// ─── PE constants ───

const PE_SIG_SIZE: usize = 4;
const COFF_HEADER_SIZE: usize = 20;
const SECTION_HEADER_SIZE: usize = 40;
const IMPORT_DESCRIPTOR_SIZE: usize = 20;
const DATA_DIR_ENTRY_SIZE: usize = 8;
const THUNK_SIZE: usize = 4;

// ─── Little-endian helpers ───

fn read_u16_le(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn write_u16_le(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn align_up(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    value.div_ceil(alignment) * alignment
}

// ─── Public API ───

/// Patches a PE DLL to add a new import dependency by appending a new section.
///
/// This preserves every existing byte and RVA — only a new section is appended
/// at the end of the file. The relocation table and all other data directories
/// remain untouched.
pub fn patch_dll_imports(wow_dir: &Path, dll_name: &str, import_dll: &str) -> PatchResult<()> {
    let dll_path = wow_dir.join(dll_name);
    let backup_path = wow_dir.join(format!("{}.bak", dll_name));

    // Already patched?
    if backup_path.exists() {
        return Ok(());
    }

    if !dll_path.exists() {
        return Err(PatchError::DllNotFound(dll_path));
    }

    // Read original file
    let mut bytes = fs::read(&dll_path)?;
    let original_len = bytes.len();

    // ── Parse headers (read-only) ──

    let e_lfanew = read_u32_le(&bytes, 0x3C) as usize;
    let coff_offset = e_lfanew + PE_SIG_SIZE;
    let optional_offset = coff_offset + COFF_HEADER_SIZE;

    let number_of_sections = read_u16_le(&bytes, coff_offset + 2) as usize;
    let size_of_optional_header = read_u16_le(&bytes, coff_offset + 16) as usize;

    let magic = read_u16_le(&bytes, optional_offset);
    if magic != 0x10b {
        return Err(PatchError::InvalidPe(format!(
            "Expected PE32 (magic 0x10b), got 0x{:x}",
            magic
        )));
    }

    let section_alignment = read_u32_le(&bytes, optional_offset + 32);
    let file_alignment = read_u32_le(&bytes, optional_offset + 36);
    let _size_of_image = read_u32_le(&bytes, optional_offset + 56);
    let size_of_headers = read_u32_le(&bytes, optional_offset + 60);

    let data_dir_offset = optional_offset + 96;
    let import_dir_offset = data_dir_offset + DATA_DIR_ENTRY_SIZE;
    let import_dir_rva = read_u32_le(&bytes, import_dir_offset);
    let import_dir_size = read_u32_le(&bytes, import_dir_offset + 4);

    let section_table_offset = optional_offset + size_of_optional_header;

    // ── Capacity check ──

    let section_table_end = section_table_offset + number_of_sections * SECTION_HEADER_SIZE;
    if section_table_end + SECTION_HEADER_SIZE > size_of_headers as usize {
        return Err(PatchError::InvalidPe("No free section header slots".into()));
    }

    // ── Build RVA → file-offset map ──

    let rva_to_file = |rva: u32| -> Option<usize> {
        for i in 0..number_of_sections {
            let sec_off = section_table_offset + i * SECTION_HEADER_SIZE;
            let sec_va = read_u32_le(&bytes, sec_off + 12);
            let sec_raw = read_u32_le(&bytes, sec_off + 20);
            let sec_vsize = read_u32_le(&bytes, sec_off + 16);
            let sec_raw_size = read_u32_le(&bytes, sec_off + 16);
            let sec_size = sec_vsize.max(sec_raw_size);
            if rva >= sec_va && rva < sec_va + sec_size {
                return Some((sec_raw + (rva - sec_va)) as usize);
            }
        }
        None
    };

    // ── Read existing import descriptors ──

    let mut existing_descriptors: Vec<[u8; IMPORT_DESCRIPTOR_SIZE]> = Vec::new();
    if import_dir_rva != 0 && import_dir_size >= IMPORT_DESCRIPTOR_SIZE as u32 {
        let mut desc_file_off = rva_to_file(import_dir_rva)
            .ok_or_else(|| PatchError::InvalidPe("Cannot map import dir RVA".into()))?;

        loop {
            if desc_file_off + IMPORT_DESCRIPTOR_SIZE > original_len {
                break;
            }
            let mut desc = [0u8; IMPORT_DESCRIPTOR_SIZE];
            desc.copy_from_slice(&bytes[desc_file_off..desc_file_off + IMPORT_DESCRIPTOR_SIZE]);

            // Null descriptor?
            if desc.iter().all(|b| *b == 0) {
                break;
            }
            existing_descriptors.push(desc);
            desc_file_off += IMPORT_DESCRIPTOR_SIZE;
        }
    }

    // ── Check if already imports the target DLL ──

    for desc in &existing_descriptors {
        let name_rva = read_u32_le(desc, 12);
        if let Some(name_off) = rva_to_file(name_rva) {
            if name_off < original_len {
                // Read null-terminated string
                let end = bytes[name_off..]
                    .iter()
                    .position(|b| *b == 0)
                    .unwrap_or(bytes.len() - name_off);
                let name = std::str::from_utf8(&bytes[name_off..name_off + end]).unwrap_or("");
                if name.eq_ignore_ascii_case(import_dll) {
                    return Ok(()); // Already present
                }
            }
        }
    }

    // ── Lay out the new `.winep` section ──

    // Compute max VA end of existing sections
    let mut max_va_end = size_of_headers;
    for i in 0..number_of_sections {
        let sec_off = section_table_offset + i * SECTION_HEADER_SIZE;
        let sec_va = read_u32_le(&bytes, sec_off + 12);
        let sec_vsize = read_u32_le(&bytes, sec_off + 8);
        let sec_end = sec_va + sec_vsize;
        if sec_end > max_va_end {
            max_va_end = sec_end;
        }
    }

    let new_va = align_up(max_va_end, section_alignment);
    let new_raw = align_up(original_len as u32, file_alignment);

    // Content layout inside the new section (RVAs relative to new_va)
    let descriptors_size = (existing_descriptors.len() + 2) * IMPORT_DESCRIPTOR_SIZE; // +1 new + null
    let ilt_offset = descriptors_size;
    let iat_offset = ilt_offset + THUNK_SIZE;
    let name_offset = iat_offset + THUNK_SIZE;
    let name_bytes = import_dll.as_bytes();
    let name_len = name_bytes.len() + 1; // + null terminator

    let content_size = name_offset + name_len;
    let padded_size = align_up(content_size as u32, file_alignment) as usize;

    let mut section_data = vec![0u8; padded_size];

    // Write existing descriptors
    let mut pos = 0usize;
    for desc in &existing_descriptors {
        section_data[pos..pos + IMPORT_DESCRIPTOR_SIZE].copy_from_slice(desc);
        pos += IMPORT_DESCRIPTOR_SIZE;
    }

    // Write new descriptor
    let new_ilt_rva = new_va + ilt_offset as u32;
    let new_iat_rva = new_va + iat_offset as u32;
    let new_name_rva = new_va + name_offset as u32;

    write_u32_le(&mut section_data, pos, new_ilt_rva); // OriginalFirstThunk
    write_u32_le(&mut section_data, pos + 4, 0); // TimeDateStamp
    write_u32_le(&mut section_data, pos + 8, 0); // ForwarderChain
    write_u32_le(&mut section_data, pos + 12, new_name_rva); // Name
    write_u32_le(&mut section_data, pos + 16, new_iat_rva); // FirstThunk

    // Null descriptor (already zeroed) follows at pos + IMPORT_DESCRIPTOR_SIZE

    // ILT entry (null thunk)
    // (already zeroed)

    // IAT entry (null thunk)
    // (already zeroed)

    // Name string
    section_data[name_offset..name_offset + name_bytes.len()].copy_from_slice(name_bytes);
    section_data[name_offset + name_bytes.len()] = 0;

    // ── Mutate headers ──

    // Write new section header
    let new_sec_header_off = section_table_end;
    let sec_name = b".winep\0\0";
    bytes[new_sec_header_off..new_sec_header_off + 8].copy_from_slice(sec_name);
    write_u32_le(&mut bytes, new_sec_header_off + 8, content_size as u32); // VirtualSize
    write_u32_le(&mut bytes, new_sec_header_off + 12, new_va); // VirtualAddress
    write_u32_le(&mut bytes, new_sec_header_off + 16, padded_size as u32); // SizeOfRawData
    write_u32_le(&mut bytes, new_sec_header_off + 20, new_raw); // PointerToRawData
    write_u32_le(&mut bytes, new_sec_header_off + 24, 0); // PointerToRelocations
    write_u32_le(&mut bytes, new_sec_header_off + 28, 0); // PointerToLinenumbers
    write_u16_le(&mut bytes, new_sec_header_off + 32, 0); // NumberOfRelocations
    write_u16_le(&mut bytes, new_sec_header_off + 34, 0); // NumberOfLinenumbers
    write_u32_le(&mut bytes, new_sec_header_off + 36, 0xC0000040); // Characteristics

    // NumberOfSections += 1
    write_u16_le(&mut bytes, coff_offset + 2, (number_of_sections + 1) as u16);

    // SizeOfImage
    let new_size_of_image = align_up(new_va + content_size as u32, section_alignment);
    write_u32_le(&mut bytes, optional_offset + 56, new_size_of_image);

    // Import DataDir
    let new_import_dir_rva = new_va; // descriptor array starts at section VA
    let new_import_dir_size =
        (existing_descriptors.len() + 2) as u32 * IMPORT_DESCRIPTOR_SIZE as u32;
    write_u32_le(&mut bytes, import_dir_offset, new_import_dir_rva);
    write_u32_le(&mut bytes, import_dir_offset + 4, new_import_dir_size);

    // Clear BoundImport DataDir if present (index 11)
    let bound_import_offset = data_dir_offset + 11 * DATA_DIR_ENTRY_SIZE;
    let bound_import_rva = read_u32_le(&bytes, bound_import_offset);
    if bound_import_rva != 0 {
        write_u32_le(&mut bytes, bound_import_offset, 0);
        write_u32_le(&mut bytes, bound_import_offset + 4, 0);
    }

    // ── Append section data ──

    bytes.resize(new_raw as usize + padded_size, 0);
    bytes[new_raw as usize..new_raw as usize + padded_size].copy_from_slice(&section_data);

    // ── Backup and write ──

    fs::copy(&dll_path, &backup_path).map_err(|e| PatchError::BackupFailed(format!("{}", e)))?;

    fs::write(&dll_path, &bytes).map_err(|e| PatchError::PeWrite(format!("{}", e)))?;

    Ok(())
}

/// Restores a DLL from its .bak backup.
pub fn restore_dll_backup(wow_dir: &Path, dll_name: &str) -> PatchResult<()> {
    let dll_path = wow_dir.join(dll_name);
    let backup_path = wow_dir.join(format!("{}.bak", dll_name));

    if !backup_path.exists() {
        return Err(PatchError::RestoreFailed(format!(
            "Backup not found: {}",
            backup_path.display()
        )));
    }

    fs::remove_file(&dll_path)?;
    fs::rename(&backup_path, &dll_path)?;

    Ok(())
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_dll(dir: &std::path::Path) -> PathBuf {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal.dll");
        let target = dir.join("DivxDecoder.dll");
        fs::copy(&fixture, &target).expect("copy fixture");
        target
    }

    #[test]
    fn test_patch_adds_import() {
        let tmp = tempdir().unwrap();
        let wow_dir = tmp.path();
        create_test_dll(wow_dir);

        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
            .expect("patch should succeed");

        assert!(wow_dir.join("DivxDecoder.dll.bak").exists());

        // Verify by parsing the patched file
        let patched = fs::read(wow_dir.join("DivxDecoder.dll")).unwrap();
        let e_lfanew = read_u32_le(&patched, 0x3C) as usize;
        let coff = e_lfanew + 4;
        let num_sections = read_u16_le(&patched, coff + 2);
        assert!(num_sections >= 1, "Should have at least one section");

        // Find .winep section
        let optional = coff + 20;
        let size_opt = read_u16_le(&patched, coff + 16) as usize;
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
    fn test_patch_is_idempotent() {
        let tmp = tempdir().unwrap();
        let wow_dir = tmp.path();
        create_test_dll(wow_dir);

        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
            .expect("first patch should succeed");
        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
            .expect("second patch should be idempotent");

        assert!(wow_dir.join("DivxDecoder.dll.bak").exists());
    }

    #[test]
    fn test_restore_backup() {
        let tmp = tempdir().unwrap();
        let wow_dir = tmp.path();
        let original = create_test_dll(wow_dir);
        let original_bytes = fs::read(&original).unwrap();

        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll").unwrap();
        restore_dll_backup(wow_dir, "DivxDecoder.dll").expect("restore should succeed");

        let restored_bytes = fs::read(&original).unwrap();
        assert_eq!(original_bytes, restored_bytes);
        assert!(!wow_dir.join("DivxDecoder.dll.bak").exists());
    }

    #[test]
    fn test_patch_missing_dll_fails() {
        let tmp = tempdir().unwrap();
        let wow_dir = tmp.path();

        let result = patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll");
        assert!(result.is_err());
    }

    #[test]
    fn test_byte_preservation() {
        let tmp = tempdir().unwrap();
        let wow_dir = tmp.path();
        let original = create_test_dll(wow_dir);
        let original_bytes = fs::read(&original).unwrap();
        let original_len = original_bytes.len();

        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
            .expect("patch should succeed");

        let patched_bytes = fs::read(wow_dir.join("DivxDecoder.dll")).unwrap();

        // File should have grown (new section appended)
        assert!(
            patched_bytes.len() > original_len,
            "File should grow by new section"
        );

        // Parse headers to find where section data starts
        let e_lfanew = read_u32_le(&patched_bytes, 0x3C) as usize;
        let coff = e_lfanew + 4;
        let _num_sections = read_u16_le(&patched_bytes, coff + 2) as usize;
        let size_opt = read_u16_le(&patched_bytes, coff + 16) as usize;
        let optional = coff + 20;
        let sec_table = optional + size_opt;

        // Find first section's raw offset
        let first_raw = read_u32_le(&patched_bytes, sec_table + 20);

        // Bytes before first section should be identical (DOS stub + headers are unchanged
        // except for the fields we deliberately mutate)
        // For this test, we verify that the section data itself is untouched
        let first_raw = first_raw as usize;
        assert_eq!(
            original_bytes[first_raw..original_len],
            patched_bytes[first_raw..original_len],
            "Existing section data must be preserved"
        );

        // Verify NumberOfSections increased
        assert_eq!(
            read_u16_le(&patched_bytes, coff + 2),
            read_u16_le(&original_bytes, coff + 2) + 1,
            "NumberOfSections should increase by 1"
        );
    }

    #[test]
    #[ignore = "Requires real WoW client"]
    fn test_real_divxdecoder_patching() {
        use std::path::Path;

        let wow_dir = Path::new("/Users/mykino/Documents/ChromieCraft_3.3.5a");
        if !wow_dir.exists() {
            return; // Skip if no real client
        }

        // Remove any existing backup
        let _ = fs::remove_file(wow_dir.join("DivxDecoder.dll.bak"));

        // Patch
        patch_dll_imports(wow_dir, "DivxDecoder.dll", "mods/winerosetta.dll")
            .expect("patch failed");

        // Verify structure
        let patched = fs::read(wow_dir.join("DivxDecoder.dll")).unwrap();
        let original = fs::read(wow_dir.join("DivxDecoder.dll.bak")).unwrap();

        assert!(patched.len() > original.len(), "File should grow");

        // Parse headers
        let e_lfanew = read_u32_le(&patched, 0x3C) as usize;
        let coff = e_lfanew + 4;
        let num_sections = read_u16_le(&patched, coff + 2);
        let size_opt = read_u16_le(&patched, coff + 16) as usize;
        let optional = coff + 20;
        let sec_table = optional + size_opt;

        assert!(num_sections >= 1, "Should have sections");

        // Check .winep section exists
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

        // Verify original section data is preserved
        let first_raw = read_u32_le(&patched, sec_table + 20) as usize;
        assert_eq!(
            original[first_raw..],
            patched[first_raw..original.len()],
            "Section data must be preserved"
        );

        // Check relocation directory is unchanged
        let data_dir_off = optional + 96;
        let reloc_rva = read_u32_le(&patched, data_dir_off + 40);
        let original_reloc_rva = read_u32_le(&original, data_dir_off + 40);
        assert_eq!(reloc_rva, original_reloc_rva, "Reloc RVA must be unchanged");
    }
}
