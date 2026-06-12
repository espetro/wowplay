#!/usr/bin/env bash
# Contributor setup — init submodules, build rosettax87_jit, and install git hooks.
# Not needed for end users (download the release zip instead).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Setting up play-wow-on-silicon development environment..."

# ── Vendor submodules ────────────────────────────────────────────────────────
echo "→ Initialising vendor submodules…"
git -C "$REPO_ROOT" submodule update --init --recursive vendor/rosettax87_jit

# ── Build rosettax87_jit (CMake) ─────────────────────────────────────────────
echo "→ Building rosettax87_jit (CMake)…"
cmake \
  -B "$REPO_ROOT/vendor/rosettax87_jit/build" \
  -S "$REPO_ROOT/vendor/rosettax87_jit" \
  -DCMAKE_BUILD_TYPE=Release \
  -Wno-dev \
  --log-level=WARNING
cmake --build "$REPO_ROOT/vendor/rosettax87_jit/build" --config Release

# ── Git hooks via lefthook ───────────────────────────────────────────────────
echo "→ Installing git hooks…"
lefthook install

echo ""
echo "✅ Setup complete."
echo ""
echo "Next steps:"
echo "  cargo build -p wowplay"
echo "  wowplay setup --wow-dir ~/Documents/WoW_3.3.5a"
echo "  wowplay run   --wow-dir ~/Documents/WoW_3.3.5a"
