#!/usr/bin/env bash
set -e

echo "Running pre-commit checks..."

# Rust checks
if [ -d "packages/rust-core" ]; then
    echo "→ Rust typecheck..."
    cd packages/rust-core
    cargo check --all-targets
    cargo clippy -- -D warnings
    cargo fmt --check
    cd - > /dev/null
fi

# Integration checks
if [ -d "packages/integration" ]; then
    echo "→ MRE tests..."
    cd packages/integration
    cargo test --test mre
    cd - > /dev/null
fi

# Zig checks (use mise shim if bare `zig` not on PATH)
ZIG="${ZIG:-zig}"
if ! command -v "$ZIG" &>/dev/null; then
    MISE_SHIM="$HOME/.local/share/mise/shims/zig"
    if [ -x "$MISE_SHIM" ]; then
        ZIG="$MISE_SHIM"
    fi
fi
if [ -d "packages/zig-glue" ]; then
    echo "→ Zig fmt check..."
    "$ZIG" fmt --check packages/zig-glue/build.zig
fi

echo "✅ All checks passed"
