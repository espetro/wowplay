# Screen: Post-Setup (Variation 2)

Application state after successful configuration.

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
│   │ CrossOver                 [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/World of Warc…│ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │     Run       │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## Elements


| Element      | State    | Description                                    |
| ------------ | -------- | ---------------------------------------------- |
| Window Title | Active   | "WoW on Silicon" with traffic lights           |
| Menu Button  | Default  | "..." at top-right, opens options menu         |
| Title        | Static   | "WoW on Silicon" heading                       |
| Subtitle     | Static   | Description text                               |
| Runner       | Selected | Shows active runner (e.g., "CrossOver")        |
| Game Folder  | Selected | Shows path to WoW installation                 |
| Run Button   | Primary  | Blue/primary action button, launches the game  |


## Behavior

- **Run button**: Launches WoW with the configured runner
- **Runner dropdown**: Can be changed to switch runners
- **Game Folder**: Can be updated via Browse button
- **Menu**: Reset returns to first-load state; Show alerts toggles info messages

