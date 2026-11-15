# LiminalNet architecture

Status: implementation in progress. No release has been validated yet.

The application owns the terminal event loop. Capture runs on a worker thread and publishes bounded batches to the application. Packet parsing and accounting do not depend on terminal widgets. DNS, geographical databases, process attribution and firewall operations must not block rendering.

Modules: `capture` owns pcap handles and capture lifecycle; `model` owns packet parsing, flow keys, counters and bounded history; `config` owns versioned user preferences; `ui` renders application state; `app` dispatches input and worker events. Platform-specific features are isolated before introduction.

Session states: welcome, source selection, running, paused, completed and failed. Returning to source selection requires confirmation and discards session statistics only after capture has stopped. Pause must stop packet collection, not just freeze the display. Preview counters are separate from session accounting.

Errors retain context and are visible to the user. Unknown process, missing country data and unresolved DNS must remain explicit. A failed firewall operation must never produce a blocked badge. Terminal restoration is required on exit and panic.

Resource limits: bounded worker channel, bounded flow table and history, bounded notifications. Export files are user-selected and must not silently overwrite existing files. Network-derived text is stripped of control characters before display.

Windows uses Npcap; Linux uses libpcap. Npcap is not bundled without redistribution permission. WSL is a separate test environment, not proof of native Linux firewall or desktop integration behavior.

No external AI service is required at runtime. The Git link is optional release metadata. No payment, telemetry or automatic update installation is included.
