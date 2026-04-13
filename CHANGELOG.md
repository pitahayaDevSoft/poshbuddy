# Changelog

All notable changes to PoshBuddy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
