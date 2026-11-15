# Requirements and acceptance ledger

Implementation status for 0.1.0. Evidence and limits are recorded in validation.md.

| Requirement | Status |
| --- | --- |
| Windows and Linux terminal application | Implemented; native Windows and WSL exercised. Native Linux acceptance remains. |
| Midnight/yellow/cyan layout, keyboard, mouse, compact display | Implemented; terminal snapshots and interactive review. Mouse coverage is partial. |
| First-launch Threshold welcome, replay setting, loading state, ASCII door | Implemented. |
| Adapters, activity previews, IP addresses, BPF, optional recording | Implemented; capture and recording exercised on both platforms. |
| Capture, pause/resume, stop confirmation, session reset | Implemented and exercised. |
| Incoming/outgoing graph, time axis, ring, units and drop counts | Implemented; 30-second graph window. |
| Hosts, country/ASN/domain grouping, services, programs | Implemented; inference and attribution have stated limits. |
| Independent filters, sorting and favorites | Implemented. |
| Embedded country/ASN, reverse DNS, custom MMDB | Implemented; embedded lookup tested. Custom data and resolver coverage are partial. |
| Inspection, exact/exclusion filters, details and RTT | Implemented; details and explicit local probe exercised. |
| PCAP/PCAPNG import and PCAP export | Implemented; fixtures, failures and round trip tested. |
| Threshold/favorite/blacklist alerts | Implemented and exercised. History, optional bell/webhook; no native desktop toasts. |
| IP/CIDR blacklist import | Implemented; surveillance does not imply firewall blocking. |
| IP/program blocking and reversal | Implemented with confirmations. Linux IP integration passed. Windows elevated and native Linux program integration remain unvalidated. |
| Themes, custom colors and gradients | Implemented; visual review, not exhaustive acceptance. |
| French and English | Implemented catalogs; some technical diagnostics remain English. Other languages deferred. |
| Login autostart and automatic capture preference | Implemented; isolated registration/removal tested. Real login cycle remains. |
| Git link and credits | Implemented; donation controls removed. |
| Free use, attribution, resale restrictions | Community license draft supplied; legal review remains. |
| Modular English source, packaging, documentation and errors | Implemented; locked builds, local diagnostics and regression scripts included. |

No simulated traffic is used as evidence of live capture. A build, source review or screenshot alone does not establish operating-system integration.
