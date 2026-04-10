<!-- markdownlint-disable MD033 -->
<div align="center">
  <img src="docs/logo.png" alt="PoshBuddy Logo" width="180" onerror="this.src='https://placehold.co/200x200/222222/00d2ff?text=PoshBuddy'"/>

  # PoshBuddy
  
  **The definitive TUI manager for Oh My Posh on Windows & PowerShell**
  
  *Sleek, fast, and unified theme management for your terminal.*

  <p align="center">
    <a href="https://www.rust-lang.org">
      <img src="https://img.shields.io/badge/Rust-1.94+-orange.svg?style=flat-square&logo=rust" alt="Rust"/>
    </a>
    <a href="LICENSE">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License"/>
    </a>
    <img src="https://img.shields.io/badge/Platform-Windows-blue?style=flat-square&logo=windows" alt="Platform"/>
    <img src="https://img.shields.io/badge/State-Beta-green?style=flat-square" alt="State"/>
  </p>
</div>

---

## ⚡ Why PoshBuddy?

Customizing a PowerShell prompt shouldn't feel like a chore. PoshBuddy eliminates the manual labor of editing profiles and configuration files, bringing a **modern TUI experience** to your developer workflow. Whether you're a seasoned terminal user or just starting your customization journey, PoshBuddy provides a safe, guided path to a beautiful prompt.

- **Zero-Config Profile Sync**: Automatically detects and updates both PowerShell 5.1 and 7 profiles simultaneously.
- **Accurate Previews**: See exactly how a theme looks before applying it, with full ANSI color support and environment isolation.
- **Dependency Guardian**: Built-in, transparent installer for Oh My Posh and Nerd Fonts.

---

## ✨ Deep Dive into Features

### 🎨 Real-Time Visual Previews
Under the hood, PoshBuddy uses a custom execution engine that runs Oh My Posh in an isolated environment (`env_clear`). This means the preview you see in the TUI is exactly what you get, unaffected by your current desktop environment or existing shell configurations.

### ⚕️ Onboarding & Diagnostics
Not sure why symbols are broken? PoshBuddy runs a comprehensive diagnostic at startup:
*   **Font Check**: Verifies the presence of Nerd Fonts.
*   **Shell Check**: Detects if you're on the slower PowerShell 5.1 or the high-performance PowerShell 7.
*   **Terminal Check**: Alerts you if you're using the legacy `conhost.exe` and recommends the modern **Windows Terminal**.

### 🚀 Smart Dependency Management
If Oh My Posh isn't found in your `$PATH`, PoshBuddy won't just fail. It offers to install it for you using `winget`, providing a live, scrollable terminal log so you can monitor the installation process with total transparency.

---

## 🛠️ Technical Stack

PoshBuddy is built for performance and reliability:
- **Rust**: The core language, ensuring safety and speed.
- **Tokio**: Powering the asynchronous background tasks (theme fetching, installers).
- **Ratatui**: The state-of-the-art framework for the TUI render loop.
- **ansi-to-tui**: Translating complex Oh My Posh ANSI sequences into TUI-compatible text.

---

## 📦 Getting Started

Ensure you have [Rust](https://rustup.rs/) installed.

```powershell
# Get the source
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy

# Compile and Launch
cargo run --release
```

### 🎮 Controls & Navigation

| Key | Action |
| :--- | :--- |
| `[1]` | **Themes Explorer** — Browse and filter styles. |
| `[2]` | **Font Manager** — Get the glyphs you need. |
| `[3]` | **Plugins** (Upcoming) — Extend your prompt functionality. |
| `[TAB]` | Cycle focus between the List and the Info panel. |
| `[ENTER]` | Apply theme or start installation. |
| `[ANY CHAR]` | Instantly filter the active list. |
| `[Q] / [ESC]` | Clean exit. |

---

## ❓ Frequently Asked Questions

**Q: Does it modify my $PROFILE permanently?**  
A: Yes, it adds or updates a single `oh-my-posh init` line. It is non-destructive to other configurations.

**Q: Can I use it on Linux/macOS?**  
A: While built in Rust, current profile sync is optimized for Windows PowerShell. Native Linux/macOS support is in the V0.4.0 roadmap.

**Q: Where are the themes stored?**  
A: Themes are cached in `~/.poshthemes/` and synced from the official Oh My Posh repository.

---

## 🗺️ Roadmap: The Future of PoshBuddy

- [ ] **🔌 Plugin Architecture**: One-click install for modules like Z-Location, posh-git, and PSReadLine.
- [ ] **🌐 Native Globalization**: Full multi-language support (starting with English and Spanish).
- [ ] **📦 Universal Binaries**: Official distribution via Scoop, Winget, and Chocolatey.
- [ ] **💾 Cloud Backup**: Sync your terminal aesthetic across all your dev machines.

---

## 🤝 Community & Support

**Contributions are highly welcome!** 
- Read our [Wiki](docs/wiki/index.md) for a technical deep dive.
- Check the [Troubleshooting Guide](docs/wiki/troubleshooting.md) if symbols don't show correctly.
- Post issues or feature requests on our GitHub tracker.

*Built with ❤️ for the developer community on Windows.*

<!-- markdownlint-enable MD033 -->
