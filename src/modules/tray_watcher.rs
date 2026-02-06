// ============ modules/tray_watcher.rs ============
// StatusNotifierWatcher - Waybar-compatible implementation

use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::message::MatchRule;
use dbus_crossroads::Crossroads;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const WATCHER_BUS: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const WATCHER_IFACE: &str = "org.kde.StatusNotifierWatcher";

#[derive(Clone)]
struct WatchInfo {
    // service: String,
    bus_name: String,
    object_path: String,
}

type ItemMap = Arc<Mutex<HashMap<String, WatchInfo>>>; // key: bus_name, value: WatchInfo
type HostMap = Arc<Mutex<HashMap<String, WatchInfo>>>;

#[derive(Clone)]
struct WatcherData {
    items: ItemMap,
    hosts: HostMap,
}

pub fn start_tray_watcher() -> Result<(), Box<dyn std::error::Error>> {
    // println!("[WATCHER] Starting StatusNotifierWatcher...");

    let conn = Connection::new_session()?;

    // Request name with ALLOW_REPLACEMENT | REPLACE flags (same as Waybar)
    match conn.request_name(WATCHER_BUS, false, true, true) {
        Ok(_) => {} //Ok(_) => print!("[WATCHER] ✓ Acquired: {}", WATCHER_BUS),
        Err(e) => {
            eprintln!("[WATCHER] ✗ Failed: {}", e);
            return Err(e.into());
        }
    }

    let data = WatcherData {
        items: Arc::new(Mutex::new(HashMap::new())),
        hosts: Arc::new(Mutex::new(HashMap::new())),
    };

    // Start name watcher thread
    start_name_watcher(data.clone());

    let mut cr = Crossroads::new();

    let data_clone = data.clone();
    let iface_token = cr.register(WATCHER_IFACE, move |b| {
        let items_prop = data_clone.items.clone();
        let hosts_prop = data_clone.hosts.clone();
        let items_method = data_clone.items.clone();
        let hosts_method = data_clone.hosts.clone();

        // Property: RegisteredStatusNotifierItems
        b.property("RegisteredStatusNotifierItems")
            .get(move |_, _: &mut ()| {
                let items = items_prop.lock().unwrap();
                let list: Vec<String> = items
                    .values()
                    .map(|info| format!("{}{}", info.bus_name, info.object_path))
                    .collect();
                Ok(list)
            });

        // Property: IsStatusNotifierHostRegistered
        b.property("IsStatusNotifierHostRegistered")
            .get(move |_, _: &mut ()| Ok(!hosts_prop.lock().unwrap().is_empty()));

        // Property: ProtocolVersion
        b.property("ProtocolVersion").get(|_, _: &mut ()| Ok(0i32));

        // Method: RegisterStatusNotifierItem
        b.method(
            "RegisterStatusNotifierItem",
            ("service",),
            (),
            move |ctx, _: &mut (), (service,): (String,)| {
                let sender = ctx.message().sender().unwrap().to_string();

                // Parse exactly like Waybar does
                let (bus_name, object_path) = if service.starts_with('/') {
                    // service is object_path, use sender as bus_name
                    (sender.clone(), service.clone())
                } else {
                    // service is bus_name, use default object_path
                    (service.clone(), "/StatusNotifierItem".to_string())
                };

                /* println!(
                    "[WATCHER] RegisterItem: sender='{}', service='{}' -> bus='{}', path='{}'",
                    sender, service, bus_name, object_path
                );*/

                let mut items = items_method.lock().unwrap();

                // Check if already registered (by bus_name)
                if items.contains_key(&bus_name) {
                    // println!("[WATCHER]   Already registered");
                    return Ok(());
                }

                // Create watch info
                let info = WatchInfo {
                    // service: service.clone(),
                    bus_name: bus_name.clone(),
                    object_path: object_path.clone(),
                };

                items.insert(bus_name.clone(), info);
                let _count = items.len();
                drop(items);

                // println!("[WATCHER]   ✓ Registered (total: {})", count);

                // Emit signal with format: bus_name + object_path (like Waybar)
                let signal_arg = format!("{}{}", bus_name, object_path);
                send_signal("StatusNotifierItemRegistered", Some(&signal_arg));

                Ok(())
            },
        );

        // Method: RegisterStatusNotifierHost
        b.method(
            "RegisterStatusNotifierHost",
            ("service",),
            (),
            move |ctx, _: &mut (), (service,): (String,)| {
                let sender = ctx.message().sender().unwrap().to_string();

                let (bus_name, object_path) = if service.starts_with('/') {
                    (sender.clone(), service.clone())
                } else {
                    (service.clone(), "/StatusNotifierHost".to_string())
                };

                /*println!(
                    "[WATCHER] RegisterHost: sender='{}', service='{}' -> bus='{}', path='{}'",
                    sender, service, bus_name, object_path
                );*/

                let mut hosts = hosts_method.lock().unwrap();

                if hosts.contains_key(&bus_name) {
                    // println!("[WATCHER]   Already registered");
                    return Ok(());
                }

                let was_empty = hosts.is_empty();

                let info = WatchInfo {
                    // service: service.clone(),
                    bus_name: bus_name.clone(),
                    object_path: object_path.clone(),
                };

                hosts.insert(bus_name, info);
                // let count = hosts.len();
                drop(hosts);

                // println!("[WATCHER]   ✓ Registered (total: {})", count);

                // Emit signal only if this was the first host (like Waybar)
                if was_empty {
                    send_signal("StatusNotifierHostRegistered", None);
                }

                Ok(())
            },
        );

        // Signals
        b.signal::<(String,), _>("StatusNotifierItemRegistered", ("service",));
        b.signal::<(String,), _>("StatusNotifierItemUnregistered", ("service",));
        b.signal::<(), _>("StatusNotifierHostRegistered", ());
    });

    cr.insert(WATCHER_PATH, &[iface_token], ());

    // println!("[WATCHER] Ready at {}", WATCHER_PATH);
    // println!("[WATCHER] Interface: {}", WATCHER_IFACE);

    cr.serve(&conn)?;

    Ok(())
}

