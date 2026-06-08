# Testing Strategy: MRE Harness and Validation

## Overview
We use a layered testing approach with **Minimal Reproducible Examples (MRE)** at the core. This enables fast, headless validation of critical functionality without requiring WoW to be running.

## Testing Philosophy

### Test Early, Test Often
- Write tests alongside code (test-first when possible)
- Run MRE tests on every commit
- Validate critical paths with headless tests

### Test at Boundaries
- Test Port implementations
- Test Adapter FFI calls
- Test Domain logic invariants

### Test Hierarchically
1. **Unit tests**: Fast, isolated, test pure functions
2. **MRE tests**: Headless integration tests of external libraries
3. **E2E tests**: Full WoW launch and gameplay (manual)

## Test Layers

### Layer 1: Unit Tests (Rust)
Test pure functions and domain logic:

```rust
#[test]
fn test_x87_instruction_parsing() {
    let bytes = vec![0xD9, 0xC9]; // FXCH ST(1)
    let instruction = X87Instruction::parse(&bytes).unwrap();
    assert_eq!(instruction.opcode, OpCode::FXCH);
    assert_eq!(instruction.operands[0], Operand::ST(1));
}
```

**Run:** `cargo test --lib`

**Speed:** < 1 second

### Layer 2: MRE Tests (Minimal Reproducible Examples)
Headless integration tests of external libraries:

```rust
#[test]
fn test_rosettax87_fxch_translation() {
    // Arrange: Load adapter
    let adapter = Rosettax87Adapter::new().unwrap();
    let fxch_bytes = vec![0xD9, 0xC9]; // FXCH ST(1)
    
    // Act: Translate
    let result = adapter.translate_x87_instruction(&fxch_bytes);
    
    // Assert: Valid AArch64 output
    assert!(result.is_ok());
    let aarch64_bytes = result.unwrap();
    assert!(!aarch64_bytes.is_empty());
    
    // Verify: Contains expected instruction
    assert!(aarch64_bytes.contains(&0x1A)); // Some AArch64 opcode
}
```

**Run:** `cargo test --test mre_tests`

**Speed:** 1-5 seconds

**Why MRE?**
- Tests real external library behavior
- No WoW process required
- Fast feedback on integration
- Catches FFI boundary issues

### Layer 3: End-to-End Tests (Manual)
Full WoW launch and gameplay:

```rust
#[test]
#[ignore] // Run manually with actual WoW installation
fn test_wow_launches_and_runs() {
    // 1. Setup CrossOver bottle with WoW 3.3.5a
    // 2. Configure rosettax87_jit
    // 3. Launch game
    // 4. Monitor for x87 instruction translation
    // 5. Verify game is playable (can move, interact)
    
    let launcher = CrossoverLauncher::new("~/Crossover/Bottles/WoW");
    let process = launcher.launch_wow().unwrap();
    
    // Monitor x87 translation
    assert!(process.has_x87_translation_active());
    
    // Verify gameplay
    assert!(process.can_move_character());
    assert!(process.can_interact_with_npc());
}
```

**Run:** `cargo test --test e2e_tests -- --ignored`

**Speed:** 30-60 seconds (requires WoW process)

## Test Organization

```
packages/
├── rust-core/
│   ├── src/
│   │   ├── ports/
│   │   │   ├── rosetta_port.rs
│   │   │   └── wine_port.rs
│   │   ├── adapters/
│   │   │   ├── rosettax87_adapter.rs
│   │   │   └── winerosetta_adapter.rs
│   │   └── domain/
│   │       └── x87_translator.rs
│   └── tests/                    # Unit tests
│       ├── rosetta_port_tests.rs
│       └── adapter_tests.rs
│
└── integration/
    └── tests/
        ├── mre/                 # MRE tests
        │   ├── rosettax87_tests.rs
        │   └── winerosetta_tests.rs
        └── e2e/                # E2E tests (manual)
            ├── wow_launch_tests.rs
            └── gameplay_tests.rs
```

## MRE Test Categories

### 1. FFI Boundary Tests
Test that external libraries load and respond:

```rust
#[test]
fn test_rosettax87_library_loads() {
    let library = unsafe {
        libloading::Library::new("/opt/homebrew/lib/librosettax87_jit.dylib")
    };
    assert!(library.is_ok());
}

#[test]
fn test_rosettax87_function_callable() {
    let adapter = Rosettax87Adapter::new().unwrap();
    // Ensure we can call the translate function
    assert!(adapter.translate_x87_instruction(&[0x90]).is_ok());
}
```

