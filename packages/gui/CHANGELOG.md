# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed
- `e2e/` TOML-only E2E tests (Gauge is now the sole E2E format)
- `test:e2e` script (superseded by `test:gauge`)

## [0.4.0] - 2026-06-10

### Added
- Initial GUI app shipping as a signed, notarized `WoW on Silicon.app`
- `BottleInput` component wired to the bottle parameter passed to the sidecar
- Persistent `AppState` hydration from the Tauri store on startup
- Gauge E2E tests with `data-testid` attributes

### Changed
- Architecture switches from direct library calls to a sidecar launcher (wowplay binary embedded in the bundle)
