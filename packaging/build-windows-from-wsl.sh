#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export PCAP_LIBDIR="${PCAP_LIBDIR:-$(pwd)/target/npcap-sdk/Lib/x64}"
cargo build --release --locked --target x86_64-pc-windows-gnu
build_dir="${CARGO_TARGET_DIR:-target}"
mkdir -p dist/windows
cp "$build_dir/x86_64-pc-windows-gnu/release/liminalnet.exe" "$build_dir/x86_64-pc-windows-gnu/release/liminalnet-core.exe" dist/windows/
cp LICENSE LICENSES/DATA-SOURCES.txt LICENSES/THIRD-PARTY-NOTICES.txt dist/windows/
cp docs/usage.md docs/validation.md dist/windows/
cp packaging/install-windows.ps1 dist/windows/install.ps1
