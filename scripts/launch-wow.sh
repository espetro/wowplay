#!/usr/bin/env bash
# Launch WoW 3.3.5a on Apple Silicon via WoWSilicon assets + CrossOver.
#
# Usage:
#   WOW_DIR=/path/to/WoW/client ./scripts/launch-wow.sh
#
# Prerequisites:
#   - WoWSilicon.app installed in ~/Applications or /Applications
#   - CrossOver.app installed in ~/Applications or /Applications
#
# This script applies the same patches WoWSilicon would apply via its UI,
# then launches using the proven rosettax87 + wineloader64 chain.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── Locate WoWSilicon resources ───────────────────────────────────────────────
WOWSILICON_RES=""
for candidate in \
    "$HOME/Applications/WoWSilicon.app/Contents/Resources/WoWSilicon-swift_WoWSiliconSwift.bundle/Patching" \
    "/Applications/WoWSilicon.app/Contents/Resources/WoWSilicon-swift_WoWSiliconSwift.bundle/Patching"; do
    if [ -d "$candidate" ]; then
        WOWSILICON_RES="$candidate"
        break
    fi
done
if [ -z "$WOWSILICON_RES" ]; then
    echo "❌ WoWSilicon.app not found. Download from https://github.com/WoWSilicon/WoWSilicon"
    exit 1
fi

# ── Locate CrossOver ─────────────────────────────────────────────────────────
CROSSOVER=""
for candidate in "$HOME/Applications/CrossOver.app" "/Applications/CrossOver.app"; do
    if [ -d "$candidate" ]; then
        CROSSOVER="$candidate"
        break
    fi
done
if [ -z "$CROSSOVER" ]; then
    echo "❌ CrossOver.app not found. Install from codeweavers.com."
    exit 1
fi

CX_ROOT="$CROSSOVER/Contents/SharedSupport/CrossOver"
CX_HOSTED="$CROSSOVER/Contents/SharedSupport/CrossOver/CrossOver-Hosted Application"

# Must use wineloader64 (x86_64): macOS 15 dropped i386 support.
WINELOADER_SRC="$CX_HOSTED/wineloader64"
if [ ! -f "$WINELOADER_SRC" ]; then
    echo "❌ wineloader64 not found at $WINELOADER_SRC"
    exit 1
fi

# ── Locate WoW directory ─────────────────────────────────────────────────────
WOW_DIR="${WOW_DIR:-$HOME/Documents/ChromieCraft_3.3.5a}"
if [ ! -d "$WOW_DIR" ]; then
    echo "❌ WoW directory not found: $WOW_DIR"
    echo "   Set WOW_DIR=/path/to/client and re-run."
    exit 1
fi

WOW_EXE=""
for name in WoW.exe wow.exe Wow.exe; do
    if [ -f "$WOW_DIR/$name" ]; then
        WOW_EXE="$WOW_DIR/$name"
        break
    fi
done
if [ -z "$WOW_EXE" ]; then
    echo "❌ WoW.exe not found in $WOW_DIR"
    exit 1
fi

# ── Apply Game Patch (mirrors WoWSilicon's applyGamePatch) ───────────────────
echo "→ Applying game patch..."

# d3d9.dll (D9VK: DirectX9 → Vulkan → MoltenVK → Metal)
# WINEDLLOVERRIDES=d3d9=n,b causes Wine to load this instead of its own d3d9.
cp "$WOWSILICON_RES/d9vk/d3d9.dll" "$WOW_DIR/d3d9.dll"

# winerosetta.dll at game root so WINEDLLOVERRIDES=winerosetta=n,b finds it.
# Also copy to mods/ for dlls.txt on newer CrossOver/Wine builds.
mkdir -p "$WOW_DIR/mods"
cp "$WOWSILICON_RES/winerosetta/winerosetta.dll" "$WOW_DIR/winerosetta.dll"
cp "$WOWSILICON_RES/winerosetta/winerosetta.dll" "$WOW_DIR/mods/winerosetta.dll"
cp "$WOWSILICON_RES/libSiliconPatch/wotlk/libSiliconPatch.dll" "$WOW_DIR/mods/libSiliconPatch.dll"

# libDllLdr.dll enables DivxDecoder-based DLL loading (WoWSilicon requires it)
cp "$WOWSILICON_RES/winerosetta/libDllLdr.dll" "$WOW_DIR/libDllLdr.dll"

# rosettax87 binaries — the x87 JIT translator
mkdir -p "$WOW_DIR/rosettax87"
cp "$WOWSILICON_RES/rosettax87/rosettax87"            "$WOW_DIR/rosettax87/rosettax87"
cp "$WOWSILICON_RES/rosettax87/libRuntimeRosettax87"  "$WOW_DIR/rosettax87/libRuntimeRosettax87"
chmod 755 "$WOW_DIR/rosettax87/rosettax87" "$WOW_DIR/rosettax87/libRuntimeRosettax87"
# Remove macOS quarantine flag so the system doesn't block execution with "damaged" alert
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/rosettax87" 2>/dev/null || true
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/libRuntimeRosettax87" 2>/dev/null || true

# dlls.txt: Wine reads this to load additional DLLs at startup
DLLS_TXT="$WOW_DIR/dlls.txt"
touch "$DLLS_TXT"
for dll in "mods/winerosetta.dll" "mods/libSiliconPatch.dll"; do
    grep -qiF "$dll" "$DLLS_TXT" || echo "$dll" >> "$DLLS_TXT"
done

echo "   d3d9.dll (D9VK), winerosetta.dll, libSiliconPatch.dll, libDllLdr.dll, rosettax87 ✓"

# ── Create unsigned wineloader64 in writable location ───────────────────────
# Must keep filename as `wineloader64` — Wine re-execs itself by that name.
mkdir -p /tmp/cx-bin
WINELOADER2="/tmp/cx-bin/wineloader64"
echo "→ Creating unsigned wineloader64 at $WINELOADER2..."
cp "$WINELOADER_SRC" "$WINELOADER2"
codesign --remove-signature "$WINELOADER2" 2>/dev/null || true

# ── Wine environment ─────────────────────────────────────────────────────────
BOTTLE="${CX_BOTTLE:-Win10}"
export CX_ROOT CX_BOTTLE="$BOTTLE"
export WINEPREFIX="$HOME/Library/Application Support/CrossOver/Bottles/$BOTTLE"
export WINESERVER="$CX_HOSTED/wineserver"
export WINELOADER="$WINELOADER2"
export WINEDLLPATH="$CX_ROOT/lib/wine:$CX_ROOT/lib64/wine"
# d3d9=n,b: load D9VK; winerosetta=n,b: load x87 VEH patcher
export WINEDLLOVERRIDES="d3d9=n,b;winerosetta=n,b"
export DYLD_LIBRARY_PATH="$CX_ROOT/lib:$CX_HOSTED"
export DYLD_FALLBACK_LIBRARY_PATH="$CX_ROOT/lib:/usr/lib"
# D9VK/MoltenVK performance settings
export MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS=1
export DXVK_ASYNC=1

ROSETTAX87="$WOW_DIR/rosettax87/rosettax87"

# ── Launch ────────────────────────────────────────────────────────────────────
echo "→ Launching WoW via rosettax87 + wineloader64..."
echo "   rosettax87:   $ROSETTAX87"
echo "   wineloader64: $WINELOADER2"
echo "   WoW:          $WOW_EXE"
echo "   bottle:       $BOTTLE"
echo ""

cd "$WOW_DIR"
exec "$ROSETTAX87" "$WINELOADER2" "$WOW_EXE"
