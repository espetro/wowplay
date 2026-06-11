#!/usr/bin/env bash
# Stage dist/patching/ from explicit, auditable per-artifact sources.
# Every staged binary has known provenance: source-built or vendored+checksummed.
# Called by release.sh and build-release.yml.
#
# Prerequisites (must be satisfied before calling this script):
#   vendor/prebuilt/           — vendored binaries with CHECKSUMS.sha256
#   packages/zig-glue/zig-out/ — zig build --release=safe output
#   vendor/rosettax87_jit/build/bin/  — cmake build output
#
# Environment:
#   DIST_DIR            — staging root (default: <repo_root>/dist)
#   ROSETTAX87_BIN_DIR  — override rosettax87 binary source dir

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${DIST_DIR:-${REPO_ROOT}/dist}"

# ── Verify vendored binaries ──────────────────────────────────────────────────

echo "  Verifying vendor/prebuilt checksums…"
(cd "${REPO_ROOT}/vendor/prebuilt" && shasum -a 256 -c CHECKSUMS.sha256)

# ── Resolve source paths ──────────────────────────────────────────────────────

PREBUILT="${REPO_ROOT}/vendor/prebuilt"
ZIG_OUT="${REPO_ROOT}/packages/zig-glue/zig-out/bin"
RTX87_BIN="${ROSETTAX87_BIN_DIR:-${REPO_ROOT}/vendor/rosettax87_jit/build/bin}"

if [[ ! -f "${ZIG_OUT}/winerosetta.dll" ]]; then
  echo "error: winerosetta.dll not found at ${ZIG_OUT}" >&2
  echo "       run: cd packages/zig-glue && zig build --release=safe" >&2
  exit 1
fi
if [[ ! -f "${RTX87_BIN}/runtime_loader" ]]; then
  echo "error: runtime_loader not found at ${RTX87_BIN}" >&2
  echo "       run: cmake -B vendor/rosettax87_jit/build -S vendor/rosettax87_jit && cmake --build vendor/rosettax87_jit/build" >&2
  exit 1
fi
if [[ ! -f "${RTX87_BIN}/libRuntimeRosettax87" ]]; then
  echo "error: libRuntimeRosettax87 not found at ${RTX87_BIN}" >&2
  exit 1
fi

# ── Stage dist/patching/ ──────────────────────────────────────────────────────

echo "  Staging dist/patching/…"
mkdir -p \
  "${DIST_DIR}/patching/d9vk" \
  "${DIST_DIR}/patching/winerosetta" \
  "${DIST_DIR}/patching/rosettax87" \
  "${DIST_DIR}/patching/libSiliconPatch/wotlk"

# D9VK — vendored binary (Gcenx/DXVK-macOS, zlib/libpng)
cp "${PREBUILT}/d9vk/d3d9.dll" "${DIST_DIR}/patching/d9vk/d3d9.dll"

# winerosetta.dll — source-built from packages/zig-glue
cp "${ZIG_OUT}/winerosetta.dll" "${DIST_DIR}/patching/winerosetta/winerosetta.dll"
[[ -f "${ZIG_OUT}/winerosetta.pdb" ]] && \
  cp "${ZIG_OUT}/winerosetta.pdb" "${DIST_DIR}/patching/winerosetta/winerosetta.pdb"

# rosettax87 pair — source-built from vendor/rosettax87_jit
# Staged as "rosettax87" (the name apply_game_patch expects when no ROSETTAX87_BIN_DIR override).
cp "${RTX87_BIN}/runtime_loader"       "${DIST_DIR}/patching/rosettax87/rosettax87"
cp "${RTX87_BIN}/libRuntimeRosettax87" "${DIST_DIR}/patching/rosettax87/libRuntimeRosettax87"

# libSiliconPatch — vendored, opt-in only (staged so --enable-lib-silicon works out of the box)
cp "${PREBUILT}/libSiliconPatch/wotlk/libSiliconPatch.dll" \
   "${DIST_DIR}/patching/libSiliconPatch/wotlk/libSiliconPatch.dll"

# Explicitly omitted:
#   winerosetta/libDllLdr.dll     — replaced by native Rust PE patcher (19e1eaa)
#   winerosetta/ntdll.so          — no loader references; never copied to WoW dir; dropped
#   libSiliconPatch/vanilla/      — not referenced by apply_game_patch
#   vanilla-tweaks/               — unused MIT binary

echo "  dist/patching/ staged."
