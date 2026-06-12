#!/usr/bin/env bash
# pre-commit.sh — run just check (the canonical pre-commit validation path).
# Called manually or by lefthook; both paths execute identical checks.
set -euo pipefail

cd "$(dirname "$0")/.."
exec just check
