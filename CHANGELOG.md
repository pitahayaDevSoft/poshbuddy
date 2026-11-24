# Changelog

All notable changes to PoshBuddy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1] - 2026-11-21

### Fixed
- **CLI Remote Theme & Font Stability**: Fixed remote catalog channel handling in CLI commands (`set theme`, `list themes`, `list fonts`, `install font`) so `RemoteThemesLoaded` and `FontsLoaded` are no longer discarded and network failures exit cleanly with non-zero status codes instead of deadlocking.
- **Guaranteed Terminal Restoration**: Added an RAII `TerminalGuard` and panic hook in `run_tui()` so raw mode, alternate screen, and cursor visibility are always restored on error or panic.
- **CLI Font Installation & Exit Status Validation**: Preloaded the Nerd Fonts catalog in `poshbuddy install font`, added strict font name validation (`App::is_valid_font_name`), and checked `oh-my-posh` process exit codes in both single and bulk font installations.
- **Multi-Shell Backup Support**: Skipped non-existent detected shell profiles during `apply_theme_advanced` / `toggle_plugin` backup stages, and fixed `BackupManager::list_backups` / `restore_latest` to list, rotate, and restore Fish (`config.fish`) and Unix shell profile backups.
- **TUI Startup & Kitty Test Stability**: Removed duplicate startup GitHub catalog fetch in `run_tui()` and converted `test_handle_messages_fonts_loaded` to `#[tokio::test]` so tests pass inside Kitty terminals.
- **CI/CD & Repository Migration**: Updated repository/Pages URLs to `pitahayadevsoft/poshbuddy`, switched `security.yml` to the open-source Gitleaks CLI (`v8.30.1` with SHA-256 verification), fixed `static.yml` to publish `docs/`, and expanded `rust.yml` to run `fmt`, `clippy`, and `test` across Ubuntu, Windows, and macOS.

### Changed
- **CLI Version Flag**: Added `--version` support via Clap.
- **Dependency Updates**: Updated `Cargo.lock` across 194 crates (`dirs 7.0.0`, `ratatui 0.30.2`, `tokio 1.54.1`, `reqwest 0.13.5`, `clap 4.6.9`, `rustls 0.23.46`, `aws-lc-rs 1.19.0`, `quinn-proto 0.11.19`, `chacha20 0.10.2`), removed unused `futures` crate and `crossterm/event-stream` feature, and moved `tempfile` to `[dev-dependencies]`.

## [0.8.0] - 2026-08-04

### Added
- **Dynamic Official Schema Sync**: Automatic background fetching and parsing of all 117+ official segments from Oh My Posh's upstream repository (`JanDeDobbeleer/oh-my-posh`).
- **Bulletproof Segment Template Guards**: Enclosed all segment properties (`.Full`, `.Context`, `.Profile`, `.Track`, `.CumulativeTotal`, `.FormattedMs`) in Go template conditional guards (`{{ if ... }}`) to prevent Oh My Posh rendering panics and `unable to create text based on template` errors.
- **Active Prompt Preview Bar**: Added a thin preview box above navigation tabs displaying the live ANSI prompt or active segment structure.
- **Diagnostics Engine (`poshbuddy doctor`)**: Comprehensive system, shell profile syntax, font, and binary health diagnostic tool.

## [0.7.0] - 2026-08-04

### Added
- **Complete Oh My Posh Segment Engine & Catalog**: Integrated 70+ official segments across 6 categories (Version Control, System, Development, Package Managers, Cloud, Time) with surgical JSON manipulation (`add_segment`, `remove_segment`, `toggle_segment`, `move_segment`).
- **Active Prompt Preview Bar**: Added a thin preview box above the navigation tabs displaying the live rendering or dynamic segment structure of the active theme.
- **Diagnostics Module (`poshbuddy doctor` / `poshbuddy diagnostics`)**: Added system, binary, font, profile syntax, and permission health checks in both TUI and CLI.
- **CLI Segment Commands**: Added `poshbuddy list segments`, `poshbuddy toggle segment <name>`, and `poshbuddy set segment <name>`.

## [0.6.1] - 2026-06-30

### Changed
- **Compact & Uniform TUI Tab Layout:** Redesigned the main tab bar to use consistent rounded borders for active and inactive views and unified the left list and right preview column widths across all tabs to 40% / 60% for a smooth transition.
- **Smaller & Tighter VHS Demo Recording:** Re-recorded the project demo using a classic `Builtin Dark` theme (avoiding modern themes), a smaller font size (`10`), and a minimum line height (`1.0`) to make the interface visual representation more compact.

## [0.6.0] - 2026-06-26

### Added
- **Split ANSI Prompt Preview**: Divided the ANSI theme preview panel into horizontal columns (Left Prompt and Right Prompt).
- **Smart Whitespace Splitter**: Implemented a string splitting algorithm to separate left and right segments based on the largest spacing gaps while preserving ANSI styling.
- **Alignment Simulation**: Configured the right prompt column to render text with right alignment (`Alignment::Right`), simulating native terminal prompt layout.

