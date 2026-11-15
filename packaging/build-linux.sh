#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --locked
mkdir -p dist/linux
build_dir="${CARGO_TARGET_DIR:-target}"
cp "$build_dir/release/liminalnet" "$build_dir/release/liminalnet-core" dist/linux/
cp LICENSE LICENSES/DATA-SOURCES.txt LICENSES/THIRD-PARTY-NOTICES.txt dist/linux/
cp docs/usage.md docs/validation.md dist/linux/
cp packaging/install-linux.sh dist/linux/install.sh
chmod +x dist/linux/liminalnet dist/linux/liminalnet-core dist/linux/install.sh
python3 packaging/archive-linux.py
