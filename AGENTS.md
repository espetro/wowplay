# play-wow-on-silicon - Agent Guide

## Project Overview
Working tool for World of Warcraft 3.3.5a on Apple Silicon using a Ports & Adapters architecture.

## Commit Strategy

- Ensure changes are saved in atomic commits
- Follow conventional commit message style

## Quick Start for Agents
1. Read the relevant AGENTS.md for your layer:
   - Root (this file): Project overview, architecture links
   - packages/rust-core/AGENTS.md: Rust domain and FFI work
   - packages/zig-glue/AGENTS.md: Zig cross-compilation to Windows x86 DLL
   - packages/integration/AGENTS.md: Testing and MRE harness
   - tools/profiler/AGENTS.md: Python profiling toolkit (Frida + macOS sample)
   - tools/launch-diagnostics/AGENTS.md: Launch log classification

2. Before making changes:
   - Run `just check` to validate your work (fmt-check + lint + validate)
   - Ensure all checks pass before committing

3. Commit format: Conventional Commits (see docs/git-workflow.md)

## Architecture
See docs/architecture.md for Ports & Adapters design.
See docs/profiling-guide.md for WoW profiling workflow (Frida, x87 tracing, CPU sampling).

## Release Process
See docs/release-process.md for versioning workflow, conventional commits, and automated releases via release-plz.

## Build Systems

This project has three build systems that must stay aligned:

- **Cargo** — builds the `wowplay` CLI and Rust core
- **CMake** (in `vendor/rosettax87_jit/`) — builds `runtime_loader` and `libRuntimeRosettax87`
- **Zig** (`packages/zig-glue/`) — cross-compiles `winerosetta.dll`

The unified task runner is `just`. Use `just check` before every commit. The `wowplay` binary crate includes a `build.rs` that warns if the CMake binaries are stale.

## Key Principles
- Build UPON working libraries, don't replace initially
- Test-first, agent-first development
- Progressive disclosure: each layer has its own AGENTS.md

## Porting Policy
See docs/porting-policy.md for when and why we port to Rust.
