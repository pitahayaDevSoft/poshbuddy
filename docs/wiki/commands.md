# PoshBuddy Wiki: Interface Shortcuts

> **Updated**: 2026-04-29
> **Version**: v0.4.5
> **Read Time**: 3 min  

PoshBuddy utilizes a keyboard-centric interface but also supports a powerful headless CLI mode for high-velocity environment management.

## CLI Command Reference

Execute commands directly from your terminal for atomic operations without launching the TUI.

| Command | Arguments | Description |
| :--- | :--- | :--- |
| `poshbuddy set theme` | `<name>` | Applies a theme (local or remote) to all profiles. |
| `poshbuddy install font` | `<name>` | Downloads and installs a specific Nerd Font. |
| `poshbuddy list themes` | `[--local / --remote]` | Displays a table of all discoverable themes. |
| `poshbuddy list fonts` | | Lists all available fonts in the official collection. |

### CLI Examples

```powershell
# Set theme directly
poshbuddy set theme bubbles

# List only local themes
poshbuddy list themes --local

# Atomic font installation
poshbuddy install font FiraCode
```

## Global Navigation (TUI)

| Key | Action |
| :--- | :--- |
| `[TAB]` | Cycle focus between the List (left) and Panel (right/bottom) |
| `[1]` | Jump to **Themes Explorer** |
| `[2]` | Jump to **Fonts Manager** |
| `[3]` | Jump to **Segments Explorer** |
| `[B]` | Jump to **Backups Manager** |
| `[D]` | Jump to **Diagnostics** |
| `[H]` | Home (Dashboard) |
| `[ESC]` | Cancel / Go Back / Reset State |
| `[Q]` | Exit the application |

## View-Specific Controls

### Themes Explorer [1]

- `[UP / DOWN]`: Navigate the theme list. This initiates an asynchronous preview.
- `[ANY CHAR]`: Start typing to filter themes by name.
- `[BACKSPACE]`: Remove characters from the search filter.
- `[ENTER]`: Apply the selected theme to all detected PowerShell profiles.

### Fonts Manager [2]

- `[UP / DOWN]`: Navigate the list of recommended Nerd Fonts.
- `[ENTER]`: Initiate the downloader and installer for the selected font.
- `[ANY CHAR]`: Filter fonts by name.

## State Transitions

1. **Startup**: `Welcome` -> `Loading` (Fetching GitHub themes) -> `Main`.
2. **Action**: `Main` -> `Robust Transaction` (Atomic Pipeline) -> `Success`.
3. **Recovery**: `Error` state supports `[ESC]` to return to the last valid screen without exiting.
4. **Backup**: `[B]` -> `Backups` screen -> `[ENTER]` to restore a profile snapshot instantly.

---
**Return to**: [Wiki Dashboard](./index.md)

---
**Return to**: [Wiki Dashboard](./index.md)
