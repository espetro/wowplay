#!/usr/bin/env bash
set -euo pipefail

TRIPLE=$(rustc -vV | awk '/^host:/ { print $2 }')
REPO_ROOT=$(cd "$(dirname "$0")/../../.." && pwd)

cargo build --release --target "$TRIPLE" -p wowplay \
  --manifest-path "$REPO_ROOT/Cargo.toml"

OUT_DIR="$REPO_ROOT/packages/gui/binaries"
mkdir -p "$OUT_DIR"
cp "$REPO_ROOT/target/$TRIPLE/release/wowplay" "$OUT_DIR/wowplay-$TRIPLE"
echo "sidecar ready: binaries/wowplay-$TRIPLE"

# Stage patching resources so Tauri can bundle them via tauri.conf.json resources glob.
# DIST_DIR=packages/gui causes stage-patching.sh to write into packages/gui/patching/.
rm -rf "$REPO_ROOT/packages/gui/patching"
mkdir -p "$REPO_ROOT/packages/gui/patching"
DIST_DIR="$REPO_ROOT/packages/gui" bash "$REPO_ROOT/scripts/stage-patching.sh"
echo "patching staged: packages/gui/patching/"
