// ============ modules/audio.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct AudioWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct AudioConfig {
    pub format: String,
    pub format_muted: String,
    pub interval: u64,
    pub tooltip: bool,
    pub on_click: String,
    pub on_scroll_up: String,
    pub on_scroll_down: String,
    pub scroll_step: i32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            format: "{icon} {volume}%".to_string(),
            format_muted: "{icon} Muted".to_string(),
            interval: 100,
            tooltip: true,
            on_click: "".to_string(),
            on_scroll_up: "".to_string(),
            on_scroll_down: "".to_string(),
            scroll_step: 5,
        }
    }
}

impl AudioConfig {
    pub fn from_config(config: &crate::config::AudioConfig) -> Self {
        Self {
            format: config.format.clone(),
            format_muted: config.format_muted.clone(),
            interval: config.interval,
            tooltip: config.tooltip,
            on_click: config.on_click.clone(),
            on_scroll_up: config.on_scroll_up.clone(),
            on_scroll_down: config.on_scroll_down.clone(),
            scroll_step: config.scroll_step,
        }
    }
}

#[derive(Clone, Debug)]
struct AudioInfo {
    volume: i32,
    muted: bool,
    backend: AudioBackend,
}

#[derive(Clone, Debug, PartialEq)]
enum AudioBackend {
    PipeWire,
    PulseAudio,
    Unknown,
}

impl AudioWidget {
    pub fn new(config: AudioConfig) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        container.add_css_class("audio");
        container.add_css_class("module");

        let button = gtk::Button::new();
        button.add_css_class("audio-button");

        let label = gtk::Label::new(Some(""));
        label.add_css_class("audio-label");
        button.set_child(Some(&label));

        container.append(&button);

        let audio_info = Arc::new(Mutex::new(AudioInfo {
            volume: 0,
            muted: false,
            backend: AudioBackend::Unknown,
        }));

        // Detect backend and get initial info
        let backend = detect_audio_backend();
        let info = get_audio_info(&backend);
        *audio_info.lock().unwrap() = info.clone();
        update_label(&label, &info, &config);

        // Set up click handler
        let config_click = config.clone();
        let backend_click = backend.clone();
        button.connect_clicked(move |_| {
            if !config_click.on_click.is_empty() {
                run_command_async(&config_click.on_click);
            } else {
                // Default action: toggle mute
                toggle_mute(&backend_click);
            }
        });

        // Set up scroll handler
        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);

        let config_scroll = config.clone();
        let backend_scroll = backend.clone();
        scroll_controller.connect_scroll(move |_, _, dy| {
            if dy < 0.0 {
                // Scroll up - increase volume
                if !config_scroll.on_scroll_up.is_empty() {
                    run_command_async(&config_scroll.on_scroll_up);
                } else {
                    change_volume(&backend_scroll, config_scroll.scroll_step);
                }
            } else {
                // Scroll down - decrease volume
                if !config_scroll.on_scroll_down.is_empty() {
                    run_command_async(&config_scroll.on_scroll_down);
                } else {
                    change_volume(&backend_scroll, -config_scroll.scroll_step);
                }
            }
            gtk4::glib::Propagation::Stop
        });

        button.add_controller(scroll_controller);

        // Set up periodic updates
        let label_clone = label.clone();
        let config_clone = config.clone();
        let audio_info_clone = audio_info.clone();
        let backend_clone = backend.clone();

        gtk4::glib::timeout_add_local(
            std::time::Duration::from_millis(config.interval),
            move || {
                let info = get_audio_info(&backend_clone);
                *audio_info_clone.lock().unwrap() = info.clone();
                update_label(&label_clone, &info, &config_clone);
                gtk4::glib::ControlFlow::Continue
            },
        );

        // Add tooltip if enabled
        if config.tooltip {
            let audio_info_clone = audio_info.clone();
            container.set_has_tooltip(true);
            container.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let info = audio_info_clone.lock().unwrap();
                let tooltip_text = format!(
                    "Volume: {}%\nStatus: {}\nBackend: {:?}",
                    info.volume,
                    if info.muted { "Muted" } else { "Playing" },
                    info.backend
                );
                tooltip.set_text(Some(&tooltip_text));
                true
            });
        }

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
}

fn update_label(label: &gtk::Label, info: &AudioInfo, config: &AudioConfig) {
    let icon = get_icon_for_volume(info.volume, info.muted);

    let format_template = if info.muted {
        &config.format_muted
    } else {
        &config.format
    };

    let text = format_template
        .replace("{icon}", &icon)
        .replace("{volume}", &info.volume.to_string());

    label.set_text(&text);

    // Update CSS classes
    label.remove_css_class("muted");
    label.remove_css_class("low");
    label.remove_css_class("medium");
    label.remove_css_class("high");

    if info.muted {
        label.add_css_class("muted");
    } else if info.volume <= 33 {
        label.add_css_class("low");
    } else if info.volume <= 66 {
        label.add_css_class("medium");
    } else {
        label.add_css_class("high");
    }
}

