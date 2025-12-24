// ============ modules/tray/mod.rs ============
mod host;
mod watcher;
mod widget;

pub use widget::{TrayConfig, TrayWidget};

use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

// Global watcher connection
static WATCHER: OnceLock<Arc<Mutex<Option<zbus::Connection>>>> = OnceLock::new();

// Flag to track if SNI system is initialized
static SNI_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Initialize the SNI watcher and host (called only once when TrayWidget is first created)
pub(crate) async fn init_sni_system() -> Result<(), Box<dyn std::error::Error>> {
    // Check if already initialized
    if SNI_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        println!("[TRAY] SNI system already initialized, skipping");
        return Ok(());
    }

    println!("[TRAY] Initializing SNI system...");

    // Start the watcher
    let watcher_conn = watcher::StatusNotifierWatcher::start().await?;

    // Store the connection
    WATCHER
        .set(Arc::new(Mutex::new(Some(watcher_conn))))
        .map_err(|_| "Watcher already initialized")?;

    // Small delay to ensure watcher is ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create and register the host
    let host = host::StatusNotifierHost::new().await?;
    host.register().await?;

    // Mark as initialized
    SNI_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);

    println!("[TRAY] SNI system initialized successfully");

    Ok(())
}
