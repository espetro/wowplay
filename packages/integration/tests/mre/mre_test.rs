use wow_silicon_core::adapters::errors::{AdapterError, LaunchError, TranslationError};
use wow_silicon_core::adapters::rosettax87_jit_adapter::Rosettax87JitAdapter;
use wow_silicon_core::integration::crossover;
use wow_silicon_core::ports::launcher::RosettaLauncherPort;
use wow_silicon_core::ports::rosetta::RosettaTranslationPort;

mod hook_injection_tests;
mod x87_corpus;

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

#[test]
fn test_x87_corpus_size() {
    assert!(
        x87_corpus::corpus_size() >= 30,
        "Corpus should have at least 30 instructions, got {}",
        x87_corpus::corpus_size()
    );
}

#[test]
fn test_x87_corpus_all_have_bytes() {
    for i in 0..x87_corpus::corpus_size() {
        let inst = x87_corpus::get_instruction(i).unwrap();
        assert!(
            !inst.bytes.is_empty(),
            "Instruction {} ({}) has no bytes",
            i,
            inst.description
        );
    }
}

#[test]
fn test_x87_corpus_categories_present() {
    let arithmetic = x87_corpus::get_by_result_type("addition");
    assert!(
        !arithmetic.is_empty(),
        "Corpus should contain addition instructions"
    );

    let trig = x87_corpus::get_by_result_type("trigonometric");
    assert!(
        !trig.is_empty(),
        "Corpus should contain trigonometric instructions"
    );

    let stack = x87_corpus::get_by_result_type("stack_exchange");
    assert!(
        !stack.is_empty(),
        "Corpus should contain stack exchange instructions"
    );
}

#[test]
fn test_translate_x87_corpus_no_panic() {
    let adapter_result = Rosettax87JitAdapter::discover();
    if adapter_result.is_err() {
        return;
    }
    let adapter = adapter_result.unwrap();

    for i in 0..x87_corpus::corpus_size() {
        let inst = x87_corpus::get_instruction(i).unwrap();
        let _result = adapter.translate_x87_instruction(inst.bytes);
    }
}