### Changed
- **Super Compact Welcome Logo**: Reduced the welcome screen logo height to 5 lines (down from 8) by using a compact 3-line ASCII block font and vertically centering the cat ears/chin, saving valuable vertical space on smaller terminals.
- **Lowered Welcome Screen Logo Threshold**: Lowered the minimum screen height requirement to render the welcome logo from 23 to 20 lines.

### Fixed
- **Clippy Warning Resolution**: Resolved a clippy warning regarding collapsible nested `if` statements in `split_line_by_gap`.

## [0.5.3] - 2026-06-25

### Added
- **Kitty Font Preview**: Implemented real font preview support using Kitty Graphics Protocol and improved preview formatting.
- **Dynamic Environment Detection**: Auto-detects active shell and terminal on Linux, macOS, and Windows.

## [0.5.2] - 2026-06-24

### Added
- **Horizontal Welcome Screen Logo Layout**: Replaced the vertical stack of the ASCII cat and text logo with a more compact horizontal layout. This reduces the logo's vertical height from 14 lines to 8 lines, giving standard terminals 6 extra lines of space.
- **Lower Screen Height Limit**: Lowered the screen height threshold to render the logo on smaller terminal windows (from 25 to 23).

### Fixed
- **Codebase Linting & Debt Resolution**: Ran a comprehensive clippy audit and resolved all style warnings, collapsible if-statements, assert-boolean comparisons, unnecessary clones, and unused helper code.

## [0.5.1] - 2026-06-03

### Added
- **Expanded Profile Detection**: Added support for more shell configuration files (bash, zsh, fish) and robust detection of installed shells.
- **Improved Path Handling**: `check_omp_installed` now searches common fallback paths and dynamically updates PATH if needed.
- **Cross-Platform Font Detection**: Improved Nerd Font detection for Linux and macOS.
- **UI Improvements**: Platform-specific installation instructions in the dependency missing screen.

### Fixed
- **Enhanced Dependency Management**: Improved Oh My Posh installation flow and more robust `pwsh` detection.
- **Reliability Fixes**: Ensured parent directories are created before writing profiles or installing dependencies.

## [0.5.0] - 2026-06-03

### Added
- **Multi-Shell Compatibility**: Integrated profile detection and theme injection formatting for Bash (`.bashrc`), Zsh (`.zshrc`), and Fish (`config.fish`) shells, alongside standard PowerShell (`.ps1`).
- **Linux Dependency Installation Fallback**: Configured the Oh My Posh installer to fall back to the official download and installation bash script (`curl -s https://ohmyposh.dev/install.sh | bash`) if Homebrew is missing.
- **Safety Guards**: Added profile path syntax filters in plugin toggling to skip non-PowerShell configurations.

### Fixed
- **Winget Label Leak**: Corrected error reporting strings on Unix platforms to specify the exact tool name (Homebrew or Official Install Script) instead of referencing Winget.
- **Test Isolation**: Hardened profile detection unit tests to mock `HOME`, `USERPROFILE`, and `XDG_CONFIG_HOME` directories, ensuring hermetic test environments.

## [0.4.5] - 2026-04-29

### Changed

- **Core Dependency Overhaul**: Updated `ratatui` to v0.30 and `crossterm` to v0.29 for improved performance and modern API support.
- **API Migration**: Refactored rendering logic to use `Frame::area()` instead of the deprecated `f.size()`.
- **System Identity Handling**: Updated `whoami` to v2.1, implementing robust `Result` handling for username and hostname resolution.
- **ANSI Rendering Pipeline**: Migrated `ansi-to-tui` to v8.0, optimizing the byte-stream to TUI text conversion trait.
- **Modernized Toolchain**: Updated project to Rust 2024 edition.

### Fixed

- **Compiler Warnings Purge**: Resolved all deprecation warnings related to TUI and system identity crates.

## [0.4.1] - 2026-04-23

### Changed

- **Modernized TUI Layout**: Replaced rigid structures with a dynamic, responsive 50/50 horizontal layout for the Dashboard.
- **Strict Nerd Font Integration**: Eliminated all standard emojis to prevent character width drift, ensuring pixel-perfect alignment across all terminals.
- **ASCII Logo Stabilization**: Standardized internal padding for the main logo to ensure consistent centering and eliminate the "staircase" visual bug.
- **Responsive Sizing**: Implemented a maximum container width (100 chars) for wide terminals and a stacked vertical layout for narrow terminals.

### Fixed

- **Icon Desync**: Resolved misalignment in the "System Identity" and "Quick Steps" columns by standardizing on Nerd Font glifs (`f140b`, etc.).

## [0.4.0] - 2026-04-22

### Added

- **Advanced Dashboard**: Completely redesigned welcome screen with real-time system diagnostics and quick-action menu.
- **Mass Installation Pipeline**: Integrated support for bulk font and plugin installations with global progress overlays.
- **Diagnostic Engine**: Real-time detection of terminal capabilities (Windows Terminal, VS Code) and Nerd Font presence.

### Fixed

