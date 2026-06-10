# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-06-10

### Added
- `--disable-lib-silicon` flag to make `libSiliconPatch.dll` optional at launch

### Fixed
- CrossOver runner option label no longer prefixed with "Wine"

### Changed
- Use extracted `rust-core` modules (setup, diagnostics, resources) instead of internal functions

## [0.3.1] - 2026-06-09

### Changed
- Release binary is now signed and notarized — no `xattr` workaround needed

## [0.3.0] - 2026-06-09

### Added
- `--runner` flag: choose between `crossover`, `whisky`, and `moonshine`
- `--whisky-bundle` flag: override the path to Whisky.app
- Runner table shown during `wowplay setup` listing all detected runners
- 4-path patching resolution in `wowplay setup` (staged resources, symlinks, in-place, bundled)

### Changed
- Log path is now printed before the process starts (was printed after exit)
- Removed "WoW started (pid …)" message; process output no longer mirrored to terminal when logging

## [0.2.0] - 2026-06-09

### Added
- Initial release with synchronized versioning workflow

### Changed
- `--no-sudo` flag replaced by `--sudo`; sudo is no longer required by default
