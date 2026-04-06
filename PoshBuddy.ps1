# PoshBuddy v0.0.1-alpha.1
$PoshBuddy = @{
    Version = "0.0.1-alpha.1"
    ThemesDir = Join-Path $HOME ".poshthemes"
    Config = @{}
}
Write-Host "PoshBuddy Engine Initialized"
function Get-RemoteThemes {
    $url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes"
    return (Invoke-RestMethod -Uri $url | Where-Object { $_.name -like "*.omp.json" }).name
}
function Draw-UIBox ($x, $y, $w, $h, $title, $color) {
    $pos = $Host.UI.RawUI.CursorPosition
    $pos.X = $x; $pos.Y = $y
    $Host.UI.RawUI.CursorPosition = $pos
    Write-Host ("┌─ $title " + ("─" * ($w - $title.Length - 5)) + "┐") -ForegroundColor $color
    for ($i = 1; $i -lt $h - 1; $i++) {
        $pos.Y = $y + $i; $Host.UI.RawUI.CursorPosition = $pos
        Write-Host "│" -ForegroundColor $color
        $pos.X = $x + $w - 1; $Host.UI.RawUI.CursorPosition = $pos
        Write-Host "│" -ForegroundColor $color
    }
    $pos.Y = $y + $h - 1; $Host.UI.RawUI.CursorPosition = $pos
    Write-Host ("└" + ("─" * ($w - 2)) + "┘") -ForegroundColor $color
}
