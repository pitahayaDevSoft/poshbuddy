<#
.SYNOPSIS
    PoshBuddy - The Ultimate Windows Terminal Experience Manager.
.DESCRIPTION
    A professional TUI for managing Oh My Posh themes and PowerShell modules.
    Version: 0.1.0-beta.1 (MVC Refactor)
#>

# --- CONFIGURATION (GLOBAL STATE) ---
$PoshBuddy = @{
    Version   = "0.1.0-beta.1"
    ThemesDir = Join-Path $HOME ".poshthemes"
    Profile   = if ($PROFILE) { $PROFILE } else { Join-Path $HOME "Documents\PowerShell\Microsoft.PowerShell_profile.ps1" }
    CacheFile = Join-Path $HOME ".poshthemes\themes_cache.json"
}

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

# Ensure environment
if (!(Get-Command oh-my-posh -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: 'oh-my-posh' no detectado. Instálalo primero." -ForegroundColor Red
    exit
}
if (!(Test-Path $PoshBuddy.ThemesDir)) { New-Item -ItemType Directory -Path $PoshBuddy.ThemesDir -Force | Out-Null }

# --- MODEL (DATA LAYER) ---
function Get-Themes {
    $cacheValid = $false
    if (Test-Path $PoshBuddy.CacheFile) {
        try {
            $content = Get-Content $PoshBuddy.CacheFile -ErrorAction SilentlyContinue
            if (![string]::IsNullOrWhiteSpace($content)) {
                $cache = $content | ConvertFrom-Json
                $age = (Get-Date) - [DateTime]$cache.Timestamp
                if ($age.TotalHours -lt 24) { $cacheValid = $true }
            }
        } catch {
            $cacheValid = $false
        }
    }

    if ($cacheValid) {
        return $cache.Themes
    }

    $url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes"
    try {
        $themes = (Invoke-RestMethod -Uri $url -TimeoutSec 15 | Where-Object { $_.name -like "*.omp.json" }).name
        $cacheObj = @{ Timestamp = Get-Date; Themes = $themes }
        $cacheObj | ConvertTo-Json | Set-Content $PoshBuddy.CacheFile
        return $themes
    } catch { 
        if (Test-Path $PoshBuddy.CacheFile) { 
            try {
                $content = Get-Content $PoshBuddy.CacheFile -ErrorAction SilentlyContinue
                if (![string]::IsNullOrWhiteSpace($content)) {
                    return ($content | ConvertFrom-Json).Themes
                }
            } catch { }
        }
        return @() 
    }
}

function Save-Theme ($themeName, $localPath) {
    if (!(Test-Path $localPath)) {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/$themeName" -OutFile $localPath -ErrorAction SilentlyContinue -TimeoutSec 10
    }
}

function Apply-Theme ($themePath) {
    if (!$themePath -or !(Test-Path $themePath)) { return }
    $line = "oh-my-posh init pwsh --config '$themePath' | Invoke-Expression"
    if (!(Test-Path $PoshBuddy.Profile)) { New-Item -ItemType File -Path $PoshBuddy.Profile -Force | Out-Null }
    
    $content = Get-Content $PoshBuddy.Profile
    $pattern = "^oh-my-posh init .*"
    if ($content -match $pattern) {
        $newContent = $content | ForEach-Object { if ($_ -match $pattern) { $line } else { $_ } }
        $newContent | Set-Content $PoshBuddy.Profile
    } else {
        Add-Content -Path $PoshBuddy.Profile -Value "`n$line"
    }
}

# --- VIEW (UI LAYER) ---
function Draw-Box ($x, $y, $w, $h, $title, $color) {
    if ($w -le 0 -or $h -le 0) { return }
    $pos = $Host.UI.RawUI.CursorPosition
    $pos.X = $x; $pos.Y = $y; $Host.UI.RawUI.CursorPosition = $pos
    Write-Host ("┌─ $title " + ("─" * [Math]::Max(0, ($w - $title.Length - 5))) + "┐") -ForegroundColor $color
    for ($i = 1; $i -lt $h - 1; $i++) {
        $pos.Y = $y + $i; $Host.UI.RawUI.CursorPosition = $pos
        Write-Host "│" -ForegroundColor $color
        $pos.X = $x + $w - 1; $Host.UI.RawUI.CursorPosition = $pos
        Write-Host "│" -ForegroundColor $color
    }
    $pos.Y = $y + $h - 1; $Host.UI.RawUI.CursorPosition = $pos
    Write-Host ("└" + ("─" * [Math]::Max(0, ($w - 2))) + "┘") -ForegroundColor $color
}

function Clear-Panel ($x, $y, $w, $h) {
    for ($i = 0; $i -lt $h; $i++) {
        $p = $Host.UI.RawUI.CursorPosition; $p.X = $x; $p.Y = $y + $i; $Host.UI.RawUI.CursorPosition = $p
        Write-Host (" " * $w)
    }
}

# --- CONTROLLER (LOGIC & LOOP) ---
function Start-PoshBuddy {
    $allThemes = Get-Themes; $index = 0; $filter = ""; $isRunning = $true
    $hasNerdFont = Test-NerdFont
    $lastWin = $Host.UI.RawUI.WindowSize; [Console]::Clear()

    while ($isRunning) {
        $win = $Host.UI.RawUI.WindowSize; $w = $win.Width; $h = $win.Height
        if ($win.Width -ne $lastWin.Width -or $win.Height -ne $lastWin.Height) {
            [Console]::Clear(); $lastWin = $win
        }

        $leftW = [int]($w * 0.30); $rightW = $w - $leftW - 3; $panelH = $h - 5
        $filtered = if ($filter) { $allThemes | Where-Object { $_ -like "*$filter*" } } else { $allThemes }
        $total = $filtered.Count
        if ($index -ge $total) { $index = 0 }

        Draw-Box 0 0 $leftW $panelH "TEMAS v$($PoshBuddy.Version)" "Cyan"
        Draw-Box ($leftW + 1) 0 $rightW $panelH "PREVIEW" "Yellow"

        # View: Themes List
        for ($i = 0; $i -lt ($panelH - 4); $i++) {
            $p = $Host.UI.RawUI.CursorPosition; $p.X = 2; $p.Y = 2 + $i; $Host.UI.RawUI.CursorPosition = $p
            if ($i -lt $total) {
                $isSel = ($i -eq $index)
                $color = if ($isSel) { "Green" } else { "Gray" }
                $text = if ($isSel) { "> " + $filtered[$i] } else { "  " + $filtered[$i] }
                if ($text.Length -gt ($leftW-4)) { $text = $text.Substring(0, $leftW-4) }
                Write-Host ($text.PadRight($leftW-4)) -ForegroundColor $color
            } else { Write-Host (" " * ($leftW-4)) }
        }

        # View: Preview Panel
        if ($total -gt 0) {
            $theme = $filtered[$index]; $localPath = Join-Path $PoshBuddy.ThemesDir $theme
            Save-Theme $theme $localPath

            Clear-Panel ($leftW + 2) 1 ($rightW - 2) ($panelH - 2)
            $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = 4; $Host.UI.RawUI.CursorPosition = $p
            oh-my-posh print primary --config $localPath --shell pwsh | Out-String | Write-Host -NoNewline

            if (!$hasNerdFont) {
                $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = $panelH - 2
                $Host.UI.RawUI.CursorPosition = $p
                Write-Host "⚠️ Nerd Font no detectada. Los iconos podrían no verse bien." -ForegroundColor DarkYellow
            }

            $p.Y = 7; $Host.UI.RawUI.CursorPosition = $p; Write-Host "METRICAS DE CARGA:" -ForegroundColor DarkGray
            $debug = oh-my-posh debug --config $localPath | Out-String
            $p.Y = 9; $Host.UI.RawUI.CursorPosition = $p
            ($debug -split "`n") | Where-Object { $_ -match "Run duration" } | Write-Host -ForegroundColor Cyan
        } else {
            $localPath = $null
            Clear-Panel ($leftW + 2) 1 ($rightW - 2) ($panelH - 2)
            $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = 4; $Host.UI.RawUI.CursorPosition = $p
            Write-Host "No se encontraron temas con el filtro: '$filter'" -ForegroundColor DarkGray
        }

        $keyInfo = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        $vKey = $keyInfo.VirtualKeyCode
        switch ($vKey) {
            33 { $index = [Math]::Max(0, $index - 10) }
            34 { $index = [int][Math]::Min($total - 1, $index + 10) }
            38 { $index = [Math]::Max(0, $index - 1) }
            40 { $index = [int][Math]::Min($total - 1, $index + 1) }
            13 { 
                if ($total -gt 0 -and $null -ne $filtered[$index]) {
                    $p = $Host.UI.RawUI.CursorPosition; $p.X = 0; $p.Y = $panelH + 1; $Host.UI.RawUI.CursorPosition = $p
                    Write-Host "¿Aplicar $($filtered[$index])? (s/n): " -NoNewline
                    if ((Read-Host) -eq "s") {
                        Apply-Theme $localPath
                        Write-Host "¡Exito! Reinicia la terminal." -ForegroundColor Green; $isRunning = $false
                    } else { [Console]::Clear() }
                }
            }
            27 { $isRunning = $false }
            8  { if ($filter.Length -gt 0) { $filter = $filter.Substring(0, $filter.Length - 1); $index = 0 }; [Console]::Clear() }
            default {
                $char = $keyInfo.Character
                if ([char]::IsLetterOrDigit($char)) { $filter += [string]$char; $index = 0; [Console]::Clear() }
            }
        }
    }
}

# START APPLICATION
Start-PoshBuddy
