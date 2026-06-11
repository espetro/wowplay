//! Error types for PE import patching.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during PE import table patching.
#[derive(Error, Debug)]
pub enum PatchError {
    /// Failed to read PE file.
    #[error("PE read error: {0}")]
    PeRead(String),

    /// Failed to write PE file.
    #[error("PE write error: {0}")]
    PeWrite(String),

    /// The target DLL was not found.
    #[error("DLL not found: {0}")]
    DllNotFound(PathBuf),

    /// The file is not a valid PE or has an unsupported PE format.
    #[error("Invalid PE file: {0}")]
    InvalidPe(String),

    /// Failed to create backup of the original DLL.
    #[error("Backup failed: {0}")]
    BackupFailed(String),

    /// Failed to restore DLL from backup.
    #[error("Restore failed: {0}")]
    RestoreFailed(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type specialized for patch operations.
pub type PatchResult<T> = Result<T, PatchError>;
