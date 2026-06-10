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
PATCHING_SRC="$REPO_ROOT/vendor/wowsilicon/Sources/WoWSiliconSwift/Resources/Patching"
PATCHING_DST="$REPO_ROOT/packages/gui/patching"
if [[ -d "$PATCHING_SRC" ]]; then
  rm -rf "$PATCHING_DST"
  mkdir -p "$PATCHING_DST"
  cp -R "$PATCHING_SRC/." "$PATCHING_DST/"
  WINEROSETTA_DLL="$REPO_ROOT/packages/zig-glue/zig-out/bin/winerosetta.dll"
  if [[ -f "$WINEROSETTA_DLL" ]]; then
    cp "$WINEROSETTA_DLL" "$PATCHING_DST/winerosetta/winerosetta.dll"
    echo "patching staged: packages/gui/patching/ (with winerosetta.dll)"
  else
    echo "patching staged: packages/gui/patching/ (winerosetta.dll missing — run zig build first)"
  fi
else
  echo "warning: patching source not found at $PATCHING_SRC — submodule not initialised?"
fi
