#![warn(missing_docs)]
#![warn(unsafe_code)]

//! WoW on Apple Silicon — core library.
//!
//! Provides ports (trait definitions) and adapters (implementations) for
//! running World of Warcraft 3.3.5a on Apple Silicon via rosettax87_jit +
//! winerosetta + CrossOver.

pub mod adapters;
pub mod diagnostics;
pub mod integration;
pub mod ports;
pub mod resources;
pub mod runner_registry;
pub mod setup;

pub use adapters::crossover_adapter::CrossOverAdapter;
pub use adapters::errors::{AdapterError, LaunchError, TranslationError};
pub use adapters::mock_wine_adapter::MockWineAdapter;
pub use adapters::rosettax87_jit_adapter::Rosettax87JitAdapter;
pub use adapters::whisky_adapter::WhiskyAdapter;
pub use ports::launcher::{RosettaLauncherPort, WowLauncherPort};
pub use ports::rosetta::RosettaTranslationPort;
pub use ports::runner::RunnerPort;
pub use ports::silicon_patch::{HookStatus, HookTarget, ProfilingReport, SiliconPatchAdapter};
pub use ports::wine::WineIntegrationPort;
pub use runner_registry::RunnerRegistry;
