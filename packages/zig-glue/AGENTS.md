# zig-glue Package - Agent Guide

## Overview
Cross-compile winerosetta.dll (Windows x86 DLL) from C++ source using Zig's native cross-compilation without external mingw toolchain.

## Your Workspace
```
packages/zig-glue/
├── AGENTS.md                  # This file
├── build.zig                  # Zig build system entry
└── src/
    └── root.zig               # Build root (orchestrates compilation)
vendor/
└── winerosetta/
    └── winerosetta.cpp        # C++17 source: Wine x87 translator
zig-out/
└── bin/
    └── winerosetta.dll        # Output: Windows x86 DLL
```

## Quick Start
1. Ensure Zig is installed: `zig version`
2. Cross-compile: `zig build`
3. Output: `zig-out/bin/winerosetta.dll` (Windows x86 executable code)

## Key Purpose

**Zig solves a critical problem**: Cross-compiling C++17 (`winerosetta.cpp`) to Windows x86 without:
- macOS→Windows mingw toolchain complexity
- C++ standard library incompatibilities
- System-specific header path issues

Zig's `zig cc` and `x86-windows-gnu` target handle this in one command.

## Build Configuration

### build.zig Structure
```zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    // Windows x86 target (not native macOS ARM)
    const target = b.resolveTargetQuery(.{
        .cpu_arch = .x86,
        .os_tag = .windows,
        .abi = .gnu,
    });
    
    const exe = b.addExecutable(.{
        .name = "winerosetta",
        .target = target,
        .optimize = .ReleaseFast,
    });
    
    exe.addCSourceFile(.{
        .file = .{ .path = "vendor/winerosetta/winerosetta.cpp" },
        .flags = &.{ "-std=c++17", "-O2" },
    });
    
    exe.linkLibCpp(); // C++ standard library
    exe.subsystem = .Windows; // Windows subsystem
    b.installArtifact(exe);
}
```

## Common Tasks

### Build for Windows x86
```bash
zig build
# Output: zig-out/bin/winerosetta.dll
```

### Debug Build
```bash
zig build -Doptimize=Debug
# Output: zig-out/bin/winerosetta.dll (with symbols)
```

### Check Build Environment
```bash
zig version              # Must be installed
zig cc --version        # Verifies Zig's C compiler
```

### Cross-Compilation Targets
Zig can target other platforms from macOS:
```
x86-windows-gnu         # Current: 32-bit Windows
x86_64-windows-gnu      # 64-bit Windows (future)
aarch64-windows-msvc    # Arm64 Windows (future)
x86-linux-gnu           # 32-bit Linux testing
```

## Integration Points

### With rust-core
- Compiled DLL is loaded via `libloading` in adapters
- Path: typically `libloading::Library::new("winerosetta.dll")`

### With integration tests
- Hook injection tests (hook_injection_tests.rs) validate DLL injection
- E2E tests require WoW.exe environment

## FFI Safety Rules
1. **DLL entry points**: Must be C calling convention (`extern "C"`)
2. **Buffer sizes**: All pointers must be validated by Wine adapter
3. **Error codes**: Return integers (not Rust Results) from C++

## Testing

### Smoke Test the Build
```bash
# Verify DLL compiles and exports symbols
zig build
file zig-out/bin/winerosetta.dll  # Should say PE Windows
```

### Integration via Rust
The compiled DLL is consumed by:
- `packages/rust-core/src/adapters/` (dynamic loading)
- `packages/integration/tests/mre/` (MRE validation)

## Why Zig (Not mingw)
| Aspect | Zig | mingw |
|--------|-----|-------|
| Setup | Single brew/release | Complex toolchain |
| C++ std | Built-in | External gcc/g++ |
| Target-specific | Native, no prefix | Requires x86_64-w64-mingw32- prefix |
| Debuggability | Integrated | Separate gdb setup |

## Performance Notes
- **ReleaseFast**: Optimized, ~50KB DLL, x87 translation runs at 1-5µs per instruction
- **Debug**: Larger with symbols, ~500KB DLL (development only)

## Future Optimizations
- Static library variant (libwinerosetta.a) for Rust linkage
- Native macOS build for local testing (via `aarch64-macos` target)

## See Also
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Rust Core](../rust-core/AGENTS.md) - FFI and adapter usage
- [Integration](../integration/AGENTS.md) - Test harness that validates DLL
