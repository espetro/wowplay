#!/usr/bin/env bash
# DEPRECATED: superseded by `wowplay run`. Kept as a debugging reference.
# New contributors: use `wowplay setup` + `wowplay run` instead.
# Launch WoW 3.3.5a on Apple Silicon via CrossOver + rosettax87.
#
# Usage:
#   WOW_DIR=/path/to/WoW/client ./scripts/launch-wow.sh [--diagnose]
#
# Prerequisites:
#   - CrossOver.app installed in ~/Applications or /Applications
#   - WoW 3.3.5a client with DivxDecoder.dll (ChromieCraft 3.3.5a ships it)
#   - WoWSilicon.app (for d3d9.dll, winerosetta.dll, libSiliconPatch.dll, libDllLdr.dll)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
DIAGNOSE=false

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
    if [ -S "/var/run/rosetta_helper.sock" ] 2>/dev/null; then
        return 0
    fi
    if pgrep -x "rosettax87" >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

start_rosetta_service() {
    local rosettax87="$1"
    info "Starting rosettax87 service..."
    if sudo -n true 2>/dev/null; then
        sudo "$rosettax87" &
        sleep 1
    else
        echo "   rosettax87 requires sudo. Enter your password when prompted:"
        sudo "$rosettax87" &
        sleep 1
    fi
    if ! is_rosetta_service_running; then
        die "rosettax87 service failed to start. Check sudo permissions."
    fi
    ok "rosettax87 service running"
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

    if [ -n "$WOWSILICON_RES" ]; then
        ok "WoWSilicon found: $WOWSILICON_RES"
    else
        warn "WoWSilicon not found. Download from https://github.com/WoWSilicon/WoWSilicon"
    fi

    if [ -n "$CROSSOVER" ]; then
        ok "CrossOver found: $CROSSOVER"
        if [ -f "$CX_HOSTED/wineloader" ]; then
            ok "wineloader found"
        else
            warn "wineloader not found at $CX_HOSTED/wineloader"
        fi
        if [ -f "$CX_HOSTED/wineloader2" ] && file "$CX_HOSTED/wineloader2" | grep -q "x86_64"; then
            ok "wineloader2 (x86_64) found"
        else
            info "wineloader2 not found or wrong arch (will be created from wineloader)"
        fi
    else
        warn "CrossOver not found. Install from codeweavers.com"
    fi

    if [ -d "$WOW_DIR" ]; then
        ok "WoW directory: $WOW_DIR"
        [ -n "$WOW_EXE" ] && ok "WoW executable: $(basename "$WOW_EXE")" || warn "WoW.exe not found in $WOW_DIR"
    else
        warn "WoW directory not found: $WOW_DIR"
        echo "   Set WOW_DIR=/path/to/client and re-run."
    fi

    echo ""
    echo "── Game Files ────────────────────────────────────────────"

    if [ -d "$WOW_DIR" ]; then
        for file in d3d9.dll libDllLdr.dll; do
            [ -f "$WOW_DIR/$file" ] && ok "$file present" || warn "$file missing"
        done

        if [ -f "$WOW_DIR/DivxDecoder.dll.bak" ]; then
            ok "DivxDecoder.dll patched (DivxDecoder.dll.bak present)"
        elif [ -f "$WOW_DIR/DivxDecoder.dll" ]; then
            ok "DivxDecoder.dll present (not yet patched — will patch on next launch)"
        else
            warn "DivxDecoder.dll missing — winerosetta cannot be injected. Reinstall client."
        fi

        for file in mods/winerosetta.dll mods/libSiliconPatch.dll; do
            [ -f "$WOW_DIR/$file" ] && ok "$file present" || warn "$file missing"
        done

        if [ -f "$WOW_DIR/dlls.txt" ]; then
            ok "dlls.txt exists"
            while IFS= read -r line || [ -n "$line" ]; do
                echo "     $line"
            done < "$WOW_DIR/dlls.txt"
        else
            warn "dlls.txt missing"
        fi
    fi

    echo ""
    echo "── Rosettax87 ────────────────────────────────────────────"

    if [ -f "$WOW_DIR/rosettax87/rosettax87" ]; then
        ok "rosettax87 binary present"
    else
        warn "rosettax87 binary missing"
    fi

    is_rosetta_service_running && ok "rosettax87 service is running" \
        || warn "rosettax87 service NOT running (will be auto-started)"

    echo ""
    echo "── Environment ───────────────────────────────────────────"
    echo "   WINEDLLOVERRIDES: d3d9=n,b"
    echo "   Launch chain:     rosettax87 → wineloader2 → WoW.exe"
    echo "   DivxDecoder bootstrap (one-time):"
    echo "     wineloader2 rundll32 libDllLdr.dll,PatchDivxDecoder <wow_dir>"
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
[ -n "$CROSSOVER" ]      || die "CrossOver.app not found. Install from codeweavers.com."
[ -d "$WOW_DIR" ]        || die "WoW directory not found: $WOW_DIR — set WOW_DIR=/path/to/client."
[ -n "$WOW_EXE" ]        || die "WoW.exe not found in $WOW_DIR"

# DivxDecoder.dll (or its backup) must exist — it is the winerosetta injection anchor.
if [ ! -f "$WOW_DIR/DivxDecoder.dll" ] && [ ! -f "$WOW_DIR/DivxDecoder.dll.bak" ]; then
    die "DivxDecoder.dll not found in $WOW_DIR. The client may be corrupt — please reinstall."
fi

# ── Patch CrossOver: create wineloader2 ──────────────────────────────────────
# macOS 10.15+ cannot exec i386 Mach-O binaries. wineloader is already x86_64
# on CrossOver 24. Copy it to wineloader2 and strip the code signature so
# rosettax87 can install its JIT hooks before Wine starts.

WINELOADER_SRC="$CX_HOSTED/wineloader"
WINELOADER2="$CX_HOSTED/wineloader2"

[ -f "$WINELOADER_SRC" ] || die "wineloader not found at $WINELOADER_SRC"

if [ -f "$WINELOADER2" ] && [ -x "$WINELOADER2" ] && file "$WINELOADER2" | grep -q "x86_64"; then
    ok "wineloader2 (x86_64) already exists"
else
    [ -f "$WINELOADER2" ] && rm -f "$WINELOADER2"
    info "Creating wineloader2 (unsigned x86_64)..."
    cp "$WINELOADER_SRC" "$WINELOADER2"
    codesign --remove-signature "$WINELOADER2" 2>/dev/null || true
    chmod +x "$WINELOADER2"
    ok "wineloader2 created and unsigned"
fi

# ── Apply Game Patch ─────────────────────────────────────────────────────────
# Mirrors WoWSilicon applyGamePatch:
#   mods/   → winerosetta.dll, libSiliconPatch.dll
#   root/   → d3d9.dll, libDllLdr.dll
# Never touches DivxDecoder.dll or any other original client DLL.

info "Applying game patch..."

mkdir -p "$WOW_DIR/mods"

cp "$WOWSILICON_RES/winerosetta/winerosetta.dll"                    "$WOW_DIR/mods/winerosetta.dll"
ok "mods/winerosetta.dll"

cp "$WOWSILICON_RES/libSiliconPatch/wotlk/libSiliconPatch.dll"      "$WOW_DIR/mods/libSiliconPatch.dll"
ok "mods/libSiliconPatch.dll"

cp "$WOWSILICON_RES/d9vk/d3d9.dll"                                  "$WOW_DIR/d3d9.dll"
ok "d3d9.dll (D9VK)"

cp "$WOWSILICON_RES/winerosetta/libDllLdr.dll"                      "$WOW_DIR/libDllLdr.dll"
ok "libDllLdr.dll (DivxDecoder bootstrap)"

mkdir -p "$WOW_DIR/rosettax87"
cp "$WOWSILICON_RES/rosettax87/rosettax87"            "$WOW_DIR/rosettax87/rosettax87"
cp "$WOWSILICON_RES/rosettax87/libRuntimeRosettax87"  "$WOW_DIR/rosettax87/libRuntimeRosettax87"
chmod 755 "$WOW_DIR/rosettax87/rosettax87" "$WOW_DIR/rosettax87/libRuntimeRosettax87"
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/rosettax87" 2>/dev/null || true
xattr -d com.apple.quarantine "$WOW_DIR/rosettax87/libRuntimeRosettax87" 2>/dev/null || true
ok "rosettax87 binaries"

# dlls.txt — mirrors updateDllsTxt: mods/ paths only.
# Removes legacy root-level entries that would cause double-loading.
DLLS_TXT="$WOW_DIR/dlls.txt"
touch "$DLLS_TXT"
sed -i '' -e '/^winerosetta\.dll$/Id' -e '/^libSiliconPatch\.dll$/Id' "$DLLS_TXT" 2>/dev/null || true
for entry in "mods/winerosetta.dll" "mods/libSiliconPatch.dll"; do
    grep -qiF "$entry" "$DLLS_TXT" || printf '%s\n' "$entry" >> "$DLLS_TXT"
done
ok "dlls.txt updated (mods/ layout)"

# ── Wine environment ─────────────────────────────────────────────────────────

BOTTLE="${CX_BOTTLE:-Win10}"
export CX_ROOT CX_BOTTLE="$BOTTLE"
export WINEPREFIX="$HOME/Library/Application Support/CrossOver/Bottles/$BOTTLE"
export WINESERVER="$CX_HOSTED/wineserver"
export WINELOADER="$WINELOADER2"

# ── DivxDecoder bootstrap (idempotent, one-time) ─────────────────────────────
# winerosetta cannot self-load; libDllLdr.dll patches DivxDecoder.dll so that
# Wine's native DLL loader brings winerosetta in on game startup. winerosetta
# then reads dlls.txt to load libSiliconPatch.dll.
# Presence of DivxDecoder.dll.bak means the patch was already applied.

if [ ! -f "$WOW_DIR/DivxDecoder.dll.bak" ]; then
    info "Bootstrapping winerosetta via DivxDecoder (one-time)..."
    (
        cd "$WOW_DIR"
        WINEDLLOVERRIDES="winemenubuilder.exe=d;mscoree=d;mshtml=d" \
        WINEDEBUG=-all \
            "$WINELOADER2" rundll32 "libDllLdr.dll,PatchDivxDecoder" "$WOW_DIR"
    )
    ok "DivxDecoder.dll patched → DivxDecoder.dll.bak"

    if [ -f "$WOW_DIR/DivxTac.dll" ] && [ ! -f "$WOW_DIR/DivxTac.dll.bak" ]; then
        (
            cd "$WOW_DIR"
            WINEDLLOVERRIDES="winemenubuilder.exe=d;mscoree=d;mshtml=d" \
            WINEDEBUG=-all \
                "$WINELOADER2" rundll32 "libDllLdr.dll,PatchDivxTac" "$WOW_DIR"
        )
        ok "DivxTac.dll patched → DivxTac.dll.bak"
    fi
else
    ok "DivxDecoder.dll already patched (skipping bootstrap)"
fi

# ── Start rosettax87 service ─────────────────────────────────────────────────

ROSETTAX87="$WOW_DIR/rosettax87/rosettax87"

if ! is_rosetta_service_running; then
    start_rosetta_service "$ROSETTAX87"
else
    ok "rosettax87 service already running"
fi

# ── Log capture setup ────────────────────────────────────────────────────────

LOG_DIR="$REPO_ROOT/data/launch"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/$(date +%Y%m%dT%H%M%S).log"
WINEDEBUG_PROFILE="${WINEDEBUG:-warn+all,err+all,+loaddll,+module}"

# ── Cleanup on exit ──────────────────────────────────────────────────────────

cleanup() {
    local exit_code=$?
    echo ""
    info "Cleanup complete (exit: $exit_code) — log: $LOG_FILE"
}
trap cleanup EXIT INT TERM

# ── Launch ────────────────────────────────────────────────────────────────────
# Proven chain (mirrors turtlesilicon continueLaunch + WoWSilicon v25OrLower):
#   rosettax87 → wineloader2 → WoW.exe
# No winewrapper.exe, no WINEDLLPATH, no DYLD overrides.

info "Launching WoW..."
echo "   rosettax87:  $ROSETTAX87"
echo "   wineloader2: $WINELOADER2"
echo "   WoW:         $WOW_EXE"
echo "   bottle:      $BOTTLE"
echo "   log:         $LOG_FILE"
echo ""

cd "$WOW_DIR"
WINEDLLOVERRIDES="d3d9=n,b" \
    MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS=1 \
    DXVK_ASYNC=1 \
    MTL_HUD_ENABLED=0 \
    WINEDEBUG="$WINEDEBUG_PROFILE" \
    "$ROSETTAX87" "$WINELOADER2" "$WOW_EXE" 2>&1 | tee "$LOG_FILE"
