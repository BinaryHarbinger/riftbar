// ============ modules/revealer.rs ============

use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};

pub struct RevealerWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct RevealerConfig {
    pub modules: Vec<String>,
    pub spacing: i32,
    pub orientation: String,
    pub trigger: String,
    pub transition: String,
    pub transition_duration: u32,
    pub reveal_on_hover: bool,
}

impl Default for RevealerConfig {
    fn default() -> Self {
        Self {
            modules: Vec::new(),
            spacing: 10,
            orientation: "horizontal".to_string(),
            trigger: String::new(),
            transition: "slide_left".to_string(),
            transition_duration: 200,
            reveal_on_hover: false,
        }
    }
}

impl RevealerWidget {
    pub fn new(name: &str, config: RevealerConfig, app_config: &crate::config::Config) -> Self {
        // Main container
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add_css_class("revealer-widget");
        container.add_css_class(&format!("revealer-{}", name));

        // Determine orientation for the content box
        let content_orientation = match config.orientation.as_str() {
            "vertical" => gtk::Orientation::Vertical,
            _ => gtk::Orientation::Horizontal,
        };

        // Create the revealer
        let revealer = gtk::Revealer::new();
        revealer.set_transition_duration(config.transition_duration);

        // Set transition type
        let transition_type = match config.transition.as_str() {
            "slide_left" => gtk::RevealerTransitionType::SlideLeft,
            "slide_right" => gtk::RevealerTransitionType::SlideRight,
            "slide_up" => gtk::RevealerTransitionType::SlideUp,
            "slide_down" => gtk::RevealerTransitionType::SlideDown,
            "crossfade" => gtk::RevealerTransitionType::Crossfade,
            _ => gtk::RevealerTransitionType::SlideLeft,
        };
        revealer.set_transition_type(transition_type);

        // Content box that will be revealed
        let content_box = gtk::Box::new(content_orientation, config.spacing);
        content_box.add_css_class("revealer-content");

        // Build modules in the content box
        Self::build_modules(&content_box, &config.modules, app_config);
        revealer.set_child(Some(&content_box));

        // State tracking
        let is_revealed = Arc::new(Mutex::new(false));

        // If there's a trigger, create it
        if !config.trigger.is_empty() {
            let trigger_button = gtk::Button::with_label(&config.trigger);
            trigger_button.add_css_class("revealer-trigger");

            // Toggle on click
            let revealer_clone = revealer.clone();
            let is_revealed_clone = is_revealed.clone();
            trigger_button.connect_clicked(move |_| {
                let mut revealed = is_revealed_clone.lock().unwrap();
                *revealed = !*revealed;
                revealer_clone.set_reveal_child(*revealed);
            });

            container.append(&trigger_button);
        }

        // Add hover behavior if enabled
        if config.reveal_on_hover {
            let hover_controller = gtk::EventControllerMotion::new();
            let revealer_hover = revealer.clone();
            let is_revealed_hover = is_revealed.clone();

            hover_controller.connect_enter(move |_, _, _| {
                *is_revealed_hover.lock().unwrap() = true;
                revealer_hover.set_reveal_child(true);
            });

            container.add_controller(hover_controller);

            let leave_controller = gtk::EventControllerMotion::new();
            let revealer_leave = revealer.clone();
            let is_revealed_leave = is_revealed.clone();

            leave_controller.connect_leave(move |_| {
                *is_revealed_leave.lock().unwrap() = false;
                revealer_leave.set_reveal_child(false);
            });

            container.add_controller(leave_controller);
        }

        container.append(&revealer);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn build_modules(
        container: &gtk::Box,
        module_names: &[String],
        config: &crate::config::Config,
    ) {
        use crate::modules::*;

        println!("Building modules in revealer: {:?}", module_names);

        for name in module_names {
            match name.as_str() {
                "clock" => {
                    let clock_config = ClockConfig::from_config(&config.clock);
                    let clock = ClockWidget::new(clock_config);
                    container.append(clock.widget());
                }
                "hyprland/workspaces" => {
                    let workspaces_config =
                        Arc::new(WorkspacesConfig::from_config(&config.workspaces));
                    let workspaces = HyprWorkspacesWidget::new(workspaces_config);

                    container.append(workspaces.widget());
                }
                "mpris" => {
                    let mpris_config = MprisConfig::from_config(&config.mpris);
                    let mpris = MprisWidget::new(mpris_config);
                    container.append(mpris.widget());
                }
                "network" => {
                    let network_config = Arc::new(NetworkConfig::from_config(&config.network));
                    let network = NetworkWidget::new(network_config);
                    container.append(network.widget());
                }
                "battery" => {
                    let battery_config = BatteryConfig::from_config(&config.battery);
                    let battery = BatteryWidget::new(battery_config);
                    container.append(battery.widget());
                }
                "audio" => {
                    let audio_config = AudioConfig::from_config(&config.audio);
                    let audio = AudioWidget::new(audio_config);
                    container.append(audio.widget());
                }
                name if name.starts_with("custom/") => {
                    let custom_name = name.strip_prefix("custom/").unwrap();
                    if let Some(custom_config) = config.custom_modules.get(custom_name) {
                        let custom = CustomModuleWidget::new(
                            custom_name,
                            custom_config.action.clone(),
                            custom_config.exec.clone(),
                            custom_config.interval,
                            custom_config.format.clone(),
                        );
                        container.append(custom.widget());
                    } else {
                        eprintln!("Custom module '{}' not found in config", custom_name);
                    }
                }
                name if name.starts_with("box/") => {
                    let box_name = name.strip_prefix("box/").unwrap();
                    if let Some(box_config) = config.boxes.get(box_name) {
                        let box_widget_config = BoxWidgetConfig {
                            modules: box_config.modules.clone(),
                            action: box_config.action.clone(),
                            spacing: box_config.spacing,
                            orientation: box_config
                                .orientation
                                .clone()
                                .unwrap_or_else(|| "horizontal".to_string()),
                        };
                        let box_widget = BoxWidget::new(box_name, box_widget_config, config);
                        container.append(box_widget.widget());
                    } else {
                        eprintln!("Box widget '{}' not found in config", box_name);
                    }
                }
                name if name.starts_with("revealer/") => {
                    let revealer_name = name.strip_prefix("revealer/").unwrap();
                    if let Some(revealer_config) = config.revealers.get(revealer_name) {
                        let revealer_widget_config = RevealerConfig {
                            modules: revealer_config.modules.clone(),
                            spacing: revealer_config.spacing,
                            orientation: revealer_config
                                .orientation
                                .clone()
                                .unwrap_or_else(|| "horizontal".to_string()),
                            trigger: revealer_config.trigger.clone().unwrap_or_default(),
                            transition: revealer_config
                                .transition
                                .clone()
                                .unwrap_or_else(|| "slide_left".to_string()),
                            transition_duration: revealer_config.transition_duration.unwrap_or(200),
                            reveal_on_hover: revealer_config.reveal_on_hover.unwrap_or(false),
                        };
                        let revealer_widget =
                            RevealerWidget::new(revealer_name, revealer_widget_config, config);
                        container.append(revealer_widget.widget());
                    } else {
                        eprintln!("Revealer widget '{}' not found in config", revealer_name);
                    }
                }
                _ => {
                    eprintln!("Unknown module in revealer: {}", name);
                }
            }
        }
    }
}
