// ============ modules/network.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct NetworkWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct NetworkConfig {
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
    pub fn new(config: NetworkConfig) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        container.add_css_class("network");
        container.add_css_class("module");

        let label = gtk::Label::new(Some("󰖪"));
        label.add_css_class("network-label");
        container.append(&label);

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
        update_label(&label, &info, &config);

        // Set up periodic updates
        let label_clone = label.clone();
        let config_clone = config.clone();
        let network_info_clone = network_info.clone();

        glib::timeout_add_seconds_local(config.interval as u32, move || {
            let info = get_network_info(config_clone.interface.as_deref());
            *network_info_clone.lock().unwrap() = info.clone();
            update_label(&label_clone, &info, &config_clone);
            glib::ControlFlow::Continue
        });

        // Add tooltip if enabled
        if config.tooltip {
            let network_info_clone = network_info.clone();
            container.set_has_tooltip(true);
            container.connect_query_tooltip(move |_, _, _, _, tooltip| {
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

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
}

fn update_label(label: &gtk::Label, info: &NetworkInfo, config: &NetworkConfig) {
    let text = if !info.connected {
        config.format_disconnected.clone()
    } else if info.is_ethernet {
        format_string(&config.format_ethernet, info)
    } else {
        format_string(&config.format, info)
    };

    label.set_text(&text);

    // Update CSS classes based on signal strength
    label.remove_css_class("excellent");
    label.remove_css_class("good");
    label.remove_css_class("ok");
    label.remove_css_class("weak");
    label.remove_css_class("disconnected");
    label.remove_css_class("ethernet");

    if !info.connected {
        label.add_css_class("disconnected");
    } else if info.is_ethernet {
        label.add_css_class("ethernet");
    } else if info.signal_strength >= 75 {
        label.add_css_class("excellent");
    } else if info.signal_strength >= 50 {
        label.add_css_class("good");
    } else if info.signal_strength >= 25 {
        label.add_css_class("ok");
    } else {
        label.add_css_class("weak");
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
        .args(&["-t", "-f", "ACTIVE,SSID,SIGNAL,DEVICE,TYPE", "dev", "wifi"])
        .output()
        .ok()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 && parts[0] == "yes" && parts[4] == "wifi" {
                let interface = parts[3].to_string();

                // Filter by interface if specified
                if let Some(filter) = interface_filter {
                    if interface != filter {
                        continue;
                    }
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
        .args(&["dev", interface, "link"])
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
        } else if line.starts_with("signal:") {
            if let Some(signal_str) = line.strip_prefix("signal:") {
                if let Some(dbm_str) = signal_str.trim().split_whitespace().next() {
                    if let Ok(dbm) = dbm_str.parse::<i32>() {
                        // Convert dBm to percentage (rough approximation)
                        signal_strength = ((dbm + 100) * 2).max(0).min(100);
                    }
                }
            }
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
        .args(&["-4", "addr", "show", interface])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().starts_with("inet ") {
                    if let Some(ip) = line.trim().split_whitespace().nth(1) {
                        return ip.split('/').next().unwrap_or("").to_string();
                    }
                }
            }
        }
    }

    String::from("N/A")
}
