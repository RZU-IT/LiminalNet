# LiminalNet

Interactive terminal network monitor for Windows and Linux. Developed by RZU Informatique with OpenAI Codex assistance. Highly inspired by Sniffnet, created by Giuliano Bellini and contributors.

**Experimental.** Development build; not ready for production use. Platform limits and test results: [validation report](docs/validation.md).

![LiminalNet door symbol](assets/liminalnet-symbol.svg)

LiminalNet shows live incoming and outgoing traffic, connections, hosts, inferred services and attributed programs. It includes BPF filters, favorites, PCAP/PCAPNG import, PCAP recording, alert history and configurable terminal colors. The interface supports French and English.

## Build from source

This repository contains the source code. Compiled archives are not part of the Git history.

On Ubuntu 24.04, install Rust, a C compiler, pkg-config and libpcap development files. Then run:

```sh
cargo build --release --locked
./target/release/liminalnet
```

Cargo uses the toolchain specified in `rust-toolchain.toml`. Live capture needs suitable permissions. Windows builds also need a C linker and the Npcap SDK; see `packaging/build-windows-from-wsl.sh` for the cross-compilation setup. Npcap itself must be installed separately to run the Windows application.

## Launch

Extract the complete platform archive. Launch `liminalnet.exe` on Windows, or `./liminalnet` on Linux. Keep the launcher and `liminalnet-core` together. The launcher checks Npcap on Windows and passes arguments to the engine. No Rust installation is required to use a packaged build.

Windows requires Npcap, installed separately from its official website. Linux requires libpcap (Ubuntu package `libpcap0.8t64`). Capture permissions and firewall permissions are separate. Linux capture can be granted the capabilities printed by the optional installer; changing firewall rules still requires administrative privileges.

The Linux build targets Ubuntu 24.04 x86_64 and compatible systems. WSL testing is not native desktop-Linux validation. Windows binaries are unsigned development builds.

## Controls

- Sources: arrows select; Enter starts; `f` edits BPF; `i` selects a capture file; `e` sets a new PCAP recording path; `r` refreshes adapters. Empty file/export inputs disable that option.
- Session: `1` overview, `2` inspect, `3` notifications, `4` firewall; Space pauses/resumes; `u` changes units; `m` toggles compact mode.
- Overview: Tab selects a panel; arrows select entries; `*` changes a favorite; `f` filters favorites; `t` changes sort; `g` groups hosts by IP/country/ASN/domain; Enter inspects the selected item.
- Inspect: `/` searches IPs, ports, protocols, services, programs and resolved host metadata. Prefix `=` matches a whole field, `!` excludes a substring and `!=` excludes a whole-field match. Enter shows details. `b` proposes blocking the remote IP; `B` proposes blocking the identified executable.
- Firewall: arrows select a recorded rule; `d` proposes removing it. Operations require explicit confirmation. Recorded rules are not a continuous guarantee of OS policy state.
- `s` opens settings, `a` opens About, `?` opens help, Escape returns or requests session termination. Enter cancels the termination dialog; `y` confirms it. `q` quits from sources.

Capture-file direction uses addresses of the selected reference adapter. If those addresses do not match the original capturing machine, direction totals are not meaningful. Process attribution is live-only, best effort, and explicitly unknown when not identified. Port-based service inference does not prove a protocol or malware identity.

## Firewall behavior

Windows rules target remote IP/CIDR ranges or executable paths in both directions and remain in the system until removed. Linux IP rules use dedicated nftables tables in both directions and do not persist across reboot. Linux executable rules require a running process in a dedicated cgroup v2 scope; shared cgroups are rejected. These rules apply to that scope's outbound traffic and must be reapplied after the application restarts. Kernel support and permissions are checked by nftables before applying a transaction.

Run in an elevated terminal when deliberately changing firewall rules. No automatic privilege escalation is implemented. Only LiminalNet-owned rule identifiers are accepted for removal. Do not assume imported blacklists block anything: they only drive surveillance alerts unless a separate real rule is installed.

## Diagnostics and development

`--help`, `--version`, `--list-interfaces`, `--config-path`, `--diagnose`, `--reset-settings`, `--read`, `--filter`, `--record`, `--interface`, and `--capture-seconds` are available. Reset preserves a backup of an existing configuration. Capture recordings are opt-in and existing output files are refused. Logs rotate locally and are never uploaded automatically.

Build: install the pinned Rust toolchain, C linker and pcap development files, then `cargo build --release --locked`. For Windows cross-compilation, set `PCAP_LIBDIR` to the Npcap SDK x64 libraries and configure the GNU Windows linker. Do not redistribute the Npcap driver without appropriate permission.

Run `cargo test` and `cargo clippy --all-targets -- -D warnings`. Live capture tests are explicitly ignored unless invoked with suitable privileges. `tests/capture_regression.py`, `tests/terminal_smoke.py`, and `tests/firewall_namespace.py` cover additional integration cases; the firewall script refuses the host network namespace.

## Credits and license

Développé par RZU Informatique.
Conçu, testé et débogué par RZU Informatique avec l'assistance d'OpenAI Codex **(not vibe coded)**.
Highly inspired by Sniffnet, created by Giuliano Bellini and contributors.

The original code uses the draft LiminalNet Community License. It permits free use, modification and attributed redistribution, with restrictions on paid distribution. It is not an OSI-approved open-source license. Third-party components retain their own licenses. DB-IP Lite country/ASN data are CC BY 4.0; port mappings come from IANA. See `LICENSES/`.

The Git button opens the project repository when configured. No donation button or RZU website URL is included.
