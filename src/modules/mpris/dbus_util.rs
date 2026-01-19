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
}

pub fn init_dbus() -> DbusContext {
    let conn = dbus::blocking::Connection::new_session().expect("DBus session failed");

    // let (tx, rx) = std::sync::mpsc::channel();

    // register_name_owner_match(&conn, tx.clone());
    // register_properties_match(&conn, tx);

    DbusContext { conn /*, rx*/ }
}

/* pub fn get_active_player(conn: &Connection, interval_ms: Option<u64>) -> String {
    // Connect to session bus
    let interval: u64 = interval_ms.unwrap_or(200);
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
            Err(_) => break "No Player".to_string(), // Return No Player
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
}*/

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
