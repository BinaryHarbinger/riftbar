// ============ modules/clock.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;
use tokio::process::Command;

pub struct ClockWidget {
    pub button: gtk::Button,
}

#[derive(Clone)]
pub struct ClockConfig {
    pub format: String,
    pub interval: u64,
    pub tooltip: bool,
    pub tooltip_format: String,
    pub action: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: "%H:%M".to_string(),
            interval: 1,
            tooltip: true,
            tooltip_format: "%A, %B %d, %Y".to_string(),
            action: String::new(),
        }
    }
}

impl ClockConfig {
    pub fn from_config(config: &crate::config::ClockConfig) -> Self {
        Self {
            format: config.format.clone(),
            interval: config.interval,
            tooltip: config.tooltip,
            tooltip_format: config.tooltip_format.clone(),
            action: config.action.clone(),
        }
    }
}

impl ClockWidget {
    pub fn new(config: ClockConfig) -> Self {
        let button = gtk::Button::with_label("--:--");
        button.set_css_classes(&["clock", "module"]);
        button.set_widget_name("clock");
        let (sender, receiver) = mpsc::channel::<String>();

        // Connect button click handler
        let config_click = config.clone();
        button.connect_clicked(move |btn| {
            if !config_click.action.is_empty() {
                Self::run_command_async(&config_click.action);
            } else {
                println!("Clock clicked! Current time: {}", btn.label().unwrap());
            }
        });

        // Set up tooltip if enabled
        if config.tooltip {
            let tooltip_format = config.tooltip_format.clone();
            button.set_has_tooltip(true);
            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let tooltip_text = rt.block_on(async {
                    let output = Command::new("date")
                        .arg(format!("+{}", tooltip_format))
                        .output()
                        .await;

                    if let Ok(output) = output {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        "Date unavailable".to_string()
                    }
                });
                tooltip.set_text(Some(&tooltip_text));
                true
            });
        }

        // Clone button for the closure
        let button_clone = button.clone();
        let format = config.format.clone();
        let interval = config.interval;

        // Spawn async updater
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    let output = Command::new("date")
                        .arg(format!("+{}", format))
                        .output()
                        .await;

                    if let Ok(output) = output {
                        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let _ = sender.send(result);
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
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

    fn run_command_async(command: &str) {
        let command = command.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("sh").arg("-c").arg(&command).output().await;
            });
        });
    }
}
