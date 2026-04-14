# PoshBuddy Wiki: Prerequisites & Setup

> **Updated**: 2026-04-13
> **Version**: v0.3.3-rust
> **Read Time**: 4 min  

PoshBuddy exists as a standalone binary but requires specific external dependencies to reach the "Golden Standard" terminal experience.

## Developer Environment

- **Rust Toolchain**: 1.76 or higher.
- **Git**: Required for cloning and updating the source repository.

## Runtime Dependencies

PoshBuddy executes validation checks for these at startup:
- **Connectivity**: A pre-flight internet check guaranteed to prevent UI hangs during remote operations.
- **Oh My Posh**: The core engine. If missing, PoshBuddy initiates a 1-click install via `winget` with transparent log streaming.
- **Nerd Fonts**: Essential for icon rendering and prompt integrity.

## Setup Verification

The **System Diagnostic** screen validates your environment upon execution:

```text
  🔍 SYSTEM DIAGNOSTICS

  [ √ ] Nerd Font Detected
  [ √ ] PowerShell 7 Detected
  [ ! ] Classic Console (Windows Terminal recommended)
```

### Remediation Steps

- **Nerd Font [ ! ]**: Access the **Fonts [2]** tab, select a font (e.g., MesloLGS NF), and execute the installer. You must then manually select this font in your terminal settings.
- **PowerShell 7 [ ! ]**: PoshBuddy is compatible with Windows PowerShell 5.1, but performance is optimized for [PowerShell 7](https://aka.ms/pscore6).
- **Classic Console [ ! ]**: If the diagnostic detects `conhost.exe`, icon quality and color depth are degraded. Install **Windows Terminal** from the Microsoft Store.

---
**Return to**: [Wiki Dashboard](./index.md)
