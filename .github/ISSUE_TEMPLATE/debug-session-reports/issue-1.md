# Bug Report: Cache Corruption Vulnerability

## Description
`PoshBuddy.ps1` fails to handle cases where the `themes_cache.json` file exists but is empty or contains malformed JSON. This occurs because `ConvertFrom-Json` is called without a `try/catch` block or content validation when checking the cache age.

## Root Cause Investigation
In the `Get-Themes` function, the code checks for the existence of the cache file:
```powershell
if (Test-Path $PoshBuddy.CacheFile) {
    $cache = Get-Content $PoshBuddy.CacheFile | ConvertFrom-Json
    ...
}
```
If `themes_cache.json` is empty, `ConvertFrom-Json` throws an error: "ConvertFrom-Json: Input string was not in a correct format." This crashes the script or leads to an inconsistent state.

## Impact
- **Severity:** High
- **Frequency:** Rare but catastrophic (e.g., if a previous save attempt was interrupted or if the disk is full).
- **Behavior:** Script terminates with a red error message.

## Proposed Fix
- Wrap the cache loading logic in a `try/catch` block.
- Validate that the content is not null or whitespace before parsing.
- If parsing fails, treat the cache as invalid and proceed to fetch from the network.

## Evidence
Manual reproduction by creating an empty file: `Set-Content $PoshBuddy.CacheFile ""` followed by running the script.