- **Compiler Warnings**: Resolved all remaining `dead_code` warnings in the models layer.


## [0.3.4] - 2026-04-13

### Added

- **Startup Theme Scanning**: Automatically detects local `.omp.json` and `.json` themes in the themes directory upon application launch.
- **Initial Preview Load**: Automatically triggers a theme preview when first entering the Themes view from the Dashboard.
- **Enhanced Keybindings**: Standardized numeric navigation (1, 2, 3) for tabs and mnemonic quick actions (R, N, I, D, V, B) on the Dashboard.

### Fixed

- **Static Preview Bug**: Resolved the issue where theme previews failed to update during list navigation by correcting file extension handling and path canonicalization.
- **Active Theme Fallback**: Forced `oh-my-posh` to ignore the current shell prompt by clearing `POSH_THEME` and `OMP_CONFIG` environment variables during preview generation.
- **UI Freezes Eradicated**: Migrated all remaining synchronous `std::fs` operations to asynchronous `tokio::fs` to ensure smooth UI responsiveness during theme application and downloads.
- **Stuttering Fix**: Implemented an in-memory cache for active segments, eliminating per-frame disk I/O when rendering the segments list.
- **Keybinding Conflicts**: Resolved overlapping keybinds between global navigation and view-specific actions.

## [0.3.3-2-fix] - 2026-04-13

### Fixed

- **Preview Execution Restoration**: Re-enabled environment variable inheritance for the `oh-my-posh` process. This fixes the "Loading..." hang caused by the absence of critical system variables in Windows.
- **Process Leak Prevention**: Integrated `kill_on_drop(true)` in the preview command to ensure that any aborted task immediately terminates its child process.

## [0.3.3-1-fix] - 2026-04-13

### Fixed

- **Process Flooding Fix**: Implemented proactive `JoinHandle::abort()` on the preview task when selection changes, stopping orphan `oh-my-posh` processes mid-execution.
- **Request Isolation**: Added `preview_request_id` versioning to all preview tasks. The TUI now ignores late-arriving messages from old requests, eliminating race conditions and UI "jitter".
- **Instant TUI Feedback**: UI now displays "Loading preview..." immediately upon navigation input, providing a predictable and responsive user experience.
- **Resource Leak Mitigation**: Eradicated task accumulation that led to excessive memory and handle consumption during fast scrolling.

## [0.3.3] - 2026-04-13

### Added

- **Network Robustness Layer**: Integrated a centralized HTTP client engine with mandatory 10-second timeouts to eliminate TUI hangs.
- **Connectivity Pre-flight**: Added automatic internet availability checks before initiating theme downloads or font installations.
- **Binary Execution Guard**: Implemented a 2-second timeout wrapper around `oh-my-posh` binary calls to prevent external process stalls in the UI thread.
- **Mass Font Installer**: High-performance sequential font installer with a global progress bar and stability verification.
- **TUI State Security**: Hardened the state machine to handle unexpected network drops and binary failures gracefully.

### Changed

- Refactored `src/api.rs` to use a singleton-pattern robust client.
- Updated `index.md` and project Wiki to reflect the "Unstuck" architecture.

## [0.3.2] - 2026-04-13

### Added

- **4-Stage Installation Pipeline**: Atomic theme application process (Download -> Verify -> Backup -> Apply).
- **Manual Backups**: Dedicated hotkey `B` to trigger user-initiated profile snapshots.
- **Enhanced Navigation**: Global `Esc` and `H` mapping to ensure instant return to the Dashboard.

### Changed

- **Full Localization (EN)**: Completed the 100% translation of all UI assets and internal documentation.
- **Code Hygiene**: Purged all legacy code and addressed compiler warnings for 0-warning builds.
- **Standardized Config Markers**: Migrated to `## POSHBUDDY AUTO-GENERATED CONFIG` for consistent profile management.

## [0.3.1] - 2026-04-12

### Added

- **Interactive Theme Previews**: Real-time ANSI rendering of Oh My Posh themes within the TUI.
- **Environment Isolation**: Used `env_clear` for OMP preview commands to prevent host shell pollution.
- **English Documentation**: Initial draft of core technical docs and inline comments.

### Fixed

- Resolved race conditions in asynchronous theme loading.
- Optimized terminal buffer clearing during rapid navigation.

## [0.2.0] - 2026-04-09

### Added

- **Modern Dashboard**: High-density grid-based UI replacing simple lists.
- **System Diagnostics**: Automated Nerd Font detection and PowerShell capability analysis.
- **Onboarding Experience**: Guidance screen for first-time configuration of Oh My Posh.
- **Success Feedback**: Visual confirmation modals for theme and font operations.

## [0.1.0] - 2026-04-09

### Added

- **Core TUI Engine**: Initial implementation using `ratatui` and `crossterm`.
- **Oh My Posh Integration**: Automated dependency check and interactive installer via WinGet.
- **Profile Detection**: Multi-shell support (PS 5.1 & 7) with dynamic path discovery ($PROFILE).
- **Theme Asset Engine**: Unified management of local files and GitHub-hosted theme catalogs.
