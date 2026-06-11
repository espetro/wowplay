#!/usr/bin/env bash
# Build, sign, notarize, and upload wowplay as a signed ARM64 CLI binary.
#
# Produces:
#   dist/wowplay                        — signed ARM64 CLI binary
#   dist/patching/                      — patching resources
#   wowplay-vVERSION-macos-arm64.zip    — archive uploaded to the GitHub release
#
# Local install (default):
#   Copies the binary to ~/.local/bin/wowplay.
#   Pass --skip-install to suppress.
#
# Usage:
#   ./scripts/release.sh [--profile <keychain-profile>] [--skip-notarize] [--skip-install] [--skip-upload]
#
# Apple notarization credentials:
#   Store them once: xcrun notarytool store-credentials default \
#     --apple-id <your@apple.id> --team-id Y3ZC4LB357 --password <app-specific-password>

set -euo pipefail

SIGN_IDENTITY="Developer ID Application: Joaquin Terrasa Moya (Y3ZC4LB357)"
TARGET="aarch64-apple-darwin"
BIN_NAME="wowplay"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"

# Parse flags
KEYCHAIN_PROFILE="default"
SKIP_NOTARIZE=false
SKIP_INSTALL=false
SKIP_UPLOAD=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)       KEYCHAIN_PROFILE="$2"; shift 2 ;;
    --skip-notarize) SKIP_NOTARIZE=true; shift ;;
    --skip-install)  SKIP_INSTALL=true; shift ;;
    --skip-upload)   SKIP_UPLOAD=true; shift ;;
    *) echo "Unknown flag: $1" && exit 1 ;;
  esac
done

# Resolve notarytool auth args
NOTARY_AUTH=()
if [[ "${SKIP_NOTARIZE}" == "true" ]]; then
  echo "warning: --skip-notarize set — binary will be signed but not notarized."
else
  NOTARY_AUTH=(--keychain-profile "${KEYCHAIN_PROFILE}")
fi

VERSION="$(cargo metadata --manifest-path "${REPO_ROOT}/Cargo.toml" --no-deps --format-version 1 \
  | python3 -c 'import sys,json; pkgs=json.load(sys.stdin)["packages"]; print(next(p["version"] for p in pkgs if p["name"]=="wowplay"))')"

echo "Building wowplay v${VERSION} (${TARGET})…"

# ── CLI binary ───────────────────────────────────────────────────────────────

echo ""
echo "==> Building CLI binary…"
rustup target add "${TARGET}"
cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --target "${TARGET}" -p wowplay
CLI_BIN="${REPO_ROOT}/target/${TARGET}/release/${BIN_NAME}"

# ── winerosetta.dll ──────────────────────────────────────────────────────────

echo ""
echo "==> Building winerosetta.dll…"
cd "${REPO_ROOT}/packages/zig-glue"
zig build --release=safe
cd "${REPO_ROOT}"

ZIG_OUT="${REPO_ROOT}/packages/zig-glue/zig-out"

# ── Build rosettax87_jit ─────────────────────────────────────────────────────

echo ""
echo "==> Building rosettax87 from vendor/rosettax87_jit…"
cmake \
  -B "${REPO_ROOT}/vendor/rosettax87_jit/build" \
  -S "${REPO_ROOT}/vendor/rosettax87_jit" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_RUNTIME_OUTPUT_DIRECTORY="${REPO_ROOT}/vendor/rosettax87_jit/build/bin" \
  -Wno-dev \
  --log-level=WARNING
cmake --build "${REPO_ROOT}/vendor/rosettax87_jit/build" --config Release

# ── Stage dist/ ─────────────────────────────────────────────────────────────

echo ""
echo "==> Staging dist/…"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

cp "${CLI_BIN}" "${DIST_DIR}/${BIN_NAME}"

DIST_DIR="${DIST_DIR}" "${REPO_ROOT}/scripts/stage-patching.sh"

# ── Sign inner binaries (rosettax87 pair) ───────────────────────────────────

