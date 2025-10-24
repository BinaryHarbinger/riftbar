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

    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub format: Option<String>,

    #[serde(default)]
    pub tooltip: Option<bool>,
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

fn default_modules_left() -> Vec<String> {
    vec!["mpris".to_string()]
}

fn default_modules_center() -> Vec<String> {
    vec!["hyprland/workspaces".to_string()]
}

fn default_modules_right() -> Vec<String> {
    vec!["network".to_string(), "clock".to_string()]
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
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    println!("Config content:\n{}", content);
                    match toml::from_str::<Config>(&content) {
                        Ok(config) => {
                            println!("Loaded config from: {:?}", config_path);
                            println!("Parsed modules_left: {:?}", config.modules_left);
                            println!("Parsed modules_center: {:?}", config.modules_center);
                            println!("Parsed modules_right: {:?}", config.modules_right);
                            return config;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse config: {}. Using defaults.", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config: {}. Using defaults.", e);
                }
            }
        } else {
            println!(
                "Config file not found, using defaults. Creating example at: {:?}",
                config_path
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

# Module positions (must be at root level, not in [bar])
modules_left = ["mpris"]
modules_center = ["hyprland/workspaces"]
modules_right = ["network", "clock"]

[bar]
height = 30
position = "top"  # top, bottom
layer = "top"     # background, bottom, top, overlay
spacing = 10


# Custom modules
[custom_modules.weather]
exec = "curl -s 'wttr.in/?format=%t'"
interval = 600
format = "🌡️ {}"

[custom_modules.uptime]
exec = "uptime -p | sed 's/up //'"
interval = 60
format = "⏱️ {}"
"#;

        fs::write(path, example)?;
        println!("Created example config at: {:?}", path);
        Ok(())
    }
}
