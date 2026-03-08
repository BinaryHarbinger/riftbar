// ============ slider.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::sleep;

pub struct SliderModuleWidget {
    slider: gtk::Scale,
}

pub struct SliderModuleConfig<'a> {
    pub name: &'a str,
    // When true, `exec` is run once and kept alive; every line it writes to
    // stdout immediately becomes the new label.  The process is restarted
    // automatically if it exits.  `interval` is ignored in this mode.
    pub listen: bool,
    pub length: u32,
    pub scroll_step: u32,
    pub scroll_cmd: String,
    pub exec: String,
    pub interval: u64,
}

impl SliderModuleWidget {
    pub fn new(config: SliderModuleConfig) -> Self {
        let slider = gtk::Scale::with_range(
            gtk::Orientation::Horizontal,
            0.0,
            100.0,
            config.scroll_step.into(),
        );
        let width = config.length.try_into().unwrap_or(100);
        let height = 0;
        slider.set_size_request(width, height);
        slider.add_css_class("slider-module");
        slider.add_css_class(&format!("slider-{}", config.name));

        // Create gesture handlers
        // Run a command if value of scale changes
        if !config.scroll_cmd.is_empty() {
            let scroll_cmd = config.scroll_cmd.clone();
            slider.connect_value_changed(move |s| {
                let value = s.value();
                let cmd = scroll_cmd.replace("{}", &value.to_string());
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
        }

        let widget = Self {
            slider: slider.clone(),
        };

        if config.listen {
            widget.start_listen(config.exec);
        } else {
            widget.start_updates(config.exec, config.interval);
        }

        widget
    }

    pub fn widget(&self) -> &gtk::Scale {
        &self.slider
    }

    // ── Polling mode ─────────────────────────────────────────────────────────

    fn start_updates(&self, exec: String, interval: u64) {
        let slider = self.slider.clone();
        let (sender, receiver) = mpsc::channel::<f64>();

        std::thread::spawn(move || {
            loop {
                if !exec.is_empty() {
                    let output = Command::new("sh").arg("-c").arg(&exec).output();

                    match output {
                        Ok(output) => {
                            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            let formatted: f64 = result.parse().unwrap_or(0.0);
                            let _ = sender.send(formatted);
                        }
                        Err(e) => {
                            eprintln!("Custom module exec failed: {}", e);
                        }
                    }
                } else {
                    let _ = sender.send(0.0);
                    break;
                }

                sleep(std::time::Duration::from_secs(interval));
            }
        });

        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                slider.set_value(msg);
            }
            glib::ControlFlow::Continue
        });
    }

    // ── Listen mode ──────────────────────────────────────────────────────────

    /// Spawn `exec` and read its stdout line by line.  Each non-empty line
    /// is sent to the GTK main thread as the new label text.  If the process
    /// exits for any reason it is restarted after a short back-off so a
    /// crashing script doesn't spam the CPU.
    fn start_listen(&self, exec: String) {
        let slider = self.slider.clone();
        let (sender, receiver) = mpsc::channel::<f64>();

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
                        eprintln!("[slider/listen] failed to spawn '{}': {}", exec, e);
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
                                let formatted: f64 = raw.trim().parse().unwrap_or(0.0);
                                // If the receiver has been dropped (widget
                                // destroyed), stop the thread silently.

                                if sender.send(formatted).is_err() {
                                    return;
                                }
                            }
                            Err(e) => {
                                eprintln!("[slider/listen] read error for '{}': {}", exec, e);
                                break;
                            }
                        }
                    }
                }

                // Wait for the child so we don't leave zombies.
                let _ = child.wait();

                eprintln!(
                    "[slider/listen] script '{}' exited, restarting in 2 seconds…",
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
            let mut last: Option<f64> = None;
            while let Ok(msg) = receiver.try_recv() {
                last = Some(msg);
            }
            if let Some(msg) = last {
                slider.set_value(msg);
            }
            glib::ControlFlow::Continue
        });
    }
}
