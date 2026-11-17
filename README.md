<img src="assets/liminalnet-symbol.svg" width="56" alt="LiminalNet">

# LiminalNet

Network monitor for the terminal, built with Rust and Ratatui. Windows and Linux. Beta 1.

Traffic graphs, connections, hosts, services and programs, with filters, favorites, PCAP recording and imports. French and English interface, configurable colors and alerts.

## Run

[Download Beta 1](https://github.com/RZU-IT/LiminalNet/releases/tag/v0.1.0-beta.1), extract the full package and launch `liminalnet.exe` on Windows or `./liminalnet` on Linux.

Windows requires [Npcap](https://npcap.com/), installed separately. Linux packages target Ubuntu 24.04 and require libpcap. Capture and firewall operations need suitable permissions.

Tested on one Windows 11 PC, WSL2 and an Ubuntu 24.04 VM. Linux application blocking requires kernel features unavailable in the tested WSL environment. Process attribution is best effort; service names are inferred from ports.

## Build

```sh
cargo build --release --locked
```

[Usage](docs/usage.md) · [Build details](docs/reference.md) · [Architecture](docs/architecture.md) · [Validation](docs/validation.md)

## Credits and license

Développé par RZU Informatique.

Made with care by RZU Informatique, with help from OpenAI Codex. **(not vibe coded)**

Highly inspired by [Sniffnet](https://github.com/GyulyVGC/sniffnet), created by Giuliano Bellini and contributors.

[Community License](LICENSE), development draft: free use and attributed redistribution, with restrictions on paid distribution. This is a source-available license, not an OSI-approved open-source license. Please credit RZU Informatique and link to this project when reusing it. Retain the license and notices when redistributing.

[Third-party notices](LICENSES/), including DB-IP Lite and IANA data.