### 2. Translation Correctness Tests
Validate instruction-by-instruction:

```rust
#[test]
fn test_fxch_translation() {
    test_x87_instruction([0xD9, 0xC9], "FXCH ST(1)");
}

#[test]
fn test_fadd_translation() {
    test_x87_instruction([0xD8, 0xC1], "FADD ST(1)");
}

#[test]
fn test_fstp_translation() {
    test_x87_instruction([0xDD, 0xD9], "FSTP ST(1)");
}
```

### 3. Error Handling Tests
Test failure modes:

```rust
#[test]
fn test_invalid_x87_instruction_returns_error() {
    let adapter = Rosettax87Adapter::new().unwrap();
    let invalid = vec![0xFF, 0xFF];
    let result = adapter.translate_x87_instruction(&invalid);
    assert!(result.is_err());
}

#[test]
fn test_empty_input_returns_error() {
    let adapter = Rosettax87Adapter::new().unwrap();
    let result = adapter.translate_x87_instruction(&[]);
    assert!(result.is_err());
}
```

### 4. Caching Tests
Test performance optimizations:

```rust
#[test]
fn test_translation_caching() {
    let adapter = Rosettax87Adapter::new().unwrap();
    let instruction = vec![0xD9, 0xC9];
    
    // First call: cache miss
    let start = Instant::now();
    let _ = adapter.translate_x87_instruction(&instruction).unwrap();
    let first_duration = start.elapsed();
    
    // Second call: cache hit (should be faster)
    let start = Instant::now();
    let _ = adapter.translate_x87_instruction(&instruction).unwrap();
    let second_duration = start.elapsed();
    
    assert!(second_duration < first_duration);
}
```

## Running Tests

### All Tests (Fast)
```bash
# Run all tests except ignored E2E
cargo test
```

### MRE Tests Only
```bash
# Run just MRE tests
cargo test --test mre_tests
```

### Specific Test
```bash
# Run specific test
cargo test test_rosettax87_fxch_translation
```

### E2E Tests (Manual)
```bash
# Requires WoW installation
cargo test --test e2e_tests -- --ignored
```

## CI/CD Integration

### Pre-Commit Hook
```bash
#!/bin/bash
# scripts/pre-commit.sh

# Run unit tests
cargo test --lib

# Run MRE tests
cargo test --test mre_tests

# Run formatting/linting
cargo fmt --check
cargo clippy -- -D warnings
```

### CI Pipeline
```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/coverage@v1
      - name: Run unit tests
        run: cargo test --lib
      - name: Run MRE tests
        run: cargo test --test mre_tests
      - name: Check formatting
        run: cargo fmt --check
```

## Coverage Goals

### Phase 0 (Foundation)
- [ ] Pre-commit hooks working
- [ ] Test infrastructure in place

### Phase 1 (Architecture)
- [ ] All Ports have trait tests
- [ ] All Adapters have MRE tests
- [ ] FFI boundaries covered

### Phase 2 (POC)
- [ ] End-to-end WoW launch test passes
- [ ] x87 translation validated
- [ ] Performance benchmarks pass

### Phase 3 (Enhancement)
- [ ] >80% code coverage
- [ ] All features tested
- [ ] Regression tests in place

## Debugging Tests

### Enable Test Output
```bash
# Show stdout for tests
cargo test -- --nocapture

# Show test execution logs
RUST_LOG=debug cargo test
```

### Run Single Test
```bash
# Run just one test
cargo test test_rosettax87_fxch_translation -- --exact
```

### Debug with LLDB
```bash
# Debug test interactively
rust-lldb -- cargo test test_rosettax87_fxch_translation
```

## Best Practices

### DO ✅
- Write tests alongside code
- Test error cases
- Use descriptive test names
- Keep tests fast (prefer MRE over E2E)
- Use test helpers to reduce duplication

### DON'T ❌
- Don't skip writing tests
- Don't write slow tests in hot paths
- Don't test external libraries (test our usage of them)
- Don't ignore flaky tests
- Don't commit broken tests

## See Also
- [Architecture](architecture.md) - Port and adapter design
- [Git Workflow](git-workflow.md) - Commit format for tests
