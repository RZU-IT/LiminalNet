param(
    [string]$Destination = (Join-Path $env:LOCALAPPDATA 'Programs\LiminalNet'),
    [string]$ShortcutPath = (Join-Path ([Environment]::GetFolderPath('Desktop')) 'LiminalNet.lnk')
)
$ErrorActionPreference = 'Stop'
$source = $PSScriptRoot
foreach ($file in @('liminalnet.exe', 'liminalnet-core.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $source $file))) { throw "Missing $file. Extract the complete archive first." }
}
New-Item -ItemType Directory -Path $Destination -Force | Out-Null
foreach ($file in @('liminalnet.exe', 'liminalnet-core.exe', 'LICENSE', 'DATA-SOURCES.txt', 'THIRD-PARTY-NOTICES.txt', 'START-HERE.txt', 'usage.md', 'validation.md')) {
    if (Test-Path -LiteralPath (Join-Path $source $file)) { Copy-Item -LiteralPath (Join-Path $source $file) -Destination $Destination -Force }
}
$shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut($ShortcutPath)
$shortcut.TargetPath = Join-Path $Destination 'liminalnet.exe'
$shortcut.WorkingDirectory = $Destination
$shortcut.Save()
Write-Output "Installed in $Destination. Desktop shortcut created."
Write-Output 'Npcap must be installed separately from https://npcap.com/. No driver is bundled.'
