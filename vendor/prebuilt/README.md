# vendor/prebuilt — Vendored prebuilt binaries

Binaries in this directory are committed to the repository. Each has documented
provenance and a pinned SHA-256 in `CHECKSUMS.sha256`, verified at build time by
`scripts/stage-patching.sh`.

## Contents

| File | Source | License | Notes |
|---|---|---|---|
| `d9vk/d3d9.dll` | [Gcenx/DXVK-macOS](https://github.com/Gcenx/DXVK-macOS) (via WoWSilicon bundle) | zlib/libpng | DirectX 9 → Vulkan/MoltenVK/Metal |
| `libSiliconPatch/wotlk/libSiliconPatch.dll` | WoWSilicon.app bundle (closed-source) | proprietary | x87 patch library — **opt-in** via `--enable-lib-silicon`; default-disabled |

## Dropped artifacts (no longer shipped)

| Artifact | Reason |
|---|---|
| `winerosetta/libDllLdr.dll` | Superseded by the native Rust PE patcher (HEAD `19e1eaa`) |
| `winerosetta/ntdll.so` | No loader references found in any source tree; never copied to the WoW directory by `apply_game_patch`; dropped as dead weight |
| `libSiliconPatch/vanilla/libSiliconPatch.dll` | Never referenced by `apply_game_patch` |
| `vanilla-tweaks/vanilla-tweaks.exe` | Unused MIT binary |

## Source-built artifacts (not vendored here)

| Artifact | Built from |
|---|---|
| `winerosetta/winerosetta.dll` | `packages/zig-glue` (Zig cross-compile to Windows x86) |
| `rosettax87/rosettax87` | `vendor/rosettax87_jit` (CMake → `build/bin/runtime_loader`) |
| `rosettax87/libRuntimeRosettax87` | `vendor/rosettax87_jit` (CMake → `build/bin/libRuntimeRosettax87`) |
