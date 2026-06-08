//! Rosetta translation port
//!
//! Defines the contract for x87 to AArch64 translation.

use crate::adapters::errors::TranslationError;

/// Port for x87 to AArch64 translation
pub trait RosettaTranslationPort: Send + Sync {
    /// Translate x87 instruction bytes to AArch64
    fn translate_x87_instruction(&self, bytes: &[u8]) -> Result<Vec<u8>, TranslationError>;

    /// Get cached translation if available
    fn get_cached_translation(&self, key: u64) -> Option<Vec<u8>>;

    /// Store translation in cache
    fn cache_translation(&self, key: u64, bytes: Vec<u8>);
}
