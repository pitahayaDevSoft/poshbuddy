<#
.SYNOPSIS
    PoshBuddy - The Ultimate Windows Terminal Experience Manager.
.DESCRIPTION
    A professional TUI for managing Oh My Posh themes and PowerShell modules.
    Currently in Pre-Alpha.
#>
$PoshBuddy = @{
    Version = "0.0.1-alpha.11"
    ThemesDir = Join-Path $HOME ".poshthemes"
    Profile = if ($PROFILE) { $PROFILE } else { Join-Path $HOME "Documents\PowerShell\Microsoft.PowerShell_profile.ps1" }
}

if (!(Get-Command oh-my-posh -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: 'oh-my-posh' no detectado. Instálalo primero." -ForegroundColor Red
    exit
}

if (!(Test-Path $PoshBuddy.ThemesDir)) { New-Item -ItemType Directory -Path $PoshBuddy.ThemesDir -Force }

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

function Get-Themes {
    $url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes"
    try { return (Invoke-RestMethod -Uri $url | Where-Object { $_.name -like "*.omp.json" }).name }
    catch { return @() }
}

$allThemes = Get-Themes; $index = 0; $filter = ""; $isRunning = $true
$lastWin = $Host.UI.RawUI.WindowSize
[Console]::Clear()

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

    # Listado
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

    # Preview
    if ($total -gt 0) {
        $theme = $filtered[$index]; $localPath = Join-Path $PoshBuddy.ThemesDir $theme
        if (!(Test-Path $localPath)) { 
            $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = 2; $Host.UI.RawUI.CursorPosition = $p
            Write-Host "Descargando..." -ForegroundColor Cyan
            Invoke-WebRequest -Uri "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/$theme" -OutFile $localPath -ErrorAction SilentlyContinue 
        }
        
        # Limpieza y Render del Prompt
        for ($j = 2; $j -lt $panelH - 1; $j++) {
            $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 3; $p.Y = $j; $Host.UI.RawUI.CursorPosition = $p
            Write-Host (" " * [Math]::Max(0, ($rightW - 4)))
        }
        $p = $Host.UI.RawUI.CursorPosition; $p.X = $leftW + 4; $p.Y = 4; $Host.UI.RawUI.CursorPosition = $p
        oh-my-posh print primary --config $localPath --shell pwsh | Out-String | Write-Host -NoNewline
        
        $p.Y = 7; $Host.UI.RawUI.CursorPosition = $p; Write-Host "METRICAS DE CARGA:" -ForegroundColor DarkGray
        $debug = oh-my-posh debug --config $localPath | Out-String
        $p.Y = 9; $Host.UI.RawUI.CursorPosition = $p
        ($debug -split "`n") | Where-Object { $_ -match "Run duration" } | Write-Host -ForegroundColor Cyan
    }

    $keyInfo = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    $vKey = $keyInfo.VirtualKeyCode
    switch ($vKey) {
        33 { $index = [Math]::Max(0, $index - 10) }
        34 { $index = [int][Math]::Min($total - 1, $index + 10) }
        38 { $index = [Math]::Max(0, $index - 1) }
        40 { $index = [int][Math]::Min($total - 1, $index + 1) }
        13 { 
            $p = $Host.UI.RawUI.CursorPosition; $p.X = 0; $p.Y = $panelH + 1; $Host.UI.RawUI.CursorPosition = $p
            Write-Host "¿Aplicar $($filtered[$index])? (s/n): " -NoNewline
            if ((Read-Host) -eq "s") {
                $line = "oh-my-posh init pwsh --config '$localPath' | Invoke-Expression"
                if (!(Test-Path $PoshBuddy.Profile)) { 
                    New-Item -ItemType File -Path $PoshBuddy.Profile -Force | Out-Null 
                }
                $content = Get-Content $PoshBuddy.Profile
                if ($content -match "oh-my-posh init") {
                    $newContent = $content | ForEach-Object { if ($_ -match "oh-my-posh init") { $line } else { $_ } }
                    $newContent | Set-Content $PoshBuddy.Profile
                } else {
                    Add-Content -Path $PoshBuddy.Profile -Value "`n$line"
                }
                Write-Host "¡Exito! Reinicia la terminal." -ForegroundColor Green; $isRunning = $false
            } else { [Console]::Clear() }
        }
        27 { $isRunning = $false }
        8  { if ($filter.Length -gt 0) { $filter = $filter.Substring(0, $filter.Length - 1); $index = 0 }; [Console]::Clear() }
        default {
            $char = $keyInfo.Character
            if ([char]::IsLetterOrDigit($char)) { $filter += [string]$char; $index = 0; [Console]::Clear() }
        }
    }
}

