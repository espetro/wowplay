# play-wow-on-silicon

Working tool for playing World of Warcraft 3.3.5a on Apple Silicon using a Ports & Adapters architecture.

## Overview

This project enables playing WoW 3.3.5a on Apple Silicon Macs by integrating existing open-source libraries (rosettax87_jit, winerosetta) with a composable, testable architecture.

## Key Features

- **Open Source**: Fully auditable codebase
- **Testable**: MRE harness for headless validation
- **Agent-First**: Progressive enhancement workflow
- **Ports & Adapters**: Composable architecture

## Architecture

Uses Hexagonal (Ports & Adapters) architecture:
- **Domain Layer**: Rust business logic and port definitions
- **Adapter Layer**: FFI wrappers for external libraries
- **Integration Layer**: Testing and CrossOver integration

See [docs/architecture.md](docs/architecture.md) for details.

## Quick Start

### Prerequisites

- macOS (Apple Silicon)
- Rust 1.85+
- Zig 0.13+ (optional)
- CrossOver 26+
- WoW 3.3.5a installation

### Setup

```bash
# Clone repository
git clone https://github.com/yourusername/play-wow-on-silicon.git
cd play-wow-on-silicon

# Run setup script
./scripts/setup.sh

# Install external dependencies
brew install rosettax87_jit winerosetta
```

## Usage

```bash
# Run tests
cargo test

# Launch WoW (after configuration)
cargo run --bin wow_launcher -- --bottle ~/Crossover/Bottles/WoW
```

## Documentation

- [Agent Guide](AGENTS.md) - For developers and AI agents
- [Architecture](docs/architecture.md) - System design
- [Porting Policy](docs/porting-policy.md) - When and why we port to Rust
- [Git Workflow](docs/git-workflow.md) - Commit format and workflow
- [Testing Strategy](docs/testing-strategy.md) - Testing approach

## Project Structure

```
play-wow-on-silicon/
├── AGENTS.md              # Root agent documentation
├── docs/                  # Architecture and policy docs
├── packages/
│   ├── rust-core/        # Rust domain and adapters
│   ├── zig-glue/         # Zig C/C++ interop
│   └── integration/      # Testing harness
└── scripts/              # Development tooling
```

## Contributing

This project uses an agent-first development workflow. See [AGENTS.md](AGENTS.md) for details.

## License

MIT License - See LICENSE file for details

## External Dependencies

- [rosettax87_jit](https://github.com/Lifeisawful/rosettax87_jit) - x87 to AArch64 translation
- [winerosetta](https://github.com/Gcenx/winerosetta) - Wine/CrossOver integration
- [CrossOver](https://www.codeweavers.com/crossover) - Windows API translation

## Status

**Phase 0**: ✅ Foundation & Agent Infrastructure - Complete
**Phase 1**: ⏳ Architecture & Integration Layer - In Progress  
**Phase 2**: 🔜 Working WoW 3.3.5a POC - Not Started
**Phase 3**: 🔜 Progressive Enhancement - Not Started

## Acknowledgments

Built upon the excellent work of:
- @Lifeisawful for rosettax87_jit
- @Gcenx for winerosetta
- The Wine and CrossOver communities