echo ""
echo "==> Signing inner binaries (rosettax87)…"
ENTITLEMENTS="${REPO_ROOT}/packages/gui/Rosettax87.entitlements"
# Sign libRuntimeRosettax87 first (dependency), then rosettax87.
codesign \
  --sign "${SIGN_IDENTITY}" \
  --options runtime \
  --timestamp \
  --force \
  --entitlements "${ENTITLEMENTS}" \
  "${DIST_DIR}/patching/rosettax87/libRuntimeRosettax87"
codesign --verify --verbose "${DIST_DIR}/patching/rosettax87/libRuntimeRosettax87"
codesign \
  --sign "${SIGN_IDENTITY}" \
  --options runtime \
  --timestamp \
  --force \
  --entitlements "${ENTITLEMENTS}" \
  "${DIST_DIR}/patching/rosettax87/rosettax87"
codesign --verify --verbose "${DIST_DIR}/patching/rosettax87/rosettax87"

# ── Sign CLI binary ──────────────────────────────────────────────────────────

echo ""
echo "==> Signing ${BIN_NAME}…"
codesign \
  --sign "${SIGN_IDENTITY}" \
  --options runtime \
  --timestamp \
  --force \
  "${DIST_DIR}/${BIN_NAME}"
codesign --verify --verbose "${DIST_DIR}/${BIN_NAME}"

# ── Local install ─────────────────────────────────────────────────────────────

if [[ "${SKIP_INSTALL}" != "true" ]]; then
  echo ""
  echo "==> Installing to ~/.local/bin/${BIN_NAME}…"
  mkdir -p "${HOME}/.local/bin"
  cp "${DIST_DIR}/${BIN_NAME}" "${HOME}/.local/bin/${BIN_NAME}"
  echo "    Installed: $(which ${BIN_NAME} 2>/dev/null || echo ~/.local/bin/${BIN_NAME})"
fi

# ── Package ──────────────────────────────────────────────────────────────────

ZIP_NAME="${BIN_NAME}-v${VERSION}-macos-arm64.zip"
ZIP_PATH="${REPO_ROOT}/${ZIP_NAME}"
echo ""
echo "==> Packaging ${ZIP_PATH}…"
(cd "${DIST_DIR}" && zip -r "${ZIP_PATH}" .)

# ── Notarization + stapling ──────────────────────────────────────────────────

if [[ "${SKIP_NOTARIZE}" != "true" ]]; then
  echo ""
  echo "==> Notarizing…"
  NOTARY_OUTPUT=$(xcrun notarytool submit "${ZIP_PATH}" "${NOTARY_AUTH[@]}" --wait --output-format json)
  echo "${NOTARY_OUTPUT}"
  NOTARY_STATUS=$(echo "${NOTARY_OUTPUT}" | python3 -c 'import sys, json; print(json.load(sys.stdin)["status"])')
  if [[ "${NOTARY_STATUS}" != "Accepted" ]]; then
    echo "Notarization status: ${NOTARY_STATUS} — aborting"
    exit 1
  fi
  echo "Notarization accepted."

  # Flat binaries cannot be stapled (only .app bundles, .dmg, .pkg support
  # stapling). The notarization ticket is verified online by Gatekeeper.
fi

# ── GitHub release upload ────────────────────────────────────────────────────

TAG="v${VERSION}"
if [[ "${SKIP_UPLOAD}" != "true" ]]; then
  echo ""
  echo "==> Uploading ${ZIP_NAME} to GitHub release ${TAG}…"
  gh release upload "${TAG}" "${ZIP_PATH}" --clobber
  echo "    Uploaded: https://github.com/espetro/wowplay/releases/tag/${TAG}"
fi

echo ""
echo "Done. wowplay v${VERSION} is ready:"
echo "  Binary  : ${DIST_DIR}/${BIN_NAME}"
echo "  Patching: ${DIST_DIR}/patching/"
echo "  Archive : ${ZIP_PATH}"
[[ "${SKIP_INSTALL}" != "true" ]] && echo "  Installed: ~/.local/bin/${BIN_NAME}"
