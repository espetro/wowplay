#!/usr/bin/env bash
# Launch WoW 3.3.5a on Apple Silicon via CrossOver + rosettax87.
#
# Usage:
#   WOW_DIR=/path/to/WoW/client ./scripts/launch-wow.sh [--diagnose]
#
# Prerequisites:
#   - CrossOver.app installed in ~/Applications or /Applications
#   - WoW 3.3.5a client (32-bit x86 Windows application)
#   - WoWSilicon.app (for d3d9.dll, winerosetta.dll, libSiliconPatch.dll)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
DIAGNOSE=false

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --diagnose)
            DIAGNOSE=true
            ;;
        *)
            echo "❌ Unknown argument: $arg"
            echo "   Usage: $SCRIPT_NAME [--diagnose]"
            exit 1
            ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────────────

die() {
    echo "❌ $1" >&2
    exit 1
}

info() {
    echo "→ $1"
}

ok() {
    echo "   ✓ $1"
}

warn() {
    echo "   ⚠ $1"
}

is_rosetta_service_running() {
    # Check for rosettax87 daemon socket or process
    if [ -S "/var/run/rosetta_helper.sock" ] 2>/dev/null; then
        return 0
    fi
    if pgrep -x "rosettax87" >/dev/null 2>&amp;1; then
        return 0
    fi
    return 1
}

start_rosetta_service() {
    local rosettax87="$1"
    info "Starting rosettax87 service..."
    
    # Check if we can run sudo without password
    if sudo -n true 2>/dev/null; then
        sudo "$rosettax87" &
        sleep 1
    else
        echo "   rosettax87 service requires sudo to create its socket."
        echo "   Please enter your password when prompted:"
        sudo "$rosettax87" &
        sleep 1
    fi
    
    # Verify service started
    if ! is_rosetta_service_running; then
        die "rosettax87 service failed to start. Check sudo permissions."
    fi
    ok "rosettax87 service running"
}

stop_rosetta_service() {
    if is_rosetta_service_running; then
        info "Stopping rosettax87 service..."
        sudo pkill -x "rosettax87" 2>/dev/null || true
        # Give it a moment to clean up
        sleep 0.5
    fi
}

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

# ── Locate CrossOver ─────────────────────────────────────────────────────────

CROSSOVER=""
for candidate in "$HOME/Applications/CrossOver.app" "/Applications/CrossOver.app"; do
    if [ -d "$candidate" ]; then
        CROSSOVER="$candidate"
        break
    fi
done

CX_ROOT="${CROSSOVER:+$CROSSOVER/Contents/SharedSupport/CrossOver}"
CX_HOSTED="${CX_ROOT:+$CX_ROOT/CrossOver-Hosted Application}"

# ── Locate WoW directory ─────────────────────────────────────────────────────

WOW_DIR="${WOW_DIR:-$HOME/Documents/ChromieCraft_3.3.5a}"

WOW_EXE=""
if [ -d "$WOW_DIR" ]; then
    for name in WoW.exe wow.exe Wow.exe; do
        if [ -f "$WOW_DIR/$name" ]; then
            WOW_EXE="$WOW_DIR/$name"
            break
        fi
    done
fi

# ── Diagnose mode ────────────────────────────────────────────────────────────

