// ============ modules/tray.rs ============
// Waybar-compatible SNI Host implementation

use crate::modules::tray_watcher::start_tray_watcher;
use dbus::Message;
use dbus::arg::{RefArg, Variant};
use dbus::blocking::{BlockingSender, Connection};
use dbus::message::MatchRule;
use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

// Like Waybar's Item
#[derive(Clone, Debug)]
struct TrayItem {
    bus_name: String,
    object_path: String,
    title: String,
    icon_name: String,
    menu_path: String,
}

#[derive(Clone, Debug)]
struct MenuItem {
    id: i32,
    label: String,
    enabled: bool,
    visible: bool,
}

#[derive(Clone, Debug)]
enum TrayUpdate {
    Add(TrayItem),
    Remove(String), // bus_name
}

impl TrayWidget {
    pub fn new(config: TrayConfig) -> Self {
        // Start watcher (like Waybar's Tray constructor)
        std::thread::spawn(|| {
            let _ = start_tray_watcher();
        });

        std::thread::sleep(Duration::from_millis(200));

        let container = gtk::Box::new(gtk::Orientation::Horizontal, config.spacing);
        container.add_css_class("tray");
        container.add_css_class("module");

        let tray_items: Arc<Mutex<HashMap<String, TrayItem>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (tx, rx) = mpsc::channel::<TrayUpdate>();

        // Start Host (like Waybar's Host constructor)
        Self::start_sni_host(tray_items, tx);

        // Handle UI updates on main thread
        let container_clone = container.clone();
        let config_clone = config.clone();

        glib::timeout_add_local(Duration::from_millis(100), move || {
            while let Ok(update) = rx.try_recv() {
                match update {
                    TrayUpdate::Add(item) => {
                        Self::add_tray_button(&container_clone, &item, &config_clone);
                    }
                    TrayUpdate::Remove(bus_name) => {
                        Self::remove_tray_button(&container_clone, &bus_name);
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

    // Like Waybar's Host::busAcquired
    fn start_sni_host(
        tray_items: Arc<Mutex<HashMap<String, TrayItem>>>,
        tx: mpsc::Sender<TrayUpdate>,
    ) {
        std::thread::spawn(move || {
            let conn = match Connection::new_session() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[TRAY HOST] Failed to connect: {}", e);
                    return;
                }
            };

            // Generate unique host name (like Waybar: org.kde.StatusNotifierHost-{pid}-{id})
            let pid = std::process::id();
            let host_id = 0;
            let bus_name = format!("org.kde.StatusNotifierHost-{}-{}", pid, host_id);
            let object_path = format!("/StatusNotifierHost/{}", host_id);

            // Own the host name
            match conn.request_name(&bus_name, false, false, false) {
                Ok(_) => {} // println!("[TRAY HOST] ✓ Acquired: {}", bus_name),
                Err(e) => eprintln!("[TRAY HOST] ✗ Failed to acquire name: {}", e),
            }

            // Register with watcher (like Host::registerHost)
            if let Err(e) = Self::register_host(&conn, &object_path) {
                eprintln!("[TRAY HOST] Failed to register: {}", e);
                return;
            }

            // Get initial items (like Host::registerHost callback)
            match Self::get_registered_items(&conn) {
                Ok(items) => {
                    // println!("[TRAY HOST] Initial items: {}", items.len());
                    for service_str in items {
                        if let Some(item) = Self::create_tray_item(&conn, &service_str) {
                            let bn = item.bus_name.clone();
                            tray_items.lock().unwrap().insert(bn, item.clone());
                            let _ = tx.send(TrayUpdate::Add(item));
                        }
                    }
                }
                Err(e) => eprintln!("[TRAY HOST] Failed to get initial items: {}", e),
            }

            // Listen for ItemRegistered (like Host::itemRegistered signal)
            let items_clone = tray_items.clone();
            let tx_clone = tx.clone();

            let rule = MatchRule::new_signal(
                "org.kde.StatusNotifierWatcher",
                "StatusNotifierItemRegistered",
            );
            conn.add_match(rule, move |_: (), _, msg| {
                if let Some(service_str) = msg.get1::<String>() {
                    // println!("[TRAY HOST] → ItemRegistered: {}", service_str);
                    // Create new connection for item creation
                    if let Ok(conn) = Connection::new_session()
                        && let Some(item) = Self::create_tray_item(&conn, &service_str)
                    {
                        let bn = item.bus_name.clone();
                        items_clone.lock().unwrap().insert(bn, item.clone());
                        let _ = tx_clone.send(TrayUpdate::Add(item));
                    }
                }
                true
            })
            .expect("Failed to add ItemRegistered match");

            // Listen for ItemUnregistered (like Host::itemUnregistered signal)
            let items_clone = tray_items.clone();
            let tx_clone = tx.clone();

            let rule = MatchRule::new_signal(
                "org.kde.StatusNotifierWatcher",
                "StatusNotifierItemUnregistered",
            );
            conn.add_match(rule, move |_: (), _, msg| {
                if let Some(service_str) = msg.get1::<String>() {
                    // println!("[TRAY HOST] → ItemUnregistered: {}", service_str);
                    let (bus_name, _) = Self::parse_service_string(&service_str);
                    if items_clone.lock().unwrap().remove(&bus_name).is_some() {
                        let _ = tx_clone.send(TrayUpdate::Remove(bus_name));
                    }
                }
                true
            })
            .expect("Failed to add ItemUnregistered match");

            // println!("[TRAY HOST] Listening for items...");

            // Process D-Bus messages
            loop {
                if let Err(e) = conn.process(Duration::from_secs(1)) {
                    eprintln!("[TRAY HOST] Process error: {}", e);
                    break;
                }
            }
        });
    }

    // Like Waybar's Host::registerHost
    fn register_host(conn: &Connection, object_path: &str) -> Result<(), dbus::Error> {
        let proxy = conn.with_proxy(
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            Duration::from_secs(5),
        );

        proxy.method_call(
            "org.kde.StatusNotifierWatcher",
            "RegisterStatusNotifierHost",
            (object_path,),
        )
    }

    fn get_registered_items(conn: &Connection) -> Result<Vec<String>, dbus::Error> {
        let proxy = conn.with_proxy(
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            Duration::from_secs(5),
        );

        use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
        proxy.get(
            "org.kde.StatusNotifierWatcher",
            "RegisteredStatusNotifierItems",
        )
    }

    // Like Waybar's Host::getBusNameAndObjectPath
    fn parse_service_string(service_str: &str) -> (String, String) {
        if let Some(pos) = service_str.find('/') {
            (
                service_str[..pos].to_string(),
                service_str[pos..].to_string(),
            )
        } else {
            (service_str.to_string(), "/StatusNotifierItem".to_string())
        }
    }

    fn create_tray_item(conn: &Connection, service_str: &str) -> Option<TrayItem> {
        let (bus_name, object_path) = Self::parse_service_string(service_str);

        let title = Self::get_sni_property(conn, &bus_name, &object_path, "Title")
            .unwrap_or_else(|| bus_name.clone());

        let icon_name = Self::get_sni_property(conn, &bus_name, &object_path, "IconName")
            .unwrap_or_else(|| Self::guess_icon_from_name(&bus_name));

        /*let icon_theme_path =
        Self::get_sni_property(conn, &bus_name, &object_path, "IconThemePath"); */

        let menu_path = Self::get_sni_property(conn, &bus_name, &object_path, "Menu")
            .unwrap_or_else(|| "/MenuBar".to_string());

        /* println!(
            "[TRAY] Added: {} (icon: '{}', theme_path: {:?})",
            title, icon_name, icon_theme_path
        );*/

        Some(TrayItem {
            bus_name,
            object_path,
            title,
            icon_name,
            menu_path,
        })
    }

    fn get_sni_property(
        conn: &Connection,
        bus_name: &str,
        path: &str,
        prop: &str,
    ) -> Option<String> {
        let proxy = conn.with_proxy(bus_name, path, Duration::from_secs(2));
        use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;

        // Try both KDE and Ayatana interfaces (like Waybar)
        for iface in [
            "org.kde.StatusNotifierItem",
            "org.ayatana.StatusNotifierItem",
        ] {
            if let Ok(value) = proxy.get::<String>(iface, prop) {
                return Some(value);
            }

            // Menu property might be Path type
            if prop == "Menu"
                && let Ok(value) = proxy.get::<dbus::Path>(iface, prop)
            {
                return Some(value.to_string());
            }
        }

        None
    }

    fn guess_icon_from_name(name: &str) -> String {
        let lower = name.to_lowercase();

        if lower.contains("discord") {
            "discord"
        } else if lower.contains("spotify") {
            "spotify"
        } else if lower.contains("telegram") {
            "telegram"
        } else if lower.contains("steam") {
            "steam"
        } else if lower.contains("network") {
            "network-wireless"
        } else if lower.contains("bluetooth") {
            "bluetooth"
        } else if lower.contains("audio") || lower.contains("volume") {
            "audio-volume-high"
        } else if lower.contains("tuxedo") {
            "computer"
        } else {
            "application-x-executable"
        }
        .to_string()
    }

    fn add_tray_button(container: &gtk::Box, item: &TrayItem, config: &TrayConfig) {
        let button = gtk::Button::new();
        button.add_css_class("tray-item");
        button.set_widget_name(&item.bus_name);

        // Get Status property for CSS classes (like Waybar)
        if let Ok(conn) = Connection::new_session()
            && let Some(status) =
                Self::get_sni_property(&conn, &item.bus_name, &item.object_path, "Status")
        {
            button.add_css_class(&status.to_lowercase());
        }

        let icon = gtk::Image::from_icon_name(&item.icon_name);
        icon.set_pixel_size(config.icon_size);
        button.set_child(Some(&icon));
        button.set_tooltip_text(Some(&item.title));

        // Left click - Activate (like Waybar's Item::handleClick)
        let bus_name = item.bus_name.clone();
        let path = item.object_path.clone();
        button.connect_clicked(move |_| {
            let bn = bus_name.clone();
            let p = path.clone();
            std::thread::spawn(move || {
                if let Ok(conn) = Connection::new_session() {
                    let proxy = conn.with_proxy(&bn, &p, Duration::from_secs(5));

                    for iface in [
                        "org.kde.StatusNotifierItem",
                        "org.ayatana.StatusNotifierItem",
                    ] {
                        let result: Result<(), dbus::Error> =
                            proxy.method_call(iface, "Activate", (0i32, 0i32));
                        if result.is_ok() {
                            break;
                        }
                    }
                }
            });
        });

        // Right click - Context menu
        let right_click = gtk::GestureClick::new();
        right_click.set_button(3);
        let bus_name_rc = item.bus_name.clone();
        let path_rc = item.object_path.clone();
        let menu_path_rc = item.menu_path.clone();

        right_click.connect_pressed(move |gesture, _n, x, y| {
            if let Some(widget) = gesture.widget()
                && let Ok(button) = widget.downcast::<gtk::Button>()
            {
                Self::show_context_menu(&button, &bus_name_rc, &path_rc, &menu_path_rc, x, y);
            }
        });
        button.add_controller(right_click);

        container.append(&button);
    }

    fn remove_tray_button(container: &gtk::Box, bus_name: &str) {
        let mut child = container.first_child();
        while let Some(widget) = child {
            if let Some(button) = widget.downcast_ref::<gtk::Button>()
                && button.widget_name() == bus_name
            {
                container.remove(&widget);
                break;
            }
            child = widget.next_sibling();
        }
    }

    fn show_context_menu(
        button: &gtk::Button,
        bus_name: &str,
        path: &str,
        menu_path: &str,
        x: f64,
        y: f64,
    ) {
        let bus_name = bus_name.to_string();
        let path = path.to_string();
        let menu_path = menu_path.to_string();
        let button_weak = button.downgrade();

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let items = Self::get_dbus_menu(&bus_name, &menu_path);
            let _ = tx.send((bus_name, path, menu_path, items));
        });

        glib::timeout_add_local(Duration::from_millis(10), move || {
            if let Ok((bn, p, mp, items)) = rx.try_recv() {
                if let Some(btn) = button_weak.upgrade() {
                    if items.is_empty() {
                        Self::show_fallback_menu(&btn, &bn, &p, x, y);
                    } else {
                        Self::show_dynamic_menu(&btn, &bn, &mp, items, x, y);
                    }
                }
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    fn get_dbus_menu(bus_name: &str, menu_path: &str) -> Vec<MenuItem> {
        let conn = match Connection::new_session() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // Verify menu interface exists
        let proxy = conn.with_proxy(bus_name, menu_path, Duration::from_secs(2));
        let introspect: Result<(String,), dbus::Error> =
            proxy.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ());

        match introspect {
            Ok((xml,)) => {
                if !xml.contains("com.canonical.dbusmenu") {
                    return Vec::new();
                }
            }
            Err(_) => return Vec::new(),
        }

        // Call GetLayout
        let mut msg =
            Message::new_method_call(bus_name, menu_path, "com.canonical.dbusmenu", "GetLayout")
                .unwrap();

        msg = msg.append3(0i32, -1i32, Vec::<String>::new());

        let reply = match conn.send_with_reply_and_block(msg, Duration::from_secs(5)) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut items = Vec::new();
        Self::parse_menu_reply(&reply, &mut items);

        /*if !items.is_empty() {
            println!("[TRAY] Menu loaded: {} items", items.len());
        }*/

        items
    }

    fn parse_menu_reply(reply: &Message, items: &mut Vec<MenuItem>) {
        let mut iter = reply.iter_init();

        if !iter.next() {
            return;
        }

        if let Some(layout) = iter.get_refarg() {
            Self::parse_menu_item(&*layout, items);
        }
    }

    fn parse_menu_item(arg: &dyn RefArg, items: &mut Vec<MenuItem>) {
        if let Some(struct_iter) = arg.as_iter() {
            let elements: Vec<&dyn RefArg> = struct_iter.collect();

            if elements.len() < 3 {
                return;
            }

            let id = elements[0].as_i64().unwrap_or(0) as i32;
            let mut label = String::new();
            let mut is_separator = false;
            let mut enabled = true;
            let mut visible = true;

            // Parse properties (flat array: key, value, key, value, ...)
            if let Some(props_iter) = elements[1].as_iter() {
                let props: Vec<&dyn RefArg> = props_iter.collect();

                let mut i = 0;
                while i + 1 < props.len() {
                    let key_arg = props[i];
                    let val_arg = props[i + 1];

                    if let Some(key) = key_arg.as_str() {
                        match key {
                            "label" => {
                                if let Some(mut v_iter) = val_arg.as_iter()
                                    && let Some(text) = v_iter.next().and_then(|v| v.as_str())
                                {
                                    label = Self::clean_menu_label(text);
                                }
                            }
                            "type" => {
                                if let Some(mut v_iter) = val_arg.as_iter()
                                    && let Some(t) = v_iter.next().and_then(|v| v.as_str())
                                {
                                    is_separator = t == "separator";
                                }
                            }
                            "enabled" => {
                                if let Some(mut v_iter) = val_arg.as_iter()
                                    && let Some(e) = v_iter.next().and_then(|v| v.as_i64())
                                {
                                    enabled = e != 0;
                                }
                            }
                            "visible" => {
                                if let Some(mut v_iter) = val_arg.as_iter()
                                    && let Some(v) = v_iter.next().and_then(|v| v.as_i64())
                                {
                                    visible = v != 0;
                                }
                            }
                            _ => {}
                        }
                    }

                    i += 2;
                }
            }

            // Add item if valid
            if !is_separator && !label.is_empty() {
                items.push(MenuItem {
                    id,
                    label,
                    enabled,
                    visible,
                });
            }

            // Parse children (unwrap Variants)
            if let Some(children_iter) = elements[2].as_iter() {
                let children: Vec<&dyn RefArg> = children_iter.collect();
                for child in children.iter() {
                    if child.signature() == "v" {
                        if let Some(mut variant_iter) = child.as_iter()
                            && let Some(inner) = variant_iter.next()
                        {
                            Self::parse_menu_item(inner, items);
                        }
                    } else {
                        Self::parse_menu_item(child, items);
                    }
                }
            }
        }
    }

    fn clean_menu_label(label: &str) -> String {
        label.replace('_', "")
    }

    fn show_dynamic_menu(
        button: &gtk::Button,
        bus_name: &str,
        menu_path: &str,
        items: Vec<MenuItem>,
        x: f64,
        y: f64,
    ) {
        let menu = gio::Menu::new();
        let actions = gio::SimpleActionGroup::new();

        for item in items {
            if !item.visible {
                continue;
            }

            let action_name = format!("item-{}", item.id);
            menu.append(Some(&item.label), Some(&format!("tray.{}", action_name)));

            let action = gio::SimpleAction::new(&action_name, None);
            action.set_enabled(item.enabled);

            let bn = bus_name.to_string();
            let mp = menu_path.to_string();
            let id = item.id;

            action.connect_activate(move |_, _| {
                let b = bn.clone();
                let m = mp.clone();
                std::thread::spawn(move || {
                    Self::trigger_menu_item(&b, &m, id);
                });
            });

            actions.add_action(&action);
        }

        button.insert_action_group("tray", Some(&actions));

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        popover.add_css_class("tray-menu");
        popover.set_parent(button);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.popup();
    }

    fn trigger_menu_item(bus_name: &str, menu_path: &str, item_id: i32) {
        if let Ok(conn) = Connection::new_session() {
            let mut msg =
                Message::new_method_call(bus_name, menu_path, "com.canonical.dbusmenu", "Event")
                    .unwrap();

            msg = msg.append1(item_id);
            msg = msg.append1("clicked");
            msg = msg.append1(Variant(0i32));
            msg = msg.append1(0u32);

            let _ = conn.send_with_reply_and_block(msg, Duration::from_secs(5));
        }
    }

    fn show_fallback_menu(button: &gtk::Button, bus_name: &str, path: &str, x: f64, y: f64) {
        let menu = gio::Menu::new();
        let actions = gio::SimpleActionGroup::new();

        menu.append(Some("Open"), Some("tray.activate"));
        let activate = gio::SimpleAction::new("activate", None);
        let bn1 = bus_name.to_string();
        let p1 = path.to_string();
        activate.connect_activate(move |_, _| {
            let b = bn1.clone();
            let p = p1.clone();
            std::thread::spawn(move || {
                if let Ok(conn) = Connection::new_session() {
                    let proxy = conn.with_proxy(&b, &p, Duration::from_secs(5));
                    let _: Result<(), dbus::Error> =
                        proxy.method_call("org.kde.StatusNotifierItem", "Activate", (0i32, 0i32));
                }
            });
        });
        actions.add_action(&activate);

        button.insert_action_group("tray", Some(&actions));

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        popover.add_css_class("tray-menu");
        popover.set_parent(button);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.popup();
    }
}
