# Contributing to play-wow-on-silicon

Thanks for your interest! This project uses an **agent-first development workflow** — AI agents and human contributors collaborate through structured documentation and atomic commits.

## Prerequisites

- **macOS** (Apple Silicon)
- **Rust** 1.85+
- **Zig** 0.13+ (optional, used for cross-compiling `winerosetta.dll`)
- **[CrossOver](https://www.codeweavers.com/crossover)** 23+ (external runtime)
- **WoW 3.3.5a** installation (32-bit x86 client)

## Dependency Management

We use [`mise`](https://mise.jdx.dev/) to manage tool versions (Zig, and potentially Rust in the future). If you already have Rust and Zig installed globally, `mise` is optional.

```bash
# Install mise (optional, but recommended)
brew install mise

# Install tools via mise
mise install
```

The setup script also checks common install locations (Homebrew, `~/.cargo`, `~/.local/share/mise`) and falls back gracefully.

## Setup

```bash
# Clone repository
git clone https://github.com/yourusername/play-wow-on-silicon.git
cd play-wow-on-silicon

# Run setup (inits submodules, builds rosettax87_jit via CMake, installs git hooks)
./scripts/setup.sh
```

This will:
1. Initialise and build `vendor/rosettax87_jit` (CMake, Release)
2. Install the pre-commit git hook via lefthook

## Build

```bash
# Build CLI (Cargo) — also warns if rosettax87_jit binaries are stale
cargo build -p wowplay

# Build everything (CLI + GUI sidecar + rosettax87_jit)
just build
```

## Development Workflow

We follow an **agent-first** approach. Before touching code, read the relevant layer guide:

- [Root AGENTS.md](AGENTS.md) — Project overview and conventions
- [packages/rust-core/AGENTS.md](packages/rust-core/AGENTS.md) — Rust domain and FFI work
- [packages/zig-glue/AGENTS.md](packages/zig-glue/AGENTS.md) — Zig cross-compilation
- [packages/integration/AGENTS.md](packages/integration/AGENTS.md) — Testing and MRE harness

### Commit Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add x87 opcode fallback for fsin
fix: correct Wine loader path on macOS 15
docs: update troubleshooting for CrossOver 26
```

Run `scripts/pre-commit.sh` before committing to validate your work.

## Architecture

The project uses **Hexagonal (Ports & Adapters)** architecture:

- **Domain Layer** (`packages/rust-core/`) — Rust business logic and port definitions
- **Adapter Layer** (`packages/zig-glue/`, FFI wrappers) — Bridges to external libraries
- **Integration Layer** (`packages/integration/`) — Testing harness and CrossOver integration

See [docs/architecture.md](docs/architecture.md) for the full system design.

## Project Structure

```
play-wow-on-silicon/
├── AGENTS.md              # Root agent documentation
├── CONTRIBUTING.md        # This file
├── docs/                  # Architecture and policy docs
│   ├── architecture.md
│   ├── porting-policy.md
│   ├── git-workflow.md
│   ├── testing-strategy.md
│   └── release-process.md
├── packages/
│   ├── rust-core/        # Rust domain and adapters
│   ├── cli/              # wowplay CLI binary
│   ├── zig-glue/         # Zig C/C++ interop
│   └── integration/      # Testing harness
├── scripts/              # Development tooling
└── vendor/               # External submodules
    ├── rosettax87_jit/   # x87 JIT translation
    └── wowsilicon/       # Swift launcher (patching resources)
```

## Testing

```bash
# Run the full Rust test suite
cargo test

# Run integration / MRE harness
cargo test -p wow-silicon-integration

# Diagnose a WoW install without launching
./scripts/launch-wow.sh --diagnose
```

See [docs/testing-strategy.md](docs/testing-strategy.md) for our testing philosophy.

## How to Contribute

1. **Open an issue** first for bugs, features, or architectural questions.
2. **Fork and branch** from `main`.
3. **Write tests** for new behaviour (see [docs/testing-strategy.md](docs/testing-strategy.md)).
4. **Run `scripts/pre-commit.sh`** to validate formatting, clippy, and tests.
5. **Open a PR** with a clear description referencing the issue.

## Porting Policy

When and why we port code to Rust is governed by [docs/porting-policy.md](docs/porting-policy.md). In short: build upon working libraries first, port incrementally when the abstraction boundary is clear.

## Release Process

Releases are automated with [release-plz](https://github.com/MarcoIeni/release-plz). Pushing to `main` triggers the release workflow (`.github/workflows/release.yml`). See [docs/release-process.md](docs/release-process.md) for versioning details.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
