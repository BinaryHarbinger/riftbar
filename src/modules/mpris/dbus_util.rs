// ============ modules/mpris/widget.rs ============
use dbus::{
    // arg::{RefArg, Variant},
    blocking::Connection,
    // message::{MatchRule, Message},
};
use std::{/*collections::HashMap, sync::mpsc::Sender,*/ time::Duration};

// Structs
pub struct DbusContext {
    pub conn: dbus::blocking::Connection,
    //pub rx: std::sync::mpsc::Receiver<DbusEvent>,
}
// Enums
/* #[derive(Debug)]
pub enum DbusEvent {
    PlayerAppeared(String),
    PlayerVanished(String),
    MetadataChanged(String),
    PlaybackStatusChanged { player: String, status: String },
} */

pub fn init_dbus() -> DbusContext {
    let conn = dbus::blocking::Connection::new_session().expect("DBus session failed");

    // let (tx, rx) = std::sync::mpsc::channel();

    // register_name_owner_match(&conn, tx.clone());
    // register_properties_match(&conn, tx);

    DbusContext { conn /*, rx*/ }
}

/* pub fn process_dbus(ctx: &DbusContext) {
    ctx.conn
        .process(Duration::from_millis(1000))
        .expect("DBus process failed");
}

pub fn try_next_event(ctx: &DbusContext) -> Option<DbusEvent> {
    ctx.rx.try_recv().ok()
}

fn register_name_owner_match(conn: &Connection, tx: Sender<DbusEvent>) {
    let rule = MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged");

    conn.add_match(rule, move |_state: (), _conn, msg: &Message| {
        let (name, old_owner, new_owner): (String, String, String) = match msg.read3() {
            Ok(v) => v,
            Err(_) => return false,
        };

        if !name.starts_with("org.mpris.MediaPlayer2.") {
            return false;
        }

        if old_owner.is_empty() && !new_owner.is_empty() {
            let _ = tx.send(DbusEvent::PlayerAppeared(name));
        } else if !old_owner.is_empty() && new_owner.is_empty() {
            let _ = tx.send(DbusEvent::PlayerVanished(name));
        }

        false
    })
    .expect("add_match NameOwnerChanged failed");
}

fn register_properties_match(conn: &Connection, tx: Sender<DbusEvent>) {
    let rule = MatchRule::new_signal("org.freedesktop.DBus.Properties", "PropertiesChanged");

    conn.add_match(rule, move |_state: (), _conn, msg: &Message| {
        let (iface, _changed, _invalidated): (
            String,
            HashMap<String, Variant<Box<dyn RefArg>>>,
            Vec<String>,
        ) = match msg.read3() {
            Ok(v) => v,
            Err(_) => return false,
        };

        if iface == "org.mpris.MediaPlayer2.Player"
            && let Some(sender) = msg.sender()
        {
            let _ = tx.send(DbusEvent::MetadataChanged(sender.to_string()));
        }

        false
    })
    .expect("add_match PropertiesChanged failed");
}
*/
/// Wait for an active MPRIS player and return true
pub fn wait_for_active_player(conn: &Connection, interval_ms: Option<u64>) -> String {
    // Connect to session bus
    let interval: u64 = interval_ms.unwrap_or(1000);
    // println!("[DBUS UTIL]: Waiting for player...");

    loop {
        // Process incoming messages with 1 second timeout
        conn.process(Duration::from_millis(interval)).unwrap();

        // Create a proxy to the D-Bus daemon
        let proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            Duration::from_millis(interval / 2),
        );

        // Call ListNames method directly
        let result: Result<(Vec<String>,), _> =
            proxy.method_call("org.freedesktop.DBus", "ListNames", ());

        let names = match result {
            Ok((names,)) => names,
            Err(_) => continue, // skip if error
        };

        // Check if any active MPRIS player exists
        if let Some(player) = names
            .iter()
            .find(|name| name.starts_with("org.mpris.MediaPlayer2."))
        {
            // println!("[DBUS UTIL]: Player detected: {}", player);
            return player.to_string();
        }
    }
}
