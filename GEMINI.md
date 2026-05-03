# PoshBuddy: Project Instructions

This document provides foundational mandates and context for working on the PoshBuddy project.

## Project Overview

PoshBuddy is a professional management tool for Oh My Posh configurations, written in Rust. It features both a responsive Terminal User Interface (TUI) and a robust Command Line Interface (CLI).

- **Purpose:** Simplify Oh My Posh theme management, segment manipulation, and Nerd Font installation.
- **Main Technologies:**
  - **Core:** Rust (2024 Edition)
  - **Async Runtime:** [Tokio](https://tokio.rs/)
  - **TUI:** [Ratatui](https://ratatui.rs/) with [Crossterm](https://github.com/crossterm-rs/crossterm)
  - **CLI:** [Clap](https://clap.rs/)
  - **HTTP:** [Reqwest](https://github.com/seanmonstar/reqwest)
  - **Serialization:** [Serde](https://serde.rs/)
  - **Testing:** Rust standard tests and [Playwright](https://playwright.dev/) for E2E/UI.

## Architecture

The project follows a modular architecture centered around an asynchronous message-passing system.

- **`src/main.rs`**: Entry point. Dispatches to CLI commands if arguments are provided, otherwise launches the TUI.
- **`src/cli.rs`**: Defines the CLI command surface and argument parsing using Clap.
- **`src/ui.rs`**: Contains the TUI rendering loop and view logic using Ratatui.
- **`src/app/`**: The core application engine.
  - `mod.rs`: Orchestrates state and communication.
  - `handlers.rs`: Bridges UI/CLI events to service actions.
  - `services.rs`: Contains business logic for theme application and font management.
  - `models.rs`: Defines the application state and data models.
- **`src/api.rs`**: Handles remote interactions with the Oh My Posh theme catalog.
- **`src/backup.rs`**: Manages safe backups of PowerShell profiles.
- **`src/diagnostic.rs`**: Performs environment checks (fonts, shell versions).
- **`src/plugin_installer.rs`**: Logic for downloading and installing fonts/binaries.

## Development Workflows

### Building and Running
- **Build project:** `cargo build`
- **Run TUI:** `cargo run`
- **Run CLI commands:** `cargo run -- <command> <args>`1
  - Example: `cargo run -- set theme bubbles`
  - Example: `cargo run -- list fonts`
- **Install locally:** `cargo install --path .`

### Testing and Validation
- **Run Rust tests:** `cargo test`
- **Run Playwright tests:** `npx playwright test`
- **Linting:** `cargo clippy`
- **Formatting:** `cargo fmt`

## Development Conventions

- **Rust Idioms:** Adhere to standard Rust practices and the 2024 edition features.
- **Asynchronous Logic:** Use `tokio::sync::mpsc` channels for communication between services and the UI. UI/CLI should remain responsive during long-running tasks (e.g., downloads).
- **Safety and Reliability:**
  - Always create backups before modifying system files (e.g., PowerShell profiles).
  - Use non-destructive "marker-based" injection for profile modifications.
  - Ensure cross-platform compatibility where possible, though Windows is the primary target.
- **Error Handling:** Use `Result` and informative error messages. Prefer structured logging or status messages via the `AppMessage` system.
- **TUI Responsiveness:** Ensure the Ratatui terminal loop is not blocked by synchronous I/O; delegate to Tokio tasks.

## Key Files

- `Cargo.toml`: Rust dependencies and metadata.
- `README.md`: High-level user documentation.
- `DESIGN.md`: Architectural notes.
- `AGENTS.md`: Specific instructions for AI agents (complementary to this file).
- `package.json`: Node.js dependencies for Playwright testing.
