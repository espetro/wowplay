# Screen: Runner Selector Expanded

The Runner dropdown in its expanded state showing available options.

## Dropdown Closed

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ Select runner...          [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
└─────────────────────────────────────────┘
```

## Dropdown Open

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ Wine (CrossOver)          [▲] │     │
│   └───────────────────────────────┘     │
│   ┌───────────────────────────────┐     │
│   │ ☐ Wine (CrossOver)            │     │
│   │ ☐ Whisky                      │     │
│   │ ☐ Moonshine                   │     │
│   │ ─────────────────────────     │     │
│   │ ☐ Custom...                   │     │
│   └───────────────────────────────┘     │
│                                         │
└─────────────────────────────────────────┘
```

## Runner Options

| Option | Description |
|--------|-------------|
| Wine (CrossOver) | CrossOver commercial Wine wrapper |
| Whisky | Native macOS Wine wrapper app; legacy |
| Moonshine | Modern (2026) Whisky fork |
| Custom... | User-defined Wine/VM path |

## Behavior

- **Selection**: Single-select, closes dropdown on selection
- **Custom**: Opens file picker to select custom Wine binary
- **Keyboard navigation**: Arrow keys + Enter to select
- **Dismiss**: Click outside or press Escape to close without selection
