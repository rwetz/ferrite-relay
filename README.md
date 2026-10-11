# Relay

![Relay icon](assets/logo.svg)

Frameless, always-on-top Ferrite desktop panels: a local clock, live system CPU load with a one-minute chart, and Wireless's current station and track.

Pin or close panels from Relay, its Ctrl+Shift+P command palette, or each panel's close button. Drag headings to position panels. Pinned state and window positions are saved locally. All open panels follow Relay's selected scheme and appearance. Closing the manager leaves pinned panels running.

Now playing reads Wireless v0.3.3 or newer through %APPDATA%/ferrite/wireless-now-playing.json. A stopped player, unsupported snapshot version, or snapshot older than ten seconds shows inactive, never an old track as live. It integrates with Wireless only.

## Install

Download the Windows x64 zip from Releases, or install Relay through Lodestone. SHA256SUMS.txt accompanies each release. Main pushes automatically produce versioned releases. This initial release targets Windows; CPU sampling and native frameless pinning use Windows APIs.

## Develop

Rust 1.92 or newer: cargo run. Run cargo test and cargo clippy --all-targets. `--smoke` checks CPU sampling; `--widgets` opens all three panels for inspection; `--version` reports the version.

Panel state: %APPDATA%/ferrite/relay-widgets.json. Window positions use the Ferrite window store. Apache-2.0.
