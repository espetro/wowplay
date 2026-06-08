//! Error types for all adapter and launcher operations.

use thiserror::Error;

/// Common adapter errors
#[derive(Error, Debug)]
pub enum AdapterError {
    /// Library file not found at specified path
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
    /// Required function not found in library
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    /// Function call returned a failure code
    #[error("Call failed: {0}")]
    CallFailed(String),
}

/// Translation-specific errors
#[derive(Error, Debug)]
pub enum TranslationError {
    /// Translation operation failed
    #[error("Translation failed: {0}")]
    TranslationFailed(String),
    /// Input bytes are not valid x87 instructions
    #[error("Invalid instruction bytes")]
    InvalidInstruction,
    /// Underlying adapter error
    #[error("Adapter error: {0}")]
    AdapterError(#[from] AdapterError),
}

/// Wine/CrossOver specific errors
#[derive(Error, Debug)]
pub enum WineError {
    /// Named process not found in Wine session
    #[error("Process not found: {0}")]
    ProcessNotFound(String),
    /// DLL injection into process failed
    #[error("Injection failed: {0}")]
    InjectionFailed(String),
    /// CrossOver/Wine environment not initialized
    #[error("CrossOver not initialized")]
    NotInitialized,
    /// Underlying adapter error
    #[error("Adapter error: {0}")]
    AdapterError(#[from] AdapterError),
}

/// Errors from the WoW launcher pipeline
#[derive(Error, Debug)]
pub enum LaunchError {
    /// `runtime_loader` binary not found at expected path
    #[error("Runtime loader not found: {0}")]
    RuntimeNotFound(String),
    /// CrossOver.app not found on this machine
    #[error("CrossOver not found at: {0}")]
    CrossoverNotFound(String),
    /// WoW client directory does not exist
    #[error("WoW directory not found: {0}")]
    WowDirNotFound(String),
    /// A setup step (copy, codesign, etc.) failed
    #[error("Setup failed: {0}")]
    SetupFailed(String),
    /// OS-level process spawn failed
    #[error("Spawn failed: {0}")]
    SpawnFailed(#[from] std::io::Error),
    /// `codesign --remove-signature` failed
    #[error("Codesign strip failed: {0}")]
    CodesignFailed(String),
}
