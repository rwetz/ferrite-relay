<p align="center">
  <img src="assets/logo.svg" width="128" height="128" alt="Relay logo">
</p>

<h1 align="center">Relay</h1>

<p align="center">
  Frameless, always-on-top Ferrite desktop panels: clock, CPU load and Wireless's now playing.
</p>

<p align="center">
  <a href="https://github.com/rwetz/ferrite-relay/actions/workflows/ci.yml"><img src="https://github.com/rwetz/ferrite-relay/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/rwetz/ferrite-relay/releases/latest"><img src="https://img.shields.io/github/v/release/rwetz/ferrite-relay?color=F2A93B&labelColor=18181B" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-3D3D42?labelColor=18181B" alt="License"></a>
</p>

![Relay's panel manager listing the clock, CPU and now-playing panels](docs/main.png)

Pin or close panels from Relay, its Ctrl+Shift+P command palette, or each panel's close button. Drag headings to position panels. Pinned state and window positions are saved locally. All open panels follow Relay's selected scheme and appearance. Closing the manager leaves pinned panels running.

Now playing reads Wireless v0.3.3 or newer through %APPDATA%/ferrite/wireless-now-playing.json. A stopped player, unsupported snapshot version, or snapshot older than ten seconds shows inactive, never an old track as live. It integrates with Wireless only.

## Install

Download the Windows x64 zip from Releases, or install Relay through Lodestone. SHA256SUMS.txt accompanies each release. Main pushes automatically produce versioned releases. This initial release targets Windows; CPU sampling and native frameless pinning use Windows APIs.

## Develop

Rust 1.92 or newer: cargo run. Run cargo test and cargo clippy --all-targets. `--smoke` checks CPU sampling; `--widgets` opens all three panels for inspection; `--version` reports the version.

Panel state: %APPDATA%/ferrite/relay-widgets.json. Window positions use the Ferrite window store. Apache-2.0.

## Settings

Open the sliders button in the title bar, press Ctrl+, or choose Settings from the command palette. All ten schemes, Dark/Light/System appearance, motion refresh rate (12 to 240 FPS), and row density apply immediately and persist per app. Save failures appear in Settings. Start with `--settings` to open the drawer directly. Ferrite launcher environment overrides take precedence at startup.

## Keys

| Keys | |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> | Command palette |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Settings |

---

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/brand/heptagram-white.svg">
    <img src="docs/brand/heptagram-black.svg" alt="heptagram" width="48">
  </picture>
</p>