fn get_icon_for_volume(volume: i32, muted: bool) -> String {
    if muted {
        return "".to_string(); // Muted icon
    }

    if volume == 0 {
        "".to_string() // No volume
    } else if volume <= 33 {
        "".to_string() // Low volume
    } else if volume <= 66 {
        "".to_string() // Medium volume
    } else {
        "".to_string() // High volume
    }
}

fn detect_audio_backend() -> AudioBackend {
    // Check for wpctl (PipeWire/WirePlumber)
    if Command::new("wpctl").arg("--version").output().is_ok() {
        return AudioBackend::PipeWire;
    }

    // Check for pactl (PulseAudio)
    if Command::new("pactl").arg("--version").output().is_ok() {
        return AudioBackend::PulseAudio;
    }

    AudioBackend::Unknown
}

fn get_audio_info(backend: &AudioBackend) -> AudioInfo {
    match backend {
        AudioBackend::PipeWire => get_pipewire_info(),
        AudioBackend::PulseAudio => get_pulseaudio_info(),
        AudioBackend::Unknown => AudioInfo {
            volume: 0,
            muted: false,
            backend: AudioBackend::Unknown,
        },
    }
}

fn get_pipewire_info() -> AudioInfo {
    // Get default sink ID
    let sink_output = Command::new("wpctl")
        .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output();

    if let Ok(output) = sink_output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Output format: "Volume: 0.50" or "Volume: 0.50 [MUTED]"

        let muted = output_str.contains("[MUTED]");

        // Parse volume (0.0 to 1.0)
        let volume = output_str
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<f32>().ok())
            .map(|v| (v * 100.0) as i32)
            .unwrap_or(0);

        return AudioInfo {
            volume,
            muted,
            backend: AudioBackend::PipeWire,
        };
    }

    AudioInfo {
        volume: 0,
        muted: false,
        backend: AudioBackend::PipeWire,
    }
}

fn get_pulseaudio_info() -> AudioInfo {
    let output = Command::new("pactl")
        .args(&["get-sink-volume", "@DEFAULT_SINK@"])
        .output();

    let mute_output = Command::new("pactl")
        .args(&["get-sink-mute", "@DEFAULT_SINK@"])
        .output();

    let mut volume = 0;
    let mut muted = false;

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse volume percentage from output like: "Volume: front-left: 65536 / 100% ..."
        for part in output_str.split_whitespace() {
            if part.ends_with('%') {
                if let Ok(vol) = part.trim_end_matches('%').parse::<i32>() {
                    volume = vol;
                    break;
                }
            }
        }
    }

    if let Ok(output) = mute_output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        muted = output_str.contains("yes");
    }

    AudioInfo {
        volume,
        muted,
        backend: AudioBackend::PulseAudio,
    }
}

fn toggle_mute(backend: &AudioBackend) {
    std::thread::spawn({
        let backend = backend.clone();
        move || match backend {
            AudioBackend::PipeWire => {
                let _ = Command::new("wpctl")
                    .args(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                    .output();
            }
            AudioBackend::PulseAudio => {
                let _ = Command::new("pactl")
                    .args(&["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                    .output();
            }
            AudioBackend::Unknown => {}
        }
    });
}

fn change_volume(backend: &AudioBackend, delta: i32) {
    std::thread::spawn({
        let backend = backend.clone();
        move || match backend {
            AudioBackend::PipeWire => {
                let change = if delta > 0 {
                    format!("{}%+", delta)
                } else {
                    format!("{}%-", -delta)
                };
                let _ = Command::new("wpctl")
                    .args(&["set-volume", "@DEFAULT_AUDIO_SINK@", &change])
                    .output();
            }
            AudioBackend::PulseAudio => {
                let change = if delta > 0 {
                    format!("+{}%", delta)
                } else {
                    format!("{}%", delta)
                };
                let _ = Command::new("pactl")
                    .args(&["set-sink-volume", "@DEFAULT_SINK@", &change])
                    .output();
            }
            AudioBackend::Unknown => {}
        }
    });
}

fn run_command_async(command: &str) {
    let command = command.to_string();
    std::thread::spawn(move || {
        let _ = Command::new("sh").arg("-c").arg(&command).output();
        println!("Executed command: {}", command);
    });
}
