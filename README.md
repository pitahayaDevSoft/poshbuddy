# PoshBuddy

![Build](https://github.com/julesklord/poshbuddy/actions/workflows/rust.yml/badge.svg) ![Version](https://img.shields.io/badge/version-0.4.7-blue) ![License](https://img.shields.io/badge/license-MIT-green) [![Rust](https://github.com/julesklord/poshbuddy/actions/workflows/rust.yml/badge.svg)](https://github.com/julesklord/poshbuddy/actions/workflows/rust.yml) [![Security Scan](https://github.com/julesklord/poshbuddy/actions/workflows/security.yml/badge.svg)](https://github.com/julesklord/poshbuddy/actions/workflows/security.yml)

PoshBuddy is a TUI tool for Oh My Posh configurations. It provides a modernized, responsive terminal user interface (TUI) designed for high-density information display, surgical theme manipulation, and seamless Nerd Font management.

<p align="center">
  <img src="assets/demo.webp" alt="PoshBuddy TUI Demo" width="90%">
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

The easiest way to install PoshBuddy is via [crates.io](https://crates.io/crates/poshbuddy):

```powershell
cargo install poshbuddy
```

### Building from Source

If you prefer to build from source, ensure you have the [Rust toolchain](https://rustup.rs/) installed.

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

### Graphic Structure


```mermaid
flowchart TD

subgraph group_runtime["Runtime"]
  node_main(("Main<br/>entrypoint<br/>[main.rs]"))
  node_cli["CLI<br/>command surface<br/>[cli.rs]"]
  node_ui["TUI<br/>terminal ui<br/>[ui.rs]"]
  node_appmod{{"App core<br/>orchestration module<br/>[mod.rs]"}}
  node_handlers["Handlers<br/>event bridge<br/>[handlers.rs]"]
  node_services["Services<br/>business logic<br/>[services.rs]"]
  node_models["Models<br/>state model<br/>[models.rs]"]
  node_api["Theme API<br/>remote catalog<br/>[api.rs]"]
  node_assets["Assets<br/>asset catalog<br/>[assets.rs]"]
  node_backup[("Backup<br/>safe edit<br/>[backup.rs]")]
  node_installer["Installer<br/>dependency install"]
  node_diagnostic["Diagnostics<br/>environment checks<br/>[diagnostic.rs]"]
end

subgraph group_external["External"]
  node_profilefs[("Profiles<br/>filesystem")]
  node_ompremote["OMP themes<br/>remote source"]
  node_fonts["Nerd Fonts<br/>download source"]
  node_powershell(("PowerShell<br/>host environment"))
end

subgraph group_delivery["Delivery"]
  node_githubci["GitHub CI<br/>automation<br/>[workflows]"]
  node_tests["Tests<br/>integration harness<br/>[example.spec.ts]"]
end

node_main -->|"dispatches"| node_cli
node_main -->|"dispatches"| node_ui
node_main -->|"boots"| node_appmod
node_cli -->|"uses"| node_services
node_ui -->|"events"| node_handlers
node_handlers -->|"invokes"| node_services
node_appmod -->|"holds"| node_models
node_appmod -->|"wires"| node_handlers
node_appmod -->|"orchestrates"| node_services
node_services -->|"queries"| node_api
node_services -->|"manages"| node_assets
node_services -->|"protects"| node_backup
node_services -->|"installs"| node_installer
node_services -->|"checks"| node_diagnostic
node_services -->|"updates"| node_models
node_api -->|"fetches"| node_ompremote
node_assets -->|"syncs"| node_ompremote
node_backup -->|"writes"| node_profilefs
node_installer -->|"downloads"| node_fonts
node_diagnostic -->|"inspects"| node_powershell
node_services -->|"edits"| node_profilefs
node_services -->|"guards"| node_powershell
node_githubci -->|"runs"| node_tests

click node_main "https://github.com/julesklord/poshbuddy/blob/main/src/main.rs"
click node_cli "https://github.com/julesklord/poshbuddy/blob/main/src/cli.rs"
click node_ui "https://github.com/julesklord/poshbuddy/blob/main/src/ui.rs"
click node_appmod "https://github.com/julesklord/poshbuddy/blob/main/src/app/mod.rs"
click node_handlers "https://github.com/julesklord/poshbuddy/blob/main/src/app/handlers.rs"
click node_services "https://github.com/julesklord/poshbuddy/blob/main/src/app/services.rs"
click node_models "https://github.com/julesklord/poshbuddy/blob/main/src/app/models.rs"
click node_api "https://github.com/julesklord/poshbuddy/blob/main/src/api.rs"
click node_assets "https://github.com/julesklord/poshbuddy/blob/main/src/assets.rs"
click node_backup "https://github.com/julesklord/poshbuddy/blob/main/src/backup.rs"
click node_installer "https://github.com/julesklord/poshbuddy/blob/main/src/plugin_installer.rs"
click node_diagnostic "https://github.com/julesklord/poshbuddy/blob/main/src/diagnostic.rs"
click node_githubci "https://github.com/julesklord/poshbuddy/blob/main/.github/workflows"
click node_tests "https://github.com/julesklord/poshbuddy/blob/main/tests/example.spec.ts"

classDef toneNeutral fill:#f8fafc,stroke:#334155,stroke-width:1.5px,color:#0f172a
classDef toneBlue fill:#dbeafe,stroke:#2563eb,stroke-width:1.5px,color:#172554
classDef toneAmber fill:#fef3c7,stroke:#d97706,stroke-width:1.5px,color:#78350f
classDef toneMint fill:#dcfce7,stroke:#16a34a,stroke-width:1.5px,color:#14532d
classDef toneRose fill:#ffe4e6,stroke:#e11d48,stroke-width:1.5px,color:#881337
classDef toneIndigo fill:#e0e7ff,stroke:#4f46e5,stroke-width:1.5px,color:#312e81
classDef toneTeal fill:#ccfbf1,stroke:#0f766e,stroke-width:1.5px,color:#134e4a
class node_main,node_cli,node_ui,node_appmod,node_handlers,node_services,node_models,node_api,node_assets,node_backup,node_installer,node_diagnostic toneBlue
class node_profilefs,node_ompremote,node_fonts,node_powershell toneAmber
class node_githubci,node_tests toneMint

```

## Community and Support

Contributions to PoshBuddy are welcome. For technical details, troubleshooting, or feature requests, please refer to the following resources:

- [Wiki](docs/wiki/index.md) - Technical deep dive and architectural overview.
- [GitHub repository](https://github.com/julesklord/poshbuddy) - Source code and issue tracker.
- [Changelog](CHANGELOG.md) - Detailed version history.

---

Developed for terminal perfectionists.
**Your prompt. Your rules. Zero friction.**

[GitHub](https://github.com/julesklord/poshbuddy) · [Documentation](./docs) · [Changelog](./CHANGELOG.md)
