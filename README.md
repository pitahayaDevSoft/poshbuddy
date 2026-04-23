<!-- trunk-ignore-all(markdownlint/MD033) -->
<!-- trunk-ignore-all(markdownlint/MD041) -->
<div align="center">
  <img src="docs/logo.png" alt="PoshBuddy Logo" width="180" onerror="this.src='https://placehold.co/200x200/222222/00d2ff?text=PoshBuddy'"/>
</div>

# PoshBuddy

![Build](https://github.com/julesklord/poshbuddy/actions/workflows/rust.yml/badge.svg) ![Version](https://img.shields.io/badge/version-0.4.1-blue) ![License](https://img.shields.io/badge/license-MIT-green)

PoshBuddy is a professional management tool for Oh My Posh configurations. It provides a modernized, responsive terminal user interface (TUI) designed for high-density information display, surgical theme manipulation, and seamless Nerd Font management.

<p align="center">
  <img src="assets/demo.gif" alt="PoshBuddy TUI Demo" width="90%">
</p>

Developed in Rust, PoshBuddy prioritizes safety and performance, ensuring that your PowerShell environment remains stable and your configuration files remain auditable at all times.

## Core Pillars

### Unified Theme Management

PoshBuddy bridges the gap between your local setup and the extensive Oh My Posh ecosystem. Our ThemeAsset engine unifies local files and remote GitHub repositories, allowing you to discover, preview, and install the entire official theme collection with a single action.

### Surgical Segment Manipulation

Unlike traditional theme managers that overwrite your entire configuration, PoshBuddy performs precise edits. You can toggle specific segments—such as Git status, battery indicators, or execution time—directly into your active theme without disturbing your custom styles or layout.

### Non-Destructive Profile Injection

Environment stability is critical. PoshBuddy manages your PowerShell profile using an injection system based on secure markers. This ensures that any change made by the tool is fully reversible and localized, preventing corruption of your existing scripts.

## Why PoshBuddy?

Customizing a PowerShell prompt should not be a repetitive or manual task. PoshBuddy eliminates the friction of editing profiles and configuration files, bringing a modern interface to your developer workflow.

- **Unstuck Guarantee**: Integrated network timeouts and OMP binary guard to prevent TUI hangs in any condition.
- **Zero-Config Profile Sync**: Automatically detects and updates both PowerShell 5.1 and 7 profiles.
- **Accurate Previews**: Environment isolation and corrected extension handling ensure that theme previews are unaffected by your current shell state.
- **Diagnostic Intelligence**: Automated checks for Nerd Fonts, shell versions, and terminal compatibility (Windows Terminal recommended).

## Technical Architecture

PoshBuddy is built for performance and reliability using modern systems programming patterns:

- **Rust**: Core language for memory safety and execution speed.
- **Tokio**: Asynchronous runtime for non-blocking network operations, installers, and file I/O.
- **Ratatui**: State-of-the-art framework for the TUI rendering loop.
- **Serde**: High-performance serialization for JSON configuration manipulation.

## Installation

Ensure you have the [Rust toolchain](https://rustup.rs/) installed.

```powershell
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy
cargo install --path .
```

_Note: This tool requires the Oh My Posh binary to be present in your system PATH._

## CLI Mode (Headless)

PoshBuddy includes a robust command-line interface for quick actions without launching the full TUI.

### Theme Commands

- **Set Theme**: `poshbuddy set theme <name>`
  - Automatically searches local themes and the official remote catalogue.
- **List Themes**: `poshbuddy list themes [--local] [--remote]`
  - Displays a formatted table of all available themes.

### Font Commands

- **Install Font**: `poshbuddy install font <name>`
  - Installs the specified Nerd Font directly to your system.
- **List Fonts**: `poshbuddy list fonts`
  - Lists all available Nerd Fonts in the official collection.

### Usage Examples

```powershell
# Set a specific theme by name
poshbuddy set theme bubbles

# List all local themes only
poshbuddy list themes --local

# Install FiraCode Nerd Font
poshbuddy install font FiraCode
```

## Controls and Navigation

| Key         | Action                                                         |
| :---------- | :------------------------------------------------------------- |
| **1**       | **Themes Explorer** — Browse local and official remote themes. |
| **2**       | **Font Manager** — Install and manage Nerd Fonts.              |
| **3**       | **Segment Manager** — Toggle theme components in real-time.    |
| **Esc / H** | **Dashboard** — Return to the main welcome screen.             |
| **Enter**   | Apply theme, toggle segment, or start installation.            |
| **Q**       | Exit the application.                                          |

### Dashboard Quick Actions

| Key   | Action                                |
| :---- | :------------------------------------ |
| **R** | Apply a **Random Theme** immediately. |
| **N** | Install **All Nerd Fonts**.           |
| **I** | Toggle **Terminal Icons**.            |
| **D** | Run System **Diagnostics**.           |
| **V** | View and restore **Backups**.         |
| **B** | Create **Manual Backup** of profiles. |

## Community and Support

Contributions to PoshBuddy are welcome. For technical details, troubleshooting, or feature requests, please refer to the following resources:

- [Wiki](docs/wiki/index.md) - Technical deep dive and architectural overview.
- [GitHub repository](https://github.com/julesklord/poshbuddy) - Source code and issue tracker.
- [Changelog](CHANGELOG.md) - Detailed version history.

---

Developed for terminal perfectionists.
**Your prompt. Your rules. Zero friction.**

[GitHub](https://github.com/julesklord/poshbuddy) · [Documentation](./docs) · [Changelog](./CHANGELOG.md)
