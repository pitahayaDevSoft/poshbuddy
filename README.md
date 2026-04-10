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

Customizing a PowerShell prompt shouldn't feel like a chore. PoshBuddy eliminates the manual labor of editing profiles and configuration files, bringing a **modern TUI experience** to your workflow.

- **Zero-Config Profile Sync**: Automatically detects and updates both PowerShell 5.1 and 7 profiles.
- **Accurate Previews**: See exactly how a theme looks before applying it, with full ANSI color support.
- **Dependency Guardian**: Built-in installer for Oh My Posh and Nerd Fonts.

---

## ✨ Feature Highlights

| Feature | Description |
| :--- | :--- |
| **🎨 Real ANSI Preview** | Native rendering of theme prompts using the `ansi-to-tui` engine. |
| **🚀 Smart Installer** | Missing `oh-my-posh`? We'll install it for you via Winget in real-time. |
| **⚕️ Onboarding Diagnostics** | Guided system check for fonts, terminal support, and shell versions. |
| **🔤 Font Discovery** | Browse and install Nerd Fonts directly from the application. |
| **🔗 Universal $PROFILE** | Dynamic detection of profiles, even if relocated to custom drives. |

---

## 🚀 Getting Started

Ensure you have [Rust](https://rustup.rs/) installed.

```powershell
# Get the source
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy

# Compile and Launch
cargo run --release
```

### 🎮 Controls
- `[1]` **Themes Explorer** — Browse and filter styles.
- `[2]` **Font Manager** — Get the glyphs you need.
- `[TAB]` or `Arrows` to navigate.
- `[ENTER]` to apply changes globally.

---

## 🗺️ Roadmap: The Future of PoshBuddy

We are building the next generation of terminal customization tools.

- [ ] **🔌 Plugin Architecture**: One-click install for PowerShell modules (Z-Location, posh-git, etc.).
- [ ] **🌐 Native Globalization**: Full multi-language support (English/Spanish/More).
- [ ] **📦 Universal Binaries**: Standalone installers via Scoop and Winget.
- [ ] **💾 Cloud Sync**: (Experimental) Backup and sync your prompt settings across machines.

---

## 🤝 Community & Support

**Contributions are highly welcome!** Check out our [Wiki](docs/wiki/index.md) for deep dives into internals or the troubleshooting guide.

*Built with ❤️ for the developer community on Windows.*

<!-- markdownlint-enable MD033 -->
