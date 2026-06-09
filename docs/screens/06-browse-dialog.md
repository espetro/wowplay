# Screen: Game Folder Browse Dialog

Native file picker dialog for selecting the WoW installation directory.

## Main Window (Browse Triggered)

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/WoW           │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│                              [loading]  │
│                                         │
└─────────────────────────────────────────┘
```

## macOS File Picker Dialog

```
┌──────────────────────────────────────────────────┐
│                                              🔍  │
│  ┌─────────────┬──────────────────────────────┐  │
│  │ Favorites   │  Users                       │  │
│  │ ▸ mykino    │                              │  │
│  │ ▸ Games     │  📁 Applications             │  │
│  │ ▸ Desktop   │  📁 Desktop                  │  │
│  │             │  📁 Documents                │  │
│  │ Locations   │  📁 Downloads                │  │
│  │ ▸ mykino    │  📁 Games                    │  │
│  │ ▸ Mac HD    │  │  📁 Battle.net            │  │
│  │             │  │  📁 Steam                 │  │
│  │             │  │  📁 World of Warcraft ◀───│  │
│  │             │  │     📁 _retail_           │  │
│  │             │  │     📁 _classic_          │  │
│  │             │  │     📄 WoW.exe            │  │
│  │             │  │     📄 WowClassic.exe     │  │
│  │             │  📁 Movies                   │  │
│  │             │  📁 Music                    │  │
│  └─────────────┴──────────────────────────────┘  │
│                                                  │
│  Selected: /Users/mykino/Games/World of Warcraft │
│                                                  │
│  ┌────────────┐        ┌────────────────────┐    │
│  │  Cancel    │        │   Choose Folder    │    │
│  └────────────┘        └────────────────────┘    │
└──────────────────────────────────────────────────┘
```

## Valid Selection

When a valid WoW folder is selected:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/World of Warc…│ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│   ✓ WoW.exe found                       │
│                                         │
│   ─────────────────────────────────     │
│                                         │
│                    ┌───────────────┐    │
│                    │     Run       │    │
│                    └───────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## Invalid Selection

When selected folder doesn't contain WoW:

```
┌─────────────────────────────────────────┐
│ ○ ○ ○  WoW on Silicon           ...     │
├─────────────────────────────────────────┤
│                                         │
│   GAME FOLDER                           │
│   ┌──────────────────────────┐ ┌─────┐  │
│   │ 📁 ~/Games/OtherGame     │ │Browse│  │
│   └──────────────────────────┘ └─────┘  │
│   ✖ WoW.exe not found in this folder    │
│                                         │
│   ┌─────────────────────────────────┐   │
│   │ ✖  No World of Warcraft         │   │
│   │    installation detected.       │   │
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

## Behavior

- **Browse button**: Opens native OS file picker (folder selection mode)
- **Validation**: Checks for `WoW.exe` or `World of Warcraft.app` in selected folder
- **Auto-detect**: Suggests common install locations (`~/Games`, `~/Applications`, `/Applications`)
- **Path display**: Truncates long paths with ellipsis in middle (`...`)
- **Tooltip**: Hover shows full path
