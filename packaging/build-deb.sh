#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
stage=$(mktemp -d /tmp/liminalnet-deb-XXXXXXXX)
case "$stage" in /tmp/liminalnet-deb-*) ;; *) exit 1 ;; esac
trap 'rm -rf -- "$stage"' EXIT
mkdir -p "$stage/DEBIAN" "$stage/usr/lib/liminalnet" "$stage/usr/bin" "$stage/usr/share/applications" "$stage/usr/share/icons/hicolor/scalable/apps" "$stage/usr/share/doc/liminalnet"
cp dist/linux/liminalnet dist/linux/liminalnet-core "$stage/usr/lib/liminalnet/"
ln -sfn ../lib/liminalnet/liminalnet "$stage/usr/bin/liminalnet"
cp assets/liminalnet-symbol.svg "$stage/usr/share/icons/hicolor/scalable/apps/liminalnet.svg"
cp LICENSE LICENSES/DATA-SOURCES.txt LICENSES/THIRD-PARTY-NOTICES.txt "$stage/usr/share/doc/liminalnet/"
cat > "$stage/DEBIAN/control" <<'EOF'
Package: liminalnet
Version: 0.1.0
Section: net
Priority: optional
Architecture: amd64
Maintainer: RZU Informatique
Depends: libc6 (>= 2.39), libpcap0.8t64
Recommends: nftables, iputils-ping
Description: Interactive terminal network monitor
 Sniffnet-inspired terminal interface with traffic statistics and inspection.
 Development build. Capture and firewall operations require appropriate rights.
EOF
cat > "$stage/usr/share/applications/liminalnet.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=LiminalNet
Exec=/usr/bin/liminalnet
Icon=liminalnet
Terminal=true
Categories=Network;Monitor;
EOF
find "$stage" -type d -exec chmod 755 {} +
find "$stage" -type f -exec chmod 644 {} +
chmod 755 "$stage/usr/lib/liminalnet/liminalnet" "$stage/usr/lib/liminalnet/liminalnet-core"
dpkg-deb --root-owner-group --build "$stage" dist/LiminalNet-0.1.0-ubuntu24.04-amd64.deb
