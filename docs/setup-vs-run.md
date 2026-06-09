# CLI Commands: Setup vs Run

This document explains the relationship between `wowplay setup` and `wowplay run`, when each is needed, and why `setup` is not strictly required.

## `wowplay run` is self-sufficient

`run` automatically handles everything needed to launch the game on first use:

- **Applies the game patch** — copies D9VK, winerosetta, rosettax87 binaries into your WoW directory
- **Prepares the Wine loader** — creates `wineloader2` (CrossOver) or locates `wine64` (Whisky/Moonshine)
- **Bootstraps DivxDecoder.dll** — patches it once for winerosetta injection
- **Starts the rosettax87 service** if not already running

You can run `wowplay run` immediately after download without ever running `setup`.

## `wowplay setup` stages and verifies

`setup` does what `run` does, plus two extras:

1. **Permanently stages patching resources** to `~/.local/share/wowplay/patching`

   This matters when you move the `wowplay` binary away from the release zip (e.g., into `~/bin` or `/usr/local/bin`). After `setup`, `run` can find resources regardless of where the binary lives.

2. **Prints a diagnostics table** showing which runners are detected

## When to re-run `setup`

| Scenario | Re-run `setup`? | Why |
|----------|----------------|-----|
| First download / fresh install | **Yes** (recommended) | Stage resources permanently and verify runners |
| Switching runners (e.g., CrossOver → Whisky) | No | Game patch is runner-agnostic |
| Changing `--wow-dir` | No | `run` patches the new directory automatically |
| Updating the `wowplay` binary | No* | *Unless you moved it away from the `patching/` folder — then yes, to re-stage |
| Game crashes with "patching resources not found" | Yes | Resources were not staged; `setup` fixes this |

## Best practice

Run `setup` once after downloading to stage resources and confirm your environment. After that, use `run` exclusively. If you move the binary, run `setup` again.
