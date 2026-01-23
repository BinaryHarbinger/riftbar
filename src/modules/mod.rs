// ============ modules/mod.rs ============
mod active_window;
mod audio;
mod battery;
mod box_module;
mod clock;
mod custom_module;
mod hyprland_workspaces;
mod mpris;
mod network;
mod revealer;
mod tray;

pub use active_window::{ActiveWindowConfig, ActiveWindowWidget};
pub use audio::{AudioConfig, AudioWidget};
pub use battery::{BatteryConfig, BatteryWidget};
pub use box_module::{BoxWidget, BoxWidgetConfig};
pub use clock::{ClockConfig, ClockWidget};
pub use custom_module::CustomModuleWidget;
pub use hyprland_workspaces::{HyprWorkspacesWidget, WorkspacesConfig};
pub use mpris::{MprisConfig, MprisWidget};
pub use network::{NetworkConfig, NetworkWidget};
pub use revealer::{RevealerConfig, RevealerWidget};
pub use tray::{TrayConfig, TrayWidget};
