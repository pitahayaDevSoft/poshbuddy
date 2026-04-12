# poshbuddy

![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Version](https://img.shields.io/badge/version-0.2.1-blue)
![License](https://img.shields.io/badge/license-MIT-green)

_Your prompt. Your rules. Zero friction._

---

## The Vision

You've stared at that ugly PowerShell prompt for months. You know Oh My Posh exists. You know it can transform your terminal into something that makes other devs ask "what theme is that?"

But then comes the friction:

- Digging through 200+ themes on GitHub
- Manual profile edits that break when you breathe wrong
- Fonts that render as rectangles
- Dependency hell between PowerShell 5.1 and 7
- That one time you broke `$PROFILE` and couldn't tab-complete anymore

**Your terminal should be a weapon, not a chore.**

**poshbuddy** is the TUI that treats your prompt like the precision instrument it is. Browse 200+ themes with live previews. Install fonts without leaving the terminal. Toggle PowerShell modules like you're switching channels. No `notepad $PROFILE`. No broken configs. No regrets.

This isn't theme management. This is **terminal aesthetics at the speed of thought**.

> "A developer's prompt is their throne. PoshBuddy makes sure it actually looks like one."
> — Someone who got tired of `cmd.exe` flashbacks

---

## Architecture: Battle-Tested in Rust

This isn't a Python script glued together with hope. **poshbuddy** is systems programming for people who care about milliseconds.

### Modular TUI Architecture

**Layer 1: Input**
| Component | Tech | Purpose |
|-----------|------|---------|
| Event Handler | Crossterm | Raw keyboard input capture |

**Layer 2: State Machine**
| State | Description |
|-------|-------------|
| `Onboarding` | System diagnostics |
| `Loading` | Async data fetch |
| `Main` | Interactive TUI |
| `Success/Error` | Result reporting |

**Layer 3: Async Runtime (Tokio)**
| Service | Data Source |
|---------|-------------|
| Theme Fetcher | GitHub API |
| Font Installer | Oh My Posh CLI |
| Preview Generator | Isolated env |
| Profile Sync | Multi-shell detection |

**Layer 4: Render**
| Component | Performance |
|-----------|-------------|
| Ratatui | 60fps, zero-allocation hot path |
| ansi-to-tui | ANSI sequence conversion |

**Targets:** PowerShell 5.1 / PowerShell 7 / Windows Terminal / Nerd Fonts

### Zero-Copy Previews

Theme previews run in an **isolated environment** (`env_clear`) with only essential paths. What you see in the TUI is exactly what you get in a fresh shell — no pollution from your current session.

### Low-Overhead Profile Sync

| Operation        | Manual Method                      | poshbuddy                      |
| ---------------- | ---------------------------------- | ------------------------------ |
| Theme discovery  | Browse GitHub, download JSON       | Live API fetch + filter        |
| Profile update   | Edit `$PROFILE`, pray it works     | Atomic write, backup preserved |
| Font install     | GUI wizard, 15 clicks              | One `ENTER` key                |
| Multi-shell sync | Manual copy-paste between PS 5.1/7 | Automatic detection & sync     |
| Rollback         | Hope you committed to git          | Non-destructive, idempotent    |

### Headless Capable

Every operation that's possible in the TUI is exposed through the underlying logic. Build your own automation:

```rust
use poshbuddy::App;

let app = App::new();
app.apply_theme("jandedobbeleer.omp.json")?;  // Single API call
```

---

## Showcase: Terminal Velocity

Launch it:

```powershell
# One command, total control
cargo run --release
```

### What You Get

**Header:** `PoshBuddy v0.2.1-rust`

**Left Panel: Themes Explorer**
| Selection | Theme |
|-----------|-------|
| `>>` | bubbles.omp.json |
| | jandedobbeleer... |
| | atomic.omp.json |
| | catppuccin... |
| | [Type to filter] |

**Right Panel: Visual Preview**

```
~> poshbuddy (main)
```

**Context Panel:**

- **Theme:** bubbles.omp.json
- **Profile Sync:** 2 shells detected

### The Power of Keys

| Key       | What It Does                                            |
| --------- | ------------------------------------------------------- |
| `[1]`     | **Themes** — Browse 200+ themes, live preview           |
| `[2]`     | **Fonts** — Install Nerd Fonts without leaving terminal |
| `[3]`     | **Plugins** — Toggle posh-git, Terminal-Icons, zoxide   |
| `[a-z]`   | **Instant filter** — Type "catp" to find catppuccin     |
| `[↑↓]`    | Navigate with wrap-around                               |
| `[ENTER]` | Apply theme / Install font / Toggle plugin              |
| `[Q]`     | Clean exit, always                                      |

---

## Quick Start: No Excuses

**Prerequisites:**

- Rust 1.70+ (we use modern features)
- Windows (PowerShell 5.1 or 7)
- A terminal that supports Unicode (Windows Terminal recommended)

```powershell
# Clone and build
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy
cargo build --release

# Run
cargo run --release

# Or install permanently
cargo install --path .
```

**First Launch:**

1. PoshBuddy diagnoses your system (font, shell, terminal)
2. If Oh My Posh is missing, it offers to install via winget
3. Themes auto-sync from GitHub
4. You browse, preview, and apply in under 30 seconds

---

## The Stack (Why It's Fast)

- **Rust** — Zero-cost abstractions, memory safety without GC
- **Tokio** — Async runtime, concurrent theme/font fetching
- **Ratatui** — Immediate-mode TUI, 60fps render loop
- **Crossterm** — Cross-platform terminal control
- **ansi-to-tui** — Zero-copy ANSI sequence parsing

Release profile optimized for size and speed:

- `opt-level = 3` — Maximum optimization
- `lto = true` — Link-time optimization
- `codegen-units = 1` — Single codegen unit
- `panic = "abort"` — No unwinding overhead
- `strip = true` — Debug symbols stripped

---

## The Identity

This tool was born from the intersection of:

- **Arch Linux ricing culture** — If it doesn't look good, it doesn't ship
- **Windows system administration** — PowerShell is a real shell, fight me
- **Rust systems programming** — Safety and speed aren't mutually exclusive
- **Developer ergonomics** — Every keystroke should have purpose

It's software by someone who:

- Knows the difference between `pwsh` and `powershell`
- Has strong opinions on Nerd Font ligatures
- Believes `winget` is the best thing to happen to Windows
- Uses `cmd /c` as a swear word

---

## Roadmap

- [x] Theme browser with live previews
- [x] Font installation via Oh My Posh CLI
- [x] Plugin toggle (posh-git, Terminal-Icons, zoxide)
- [x] Multi-shell profile sync (PS 5.1 + 7)
- [x] Onboarding diagnostics
- [ ] Scoop/Winget distribution
- [ ] Linux/macOS port (for the WSL crowd)
- [ ] Custom theme builder
- [ ] Cloud profile sync

---

<p align="center">
  Built for terminal perfectionists, by one.<br>
  <strong>Your prompt. Your rules. Zero friction.</strong>
</p>

<p align="center">
  <a href="https://github.com/julesklord/poshbuddy">GitHub</a> ·
  <a href="./docs">Documentation</a> ·
  <a href="./CHANGELOG.md">Changelog</a>
</p>
