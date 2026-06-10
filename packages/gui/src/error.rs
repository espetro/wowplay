//! Error types for the Tauri GUI backend.

use serde::Serialize;

/// A serializable error for Tauri commands.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for CommandError {
    fn from(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: format!("IO error: {err}"),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
