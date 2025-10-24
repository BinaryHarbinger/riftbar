// ============ modules/mod.rs ============
mod clock;
mod custom_module;
mod hyprland_workspaces;
mod mpris_module;
mod network;

pub use clock::ClockWidget;
pub use custom_module::CustomModuleWidget;
pub use hyprland_workspaces::HyprWorkspacesWidget;
pub use mpris_module::MprisWidget;
pub use network::NetworkWidget;
