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
   - Run `scripts/pre-commit.sh` to validate your work
   - Ensure all checks pass before committing

3. Commit format: Conventional Commits (see docs/git-workflow.md)

## Architecture
See docs/architecture.md for Ports & Adapters design.
See docs/profiling-guide.md for WoW profiling workflow (Frida, x87 tracing, CPU sampling).

## Key Principles
- Build UPON working libraries, don't replace initially
- Test-first, agent-first development
- Progressive disclosure: each layer has its own AGENTS.md

## Porting Policy
See docs/porting-policy.md for when and why we port to Rust.
