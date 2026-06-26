# PoshBuddy — Plugin Management System (PowerShell Modules)

## Goal

Add a new section to PoshBuddy that allows users to discover, install, and "activate" (auto-import in profiles) essential PowerShell modules that enhance the terminal experience (Icons, Git status, Directory jumping, etc.).

## Proposed Changes

### [MODIFY] [app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)

**1. New Data Structures**:
```rust
pub struct PluginAsset {
    pub name: String,
    pub description: String,
    pub documentation: String,
    pub module_name: String, // Actual name for Install-Module
}

pub enum ActiveView {
    Themes,
    Fonts,
    Plugins, // NEW
}
```

**2. List of Curated Plugins**:
Include a hardcoded list of high-quality modules:
- **Terminal-Icons**: File/folder icons for `ls`.
- **posh-git**: Advanced git status.
- **zoxide**: Smarter directory jumping.
- **PSReadLine**: Predictive IntelliSense configuration.

**3. Application Logic**:
- `install_plugin()`: Runs `Install-Module -Name X -Scope CurrentUser -Confirm:$false`.
- `toggle_plugin()`: Scans `$PROFILE` files and adds/removes `Import-Module X`.
- `is_plugin_active()`: Checks if the `Import-Module` line exists in the profile.

### [MODIFY] [ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)

**Plugins View**:
- **Left Panel**: List of available plugins with status indicators (`[X]` for Active, `[ ]` for Inactive, `(Not Installed)`).
- **Right Panel**: Shows the **Documentation** and use-cases for the selected plugin instead of a visual preview.
- **Controls**: `[ENTER]` to Install/Activate, `[BACKSPACE]` or `[X]` to Deactivate/Uninstall.

### [MODIFY] [main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)

- Add key listener for `[3]` to quickly jump to Plugins.
- Handle messages for plugin installation completion.

---

## Verification Plan

### Automated Tests
- `cargo check` to validate state logic.

### Manual Verification
1. Navigate to the Plugins tab.
2. Select `Terminal-Icons` and press ENTER.
3. Verify that the command `Install-Module` is triggered (viewable in logs if we use the existing log box).
4. Check that your `$PROFILE` now contains `Import-Module Terminal-Icons`.

## Open Questions

> [!IMPORTANT]
> Some plugins (like `zoxide`) require extra init logic (e.g., `zoxide init pwsh | Invoke-Expression`). Do you want me to handle these special initialization strings automatically for a curated list of plugins?
