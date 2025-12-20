// ============ config.rs ============
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub bar: BarConfig,

    #[serde(default = "default_modules_left")]
    pub modules_left: Vec<String>,

    #[serde(default = "default_modules_center")]
    pub modules_center: Vec<String>,

    #[serde(default = "default_modules_right")]
    pub modules_right: Vec<String>,

    #[serde(default)]
    pub custom_modules: std::collections::HashMap<String, CustomModule>,

    #[serde(default)]
    pub workspaces: WorkspacesConfig,

    #[serde(default)]
    pub network: NetworkConfig,

    #[serde(default)]
    pub mpris: MprisConfig,

    #[serde(default)]
    pub battery: BatteryConfig,

    #[serde(default)]
    pub audio: AudioConfig,

    #[serde(default)]
    pub clock: ClockConfig,

    /*#[serde(default)]
    pub tray: TrayConfig,*/
    #[serde(default)]
    pub boxes: std::collections::HashMap<String, BoxConfig>,

    #[serde(default)]
    pub revealers: std::collections::HashMap<String, RevealerConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BarConfig {
    #[serde(default = "default_height")]
    pub height: u32,

    #[serde(default = "default_position")]
    pub position: String,

    #[serde(default = "default_layer")]
    pub layer: String,

    #[serde(default = "default_spacing")]
    pub spacing: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomModule {
    pub exec: String,

    #[serde(default = "default_action")]
    pub action: String,

    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub format: Option<String>,

    #[serde(default)]
    pub tooltip: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspacesConfig {
    #[serde(default = "WorkspacesConfig::default_workspaces_count")]
    pub min_workspace_count: i32,

    #[serde(default = "WorkspacesConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default)]
    pub tooltip_format: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_action")]
    pub action: String,

    #[serde(default = "NetworkConfig::default_format")]
    pub format: String,

    #[serde(default = "NetworkConfig::default_format_disconnected")]
    pub format_disconnected: String,

    #[serde(default = "NetworkConfig::default_format_ethernet")]
    pub format_ethernet: String,

    #[serde(default = "NetworkConfig::default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub interface: Option<String>,

    #[serde(default = "NetworkConfig::default_tooltip")]
    pub tooltip: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MprisConfig {
    #[serde(default = "MprisConfig::default_format_mpris")]
    pub format: String,

    #[serde(default = "MprisConfig::default_format_mpris")]
    pub format_playing: String,

    #[serde(default = "MprisConfig::default_format_mpris")]
    pub format_paused: String,

    #[serde(default = "MprisConfig::default_format_stopped")]
    pub format_stopped: String,
    
    #[serde(default = "MprisConfig::default_format_nothing")]
    pub format_nothing: String,



    #[serde(default = "MprisConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "MprisConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "MprisConfig::default_tooltip_format")]
    pub tooltip_format: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatteryConfig {
    #[serde(default = "BatteryConfig::default_format")]
    pub format: String,

    #[serde(default = "BatteryConfig::default_format_charging")]
    pub format_charging: String,

    #[serde(default = "BatteryConfig::default_format_full")]
    pub format_full: String,

    #[serde(default = "BatteryConfig::default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub battery: Option<String>,

    #[serde(default = "BatteryConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "default_action")]
    pub action: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AudioConfig {
    #[serde(default = "AudioConfig::default_format")]
    pub format: String,

    #[serde(default = "AudioConfig::default_format_muted")]
    pub format_muted: String,

    #[serde(default = "AudioConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "AudioConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "AudioConfig::default_action")]
    pub action: String,

    #[serde(default = "AudioConfig::default_on_scroll_up")]
    pub on_scroll_up: String,

    #[serde(default = "AudioConfig::default_on_scroll_down")]
    pub on_scroll_down: String,

    #[serde(default = "AudioConfig::default_scroll_step")]
    pub scroll_step: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClockConfig {
    #[serde(default = "ClockConfig::default_format")]
    pub format: String,

    #[serde(default = "ClockConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "ClockConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "ClockConfig::default_tooltip_format")]
    pub tooltip_format: String,

    #[serde(default = "ClockConfig::default_action")]
    pub action: String,
}

/*#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrayConfig {
    #[serde(default = "ClockConfig::default_format")]
    pub format: String,

    #[serde(default = "ClockConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "ClockConfig::default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "ClockConfig::default_tooltip_format")]
    pub tooltip_format: String,

    #[serde(default = "ClockConfig::default_action")]
    pub action: String,
}*/

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoxConfig {
    #[serde(default)]
    pub modules: Vec<String>,

    #[serde(default = "default_action")]
    pub action: String,

    #[serde(default = "default_spacing")]
    pub spacing: i32,

    #[serde(default)]
    pub orientation: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevealerConfig {
    #[serde(default)]
    pub modules: Vec<String>,

    #[serde(default = "default_spacing")]
    pub spacing: i32,

    #[serde(default)]
    pub orientation: Option<String>,

    #[serde(default)]
    pub trigger: Option<String>,

    #[serde(default)]
    pub transition: Option<String>,

    #[serde(default)]
    pub transition_duration: Option<u32>,

    #[serde(default)]
    pub reveal_on_hover: Option<bool>,
}

fn default_height() -> u32 {
    30
}
fn default_position() -> String {
    "top".to_string()
}
fn default_layer() -> String {
    "top".to_string()
}
fn default_spacing() -> i32 {
    10
}
fn default_interval() -> u64 {
    1
}

fn default_action() -> String {
    ":".to_string()
}

fn default_modules_left() -> Vec<String> {
    vec!["mpris".to_string()]
}

fn default_modules_center() -> Vec<String> {
    vec!["hyprland/workspaces".to_string()]
}

fn default_modules_right() -> Vec<String> {
    vec![
        "network".to_string(),
        "audio".to_string(),
        "battery".to_string(),
        "clock".to_string(),
    ]
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: default_height(),
            position: default_position(),
            layer: default_layer(),
            spacing: default_spacing(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bar: BarConfig::default(),
            modules_left: default_modules_left(),
            modules_center: default_modules_center(),
            modules_right: default_modules_right(),
            custom_modules: std::collections::HashMap::new(),
            workspaces: WorkspacesConfig::default(),
            network: NetworkConfig::default(),
            mpris: MprisConfig::default(),
            battery: BatteryConfig::default(),
            audio: AudioConfig::default(),
            clock: ClockConfig::default(),
            //tray: TrayConfig::default(),
            boxes: std::collections::HashMap::new(),
            revealers: std::collections::HashMap::new(),
        }
    }
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            min_workspace_count: Self::default_workspaces_count(),
            tooltip: Self::default_tooltip(),
            tooltip_format: Self::default_tooltip_format(),
        }
    }
}

impl WorkspacesConfig {
    fn default_workspaces_count() -> i32 {
        4
    }

    fn default_tooltip() -> bool {
        true
    }

    fn default_tooltip_format() -> String {
        "Workspaces".to_string()
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            action: default_action(),
            format: Self::default_format(),
            format_disconnected: Self::default_format_disconnected(),
            format_ethernet: Self::default_format_ethernet(),
            interval: Self::default_interval(),
            interface: None,
            tooltip: Self::default_tooltip(),
        }
    }
}

impl NetworkConfig {
    fn default_format() -> String {
        "{icon} {essid}".to_string()
    }

    fn default_format_disconnected() -> String {
        "󰖪 Disconnected".to_string()
    }

    fn default_format_ethernet() -> String {
        "󰈀 {ifname}".to_string()
    }

    fn default_interval() -> u64 {
        5
    }

    fn default_tooltip() -> bool {
        true
    }
}

impl Default for MprisConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format_mpris(),
            format_playing: Self::default_format_mpris(),
            format_paused: Self::default_format_mpris(),
            format_stopped: Self::default_format_stopped(),
            format_nothing: Self::default_format_nothing(),
            interval: Self::default_interval(),
            tooltip: Self::default_tooltip(),
            tooltip_format: Self::default_tooltip_format(),
        }
    }
}

