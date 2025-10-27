// ============ modules/battery.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct BatteryWidget {
    button: gtk::Button,
}

#[derive(Clone)]
pub struct BatteryConfig {
    pub format: String,
    pub format_charging: String,
    pub format_full: String,
    pub interval: u64,
    pub battery: Option<String>,
    pub tooltip: bool,
    pub action: String,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: "{icon} {capacity}%".to_string(),
            format_charging: "{icon} {capacity}%".to_string(),
            format_full: "{icon} Full".to_string(),
            interval: 30,
            battery: None,
            tooltip: true,
            action: "".to_string(),
        }
    }
}

impl BatteryConfig {
    pub fn from_config(config: &crate::config::BatteryConfig) -> Self {
        Self {
            format: config.format.clone(),
            format_charging: config.format_charging.clone(),
            format_full: config.format_full.clone(),
            interval: config.interval,
            battery: config.battery.clone(),
            tooltip: config.tooltip,
            action: config.action.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct BatteryInfo {
    capacity: i32,
    status: String,
    time_remaining: String,
    power_now: f64,
}

impl BatteryWidget {
    pub fn new(config: BatteryConfig) -> Self {
        let button = gtk::Button::with_label("");

        // Connect button click handler
        let action_command = config.action.clone();
        button.connect_clicked(move |_| {
            if !action_command.is_empty() {
                Self::run_command_async(&action_command);
            }
        });

        button.add_css_class("battery");
        button.add_css_class("module");

        let battery_info = Arc::new(Mutex::new(BatteryInfo {
            capacity: 0,
            status: String::new(),
            time_remaining: String::new(),
            power_now: 0.0,
        }));

        // Update immediately
        let info = get_battery_info(config.battery.as_deref());
        *battery_info.lock().unwrap() = info.clone();
        update_button(&button, &info, &config);

        // Set up periodic updates
        let button_clone = button.clone();
        let config_clone = config.clone();
        let battery_info_clone = battery_info.clone();

        glib::timeout_add_seconds_local(config.interval as u32, move || {
            let info = get_battery_info(config_clone.battery.as_deref());
            *battery_info_clone.lock().unwrap() = info.clone();
            update_button(&button_clone, &info, &config_clone);
            glib::ControlFlow::Continue
        });

        // Add tooltip if enabled
        if config.tooltip {
            let battery_info_clone = battery_info.clone();
            button.set_has_tooltip(true);
            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let info = battery_info_clone.lock().unwrap();
                let tooltip_text = format!(
                    "Status: {}\nCapacity: {}%\n{}Power: {:.2}W",
                    info.status,
                    info.capacity,
                    if !info.time_remaining.is_empty() {
                        format!("{}\n", info.time_remaining)
                    } else {
                        String::new()
                    },
                    info.power_now
                );
                tooltip.set_text(Some(&tooltip_text));
                true
            });
        }

        Self { button }
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn run_command_async(command: &str) {
        let command = command.to_string();
        std::thread::spawn(move || {
            let _ = Command::new("sh").arg("-c").arg(&command).output();
        });
    }
}

fn update_button(button: &gtk::Button, info: &BatteryInfo, config: &BatteryConfig) {
    let icon = get_icon_for_capacity(info.capacity, &info.status);

    let format_template = if info.status == "Full" {
        &config.format_full
    } else if info.status == "Charging" {
        &config.format_charging
    } else {
        &config.format
    };

    let text = format_template
        .replace("{icon}", &icon)
        .replace("{capacity}", &info.capacity.to_string())
        .replace("{status}", &info.status)
        .replace("{time}", &info.time_remaining);

    button.set_label(&text);

    // Update CSS classes based on capacity and status
    button.remove_css_class("charging");
    button.remove_css_class("full");
    button.remove_css_class("critical");
    button.remove_css_class("low");
    button.remove_css_class("medium");
    button.remove_css_class("high");

    if info.status == "Charging" {
        button.add_css_class("charging");
    } else if info.status == "Full" {
        button.add_css_class("full");
    } else if info.capacity <= 10 {
        button.add_css_class("critical");
    } else if info.capacity <= 25 {
        button.add_css_class("low");
    } else if info.capacity <= 50 {
        button.add_css_class("medium");
    } else {
        button.add_css_class("high");
    }
}

fn get_icon_for_capacity(capacity: i32, status: &str) -> String {
    if status == "Charging" {
        return "󰂄".to_string(); // Charging icon
    }

    // Battery level icons
    if capacity >= 90 {
        "󰁹".to_string() // Full
    } else if capacity >= 80 {
        "󰂂".to_string() // 90%
    } else if capacity >= 70 {
        "󰂁".to_string() // 80%
    } else if capacity >= 60 {
        "󰂀".to_string() // 70%
    } else if capacity >= 50 {
        "󰁿".to_string() // 60%
    } else if capacity >= 40 {
        "󰁾".to_string() // 50%
    } else if capacity >= 30 {
        "󰁽".to_string() // 40%
    } else if capacity >= 20 {
        "󰁼".to_string() // 30%
    } else if capacity >= 10 {
        "󰁻".to_string() // 20%
    } else {
        "󰁺".to_string() // 10% or less - critical
    }
}

fn get_battery_info(battery_filter: Option<&str>) -> BatteryInfo {
    let battery_name = if let Some(name) = battery_filter {
        name.to_string()
    } else {
        // Auto-detect battery
        find_battery().unwrap_or_else(|| "BAT0".to_string())
    };

    let base_path = PathBuf::from(format!("/sys/class/power_supply/{}", battery_name));

    // Read capacity
    let capacity = read_sys_file(&base_path.join("capacity"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Read status
    let status = read_sys_file(&base_path.join("status")).unwrap_or_else(|| "Unknown".to_string());

    // Calculate time remaining
    let time_remaining = calculate_time_remaining(&base_path, &status);

    // Read power consumption
    let power_now = read_sys_file(&base_path.join("power_now"))
        .and_then(|s| s.parse::<f64>().ok())
        .map(|p| p / 1_000_000.0) // Convert from µW to W
        .unwrap_or(0.0);

    BatteryInfo {
        capacity,
        status,
        time_remaining,
        power_now,
    }
}

fn find_battery() -> Option<String> {
    let power_supply_path = PathBuf::from("/sys/class/power_supply");

    if let Ok(entries) = fs::read_dir(power_supply_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                return Some(name);
            }
        }
    }

    None
}

fn read_sys_file(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn calculate_time_remaining(base_path: &PathBuf, status: &str) -> String {
    let energy_now =
        read_sys_file(&base_path.join("energy_now")).and_then(|s| s.parse::<f64>().ok());

    let power_now = read_sys_file(&base_path.join("power_now")).and_then(|s| s.parse::<f64>().ok());

    let energy_full =
        read_sys_file(&base_path.join("energy_full")).and_then(|s| s.parse::<f64>().ok());

    if let (Some(energy), Some(power)) = (energy_now, power_now) {
        if power > 0.0 {
            let hours = if status == "Charging" {
                if let Some(full) = energy_full {
                    (full - energy) / power
                } else {
                    return String::new();
                }
            } else {
                energy / power
            };

            let hours_int = hours.floor() as i32;
            let minutes = ((hours - hours.floor()) * 60.0) as i32;

            if status == "Charging" {
                return format!("{}:{:02} until full", hours_int, minutes);
            } else {
                return format!("{}:{:02} remaining", hours_int, minutes);
            }
        }
    }

    String::new()
}
