# Architecture: Ports & Adapters

## Overview
This project uses the Ports & Adapters (Hexagonal) architecture to create a composable, testable system for playing WoW 3.3.5a on Apple Silicon.

## Core Principles

### Dependency Rule
Dependencies point INWARD: External libraries → Adapters → Ports → Domain Logic.

This means:
- Domain logic knows nothing about external libraries
- Adapters implement Ports to interface with external libraries
- We can swap implementations without touching core logic

### Build UPON, Don't Replace
We CALL existing libraries, we don't modify them:
- **rosettax87_jit**: Loaded via FFI, wrapped in adapter
- **winerosetta**: Loaded via FFI, wrapped in adapter
- **CrossOver**: Integrated via macOS APIs, wrapped in adapter

## Architecture Layers

### 1. Domain Layer (Rust Core)
Contains business logic and Port definitions:

```rust
// Ports define the contract
pub trait RosettaTranslationPort: Send + Sync {
    fn translate_x87_instruction(&self, bytes: &[u8]) -> Result<Vec<u8>, TranslationError>;
}

// Domain logic uses ports
pub struct X87Translator {
    rosetta: Arc<dyn RosettaTranslationPort>,
}
```

### 2. Adapter Layer
Wraps external libraries to implement Ports:

```rust
pub struct Rosettax87Adapter {
    library: libloading::Library,
}

impl RosettaTranslationPort for Rosettax87Adapter {
    fn translate_x87_instruction(&self, bytes: &[u8]) -> Result<Vec<u8>, TranslationError> {
        unsafe {
            let func = self.library.get("rosettax87_translate")?;
            // Call external library
        }
    }
}
```

### 3. Integration Layer
Launches WoW, manages process, injects monitoring:

```rust
pub struct CrossoverLauncher {
    bottle_path: PathBuf,
    wine_port: Arc<dyn WineIntegrationPort>,
    rosetta_port: Arc<dyn RosettaTranslationPort>,
}
```

## Benefits

### Testability
- Mock ports for unit tests
- Real adapters for integration tests
- Headless MRE tests require no running WoW

### Composability
- Mix and match library implementations
- Swap adapters without changing domain logic
- Progressive replacement of closed-source components

### Agent-First Development
- Clear boundaries between layers
- Each layer has its own AGENTS.md
- Atomic changes within well-defined scopes

## Port Definitions

### RosettaTranslationPort
Interface for x87 to AArch64 translation:
- `translate_x87_instruction`: Translate individual x87 instructions
- `get_cached_translation`: Retrieve cached translations
- `cache_translation`: Store translation results

### WineIntegrationPort
Interface for Wine/CrossOver process management:
- `get_process_handle`: Get handle to Windows process
- `inject_dylib`: Inject dynamic library into process
- `is_initialized`: Check CrossOver state

### GraphicsTranslationPort
Interface for graphics translation (d9vk, Metal):
- `enable_dxvk_translation`: Activate DXVK for DirectX 9
- `get_graphics_backend`: Query current graphics backend

## Adapter Implementations

### Rosettax87Adapter
Wraps rosettax87_jit for x87 instruction translation.

### WinerosettaAdapter
Wraps winerosetta for Wine/CrossOver integration.

### Future: SiliconPatchAdapter
Open-source replacement for closed-source libSiliconPatch.

## Data Flow

```
WoW.exe (Windows)
    ↓
CrossOver (Windows API translation)
    ↓
winerosetta (Rosetta 2 injection)
    ↓
rosettax87_jit (x87 → AArch64 translation)
    ↓
Native macOS execution
```

## Testing Strategy

### MRE (Minimal Reproducible Example) Tests
Headless tests that validate translation without running WoW:
```rust
#[test]
fn test_rosettax87_fxch_translation() {
    let adapter = Rosettax87Adapter::new().unwrap();
    let fxch_bytes = vec![0xD9, 0xC9];
    let result = adapter.translate_x87_instruction(&fxch_bytes);
    assert!(result.is_ok());
}
```

### End-to-End Tests
Full WoW launch and gameplay validation:
```rust
#[test]
#[ignore] // Run manually
fn test_wow_launches_and_runs() {
    // Launch WoW, verify x87 translation active
}
```

## Migration Path

### Phase 1: Open-Source Integration
- Wrap rosettax87_jit via FFI
- Wrap winerosetta via FFI
- Test with MRE harness

### Phase 2: Working POC
- Integrate with CrossOver
- Launch WoW successfully
- Verify gameplay

### Phase 3: Progressive Enhancement
- Recreate libSiliconPatch in Rust
- Add new features via ports/adapters
- Improve test coverage

## See Also
- [Porting Policy](porting-policy.md)
- [Testing Strategy](testing-strategy.md)
- [Git Workflow](git-workflow.md)
