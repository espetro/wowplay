#!/usr/bin/env bash
set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Setting up play-wow-on-silicon development environment..."

# ── Rust ────────────────────────────────────────────────────────────────────
if ! command -v rustup &> /dev/null; then
    echo "→ Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
else
    echo "→ Rust already installed, ensuring components..."
    rustup component add clippy rustfmt
fi

# ── Zig (via mise or homebrew) ───────────────────────────────────────────────
if command -v zig &> /dev/null; then
    echo "→ Zig $(zig version) already available"
elif [ -x "$HOME/.local/share/mise/installs/zig/0.16.0/bin/zig" ]; then
    echo "→ Zig 0.16.0 found via mise (not on PATH; add mise shims to PATH)"
else
    echo "⚠️  Zig not found. Install with: brew install zig  (or: mise install zig@0.16.0)"
fi

ZIG="${ZIG:-zig}"
if [ -x "$HOME/.local/share/mise/installs/zig/0.16.0/bin/zig" ]; then
    ZIG="$HOME/.local/share/mise/installs/zig/0.16.0/bin/zig"
fi

# ── Build rosettax87_jit (native macOS arm64 runtime_loader) ─────────────────
ROSETTA_JIT_SRC="$REPO_ROOT/vendor/rosettax87_jit"
ROSETTA_JIT_BIN="$ROSETTA_JIT_SRC/build/bin/runtime_loader"

if [ ! -f "$ROSETTA_JIT_BIN" ]; then
    echo "→ Building rosettax87_jit..."
    if [ ! -d "$ROSETTA_JIT_SRC/.git" ]; then
        git -C "$REPO_ROOT" submodule update --init --recursive vendor/rosettax87_jit
    fi
    cmake -B "$ROSETTA_JIT_SRC/build" -DCMAKE_BUILD_TYPE=Release "$ROSETTA_JIT_SRC"
    cmake --build "$ROSETTA_JIT_SRC/build" --config Release
    echo "→ runtime_loader built at $ROSETTA_JIT_BIN"
else
    echo "→ runtime_loader already built at $ROSETTA_JIT_BIN"
fi

# ── Build winerosetta.dll (cross-compiled Windows x86 via Zig) ───────────────
WINEROSETTA_DLL="$REPO_ROOT/packages/zig-glue/zig-out/bin/winerosetta.dll"

if [ ! -f "$WINEROSETTA_DLL" ]; then
    echo "→ Building winerosetta.dll (cross-compile x86-windows-gnu via Zig)..."
    cd "$REPO_ROOT/packages/zig-glue"
    "$ZIG" build --release=safe
    cd - > /dev/null
    echo "→ winerosetta.dll built at $WINEROSETTA_DLL"
else
    echo "→ winerosetta.dll already built"
fi

# ── Git hooks ────────────────────────────────────────────────────────────────
echo "→ Installing git hooks..."
mkdir -p "$REPO_ROOT/.git/hooks"
cp "$REPO_ROOT/scripts/pre-commit.sh" "$REPO_ROOT/.git/hooks/pre-commit"
chmod +x "$REPO_ROOT/.git/hooks/pre-commit"

echo ""
echo "✅ Setup complete."
echo ""
echo "Next steps:"
echo "  1. Put your WoW 3.3.5a client somewhere, e.g. ~/Desktop/wow/WoW"
echo "  2. Launch: WOW_DIR=~/Desktop/wow/WoW scripts/launch-wow.sh"
