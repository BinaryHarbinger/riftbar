// ============ mpris_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use mpris::{Event, MetadataValue, PlaybackStatus, PlayerFinder};
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use tokio::runtime::Runtime;

use crate::modules::mpris::dbus_util::{self, wait_for_active_player/*, get_active_player*/};

pub struct MprisWidget {
    pub button: gtk::Button,
}

#[derive(Clone)]
pub struct MprisConfig {
    pub format_playing: String,
    pub format_paused: String,
    pub format_stopped: String,
    pub format_nothing: String,
    pub lenght_lim: u64,
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
            lenght_lim: 0,
            interval: 100,
            tooltip: true,
            tooltip_format: "{artist}\n{album}\n{title}".to_string(),
        }
    }
}

impl MprisConfig {
    pub fn from_config(config: &crate::config::MprisConfig) -> Self {
        Self {
            format_playing: config.format_playing.clone().expect("How?"),
            format_paused: config.format_paused.clone().expect(""),
            format_stopped: config.format_stopped.clone().expect(""),
            format_nothing: config.format_nothing.clone(),
            lenght_lim: config.lenght_lim,
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
            crate::shared::util::run_command_async("playerctl play-pause".to_string());
        });

        // Middle and right click handler
        let gesture = gtk::GestureClick::new();
        gesture.set_button(0); // Listen to all buttons

        gesture.connect_released(move |gesture, _, _, _| {
            let button_num = gesture.current_button();
            match button_num {
                2 => {
                    // Middle Click
                    crate::shared::util::run_command_async("playerctl previous".to_string());
                }
                3 => {
                    // Right Click
                    crate::shared::util::run_command_async("playerctl next".to_string());
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
        let format_playing = config.format_playing;
        let format_paused = config.format_paused;
        let format_stopped = config.format_stopped;
        let format_nothing = config.format_nothing;
        let media_info_clone = media_info.clone();

        // Spawn thread to get metadata and status
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let dbus_obj = dbus_util::init_dbus();

                let player_finder = match PlayerFinder::new() {
                    Ok(pf) => pf,
                    Err(e) => {
                        eprintln!("[MPRIS]: Could not connect to D-Bus \n ERROR:{}", e);
                        std::thread::sleep(Duration::from_millis(interval));
                        return "";
                    }
                };
                let mut player_name = "".to_string();

                let mut player = loop {
                    match player_finder.find_active() {
                        Ok(p) => break p,
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(interval * 4));
                            let _ = state_sender.send("Nothing".to_string());
                            let _ = label_sender.send(format_nothing.to_string());
                            player_name = wait_for_active_player(&dbus_obj.conn, Some(950));
                            continue;
                        }
                    };
                };

                loop {
                    // Update player if needed
                    let new_player_name = wait_for_active_player(&dbus_obj.conn, Some(250));
                    if player_name != new_player_name {
                        player = loop {
                            match player_finder.find_active() {
                                Ok(p) => break p,
                                Err(_) => {
                                    std::thread::sleep(std::time::Duration::from_millis(
                                        interval * 4,
                                    ));
                                    let _ = state_sender.send("Nothing".to_string());
                                    let _ = label_sender.send(format_nothing.to_string());
                                    player_name = wait_for_active_player(&dbus_obj.conn, None);
                                    continue;
                                }
                            };
                        };
                    };
                    // Get playback status
                    let playback_status = match player.get_playback_status() {
                        Ok(PlaybackStatus::Playing) => "Playing",
                        Ok(PlaybackStatus::Paused) => "Paused",
                        Ok(PlaybackStatus::Stopped) => "Stopped",
                        _ => "Nothing",
                    };

                    // Get metadata
                    let metadata = match player.get_metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("metadata error: {}", e);
                            continue;
                        }
                    };

                    let title = get_string_from_metadata(&metadata, "xesam:title");
                    let artist = get_string_from_metadata(&metadata, "xesam:artist");
                    let album = get_string_from_metadata(&metadata, "xesam:album");

                    // Update shared media info
                    {
                        let mut info = media_info_clone.lock().unwrap();
                        info.artist = artist.to_string();
                        info.title = title.to_string();
                        info.album = album.to_string();
                        info.status = playback_status.to_string();
                    }

                    // Select indicator icon
                    let icon = match playback_status {
                        "Playing" => "",
                        "Paused" => "",
                        "Stopped" => "",
                        _ => "", // Playing
                    };

                    // Format the display text
                    let format_template = match playback_status {
                        "Playing" => &format_playing,
                        "Paused" => &format_paused,
                        "Stopped" => &format_stopped,
                        _ => &format_nothing,
                    };

                    let pre_display = format_template
                        .replace("{icon}", icon)
                        .replace("{artist}", artist)
                        .replace("{title}", title)
                        .replace("{album}", album)
                        .replace("{status}", playback_status);
                    drop(metadata);

                    let display = if config.lenght_lim != 0
                        && pre_display.chars().count() as u64 > config.lenght_lim
                    {
                        crate::shared::take_chars(pre_display.as_str(), config.lenght_lim)
                            .to_string()
                            + "…"
                    } else {
                        pre_display.to_string()
                    };

                    let _ = state_sender.send(playback_status.to_string());
                    let _ = label_sender.send(display);
                    let _ = playback_status;
                    drop(pre_display);
                    std::thread::sleep(std::time::Duration::from_millis(interval));

                    for event_result in player.events().unwrap() {
                        if let Err(e) = event_result {
                            eprintln!("[MPRIS]: Event error \n{:?}", e);
                            continue;
                        }
                         
                        let event = event_result.unwrap();
                        if matches!(
                            event,
                            Event::Playing
                                | Event::Paused
                                | Event::Stopped
                                | Event::PlaybackRateChanged(_)
                                | Event::TrackChanged(_)
                                | Event::TrackMetadataChanged { .. }
                                | Event::PlayerShutDown
                        ) {
                            break;
                        }
                    }
                }
            })
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
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
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
}

// Metadata to string converter
fn get_string_from_metadata<'a>(metadata: &'a mpris::Metadata, key: &str) -> &'a str {
    metadata
        .get(key)
        .and_then(|v| match v {
            MetadataValue::String(s) => Some(s.as_str()),
            MetadataValue::Array(arr) => arr.first().and_then(|v| {
                if let MetadataValue::String(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            }),
            _ => None,
        })
        .unwrap_or("")
}
