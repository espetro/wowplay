#![warn(missing_docs)]
#![warn(unsafe_code)]

//! WoW on Apple Silicon — core library.
//!
//! Provides ports (trait definitions) and adapters (implementations) for
//! running World of Warcraft 3.3.5a on Apple Silicon via rosettax87_jit +
//! winerosetta + CrossOver.

pub mod adapters;
pub mod commands;
pub mod config;
pub mod diagnostics;
pub mod integration;
pub mod ports;
pub mod reset;
pub mod resources;
pub mod runner_registry;
pub mod setup;

pub use adapters::crossover_adapter::CrossOverAdapter;
pub use adapters::errors::{AdapterError, LaunchError, TranslationError};
pub use adapters::mock_wine_adapter::MockWineAdapter;
pub use adapters::rosettax87_jit_adapter::Rosettax87JitAdapter;
pub use adapters::whisky_adapter::WhiskyAdapter;
pub use commands::config::{list_config, set_config, CONFIG_KEYS};
pub use commands::run::{run_wow, RunOverrides, WowSession};
pub use config::{AppConfig, ConfigStore, TomlConfigStore};
pub use ports::launcher::{RosettaLauncherPort, WowLauncherPort};
pub use ports::prompt::{PromptItem, PromptPort};
pub use ports::rosetta::RosettaTranslationPort;
pub use ports::runner::RunnerPort;
pub use ports::silicon_patch::{HookStatus, HookTarget, ProfilingReport, SiliconPatchAdapter};
pub use ports::wine::WineIntegrationPort;
pub use reset::ResetOrchestrator;
pub use runner_registry::RunnerRegistry;
