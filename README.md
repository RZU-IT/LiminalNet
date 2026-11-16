<img src="assets/liminalnet-symbol.svg" width="56" alt="LiminalNet door symbol">

# LiminalNet

Network traffic monitor for the terminal. Windows and Linux, built with Rust and Ratatui.

**Experimental.** Platform support and completed checks are documented in the [validation report](docs/validation.md).

[Download Beta 1](https://github.com/RZU-IT/LiminalNet/releases/tag/v0.1.0-beta.1). Windows x86-64 and Ubuntu 24.04 x86-64 packages are available in the release assets.

## Features

- Incoming and outgoing traffic, rates and packet counts.
- Hosts, country/ASN data, inferred services and process attribution.
- Connection inspection, BPF filters, sorting and favorites.
- PCAP/PCAPNG import, PCAP recording and session alerts.
- Themes, custom colors, French and English interface.

Process attribution is best effort. Service names come from port assignments; they do not identify application protocols with certainty.

## Build and run

On Ubuntu 24.04, install Rust, a C compiler, pkg-config and libpcap development files. From the repository root:

```sh
cargo build --release --locked
./target/release/liminalnet
```

The toolchain is pinned in [rust-toolchain.toml](rust-toolchain.toml). No Rust installation is needed to run a compiled package; keep the launcher and `liminalnet-core` together.

| Platform | Capture dependency | Validation |
| --- | --- | --- |
| Windows x86-64 | Npcap, installed separately | Native capture, installation and elevated IP/application blocking tested. |
| Ubuntu 24.04 x86-64 | libpcap | Capture, IP/application blocking and XFCE login autostart tested in an Ubuntu VM. WSL capture also tested. |

Windows builds also need a C linker and the Npcap SDK. The [cross-build script](packaging/build-windows-from-wsl.sh) documents the GNU target setup. Windows binaries are unsigned; Npcap is not bundled.

Capture and firewall operations require suitable permissions. Linux application blocking needs a dedicated cgroup v2 scope and kernel support that is unavailable in the tested WSL environment.

## Usage and development

[Usage](docs/usage.md) covers keyboard controls, recording and firewall behavior. [Architecture](docs/architecture.md) describes the modules. [Validation](docs/validation.md) records passed checks and remaining work.

```sh
cargo test --locked
cargo clippy --all-targets -- -D warnings
```

Live capture tests are ignored unless explicitly enabled with capture permissions. Integration scripts are in [tests/](tests/); packaging scripts are in [packaging/](packaging/).

Recordings are opt-in. Existing output files are not overwritten. Diagnostics stay local. Alert webhooks are opt-in.

## Credits and license

Développé par RZU Informatique.

Conçu, testé et débogué par RZU Informatique avec l'assistance d'OpenAI Codex **(not vibe coded)**.

Highly inspired by [Sniffnet](https://github.com/GyulyVGC/sniffnet), created by Giuliano Bellini and contributors.

The [LiminalNet Community License](LICENSE) is a source-available development draft. Free use and attributed redistribution are permitted, with restrictions on paid distribution. It is not an OSI-approved open-source license.

## Reuse and attribution

If you reuse or adapt LiminalNet, please credit **RZU Informatique** and link to [the original project](https://github.com/RZU-IT/LiminalNet). When redistributing it, retain the license and notices, and identify your changes as required by the license.

Suggested credit: “Based on LiminalNet by RZU Informatique.”

Third-party components retain their own licenses. DB-IP Lite country/ASN data are CC BY 4.0; service mappings come from IANA. Attribution and dependency notices are in [LICENSES/](LICENSES/).
