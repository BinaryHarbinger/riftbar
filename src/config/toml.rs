// ============ config/toml.rs ============
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub bar: BarConfig,

    #[serde(default)]
    pub modules_left: Vec<String>,

    #[serde(default)]
    pub modules_center: Vec<String>,

    #[serde(default)]
    pub modules_right: Vec<String>,

    #[serde(default)]
    pub custom_modules: std::collections::HashMap<String, CustomModule>,

    #[serde(default)]
    pub workspaces: WorkspacesConfig,

    #[serde(default)]
    pub active_window: ActiveWindowConfig,

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

    #[serde(default)]
    pub tray: TrayConfig,

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

    #[serde(default = "default_command")]
    pub on_click: String,

    #[serde(default = "default_command")]
    pub on_click_right: String,

    #[serde(default = "default_command")]
    pub on_click_middle: String,

    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub format: Option<String>,

    #[serde(default)]
    pub tooltip: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspacesConfig {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub icons: Option<HashMap<String, String>>,
    #[serde(default = "WorkspacesConfig::default_workspaces_count")]
    pub min_workspace_count: i32,
    #[serde(default)]
    pub workspace_formating: Option<HashMap<u32, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_command")]
    pub on_click: String,

    #[serde(default = "NetworkConfig::default_format")]
    pub format: String,

    #[serde(default = "NetworkConfig::default_active_icons")]
    pub active_icons: Vec<String>,

    #[serde(default)]
    pub ethernet_icon: Option<String>,

    #[serde(default)]
    pub disconnected_icon: Option<String>,

    #[serde(default = "NetworkConfig::default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub interface: Option<String>,

    #[serde(default = "default_tooltip")]
    pub tooltip: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActiveWindowConfig {
    #[serde(default)]
    pub format: Option<String>,

    #[serde(default = "default_length")]
    pub length_lim: u64,

    #[serde(default = "default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "default_on_click")]
    pub on_click: String,

    #[serde(default)]
    pub no_window_format: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MprisConfig {
    #[serde(default = "MprisConfig::default_format")]
    pub format: String,

    #[serde(default)]
    pub format_playing: Option<String>,

    #[serde(default)]
    pub format_paused: Option<String>,

    #[serde(default)]
    pub format_stopped: Option<String>,

    #[serde(default = "MprisConfig::default_format_nothing")]
    pub format_nothing: String,

    #[serde(default = "default_length")]
    pub length_lim: u64,

    #[serde(default = "MprisConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "default_tooltip")]
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

    #[serde(default = "BatteryConfig::default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub battery: Option<String>,

    #[serde(default = "default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "default_command")]
    pub on_click: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AudioConfig {
    #[serde(default = "AudioConfig::default_format")]
    pub format: String,

    #[serde(default = "AudioConfig::default_icons")]
    pub icons: Vec<String>,

    #[serde(default = "AudioConfig::default_muted_icon")]
    pub muted_icon: String,

    #[serde(default = "AudioConfig::default_interval")]
    pub interval: u64,

    #[serde(default = "default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "default_on_click")]
    pub on_click: String,

    #[serde(default = "default_command")]
    pub on_click_right: String,

    #[serde(default = "default_command")]
    pub on_click_middle: String,

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

    #[serde(default = "default_tooltip")]
    pub tooltip: bool,

    #[serde(default = "ClockConfig::default_tooltip_format")]
    pub tooltip_format: String,

    #[serde(default = "default_on_click")]
    pub on_click: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrayConfig {
    #[serde(default = "TrayConfig::default_spacing")]
    pub spacing: i32,

    #[serde(default = "TrayConfig::default_icon_size")]
    pub icon_size: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoxConfig {
    #[serde(default)]
    pub modules: Vec<String>,

    #[serde(default = "default_command")]
    pub on_click: String,

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

fn default_command() -> String {
    "".to_string()
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
            modules_left: Vec::new(),
            modules_center: Vec::new(),
            modules_right: Vec::new(),
            custom_modules: std::collections::HashMap::new(),
            workspaces: WorkspacesConfig::default(),
            active_window: ActiveWindowConfig::default(),
            network: NetworkConfig::default(),
            mpris: MprisConfig::default(),
            battery: BatteryConfig::default(),
            audio: AudioConfig::default(),
            clock: ClockConfig::default(),
            tray: TrayConfig::default(),
            boxes: std::collections::HashMap::new(),
            revealers: std::collections::HashMap::new(),
        }
    }
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: optional_format(),
            icons: default_icons(),
            min_workspace_count: Self::default_workspaces_count(),
            workspace_formating: Self::workspace_formating(),
            // tooltip: Self::default_tooltip(),
            // tooltip_format: Self::default_tooltip_format(),
        }
    }
}

impl WorkspacesConfig {
    fn default_workspaces_count() -> i32 {
        4
    }

    fn workspace_formating() -> Option<HashMap<u32, String>> {
        None
    }

    /* fn default_tooltip() -> bool {
        true
    }

    fn default_tooltip_format() -> String {
        "Workspaces".to_string()
    } */
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            spacing: Self::default_spacing(),
            icon_size: Self::default_icon_size(),
        }
    }
}
impl TrayConfig {
    fn default_spacing() -> i32 {
        32
    }

