# Bug Report: Index Out of Bounds and Null Reference on Enter with Empty Filter

## Description
In `Start-PoshBuddy`, the logic for applying a theme (vKey 13) does not check if any themes are actually filtered. If a user types a filter that returns 0 results and presses Enter, the script attempts to reference an index in an empty list and call `Apply-Theme` with an invalid or null path.

## Root Cause Investigation
In the `switch ($vKey)` block for `13`:
```powershell
13 { 
    $p = $Host.UI.RawUI.CursorPosition; ...
    Write-Host "¿Aplicar $($filtered[$index])? (s/n): " -NoNewline
    if ((Read-Host) -eq "s") {
        Apply-Theme $localPath
        ...
    }
}
```
If `$total` is 0, `$filtered` is an empty collection or `$null`. `$filtered[$index]` evaluates to `$null`.
More critically, `$localPath` is only updated inside an `if ($total -gt 0)` block. If `$total` is 0, `$localPath` might contain a value from a *previous* valid theme, causing the user to inadvertently apply a theme that is no longer visible, or `$null` if it's the first run.

## Impact
- **Severity:** Medium
- **Frequency:** High (if the user makes a typo in the filter and instinctively presses Enter).
- **Behavior:** Incorrect profile configuration (applying an empty or wrong config path).

## Proposed Fix
- Wrap the entire "Enter" logic in an `if ($total -gt 0)` check.
- Provide feedback if the user tries to select/apply when no themes are available.
- Ensure `$localPath` is reset or checked before use.

## Evidence
Reproduction: Type "non-existent-theme-xyz" in the filter (listing 0 results) and press Enter.
