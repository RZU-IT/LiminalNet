param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(ValueFromRemainingArguments=$true)][string[]]$TestArguments
)
$ErrorActionPreference = 'Stop'
$env:PATH = (Join-Path $env:SystemRoot 'System32\Npcap') + ';' + $env:PATH
& $Executable @TestArguments
exit $LASTEXITCODE