    fn default_icon_size() -> i32 {
        32
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            on_click: default_command(),
            format: Self::default_format(),
            active_icons: Self::default_active_icons(),
            ethernet_icon: None,
            disconnected_icon: None,
            interval: Self::default_interval(),
            interface: None,
            tooltip: default_tooltip(),
        }
    }
}

impl NetworkConfig {
    fn default_format() -> String {
        "{icon} {essid}".to_string()
    }

    pub fn default_active_icons() -> Vec<String> {
        vec![
            "󰤯".to_string(),
            "󰤟".to_string(),
            "󰤢".to_string(),
            "󰤥".to_string(),
            "󰤨".to_string(),
        ]
    }

    fn default_interval() -> u64 {
        5
    }
}

impl Default for MprisConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            format_playing: None,
            format_paused: None,
            format_stopped: None,
            format_nothing: Self::default_format_nothing(),
            length_lim: default_length(),
            interval: Self::default_interval(),
            tooltip: default_tooltip(),
            tooltip_format: Self::default_tooltip_format(),
        }
    }
}

impl MprisConfig {
    fn default_format() -> String {
        "{icon} {artist} - {title}".to_string()
    }

    fn default_format_nothing() -> String {
        "No Media".to_string()
    }

    fn default_interval() -> u64 {
        100
    }

    fn default_tooltip_format() -> String {
        "{artist}\n{album}\n{title}".to_string()
    }

