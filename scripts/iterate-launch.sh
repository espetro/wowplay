#!/usr/bin/env bash
# iterate-launch.sh — timed WoW launch + log classification for agent loop.
#
# Launches WoW via launch-wow.sh, kills it after TIMEOUT seconds, then runs
# classify.py on the captured log and prints the JSON verdict.
#
# Usage:
#   TIMEOUT=60 WOW_DIR=~/Documents/ChromieCraft_3.3.5a ./scripts/iterate-launch.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TIMEOUT="${TIMEOUT:-60}"

echo "→ Starting timed launch (${TIMEOUT}s timeout)..."
timeout "$TIMEOUT" wowplay run --wow-dir "${WOW_DIR:-$HOME/Documents/ChromieCraft_3.3.5a}" || true

echo "→ Launch ended. Classifying log..."
LATEST_LOG=$(ls -t "$HOME/.local/share/wowplay/logs/"*.log 2>/dev/null | head -1)
python3 "$REPO_ROOT/tools/launch-diagnostics/classify.py" "${LATEST_LOG:-}"
