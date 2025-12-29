
# Riftbar

[![Build Status](https://img.shields.io/github/actions/workflow/status/BinaryHarbinger/riftbar/ci.yml?branch=main)](https://github.com/BinaryHarbinger/riftbar/actions)

[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)

[![AUR](https://img.shields.io/aur/version/riftbar?label=AUR)](https://aur.archlinux.org/packages/riftbar-stable-git)

### ❗Caution
Riftbar is currently experimental and some key features are missing or WIP.


Riftbar is a **Waybar-like status bar** written in **Rust**, designed to be fast, safe, and modern. It uses **GTK4** for GUI and **Tokio** for asynchronous tasks, making it suitable for Wayland compositors like Sway, Hyprland, and Wayfire.

## Features

- Async updates using Tokio
- Layer-shell support for Wayland
- Modular design for CPU, network, battery, clock, and more
- Lightweight and fast, leveraging Rust’s safety

## TODO

- [X] Add Hyprland workspace integration
- [ ] Calendar sub-widget // Not planned
- [X] Custom style via style.css 
- [X] Support for scss
- [ ] System tray // Work in progress
- [X] Configuration file using TOML
- [X] Improve customization

## Installation

Install trough AUR:

```bash
yay -S riftbar-stable-git
```
OR 
```bash
paru -S riftbar-stable-git
```
OR
```bash
git clone https://aur.archlinux.org/riftbar-stable-git.git # Clone AUR package
cd riftbar-stable-git # Get into directory
makepkg -si # Make the package and install as system package
```

Clone and build from source:

Dependecies are: `gtk4 gtk4-layer-shell wayland`

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

