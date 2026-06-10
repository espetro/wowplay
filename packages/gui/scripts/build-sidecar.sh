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
