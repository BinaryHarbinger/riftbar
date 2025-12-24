// ============ modules/tray/host.rs ============
use zbus::{Connection, message::Body};

pub struct StatusNotifierHost {
    connection: Connection,
}

impl StatusNotifierHost {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("[HOST] Connecting to session bus...");

        let connection = Connection::session().await?;

        println!("[HOST] Connected to session bus");

        Ok(Self { connection })
    }

    pub async fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[HOST] Registering as StatusNotifierHost...");

        // Call RegisterStatusNotifierHost directly
        self.connection
            .call_method(
                Some("org.kde.StatusNotifierWatcher"),
                "/StatusNotifierWatcher",
                Some("org.kde.StatusNotifierWatcher"),
                "RegisterStatusNotifierHost",
                &("org.kde.StatusNotifierHost-riftbar"),
            )
            .await?;

        println!("[HOST] Successfully registered as StatusNotifierHost");

        Ok(())
    }

    pub async fn get_registered_items(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let reply = self.connection
            .call_method(
                Some("org.kde.StatusNotifierWatcher"),
                "/StatusNotifierWatcher",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.kde.StatusNotifierWatcher", "RegisteredStatusNotifierItems"),
            )
            .await?;

        let body: Body = reply.body();
        let items: Vec<String> = body.deserialize()?;

        Ok(items)
    }
}
