// ============ modules/tray/widget.rs ============
use crate::modules::tray::watcher::start_tray_watcher;
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

enum TrayUpdate {
    Add(TrayItem),
    Remove(String),
    UpdateIcon(String),
    UpdateStatus(String),
}

// Shared map of currently registered tray items, keyed by bus name.
type ItemMap = Arc<Mutex<HashMap<String, TrayItem>>>;

impl TrayWidget {
    pub fn new(config: TrayConfig) -> Self {
        std::thread::spawn(|| {
            let _ = start_tray_watcher();
        });

        // Give the watcher a moment to acquire its D-Bus name before we connect.
        std::thread::sleep(Duration::from_millis(200));

        let container = gtk::Box::new(gtk::Orientation::Horizontal, config.spacing);
        container.add_css_class("tray");
        container.add_css_class("module");

        let items: ItemMap = Arc::new(Mutex::new(HashMap::new()));
        let items_ui = Arc::clone(&items);

        let (tx, rx) = mpsc::channel::<TrayUpdate>();
        Self::start_sni_host(items, tx);

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
                    TrayUpdate::UpdateIcon(bus_name) => {
                        if let Some(path) = Self::lookup_path(&items_ui, &bus_name) {
                            Self::refresh_icon(
                                &container_clone,
                                &bus_name,
                                &path,
                                config_clone.icon_size,
                            );
                        }
                    }
                    TrayUpdate::UpdateStatus(bus_name) => {
                        if let Some(path) = Self::lookup_path(&items_ui, &bus_name) {
                            Self::refresh_status(&container_clone, &bus_name, &path);
                        }
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

    fn lookup_path(items: &ItemMap, bus_name: &str) -> Option<String> {
        items
            .lock()
            .unwrap()
            .get(bus_name)
            .map(|i| i.object_path.clone())
    }

    fn start_sni_host(items: ItemMap, tx: mpsc::Sender<TrayUpdate>) {
        std::thread::spawn(move || {
            let conn = match Connection::new_session() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[tray] D-Bus connection failed: {}", e);
                    return;
                }
            };

            // Register ourselves as a StatusNotifierHost so the watcher knows
            // to send us item registrations.
            let pid = std::process::id();
            let host_bus = format!("org.kde.StatusNotifierHost-{}-0", pid);
            let host_path = "/StatusNotifierHost/0".to_string();

            if let Err(e) = conn.request_name(&host_bus, false, false, false) {
                eprintln!("[tray] couldn't acquire host name: {}", e);
            }
            if let Err(e) = Self::register_host(&conn, &host_path) {
                eprintln!("[tray] couldn't register with watcher: {}", e);
                return;
            }

            // Populate initial items that were registered before we started.
            match Self::get_registered_items(&conn) {
                Ok(services) => {
                    for svc in services {
                        if let Some(item) = Self::create_tray_item(&conn, &svc) {
                            items
                                .lock()
                                .unwrap()
                                .insert(item.bus_name.clone(), item.clone());
                            let _ = tx.send(TrayUpdate::Add(item));
                        }
                    }
                }
                Err(e) => eprintln!("[tray] couldn't fetch initial items: {}", e),
            }

            // New item appeared.
            let items_reg = items.clone();
            let tx_reg = tx.clone();
            conn.add_match(
                MatchRule::new_signal(
                    "org.kde.StatusNotifierWatcher",
                    "StatusNotifierItemRegistered",
                ),
                move |_: (), _, msg| {
                    if let Some(svc) = msg.get1::<String>()
                        && let Ok(conn) = Connection::new_session()
                        && let Some(item) = Self::create_tray_item(&conn, &svc)
                    {
                        items_reg
                            .lock()
                            .unwrap()
                            .insert(item.bus_name.clone(), item.clone());
                        let _ = tx_reg.send(TrayUpdate::Add(item));
                    }
                    true
                },
            )
            .expect("[tray] failed to watch ItemRegistered");

            // Item went away.
            let items_unreg = items.clone();
            let tx_unreg = tx.clone();
            conn.add_match(
                MatchRule::new_signal(
                    "org.kde.StatusNotifierWatcher",
                    "StatusNotifierItemUnregistered",
                ),
                move |_: (), _, msg| {
                    if let Some(svc) = msg.get1::<String>() {
                        let (bus_name, _) = Self::parse_service_string(&svc);
                        if items_unreg.lock().unwrap().remove(&bus_name).is_some() {
                            let _ = tx_unreg.send(TrayUpdate::Remove(bus_name));
                        }
                    }
                    true
                },
            )
            .expect("[tray] failed to watch ItemUnregistered");

            // Icon changed — covers NewIcon, NewOverlayIcon, NewAttentionIcon, NewTitle
            // on both the KDE and Ayatana interfaces in one loop.
            let icon_signals = ["NewIcon", "NewOverlayIcon", "NewAttentionIcon", "NewTitle"];
            let ifaces = [
                "org.kde.StatusNotifierItem",
                "org.ayatana.StatusNotifierItem",
            ];
            for signal in icon_signals {
                for iface in ifaces {
                    let tx_icon = tx.clone();
                    if let Err(e) = conn.add_match(
                        MatchRule::new_signal(iface, signal),
                        move |_: (), _, msg| {
                            if let Some(sender) = msg.sender() {
                                let _ = tx_icon.send(TrayUpdate::UpdateIcon(sender.to_string()));
                            }
                            true
                        },
                    ) {
                        eprintln!("[tray] couldn't watch {iface}::{signal}: {e}");
                    }
                }
            }

            // Status changed (affects the CSS class on the button).
            for iface in ifaces {
                let tx_status = tx.clone();
                if let Err(e) = conn.add_match(
                    MatchRule::new_signal(iface, "NewStatus"),
                    move |_: (), _, msg| {
                        if let Some(sender) = msg.sender() {
                            let _ = tx_status.send(TrayUpdate::UpdateStatus(sender.to_string()));
                        }
                        true
                    },
                ) {
                    eprintln!("[tray] couldn't watch {iface}::NewStatus: {e}");
                }
            }

            loop {
                if let Err(e) = conn.process(Duration::from_secs(1)) {
                    eprintln!("[tray] event loop error: {}", e);
                    break;
                }
            }
        });
    }

