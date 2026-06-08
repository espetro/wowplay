# WoW Profiling Guide

Complete workflow for profiling WoW 3.3.5a on Apple Silicon.

## Prerequisites

- macOS 15+ with Apple Silicon
- CrossOver installed with WoW 3.3.5a bottle
- mise/uv toolchains installed

## Quick Start

### 1. Install Dependencies

```bash
# Install Python (if not already installed)
mise install

# Install Python dependencies for profiler
cd tools/profiler
uv sync
```

### 2. Launch WoW via CrossOver

Start WoW 3.3.5a normally through CrossOver. Log into ChromieCraft.

### 3. Find the Process

```bash
# From tools/profiler/
uv run python3 attach.py
```

This returns the PID of the Wine process running WoW.exe.

### 4. Profile x87 Calls

For best results, go to a crowded area like Dalaran:

```bash
# Trace x87 calls for 5 minutes (default)
uv run python3 trace_x87.py --duration 300

# Or trace for 30 seconds for quick testing
uv run python3 trace_x87.py --duration 30
```

The trace saves to `data/profiling/raw_trace_YYYYMMDD_HHMMSS.json`.

### 5. Sample CPU Usage

In another terminal:

```bash
uv run python3 sample_cpu.py --duration 300
```

This saves to `data/profiling/cpu_samples_YYYYMMDD_HHMMSS.json`.

### 6. Generate Report

```bash
# Auto-discover latest trace and sample files
uv run python3 analyze.py --auto

# Or specify files explicitly
uv run python3 analyze.py \
  --trace data/profiling/raw_trace_*.json \
  --samples data/profiling/cpu_samples_*.json
```

The report saves to `data/profiling/reports/report_YYYYMMDD_HHMMSS.json`.

### 7. Validate Report

```bash
uv run python3 schema.py data/profiling/reports/report_*.json
```

## Reading the Report

The JSON report contains:

- **hot_functions**: Ranked list of functions by x87 usage + CPU samples
- **x87_summary**: Total calls, breakdown by function and module
- **environment**: macOS version, CrossOver version, architecture

Example hot function entry:

```json
{
  "rank": 1,
  "address_hex": "0x006A3F20",
  "estimated_name": "CMap::LoadMapTile",
  "source": "combined",
  "sample_count": 45230,
  "x87_call_count": 12891,
  "top_x87_ops": ["fsin", "fcos", "fmul"],
  "suggested_strategy": "Replace fsin/fcos with SSE lookup table"
}
```

## Configuration

Edit `tools/profiler/config.toml` to customize:

- WoW installation path
- Profiling duration defaults
- x87 functions to trace
- Address ranges

Or create `tools/profiler/config.local.toml` (gitignored) for local overrides.

## Adding Function Names

To improve report readability, add known addresses to `data/profiling/address_map.json`:

```json
{
  "functions": [
    {"address": "0x006A3F20", "name": "MyFunction::Name", "category": "custom"}
  ]
}
```

## Troubleshooting

### Frida Permission Denied (SIP)

If Frida fails with permission errors:

```bash
# Use sample-based fallback
uv run python3 trace_x87.py --duration 300 --fallback
```

### No WoW Process Found

Ensure:
- WoW is running via CrossOver
- Only one WoW instance is open
- The process name matches config (default: `WoW.exe`)

### Empty Trace

If trace has 0 calls:
- Verify WoW is actively rendering (don't minimize)
- Check that you're in a busy area with lots of x87 math
- Increase duration (some operations are sporadic)

## Headless Testing (MRE)

Run MRE tests without WoW:

```bash
cd packages/integration
cargo test --test mre
```

These tests validate:
- x87 instruction corpus (≥30 patterns)
- Translation correctness
- Mock hook injection
- Error handling

## One-Button Smoke Test

```bash
cd tools/profiler
./smoke_test.sh
```

This runs the full pipeline in 15 seconds (for quick validation).

## Architecture

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   WoW.exe via   │────▶│  trace_x87   │────▶│ raw_trace_*.json│
│   CrossOver     │     │  (Frida)     │     │                 │
└─────────────────┘     └──────────────┘     └─────────────────┘
         │                                               │
         │                       ┌──────────────┐        │
         └──────────────────────▶│ sample_cpu   │        │
                                 │ (sample cmd) │        │
                                 └──────────────┘        │
                                         │               │
                                         ▼               ▼
                              ┌─────────────────┐  ┌──────────┐
                              │cpu_samples_*.json│  │ address_ │
                              │                 │  │  map.json│
                              └─────────────────┘  └──────────┘
                                         │               │
                                         └───────┬───────┘
                                                 ▼
                                          ┌──────────┐
                                          │ analyze  │
                                          │   .py    │
                                          └──────────┘
                                                 │
                                                 ▼
                                       ┌──────────────────┐
                                       │ report_*.json    │
                                       │ (ranked hot      │
                                       │  functions)      │
                                       └──────────────────┘
```

## See Also

- [Project README](../../README.md)
- [Integration Tests](../../packages/integration/AGENTS.md)
- [Rust Core](../../packages/rust-core/AGENTS.md)
