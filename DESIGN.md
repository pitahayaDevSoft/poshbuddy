# DESIGN.md - poshbuddy

## Architecture
PoshBuddy is a command-line utility and plugin manager written in Rust, aimed at enhancing shell experiences. 

## Key Modules
- main.rs, cli.rs: Entry points and CLI parsing.
- pi.rs, plugin_installer.rs: Core logic for fetching and installing plugins.
- ui.rs: Handles terminal user interface rendering and feedback.
- diagnostic.rs, ackup.rs: System state management and safety.

## Testing Strategy
- Unit tests written in Rust (cargo test).
- E2E and UI tests using Playwright (	ests/example.spec.ts).
