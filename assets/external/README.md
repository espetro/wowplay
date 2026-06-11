# External Runtime Dependencies

This directory tracks files that are copied from WoWSilicon.app or TurtleSilicon.app
at launch time. **Binary files are not committed here** — they are sourced at runtime
from an installed app bundle.

## Inventory

| File | Source app | Open-source? | Roadmap |
|---|---|---|---|
| `winerosetta.dll` | Built by zig-glue from vendored C++ | Yes — [Gcenx/winerosetta](https://github.com/Gcenx/winerosetta) (MIT) | Already owned |
| `mods/libSiliconPatch.dll` (WotLK) | WoWSilicon.app bundle | No | **P0** — recreate in Rust (SiliconPatchAdapter) |
| `d3d9.dll` | WoWSilicon.app bundle (D9VK) | Yes — [doitsujin/dxvk](https://github.com/doitsujin/dxvk) | P1 — vendor D9VK build |
| `libDllLdr.dll` | ~~WoWSilicon.app bundle~~ | ~~Unknown~~ | ✅ Replaced by native Rust PE patcher |
| `rosettax87/runtime_loader` | Built from [WineAndAqua/rosettax87_jit](https://github.com/WineAndAqua/rosettax87_jit) via `vendor/rosettax87` | Yes — open source | ✅ Source-built; set `ROSETTAX87_BIN_DIR` to build output dir |
| `rosettax87/libRuntimeRosettax87` | Same source | Yes — same repo | ✅ Source-built alongside `runtime_loader` |

## Notes

- Files arrive here via `wowplay setup` (which calls `apply_game_patch` in rust-core).
- The P0 replacement for `libSiliconPatch.dll` is tracked in `docs/porting-policy.md`.
- Once a file is vendored or built locally, remove its row from this table and add it
  to the relevant package's build output.
