
# Riftbar

[![Build Status](https://img.shields.io/github/actions/workflow/status/BinaryHarbinger/riftbar/ci.yml?branch=main)](https://github.com/BinaryHarbinger/riftbar/actions)
[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/riftbar)](https://crates.io/crates/riftbar)

Riftbar is a **Waybar-like status bar** written in **Rust**, designed to be fast, safe, and modern. It uses **GTK4** for GUI and **Tokio** for asynchronous tasks, making it suitable for Wayland compositors like Sway, Hyprland, and Wayfire.

## Features

- Async updates using Tokio
- Layer-shell support for Wayland
- Modular design for CPU, network, battery, clock, and more
- Lightweight and fast, leveraging Rust’s safety

## TODO

- [X] Add Hyprland workspace integration
- [ ] Calendar sub-widget
- [X] Custom style via style.css 
- [X] Support for scss
- [ ] System tray
- [ ] Configuration file using TOML

## Installation

Clone and build from source:

```bash
git clone https://github.com/BinaryHarbinger/riftbar.git
cd riftbar
cargo build --release
```

Run the executable:
```bash
./target/release/riftbar
```

(Note: Ensure you are running under a Wayland compositor that supports layer-shell, e.g., Hyprland or Sway.)

## Contributing

Contributions are welcome!
To get started:

```bash
git clone https://github.com/BinaryHarbinger/riftbar.git
cd riftbar
cargo check
```


Please open pull requests against the main branch and follow Rust formatting conventions (cargo fmt).

## License

Licensed under the GPLv3 License.
Copyright © BinaryHarbinger.

