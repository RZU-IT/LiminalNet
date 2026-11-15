#!/bin/sh
set -eu
executable=$(wslpath -w "$1")
shift
runner=$(wslpath -w "$(dirname "$0")/windows-runner.ps1")
exec powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$runner" "$executable" "$@"
