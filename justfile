# play-wow-on-silicon — profiling & launch commands
#
# Quick start (two terminals):
#   just wow          # terminal 1: launch WoW with JIT profiling enabled
#   just profile 30   # terminal 2: collect 30s sample + dump + analyse
#
# Or fully automated (single command):
#   just profile-full             # launch → wait → profile 5 min → dump → analyse → MRE
#   just profile-full-patch       # same, with libSiliconPatch
#
# Other recipes:
#   just profile [DURATION]       # profile already-running WoW (default 300s)
#   just wow-patch                # launch WoW + libSiliconPatch with profiling
#   just wow-sans-patch           # launch WoW (no patch, no profiling — plain play)
#   just wow-with-patch           # launch WoW + libSiliconPatch (no profiling — plain play)

WOW_DIR          := env_var_or_default("WOW_DIR", env("HOME") + "/Documents/ChromieCraft_3.3.5a")
DURATION         := "300"
WARMUP           := "30"
RUNNER           := env_var_or_default("RUNNER", "whisky")
PROFILER_DIR     := justfile_directory() / "tools/profiler"
PROFILING_OUT    := justfile_directory() / "data/profiling"
WOWPLAY          := justfile_directory() / "target/release/wowplay"
JIT_PROFILE_OUT  := env_var_or_default("ROSETTA_X87_PROFILE_OUT", "/tmp/rosettax87_profile.json")
# Directory containing the instrumented runtime_loader + libRuntimeRosettax87 binaries.
# Defaults to the local CMake build output; override with ROSETTAX87_BIN_DIR env var.
ROSETTAX87_BIN_DIR := env_var_or_default("ROSETTAX87_BIN_DIR", justfile_directory() / "vendor/rosettax87_jit/build/bin")

# ------------------------------------------------------------------------------
# WoW launch
# ------------------------------------------------------------------------------

# Launch WoW with JIT x87 profiling enabled (pair with 'just profile' in another terminal)
wow:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}}

# Launch WoW + libSiliconPatch with JIT profiling enabled
wow-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} --enable-lib-silicon

# Launch WoW (no patch, no profiling — plain play)
wow-sans-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}}

# Launch WoW with libSiliconPatch (no profiling — plain play)
wow-with-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} --enable-lib-silicon

# One-time setup with libSiliconPatch
setup-with-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    {{WOWPLAY}} setup --wow-dir {{WOW_DIR}} --enable-lib-silicon

# One-time setup without libSiliconPatch
setup-sans-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    {{WOWPLAY}} setup --wow-dir {{WOW_DIR}}

# ------------------------------------------------------------------------------
# Profiling (run in a separate terminal while WoW is live)
# ------------------------------------------------------------------------------

# Ensure profiler Python deps are installed
profiler-setup:
    cd {{PROFILER_DIR}} && uv sync --quiet

# Profile an already-running WoW (launched with ROSETTA_X87_PROFILE=1):
#   1. Collect CPU samples for DURATION seconds
#   2. Send SIGUSR2 to dump JIT x87 opcode counts (no sudo required)
#   3. Analyze and produce a report
profile-run DURATION="300":
    @echo "→ Collecting CPU samples for {{DURATION}}s..."
    cd {{PROFILER_DIR}} && uv sync --quiet && uv run python3 sample_cpu.py --duration {{DURATION}}
    @echo "→ Requesting JIT x87 dump (SIGUSR2)..."
    @WOW_PID=$$(cd {{PROFILER_DIR}} && uv run python3 attach.py 2>/dev/null); \
        if [ -n "$$WOW_PID" ]; then \
            kill -USR2 $$WOW_PID 2>/dev/null && sleep 1 && echo "  Dumped JIT counts for PID $$WOW_PID"; \
        else \
            echo "  (WoW process not found; using existing JIT dump if present)"; \
        fi
    cd {{PROFILER_DIR}} && uv run python3 analyze.py --auto
    @echo "→ Done. Run 'just generate-mre' to build the benchmark crate."

# Profile already-running WoW; DURATION defaults to 300s
# Usage: just profile        (5 min)
#        just profile 30     (30 s quick check)
profile DURATION="300": (profile-run DURATION)

# ------------------------------------------------------------------------------
# MRE generation
# ------------------------------------------------------------------------------

# Generate MRE from the latest profiling report
generate-mre:
    @REPORT=$$(ls -t {{PROFILING_OUT}}/reports/report_*.json 2>/dev/null | head -1); \
    if [ -z "$$REPORT" ]; then echo "No reports found. Run 'just profile' first."; exit 1; fi; \
    echo "→ Using report: $$REPORT"; \
    python3 {{PROFILER_DIR}}/build_mre.py "$$REPORT" --output {{justfile_directory()}}/rust_x87_mre; \
    cd {{justfile_directory()}}/rust_x87_mre && cargo check --quiet; \
    echo "→ MRE generated. Run 'cd rust_x87_mre && cargo run --release' to benchmark."

