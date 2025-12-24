// ============ modules/tray/widget.rs ============
use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc;
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

// StatusNotifierItem representation
#[derive(Clone, Debug)]
struct TrayItem {
    service: String,
    path: String,
    title: String,
    icon_name: String,
    #[allow(dead_code)]
    menu_path: String,
}

// Menu item representation
#[derive(Clone, Debug)]
struct MenuItem {
    id: i32,
    label: String,
    #[allow(dead_code)]
    enabled: bool,
    #[allow(dead_code)]
    visible: bool,
}

// Update message type for communication between threads
#[derive(Clone, Debug)]
enum TrayUpdate {
    Add(TrayItem),
    Remove(String),
}

impl TrayWidget {
    pub fn new(config: TrayConfig) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, config.spacing);
        container.add_css_class("tray");
        container.add_css_class("module");

        // Initialize SNI system in background
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = crate::modules::tray::init_sni_system().await {
                    eprintln!("[TRAY] Failed to initialize SNI system: {}", e);
                } else {
                    println!("[TRAY] SNI system ready");
                }
            });
        });

        // Store tray items in thread-safe storage
        let tray_items: Arc<Mutex<HashMap<String, TrayItem>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Create a channel for communicating updates from the async thread to the main thread
        let (tx, rx) = mpsc::channel::<TrayUpdate>();

        // Start monitoring for tray items
        Self::monitor_tray_items(config.clone(), tray_items, tx);

        // Handle updates on the main thread
        let container_clone = container.clone();
        let config_clone = config.clone();

        // Poll for updates from the background thread
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Process all pending updates
            while let Ok(update) = rx.try_recv() {
                match update {
                    TrayUpdate::Add(item) => {
                        Self::add_tray_button(&container_clone, &item, &config_clone);
                    }
                    TrayUpdate::Remove(service) => {
                        Self::remove_tray_button(&container_clone, &service);
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn monitor_tray_items(
        _config: TrayConfig,
        tray_items: Arc<Mutex<HashMap<String, TrayItem>>>,
        tx: mpsc::Sender<TrayUpdate>,
    ) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use tokio::time::{Duration, sleep};

                // Initial delay to let watcher start
                sleep(Duration::from_millis(500)).await;

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
                            let _ = tx.send(TrayUpdate::Remove(old_key.clone()));
                        }
                    }

                    // Check for new items
                    for item in items {
                        let key = item.service.clone();
                        if !old_keys.contains(&key) {
                            tray_map.insert(key.clone(), item.clone());
                            let _ = tx.send(TrayUpdate::Add(item));
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

        let queries = vec![
            (
                "get-property",
                "org.kde.StatusNotifierWatcher",
                "RegisteredStatusNotifierItems",
            ),
        ];

        for (cmd_type, interface, property) in queries {
            let output = Command::new("busctl")
                .args(&[
                    "--user",
                    cmd_type,
                    "org.kde.StatusNotifierWatcher",
                    "/StatusNotifierWatcher",
                    interface,
                    property,
                ])
                .output()
                .await;

            match output {
                Ok(output) if output.status.success() => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let items = Self::parse_tray_items(&output_str).await;
                    if !items.is_empty() {
                        return items;
                    }
                }
                _ => continue,
            }
        }

        Vec::new()
    }

    async fn parse_tray_items(output_str: &str) -> Vec<TrayItem> {
        let mut items = Vec::new();

        for line in output_str.lines() {
            if line.contains("\"") {
                let parts: Vec<&str> = line.split('"').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 && !part.is_empty() {
                        let full_str = part.to_string();

                        let (service, path) = if full_str.contains('/') {
                            let split_pos = full_str.find('/').unwrap();
                            let svc = full_str[..split_pos].to_string();
                            let pth = full_str[split_pos..].to_string();
                            (svc, pth)
                        } else {
                            (full_str, "/StatusNotifierItem".to_string())
                        };

                        let icon_name = Self::get_icon_for_service(&service, &path).await;
                        let title = Self::get_title_for_service(&service, &path).await;

                        items.push(TrayItem {
                            service: service.clone(),
                            path: path.clone(),
                            title: title.unwrap_or_else(|| service.clone()),
                            icon_name,
                            menu_path: "/MenuBar".to_string(),
                        });
                    }
                }
            }
        }

        items
    }

    async fn get_title_for_service(service: &str, path: &str) -> Option<String> {
        use tokio::process::Command;

        let output = Command::new("busctl")
            .args(&[
                "--user",
                "get-property",
                service,
                path,
                "org.kde.StatusNotifierItem",
                "Title",
            ])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(title) = output_str.split('"').nth(1) {
                    return Some(title.to_string());
                }
            }
        }

        None
    }

    async fn get_icon_for_service(service: &str, path: &str) -> String {
        use tokio::process::Command;

        let output = Command::new("busctl")
            .args(&[
                "--user",
                "get-property",
                service,
                path,
                "org.kde.StatusNotifierItem",
                "IconName",
            ])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(icon) = output_str.split('"').nth(1) {
                    if !icon.is_empty() {
                        return icon.to_string();
                    }
                }
            }
        }

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
        } else if service_lower.contains("network") || service_lower.contains("nm-applet") {
            "network-wireless".to_string()
        } else if service_lower.contains("bluetooth") || service_lower.contains("blueman") {
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

        let icon = gtk::Image::from_icon_name(&item.icon_name);
        icon.set_pixel_size(config.icon_size);
        button.set_child(Some(&icon));

        button.set_tooltip_text(Some(&item.title));

        let service = item.service.clone();
        let path = item.path.clone();
        button.connect_clicked(move |_| {
            let svc = service.clone();
            let pth = path.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    use tokio::process::Command;
                    let _ = Command::new("busctl")
                        .args(&[
                            "--user",
                            "call",
                            &svc,
                            &pth,
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
        });

        let right_click = gtk::GestureClick::new();
        right_click.set_button(3);
        let service_rc = item.service.clone();
        let path_rc = item.path.clone();
        right_click.connect_pressed(move |gesture, _n, x, y| {
            if let Some(widget) = gesture.widget() {
                if let Ok(button) = widget.downcast::<gtk::Button>() {
                    Self::show_context_menu(&button, &service_rc, &path_rc, x, y);
                }
            }
        });
        button.add_controller(right_click);

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

    fn show_context_menu(button: &gtk::Button, service: &str, path: &str, x: f64, y: f64) {
        let service = service.to_string();
        let path = path.to_string();
        let button_weak = button.downgrade();
        let x_pos = x;
        let y_pos = y;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let menu_items = rt.block_on(async { Self::get_dbus_menu(&service, &path).await });
            let _ = tx.send((service, path, menu_items));
        });

        glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            if let Ok((svc, pth, menu_items)) = rx.try_recv() {
                if let Some(button) = button_weak.upgrade() {
                    if menu_items.is_empty() {
                        Self::show_fallback_menu(&button, &svc, &pth, x_pos, y_pos);
                    } else {
                        Self::show_dynamic_menu(&button, &svc, &pth, menu_items, x_pos, y_pos);
                    }
                }
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    async fn get_dbus_menu(service: &str, path: &str) -> Vec<MenuItem> {
        use tokio::process::Command;

        let menu_path = Self::get_menu_path(service, path).await;
        let menu_path = menu_path.unwrap_or_else(|| "/MenuBar".to_string());

        let output = Command::new("busctl")
            .args(&[
                "--user",
                "call",
                service,
                &menu_path,
                "com.canonical.dbusmenu",
                "GetLayout",
                "iias",
                "0",
                "--",
                "-1",
                "0",
            ])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return Self::parse_dbus_menu(&output_str).await;
            }
        }

        Vec::new()
    }

    async fn get_menu_path(service: &str, path: &str) -> Option<String> {
        use tokio::process::Command;

        let output = Command::new("busctl")
            .args(&[
                "--user",
                "get-property",
                service,
                path,
                "org.kde.StatusNotifierItem",
                "Menu",
            ])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(menu_path) = output_str.split('"').nth(1) {
                    return Some(menu_path.to_string());
                }
            }
        }

        None
    }

    async fn parse_dbus_menu(_output: &str) -> Vec<MenuItem> {
        let mut items = Vec::new();

        let lines = _output.lines().collect::<Vec<_>>().join(" ");
        let parts: Vec<&str> = lines.split("(ia{sv}av)").collect();

        for part in parts.iter().skip(1) {
            let tokens: Vec<&str> = part.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            if let Ok(id) = tokens[0].parse::<i32>() {
                let mut label = String::new();
                let is_separator = part.contains("\"type\" s \"separator\"");

                if !is_separator {
                    if let Some(label_pos) = part.find("\"label\" s \"") {
                        let after_label = &part[label_pos + 12..];
                        if let Some(end_quote) = after_label.find('"') {
                            label = after_label[..end_quote].to_string();
                            label = Self::clean_menu_label(&label);
                        }
                    }
                }

                if !label.is_empty() {
                    items.push(MenuItem {
                        id,
                        label,
                        enabled: true,
                        visible: true,
                    });
                }
            }
        }

        items
    }

    fn clean_menu_label(label: &str) -> String {
        // Remove underscores and decode UTF-8 sequences
        let label = label.replace('_', "");
        label.replace("\\342\\200\\246", "…")
    }

    fn show_dynamic_menu(
        button: &gtk::Button,
        service: &str,
        path: &str,
        menu_items: Vec<MenuItem>,
        x: f64,
        y: f64,
    ) {
        let menu_model = gio::Menu::new();
        let actions = gio::SimpleActionGroup::new();

        for item in menu_items {
            let action_name = format!("item-{}", item.id);
            menu_model.append(Some(&item.label), Some(&format!("tray.{}", action_name)));

            let action = gio::SimpleAction::new(&action_name, None);
            let svc = service.to_string();
            let pth = path.to_string();
            let item_id = item.id;

            action.connect_activate(move |_, _| {
                let svc = svc.clone();
                let pth = pth.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        Self::trigger_menu_item(&svc, &pth, item_id).await;
                    });
                });
            });

            actions.add_action(&action);
        }

        button.insert_action_group("tray", Some(&actions));

        let menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        menu.set_parent(button);
        menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        menu.popup();
    }

    async fn trigger_menu_item(service: &str, path: &str, item_id: i32) {
        use tokio::process::Command;

        let menu_path = Self::get_menu_path(service, path)
            .await
            .unwrap_or_else(|| "/MenuBar".to_string());

        let _ = Command::new("busctl")
            .args(&[
                "--user",
                "call",
                service,
                &menu_path,
                "com.canonical.dbusmenu",
                "Event",
                "isvu",
                &item_id.to_string(),
                "clicked",
                "i",
                "0",
                "0",
            ])
            .output()
            .await;
    }

    fn show_fallback_menu(button: &gtk::Button, service: &str, path: &str, x: f64, y: f64) {
        let menu_model = gio::Menu::new();
        let actions = gio::SimpleActionGroup::new();

        menu_model.append(Some("Open"), Some("tray.activate"));
        let activate_action = gio::SimpleAction::new("activate", None);
        let svc = service.to_string();
        let pth = path.to_string();
        activate_action.connect_activate(move |_, _| {
            let svc = svc.clone();
            let pth = pth.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    use tokio::process::Command;
                    let _ = Command::new("busctl")
                        .args(&[
                            "--user",
                            "call",
                            &svc,
                            &pth,
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
        });
        actions.add_action(&activate_action);

        button.insert_action_group("tray", Some(&actions));

        let menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        menu.set_parent(button);
        menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        menu.popup();
    }
}
