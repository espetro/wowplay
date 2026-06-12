#!/usr/bin/env bash
# One-button sanity check for the JIT-counter + CPU-sample profiling harness.
#
# Prerequisites:
#   - WoW is running, launched with:
#       ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT=/tmp/rosettax87_profile.json
#   - You are in tools/profiler/

set -e

echo "WoW Profiling Harness Smoke Test"
echo "================================="

# Check uv
if ! command -v uv &>/dev/null; then
    echo "ERROR: uv not found. Install with: mise install uv"
    exit 1
fi
echo "uv: $(uv --version)"

# Ensure dependencies
echo ""
echo "Ensuring Python dependencies..."
uv sync --quiet

# Check WoW is running
echo ""
echo "Checking for WoW process..."
if ! WOW_PID=$(uv run python3 attach.py 2>/dev/null); then
    echo "ERROR: WoW is not running or ROSETTA_X87_PROFILE=1 was not set at launch."
    echo "  Launch WoW first:"
    echo "    ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT=/tmp/rosettax87_profile.json just wow-sans-patch"
    exit 1
fi
echo "Found WoW process: PID $WOW_PID"

# Collect a short CPU sample
echo ""
echo "Running 10-second CPU sample..."
SAMPLE_FILE=$(uv run python3 sample_cpu.py --duration 10 --output-dir ../../data/profiling \
    | grep "Parsed samples saved to:" | awk '{print $NF}')
if [ -z "$SAMPLE_FILE" ]; then
    echo "ERROR: CPU sample failed"
    exit 1
fi
echo "Sample saved: $SAMPLE_FILE"

# Dump JIT x87 counts via SIGUSR2
echo ""
echo "Requesting JIT x87 opcode dump (SIGUSR2)..."
if kill -USR2 "$WOW_PID" 2>/dev/null; then
    sleep 1
    echo "Dump requested."
else
    echo "WARNING: Could not send SIGUSR2 to PID $WOW_PID; will use existing dump if present."
fi

# Show JIT counts summary
JIT_FILE="${ROSETTA_X87_PROFILE_OUT:-/tmp/rosettax87_profile.json}"
if [ -f "$JIT_FILE" ]; then
    echo ""
    echo "JIT x87 opcode counts ($JIT_FILE):"
    uv run python3 ingest_jit_counts.py "$JIT_FILE" || true
else
    echo "WARNING: JIT profile file not found at $JIT_FILE"
    echo "  Ensure WoW was launched with ROSETTA_X87_PROFILE=1"
fi

# Run analysis
echo ""
echo "Analyzing..."
REPORT_FILE=$(uv run python3 analyze.py \
    --samples "$SAMPLE_FILE" \
    --output-dir ../../data/profiling/reports \
    --duration 10 \
    | grep "Report saved to:" | awk '{print $NF}')

if [ -z "$REPORT_FILE" ]; then
    echo "ERROR: Analysis failed"
    exit 1
fi
echo "Report saved: $REPORT_FILE"

# Validate report
echo ""
echo "Validating report..."
if ! uv run python3 schema.py "$REPORT_FILE" > /dev/null 2>&1; then
    echo "ERROR: Report validation failed"
    exit 1
fi
echo "Report is valid"

echo ""
echo "================================="
echo "Smoke test passed!"
echo ""
echo "Artifacts:"
echo "  Sample:  $SAMPLE_FILE"
echo "  JIT:     $JIT_FILE"
echo "  Report:  $REPORT_FILE"
echo ""
echo "Next steps:"
echo "  - Full profile: just profile (5 min)"
echo "  - See docs/profiling-guide.md for the complete workflow"
