# Changelog

All notable changes to the play-wow-on-silicon project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

For per-package changes, see the individual CHANGELOGs:
- [`packages/gui/CHANGELOG.md`](packages/gui/CHANGELOG.md)
- [`packages/rust-core/CHANGELOG.md`](packages/rust-core/CHANGELOG.md)
- [`packages/cli/CHANGELOG.md`](packages/cli/CHANGELOG.md)
- [`packages/integration/CHANGELOG.md`](packages/integration/CHANGELOG.md)
- [`tools/profiler/CHANGELOG.md`](tools/profiler/CHANGELOG.md)

## [0.4.0] - 2026-06-10

### Added
- GUI app (Tauri-based) with sidecar launcher architecture — `WoW on Silicon.app` ships alongside the CLI
- `BottleInput` component wired to the bottle parameter in the GUI
- Persistent `AppState` hydration from the Tauri store on GUI startup
- Gauge E2E tests for the GUI with `data-testid` attributes
- `--disable-lib-silicon` flag for `wowplay` CLI to make `libSiliconPatch.dll` optional

### Fixed
- CrossOver runner option no longer incorrectly prefixed with "Wine"

### Changed
- GUI switches from direct library calls to a sidecar launcher process
- `rust-core` setup, diagnostics, and resources split into focused modules; CLI wired to them directly
- Release workflow updated to use `tauri-apps/tauri-action` for signed, notarized GUI builds

## [0.3.1] - 2026-06-09

### Changed
- Release binary is now signed with a Developer ID certificate and notarized by Apple — no more Gatekeeper quarantine or `xattr` workarounds on other Macs

## [0.3.0] - 2026-06-09

### Added
- Whisky runner adapter: run WoW via Whisky or Moonshine (free alternatives to CrossOver)
- `--runner` flag to select the runner backend (`crossover`, `whisky`, `moonshine`)
- `--whisky-bundle` flag to point at a custom Whisky.app path
- Runner table shown during `wowplay setup` listing all detected runners
- 4-path patching resolution and resource staging in setup
- Build-release CI workflow triggered on `v*` tags
- Lefthook pre-commit hook for Rust typecheck and Zig fmt

### Fixed
- Whisky bottle path resolution for both Whisky and Moonshine app layouts
- Support for user-local `~/Applications/Whisky.app` installation

### Changed
- Launch output: log path printed before process start; verbose `[info]` lines removed
- Log tee writes only to file — no longer mirrors WoW stdout/stderr to terminal

## [0.2.0] - 2026-06-09

### Added
- Workspace-wide synchronized versioning using `release-plz`
- Automated release workflow via GitHub Actions
- Per-package CHANGELOGs for all components
- `CrossOverAdapter` and `WowLauncher` core modules for CrossOver integration
- Apache-2.0 license

### Changed
- `wowplay run --no-sudo` replaced by `--sudo` (sudo is no longer required by default)

