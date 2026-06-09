# WoW Silicon Profiler

Profiling harness for World of Warcraft 3.3.5a on Apple Silicon.

## Quick Start

```bash
# Install dependencies
uv sync

# Verify Frida is working
uv run frida-trace --version

# Find and attach to WoW process
uv run python3 attach.py

# Trace x87 calls for 5 minutes
uv run python3 trace_x87.py --duration 300

# Sample CPU usage
uv run python3 sample_cpu.py --duration 300

# Generate ranked report
uv run python3 analyze.py
```

## Configuration

Edit `config.toml` to set your WoW and CrossOver paths:

```toml
[paths]
wow_dir = "/path/to/WoW_3.3.5a"
crossover_app = "/Applications/CrossOver.app"
```

Or create `config.local.toml` (gitignored) to override without modifying tracked files.

## Scripts

- **attach.py** — Find and attach to the Wine process running WoW.exe
- **trace_x87.py** — Trace x87 math calls via Frida
- **sample_cpu.py** — CPU sampling via macOS `sample` command
- **analyze.py** — Aggregate traces and samples into a ranked report
- **schema.py** — JSON report schema definition and validation

## Frida Scripts

- **frida_scripts/x87_hook.js** — Hook msvcrt.dll math functions
- **frida_scripts/d3d9_hook.js** — Hook D3D9 draw calls
- **frida_scripts/wine_filter.js** — Filter WoW vs Wine address ranges

## Architecture

```
tools/profiler/
├── pyproject.toml          # uv project config
├── config.toml             # Profiling parameters
├── attach.py               # Process discovery
├── trace_x87.py            # x87 call tracer
├── sample_cpu.py           # CPU sampler
├── analyze.py              # Report generator
├── schema.py               # JSON schema
├── frida_scripts/
│   ├── x87_hook.js         # x87 math hooks
│   ├── d3d9_hook.js        # D3D9 hooks
│   └── wine_filter.js      # Address filter
└── smoke_test.sh           # One-button sanity check
```

## Requirements

- macOS 15+ with Apple Silicon
- CrossOver (running WoW 3.3.5a)
- Python 3.12+ (managed by mise/uv)
- Frida 16+ (installed by uv)

## Guardrails

- Never disassemble or analyze `libSiliconPatch.dll` (it is optional and proprietary)
- Profile `WoW.exe` only
- Handle SIP/hardened runtime gracefully (falls back to `sample`)
- Distinguish Wine x87 from WoW x87 via address range filtering

## See Also

- [Profiling Guide](../../docs/profiling-guide.md) — Full workflow documentation
- [Project README](../../README.md) — Project overview
