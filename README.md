# PoshBuddy

![PoshBuddy Logo](https://raw.githubusercontent.com/julesklord/poshbuddy/main/assets/poshbuddy_logo.png)

**The definitive TUI manager for Oh My Posh in Windows and PowerShell environments**

  [![Rust](https://img.shields.io/badge/Rust-1.94+-orange.svg)](https://www.rust-lang.org) [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE) [![Platform](https://img.shields.io/badge/Platform-Windows-lightgray.svg)]()  [![State](https://img.shields.io/badge/State-Beta-green.svg)]()
</div>

<br>
**PoshBuddy** is a lightweight, fast, and beautifully designed terminal user interface (TUI) built in Rust. It's designed to simplify the installation, management, and customization of themes and fonts for **Oh My Posh**. Say goodbye to manual profile editing and JSON juggling—PoshBuddy does the heavy lifting for you with an immersive visual experience.

---

## ✨ Key Features

- **🎨 Real ANSI Preview**: Visualize Oh My Posh themes directly in your terminal with true color rendering and glyphs thanks to `ansi-to-tui`. It uses environment isolation to ensure previews are accurate and unaffected by your current shell settings.
- **🚀 Automatic Installation (Dependency Manager)**: PoshBuddy automatically detects if `oh-my-posh` is installed. If not, it offers a transparent, real-time installer using `winget` directly from the TUI.
- **⚕️ Dynamic Diagnostics (Onboarding)**: Evaluates your environment at startup. It alerts you if a Nerd Font is missing, if you're using an outdated PowerShell version (5.1 vs 7+), or an obsolete console compared to modern emulators like Windows Terminal.
- **🔗 Dynamic Multi-Profile Support**: Apply your chosen theme instantly across all detected PowerShell installations (classic Windows PowerShell and PowerShell Core), regardless of where your documents folder is located.
- **🔤 Font Manager (Nerd Fonts)**: Browse, download, and install popular *Nerd Fonts* required for Oh My Posh icons without ever leaving the application.

## 🛠️ Prerequisites

For the optimal experience (The Golden Standard), we recommend:

- **[Windows Terminal](https://github.com/microsoft/terminal)**
- **[PowerShell 7+](https://github.com/PowerShell/PowerShell)**
- A **Nerd Font** configured as your primary font in your terminal settings.

*(Note: PoshBuddy works in classic consoles, but the visual experience is vastly superior when meeting the above requirements. The app will guide you during the onboarding process).*

## 📦 Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

```powershell
# Clone the repository
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy

# Build and run
cargo run --release
```

## 🎮 Usage

1. Launch the executable or run `cargo run`.
2. Review the **System Diagnostics** screen and press `[ENTER]` to start.
3. Use `[UP]` and `[DOWN]` arrow keys to navigate the theme list.
4. Switch panels with `[TAB]` or use shortcuts (`[1]` for Themes, `[2]` for Fonts).
5. Select a theme or font and press `[ENTER]` to apply or install it.
6. **Enjoy your new prompt**. PoshBuddy will notify you when to reload your terminal to see the changes.

## 🗺️ Roadmap

PoshBuddy is actively evolving. We are currently paving the way for the following key features (V0.3.0+):

- [ ] **🔌 Plugin Support**: A dedicated manager to add extra segments, auxiliary scripts, and prompt modules.
- [ ] **🌐 Multi-language Support (i18n)**: Implementing native multi-language support (starting with English and Spanish) to open PoshBuddy to the global community.
- [ ] **📦 Binary Distributions**: Availability via WinGet and Scoop for installation without needing the Rust runtime.

## 🤝 Contributing

Contributions are welcome! If you'd like to help expand PoshBuddy (including the multi-language and plugin milestones):

1. *Fork* the project.
2. Create a feature branch (`git checkout -b feature/NewFeature`).
3. *Commit* your changes (`git commit -m 'feat(scope): add NewFeature'`).
4. *Push* to the branch (`git push origin feature/NewFeature`).
5. Open a **Pull Request**.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and codebase conventions.

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

---

*Built with ❤️ for the developer community on Windows.*
