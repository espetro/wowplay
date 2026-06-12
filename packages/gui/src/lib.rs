pub mod commands;
pub mod error;
pub mod logging;
pub mod state;

/// Generates TypeScript bindings from Rust types into `src-frontend/gen/bindings.ts`.
///
/// Run via: `cargo test --manifest-path packages/gui/Cargo.toml export_bindings`
#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        use crate::commands;
        let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(
            tauri_specta::collect_commands![
                commands::config::get_config,
                commands::config::set_config,
                commands::config::reset_config,
                commands::diagnostics::check_runners,
                commands::setup::run_setup,
                commands::setup::validate_wow_dir,
                commands::launch::launch_wow,
            ],
        );
        builder
            .export(
                specta_typescript::Typescript::default(),
                "src-frontend/gen/bindings.ts",
            )
            .expect("failed to export TypeScript bindings");
    }
}
