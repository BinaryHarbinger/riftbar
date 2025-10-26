/* ============ modules/tray.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TrayWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct TrayConfig {
    pub spacing: i32,
    pub icon_size: i32,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            spacing: 5,
            icon_size: 16,
        }
    }
}

impl TrayConfig {
    pub fn from_config(config: &crate::config::TrayConfig) -> Self {
        Self {
            spacing: config.spacing,
            icon_size: config.icon_size,
        }
    }
}

// StatusNotifierItem representation
#[derive(Clone, Debug)]
struct TrayItem {
    service: String,
    path: String,
    title: String,
    icon_name: String,
    menu_path: String,
}

impl TrayWidget {
    pub fn new(config: TrayConfig) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, config.spacing);
        container.add_css_class("tray");
        container.add_css_class("module");

        // Store tray items
        let tray_items: Arc<Mutex<HashMap<String, TrayItem>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Start monitoring for tray items
        Self::monitor_tray_items(container.clone(), config.clone(), tray_items);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn monitor_tray_items(
        container: gtk::Box,
        config: TrayConfig,
        tray_items: Arc<Mutex<HashMap<String, TrayItem>>>,
    ) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use tokio::time::{Duration, sleep};

                loop {
                    // Get current tray items via DBus
                    let items = Self::get_tray_items_dbus().await;

                    let mut tray_map = tray_items.lock().unwrap();
                    let old_keys: Vec<String> = tray_map.keys().cloned().collect();
                    let new_keys: Vec<String> =
                        items.iter().map(|item| item.service.clone()).collect();

                    // Check for removed items
                    for old_key in &old_keys {
                        if !new_keys.contains(old_key) {
                            tray_map.remove(old_key);
                            // Schedule UI update to remove button
                            let container_clone = container.clone();
                            let key_clone = old_key.clone();
                            glib::idle_add_once(move || {
                                Self::remove_tray_button(&container_clone, &key_clone);
                            });
                        }
                    }

                    // Check for new items
                    for item in items {
                        let key = item.service.clone();
                        if !old_keys.contains(&key) {
                            tray_map.insert(key.clone(), item.clone());
                            // Schedule UI update to add button
                            let container_clone = container.clone();
                            let config_clone = config.clone();
                            glib::idle_add_once(move || {
                                Self::add_tray_button(&container_clone, &item, &config_clone);
                            });
                        }
                    }

                    drop(tray_map);
                    sleep(Duration::from_secs(2)).await;
                }
            });
        });
    }

    async fn get_tray_items_dbus() -> Vec<TrayItem> {
        use tokio::process::Command;

        // Query StatusNotifierWatcher for registered items
        let output = Command::new("busctl")
            .args(&[
                "--user",
                "call",
                "org.kde.StatusNotifierWatcher",
                "/StatusNotifierWatcher",
                "org.kde.StatusNotifierWatcher",
                "RegisteredStatusNotifierItems",
            ])
            .output()
            .await;

        let mut items = Vec::new();

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);

                // Parse the output to extract service names
                // Format is typically: as N "service1" "service2" ...
                for line in output_str.lines() {
                    if line.contains("\"") {
                        // Extract service names between quotes
                        let parts: Vec<&str> = line.split('"').collect();
                        for (i, part) in parts.iter().enumerate() {
                            if i % 2 == 1 && !part.is_empty() {
                                // This is a service name
                                let service = part.to_string();

                                // Get icon for this service
                                let icon_name = Self::get_icon_for_service(&service).await;

                                items.push(TrayItem {
                                    service: service.clone(),
                                    path: "/StatusNotifierItem".to_string(),
                                    title: service.clone(),
                                    icon_name,
                                    menu_path: "/MenuBar".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        items
    }

    async fn get_icon_for_service(service: &str) -> String {
        use tokio::process::Command;

        // Try to get IconName property
        let output = Command::new("busctl")
            .args(&[
                "--user",
                "get-property",
                service,
                "/StatusNotifierItem",
                "org.kde.StatusNotifierItem",
                "IconName",
            ])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse output like: s "icon-name"
                if let Some(icon) = output_str.split('"').nth(1) {
                    return icon.to_string();
                }
            }
        }

        // Fallback: try to guess icon from service name
        Self::guess_icon_from_service(service)
    }

    fn guess_icon_from_service(service: &str) -> String {
        let service_lower = service.to_lowercase();

        if service_lower.contains("discord") {
            "discord".to_string()
        } else if service_lower.contains("spotify") {
            "spotify".to_string()
        } else if service_lower.contains("telegram") {
            "telegram".to_string()
        } else if service_lower.contains("steam") {
            "steam".to_string()
        } else if service_lower.contains("network") {
            "network-wireless".to_string()
        } else if service_lower.contains("bluetooth") {
            "bluetooth".to_string()
        } else if service_lower.contains("audio") || service_lower.contains("volume") {
            "audio-volume-high".to_string()
        } else {
            "application-x-executable".to_string()
        }
    }

    fn add_tray_button(container: &gtk::Box, item: &TrayItem, config: &TrayConfig) {
        let button = gtk::Button::new();
        button.add_css_class("tray-item");
        button.set_widget_name(&item.service);

        // Try to load icon
        let icon = gtk::Image::from_icon_name(&item.icon_name);
        icon.set_pixel_size(config.icon_size);
        button.set_child(Some(&icon));

        // Set tooltip
        button.set_tooltip_text(Some(&item.title));

        // Handle click to show menu
        let service = item.service.clone();
        let menu_path = item.menu_path.clone();
        button.connect_clicked(move |btn| {
            Self::show_tray_menu(btn, &service, &menu_path);
        });

        container.append(&button);
    }

    fn remove_tray_button(container: &gtk::Box, service: &str) {
        let mut child = container.first_child();
        while let Some(widget) = child {
            if let Some(button) = widget.downcast_ref::<gtk::Button>() {
                if button.widget_name() == service {
                    container.remove(&widget);
                    break;
                }
            }
            child = widget.next_sibling();
        }
    }

    fn show_tray_menu(button: &gtk::Button, service: &str, menu_path: &str) {
        // Create a popup menu
        let menu = gtk::PopoverMenu::new();

        // Try to get menu items via DBus
        let service = service.to_string();
        let menu_path = menu_path.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use tokio::process::Command;

                // Try to activate the item (some apps show their menu on activation)
                let _ = Command::new("busctl")
                    .args(&[
                        "--user",
                        "call",
                        &service,
                        "/StatusNotifierItem",
                        "org.kde.StatusNotifierItem",
                        "Activate",
                        "ii",
                        "0",
                        "0",
                    ])
                    .output()
                    .await;
            });
        });

        menu.set_parent(button);
        menu.popup();
    }
} */
