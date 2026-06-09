# Screen: Menu Open

Options menu expanded from the "..." button.

## Menu Closed (Default)

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
```

## Menu Open

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                               ┌─────────┐│
│                               │ Reset   ││
│                               ├─────────┤│
│                               │Show     ││
│                               │alerts ✓ ││
│                               └─────────┘│
│                                         │
│   WoW on Silicon                        │
│   ...                                   │
│                                         │
└─────────────────────────────────────────┘
```

## Menu Items

| Item | Icon/State | Action |
|------|-----------|--------|
| Reset | None | Resets all configuration, returns to first-load state |
| Show alerts | Checkbox (✓/☐) | Toggles visibility of info/warning alert messages |

## Behavior

- **Reset**: Clears saved configuration, shows confirmation dialog
- **Show alerts**: 
  - Checked (✓): Info alerts are visible in the UI
  - Unchecked (☐): Only error/warning alerts are shown
  - Default: Unchecked
- **Menu dismissal**: Click outside menu or select an item to close
- **Keyboard**: `Escape` closes menu without action