    // Walk the container's children to find the button that owns a given bus name.
    fn find_button(container: &gtk::Box, bus_name: &str) -> Option<gtk::Button> {
        let mut child = container.first_child();
        while let Some(widget) = child {
            if let Some(btn) = widget.downcast_ref::<gtk::Button>()
                && btn.widget_name() == bus_name
            {
                return Some(btn.clone());
            }
            child = widget.next_sibling();
        }
        None
    }

    // Re-fetch IconName from the item over D-Bus and update the GtkImage.
    // The D-Bus call runs on a worker thread; the widget update happens back on
    // the main thread once the result arrives.
    fn refresh_icon(container: &gtk::Box, bus_name: &str, object_path: &str, icon_size: i32) {
        let Some(button) = Self::find_button(container, bus_name) else {
            return;
        };

        let bus_name = bus_name.to_string();
        let object_path = object_path.to_string();
        let (tx, rx) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            if let Ok(conn) = Connection::new_session() {
                let icon = Self::get_sni_property(&conn, &bus_name, &object_path, "IconName")
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| Self::guess_icon_from_name(&bus_name));
                let _ = tx.send(icon);
            }
        });

        glib::timeout_add_local_once(Duration::from_millis(50), move || {
            if let Ok(icon_name) = rx.try_recv()
                && let Some(image) = button.child().and_then(|c| c.downcast::<gtk::Image>().ok())
            {
                image.set_icon_name(Some(&icon_name));
                image.set_pixel_size(icon_size);
            }
        });
    }

    // Re-fetch Status and swap the CSS class on the button.
    fn refresh_status(container: &gtk::Box, bus_name: &str, object_path: &str) {
        let Some(button) = Self::find_button(container, bus_name) else {
            return;
        };

        let bus_name = bus_name.to_string();
        let object_path = object_path.to_string();
        let (tx, rx) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            if let Ok(conn) = Connection::new_session()
                && let Some(status) =
                    Self::get_sni_property(&conn, &bus_name, &object_path, "Status")
            {
                let _ = tx.send(status);
            }
        });

        glib::timeout_add_local_once(Duration::from_millis(50), move || {
            if let Ok(status) = rx.try_recv() {
                for cls in ["active", "passive", "needsattention"] {
                    button.remove_css_class(cls);
                }
                button.add_css_class(&status.to_lowercase());
            }
        });
    }

    fn register_host(conn: &Connection, object_path: &str) -> Result<(), dbus::Error> {
        conn.with_proxy(
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            Duration::from_secs(5),
        )
        .method_call(
            "org.kde.StatusNotifierWatcher",
            "RegisterStatusNotifierHost",
            (object_path,),
        )
    }

    fn get_registered_items(conn: &Connection) -> Result<Vec<String>, dbus::Error> {
        use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
        conn.with_proxy(
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            Duration::from_secs(5),
        )
        .get(
            "org.kde.StatusNotifierWatcher",
            "RegisteredStatusNotifierItems",
        )
    }

    // The watcher sends items as either "bus.name/object/path" or just "bus.name".
    fn parse_service_string(service_str: &str) -> (String, String) {
        match service_str.find('/') {
            Some(pos) => (
                service_str[..pos].to_string(),
                service_str[pos..].to_string(),
            ),
            None => (service_str.to_string(), "/StatusNotifierItem".to_string()),
        }
    }

    fn create_tray_item(conn: &Connection, service_str: &str) -> Option<TrayItem> {
        let (bus_name, object_path) = Self::parse_service_string(service_str);

        let title = Self::get_sni_property(conn, &bus_name, &object_path, "Title")
            .unwrap_or_else(|| bus_name.clone());
        let icon_name = Self::get_sni_property(conn, &bus_name, &object_path, "IconName")
            .unwrap_or_else(|| Self::guess_icon_from_name(&bus_name));
        let menu_path = Self::get_sni_property(conn, &bus_name, &object_path, "Menu")
            .unwrap_or_else(|| "/MenuBar".to_string());

        Some(TrayItem {
            bus_name,
            object_path,
            title,
            icon_name,
            menu_path,
        })
    }

    // Try both the KDE and Ayatana SNI interfaces; return the first hit.
    fn get_sni_property(
        conn: &Connection,
        bus_name: &str,
        path: &str,
        prop: &str,
    ) -> Option<String> {
        use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
        let proxy = conn.with_proxy(bus_name, path, Duration::from_secs(2));

        for iface in [
            "org.kde.StatusNotifierItem",
            "org.ayatana.StatusNotifierItem",
        ] {
            if let Ok(value) = proxy.get::<String>(iface, prop) {
                return Some(value);
            }
            // The Menu property is a D-Bus object path, not a plain string.
            if prop == "Menu"
                && let Ok(value) = proxy.get::<dbus::Path>(iface, prop)
            {
                return Some(value.to_string());
            }
        }
        None
    }

    // Last-resort icon lookup when the item doesn't advertise one.
    fn guess_icon_from_name(name: &str) -> String {
        let lower = name.to_lowercase();
        let icon = if lower.contains("discord") {
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
            &lower
        };
        icon.to_string()
    }

    fn add_tray_button(container: &gtk::Box, item: &TrayItem, config: &TrayConfig) {
        let button = gtk::Button::new();
        button.add_css_class("tray-item");
        button.set_widget_name(&item.bus_name);

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

        // Left-click activates the item (e.g. opens the app window).
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
                        if proxy
                            .method_call::<(), _, _, _>(iface, "Activate", (0i32, 0i32))
                            .is_ok()
                        {
                            break;
                        }
                    }
                }
            });
        });

        // Right-click shows the context menu.
        let right_click = gtk::GestureClick::new();
        right_click.set_button(3);
        let bus_name_rc = item.bus_name.clone();
        let path_rc = item.object_path.clone();
        let menu_path_rc = item.menu_path.clone();
        right_click.connect_pressed(move |gesture, _, x, y| {
            if let Some(widget) = gesture.widget()
                && let Ok(btn) = widget.downcast::<gtk::Button>()
            {
                Self::show_context_menu(&btn, &bus_name_rc, &path_rc, &menu_path_rc, x, y);
            }
        });
        button.add_controller(right_click);

        container.append(&button);
    }

    fn remove_tray_button(container: &gtk::Box, bus_name: &str) {
        let mut child = container.first_child();
        while let Some(widget) = child {
            if let Some(btn) = widget.downcast_ref::<gtk::Button>()
                && btn.widget_name() == bus_name
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

        // Make sure the menu actually speaks com.canonical.dbusmenu before calling GetLayout.
        let proxy = conn.with_proxy(bus_name, menu_path, Duration::from_secs(2));
        let introspect: Result<(String,), dbus::Error> =
            proxy.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ());

        match introspect {
            Ok((xml,)) if xml.contains("com.canonical.dbusmenu") => {}
            _ => return Vec::new(),
        }

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
        items
    }

    fn parse_menu_reply(reply: &Message, items: &mut Vec<MenuItem>) {
        let mut iter = reply.iter_init();
        if iter.next()
            && let Some(layout) = iter.get_refarg()
        {
            Self::parse_menu_item(&*layout, items);
        }
    }

    fn parse_menu_item(arg: &dyn RefArg, items: &mut Vec<MenuItem>) {
        let Some(struct_iter) = arg.as_iter() else {
            return;
        };
        let elements: Vec<&dyn RefArg> = struct_iter.collect();
        if elements.len() < 3 {
            return;
        }

        let id = elements[0].as_i64().unwrap_or(0) as i32;
        let mut label = String::new();
        let mut is_separator = false;
        let mut enabled = true;
        let mut visible = true;

        // Properties come as a flat [key, value, key, value, ...] array.
        if let Some(props_iter) = elements[1].as_iter() {
            let props: Vec<&dyn RefArg> = props_iter.collect();
            let mut i = 0;
            while i + 1 < props.len() {
                if let Some(key) = props[i].as_str() {
                    let val = props[i + 1];
                    match key {
                        "label" => {
                            if let Some(text) = val
                                .as_iter()
                                .and_then(|mut it| it.next())
                                .and_then(|v| v.as_str())
                            {
                                label = Self::clean_menu_label(text);
                            }
                        }
                        "type" => {
                            if let Some(t) = val
                                .as_iter()
                                .and_then(|mut it| it.next())
                                .and_then(|v| v.as_str())
                            {
                                is_separator = t == "separator";
                            }
                        }
                        "enabled" => {
                            if let Some(e) = val
                                .as_iter()
                                .and_then(|mut it| it.next())
                                .and_then(|v| v.as_i64())
                            {
                                enabled = e != 0;
                            }
                        }
                        "visible" => {
                            if let Some(v) = val
                                .as_iter()
                                .and_then(|mut it| it.next())
                                .and_then(|v| v.as_i64())
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

        if !is_separator && !label.is_empty() {
            items.push(MenuItem {
                id,
                label,
                enabled,
                visible,
            });
        }

        // Recurse into children, unwrapping D-Bus Variants as needed.
        if let Some(children_iter) = elements[2].as_iter() {
            for child in children_iter {
                if child.signature() == "v" {
                    if let Some(inner) = child.as_iter().and_then(|mut it| it.next()) {
                        Self::parse_menu_item(inner, items);
                    }
                } else {
                    Self::parse_menu_item(child, items);
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

        for item in items.into_iter().filter(|i| i.visible) {
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
                std::thread::spawn(move || Self::trigger_menu_item(&b, &m, id));
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
        let bn = bus_name.to_string();
        let p = path.to_string();
        activate.connect_activate(move |_, _| {
            let b = bn.clone();
            let p = p.clone();
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
