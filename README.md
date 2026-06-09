# play-wow-on-silicon

> **Run World of Warcraft 3.3.5a on Apple Silicon.**
>
> Built around instruction-set bridging — x87 FPU emulation, Wine/CrossOver integration, and runtime patching — so 32-bit x86 games from the 2006–2010 era run efficiently on modern macOS. The architecture is designed to extend to other legacy apps that struggle with missing instruction sets.

## Download

[![Latest Release](https://img.shields.io/github/v/release/espetro/play-wow-on-silicon?style=flat&label=latest)](https://github.com/yourusername/play-wow-on-silicon/releases/latest)

**macOS only (Apple Silicon)** · Requires [CrossOver](https://www.codeweavers.com/crossover), [Whisky](https://github.com/Whisky-App/Whisky), or [Moonshine](https://github.com/ybmeng/moonshine) — any one will do.

1. Install one of the runners above.
2. Have a WoW 3.3.5a client folder ready.
3. Download the latest release zip above and unzip it.
4. Run `./wowplay setup --wow-dir ~/Documents/WoW_3.3.5a`
   (detects runners, stages patching resources, applies patches)
5. Run `./wowplay run --wow-dir ~/Documents/WoW_3.3.5a`

See [CONTRIBUTING.md](CONTRIBUTING.md) if you want to build from source or contribute.

## Features

- **One-command launch** — `wowplay run` handles CrossOver patching, x87 emulation, and Wine loader setup automatically
- **x87 FPU emulation** — bridges missing x87 instructions to AArch64 via RosettaX87 so the 32-bit client executes correctly
- **Automated patching** — applies game-folder and CrossOver patches needed for the RosettaX87 launch path
- **Built-in diagnostics** — `./scripts/launch-wow.sh --diagnose` verifies all components before launching
- **Headless test harness** — MRE (Minimal Reproducible Example) validation for regression testing

## How to Use

```bash
# One-time setup (detects runners, stages resources, applies patches)
./wowplay setup --wow-dir ~/Documents/WoW_3.3.5a

# Launch (CrossOver — default)
wowplay run --wow-dir ~/Documents/WoW_3.3.5a

# Launch with Whisky or Moonshine
wowplay run --runner whisky    --wow-dir ~/Documents/WoW_3.3.5a
wowplay run --runner moonshine --wow-dir ~/Documents/WoW_3.3.5a

# Whisky with a custom app location
wowplay run --runner whisky --whisky-bundle ~/Applications/Whisky.app --wow-dir ~/Documents/WoW_3.3.5a

# Diagnose without launching
wowplay diagnose
```

## Contributing / Build from Source

```bash
# Install lefthook, Rust, and Zig first, then:
./scripts/setup.sh
cargo build -p wowplay
```

## Troubleshooting

### "DivxDecoder.dll.InitializeDivxDecoder" error
WoW 3.3.5a is a **32-bit x86** application and requires the `wineloader2` (x86) loader, not `wineloader64` (x86_64). `wowplay setup` creates `wineloader2` automatically when CrossOver is installed. Re-run `wowplay setup --wow-dir <dir>` if this step was skipped.

### Game crashes at character selection or during gameplay
Ensure patching resources are staged: re-run `wowplay setup --wow-dir <dir>`.

### "damaged" / Gatekeeper warnings
Right-click the `wowplay` binary and choose **Open** once to bypass Gatekeeper for ad-hoc signed builds.

### Diagnostics
Run `wowplay diagnose` to verify all components before launching.

## Status

| Phase | Status | Description |
|-------|--------|-------------|
| Foundation & Agent Infrastructure | ✅ Complete | Build system, docs, agent workflows |
| Architecture & Integration Layer | ✅ Complete | Hexagonal design, FFI adapters, test harness |
| Working WoW 3.3.5a POC | ⏳ In Progress | End-to-end launch and gameplay |
| Progressive Enhancement | 🔜 Not Started | Additional clients, performance tuning |

## Contributing

This project uses an agent-first development workflow. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, architecture overview, and how to get involved.

## Acknowledgments

Built upon and inspired by:

- [rosettax87_jit](https://github.com/Lifeisawful/rosettax87_jit) by [@Lifeisawful](https://github.com/Lifeisawful) — x87 to AArch64 JIT translation
- [winerosetta](https://github.com/Gcenx/winerosetta) by [@Gcenx](https://github.com/Gcenx) — Wine/CrossOver integration layer
- [WoWSilicon](https://github.com/WoWSilicon/WoWSilicon) — Swift launcher and patching resources (vendored under `vendor/wowsilicon`)
- [CrossOver](https://www.codeweavers.com/crossover) by CodeWeavers — Windows API translation
- The Wine and CrossOver open-source communities

## License

Apache License 2.0 — See [LICENSE](LICENSE) for details.
