// ============ modules/active_window.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;

#[derive(Clone)]
pub struct ActiveWindowConfig {
    pub length_lim: u64,
    pub tooltip: bool,
    pub on_click: String,
    pub use_class: bool,
    pub no_window_format: String,
}

impl Default for ActiveWindowConfig {
    fn default() -> Self {
        Self {
            length_lim: 0,
            tooltip: false,
            on_click: String::new(),
            use_class: false,
            no_window_format: String::from("No Window"),
        }
    }
}

impl ActiveWindowConfig {
    pub fn from_config(config: &crate::config::ActiveWindowConfig) -> Self {
        Self {
            length_lim: config.length_lim,
            tooltip: config.tooltip,
            on_click: config.on_click.clone(),
            use_class: config.use_class,
            no_window_format: config.no_window_format.clone(),
        }
    }
}

pub struct ActiveWindowWidget {
    pub button: gtk::Button,
}

#[derive(Clone, Debug)]
struct WindowInfo {
    class: String,
    title: String,
}

impl ActiveWindowWidget {
    pub fn new(mut config: ActiveWindowConfig) -> Self {
        if config.no_window_format.is_empty() {
            config.no_window_format = "No Window".to_string();
        }

        let button = gtk::Button::with_label(config.no_window_format.as_str());
        button.add_css_class("active-name");
        button.add_css_class("module");

        // Click handler
        let config_click = config.clone();
        button.connect_clicked(move |_| {
            if !config_click.on_click.is_empty() {
                crate::shared::util::run_command_async(config_click.on_click.clone());
            }
        });

        let widget = Self { button };
        widget.start_updates(config);
        widget
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn start_updates(&self, config: ActiveWindowConfig) {
        let button = self.button.clone();
        let (sender, receiver) = mpsc::channel::<WindowInfo>();

        let length_lim = config.length_lim;

        let window_info = std::sync::Arc::new(std::sync::Mutex::new(WindowInfo {
            class: String::new(),
            title: String::new(),
        }));

        // Set up tooltip if enabled
        if config.tooltip {
            button.set_has_tooltip(true);
            let window_info_tooltip = window_info.clone();

            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let info = window_info_tooltip.lock().unwrap();
                if !info.title.is_empty() {
                    let tooltip_text = format!("{}\n{}", info.class, info.title);
                    tooltip.set_text(Some(&tooltip_text));
                    true
                } else {
                    tooltip.set_text(Some("No active window"));
                    true
                }
            });
        }

        let window_info_clone = window_info.clone();

        // Spawn window monitor thread
        std::thread::spawn(move || {
            if let Ok(mut monitor) = WindowMonitor::new() {
                // Get initial window
                if let Some((class, title)) = monitor.get_current_window() {
                    let _ = sender.send(WindowInfo { class, title });
                }

                // Listen for window changes
                let _ = monitor.listen(|class, title| {
                    let _ = sender.send(WindowInfo { class, title });
                });
            } else {
                eprintln!("[ActiveWindow]: Failed to initialize window monitor");
            }
        });

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(info) = receiver.try_recv() {
                // Update shared window info
                {
                    let mut shared_info = window_info_clone.lock().unwrap();
                    shared_info.class = info.class.clone();
                    shared_info.title = info.title.clone();
                }

                // Determine display text
                let display_text = if !info.title.is_empty() && !config.use_class {
                    info.title.clone()
                } else if !info.class.is_empty() {
                    info.class.clone()
                } else {
                    config.no_window_format.clone()
                };

                // Truncate if needed
                let label = if length_lim != 0 && display_text.chars().count() > length_lim as usize
                {
                    crate::shared::take_chars(&display_text, length_lim).to_string() + "…"
                } else {
                    display_text
                };

                button.set_label(&label);
            }

            glib::ControlFlow::Continue
        });
    }
}

// ============ Window Monitor Implementation ============

struct WindowMonitor {
    reader: Option<BufReader<UnixStream>>,
    compositor: Compositor,
}

enum Compositor {
    Hyprland,
    Sway,
    Unsupported,
}

