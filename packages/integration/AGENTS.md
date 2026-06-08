# integration Package - Agent Guide

## Overview
Testing harness and validation framework for WoW on Apple Silicon.

## Your Workspace
```
packages/integration/
├── AGENTS.md          # This file
├── Cargo.toml        # Test dependencies
└── tests/
    ├── mre/          # Minimal Reproducible Examples
    │   ├── rosettax87_tests.rs
    │   └── winerosetta_tests.rs
    └── e2e/          # End-to-end tests (manual)
        ├── wow_launch_tests.rs
        └── gameplay_tests.rs
```

## Quick Start
1. Ensure external libs are installed
2. Run MRE tests: `cargo test --test mre_tests`
3. Run E2E tests: `cargo test --test e2e_tests -- --ignored`

## Key Principles
- **MRE tests are fast**: No WoW process required
- **E2E tests are manual**: Require WoW installation
- **Test boundaries**: Focus on FFI and integration points
- **Document setup**: What's needed to run each test

## Test Categories

### MRE Tests (Headless)
Fast tests that validate external libraries without WoW:
- Library loading and initialization
- FFI function calls
- Translation correctness
- Error handling

```rust
#[test]
fn test_rosettax87_fxch_translation() {
    let adapter = Rosettax87Adapter::new().unwrap();
    let fxch_bytes = vec![0xD9, 0xC9];
    let result = adapter.translate_x87_instruction(&fxch_bytes);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}
```

### E2E Tests (Manual)
Full WoW launch and gameplay validation:
- Game launch
- x87 translation activation
- Gameplay verification

```rust
#[test]
#[ignore] // Run manually
fn test_wow_launches_and_runs() {
    let launcher = CrossoverLauncher::new("~/Crossover/Bottles/WoW");
    let process = launcher.launch_wow().unwrap();
    assert!(process.has_x87_translation_active());
}
```

## Running Tests

### All Tests (Fast)
```bash
cargo test
```

### MRE Tests Only
```bash
cargo test --test mre_tests
```

### E2E Tests (Manual)
```bash
cargo test --test e2e_tests -- --ignored
```

### Specific Test
```bash
cargo test test_rosettax87_fxch_translation
```

## Test Setup

### Prerequisites
1. Install external libraries:
   ```bash
   brew install rosettax87_jit winerosetta
   ```

2. Ensure CrossOver is installed (for E2E tests)

3. Configure WoW bottle path (for E2E tests)

### Test Utilities

```rust
// tests/common/mod.rs
pub mod common;

pub fn get_rosettax87_adapter() -> Rosettax87Adapter {
    Rosettax87Adapter::new().expect("Failed to load rosettax87_jit")
}

pub fn get_test_bottle_path() -> PathBuf {
    std::env::var("WOW_BOTTLE_PATH")
        .unwrap_or_else(|_| "~/Crossover/Bottles/WoW".into())
        .into()
}
```

## Common Patterns

### Test Naming
Use descriptive names that explain what's being tested:
```rust
#[test]
fn test_fxch_instruction_translates_to_valid_aarch64() { }
```

### Test Organization
Group related tests:
```rust
mod rosettax87_translation_tests {
    use super::*;

    #[test]
    fn test_fxch_translation() { }
    
    #[test]
    fn test_fadd_translation() { }
}
```

### Error Test Patterns
```rust
#[test]
fn test_invalid_instruction_returns_error() {
    let adapter = get_adapter();
    let result = adapter.translate(&[0xFF, 0xFF]);
    assert!(result.is_err());
}

#[test]
fn test_empty_input_returns_error() {
    let adapter = get_adapter();
    let result = adapter.translate(&[]);
    assert!(result.is_err());
}
```

## CI/CD Integration

### Pre-Commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run MRE tests
cargo test --test mre_tests

# Skip E2E tests (manual only)
```

### GitHub Actions
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: brew install rosettax87_jit winerosetta
      - name: Run MRE tests
        run: cargo test --test mre_tests
```

## Debugging Tests

### Enable Output
```bash
cargo test -- --nocapture
```

### Filter Tests
```bash
cargo test rosettax87
```

### Debug with LLDB
```bash
rust-lldb -- cargo test test_name
```

## See Also
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Testing Strategy](../../docs/testing-strategy.md) - Detailed testing approach
