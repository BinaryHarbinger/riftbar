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
    #[serde(default = "default_command")]
    pub exec: String,

    #[serde(default = "default_command")]
    pub on_click: String,

    #[serde(default = "default_command")]
    pub on_click_right: String,

    #[serde(default = "default_command")]
    pub on_click_middle: String,

    #[serde(default = "default_command")]
    pub scroll_up: String,

    #[serde(default = "default_command")]
    pub scroll_down: String,

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
    #[serde(default = "WorkspacesConfig::default_show_special_workspaces")]
    pub show_special_workspaces: bool,
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

    #[serde(default = "BatteryConfig::default_icons")]
    pub icons: Vec<String>,

    #[serde(default = "BatteryConfig::charging_icon")]
    pub charging_icon: String,

    #[serde(default = "BatteryConfig::not_charging_icon")]
    pub not_charging_icon: String,

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
    pub scroll_up: String,

    #[serde(default = "AudioConfig::default_on_scroll_down")]
    pub scroll_down: String,

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

    #[serde(default = "default_on_click")]
    pub on_click_middle: String,

    #[serde(default = "default_on_click")]
    pub on_click_right: String,
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

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: optional_format(),
            icons: default_icons(),
            min_workspace_count: Self::default_workspaces_count(),
            workspace_formating: Self::workspace_formating(),
            show_special_workspaces: Self::default_show_special_workspaces(),
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

    fn default_show_special_workspaces() -> bool {
        false
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
            icons: Self::default_icons(),
            charging_icon: Self::charging_icon(),
            not_charging_icon: Self::not_charging_icon(),
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

    pub fn default_icons() -> Vec<String> {
        vec![
            String::from("󰂎"),
            String::from("󰁺"),
            String::from("󰁻"),
            String::from("󰁼"),
            String::from("󰁽"),
            String::from("󰁾"),
            String::from("󰁿"),
            String::from("󰂀"),
            String::from("󰂁"),
            String::from("󰂂"),
            String::from("󰁹"),
        ]
    }

    pub fn charging_icon() -> String {
        String::from("󰂄")
    }

    pub fn not_charging_icon() -> String {
        String::from("󱟤")
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
            scroll_up: Self::default_on_scroll_up(),
            scroll_down: Self::default_on_scroll_down(),
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
            on_click_right: Self::default_on_click(),
            on_click_middle: Self::default_on_click(),
        }
    }
}

impl ClockConfig {
    fn default_format() -> String {
        "%H:%M".to_string()
    }

    fn default_interval() -> u64 {
        300
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
            println!(
                "Example configuration created automatically in ~/.config/riftbar/ folder.\n 
                You can make your changes. Riftbar will run normaly on next launch"
            );

            let _ = Self::create_example_config(&config_path);
        }
        std::process::exit(1);
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

        let example = r#"
        # Riftbar Configuration

# Module positions (MUST be at root level, BEFORE any [sections])
modules_left = ["box/left"]
modules_center = ["hyprland/workspaces"]
modules_right = ["box/right"]

[bar]
position = "top"  # top, bottom
layer = "top"     # background, bottom, top, overlay

# Clock module configuration
[clock]
format = " %H:%M"
interval = 300  # milliseconds
tooltip = true
tooltip_format = "%d %B %Y"
on_click = "ewwii open calendar --toggle --no-daemonize & ewwii close on_clickcenter musiccenter"  # Optional: command to run on click

# Available format placeholders (uses date command format):
# %H _ Hour (00_23)
# %M _ Minute (00_59)
# %S _ Second (00_59)
# %A _ Full weekday name
# %a _ Abbreviated weekday name
# %B _ Full month name
# %b _ Abbreviated month name
# %d _ Day of month (01_31)
# %Y _ Year with century

[workspaces]
# format = "{id} {icon}"
# icons = { "active" = "A", "normal" = "N"}
min_workspace_count = 4
workspace_formating = { 1 = "一", 2 = "二", 3 = "三", 4 = "四", 5 = "五", 6 = "六", 7 = "七", 8 = "八", 9 = "九", 10 = "十"}

# Network module configuration
[network]
on_click = "bash ~/Dotfiles/scripts/quickcenter.sh &  ewwii close calendar"
format = "{icon}"
active_icons = ["󰤯","󰤟","󰤢","󰤥","󰤨"] 
format_ethernet = " "
interval = 5
tooltip = true

# Audio module configuration
[audio]
format = "{icon}"
interval = 150  # milliseconds
tooltip = true
on_click = "bash ~/Dotfiles/scripts/quickcenter.sh &  ewwii close calendar"
on_click_right = "nohup foot --override=colors.alpha=1 --app-id=binarydotsTUI -e wiremix >/dev/null 2>&1 &"
on_scroll_up = ""
on_scroll_down = ""
scroll_step = 5

# MPRIS (Media Player) configuration
[mpris]
format = "{icon}  {title} - {artist}"
format_nothing = "No Media"
length_lim = 32
interval = 100  # milliseconds
tooltip = true
tooltip_format = "{artist}\n{album}\n{title}"

# Battery configuration
[battery]
format = "{icon}"
icons = ["󰂎", "󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂁", "󰂂", "󰁹"]
charging_icon = "󰂄"
on_click = "bash ~/Dotfiles/scripts/quickcenter.sh &  ewwii close calendar"
interval = 30
tooltip = true

[tray]
icon_size = 20
spacing = 2

[active_window]
format = "{class}"
use_class = true
# no_window_format = ""

[boxes.left]
modules = ["custom/search", "custom/settings", "custom/arch", "custom/seperator", "active_window"]
spacing = 2

[boxes.right]
modules = ["revealer/tray","box/quicksettings", "clock"]

[boxes.quicksettings]
modules = ["network", "audio", "battery"]
on_click = "bash ~/Dotfiles/scripts/quickcenter.sh &  ewwii close calendar"
spacing = 5
orientation = "horizontal"  # horizontal or vertical (default: horizontal)

[revealers.tray]
modules = ["tray"]
spacing = 10
trigger = " "  # Text/icon for the trigger button
transition = "slide_left"  # slide_left, slide_right, slide_up, slide_down, crossfade
transition_duration = 600  # milliseconds
reveal_on_hover = false  # Reveal on hover instead of click
# orientation = "horizontal"  # horizontal or vertical (default: horizontal)

# Custom modules
[custom_modules.search]
on_click = "~/Dotfiles/bin/launchrofi --drun"
format = " "

[custom_modules.settings]
on_click = "~/Dotfiles/bin/launchrofi -sm"
format = " "

[custom_modules.arch]
on_click = "~/Dotfiles/bin/launchrofi --menu"
format = " "

# [custom_modules.mako]
# on_click = "bash ~/Dotfiles/config/mako/scripts/walker.sh"
# on_click_right = "bash ~/Dotfiles/config/mako/scripts/riftbar.sh -d"
# exec = "~/Dotfiles/config/mako/scripts/riftbar.sh"
# interval = 1
# format = "{}"

[custom_modules.seperator]
format = "|"

        "#;

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
