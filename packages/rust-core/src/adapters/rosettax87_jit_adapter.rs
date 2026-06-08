//! Adapter wrapping the rosettax87_jit `runtime_loader` executable as a subprocess.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::adapters::errors::{LaunchError, TranslationError};
use crate::ports::launcher::RosettaLauncherPort;
use crate::ports::rosetta::RosettaTranslationPort;

/// Adapter wrapping the rosettax87_jit `runtime_loader` executable.
///
/// Usage: `runtime_loader <program> [args...]`
/// The runtime_loader hooks into Rosetta 2 and replaces x87 handlers with
/// faster JIT-compiled AArch64 equivalents before exec-ing into the target.
#[derive(Debug)]
pub struct Rosettax87JitAdapter {
    runtime_loader: PathBuf,
}

impl Rosettax87JitAdapter {
    /// Create adapter pointing at the built `runtime_loader` binary.
    pub fn new(runtime_loader: PathBuf) -> Result<Self, LaunchError> {
        if !runtime_loader.exists() {
            return Err(LaunchError::RuntimeNotFound(
                runtime_loader.display().to_string(),
            ));
        }
        Ok(Self { runtime_loader })
    }

    /// Discover the binary from common build output locations.
    pub fn discover() -> Result<Self, LaunchError> {
        let candidates = [
            // Installed by setup.sh
            PathBuf::from("/usr/local/bin/runtime_loader"),
            // Project-local build (rosettax87_jit)
            PathBuf::from("vendor/rosettax87_jit/build/bin/runtime_loader"),
            // Absolute fallback for dev builds
            PathBuf::from("/tmp/rosettax87_jit/build/bin/runtime_loader"),
        ];
        for path in &candidates {
            if path.exists() {
                return Self::new(path.clone());
            }
        }
        Err(LaunchError::RuntimeNotFound(
            "runtime_loader not found; run scripts/setup.sh to build rosettax87_jit".into(),
        ))
    }
}

impl RosettaLauncherPort for Rosettax87JitAdapter {
    fn launch(&self, program: &Path, args: &[&str]) -> Result<Child, LaunchError> {
        Command::new(&self.runtime_loader)
            .arg(program)
            .args(args)
            .spawn()
            .map_err(LaunchError::SpawnFailed)
    }

    fn is_available(&self) -> bool {
        self.runtime_loader.exists()
    }

    fn runtime_path(&self) -> &Path {
        &self.runtime_loader
    }
}

impl RosettaTranslationPort for Rosettax87JitAdapter {
    fn translate_x87_instruction(&self, bytes: &[u8]) -> Result<Vec<u8>, TranslationError> {
        if bytes.is_empty() {
            return Err(TranslationError::InvalidInstruction);
        }
        Ok(bytes.to_vec())
    }

    fn get_cached_translation(&self, _key: u64) -> Option<Vec<u8>> {
        None
    }

    fn cache_translation(&self, _key: u64, _bytes: Vec<u8>) {}
}
