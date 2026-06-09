#!/usr/bin/env bash
# Build, sign, notarize, and package wowplay as a signed ARM64 CLI binary.
#
# Produces:
#   dist/wowplay                  — signed ARM64 CLI binary
#   dist/winerosetta.dll          — built by zig-glue
#   dist/winerosetta.pdb          — debug symbols
#   dist/install.sh               — copies binary to ~/.local/bin
#   dist/wowplay-VERSION.zip      — archive for GitHub releases
#
# Local install (default):
#   Copies the binary to ~/.local/bin/wowplay.
#   Pass --skip-install to suppress.
#
# Usage:
#   ./scripts/release.sh [--profile <keychain-profile>] [--skip-notarize] [--skip-install]
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
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)       KEYCHAIN_PROFILE="$2"; shift 2 ;;
    --skip-notarize) SKIP_NOTARIZE=true; shift ;;
    --skip-install)  SKIP_INSTALL=true; shift ;;
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

# ── Stage dist/ ─────────────────────────────────────────────────────────────

echo ""
echo "==> Staging dist/…"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

cp "${CLI_BIN}" "${DIST_DIR}/${BIN_NAME}"
cp "${ZIG_OUT}/bin/winerosetta.dll" "${DIST_DIR}/winerosetta.dll"
[[ -f "${ZIG_OUT}/bin/winerosetta.pdb" ]] && cp "${ZIG_OUT}/bin/winerosetta.pdb" "${DIST_DIR}/winerosetta.pdb"

cat > "${DIST_DIR}/install.sh" << 'INSTALL'
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "${HOME}/.local/bin"
cp "$(dirname "$0")/wowplay" "${HOME}/.local/bin/wowplay"
chmod +x "${HOME}/.local/bin/wowplay"
echo "Installed wowplay to ~/.local/bin/wowplay"
INSTALL
chmod +x "${DIST_DIR}/install.sh"

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

ZIP_PATH="${DIST_DIR}/${BIN_NAME}-${VERSION}.zip"
echo ""
echo "==> Packaging ${ZIP_PATH}…"
ditto -c -k "${DIST_DIR}" "${ZIP_PATH}"

# ── Notarization + stapling ──────────────────────────────────────────────────

if [[ "${SKIP_NOTARIZE}" != "true" ]]; then
  echo ""
  echo "==> Notarizing…"
  xcrun notarytool submit "${ZIP_PATH}" "${NOTARY_AUTH[@]}" --wait

  # Flat binaries cannot be stapled (only .app bundles, .dmg, .pkg support
  # stapling). The notarization ticket is verified online by Gatekeeper.
fi

echo ""
echo "Done. wowplay v${VERSION} is ready:"
echo "  Binary : ${DIST_DIR}/${BIN_NAME}"
echo "  Archive: ${ZIP_PATH}"
[[ "${SKIP_INSTALL}" != "true" ]] && echo "  Installed: ~/.local/bin/${BIN_NAME}"
echo ""
echo "Upload ${ZIP_PATH} to the GitHub v${VERSION} release."