impl WindowMonitor {
    fn new() -> std::io::Result<Self> {
        let compositor = Self::detect_compositor();
        let reader = match compositor {
            Compositor::Hyprland => Self::connect_hyprland()?,
            Compositor::Sway => Self::connect_sway()?,
            Compositor::Unsupported => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Unsupported compositor",
                ));
            }
        };

        Ok(Self { reader, compositor })
    }

    fn detect_compositor() -> Compositor {
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            Compositor::Hyprland
        } else if env::var("SWAYSOCK").is_ok() {
            Compositor::Sway
        } else {
            Compositor::Unsupported
        }
    }

    fn connect_hyprland() -> std::io::Result<Option<BufReader<UnixStream>>> {
        let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HYPRLAND_INSTANCE_SIGNATURE not found",
            )
        })?;

        // Use XDG_RUNTIME_DIR instead of /tmp
        let runtime_dir =
            env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

        let socket_path = format!("{}/hypr/{}/.socket2.sock", runtime_dir, instance);

        match UnixStream::connect(&socket_path) {
            Ok(stream) => Ok(Some(BufReader::new(stream))),
            Err(e) => {
                eprintln!("[ActiveWindow]: Failed to connect to socket: {}", e);
                Err(e)
            }
        }
    }

    fn connect_sway() -> std::io::Result<Option<BufReader<UnixStream>>> {
        let socket_path = env::var("SWAYSOCK")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "SWAYSOCK not found"))?;

        let stream = UnixStream::connect(socket_path)?;
        Ok(Some(BufReader::new(stream)))
    }

    fn listen<F>(&mut self, callback: F) -> std::io::Result<()>
    where
        F: FnMut(String, String),
    {
        match self.compositor {
            Compositor::Hyprland => self.listen_hyprland(callback),
            Compositor::Sway => self.listen_sway(callback),
            Compositor::Unsupported => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Compositor not supported",
            )),
        }
    }

    fn listen_hyprland<F>(&mut self, mut callback: F) -> std::io::Result<()>
    where
        F: FnMut(String, String),
    {
        let reader = self.reader.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "No connection")
        })?;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("activewindow>>") {
                let parts: Vec<&str> = line
                    .strip_prefix("activewindow>>")
                    .unwrap_or("")
                    .split(',')
                    .collect();

                if parts.len() >= 2 {
                    let class = parts[0].to_string();
                    let title = parts[1].to_string();
                    callback(class, title);
                }
            }
        }
        Ok(())
    }

    fn listen_sway<F>(&mut self, mut callback: F) -> std::io::Result<()>
    where
        F: FnMut(String, String),
    {
        let reader = self.reader.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "No connection")
        })?;

        // Subscribe to window events using Sway IPC protocol
        let subscribe_msg = b"i3-ipc\x0e\x00\x00\x00\x02\x00\x00\x00[\"window\"]";
        reader.get_mut().write_all(subscribe_msg)?;

        loop {
            let mut header = [0u8; 14];
            reader.get_mut().read_exact(&mut header)?;

            let len = u32::from_le_bytes([header[6], header[7], header[8], header[9]]) as usize;
            let mut payload = vec![0u8; len];
            reader.get_mut().read_exact(&mut payload)?;

            if let Ok(json) = std::str::from_utf8(&payload) {
                // Simple JSON parsing for focus events
                if json.contains("\"change\":\"focus\"") {
                    let class = extract_field(json, "app_id").unwrap_or_default();
                    let title = extract_field(json, "name").unwrap_or_default();
                    callback(class, title);
                }
            }
        }
    }

    fn get_current_window(&self) -> Option<(String, String)> {
        match self.compositor {
            Compositor::Hyprland => Self::get_hyprland_window(),
            Compositor::Sway => Self::get_sway_window(),
            Compositor::Unsupported => None,
        }
    }

    fn get_hyprland_window() -> Option<(String, String)> {
        use std::process::Command;

        let output = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            eprintln!("[ActiveWindow]: hyprctl command failed");
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;
        let class = extract_field(&json, "class")?;
        let title = extract_field(&json, "title")?;

        Some((class, title))
    }

    fn get_sway_window() -> Option<(String, String)> {
        use std::process::Command;

        let output = Command::new("swaymsg")
            .args(["-t", "get_tree"])
            .output()
            .ok()?;

        let json = String::from_utf8(output.stdout).ok()?;

        // Find focused window
        if json.contains("\"focused\":true") {
            let class = extract_field(&json, "app_id").unwrap_or_default();
            let title = extract_field(&json, "name")?;
            Some((class, title))
        } else {
            None
        }
    }
}

/// Extract a field value from JSON string (simple parser, no dependencies)
fn extract_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", field);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')?;
    Some(json[start..start + end].to_string())
}
