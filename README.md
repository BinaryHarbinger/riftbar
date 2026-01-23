# Riftbar

[![Build Status](https://img.shields.io/github/actions/workflow/status/BinaryHarbinger/riftbar/ci.yml?branch=main)](https://github.com/BinaryHarbinger/riftbar/actions)  [![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)  [![AUR](https://img.shields.io/aur/version/riftbar-bin?cacheSeconds=0)](https://aur.archlinux.org/packages/riftbar-bin)

Riftbar is a **Waybar-like status bar** writen in **Rust** designed to be fast, safe and modern. It uses **GTK4** for GUI and gtk4-layer-shell protocol making it suitable for Wayland compositors like Sway, Hyprland, and Wayfire.
 
## Features

- Async updates, GUI stays responsive
- Layer-shell support for Wayland
- Modular design for CPU, network, battery, clock, and more
- Lightweight and fast, leveraging Rust’s safety

## Installation

> [!WARNING]
> Compositors without wlr-layer-shell protocol isn't supported. 
> Essipecially Gnome isn't supported because of that.

Currently only packages avaiable on AUR.

You can always find binary (x86_64) files in [releases](https://github.com/BinaryHarbinger/riftbar/releases) page.

## TODO for next release [v0.1.4]

- [ ] Migrate to IPC socket for hyprland workspaces instead of crate.
- [ ] Make tray module function properly.
- [ ] Make a wiki page for every module and choose a proper format for wiki.
- [ ] Improve default configuration.
- [ ] Improve GitHub CI.

## Compiling

Dependecies: `gtk4 gtk4-layer-shell wayland`

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
