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
