//! Mock hook injection tests.
//!
//! Validates the hooking mechanism using the mock Wine adapter.
//! These tests run without a real WoW process.

use wow_silicon_core::adapters::errors::WineError;
use wow_silicon_core::adapters::mock_wine_adapter::MockWineAdapter;
use wow_silicon_core::ports::wine::WineIntegrationPort;

#[test]
fn test_mock_wine_adapter_initialization() {
    let adapter = MockWineAdapter::new();
    assert!(adapter.is_initialized());
}

#[test]
fn test_mock_process_discovery() {
    let adapter = MockWineAdapter::new();

    // Should find WoW.exe
    let handle = adapter.get_process_handle("WoW.exe").unwrap();
    assert_eq!(handle.pid, 12345);
    assert_eq!(handle.name, "wine64-preloader");

    // Should find wine processes
    let handle = adapter.get_process_handle("wine64-preloader").unwrap();
    assert_eq!(handle.pid, 12345);
}

#[test]
fn test_mock_process_not_found() {
    let adapter = MockWineAdapter::new();

    let result = adapter.get_process_handle("notepad.exe");
    assert!(result.is_err());

    match result.unwrap_err() {
        WineError::ProcessNotFound(name) => {
            assert_eq!(name, "notepad.exe");
        }
        other => panic!("Expected ProcessNotFound, got {:?}", other),
    }
}

#[test]
fn test_mock_dylib_injection() {
    let adapter = MockWineAdapter::new();

    // Valid injection
    assert!(adapter.inject_dylib(12345, "libtest.dylib").is_ok());
    assert!(adapter.inject_dylib(12345, "winerosetta.dll").is_ok());

    // Wrong PID
    assert!(adapter.inject_dylib(99999, "libtest.dylib").is_err());

    // Invalid extension
    assert!(adapter.inject_dylib(12345, "test.txt").is_err());
}

#[test]
fn test_mock_address_filtering() {
    let adapter = MockWineAdapter::new();

    // WoW.exe range (0x00400000 - 0x00800000)
    assert!(adapter.is_wow_address(0x00401000));
    assert!(adapter.is_wow_address(0x006A3F20));
    assert!(adapter.is_wow_address(0x007FFFFF));

    // Outside range
    assert!(!adapter.is_wow_address(0x00001000));
    assert!(!adapter.is_wow_address(0x7FFF0000));
    assert!(!adapter.is_wow_address(0x10000000));
}

#[test]
fn test_mock_hook_installation() {
    let adapter = MockWineAdapter::new();
    let addr = 0x00401000;
    let hook_bytes = vec![0x90, 0x90, 0x90, 0x90, 0x90]; // 5 NOPs

    // Install hook
    adapter.install_hook(addr, hook_bytes.clone()).unwrap();

    // Verify hook is present
    let hooks = adapter.get_hooks();
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks.get(&addr), Some(&hook_bytes));
}

#[test]
fn test_mock_hook_outside_range_fails() {
    let adapter = MockWineAdapter::new();

    // Address outside WoW range
    let result = adapter.install_hook(0x7FFF0000, vec![0x90]);
    assert!(result.is_err());
}

#[test]
fn test_mock_hook_removal() {
    let adapter = MockWineAdapter::new();
    let addr = 0x00401000;
    let hook_bytes = vec![0x90, 0x90, 0x90];

    // Install and then remove
    adapter.install_hook(addr, hook_bytes).unwrap();
    adapter.remove_hook(addr).unwrap();

    // Verify hook is gone
    let hooks = adapter.get_hooks();
    assert!(hooks.is_empty());
}

#[test]
fn test_mock_multiple_hooks() {
    let adapter = MockWineAdapter::new();

    let hooks_to_install = vec![
        (0x00401000, vec![0x90, 0x90]),
        (0x00402000, vec![0xCC, 0x90]), // INT3 + NOP
        (0x00403000, vec![0xEB, 0x00]), // JMP +0
    ];

    for (addr, bytes) in &hooks_to_install {
        adapter.install_hook(*addr, bytes.clone()).unwrap();
    }

    let hooks = adapter.get_hooks();
    assert_eq!(hooks.len(), 3);

    for (addr, bytes) in &hooks_to_install {
        assert_eq!(hooks.get(addr), Some(bytes));
    }
}

#[test]
fn test_mock_memory_read() {
    let adapter = MockWineAdapter::new();

    // Read from valid address
    let data = adapter.read_memory(0x00400000, 16).unwrap();
    assert_eq!(data.len(), 16);
    assert!(data.iter().all(|b| *b == 0x90)); // NOP sled

    // Read from unmapped address
    let result = adapter.read_memory(0x10000000, 16);
    assert!(result.is_err());
}

#[test]
fn test_mock_memory_write() {
    let adapter = MockWineAdapter::new();

    // Write to valid address (mock just validates, doesn't actually write)
    assert!(adapter.write_memory(0x00400000, &[0xCC, 0x90]).is_ok());

    // Write to unmapped address
    let result = adapter.write_memory(0x10000000, &[0x90]);
    assert!(result.is_err());
}

#[test]
fn test_mock_hook_and_verify() {
    let adapter = MockWineAdapter::new();
    let test_addr = 0x006A3F20;
    let hook_bytes = vec![0xE9, 0x00, 0x10, 0x00, 0x00]; // JMP rel32

    // Install hook
    adapter.install_hook(test_addr, hook_bytes.clone()).unwrap();

    // Verify hook exists
    let hooks = adapter.get_hooks();
    assert!(hooks.contains_key(&test_addr));

    // Verify it's the right bytes
    assert_eq!(hooks.get(&test_addr), Some(&hook_bytes));

    // Remove and verify gone
    adapter.remove_hook(test_addr).unwrap();
    let hooks = adapter.get_hooks();
    assert!(!hooks.contains_key(&test_addr));
}

#[test]
fn test_wine_integration_port_trait() {
    // Verify MockWineAdapter implements WineIntegrationPort
    let adapter: Box<dyn WineIntegrationPort> = Box::new(MockWineAdapter::new());

    assert!(adapter.is_initialized());

    let handle = adapter.get_process_handle("WoW.exe").unwrap();
    assert_eq!(handle.pid, 12345);

    assert!(adapter.inject_dylib(12345, "test.dylib").is_ok());
}
