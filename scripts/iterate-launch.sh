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
timeout "$TIMEOUT" "$REPO_ROOT/scripts/launch-wow.sh" || true

echo "→ Launch ended. Classifying log..."
python3 "$REPO_ROOT/tools/launch-diagnostics/classify.py"
