# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-06-10

### Changed
- Extract `setup`, `diagnostics`, and `resources` into separate focused modules

## [0.3.1] - 2026-06-09

### Changed
- Version bump to align with release pipeline update

## [0.3.0] - 2026-06-09

### Added
- `WhiskyAdapter`: runner backend for Whisky and Moonshine (resolves bottle path, builds Wine env)
- `RunnerRegistry`: discovers and lists available runner backends at runtime

### Fixed
- `WhiskyAdapter` bottle path resolution for both Whisky and Moonshine app layouts
- Support for user-local `~/Applications/Whisky.app` installation

### Changed
- `WowLauncher::launch_wow_logged`: removed verbose `[info]` lines printed during launch
- Log tee (`tee_to_log`) no longer mirrors output to terminal — writes only to log file

## [0.2.0] - 2026-06-09

### Added
- Initial release with synchronized versioning workflow
- `CrossOverAdapter`: finds CrossOver.app, stages wineloader2, builds Wine env
- `WowLauncher`: orchestrates full rosettax87 + CrossOver launch sequence
- `WowSession`: wraps running WoW child process with `pid()` and `wait()`
