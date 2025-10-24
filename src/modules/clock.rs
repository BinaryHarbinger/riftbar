// ============ clock.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;
use tokio::process::Command;

pub struct ClockWidget {
    pub button: gtk::Button,
}

impl ClockWidget {
    pub fn new() -> Self {
        let button = gtk::Button::with_label("--:--");
        button.set_css_classes(&["clock"]);
        let (sender, receiver) = mpsc::channel::<String>();

        // Connect button click handler
        button.connect_clicked(|btn| {
            println!("Clock clicked! Current time: {}", btn.label().unwrap());
        });

        // Clone button for the closure
        let button_clone = button.clone();

        // Spawn async updater
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    let output = Command::new("date").arg("+%H:%M").output().await.unwrap();

                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let _ = sender.send(result);

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        });

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                button_clone.set_label(&msg);
            }
            glib::ControlFlow::Continue
        });

        Self { button }
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }
}
