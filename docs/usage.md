# Usage

Launch `liminalnet.exe` on Windows or `./liminalnet` on Linux. Keep the launcher and core executable in the same folder. On first launch, acknowledge Welcome to the Threshold, then choose a network adapter or capture file.

## Sources

Arrows select an adapter; Enter starts. `f` edits the BPF filter, `i` selects an input capture, `e` sets a new PCAP recording path, and `r` refreshes adapters. Clear a file or recording input to disable it.

## Session

| Key | Action |
| --- | --- |
| `1`, `2`, `3`, `4` | Overview, connections, notifications, firewall |
| Space | Pause or resume collection |
| `u` | Change bits/bytes/packets display |
| `m` | Toggle compact layout |
| Tab | Select an overview panel |
| Arrows | Select an entry |
| `*` | Toggle a favorite |
| `f` | Filter favorites |
| `t` | Change sort order |
| `g` | Group hosts by IP, country, ASN or domain |
| Enter | Inspect an item or open connection details |
| `/` | Search connections |
| `s`, `a`, `?` | Settings, About, help |
| Escape | Return or request session termination |
| `q` | Quit from sources |

In connection details, `p` sends an explicit ping probe. In inspection, `l` filters blacklist matches. Search prefixes: `=` for a whole-field match, `!` to exclude a substring, `!=` to exclude a whole-field match.

Pause stops collection. Returning to sources ends the session after confirmation. Enter cancels the confirmation; `y` confirms it. Existing recordings remain on disk.

## Capture files

PCAP and PCAPNG input are supported. Recording writes PCAP to a new file; existing files are refused.

Direction uses the selected reference adapter's addresses. If they differ from the original capturing machine, imported incoming/outgoing totals are not meaningful. Process attribution is live-only and can be unknown or ambiguous.

## Firewall

In inspection, `b` proposes blocking the remote IP; `B` proposes blocking the identified executable. In the firewall view, `d` proposes removing a recorded rule. Each operation requires confirmation and administrative privileges.

Windows rules apply inbound and outbound and remain until removed. Linux IP rules apply in both directions and do not persist across reboot. Linux application rules apply outbound to a dedicated cgroup v2 scope; shared groups are rejected. Reapply them after the application restarts. Kernel support is required.

Recorded rules are not a continuous check of OS policy. Imported blacklists drive alerts; they do not install firewall rules.

## Diagnostics

`--help`, `--version`, `--list-interfaces`, `--config-path`, `--diagnose`, `--read`, `--filter`, `--record`, `--interface` and `--capture-seconds` are available. `--reset-settings` preserves a backup of the existing configuration.

Logs rotate locally. Threshold, favorite and blacklist events appear in the notification history; a terminal bell and webhook are optional.

See the [validation report](validation.md) for tested behavior and platform limits.
