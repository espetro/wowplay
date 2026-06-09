# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-09

### Added
- Initial release with synchronized versioning workflow
- `CrossOverAdapter`: finds CrossOver.app, stages wineloader2, builds Wine env
- `WowLauncher`: orchestrates full rosettax87 + CrossOver launch sequence
- `WowSession`: wraps running WoW child process with `pid()` and `wait()`