if [ "$DIAGNOSE" = true ]; then
    echo "═══════════════════════════════════════════════════════════"
    echo "  WoW 3.3.5a Launch Diagnostics"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    
    echo "── Prerequisites ─────────────────────────────────────────"
    
    # WoWSilicon
    if [ -n "$WOWSILICON_RES" ]; then
        ok "WoWSilicon found: $WOWSILICON_RES"
    else
        warn "WoWSilicon not found. Download from https://github.com/WoWSilicon/WoWSilicon"
    fi
    
    # CrossOver
    if [ -n "$CROSSOVER" ]; then
        ok "CrossOver found: $CROSSOVER"
        
        # Check for wineloader (x86, 32-bit)
        if [ -f "$CX_HOSTED/wineloader" ]; then
            ok "wineloader (x86) found"
        else
            warn "wineloader (x86) not found at $CX_HOSTED/wineloader"
        fi
        
        # Check for wineloader2 (patched)
        if [ -f "$CX_HOSTED/wineloader2" ]; then
            ok "wineloader2 (patched x86) found"
        else
            warn "wineloader2 not found (will be created from wineloader)"
        fi
        
        # Check for wineloader64 (x86_64, 64-bit - WRONG for WoW 3.3.5a)
        if [ -f "$CX_HOSTED/wineloader64" ]; then
            warn "wineloader64 (x86_64) found — NOTE: WoW 3.3.5a is 32-bit, use wineloader2"
        fi
    else
        warn "CrossOver not found. Install from codeweavers.com"
    fi
    
    # WoW directory
    if [ -d "$WOW_DIR" ]; then
        ok "WoW directory: $WOW_DIR"
        if [ -n "$WOW_EXE" ]; then
            ok "WoW executable: $(basename "$WOW_EXE")"
        else
            warn "WoW.exe not found in $WOW_DIR"
        fi
    else
        warn "WoW directory not found: $WOW_DIR"
        echo "   Set WOW_DIR=/path/to/client and re-run."
    fi
    
    echo ""
    echo "── Game Files ────────────────────────────────────────────"
    
    if [ -d "$WOW_DIR" ]; then
        for file in d3d9.dll winerosetta.dll libSiliconPatch.dll; do
            if [ -f "$WOW_DIR/$file" ]; then
                ok "$file present"
            else
                warn "$file missing"
            fi
        done
        
        # Check for libDllLdr.dll (should NOT be present)
        if [ -f "$WOW_DIR/libDllLdr.dll" ]; then
            warn "libDllLdr.dll found (not needed, can be removed)"
        fi
        
        # Check dlls.txt
        if [ -f "$WOW_DIR/dlls.txt" ]; then
            ok "dlls.txt exists"
            echo "   Contents:"
            while IFS= read -r line || [ -n "$line" ]; do
                echo "     $line"
            done < "$WOW_DIR/dlls.txt"
        else
            warn "dlls.txt missing"
        fi
    fi
    
    echo ""
    echo "── Rosettax87 ────────────────────────────────────────────"
    
    if [ -d "$WOW_DIR/rosettax87" ]; then
        ok "rosettax87 directory exists"
        if [ -f "$WOW_DIR/rosettax87/rosettax87" ]; then
            ok "rosettax87 binary present"
        else
            warn "rosettax87 binary missing"
        fi
    else
        warn "rosettax87 directory missing"
    fi
    
    if is_rosetta_service_running; then
        ok "rosettax87 service is running"
    else
        warn "rosettax87 service is NOT running (will be auto-started)"
    fi
    
    echo ""
    echo "── Environment ───────────────────────────────────────────"
    echo "   WINEDLLOVERRIDES will be: d3d9=n,b"
    echo "   Launch command will be:  rosettax87 wineloader2 WoW.exe"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  End of diagnostics"
    echo "═══════════════════════════════════════════════════════════"
    exit 0
fi

# ═════════════════════════════════════════════════════════════════════════════
# Normal launch flow
# ═════════════════════════════════════════════════════════════════════════════

# ── Validate prerequisites ───────────────────────────────────────────────────

[ -n "$WOWSILICON_RES" ] || die "WoWSilicon.app not found. Download from https://github.com/WoWSilicon/WoWSilicon"
[ -n "$CROSSOVER" ] || die "CrossOver.app not found. Install from codeweavers.com."
[ -d "$WOW_DIR" ] || die "WoW directory not found: $WOW_DIR\n   Set WOW_DIR=/path/to/client and re-run."
[ -n "$WOW_EXE" ] || die "WoW.exe not found in $WOW_DIR"

# ── Patch CrossOver: create wineloader2 ──────────────────────────────────────

WINELOADER_SRC="$CX_HOSTED/wineloader"
WINELOADER2="$CX_HOSTED/wineloader2"

if [ ! -f "$WINELOADER_SRC" ]; then
    die "wineloader not found at $WINELOADER_SRC"
fi

# Check if wineloader2 already exists and is valid
if [ -f "$WINELOADER2" ] && [ -x "$WINELOADER2" ]; then
    ok "wineloader2 already exists"
else
    info "Creating wineloader2 from wineloader..."
    cp "$WINELOADER_SRC" "$WINELOADER2"
    codesign --remove-signature "$WINELOADER2" 2>/dev/null || true
    chmod +x "$WINELOADER2"
    ok "wineloader2 created and unsigned"
fi

# ── Apply Game Patch ─────────────────────────────────────────────────────────

info "Applying game patch..."

# d3d9.dll (D9VK: DirectX9 → Vulkan → MoltenVK → Metal)
cp "$WOWSILICON_RES/d9vk/d3d9.dll" "$WOW_DIR/d3d9.dll"
ok "d3d9.dll (D9VK)"

