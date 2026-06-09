# Changelog

All notable changes to the play-wow-on-silicon project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

For per-package changes, see the individual CHANGELOGs:
- [`packages/rust-core/CHANGELOG.md`](packages/rust-core/CHANGELOG.md)
- [`packages/cli/CHANGELOG.md`](packages/cli/CHANGELOG.md)
- [`packages/integration/CHANGELOG.md`](packages/integration/CHANGELOG.md)
- [`tools/profiler/CHANGELOG.md`](tools/profiler/CHANGELOG.md)

## [0.2.0] - 2026-06-09

### Added
- Workspace-wide synchronized versioning using `release-plz`
- Automated release workflow via GitHub Actions
- Per-package CHANGELOGs for all components
- `CrossOverAdapter` and `WowLauncher` core modules for CrossOver integration
- Apache-2.0 license

### Changed
- `wowplay run --no-sudo` replaced by `--sudo` (sudo is no longer required by default)

