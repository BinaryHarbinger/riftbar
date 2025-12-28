// ============ modules/network.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::process::Command as tokioCommand;

pub struct NetworkWidget {
    button: gtk::Button,
}

#[derive(Clone)]
pub struct NetworkConfig {
    pub action: String,
    pub format: String,
    pub format_disconnected: String,
    pub format_ethernet: String,
    pub interval: u64,
    pub interface: Option<String>,
    pub tooltip: bool,
}

impl NetworkConfig {
    pub fn from_config(config: &crate::config::NetworkConfig) -> Self {
        Self {
            action: config.action.clone(),
            format: config.format.clone(),
            format_disconnected: config.format_disconnected.clone(),
            format_ethernet: config.format_ethernet.clone(),
            interval: config.interval,
            interface: config.interface.clone(),
            tooltip: config.tooltip,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            action: ":".to_string(),
            format: "{icon} {essid}".to_string(),
            format_disconnected: "󰖪 Disconnected".to_string(),
            format_ethernet: "󰈀 {ifname}".to_string(),
            interval: 5,
            interface: None,
            tooltip: true,
        }
    }
}

#[derive(Clone, Debug)]
struct NetworkInfo {
    connected: bool,
    essid: String,
    signal_strength: i32,
    interface: String,
    is_ethernet: bool,
    ip_address: String,
}

impl NetworkWidget {
    pub fn new(config: Arc<NetworkConfig>) -> Self {
        let button = gtk::Button::with_label(" ");
        button.set_widget_name("network");
        button.add_css_class("module");
        button.add_css_class("network");

        let action_command = config.action.clone();

        // Left click handler
        button.connect_clicked(move |_| {
            Self::run_action_async(action_command.clone());
        });

        let network_info = Arc::new(Mutex::new(NetworkInfo {
            connected: false,
            essid: String::new(),
            signal_strength: 0,
            interface: String::new(),
            is_ethernet: false,
            ip_address: String::new(),
        }));

        // Update immediately
        let info = get_network_info(config.interface.as_deref());
        *network_info.lock().unwrap() = info.clone();
        update_button(&button, &info, &config);

        // Set up periodic updates
        let button_clone = button.clone();
        let config_clone = Arc::clone(&config);
        let network_info_clone = network_info.clone();

        glib::timeout_add_seconds_local(config.interval as u32, move || {
            let info = get_network_info(config_clone.interface.as_deref());
            *network_info_clone.lock().unwrap() = info.clone();
            update_button(&button_clone, &info, &config_clone);
            glib::ControlFlow::Continue
        });

        // Add tooltip if enabled
        if config.tooltip {
            let network_info_clone = network_info.clone();
            button.set_has_tooltip(true);
            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let info = network_info_clone.lock().unwrap();
                if info.connected {
                    let tooltip_text = if info.is_ethernet {
                        format!(
                            "Interface: {}\nType: Ethernet\nIP: {}",
                            info.interface, info.ip_address
                        )
                    } else {
                        format!(
                            "SSID: {}\nSignal: {}%\nInterface: {}\nIP: {}",
                            info.essid, info.signal_strength, info.interface, info.ip_address
                        )
                    };
                    tooltip.set_text(Some(&tooltip_text));
                } else {
                    tooltip.set_text(Some("Disconnected"));
                }
                true
            });
        }

        Self { button }
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn run_action_async(action: String) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = tokioCommand::new("sh")
                    .arg("-c")
                    .arg(action.clone())
                    .output()
                    .await;
            });
        });
    }
}

fn update_button(button: &gtk::Button, info: &NetworkInfo, config: &NetworkConfig) {
    let text = if !info.connected {
        config.format_disconnected.clone()
    } else if info.is_ethernet {
        format_string(&config.format_ethernet, info)
    } else {
        format_string(&config.format, info)
    };

    button.set_label(&text);

    // Update CSS classes based on signal strength
    button.remove_css_class("excellent");
    button.remove_css_class("good");
    button.remove_css_class("ok");
    button.remove_css_class("weak");
    button.remove_css_class("disconnected");
    button.remove_css_class("ethernet");

    if !info.connected {
        button.add_css_class("disconnected");
    } else if info.is_ethernet {
        button.add_css_class("ethernet");
    } else if info.signal_strength >= 75 {
        button.add_css_class("excellent");
    } else if info.signal_strength >= 50 {
        button.add_css_class("good");
    } else if info.signal_strength >= 25 {
        button.add_css_class("ok");
    } else {
        button.add_css_class("weak");
    }
}

