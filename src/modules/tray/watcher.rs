// ============ modules/tray/watcher.rs ============
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use zbus::interface;
use zbus::{Connection, object_server::SignalContext};

#[derive(Debug, Clone)]
pub struct StatusNotifierWatcher {
    registered_items: Arc<RwLock<HashSet<String>>>,
    registered_hosts: Arc<RwLock<HashSet<String>>>,
}

impl StatusNotifierWatcher {
    pub fn new() -> Self {
        Self {
            registered_items: Arc::new(RwLock::new(HashSet::new())),
            registered_hosts: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn start() -> Result<Connection, Box<dyn std::error::Error>> {
        println!("[WATCHER] Starting StatusNotifierWatcher...");

        let watcher = StatusNotifierWatcher::new();

        let connection = zbus::connection::Builder::session()?
            .name("org.kde.StatusNotifierWatcher")?
            .serve_at("/StatusNotifierWatcher", watcher)?
            .build()
            .await?;

        println!("[WATCHER] StatusNotifierWatcher started successfully");
        println!("[WATCHER] Service name: org.kde.StatusNotifierWatcher");
        println!("[WATCHER] Object path: /StatusNotifierWatcher");

        Ok(connection)
    }

    async fn emit_item_registered(
        &self,
        ctxt: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()> {
        Self::status_notifier_item_registered(ctxt, service).await
    }

    async fn emit_item_unregistered(
        &self,
        ctxt: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()> {
        Self::status_notifier_item_unregistered(ctxt, service).await
    }

    async fn emit_host_registered(&self, ctxt: &SignalContext<'_>) -> zbus::Result<()> {
        Self::status_notifier_host_registered(ctxt).await
    }
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    /// Register a StatusNotifierItem
    async fn register_status_notifier_item(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        service: String,
    ) {
        println!("[WATCHER] Registering item: {}", service);

        let mut items = self.registered_items.write().await;
        if items.insert(service.clone()) {
            println!("[WATCHER] Item registered: {}", service);
            drop(items);

            if let Err(e) = self.emit_item_registered(&ctxt, &service).await {
                eprintln!("[WATCHER] Failed to emit ItemRegistered signal: {}", e);
            }
        } else {
            println!("[WATCHER] Item already registered: {}", service);
        }
    }

    /// Register a StatusNotifierHost
    async fn register_status_notifier_host(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        service: String,
    ) {
        println!("[WATCHER] Registering host: {}", service);

        let mut hosts = self.registered_hosts.write().await;
        if hosts.insert(service.clone()) {
            println!("[WATCHER] Host registered: {}", service);
            drop(hosts);

            if let Err(e) = self.emit_host_registered(&ctxt).await {
                eprintln!("[WATCHER] Failed to emit HostRegistered signal: {}", e);
            }
        } else {
            println!("[WATCHER] Host already registered: {}", service);
        }
    }

    /// Get all registered items
    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        let items = self.registered_items.read().await;
        items.iter().cloned().collect()
    }

    /// Check if a host is registered
    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        let hosts = self.registered_hosts.read().await;
        !hosts.is_empty()
    }

    /// Protocol version
    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }

    /// Signal: StatusNotifierItemRegistered
    #[zbus(signal)]
    async fn status_notifier_item_registered(
        ctxt: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()> {}

    /// Signal: StatusNotifierItemUnregistered
    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        ctxt: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()> {}

    /// Signal: StatusNotifierHostRegistered
    #[zbus(signal)]
    async fn status_notifier_host_registered(ctxt: &SignalContext<'_>) -> zbus::Result<()> {}
}
