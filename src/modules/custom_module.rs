// ============ custom_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::sleep;

pub struct CustomModuleWidget {
    button: gtk::Button,
    label: gtk::Label,
}

pub struct CustomModuleConfig<'a> {
    pub name: &'a str,
    pub on_click: String,
    pub on_click_right: String,
    pub on_click_middle: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub exec: String,
    pub interval: u64,
    pub format: Option<String>,
    /// When true, `exec` is run once and kept alive; every line it writes to
    /// stdout immediately becomes the new label.  The process is restarted
    /// automatically if it exits.  `interval` is ignored in this mode.
    pub listen: bool,
}

impl CustomModuleWidget {
    pub fn new(config: CustomModuleConfig) -> Self {
        let label = gtk::Label::new(None);
        let button = gtk::Button::new();
        button.set_child(Some(&label));
        button.add_css_class("custom-module");
        button.add_css_class(&format!("custom-{}", config.name));

        // Create click handlers
        crate::shared::create_gesture_handler(
            &button,
            crate::shared::Gestures {
                on_click: config.on_click,
                on_click_middle: Some(config.on_click_middle),
                on_click_right: Some(config.on_click_right),
                scroll_up: Some(config.scroll_up),
                scroll_down: Some(config.scroll_down),
            },
        );

        let widget = Self {
            button: button.clone(),
            label: label.clone(),
        };

        if config.listen {
            widget.start_listen(config.exec, config.format);
        } else {
            widget.start_updates(config.exec, config.interval, config.format);
        }

        widget
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    // ── Polling mode ─────────────────────────────────────────────────────────

    fn start_updates(&self, exec: String, interval: u64, format: Option<String>) {
        let label = self.label.clone();
        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            loop {
                if !exec.is_empty() {
                    let output = Command::new("sh").arg("-c").arg(&exec).output();

                    match output {
                        Ok(output) => {
                            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            let formatted = if let Some(ref fmt) = format {
                                fmt.replace("{}", &result)
                            } else {
                                result
                            };
                            let _ = sender.send(formatted);
                        }
                        Err(e) => {
                            eprintln!("Custom module exec failed: {}", e);
                        }
                    }
                } else {
                    let _ = sender.send(format.clone().unwrap_or_default());
                    break;
                }

                sleep(std::time::Duration::from_secs(interval));
            }
        });

        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                label.set_markup(&msg);
            }
            glib::ControlFlow::Continue
        });
    }

    // ── Listen mode ──────────────────────────────────────────────────────────

    /// Spawn `exec` and read its stdout line by line.  Each non-empty line
    /// is sent to the GTK main thread as the new label text.  If the process
    /// exits for any reason it is restarted after a short back-off so a
    /// crashing script doesn't spam the CPU.
    fn start_listen(&self, exec: String, format: Option<String>) {
        let label = self.label.clone();
        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            if exec.is_empty() {
                return;
            }

            loop {
                // Spawn the script with its stdout piped.
                let child = Command::new("sh")
                    .arg("-c")
                    .arg(&exec)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .stdin(Stdio::null())
                    .spawn();

                let mut child = match child {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[custom/listen] failed to spawn '{}': {}", exec, e);
                        // Back off before retrying.
                        sleep(std::time::Duration::from_secs(5));
                        continue;
                    }
                };

                // Read stdout line by line until EOF.
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        match line {
                            Ok(raw) => {
                                let raw = raw.to_string();
                                if raw.is_empty() {
                                    continue;
                                }
                                let formatted = if let Some(ref fmt) = format {
                                    fmt.replace("{}", &raw)
                                } else {
                                    raw
                                };
                                // If the receiver has been dropped (widget
                                // destroyed), stop the thread silently.
                                if sender.send(formatted).is_err() {
                                    return;
                                }
                            }
                            Err(e) => {
                                eprintln!("[custom/listen] read error for '{}': {}", exec, e);
                                break;
                            }
                        }
                    }
                }

                // Wait for the child so we don't leave zombies.
                let _ = child.wait();

                eprintln!(
                    "[custom/listen] script '{}' exited, restarting in 2 seconds…",
                    exec
                );
                sleep(std::time::Duration::from_secs(2));
            }
        });

        // Poll the channel on the GTK main thread — same cadence as the
        // polling mode so there is at most ~100 ms of display lag.
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Drain all pending lines; show only the most recent one so a
            // fast-writing script doesn't stall the UI.
            let mut last: Option<String> = None;
            while let Ok(msg) = receiver.try_recv() {
                last = Some(msg);
            }
            if let Some(msg) = last {
                label.set_markup(&msg);
            }
            glib::ControlFlow::Continue
        });
    }
}
