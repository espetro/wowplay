#![warn(missing_docs)]
#![warn(unsafe_code)]

//! WoW on Apple Silicon — core library.
//!
//! Provides ports (trait definitions) and adapters (implementations) for
//! running World of Warcraft 3.3.5a on Apple Silicon via rosettax87_jit +
//! winerosetta + CrossOver.

pub mod adapters;
pub mod integration;
pub mod ports;

pub use adapters::errors::{AdapterError, LaunchError, TranslationError};
pub use adapters::mock_wine_adapter::MockWineAdapter;
pub use adapters::rosettax87_jit_adapter::Rosettax87JitAdapter;
pub use ports::launcher::{RosettaLauncherPort, WowLauncherPort};
pub use ports::rosetta::RosettaTranslationPort;
pub use ports::silicon_patch::{HookStatus, HookTarget, ProfilingReport, SiliconPatchAdapter};
pub use ports::wine::WineIntegrationPort;
