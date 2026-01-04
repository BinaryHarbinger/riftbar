// ============ modules/clock.rs ============
use chrono::Local;
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;

pub struct ClockWidget {
    pub button: gtk::Button,
}

#[derive(Clone)]
pub struct ClockConfig {
    pub format: String,
    pub interval: u64,
    pub tooltip: bool,
    pub tooltip_format: String,
    pub on_click: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: "%H:%M".to_string(),
            interval: 1,
            tooltip: true,
            tooltip_format: "%A, %B %d, %Y".to_string(),
            on_click: String::new(),
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
            on_click: config.on_click.clone(),
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
            if !config_click.on_click.is_empty() {
                crate::shared::util::run_command_async(config_click.on_click.clone());
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
                let tooltip_text =
                    rt.block_on(async { Local::now().format(&tooltip_format).to_string() });
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
                let mut last_output = String::new();
                loop {
                    let output = Local::now().format(&format).to_string();
                    if last_output != output {
                        last_output = output.clone();
                        let _ = sender.send(output);
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
}
