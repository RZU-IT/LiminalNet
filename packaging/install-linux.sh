#!/bin/sh
set -eu
cd "$(dirname "$0")"
prefix="${XDG_DATA_HOME:-$HOME/.local/share}/liminalnet"
bin_dir="${LIMINALNET_BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$prefix" "$bin_dir"
cp liminalnet liminalnet-core "$prefix/"
for document in LICENSE DATA-SOURCES.txt THIRD-PARTY-NOTICES.txt START-HERE.txt usage.md validation.md; do
    if [ -f "$document" ]; then
        cp "$document" "$prefix/"
        chmod 644 "$prefix/$document"
    fi
done
chmod 755 "$prefix/liminalnet" "$prefix/liminalnet-core"
ln -sfn "$prefix/liminalnet" "$bin_dir/liminalnet"
printf 'Installed in %s\nLaunch: %s/liminalnet\n' "$prefix" "$bin_dir"
printf 'libpcap is required. Live capture needs suitable capabilities or root.\n'
printf 'For unprivileged capture, an administrator may run:\n  sudo setcap cap_net_raw,cap_net_admin=eip "%s/liminalnet-core"\n' "$prefix"
printf 'Firewall changes still require administrative privileges.\n'