fn format_string(format: &str, info: &NetworkInfo) -> String {
    let icon = get_icon_for_strength(info.signal_strength, info.is_ethernet);

    format
        .replace("{icon}", &icon)
        .replace("{essid}", &info.essid)
        .replace("{signalStrength}", &info.signal_strength.to_string())
        .replace("{signalStrengthApp}", &format!("{}%", info.signal_strength))
        .replace("{ifname}", &info.interface)
        .replace("{ipaddr}", &info.ip_address)
}

fn get_icon_for_strength(strength: i32, is_ethernet: bool) -> String {
    if is_ethernet {
        return "󰈀".to_string();
    }

    if strength >= 75 {
        "󰤨".to_string() // Full signal
    } else if strength >= 50 {
        "󰤥".to_string() // Good signal
    } else if strength >= 25 {
        "󰤢".to_string() // Medium signal
    } else if strength > 0 {
        "󰤟".to_string() // Weak signal
    } else {
        "󰤭".to_string() // No signal
    }
}

fn get_network_info(interface_filter: Option<&str>) -> NetworkInfo {
    // Try to get WiFi info first
    if let Some(wifi_info) = get_wifi_info(interface_filter) {
        return wifi_info;
    }

    // Check for ethernet connection
    if let Some(eth_info) = get_ethernet_info(interface_filter) {
        return eth_info;
    }

    // No connection
    NetworkInfo {
        connected: false,
        essid: String::new(),
        signal_strength: 0,
        interface: String::new(),
        is_ethernet: false,
        ip_address: String::new(),
    }
}

fn get_wifi_info(interface_filter: Option<&str>) -> Option<NetworkInfo> {
    // Try nmcli first (NetworkManager)
    let output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID,SIGNAL,DEVICE,TYPE", "dev", "wifi"])
        .output()
        .ok()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 && parts[0] == "yes" && parts[4] == "wifi" {
                let interface = parts[3].to_string();

                // Filter by interface if specified
                if let Some(filter) = interface_filter
                    && interface != filter
                {
                    continue;
                }

                let ip = get_ip_address(&interface);

                return Some(NetworkInfo {
                    connected: true,
                    essid: parts[1].to_string(),
                    signal_strength: parts[2].parse().unwrap_or(0),
                    interface,
                    is_ethernet: false,
                    ip_address: ip,
                });
            }
        }
    }

    // Fallback to iw if nmcli not available
    get_wifi_info_iw(interface_filter)
}

fn get_wifi_info_iw(interface_filter: Option<&str>) -> Option<NetworkInfo> {
    let interface = interface_filter.unwrap_or("wlan0");

    let output = Command::new("iw")
        .args(["dev", interface, "link"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut essid = String::new();
    let mut signal_strength = 0;

    for line in output_str.lines() {
        let line = line.trim();
        if line.starts_with("SSID:") {
            essid = line.strip_prefix("SSID:").unwrap_or("").trim().to_string();
        } else if line.starts_with("signal:")
            && let Some(signal_str) = line.strip_prefix("signal:")
            && let Some(dbm_str) = signal_str.split_whitespace().next()
            && let Ok(dbm) = dbm_str.parse::<i32>()
        {
            // Convert dBm to percentage (rough approximation)
            signal_strength = ((dbm + 100) * 2).clamp(0, 100);
        }
    }

    if !essid.is_empty() {
        let ip = get_ip_address(interface);
        Some(NetworkInfo {
            connected: true,
            essid,
            signal_strength,
            interface: interface.to_string(),
            is_ethernet: false,
            ip_address: ip,
        })
    } else {
        None
    }
}

fn get_ethernet_info(interface_filter: Option<&str>) -> Option<NetworkInfo> {
    let interfaces = if let Some(iface) = interface_filter {
        vec![iface.to_string()]
    } else {
        vec!["eth0".to_string(), "enp0s3".to_string(), "eno1".to_string()]
    };

    for interface in interfaces {
        let output = Command::new("cat")
            .arg(format!("/sys/class/net/{}/operstate", interface))
            .output()
            .ok()?;

        if output.status.success() {
            let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if state == "up" {
                let ip = get_ip_address(&interface);
                return Some(NetworkInfo {
                    connected: true,
                    essid: String::new(),
                    signal_strength: 100,
                    interface: interface.clone(),
                    is_ethernet: true,
                    ip_address: ip,
                });
            }
        }
    }

    None
}

fn get_ip_address(interface: &str) -> String {
    let output = Command::new("ip")
        .args(["-4", "addr", "show", interface])
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.trim().starts_with("inet ")
                && let Some(ip) = line.split_whitespace().nth(1)
            {
                return ip.split('/').next().unwrap_or("").to_string();
            }
        }
    }

    String::from("N/A")
}
