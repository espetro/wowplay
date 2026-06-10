# Screen: Alert/Callout States

The alert component displays messages to the user. Positioned between the form fields and action button.

## State: Hidden (Default)

When `Show alerts` is disabled and no errors exist:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   WoW on Silicon                        │
│   Configure your Apple Silicon WoW      │
│   runner and launch settings.           │
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ CrossOver                 [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/WoW           │ │Browse│  │
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

## State: Info Alert (Show alerts enabled)

When `Show alerts` is enabled:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   WoW on Silicon                        │
│   Configure your Apple Silicon WoW      │
│   runner and launch settings.           │
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ CrossOver                 [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/WoW           │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                                         │
│   ┌─────────────────────────────────┐   │
│   │ ⓘ  Using Wine with CrossOver.   │   │
│   │    Rosetta 2 is required for    │   │
│   │    x86 emulation.               │   │
│   └─────────────────────────────────┘   │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │     Run       │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## State: Warning (Always visible)

Warnings are always shown regardless of `Show alerts` setting:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   WoW on Silicon                        │
│   Configure your Apple Silicon WoW      │
│   runner and launch settings.           │
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ CrossOver                 [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/WoW           │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                                         │
│   ┌─────────────────────────────────┐   │
│   │ ⚠  Wine prefix not found.       │   │
│   │    A new prefix will be created │   │
│   │    on first launch.             │   │
│   └─────────────────────────────────┘   │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │     Run       │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## State: Error (Always visible)

Errors are always shown and may disable the action button:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   WoW on Silicon                        │
│   Configure your Apple Silicon WoW      │
│   runner and launch settings.           │
│                                         │
│   RUNNER                                │
│   ┌───────────────────────────────┐     │
│   │ CrossOver                 [▼] │     │
│   └───────────────────────────────┘     │
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/WoW           │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                                         │
│   ┌─────────────────────────────────┐   │
│   │ ✖  Game executable not found.   │   │
│   │    Please verify the game path. │   │
│   └─────────────────────────────────┘   │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │     Run       │    │
│                    │  (disabled)   │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## Alert Types

Use `lucide-icons`.


| Type    | Icon | Color        | Visibility                                |
| ------- | ---- | ------------ | ----------------------------------------- |
| Info    | ⓘ    | Blue/Gray    | Only when `Show alerts` is enabled        |
| Warning | ⚠    | Yellow/Amber | Always visible                            |
| Error   | ✖    | Red          | Always visible, may disable action button |


## Behavior

- **Dismissal**: Info alerts can be dismissed with an X button (optional)
- **Auto-hide**: Info alerts may auto-hide after 5 seconds (optional)
- **Multiple alerts**: Stack vertically with 8px spacing between them
- **Priority**: Error &gt; Warning &gt; Info (sorted by severity)

