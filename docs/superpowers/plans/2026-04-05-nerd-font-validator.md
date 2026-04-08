# Nerd Font Validator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Implement a Nerd Font detection mechanism and display a warning in the PoshBuddy UI if a compatible font is not detected.

**Architecture:** Add a utility function `Test-NerdFont` to the model/data layer and integrate its result into the UI rendering loop within `Start-PoshBuddy`.

**Tech Stack:** PowerShell

---

### Task 1: Environment Setup & Git Configuration

**Files:**
- Modify: Local Git config
- Create: Branch `feat/nerd-font-validator`

- [x] **Step 1: Configure Git Identity**

```bash
git config user.name "Snuggles"
git config user.email "snuggles@poshbuddy.dev"
```

- [x] **Step 2: Create and switch to the feature branch**

```bash
git checkout -b feat/nerd-font-validator
```

### Task 2: Implement `Test-NerdFont` Function

**Files:**
- Modify: `/mnt/c/Users/julio/dev/poshbuddy/PoshBuddy.ps1`

- [x] **Step 1: Define `Test-NerdFont` in the script**

Insert the following function after the `$PoshBuddy` configuration hash table.

```powershell
function Test-NerdFont {
    try {
        $fontName = ""
        if ($IsWindows) {
            $fontName = $Host.UI.RawUI.FontName
            if (!$fontName) {
                $fontName = (Get-ItemProperty -Path "HKCU:\Console" -ErrorAction SilentlyContinue).FaceName
            }
        }
        return ($fontName -match "NF|Nerd|Retina")
    } catch {
        return $false
    }
}
```

### Task 3: Integrate Nerd Font Detection in `Start-PoshBuddy`

**Files:**
- Modify: `/mnt/c/Users/julio/dev/poshbuddy/PoshBuddy.ps1`

- [x] **Step 1: Call `Test-NerdFont` at the beginning of `Start-PoshBuddy`**

```powershell
function Start-PoshBuddy {
    $allThemes = Get-Themes; $index = 0; $filter = ""; $isRunning = $true
    $hasNerdFont = Test-NerdFont # ADD THIS LINE
    $lastWin = $Host.UI.RawUI.WindowSize; [Console]::Clear()
    # ...
```

- [x] **Step 2: Display the warning in the PREVIEW panel**

Inside the `View: Preview Panel` block, add the warning message if `$hasNerdFont` is false.

```powershell
        # View: Preview Panel
        if ($total -gt 0) {
            # ... (existing code)
            oh-my-posh print primary --config $localPath --shell pwsh | Out-String | Write-Host -NoNewline
            
            if (!$hasNerdFont) {
                $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = $panelH - 2
                $Host.UI.RawUI.CursorPosition = $p
                Write-Host "⚠️ Nerd Font no detectada. Los iconos podrían no verse bien." -ForegroundColor DarkYellow
            }
            
            $p.Y = 7; $Host.UI.RawUI.CursorPosition = $p; Write-Host "METRICAS DE CARGA:" -ForegroundColor DarkGray
            # ...
        }
```

### Task 4: Finalize and Commit

**Files:**
- Modify: Git Repository

- [x] **Step 1: Commit the changes**

```bash
git add PoshBuddy.ps1
git commit -m "feat: add Nerd Font detection and warning indicator"
```

- [x] **Step 2: Verify status**

```bash
git status
```