# ------------------------------------------------------------------------------
# One-command: launch → wait → profile → kill → MRE
# ------------------------------------------------------------------------------

# Launch WoW (no patch) with x87 profiling enabled, wait WARMUP seconds, profile
# for DURATION, dump JIT counts, then kill and generate MRE.
profile-full DURATION="300" WARMUP="30":
    @echo "→ Launching WoW (no libSiliconPatch) with ROSETTA_X87_PROFILE=1..."
    @ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
        ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
        {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} &> /dev/null & \
        WOW_PID=$$!; \
        echo "→ WoW PID: $$WOW_PID — waiting {{WARMUP}}s for login..."; \
        sleep {{WARMUP}}; \
        echo "→ Collecting CPU samples for {{DURATION}}s..."; \
        cd {{PROFILER_DIR}} && uv sync --quiet && uv run python3 sample_cpu.py --duration {{DURATION}}; \
        echo "→ Dumping JIT x87 counts (SIGUSR2)..."; \
        kill -USR2 $$WOW_PID 2>/dev/null && sleep 1 || true; \
        cd {{PROFILER_DIR}} && uv run python3 analyze.py --auto; \
        echo "→ Killing WoW..."; \
        kill $$WOW_PID 2>/dev/null || true; \
        wait $$WOW_PID 2>/dev/null || true; \
        echo "→ Generating MRE..."; \
        REPORT=$$(ls -t {{PROFILING_OUT}}/reports/report_*.json 2>/dev/null | head -1); \
        if [ -n "$$REPORT" ]; then \
            python3 {{PROFILER_DIR}}/build_mre.py "$$REPORT" --output {{justfile_directory()}}/rust_x87_mre; \
            cd {{justfile_directory()}}/rust_x87_mre && cargo check --quiet; \
            echo "→ Done. Report: $$REPORT"; \
        else \
            echo "→ No report generated — check profiler output above."; \
        fi

# Same but with libSiliconPatch enabled
profile-full-patch DURATION="300" WARMUP="30":
    @echo "→ Launching WoW (with libSiliconPatch) with ROSETTA_X87_PROFILE=1..."
    @ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
        ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
        {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} --enable-lib-silicon &> /dev/null & \
        WOW_PID=$$!; \
        echo "→ WoW PID: $$WOW_PID — waiting {{WARMUP}}s for login..."; \
        sleep {{WARMUP}}; \
        echo "→ Collecting CPU samples for {{DURATION}}s..."; \
        cd {{PROFILER_DIR}} && uv sync --quiet && uv run python3 sample_cpu.py --duration {{DURATION}}; \
        echo "→ Dumping JIT x87 counts (SIGUSR2)..."; \
        kill -USR2 $$WOW_PID 2>/dev/null && sleep 1 || true; \
        cd {{PROFILER_DIR}} && uv run python3 analyze.py --auto; \
        echo "→ Killing WoW..."; \
        kill $$WOW_PID 2>/dev/null || true; \
        wait $$WOW_PID 2>/dev/null || true; \
        echo "→ Generating MRE..."; \
        REPORT=$$(ls -t {{PROFILING_OUT}}/reports/report_*.json 2>/dev/null | head -1); \
        if [ -n "$$REPORT" ]; then \
            python3 {{PROFILER_DIR}}/build_mre.py "$$REPORT" --output {{justfile_directory()}}/rust_x87_mre; \
            cd {{justfile_directory()}}/rust_x87_mre && cargo check --quiet; \
            echo "→ Done. Report: $$REPORT"; \
        else \
            echo "→ No report generated — check profiler output above."; \
        fi

# ------------------------------------------------------------------------------
# Utilities
# ------------------------------------------------------------------------------

# Run the profiler smoke test (30s sample, requires WoW running with ROSETTA_X87_PROFILE=1)
smoke-test:
    cd {{PROFILER_DIR}} && ./smoke_test.sh

# Validate all profiling reports against schema
validate-reports:
    cd {{PROFILER_DIR}} && for f in {{PROFILING_OUT}}/reports/*.json; do \
        echo "Validating $$f..."; uv run python3 schema.py "$$f"; \
    done

# Check everything compiles
check:
    cargo check --all-targets
    cargo check --manifest-path {{justfile_directory()}}/rust_x87_mre/Cargo.toml --lib