fn start_name_watcher(data: WatcherData) {
    thread::spawn(move || {
        let conn = match Connection::new_session() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[WATCHER] Name watcher failed: {}", e);
                return;
            }
        };

        // Watch NameOwnerChanged like Waybar's g_bus_watch_name
        let rule = MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged");

        conn.add_match(rule, move |_: (), _, msg| {
            if let (Some(name), Some(_old_owner), Some(new_owner)) =
                msg.get3::<String, String, String>()
            {
                // new_owner empty = name released
                if new_owner.is_empty() {
                    handle_name_vanished(&data, &name);
                }
            }
            true
        })
        .expect("Failed to add match");

        // println!("[WATCHER] Name watcher started");

        loop {
            if let Err(e) = conn.process(Duration::from_secs(1)) {
                eprintln!("[WATCHER] Name watcher error: {}", e);
                break;
            }
        }
    });
}

fn handle_name_vanished(data: &WatcherData, name: &str) {
    // Remove items with this bus_name (like Waybar's nameVanished)
    let mut items = data.items.lock().unwrap();
    if let Some(info) = items.remove(name) {
        let signal_arg = format!("{}{}", info.bus_name, info.object_path);
        drop(items);

        /*println!(
            "[WATCHER] Name vanished: '{}' - removing item '{}'",
            name,
            signal_arg
        );*/
        send_signal("StatusNotifierItemUnregistered", Some(&signal_arg));
    } else {
        drop(items);
    }

    // Remove hosts with this bus_name
    let mut hosts = data.hosts.lock().unwrap();
    let had_hosts = !hosts.is_empty();
    hosts.remove(name);
    let has_hosts = !hosts.is_empty();
    drop(hosts);

    // If last host was removed, emit signal (Waybar does this)
    if had_hosts && !has_hosts {
        // println!("[WATCHER] Last host removed");
        // Note: Waybar sets IsHostRegistered to FALSE but doesn't emit signal again
    }
}

fn send_signal(signal_name: &str, arg: Option<&str>) {
    if let Ok(conn) = Connection::new_session() {
        use dbus::{Message, Path};

        let mut msg = Message::signal(
            &Path::from(WATCHER_PATH),
            &WATCHER_IFACE.into(),
            &signal_name.into(),
        );

        if let Some(a) = arg {
            msg = msg.append1(a);
        }

        match conn.send(msg) {
            Ok(_) => {
                /* if let Some(a) = arg {
                    // println!("[WATCHER]   → Signal: {}('{}')", signal_name, a);
                } else {
                    //println!("[WATCHER]   → Signal: {}", signal_name);
                }*/
            }
            Err(e) => eprintln!("[WATCHER]   ✗ Signal failed: {:?}", e),
        }
    }
}
