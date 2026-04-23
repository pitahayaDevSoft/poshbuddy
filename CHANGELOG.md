# Changelog

All notable changes to PoshBuddy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
