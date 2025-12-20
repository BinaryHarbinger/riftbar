// ============ mpris_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex, mpsc};
use tokio::process::Command;

pub struct MprisWidget {
    pub button: gtk::Button,
}

#[derive(Clone)]
pub struct MprisConfig {
    pub format_playing: String,
    pub format_paused: String,
    pub format_stopped: String,
    pub format_nothing: String,
    pub interval: u64,
    pub tooltip: bool,
    pub tooltip_format: String,
}

impl Default for MprisConfig {
    fn default() -> Self {
        Self {
            format_playing: "{icon} {artist} - {title}".to_string(),
            format_paused: "{icon} {artist} - {title}".to_string(),
            format_stopped: "{icon} Stopped".to_string(),
            format_nothing: "No Media".to_string(),
            interval: 100,
            tooltip: true,
            tooltip_format: "{artist}\n{album}\n{title}".to_string(),
        }
    }
}

impl MprisConfig {
    pub fn from_config(config: &crate::config::MprisConfig) -> Self {
        Self {
            format_playing: config.format_playing.clone(),
            format_paused: config.format_paused.clone(),
            format_stopped: config.format_stopped.clone(),
            format_nothing: config.format_nothing.clone(),
            interval: config.interval,
            tooltip: config.tooltip,
            tooltip_format: config.tooltip_format.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct MediaInfo {
    artist: String,
    title: String,
    album: String,
    status: String,
}

impl MprisWidget {
    pub fn new(config: MprisConfig) -> Self {
        // Media button
        let button = gtk::Button::with_label("No media");

        button.add_css_class("mpris");
        button.add_css_class("module");

        // Left click handler
        button.connect_clicked(|_| {
            Self::play_pause_async();
        });

        // Middle and right click handler
        let gesture = gtk::GestureClick::new();
        gesture.set_button(0); // Listen to all buttons

        gesture.connect_released(move |gesture, _, _, _| {
            let button_num = gesture.current_button();
            match button_num {
                2 => {
                    // Middle Click
                    Self::previous_track_async();
                }
                3 => {
                    // Right Click
                    Self::next_track_async();
                }
                _ => {}
            }
        });

        button.add_controller(gesture);

        let widget = Self { button };

        // Start the update loop
        widget.start_updates(config);

        widget
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn start_updates(&self, config: MprisConfig) {
        let button = self.button.clone();
        let (label_sender, label_receiver) = mpsc::channel::<String>();
        let (state_sender, state_receiver) = mpsc::channel::<String>();

        // Use Arc<Mutex> for thread-safe sharing of MediaInfo
        let media_info = Arc::new(Mutex::new(MediaInfo {
            artist: String::new(),
            title: String::new(),
            album: String::new(),
            status: String::from("Stopped"),
        }));

        let interval = config.interval;
        let config_clone = config.clone();
        let media_info_clone = media_info.clone();

        // Spawn async task to get metadata and status
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    // Get metadata
                    let artist_output = Command::new("playerctl")
                        .arg("metadata")
                        .arg("artist")
                        .output()
                        .await;

                    let title_output = Command::new("playerctl")
                        .arg("metadata")
                        .arg("title")
                        .output()
                        .await;

                    let album_output = Command::new("playerctl")
                        .arg("metadata")
                        .arg("album")
                        .output()
                        .await;

                    let status_output = Command::new("playerctl").arg("status").output().await;

                    let artist = if let Ok(output) = artist_output {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        String::new()
                    };

                    let title = if let Ok(output) = title_output {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        String::new()
                    };

                    let album = if let Ok(output) = album_output {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        String::new()
                    };

                    let status = if let Ok(output) = status_output {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        "Stopped".to_string()
                    };

                    // Update shared media info
                    {
                        let mut info = media_info_clone.lock().unwrap();
                        info.artist = artist.clone();
                        info.title = title.clone();
                        info.album = album.clone();
                        info.status = status.clone();
                    }

                    // Select indicator icon
                    let icon = match status.as_str() {
                        "Playing" => "",
                        "Paused" => "",
                        "Stopped" => "",
                        _ => "", // Playing
                    };

                    // Format the display text
                    let format_template = match status.as_str() {
                        "Playing" => &config_clone.format_playing,
                        "Paused" => &config_clone.format_paused,
                        "Stopped" => &config_clone.format_stopped,
                        _ => &config_clone.format_nothing,
                    };

                    let display = format_template
                        .replace("{icon}", icon)
                        .replace("{artist}", &artist)
                        .replace("{title}", &title)
                        .replace("{album}", &album)
                        .replace("{status}", &status);

                    let _ = state_sender.send(status.clone());
                    let _ = label_sender.send(display);

                    tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
                }
            });
        });

        // Set up tooltip if enabled
        if config.tooltip {
            button.set_has_tooltip(true);
            let tooltip_format = config.tooltip_format.clone();
            let media_info_tooltip = media_info.clone();

            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let info = media_info_tooltip.lock().unwrap();
                if !info.title.is_empty() {
                    let tooltip_text = tooltip_format
                        .replace("{artist}", &info.artist)
                        .replace("{title}", &info.title)
                        .replace("{album}", &info.album)
                        .replace("{status}", &info.status);
                    tooltip.set_text(Some(&tooltip_text));
                    return true;
                }
                tooltip.set_text(Some("No media playing"));
                true
            });
        }

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(metadata) = label_receiver.try_recv() {
                button.set_label(&metadata);
            }

            if let Ok(state) = state_receiver.try_recv() {
                let class = if state == "Playing" {
                    "mpris playing"
                } else if state == "Paused" {
                    "mpris paused"
                } else {
                    "mpris stopped"
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
            });
        });
    }

    fn next_track_async() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("playerctl").arg("next").output().await;
            });
        });
    }

    fn previous_track_async() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("playerctl").arg("previous").output().await;
            });
        });
    }
}
