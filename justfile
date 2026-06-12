# play-wow-on-silicon — unified task runner
#
# Build & dev:
#   just setup                    # one-time dev environment setup
#   just build                    # build CLI + GUI
#   just dev                      # start Tauri dev server
#   just test-all                 # run all tests
#
# Validation:
#   just check                    # pre-commit checks (fmt + lint + validate)
#   just fmt                      # auto-format code
#
# Release:
#   just release                  # build, sign, notarize, upload
#   just install                  # build, sign, skip notarize/upload (local install)
#
# WoW launch & profiling:
#   just wow                      # launch WoW (daily use)
#   just profile 30               # profile already-running WoW for 30s
#   just profile-full             # launch → wait → profile → MRE (fully automated)

set quiet
set positional-arguments

# ──────────────────────────────────────────────────────────────────────────────
# Variables (from rosetta-profiler)
# ──────────────────────────────────────────────────────────────────────────────

WOW_DIR          := env_var_or_default("WOW_DIR", env("HOME") + "/Documents/ChromieCraft_3.3.5a")
DURATION         := "300"
WARMUP           := "30"
RUNNER           := env_var_or_default("RUNNER", "whisky")
PROFILER_DIR     := justfile_directory() / "tools/profiler"
PROFILING_OUT    := justfile_directory() / "data/profiling"
WOWPLAY          := justfile_directory() / "target/release/wowplay"
JIT_PROFILE_OUT  := env_var_or_default("ROSETTA_X87_PROFILE_OUT", "/tmp/rosettax87_profile.json")
ROSETTAX87_BIN_DIR := env_var_or_default("ROSETTAX87_BIN_DIR", justfile_directory() / "vendor/rosettax87_jit/build/bin")

# ──────────────────────────────────────────────────────────────────────────────
# Default & info
# ──────────────────────────────────────────────────────────────────────────────

default:
    @just --list

# ──────────────────────────────────────────────────────────────────────────────
# Setup
# ──────────────────────────────────────────────────────────────────────────────

setup:
    bash scripts/setup.sh

# ──────────────────────────────────────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────────────────────────────────────

build-cli:
    cargo build --release -p wowplay

build-zig:
    cd packages/zig-glue && zig build --release=safe

build-rosettax87:
    cmake \
      -B vendor/rosettax87_jit/build \
      -S vendor/rosettax87_jit \
      -DCMAKE_BUILD_TYPE=Release \
      -Wno-dev \
      --log-level=WARNING
    cmake --build vendor/rosettax87_jit/build --config Release

build-sidecar: build-zig build-rosettax87
    bash packages/gui/scripts/build-sidecar.sh

build-gui: build-sidecar
    cd packages/gui && bun run build

build: build-cli build-gui

stage-patching:
    bash scripts/stage-patching.sh

# ──────────────────────────────────────────────────────────────────────────────
# Development
# ──────────────────────────────────────────────────────────────────────────────

dev:
    cd packages/gui && bunx tauri dev

# ──────────────────────────────────────────────────────────────────────────────
# Testing
# ──────────────────────────────────────────────────────────────────────────────

test:
    cargo test --workspace

test-gui:
    cd packages/gui && bun test

test-gauge:
    cd packages/gui && bun run test:gauge

test-all: test test-gui test-gauge

# ──────────────────────────────────────────────────────────────────────────────
# Code quality
# ──────────────────────────────────────────────────────────────────────────────

fmt:
    cargo fmt
    cd packages/gui && bun run fmt
    zig fmt packages/zig-glue/build.zig

fmt-check:
    cargo fmt --check
    cd packages/gui && bun run fmt -- --check
    zig fmt --check packages/zig-glue/build.zig

lint:
    cargo clippy --workspace -- -D warnings
    cd packages/gui && bun run lint

validate:
    cd packages/gui && bun run validate

check-rosetta-freshness:
    bash scripts/check-rosetta-freshness.sh

check: check-rosetta-freshness fmt-check lint validate
    @echo "✅ All checks passed"

cargo-check:
    cargo check --all-targets

# ──────────────────────────────────────────────────────────────────────────────
# Release
# ──────────────────────────────────────────────────────────────────────────────

release *args:
    bash scripts/release.sh {{args}}

install:
    bash scripts/release.sh --skip-notarize --skip-upload
    mkdir -p "${HOME}/.local/share/wowplay"
    cp -r dist/patching "${HOME}/.local/share/wowplay/patching"

# ──────────────────────────────────────────────────────────────────────────────
# WoW launch (ported from rosetta-profiler)
# ──────────────────────────────────────────────────────────────────────────────

# Launch WoW — production rosettax87 from vendor/wowsilicon (daily use)
wow:
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}}

# Launch WoW + libSiliconPatch (daily use)
wow-patch:
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} --enable-lib-silicon

# Launch WoW with ROSETTA_X87_PROFILE=1 — pair with 'just profile' in another terminal
wow-profiled:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}}

# Launch WoW + libSiliconPatch with profiling enabled
wow-profiled-patch:
    ROSETTAX87_BIN_DIR={{ROSETTAX87_BIN_DIR}} \
    ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT={{JIT_PROFILE_OUT}} \
    {{WOWPLAY}} run --wow-dir {{WOW_DIR}} --runner {{RUNNER}} --enable-lib-silicon

# Aliases kept for backward compat
wow-sans-patch: wow
wow-with-patch: wow-patch

# One-time setup with libSiliconPatch
setup-with-patch:
    {{WOWPLAY}} setup --wow-dir {{WOW_DIR}} --enable-lib-silicon \
      --patching-dir {{justfile_directory()}}/packages/gui/patching

# One-time setup without libSiliconPatch
setup-sans-patch:
    {{WOWPLAY}} setup --wow-dir {{WOW_DIR}} \
      --patching-dir {{justfile_directory()}}/packages/gui/patching

# ──────────────────────────────────────────────────────────────────────────────
# Profiling (run in a separate terminal while WoW is live)
# ──────────────────────────────────────────────────────────────────────────────

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

# ──────────────────────────────────────────────────────────────────────────────
# MRE generation
# ──────────────────────────────────────────────────────────────────────────────

# Generate MRE from the latest profiling report
generate-mre:
    @REPORT=$$(ls -t {{PROFILING_OUT}}/reports/report_*.json 2>/dev/null | head -1); \
    if [ -z "$$REPORT" ]; then echo "No reports found. Run 'just profile' first."; exit 1; fi; \
    echo "→ Using report: $$REPORT"; \
    python3 {{PROFILER_DIR}}/build_mre.py "$$REPORT" --output {{justfile_directory()}}/rust_x87_mre; \
    cd {{justfile_directory()}}/rust_x87_mre && cargo check --quiet; \
    echo "→ MRE generated. Run 'cd rust_x87_mre && cargo run --release' to benchmark."

# ──────────────────────────────────────────────────────────────────────────────
# One-command: launch → wait → profile → kill → MRE
# ──────────────────────────────────────────────────────────────────────────────

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

# ──────────────────────────────────────────────────────────────────────────────
# Utilities
# ──────────────────────────────────────────────────────────────────────────────

# Run the profiler smoke test (30s sample, requires WoW running with ROSETTA_X87_PROFILE=1)
smoke-test:
    cd {{PROFILER_DIR}} && ./smoke_test.sh

# Validate all profiling reports against schema
validate-reports:
    cd {{PROFILER_DIR}} && for f in {{PROFILING_OUT}}/reports/*.json; do \
        echo "Validating $$f..."; uv run python3 schema.py "$$f"; \
    done
