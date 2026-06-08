use wow_silicon_core::adapters::errors::{AdapterError, LaunchError, TranslationError};
use wow_silicon_core::adapters::rosettax87_jit_adapter::Rosettax87JitAdapter;
use wow_silicon_core::integration::crossover;
use wow_silicon_core::ports::launcher::RosettaLauncherPort;

#[test]
fn test_core_library_loads() {
    let _: Option<AdapterError> = None;
    let _: Option<TranslationError> = None;
    let _: Option<LaunchError> = None;
}

#[test]
fn test_translation_error_display() {
    let err = TranslationError::InvalidInstruction;
    assert_eq!(err.to_string(), "Invalid instruction bytes");
}

#[test]
fn test_adapter_error_display() {
    let err = AdapterError::LibraryNotFound("librosettax87_jit.dylib".into());
    assert!(err.to_string().contains("librosettax87_jit.dylib"));
}

#[test]
fn test_launch_error_runtime_not_found() {
    let err = Rosettax87JitAdapter::new("/nonexistent/runtime_loader".into());
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("/nonexistent/runtime_loader"));
}

#[test]
fn test_crossover_finder() {
    // CrossOver is installed on this machine; verify we can locate it
    let result = crossover::find_crossover();
    assert!(result.is_ok(), "CrossOver not found: {:?}", result.err());
    let path = result.unwrap();
    assert!(
        path.exists(),
        "CrossOver path does not exist: {}",
        path.display()
    );
}

#[test]
fn test_wineloader_exists() {
    let crossover_path = crossover::find_crossover().expect("CrossOver not installed");
    let loader = crossover::wineloader_path(&crossover_path);
    assert!(
        loader.exists(),
        "wineloader not found at {}",
        loader.display()
    );
}

// Requires runtime_loader to be built (run scripts/setup.sh first)
#[test]
#[ignore]
fn test_rosettax87_jit_adapter_discover() {
    let adapter = Rosettax87JitAdapter::discover().expect("runtime_loader not built");
    assert!(adapter.is_available());
    println!("runtime_loader at: {}", adapter.runtime_path().display());
}
