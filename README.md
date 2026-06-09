# play-wow-on-silicon

> **Run World of Warcraft 3.3.5a on Apple Silicon.**
>
> Built around instruction-set bridging — x87 FPU emulation, Wine/CrossOver integration, and runtime patching — so 32-bit x86 games from the 2006–2010 era run efficiently on modern macOS. The architecture is designed to extend to other legacy apps that struggle with missing instruction sets.

## Download

[![Latest Release](https://img.shields.io/github/v/release/espetro/play-wow-on-silicon?style=flat&label=latest)](https://github.com/yourusername/play-wow-on-silicon/releases/latest)

**macOS only** · Requires [CrossOver](https://www.codeweavers.com/crossover) 23+ (external runtime, not included)

1. Download the latest release for your Mac above.
2. Install CrossOver and run it at least once.
3. Have a WoW 3.3.5a client folder ready.
4. Run `./scripts/setup.sh` to build the native components.

See [CONTRIBUTING.md](CONTRIBUTING.md) if you want to build from source or hack on the project.

## Features

- **One-command launch** — `wowplay run` handles CrossOver patching, x87 emulation, and Wine loader setup automatically
- **x87 FPU emulation** — bridges missing x87 instructions to AArch64 via RosettaX87 so the 32-bit client executes correctly
- **Automated patching** — applies game-folder and CrossOver patches needed for the RosettaX87 launch path
- **Built-in diagnostics** — `./scripts/launch-wow.sh --diagnose` verifies all components before launching
- **Headless test harness** — MRE (Minimal Reproducible Example) validation for regression testing

## How to Use

```bash
# 1. One-time setup (builds native components and vendor libraries)
./scripts/setup.sh

# 2. Point the tool at your WoW folder and apply patches
wowplay setup --wow-dir ~/Documents/WoW_3.3.5a \
  --patching-dir vendor/wowsilicon/Sources/WoWSiliconSwift/Resources/Patching

# 3. Launch (CrossOver — default)
wowplay run --wow-dir ~/Documents/WoW_3.3.5a

# Launch with Whisky instead of CrossOver
wowplay run --runner whisky --wow-dir ~/Documents/WoW_3.3.5a

# Whisky with custom app location
wowplay run --runner whisky --whisky-bundle ~/Applications/Whisky.app --wow-dir ~/Documents/WoW_3.3.5a

# Diagnose without launching
wowplay diagnose
```

## Troubleshooting

### "DivxDecoder.dll.InitializeDivxDecoder" error
WoW 3.3.5a is a **32-bit x86** application and requires the `wineloader2` (x86) loader, not `wineloader64` (x86_64). The launch script automatically creates `wineloader2` by copying and re-signing the original `wineloader`. If this step was skipped, re-run `./scripts/setup.sh`.

### Game crashes at character selection or during gameplay
Ensure the x87 runtime components were built successfully:
```bash
./scripts/setup.sh
```
This rebuilds `rosettax87_jit/runtime_loader` and `winerosetta.dll` if needed.

### "damaged" app warnings
The script removes macOS quarantine flags from binaries, but you may still need to allow them in **System Settings > Privacy & Security**.

### Diagnostics
Run `./scripts/launch-wow.sh --diagnose` to verify all components before launching.

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
