// ============ modules/mod.rs ============
mod audio;
mod battery;
mod box_module;
mod clock;
mod custom_module;
mod hyprland_workspaces;
mod mpris_module;
mod network;

pub use audio::{AudioConfig, AudioWidget};
pub use battery::{BatteryConfig, BatteryWidget};
pub use box_module::{BoxWidget, BoxWidgetConfig};
pub use clock::ClockWidget;
pub use custom_module::CustomModuleWidget;
pub use hyprland_workspaces::HyprWorkspacesWidget;
pub use mpris_module::{MprisConfig, MprisWidget};
pub use network::{NetworkConfig, NetworkWidget};
