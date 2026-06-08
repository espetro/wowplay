//! Wine/CrossOver integration port
//!
//! Defines the contract for Wine and CrossOver integration.

use crate::adapters::errors::WineError;

/// Process handle type
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
}

/// Port for Wine/CrossOver integration
pub trait WineIntegrationPort: Send + Sync {
    /// Get handle to Windows process by name
    fn get_process_handle(&self, name: &str) -> Result<ProcessHandle, WineError>;

    /// Inject dynamic library into process
    fn inject_dylib(&self, pid: u32, dylib_path: &str) -> Result<(), WineError>;

    /// Check if CrossOver/Wine is initialized
    fn is_initialized(&self) -> bool;
}