impl MprisConfig {
    fn default_format_mpris() -> String {
        "{icon} {artist} - {title}".to_string()
    }

    fn default_format_stopped() -> String {
        "{icon} Stopped".to_string()
    }
    
    fn default_format_nothing() -> String {
        "No Media".to_string()
    }

    fn default_interval() -> u64 {
        100
    }

    fn default_tooltip() -> bool {
        true
    }

    fn default_tooltip_format() -> String {
        "{artist}\n{album}\n{title}".to_string()
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            format_charging: Self::default_format_charging(),
            format_full: Self::default_format_full(),
            interval: Self::default_interval(),
            battery: None,
            tooltip: Self::default_tooltip(),
            action: default_action(),
        }
    }
}

impl BatteryConfig {
    fn default_format() -> String {
        "{icon} {capacity}%".to_string()
    }

    fn default_format_charging() -> String {
        "{icon} {capacity}%".to_string()
    }

    fn default_format_full() -> String {
        "{icon} Full".to_string()
    }

    fn default_interval() -> u64 {
        30
    }

    fn default_tooltip() -> bool {
        true
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            format_muted: Self::default_format_muted(),
            interval: Self::default_interval(),
            tooltip: Self::default_tooltip(),
            action: Self::default_action(),
            on_scroll_up: Self::default_on_scroll_up(),
            on_scroll_down: Self::default_on_scroll_down(),
            scroll_step: Self::default_scroll_step(),
        }
    }
}

impl AudioConfig {
    fn default_format() -> String {
        "{icon} {volume}%".to_string()
    }

    fn default_format_muted() -> String {
        "{icon} Muted".to_string()
    }

    fn default_interval() -> u64 {
        100
    }

    fn default_tooltip() -> bool {
        true
    }

    fn default_action() -> String {
        String::new()
    }

    fn default_on_scroll_up() -> String {
        String::new()
    }

    fn default_on_scroll_down() -> String {
        String::new()
    }

    fn default_scroll_step() -> i32 {
        5
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            interval: Self::default_interval(),
            tooltip: Self::default_tooltip(),
            tooltip_format: Self::default_tooltip_format(),
            action: Self::default_action(),
        }
    }
}

