//! Prompt port — abstraction over interactive user input.
//!
//! The CLI implements this with `dialoguer`; a GUI can provide its own adapter.

use std::path::PathBuf;

use crate::adapters::errors::LaunchError;

/// Abstraction over interactive user input.
///
/// The CLI implements this with `dialoguer`; a GUI can provide its own adapter.
pub trait PromptPort: Send + Sync {
    /// Ask the user to pick exactly one item. Returns the selected index.
    fn select_one(
        &self,
        prompt: &str,
        items: &[PromptItem],
        default: Option<usize>,
    ) -> Result<usize, LaunchError>;

    /// Ask the user to pick zero or more items. Returns selected indices.
    fn select_many(
        &self,
        prompt: &str,
        items: &[PromptItem],
        defaults: &[bool],
    ) -> Result<Vec<usize>, LaunchError>;

    /// Ask the user for a path. Validates existence when `must_exist` is true.
    fn input_path(
        &self,
        prompt: &str,
        must_exist: bool,
        default: Option<&std::path::Path>,
    ) -> Result<PathBuf, LaunchError>;
}

/// A selectable item in a prompt.
#[derive(Debug, Clone)]
pub struct PromptItem {
    /// The main label shown to the user.
    pub label: String,
    /// Optional detailed description shown alongside the label.
    pub detail: Option<String>,
}

impl PromptItem {
    /// Creates a new PromptItem with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
        }
    }

    /// Adds a detail string to this item.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}
