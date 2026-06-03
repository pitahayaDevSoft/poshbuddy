# Project Memory: PoshBuddy

## Status: v0.4.7 (Active Development)
- **Current Goal:** Enhance the TUI dashboard and stabilize Nerd Font installation logic.
- **Last Milestone:** Integrated the "Unstuck" guarantee with network timeouts and OMP binary guards.

## Persistent Context
- **Stack:** Rust (2024), Tokio, Ratatui, Clap, Reqwest, Playwright (Testing).
- **Core Files:** `src/app/mod.rs` (Orchestration), `src/ui.rs` (TUI), `src/cli.rs` (CLI).

## Active Tasks
- [ ] Finalize MonolithUI aesthetic integration for TUI components.
- [ ] Expand E2E testing coverage with Playwright.
- [ ] Optimize the ThemeAsset sync engine for faster discovery.

## Technical Debt
- Some UI handlers in `src/app/handlers.rs` are growing large and may need further decomposition.
- Profile injection logic could be more robust for edge-case shell configurations.

## Notes
- *2026-05-18:* Jules Dev Standard v1.0 applied. Documentation consolidated in `docs/`. Previous root-level `AGENTS.md` and `DESIGN.md` integrated and scheduled for removal.
- *2026-06-03:* Completed full redesign of PoshBuddy website (`docs/index.html` and `docs/index.css`) into an interactive riced tiling window manager desktop.
- *2026-06-03:* Added profile detection and theme injection formatting compatibility for Bash, Zsh, and Fish shells alongside PowerShell.
- The project uses a message-passing architecture between the UI/CLI and the App Core.