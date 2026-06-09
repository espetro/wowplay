# Screen: First Load (Variation 1)

Initial application state when no configuration exists.

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   WoW on Silicon                        │
│   Configure your Apple Silicon WoW      │
│   runner and launch settings.           │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ Select runner...          [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 /path/to/WoW          │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │    Setup      │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## Elements

| Element | State | Description |
|---------|-------|-------------|
| Window Title | Active | "WoW on Silicon" with traffic lights |
| Menu Button | Default | "..." at top-right, opens options menu |
| Title | Static | "WoW on Silicon" heading |
| Subtitle | Static | Description text |
| Runner | Empty | Dropdown with placeholder "Select runner..." |
| Game Folder | Empty | Path input with Browse button |
| Setup Button | Primary | Blue/primary action button, starts configuration |

## Behavior

- **Setup button**: Disabled until both Runner and Game Folder are selected
- **Browse button**: Opens native file picker for WoW installation directory
- **Menu**: Access to Reset and Show alerts options
