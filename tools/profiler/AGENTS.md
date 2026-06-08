# profiler Package - Agent Guide

## Overview
Python profiling toolkit for WoW.exe on Apple Silicon. Combines Frida dynamic instrumentation with macOS sampling to measure x87 translation overhead and identify bottlenecks.

## Your Workspace
```
tools/profiler/
├── AGENTS.md              # This file
├── pyproject.toml         # Python dependencies (uv)
├── config.toml            # Profiler configuration
├── README.md              # Detailed usage guide
├── attach.py              # Attach Frida to WoW.exe
├── trace_x87.py           # Log x87 instruction patterns
├── sample_cpu.py          # Collect CPU samples via macOS sample command
├── analyze.py             # Process traces and samples
├── schema.py              # Trace data schema validation
└── frida_scripts/         # Frida instrumentation hooks
    ├── x87_tracer.js      # Hook x87 instructions
    ├── memory_trace.js    # Track memory access
    └── hook_points.js     # WoW-specific injection points
```

## Quick Start

### Setup
```bash
cd tools/profiler
uv sync                  # Install dependencies (Frida, etc.)
```

### Profile WoW
```bash
# 1. Attach and start x87 tracing
python3 attach.py

# 2. In another terminal, trace x87 instructions
python3 trace_x87.py --duration 30  # 30 seconds of trace

# 3. Collect CPU samples (macOS native)
python3 sample_cpu.py --duration 30

# 4. Analyze results
python3 analyze.py
# Output: profiling-results.json with overhead breakdown
```

### Typical Workflow
```bash
# Terminal 1: Start WoW
cd ~/Crossover/Bottles/WoW && ./WoW.exe

# Terminal 2: Run profiler
cd tools/profiler
uv sync
python3 attach.py --process WoW.exe
python3 trace_x87.py --output x87_trace_$(date +%s).log
python3 sample_cpu.py --output cpu_sample_$(date +%s).log
python3 analyze.py --trace x87_trace_*.log --samples cpu_sample_*.log
```

## Key Constraints

### DO: Profile WoW.exe only
- x87 translation overhead measurement
- x87 instruction frequency analysis
- CPU hotspot identification

### DO NOT: Disassemble or decompile
- libSiliconPatch.dll is proprietary
- Tracing is permitted; reverse engineering is not
- Hook only public entry points

## Architecture

### attach.py
- Uses Frida to inject into WoW.exe process
- Establishes RPC channel for script communication
- Manages process lifecycle (attach, detach)

### trace_x87.py
- Instruments x87 FPU instruction execution
- Logs: opcode, operands, timestamps, register state
- Frida hooks into `libSiliconPatch!execute_x87_instruction` (inferred)
- Output: `data/profiler/x87_trace_*.log`

### sample_cpu.py
- Wraps macOS `sample` command (system sampling tool)
- Collects statistical CPU profiling data
- Respects System Integrity Protection (SIP) restrictions
- Falls back gracefully if SIP blocks sampling

### analyze.py
- Aggregates x87 traces and CPU samples
- Computes: instruction frequency, overhead per opcode, hottest callstacks
- Generates JSON report for visualization
- Output: `data/profiler/analysis_*.json`

### schema.py
- Dataclass definitions for trace format
- Validates log structure before analysis
- Example:
  ```python
  @dataclass
  class X87Instruction:
      timestamp: float
      opcode: int
      src_operand: Optional[float]
      dst_operand: Optional[float]
      execution_time_us: float
  ```

## Wave 0 Status

✅ **Built and smoke-tested**: All tools compile and attach to test process
✅ **Harness validated**: Injection mechanism confirmed on mock x87 suite (hook_injection_tests.rs)
⏳ **Real-WoW validation**: Pending live WoW profiling session (requires Windows environment)

### Smoke Test
```bash
python3 -m pytest frida_scripts/  # Unit-level script validation
zig build && cargo test --test mre # Rust harness validates injection mock
```

## Guardrails: System Integrity Protection & Hardened Runtime

### On macOS
- **SIP (System Integrity Protection)**: Blocks sampling of system frameworks
  - `sample_cpu.py` detects and reports SIP restrictions
  - Falls back to lighter-weight Frida-based CPU attribution (slower)
  
- **Hardened Runtime**: Requires code signing for process attachment
  - `attach.py` handles entitlement errors gracefully
  - Suggests: `codesign -s - ./attach.py` if needed

### Error Handling
```python
try:
    cpu_trace = sample_cpu(process, duration=30)
except SIPBlockedError:
    print("SIP blocking sampling; using Frida CPU attribution (slower)")
    cpu_trace = frida_cpu_attribution(process, duration=30)
```

## Dependencies
```toml
[tool.poetry.dependencies]
python = "^3.10"
frida = "^16.0"
pydantic = "^2.0"    # Schema validation
click = "^8.0"        # CLI
```

Install via:
```bash
uv sync  # Uses pyproject.toml + uv.lock
```

## Performance Expectations
- x87 tracing overhead: ~5-10% per instruction (Frida hooks)
- CPU sampling accuracy: ±10% (macOS sample() statistical)
- Total profiling impact: ~15-20% slowdown on real workloads

## See Also
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Profiling Guide](../../docs/profiling-guide.md) - Complete profiling workflow
- [Integration](../../packages/integration/AGENTS.md) - Rust MRE harness (validate hooks locally)
- [Zig Glue](../../packages/zig-glue/AGENTS.md) - Windows DLL build
