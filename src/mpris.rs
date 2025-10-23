// ============ mpris.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
// use mpris::Player;
// use mpris::PlayerFinder;
use std::sync::mpsc;
use tokio::process::Command;

pub struct MprisWidget {
    pub container: gtk::Box,
    button: gtk::Button,
}

impl MprisWidget {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);

        // Media button
        let button = gtk::Button::with_label("No media playing");

        // Left click handler
        button.connect_clicked(|_| {
            println!("Left click!");
            Self::play_pause_async();
        });

        // Middle and right click handler
        let gesture = gtk::GestureClick::new();
        gesture.set_button(0); // Listen to all buttons

        gesture.connect_released(move |gesture, _, _, _| {
            let button_num = gesture.current_button();
            match button_num {
                2 => {
                    println!("Middle click!");
                    Self::previous_track_async();
                }
                3 => {
                    println!("Right click!");
                    Self::next_track_async();
                }
                _ => {}
            }
        });

        button.add_controller(gesture);
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
        let (label_sender, label_receiver) = mpsc::channel::<String>();
        let (state_sender, state_receiver) = mpsc::channel::<String>();

        let display_metadata: bool = false;

        // Spawn async task to get metadata and status
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    // Get metadata
                    let metadata_output = Command::new("playerctl")
                        .arg("metadata")
                        .arg("--format")
                        .arg("{{ artist }} - {{ title }}")
                        .output()
                        .await
                        .unwrap();

                    let metadata = String::from_utf8_lossy(&metadata_output.stdout)
                        .trim()
                        .to_string();

                    // Get status
                    let status_output = Command::new("playerctl")
                        .arg("status")
                        .output()
                        .await
                        .unwrap();

                    let player_status = String::from_utf8_lossy(&status_output.stdout)
                        .trim()
                        .to_string();

                    // Select a indicator icon

                    let status = if player_status == "Paused" {
                        "".to_string()
                    } else if player_status == "Stopped" {
                        "".to_string()
                    } else {
                        "".to_string()
                    };

                    // Combine and send
                    let _ = state_sender.send(player_status.clone());
                    let display = if display_metadata {
                        format!("{}  {}", status, metadata)
                    } else {
                        format!("{}  {}", status, player_status)
                    };
                    let _ = label_sender.send(display);

                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            });
        });

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(metadata) = label_receiver.try_recv() {
                button.set_label(&metadata);
            }

            if let Ok(state) = state_receiver.try_recv() {
                let class = if state == "Playing" {
                    "mpris"
                } else {
                    "mpris paused"
                };
                let classes: Vec<&str> = class.split(' ').collect();
                button.set_css_classes(&classes);
            }

            glib::ControlFlow::Continue
        });
    }

    fn play_pause_async() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("playerctl").arg("play-pause").output().await;
                println!("Play-pause toggled");
            });
        });
    }

    fn next_track_async() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("playerctl").arg("next").output().await;
                println!("Next track");
            });
        });
    }

    fn previous_track_async() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("playerctl").arg("previous").output().await;
                println!("Previous track");
            });
        });
    }
}
