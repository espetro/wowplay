use serde::Serialize;

use crate::error::CommandError;
use wow_silicon_core::setup::SetupOrchestrator;

/// Status of a runner for the frontend dropdown.
#[derive(Debug, Clone, Serialize)]
pub struct RunnerStatus {
    pub name: String,
    pub display_name: String,
    pub available: bool,
    pub path: Option<String>,
}

/// Checks all known runners and returns their availability status.
#[tauri::command]
pub async fn check_runners() -> Result<Vec<RunnerStatus>, CommandError> {
    let checks = SetupOrchestrator::check_all_runners();
    let statuses = checks
        .into_iter()
        .map(|check| RunnerStatus {
            name: check.name,
            display_name: check.display_name,
            available: check.available,
            path: check.path.map(|p| p.display().to_string()),
        })
        .collect();
    Ok(statuses)
}
