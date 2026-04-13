<div align="center">
  <img src="docs/logo.png" alt="PoshBuddy Logo" width="180" onerror="this.src='https://placehold.co/200x200/222222/00d2ff?text=PoshBuddy'"/>
</div>

# PoshBuddy

![Build](https://github.com/julesklord/poshbuddy/actions/workflows/rust.yml/badge.svg) ![Version](https://img.shields.io/badge/version-0.3.3--rust-blue) ![License](https://img.shields.io/badge/license-MIT-green)

PoshBuddy is a professional management tool for Oh My Posh configurations. It provides a high-density terminal user interface (TUI) designed to streamline theme customization, font management, and shell segment manipulation with surgical precision.

![PoshBuddy Demo](demo.gif)

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
- **Accurate Previews**: Environment isolation ensures that theme previews are unaffected by your current shell state.
- **Diagnostic Intelligence**: Automated checks for Nerd Fonts, shell versions, and terminal compatibility (Windows Terminal recommended).

## Technical Architecture

PoshBuddy is built for performance and reliability using modern systems programming patterns:

- **Rust**: Core language for memory safety and execution speed.
- **Tokio**: Asynchronous runtime for non-blocking network operations and installers.
- **Ratatui**: State-of-the-art framework for the TUI rendering loop.
- **Serde**: High-performance serialization for JSON configuration manipulation.

## Installation

Ensure you have the [Rust toolchain](https://rustup.rs/) installed.

```powershell
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy
cargo install --path .
```

*Note: This tool requires the Oh My Posh binary to be present in your system PATH.*

## Controls and Navigation

| Key | Action |
| :--- | :--- |
| **T** | **Themes Explorer** — Browse local and official remote themes. |
| **F** | **Font Manager** — Install and manage Nerd Fonts. |
| **S** | **Segment Manager** — Toggle theme components in real-time. |
| **D** | **Diagnostics** — Analyze system health and configuration. |
| **B** | **Backups** — Access version history of your profile. |
| **Enter** | Apply theme, toggle segment, or start installation. |
| **Q / Esc** | Exit the application. |

## Community and Support

Contributions to PoshBuddy are welcome. For technical details, troubleshooting, or feature requests, please refer to the following resources:

- [Wiki](docs/wiki/index.md) - Technical deep dive and architectural overview.
- [GitHub repository](https://github.com/julesklord/poshbuddy) - Source code and issue tracker.
- [Changelog](CHANGELOG.md) - Detailed version history.

---

Developed for terminal perfectionists.
**Your prompt. Your rules. Zero friction.**

[GitHub](https://github.com/julesklord/poshbuddy) · [Documentation](./docs) · [Changelog](./CHANGELOG.md)
