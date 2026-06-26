# PoshBuddy — Global Code Documentation (English)

## Objective

Standardize the codebase documentation by adding meaningful inline comments in English. This will facilitate global collaboration, improve maintainability, and clarify the internal logic of the modular architecture (TUI, API, State Management).

## Proposed Changes

### [MODIFY] [main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)
- Document the entry point, terminal initialization, and the core event loop.
- Explain the `mpsc` message handling between the async tasks and the UI thread.

### [MODIFY] [app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)
- Document the `AppState` and `App` structures.
- Explain the dynamic `$PROFILE` detection and the multi-shell theme application logic.
- Clarify the isolation environment for Oh My Posh previews.

### [MODIFY] [ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)
- Document the rendering of various application states (Onboarding, Success, Main).
- Explain the layout constraints and the integration of `ansi-to-tui` for real-time previews.

### [MODIFY] [api.rs](file:///g:/DEVELOPMENT/poshbuddy/src/api.rs)
- Document the GitHub repository interaction for fetching themes and fonts.
- Explain the JSON parsing and filtering logic.

---

## Verification Plan

### Automated Tests
- `cargo check` to ensure no syntax errors were introduced during commenting.

### Manual Verification
- Review the source code files to ensure comments are clear, concise, and professional.

## Open Questions

> [!NOTE]
> I will replace existing Spanish comments with clear English documentation to maintain a single language standard across the codebase.
