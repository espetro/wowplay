//! Mock Wine adapter for headless testing.
//!
//! Provides a mock implementation of `WineIntegrationPort` that simulates
//! a Wine process handle with a fake PE memory layout. Used by hook
//! injection tests to validate hooking mechanisms without a real process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::adapters::errors::WineError;
use crate::ports::wine::{ProcessHandle, WineIntegrationPort};

/// Mock PE memory region
#[derive(Debug, Clone)]
pub struct MockMemoryRegion {
    /// Base address
    pub base: u32,
    /// Size in bytes
    pub size: usize,
    /// Region name (e.g., ".text", ".data")
    pub name: String,
    /// Content (optional)
    pub content: Vec<u8>,
}

/// Mock Wine process with simulated memory layout
#[derive(Debug)]
pub struct MockWineAdapter {
    /// Simulated process handle
    pid: u32,
    /// Process name
    name: String,
    /// Memory regions (address -> region)
    memory: HashMap<u32, MockMemoryRegion>,
    /// Installed hooks (address -> hook bytes)
    hooks: Arc<Mutex<HashMap<u32, Vec<u8>>>>,
    /// Whether the adapter is "initialized"
    initialized: bool,
}

impl MockWineAdapter {
    /// Create a new mock adapter with a default WoW-like memory layout.
    pub fn new() -> Self {
        let mut memory = HashMap::new();

        // Simulate WoW.exe 32-bit PE memory layout
        memory.insert(
            0x00400000,
            MockMemoryRegion {
                base: 0x00400000,
                size: 0x00400000, // 4MB
                name: ".text".to_string(),
                content: vec![0x90; 0x00400000], // NOP sled
            },
        );

        memory.insert(
            0x00800000,
            MockMemoryRegion {
                base: 0x00800000,
                size: 0x00200000, // 2MB
                name: ".data".to_string(),
                content: vec![0x00; 0x00200000],
            },
        );

        Self {
            pid: 12345,
            name: "wine64-preloader".to_string(),
            memory,
            hooks: Arc::new(Mutex::new(HashMap::new())),
            initialized: true,
        }
    }

    /// Create a mock adapter with custom memory layout.
    pub fn with_memory(memory: HashMap<u32, MockMemoryRegion>) -> Self {
        Self {
            pid: 12345,
            name: "wine64-preloader".to_string(),
            memory,
            hooks: Arc::new(Mutex::new(HashMap::new())),
            initialized: true,
        }
    }

    /// Get the memory region containing an address.
    pub fn get_region(&self, addr: u32) -> Option<&MockMemoryRegion> {
        self.memory
            .values()
            .find(|region| (region.base..region.base + region.size as u32).contains(&addr))
    }

    /// Check if an address is within the mock WoW.exe range.
    pub fn is_wow_address(&self, addr: u32) -> bool {
        (0x00400000..0x00800000).contains(&addr)
    }

    /// Install a mock hook at the given address.
    pub fn install_hook(&self, addr: u32, hook_bytes: Vec<u8>) -> Result<(), WineError> {
        if !self.is_wow_address(addr) {
            return Err(WineError::ProcessNotFound(format!(
                "Address 0x{:08X} outside WoW range",
                addr
            )));
        }

        let mut hooks = self.hooks.lock().unwrap();
        hooks.insert(addr, hook_bytes);
        Ok(())
    }

    /// Remove a mock hook.
    pub fn remove_hook(&self, addr: u32) -> Result<(), WineError> {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.remove(&addr);
        Ok(())
    }

    /// Get all installed hooks.
    pub fn get_hooks(&self) -> HashMap<u32, Vec<u8>> {
        self.hooks.lock().unwrap().clone()
    }

    /// Simulate reading memory at an address.
    pub fn read_memory(&self, addr: u32, size: usize) -> Result<Vec<u8>, WineError> {
        let region = self.get_region(addr).ok_or_else(|| {
            WineError::ProcessNotFound(format!("Address 0x{:08X} not mapped", addr))
        })?;

        let offset = (addr - region.base) as usize;
        let end = (offset + size).min(region.content.len());

        Ok(region.content[offset..end].to_vec())
    }

    /// Simulate writing memory at an address.
    pub fn write_memory(&self, addr: u32, data: &[u8]) -> Result<(), WineError> {
        let region = self.get_region(addr).ok_or_else(|| {
            WineError::ProcessNotFound(format!("Address 0x{:08X} not mapped", addr))
        })?;

        let offset = (addr - region.base) as usize;
        let _end = (offset + data.len()).min(region.content.len());

        // In a real implementation, this would write to process memory
        // For mock, we just verify the address is valid
        Ok(())
    }
}

impl WineIntegrationPort for MockWineAdapter {
    fn get_process_handle(&self, name: &str) -> Result<ProcessHandle, WineError> {
        if name.to_lowercase().contains("wow") || name.to_lowercase().contains("wine") {
            Ok(ProcessHandle {
                pid: self.pid,
                name: self.name.clone(),
            })
        } else {
            Err(WineError::ProcessNotFound(name.to_string()))
        }
    }

    fn inject_dylib(&self, pid: u32, dylib_path: &str) -> Result<(), WineError> {
        if pid != self.pid {
            return Err(WineError::ProcessNotFound(format!(
                "PID {} does not match mock process {}",
                pid, self.pid
            )));
        }

        if !dylib_path.ends_with(".dylib") && !dylib_path.ends_with(".dll") {
            return Err(WineError::InjectionFailed(
                "Invalid library extension".to_string(),
            ));
        }

        // Mock injection always succeeds
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for MockWineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_adapter_creation() {
        let adapter = MockWineAdapter::new();
        assert!(adapter.is_initialized());
        assert_eq!(adapter.pid, 12345);
    }

    #[test]
    fn test_get_process_handle() {
        let adapter = MockWineAdapter::new();
        let handle = adapter.get_process_handle("WoW.exe").unwrap();
        assert_eq!(handle.pid, 12345);
    }

    #[test]
    fn test_get_process_handle_not_found() {
        let adapter = MockWineAdapter::new();
        let result = adapter.get_process_handle("notepad.exe");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_wow_address() {
        let adapter = MockWineAdapter::new();
        assert!(adapter.is_wow_address(0x00401000));
        assert!(adapter.is_wow_address(0x006A3F20));
        assert!(!adapter.is_wow_address(0x00001000));
        assert!(!adapter.is_wow_address(0x7FFF0000));
    }

    #[test]
    fn test_install_and_remove_hook() {
        let adapter = MockWineAdapter::new();
        let addr = 0x00401000;
        let hook = vec![0x90, 0x90, 0x90]; // NOP sled

        adapter.install_hook(addr, hook.clone()).unwrap();
        let hooks = adapter.get_hooks();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks.get(&addr), Some(&hook));

        adapter.remove_hook(addr).unwrap();
        let hooks = adapter.get_hooks();
        assert!(hooks.is_empty());
    }

    #[test]
    fn test_install_hook_outside_range() {
        let adapter = MockWineAdapter::new();
        let result = adapter.install_hook(0x7FFF0000, vec![0x90]);
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_dylib() {
        let adapter = MockWineAdapter::new();
        assert!(adapter.inject_dylib(12345, "test.dylib").is_ok());
        assert!(adapter.inject_dylib(99999, "test.dylib").is_err());
        assert!(adapter.inject_dylib(12345, "test.txt").is_err());
    }
}