    pub fn normalize(&mut self) {
        if self.format_playing.is_none() {
            self.format_playing = Some(self.format.clone());
        }
        if self.format_paused.is_none() {
            self.format_paused = Some(self.format.clone());
        }
        if self.format_stopped.is_none() {
            self.format_stopped = Some(self.format.clone());
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            format_charging: Self::default_format_charging(),
            interval: Self::default_interval(),
            battery: None,
            tooltip: default_tooltip(),
            on_click: default_command(),
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

    fn default_interval() -> u64 {
        30
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            icons: Self::default_icons(),
            muted_icon: Self::default_muted_icon(),
            interval: Self::default_interval(),
            tooltip: default_tooltip(),
            on_click: default_on_click(),
            on_click_right: default_command(),
            on_click_middle: default_command(),
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

    pub fn default_muted_icon() -> String {
        "".to_string()
    }

    pub fn default_icons() -> Vec<String> {
        vec!["".to_string(), "".to_string(), "".to_string()]
    }

    fn default_interval() -> u64 {
        250
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

impl Default for ActiveWindowConfig {
    fn default() -> Self {
        Self {
            format: optional_format(),
            length_lim: default_length(),
            tooltip: default_tooltip(),
            on_click: default_on_click(),
            no_window_format: String::from("No Window"),
        }
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            interval: Self::default_interval(),
            tooltip: default_tooltip(),
            tooltip_format: Self::default_tooltip_format(),
            on_click: Self::default_on_click(),
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

    fn default_tooltip_format() -> String {
        "%A, %B %d, %Y".to_string()
    }

    fn default_on_click() -> String {
        String::new()
    }
}

impl Config {
    pub fn load(c_path: PathBuf) -> Self {
        let config_path = c_path;

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        println!("Loaded config from: {:?}", config_path);

                        // MPRIS normalize
                        config.mpris.normalize();

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

    pub fn get_config_path() -> PathBuf {
        let mut path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
        path.push(".config/riftbar/config.toml");
        path
    }

    fn create_example_config(path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let example = r#"# Riftbar Example Configuration
# -----------------------------
# This is an annotated example of a Riftbar config.
# All modules, boxes, revealers, and custom modules are explained.

# -----------------------------
# Module positions (MUST be at root level, BEFORE any [sections])
# -----------------------------
modules_left = ["custom/arch","mpris"]
modules_center = ["hyprland/workspaces"]
modules_right = ["box/quickcenter", "clock"]

# -----------------------------
# Bar configuration
# -----------------------------
[bar]
height = 30                   # Bar height in pixels
position = "top"              # top or bottom
layer = "top"                 # background, bottom, top, overlay
spacing = 10                  # Space between modules

# -----------------------------
# Clock module
# -----------------------------
[clock]
format = "%H:%M"              # Display format
interval = 1                  # Update interval in seconds
tooltip = true                # Show tooltip on hover
tooltip_format = "%A, %B %d, %Y"  # Tooltip format
on_click = ""                 # Optional: command to run on click

# Available placeholders (date command style):
# %H - Hour (00-23)
# %M - Minute (00-59)
# %S - Second (00-59)
# %A - Full weekday name
# %a - Abbreviated weekday name
# %B - Full month name
# %b - Abbreviated month name
# %d - Day of month (01-31)
# %Y - Year with century

# -----------------------------
# Network module
# -----------------------------
[network]
format = "{icon} {essid}"      # Display format
interval = 5
tooltip = true
# interface = "wlan0"          # Optional: specify interface
on_click = ""                  # Optional: command to run on click

# Placeholders:
# {icon} - Dynamic icon based on signal strength
# {essid} - WiFi SSID
# {signalStrength} - Signal strength (0-100)
# {signalStrengthApp} - Signal strength with % symbol
# {ifname} - Interface name
# {ipaddr} - IP address

# -----------------------------
# Audio module
# -----------------------------
[audio]
format = "{icon} {volume}%"     # Format for normal volume
format_muted = "{icon} Muted"   # Format when muted
interval = 100                  # Update interval in milliseconds
tooltip = true
on_click = ""                   # Optional: command to run on click
on_scroll_up = ""               # Optional: scroll up behavior
on_scroll_down = ""             # Optional: scroll down behavior
scroll_step = 5                 # Volume change step in %

# Placeholders:
# {icon} - Dynamic icon based on volume level
# {volume} - Volume percentage (0-100)

# -----------------------------
# MPRIS (Media Player) module
# -----------------------------
[mpris]
format = "{icon} {artist} - {title}"                # Currently playing
# format_playing = "{icon} {artist} - {title}"      # Optional: Inherits format
# format_paused = "{icon} {artist} - {title}"       # Optional: Inherits format
# format_stopped = "{icon} Stopped"                 # Optional: Inherits format
interval = 100
tooltip = true
tooltip_format = "{artist}\n{album}\n{title}"

# Placeholders:
# {icon} - Dynamic icon based on playback state
# {artist} - Artist name
# {title} - Song title
# {album} - Album name
# {status} - Playback status (Playing, Paused, Stopped)

# -----------------------------
# Battery module
# -----------------------------
[battery]
format = "{icon} {capacity}%"
format_charging = "{icon} {capacity}%"
format_full = "{icon} Full"
interval = 30
tooltip = true
# battery = "BAT0"               # Optional: specify battery device

# Placeholders:
# {icon} - Dynamic icon based on capacity and status
# {capacity} - Battery percentage
# {status} - Charging, Discharging, Full
# {time} - Time remaining / until full

# -----------------------------
# Box widgets - simple containers
# -----------------------------
[boxes.quickcenter]
modules = ["network", "audio", "battery"]
spacing = 5
# orientation = "horizontal"   # horizontal or vertical (default: horizontal)
# Boxes are simple containers that group multiple modules together.
# Use orientation to change layout direction.

# -----------------------------
# Revealer widgets - hover/click containers
# -----------------------------
[revealers.quicksettings]
modules = ["network", "audio", "battery"]
spacing = 5
trigger = "󰣇"                  # Text or icon for the reveal button
transition = "slide_left"       # slide_left, slide_right, slide_up, slide_down, crossfade
transition_duration = 200       # Transition time in milliseconds
reveal_on_hover = true          # Reveal on hover instead of click
# orientation = "horizontal"   # horizontal or vertical (default: horizontal)
# Revealer modules allow modules to appear when triggered (hover/click).

# -----------------------------
# Custom modules
# -----------------------------
[custom_modules.arch]
exec = "echo '󰣇'"               # Command to execute
format = "{}"                  # Display format
interval = 999999              # Update interval
on_click = ""                  # Command to run on click
# Custom modules let you run any command and format its output.
# Use on_click or on_click_right or on_click_middle to define interactions.

[custom_modules.weather]
exec = "curl -s 'wttr.in/?format=%t'"
format = "🌡️ {}"
interval = 600
on_click = ""

[custom_modules.uptime]
exec = "uptime -p | sed 's/up //'"
format = "⏱️ {}"
interval = 60
on_click = """#;

        fs::write(path, example)?;
        println!("Created example config at: {:?}", path);
        Ok(())
    }
}

fn default_length() -> u64 {
    0
}

fn default_tooltip() -> bool {
    false
}

fn default_on_click() -> String {
    String::new()
}

fn optional_format() -> Option<String> {
    None
}

fn default_icons() -> Option<HashMap<String, String>> {
    None
}
