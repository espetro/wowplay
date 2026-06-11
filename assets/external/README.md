# External Runtime Dependencies

This directory tracks the provenance of every binary that ships in the release bundle.
**No binary files are committed here** — see `vendor/prebuilt/` for vendored binaries.

Staging is performed by `scripts/stage-patching.sh`, called by `scripts/release.sh`
and `.github/workflows/build-release.yml`.  The script verifies checksums before
staging and omits every artifact that is not in the table below.

## Shipped artifacts

| Artifact (in `dist/patching/`) | Provenance | License | Notes |
|---|---|---|---|
| `d9vk/d3d9.dll` | `vendor/prebuilt/d9vk/d3d9.dll` — pinned binary from [Gcenx/DXVK-macOS](https://github.com/Gcenx/DXVK-macOS) | zlib/libpng | DirectX 9 → Vulkan → MoltenVK → Metal; SHA-256 in `vendor/prebuilt/CHECKSUMS.sha256` |
| `winerosetta/winerosetta.dll` | Source-built: `packages/zig-glue` (Zig cross-compile to Windows x86) | MIT — [Gcenx/winerosetta](https://github.com/Gcenx/winerosetta) | |
| `rosettax87/rosettax87` | Source-built: `vendor/rosettax87_jit` CMake → `build/bin/runtime_loader` | MIT — [Lifeisawful/rosettax87_jit](https://github.com/Lifeisawful/rosettax87_jit) | Staged as `rosettax87` (the name `apply_game_patch` expects) |
| `rosettax87/libRuntimeRosettax87` | Source-built alongside `rosettax87` above | Same | |
| `libSiliconPatch/wotlk/libSiliconPatch.dll` | `vendor/prebuilt/libSiliconPatch/wotlk/libSiliconPatch.dll` — from WoWSilicon.app bundle | proprietary (closed-source) | **Opt-in** via `--enable-lib-silicon`; default-disabled; SHA-256 in `vendor/prebuilt/CHECKSUMS.sha256` |

## Dropped artifacts (no longer shipped)

| Artifact | Reason |
|---|---|
| `winerosetta/libDllLdr.dll` | Superseded by native Rust PE patcher (commit `19e1eaa`) |
| `winerosetta/ntdll.so` | No references in any loader source (rosettax87_jit, zig-glue, rust-core); never copied to the WoW directory by `apply_game_patch`; dropped as dead weight |
| `libSiliconPatch/vanilla/libSiliconPatch.dll` | Not referenced by `apply_game_patch` |
| `vanilla-tweaks/vanilla-tweaks.exe` | Unused MIT binary |

## Retired submodules

`vendor/wowsilicon` and `vendor/rosettax87` have been removed.  The build no longer
reads from either.  The runtime `find_wowsilicon()` fallback in `crossover.rs` is
retained as a convenience for users who have WoWSilicon.app installed locally, but
it is not required to launch WoW.