# winerosetta.dll — x87 VEH patcher, loaded via dlls.txt
cp "$WOWSILICON_RES/winerosetta/winerosetta.dll" "$WOW_DIR/winerosetta.dll"
ok "winerosetta.dll"

# libSiliconPatch.dll — Silicon-specific patches
cp "$WOWSILICON_RES/libSiliconPatch/wotlk/libSiliconPatch.dll" "$WOW_DIR/libSiliconPatch.dll"
ok "libSiliconPatch.dll"

# Note: libDllLdr.dll is NOT needed for this setup
# Remove it if present from previous runs
if [ -f "$WOW_DIR/libDllLdr.dll" ]; then
    rm "$WOW_DIR/libDllLdr.dll"
    ok "Removed old libDllLdr.dll"
fi

# rosettax87 binaries — the x87 JIT translator
mkdir -p "$WOW_DIR/rosettax87"
cp "$WOWSILICON_RES/rosettax87/rosettax87"            "$WOW_DIR/rosettax87/rosettax87"
cp "$WOWSILICON_RES/rosettax87/libRuntimeRosettax87"  "$WOW_DIR/rosettax87/libRuntimeRosettax87"
chmod 755 "$WOW_DIR/rosettax87/rosettax87" "$WOW_DIR/rosettax87/libRuntimeRosettax87"
# Remove macOS quarantine flag
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/rosettax87" 2>/dev/null || true
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/libRuntimeRosettax87" 2>/dev/null || true
ok "rosettax87 binaries"

# dlls.txt: Wine reads this to load additional DLLs at startup
DLLS_TXT="$WOW_DIR/dlls.txt"
touch "$DLLS_TXT"

# Remove old mods/ entries if present
if grep -qiF "mods/" "$DLLS_TXT" 2>/dev/null; then
    sed -i '' '/^mods\//d' "$DLLS_TXT" 2>/dev/null || true
    ok "Cleaned old mods/ entries from dlls.txt"
fi

# Add root-level DLL entries
for dll in "winerosetta.dll" "libSiliconPatch.dll"; do
    grep -qiF "$dll" "$DLLS_TXT" || echo "$dll" >> "$DLLS_TXT"
done
ok "dlls.txt updated"

# ── Wine environment ─────────────────────────────────────────────────────────

BOTTLE="${CX_BOTTLE:-Win10}"
export CX_ROOT CX_BOTTLE="$BOTTLE"
export WINEPREFIX="$HOME/Library/Application Support/CrossOver/Bottles/$BOTTLE"
export WINESERVER="$CX_HOSTED/wineserver"
export WINELOADER="$WINELOADER2"
export WINEDLLPATH="$CX_ROOT/lib/wine:$CX_ROOT/lib64/wine"
# Only d3d9 override — winerosetta is loaded via dlls.txt
export WINEDLLOVERRIDES="d3d9=n,b"
export DYLD_LIBRARY_PATH="$CX_ROOT/lib:$CX_HOSTED"
export DYLD_FALLBACK_LIBRARY_PATH="$CX_ROOT/lib:/usr/lib"
# D9VK/MoltenVK performance settings
export MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS=1
export DXVK_ASYNC=1

ROSETTAX87="$WOW_DIR/rosettax87/rosettax87"

# ── Start rosettax87 service ─────────────────────────────────────────────────

if ! is_rosetta_service_running; then
    start_rosetta_service "$ROSETTAX87"
else
    ok "rosettax87 service already running"
fi

# ── Cleanup on exit ──────────────────────────────────────────────────────────

cleanup() {
    local exit_code=$?
    echo ""
    info "Cleaning up..."
    # Optionally stop rosettax87 service
    # Uncomment next line to stop service on exit:
    # stop_rosetta_service
    ok "Cleanup complete (exit code: $exit_code)"
}
trap cleanup EXIT INT TERM

# ── Launch ────────────────────────────────────────────────────────────────────

info "Launching WoW via rosettax87 + wineloader2..."
echo "   rosettax87: $ROSETTAX87"
echo "   wineloader2: $WINELOADER2"
echo "   WoW:         $WOW_EXE"
echo "   bottle:      $BOTTLE"
echo ""

cd "$WOW_DIR"
# Note: no 'exec' so cleanup trap can run after game exits
"$ROSETTAX87" "$WINELOADER2" "$WOW_EXE"
