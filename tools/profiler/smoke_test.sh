#!/usr/bin/env bash
# One-button sanity check for the live profiling harness.
# 
# This script:
# 1. Checks uv is installed
# 2. Runs uv sync to ensure deps
# 3. Checks WoW is running
# 4. Runs a 10-second trace
# 5. Runs a 5-second CPU sample
# 6. Runs analyze
# 7. Validates the report against schema
#
# Usage: ./smoke_test.sh

set -e

echo "🧪 WoW Profiling Harness Smoke Test"
echo "===================================="

# Check uv
if ! command -v uv &>/dev/null; then
    echo "❌ uv not found. Install with: mise install uv"
    exit 1
fi
echo "✅ uv found: $(uv --version)"

# Ensure dependencies
echo ""
echo "📦 Ensuring Python dependencies..."
uv sync --quiet

# Check WoW is running
echo ""
echo "🔍 Checking for WoW process..."
if ! uv run python3 attach.py > /dev/null 2>&1; then
    echo "❌ WoW is not running. Please launch WoW via CrossOver first."
    exit 1
fi
PID=$(uv run python3 attach.py)
echo "✅ Found WoW process: PID $PID"

# Run trace
echo ""
echo "⏱️  Running 10-second x87 trace..."
TRACE_FILE=$(uv run python3 trace_x87.py --duration 10 --output-dir ../../data/profiling | grep "Trace saved to:" | awk '{print $NF}')
if [ -z "$TRACE_FILE" ]; then
    echo "❌ Trace failed"
    exit 1
fi
echo "✅ Trace saved: $TRACE_FILE"

# Run CPU sample
echo ""
echo "⏱️  Running 5-second CPU sample..."
SAMPLE_FILE=$(uv run python3 sample_cpu.py --duration 5 --output-dir ../../data/profiling | grep "Parsed samples saved to:" | awk '{print $NF}')
if [ -z "$SAMPLE_FILE" ]; then
    echo "❌ CPU sample failed"
    exit 1
fi
echo "✅ Sample saved: $SAMPLE_FILE"

# Run analysis
echo ""
echo "📊 Analyzing traces..."
REPORT_FILE=$(uv run python3 analyze.py \
    --trace "$TRACE_FILE" \
    --samples "$SAMPLE_FILE" \
    --output-dir ../../data/profiling/reports \
    --duration 15 | grep "Report saved to:" | awk '{print $NF}')

if [ -z "$REPORT_FILE" ]; then
    echo "❌ Analysis failed"
    exit 1
fi
echo "✅ Report saved: $REPORT_FILE"

# Validate report
echo ""
echo "🔍 Validating report..."
if ! uv run python3 schema.py "$REPORT_FILE" > /dev/null 2>&1; then
    echo "❌ Report validation failed"
    exit 1
fi
echo "✅ Report is valid"

# Summary
echo ""
echo "===================================="
echo "🎉 Smoke test passed!"
echo ""
echo "Artifacts:"
echo "  Trace:   $TRACE_FILE"
echo "  Sample:  $SAMPLE_FILE"
echo "  Report:  $REPORT_FILE"
echo ""
echo "Next steps:"
echo "  - Review the report for hot functions"
echo "  - Run full profiling: uv run python3 trace_x87.py --duration 300"
echo "  - Check profiling-guide.md for details"
