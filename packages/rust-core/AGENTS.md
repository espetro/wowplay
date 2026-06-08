# rust-core Package - Agent Guide

## Overview
Rust domain logic, ports, and adapters for WoW on Apple Silicon.

## Your Workspace
```
packages/rust-core/
├── AGENTS.md          # This file
├── Cargo.toml         # Rust dependencies
└── src/
    ├── ports/         # Port trait definitions
    ├── adapters/      # External library wrappers
    ├── domain/        # Core business logic
    └── lib.rs        # Library root
```

## Quick Start
1. Ensure Rust is installed: `rustc --version`
2. Add dependencies to `Cargo.toml`
3. Implement ports in `src/ports/`
4. Implement adapters in `src/adapters/`
5. Test with `cargo test`

## Key Principles
- **Ports are traits**: Define contracts, not implementations
- **Adapters wrap external libs**: Never modify C/C++ libraries
- **FFI safety**: Always validate buffer sizes and pointers
- **Error handling**: Convert C errors to Rust Results

## Common Tasks

### Add a New Port
```rust
// src/ports/my_port.rs
pub trait MyPort: Send + Sync {
    fn do_something(&self, input: &str) -> Result<Vec<u8>, MyError>;
}
```

### Implement an Adapter
```rust
// src/adapters/my_adapter.rs
use libloading::{Library, Symbol};

pub struct MyAdapter {
    library: Library,
}

impl MyAdapter {
    pub fn new() -> Result<Self, AdapterError> {
        let library = unsafe {
            Library::new("/path/to/lib.dylib")
        }?;
        Ok(Self { library })
    }
}

impl MyPort for MyAdapter {
    fn do_something(&self, input: &str) -> Result<Vec<u8>, MyError> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*const u8) -> usize> =
                self.library.get("my_function")?;
            // Call function safely
        }
    }
}
```

### Add Tests
```rust
// src/adapters/my_adapter_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_loads() {
        let adapter = MyAdapter::new().unwrap();
        assert!(adapter.is_initialized());
    }
}
```

## Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
libloading = "0.8"
thiserror = "1.0"

[dev-dependencies]
# Test-only dependencies
```

## Testing
- Unit tests: `cargo test --lib`
- Integration tests: `cargo test`
- With output: `cargo test -- --nocapture`

## FFI Safety Rules
1. **Always check return values**: C functions can fail
2. **Validate buffer sizes**: Prevent buffer overflows
3. **Use appropriate types**: `c_int`, `c_char` from `libc`
4. **Document safety invariants**: Why is this unsafe block safe?

## Common Patterns

### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
    
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("Call failed with code: {0}")]
    CallFailed(i32),
}
```

### Resource Cleanup
```rust
impl Drop for MyAdapter {
    fn drop(&mut self) {
        // Cleanup resources
        unsafe {
            ffi_cleanup(self.handle);
        }
    }
}
```

## See Also
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Architecture](../../docs/architecture.md) - System design
