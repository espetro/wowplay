# Screen Wireframes

ASCII wireframes for the Tauri desktop application.

## Screens

1. [01-first-load.md](01-first-load.md) - Initial app state with Setup button
2. [02-post-setup.md](02-post-setup.md) - After configuration with Run button
3. [03-menu-open.md](03-menu-open.md) - Options menu expanded
4. [04-alert-states.md](04-alert-states.md) - Alert/Callout component variations
5. [05-runner-selector.md](05-runner-selector.md) - Runner dropdown expanded
6. [06-browse-dialog.md](06-browse-dialog.md) - Game folder file picker

## UX Flows

User flows in mermaid format.

### 1. Setup Flow (Happy Path)

```mermaid
flowchart LR
    A[First Load] --> B[Select Runner]
    B --> C[Select Game Folder]
    C --> D{Valid?}
    D -->|Yes| E[Click Setup]
    E --> F[Post-Setup]
    D -->|No| G[Show Error]
    G --> C
```

### 2. Browse & Validate Flow

```mermaid
flowchart TD
    A[Click Browse] --> B[Open File Picker]
    B --> C[Select Folder]
    C --> D{Contains WoW.exe?}
    D -->|Yes| E[Show Checkmark]
    E --> F[Enable Run Button]
    D -->|No| G[Show Error Alert]
    G --> H[Disable Run Button]
    H --> I[Select Different Folder]
    I --> C
```

### 3. Menu Flow

```mermaid
flowchart TD
    A[Any Screen] --> B[Click Menu]
    B --> C[Open Dropdown]
    C --> D{User Action}
    D -->|Select Reset| E[Show Confirmation]
    E --> F[Reset Config]
    F --> G[Return to First Load]
    D -->|Toggle Alerts| H[Toggle Alert Visibility]
    H --> I[Close Menu]
    D -->|Click Outside| I
    D -->|Press Escape| I
```

### 4. Alert Visibility Flow

```mermaid
flowchart TD
    A[Screen State Change] --> B{Check Alerts Toggle}
    B -->|ON| C[Show Info Alerts]
    B -->|OFF| D[Hide Info Alerts]
    C --> E[Show Warnings]
    D --> E
    E --> F[Show Errors]
    F --> G[Update UI]
```

### 5. Run Flow

```mermaid
flowchart TD
    A[Post-Setup] --> B[Click Run]
    B --> C{Validation Pass?}
    C -->|Yes| D[Launch WoW]
    D --> E[Game Running]
    C -->|No| F[Show Error Alert]
    F --> G[Stay on Post-Setup]
    G --> H[Fix Issues]
    H --> B
```

## Design Notes

- **Window**: Fixed-size modal-style window with macOS-style traffic lights
- **Menu**: "..." button at top-right reveals dropdown menu
- **Alerts**: Disabled by default; toggle via menu `Show alerts`
- **Errors/Warnings**: Always displayed regardless of alert toggle
- **No Cancel button**: Primary action only (Setup/Run)
- **Responsive**: All components stack vertically with consistent spacing
