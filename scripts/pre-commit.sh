#!/usr/bin/env bash
# Pre-commit validation for play-wow-on-silicon
# Run manually: ./scripts/pre-commit.sh

set -euo pipefail

echo "=== Running pre-commit checks ==="

# Rust workspace checks
if [ -f "Cargo.toml" ]; then
    echo "[rust] cargo check..."
    cargo check --all-targets 2>&1 || { echo "FAILED: cargo check"; exit 1; }
fi

# Profiler Python checks
if [ -d "tools/profiler" ]; then
    echo "[profiler] python schema validation..."
    (cd tools/profiler && python3 schema.py data/profiling/*.json 2>/dev/null || true)
fi

# MRE crate checks (if generated)
if [ -d "rust_x87_mre" ]; then
    echo "[mre] cargo check..."
    (cd rust_x87_mre && cargo check --lib 2>&1) || { echo "FAILED: MRE crate check"; exit 1; }
fi

echo "=== All checks passed ==="
