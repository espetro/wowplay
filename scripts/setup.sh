#!/usr/bin/env bash
# Contributor setup — init submodules and install git hooks via lefthook.
# Not needed for end users (download the release zip instead).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Setting up play-wow-on-silicon development environment..."

# ── Vendor submodules ────────────────────────────────────────────────────────
echo "→ Initialising vendor submodules…"
git -C "$REPO_ROOT" submodule update --init --recursive vendor/rosettax87_jit

# ── Git hooks via lefthook ────────────────────────────────────────────────────
echo "→ Installing git hooks…"
lefthook install

echo ""
echo "✅ Setup complete."
echo ""
echo "Next steps:"
echo "  cargo build -p wowplay"
echo "  wowplay setup --wow-dir ~/Documents/WoW_3.3.5a"
echo "  wowplay run   --wow-dir ~/Documents/WoW_3.3.5a"
