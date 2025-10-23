
# Riftbar

[![Build Status](https://img.shields.io/github/actions/workflow/status/BinaryHarbinger/riftbar/rust.yml?branch=main)](https://github.com/BinaryHarbinger/riftbar/actions)
[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/riftbar)](https://crates.io/crates/riftbar)

Riftbar is a **Waybar-like status bar** written in **Rust**, designed to be fast, safe, and modern. It uses **GTK4** for GUI and **Tokio** for asynchronous tasks, making it suitable for Wayland compositors like Sway, Hyprland, and Wayfire.

## Features

- Async updates using Tokio
- Layer-shell support for Wayland
- Modular design for CPU, network, battery, clock, and more
- Lightweight and fast, leveraging Rust’s safety

## TODO

- [ ] Add Hyprland workspace integration
- [ ] Calendar sub-widget
- [ ] Custom style via style.css / SCSS
- [ ] Tray for system resources
- [ ] Configuration file using TOML

## Installation

Clone the repository:

```bash
git clone https://github.com/BinaryHarbinger/riftbar.git
cd riftbar

