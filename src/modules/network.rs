// ============ network.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;
use tokio::process::Command;

pub struct NetworkWidget {
    pub container: gtk::Box,
    button: gtk::Button,
}

impl NetworkWidget {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);

        container.set_css_classes(&["network"]);
        // Network button
        let button = gtk::Button::with_label("No connection");

        // Click handler
        button.connect_clicked(|_| {
            println!("Network button clicked!");
            Self::on_click();
        });

        container.append(&button);

        let widget = Self { container, button };

        // Start the update loop
        widget.start_updates();

        widget
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn start_updates(&self) {
        let button = self.button.clone();
        let (sender, receiver) = mpsc::channel::<String>();

        // Spawn async task to get network info
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    // Get active connection name
                    let name_output = Command::new("nmcli")
                        .args(["-t", "-f", "NAME", "connection", "show", "--active"])
                        .output()
                        .await;

                    // Get signal strength
                    let signal_output = Command::new("nmcli")
                        .args(["-t", "-f", "IN-USE,SIGNAL", "device", "wifi"])
                        .output()
                        .await;

                    let display = match (name_output, signal_output) {
                        (Ok(name), Ok(signal)) => {
                            let connection_name = String::from_utf8_lossy(&name.stdout)
                                .lines()
                                .next()
                                .unwrap_or("Unknown")
                                .trim()
                                .to_string();

                            // Parse signal strength from active connection (marked with *)
                            let signal_str = String::from_utf8_lossy(&signal.stdout);
                            let signal_strength = signal_str
                                .lines()
                                .find(|line| line.starts_with('*'))
                                .and_then(|line| line.split(':').nth(1))
                                .unwrap_or("0");

                            if connection_name.is_empty() {
                                "No connection".to_string()
                            } else {
                                format!("{} ({}%)", connection_name, signal_strength)
                            }
                        }
                        _ => "No connection".to_string(),
                    };

                    let _ = sender.send(display);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });
        });

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(network_info) = receiver.try_recv() {
                button.set_label(&network_info);
            }
            glib::ControlFlow::Continue
        });
    }

    fn on_click() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg("echo 'Clicked'")
                    .output()
                    .await;
            });
        });
    }
}
