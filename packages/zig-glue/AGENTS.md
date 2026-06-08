# zig-glue Package - Agent Guide

## Overview
Zig-based build system for C/C++ interop and low-level glue code.

## Your Workspace
```
packages/zig-glue/
├── AGENTS.md          # This file
├── build.zig         # Zig build configuration
└── src/
    ├── ffi/          # FFI bindings
    └── glue/         # Glue code
```

## Quick Start
1. Ensure Zig is installed: `zig version`
2. Configure build in `build.zig`
3. Implement glue code in `src/`
4. Build with `zig build`

## Key Principles
- **Minimal C interop**: Only what Rust can't do easily
- **Safety first**: Validate all pointers and buffers
- **No business logic**: This is glue code, not domain logic
- **Test thoroughly**: FFI bugs are hard to debug

## Common Tasks

### Add FFI Binding
```zig
// src/ffi/my_binding.zig
const c = @cImport({
    @cInclude("my_library.h");
});

pub extern "C" fn my_wrapper(arg: c_int) callconv(.C) c_int {
    // Wrapper logic
    return c.my_function(arg);
}
```

### Configure Build
```zig
// build.zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(null);
    const optimize = b.standardOptimizeOption(null);
    
    const lib = b.addStaticLibrary(.{
        .name = "zig-glue",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = optimize,
    });
    
    b.installArtifact(lib);
}
```

### Add Library Linkage
```zig
// In build.zig
lib.linkSystemLibrary("my_library");
lib.addIncludePath("/path/to/headers");
```

## Testing
```bash
# Run tests
zig build test

# Run specific test
zig test src/my_test.zig
```

## When to Use Zig vs Rust

### Use Zig For:
- Direct C library interop
- Memory layout manipulation
- System-level programming
- Assembly integration

### Use Rust For:
- Business logic
- Domain modeling
- Higher-level abstractions
- Most application code

## Common Patterns

### Safe FFI Wrapper
```zig
pub extern "C" fn safe_wrapper(ptr: ?*const anyopaque) bool {
    if (ptr == null) return false;
    
    const data = @ptrCast(*const MyData, ptr);
    return data.isValid();
}
```

### Error Handling
```zig
const Error = error{
    InvalidArgument,
    OutOfMemory,
    LibraryNotFound,
};

pub fn doWork() Error!void {
    // Work that can fail
}
```

## Dependencies
Zig can link C libraries directly:
```zig
lib.linkSystemLibrary("rosettax87_jit");
lib.linkSystemLibrary("winerosetta");
```

## Formatting
```bash
# Format code
zig fmt .

# Check formatting
zig fmt --check .
```

## See Also
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Architecture](../../docs/architecture.md) - System design