impl ClockConfig {
    fn default_format() -> String {
        "%H:%M".to_string()
    }

    fn default_interval() -> u64 {
        1
    }

    fn default_tooltip() -> bool {
        true
    }

    fn default_tooltip_format() -> String {
        "%A, %B %d, %Y".to_string()
    }

    fn default_action() -> String {
        String::new()
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(config) => {
                        println!("Loaded config from: {:?}", config_path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse config: {}. Using defaults.", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read config: {}. Using defaults.", e);
                }
            }
        } else {
            println!("Config file not found at: {:?}", config_path);
            println!("Using default configuration.");
            println!("To customize, create a config file at: {:?}", config_path);
            println!(
                "Example config will be created automatically on next run if the directory exists."
            );

            let _ = Self::create_example_config(&config_path);
        }

        Self::default()
    }

    fn get_config_path() -> PathBuf {
        let mut path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
        path.push(".config/riftbar/config.toml");
        path
    }

    fn create_example_config(path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let example = r#"# Riftbar Configuration

# Module positions (MUST be at root level, BEFORE any [sections])
modules_left = ["mpris", "custom/arch"]
modules_center = ["hyprland/workspaces"]
modules_right = ["revealer/quicksettings", "clock"]

[bar]
height = 30
position = "top"  # top, bottom
layer = "top"     # background, bottom, top, overlay
spacing = 10

# Clock module configuration
[clock]
format = "%H:%M"
interval = 1  # seconds
tooltip = true
tooltip_format = "%A, %B %d, %Y"
action = ""  # Optional: command to run on click

# Available format placeholders (uses date command format):
# %H - Hour (00-23)
# %M - Minute (00-59)
# %S - Second (00-59)
# %A - Full weekday name
# %a - Abbreviated weekday name
# %B - Full month name
# %b - Abbreviated month name
# %d - Day of month (01-31)
# %Y - Year with century

# Network module configuration
[network]
format = "{icon} {essid}"
format_disconnected = "󰖪 Disconnected"
format_ethernet = "󰈀 {ifname}"
interval = 5
# interface = "wlan0"  # Optional: specify interface
tooltip = true

# Available format placeholders for network:
# {icon} - Dynamic icon based on signal strength
# {essid} - WiFi network name
# {signalStrength} - Signal strength (0-100)
# {signalStrengthApp} - Signal strength with % symbol
# {ifname} - Interface name
# {ipaddr} - IP address

# Audio module configuration
[audio]
format = "{icon} {volume}%"
format_muted = "{icon} Muted"
interval = 100  # milliseconds
tooltip = true
# Custom actions (leave empty for default behavior)
action = ""  # Default: toggle mute
on_scroll_up = ""  # Default: increase volume by scroll_step
on_scroll_down = ""  # Default: decrease volume by scroll_step
scroll_step = 5  # Volume change step (percentage)

# Available format placeholders for audio:
# {icon} - Dynamic icon based on volume level
# {volume} - Volume percentage (0-100)

# MPRIS (Media Player) configuration
[mpris]
format = "{icon} {artist} - {title}"
format_paused = "{icon} {artist} - {title}"
format_stopped = "{icon} Stopped"
interval = 100  # milliseconds
tooltip = true
tooltip_format = "{artist}\n{album}\n{title}"

# Available format placeholders for mpris:
# {icon} - Dynamic icon based on playback state
# {artist} - Artist name
# {title} - Song title
# {album} - Album name
# {status} - Playback status (Playing, Paused, Stopped)

# Battery configuration
[battery]
format = "{icon} {capacity}%"
format_charging = "{icon} {capacity}%"
format_full = "{icon} Full"
interval = 30
# battery = "BAT0"  # Optional: specify battery (default: auto-detect)
tooltip = true

# Available format placeholders for battery:
# {icon} - Dynamic icon based on capacity and status
# {capacity} - Battery percentage
# {status} - Battery status (Charging, Discharging, Full, etc.)
# {time} - Time remaining/until full

# Box widgets - simple containers
[boxes.quickcenter]
modules = ["network", "audio", "battery"]
spacing = 5
# orientation = "horizontal"  # horizontal or vertical (default: horizontal)

# Revealer widgets - containers that reveal on hover or click
[revealers.quicksettings]
modules = ["network", "audio", "battery"]
spacing = 5
trigger = "󰣇"  # Text/icon for the trigger button
transition = "slide_left"  # slide_left, slide_right, slide_up, slide_down, crossfade
transition_duration = 200  # milliseconds
reveal_on_hover = true  # Reveal on hover instead of click
# orientation = "horizontal"  # horizontal or vertical (default: horizontal)

# Custom modules
[custom_modules.arch]
action = ""
exec = "echo ''"
interval = 999999
format = "{}"

[custom_modules.weather]
exec = "curl -s 'wttr.in/?format=%t'"
interval = 600
format = "🌡️ {}"

[custom_modules.uptime]
exec = "uptime -p | sed 's/up //'"
interval = 60
format = "⏱️ {}""#;

        fs::write(path, example)?;
        println!("Created example config at: {:?}", path);
        Ok(())
    }
}
