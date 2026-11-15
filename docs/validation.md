# Validation — 0.1.0 development build

Recorded on 2026-11-12. These results describe this build, not a guarantee of defect-free operation.

## Environments

- Native Windows x86-64 with the existing Npcap runtime. Executables were cross-compiled in WSL and executed on Windows, including real packet capture and an interactive terminal session.
- Ubuntu 24.04.4 under WSL2, kernel 6.18.33.2-microsoft-standard-WSL2. Linux artifacts target Ubuntu 24.04 x86-64; compatibility with other distributions has not been established.
- Rust 1.100.0, locked Cargo dependencies. Linux build cache resides on the WSL filesystem.

## Passed checks

- Formatting and Clippy across all targets with warnings rejected; release builds for Linux and Windows.
- Ten tests on native Windows and ten under privileged WSL: Ethernet/raw IPv4/IPv6 and ARP parsing, truncated input, accounting, process correlation against an actual TCP listener, embedded geographic databases, firewall identifier/input validation, local webhook delivery, and real loopback capture with pause/resume.
- Packaged executables on both platforms: independently generated PCAP and PCAPNG input with known byte/packet totals, BPF filtering, invalid filter errors, corrupt/unsupported capture errors, protection against overwriting recordings, preservation of invalid configuration files.
- Real capture/export/import round trip with six ICMP packets and matching totals.
- Native Windows interactive terminal: welcome, adapter selection, capture, pause, inspection, stop confirmation, return to sources and clean exit.
- WSL pseudo-terminal lifecycle and terminal restoration; imported connection details and an explicit local ping latency probe.
- WSL live alerts: bits/s, bytes/s and packets/s thresholds, favorites and CIDR blacklist matches.
- Autostart registration and removal on Windows and Linux using isolated configuration/startup directories. No user login startup was enabled during these tests.
- Linux IP blocking and removal in an isolated network namespace: baseline ping succeeds, blocked ping fails, restored ping succeeds. No host firewall policy was changed.
- Visual review of rendered terminal cells and native terminal output, including graph background, time labels, panel alignment and ASCII branding.
- Windows executable icon and version metadata; Linux archive permissions and Debian package construction.

## Boundaries and remaining validation

- Windows IP/application firewall integration requires administrator rights. This session was not elevated; actual Windows policy changes remain untested. Input rejection and rule construction checks do not substitute for this integration test.
- This WSL kernel has CONFIG_NFT_SOCKET disabled. Linux application blocking is explicitly unavailable here and was not marked passed. Validate it on native Linux with nftables socket/cgroup v2 support and a dedicated application cgroup. Shared cgroups are rejected. Linux firewall rules are runtime rules, not reboot-persistent policy.
- A full native Linux desktop/login session, distribution compatibility and prolonged high-throughput capture remain external validation tasks. WSL does not establish these results.
- UI localization covers French and English catalogs; some technical diagnostics remain English. Additional languages are not shipped.
- Notifications are in-application history, optional terminal bell and optional webhook. Native desktop notification delivery is not implemented.
- Terminal graph resolution depends on available cells; the history window is 30 seconds. Service names are port-based inference, and socket-to-process correlation can legitimately be unknown or ambiguous.
- Reverse DNS, custom databases, mouse interaction and arbitrary user palettes have not received exhaustive platform-by-platform acceptance testing. No claim of complete Sniffnet feature parity is made.
- The community license is a development draft. Its resale restriction is intentional and means it is not an OSI-approved open-source license. Public release should include legal review.

## Reproduction

Native Windows installation checked again on 2026-11-15: installed launcher/core, screen rendering, experimental notice and GitHub link. Real loopback capture recorded 40 test packets; replay matched packet and byte totals. PCAP/PCAPNG import and error regression checks passed against the installed executable. Isolated settings and synthetic local traffic only; firewall and login-startup state were unchanged.

See tests/ for regression scripts and packaging/ for reproducible build scripts. Capture tests require libpcap/Npcap and suitable capture permissions. Run firewall integration only in the guarded isolated namespace script or on a deliberately prepared test machine. Inspect README.md for commands, platform requirements and operational limits.
