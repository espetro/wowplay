#!/usr/bin/env bash
# check-rosetta-freshness.sh — verify rosettax87_jit CMake binaries are up-to-date.
#
# Single source of truth for freshness logic, called by:
#   - packages/cli/build.rs  (cargo:warning= lines)
#   - just check-rosetta-freshness
#   - lefthook / pre-commit
#
# Exit codes:
#   0  — all binaries fresh (or WOWPLAY_SKIP_ROSETTA_FRESHNESS=1)
#   1  — one or more stale or missing binaries

set -euo pipefail

SKIP_VAR="WOWPLAY_SKIP_ROSETTA_FRESHNESS"
if [[ "${!SKIP_VAR:-}" == "1" ]]; then
    exit 0
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$REPO_ROOT/vendor/rosettax87_jit/build/bin"

RUNTIME_LOADER="$BUILD_DIR/runtime_loader"
LIB_RUNTIME="$BUILD_DIR/libRuntimeRosettax87"

# Source tree to walk (relative to REPO_ROOT/vendor/rosettax87_jit/)
SOURCE_ROOT="$REPO_ROOT/vendor/rosettax87_jit"

# Find newest source mtime across all relevant extensions.
# Excludes: build/, .git/, tests/, benchmarks/
newest_mtime=0
while IFS= read -r -d '' file; do
    # Get mtime via stat; macOS (stat -f) vs Linux (stat -c) compatible
    mtime=$(stat -f '%m' "$file" 2>/dev/null || stat -c '%Y' "$file" 2>/dev/null || echo 0)
    if (( $(echo "$mtime > $newest_mtime" | bc -lq 2>/dev/null || echo 0) )); then
        newest_mtime=$mtime
    fi
done < <(
    find "$SOURCE_ROOT" \
        \( \
            -name '*.c' -o -name '*.h' -o \
            -name '*.cpp' -o -name '*.hpp' -o \
            -name '*.cmake' -o -name 'CMakeLists.txt' \
        \) \
        -not -path '*/build/*' \
        -not -path '*/.git/*' \
        -not -path '*/tests/*' \
        -not -path '*/benchmarks/*' \
        -print0 2>/dev/null || true
)

# Tolerance window in seconds — compensates for mtime granularity and build lag
TOLERANCE=2

# Check a single binary: missing or older than newest source + tolerance
check_binary() {
    local bin_path="$1"
    local bin_name
    bin_name=$(basename "$bin_path")

    if [[ ! -f "$bin_path" ]]; then
        echo "MISSING: $bin_name not found at $bin_path"
        echo "  → Rebuild: just build-rosettax87"
        return 1
    fi

    local bin_mtime
    bin_mtime=$(stat -f '%m' "$bin_path" 2>/dev/null || stat -c '%Y' "$bin_path" 2>/dev/null || echo 0)
    local age=$((newest_mtime - bin_mtime))

    # If newest_mtime is 0 (empty source tree edge case), skip comparison
    if [[ "$newest_mtime" != "0" ]] && (( age > TOLERANCE )); then
        echo "STALE: $bin_name is older than newest source (by ${age}s > ${TOLERANCE}s tolerance)"
        echo "  → Rebuild: just build-rosettax87"
        return 1
    fi

    return 0
}

status=0
check_binary "$RUNTIME_LOADER" || status=1
check_binary "$LIB_RUNTIME"    || status=1

exit $status
