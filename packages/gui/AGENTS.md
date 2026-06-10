# WoW on Silicon — GUI Agent Guide

## Overview
This package contains the Tauri v2 desktop application for WoW on Silicon.

## Tech Stack
- **Backend**: Rust (Tauri v2)
- **Frontend**: SolidJS + TypeScript
- **Styling**: Tailwind CSS v4
- **UI Components**: Ark UI (SolidJS)
- **State**: SolidJS stores (fine-grained reactivity)
- **Error Handling**: neverthrow (ResultAsync pattern matching)

## Commands

### Development
```bash
# Start Tauri dev server (hot reload for both Rust and TS)
bun run tauri:dev

# Build frontend only
bun run build

# Build Tauri app for production
bun run tauri:build
```

### Testing
```bash
# Unit tests (Vitest)
bun run test

# E2E tests (tauri-pilot)
bun run test:e2e

# E2E tests (Gauge)
cd gauge-tests && gauge run specs/
```

### Code Quality
```bash
# Lint
bun run lint

# Format
bun run fmt

# Validate build + tests
bun run validate
```

## Architecture

### Backend Commands (src/commands/)
- `config.rs` — Get/set/reset app configuration with store persistence
- `setup.rs` — Run setup sequence and validate WoW directories
- `launch.rs` — Launch WoW with selected runner
- `diagnostics.rs` — Check runner availability

### Frontend Structure (src-frontend/)
- `App.tsx` — Main app component with setup/run flows
- `stores/app.ts` — Reactive store with config, runners, alerts
- `lib/tauri.ts` — Typed IPC wrapper with neverthrow ResultAsync
- `components/` — UI components (RunnerSelect, GameFolderPicker, etc.)

### Key Patterns
- **Result-based error handling**: All IPC calls return `ResultAsync<T, TauriError>`
- **No try/catch**: Use `.match()` or `.map()` for error handling
- **Fine-grained reactivity**: SolidJS stores with derived getters
- **Neverthrow**: JS/TS mirrors Rust's Result pattern

## E2E Testing

### tauri-pilot (TOML-based declarative)
Located in `e2e/`. Run all flows with:
```bash
bun run test:e2e
# equivalent to: tauri-pilot run e2e/
```
Five flows in `e2e/`:
| File | UX flow |
|------|---------|
| `flow.toml` | Setup: runner → folder → Setup btn → "Setup complete" → Run btn |
| `validation.toml` | Browse: valid path enables btn; invalid path shows error alert |
| `menu-flow.toml` | Menu: open, toggle alerts, reset |
| `alerts.toml` | Alerts: error visible by default, info hidden |
| `run-flow.toml` | Run: post-setup Run btn → PID alert |

All selectors use `data-testid` attributes (see Step 8).

### Gauge (BDD-style)
Located in `gauge-tests/`. Uses Gauge + TypeScript + `TauriPilotFlow` builder.
```bash
bun run test:gauge
# equivalent to: cd gauge-tests && gauge run specs/
```
- Specs: `gauge-tests/specs/`
- Steps: `gauge-tests/tests/StepImplementation.ts`
- Shared builder: `gauge-tests/support/tauri-pilot.ts` — assembles TOML at runtime and executes via `tauri-pilot run`

**Architecture**: each `@Step` creates a `TauriPilotFlow`, chains fluent calls (`.click()`, `.ipc()`, `.assert()`, `.wait()`), then calls `.run()` which writes a temp TOML to `/tmp/`, invokes `tauri-pilot run <file>`, and cleans up.

**Setup requirement**: run `bun install` from the repo root before running gauge tests — this links the `@wow-silicon/tsconfig` workspace package.

**No Playwright**: `@srsholmes/tauri-playwright` and `playwright` have been removed; tauri-pilot is the sole execution engine.

## Window Config
- Size: 480x580 (fixed, non-resizable)
- Centered on screen
- macOS-style decorations

## Notes for Agents
- The app uses `tokio::task::spawn_blocking` for sync core operations
- Store plugin persists config to `config.json`
- Dialog plugin requires `dialog:default` capability
- macOS entitlements in `Entitlements.plist`
