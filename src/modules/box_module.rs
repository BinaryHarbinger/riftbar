// ============ modules/box_widget.rs ============

use gtk4 as gtk;
use gtk4::prelude::*;
use tokio::process::Command;

pub struct BoxWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct BoxWidgetConfig {
    pub modules: Vec<String>,
    pub action: String,
    pub spacing: i32,
    pub orientation: String,
}

impl BoxWidget {
    pub fn new(name: &str, config: BoxWidgetConfig, app_config: &crate::config::Config) -> Self {
        // Determine orientation
        let orientation = match config.orientation.as_str() {
            "vertical" => gtk::Orientation::Vertical,
            _ => gtk::Orientation::Horizontal,
        };

        let container = gtk::Box::new(orientation, config.spacing);
        container.add_css_class("box-widget");
        container.add_css_class(&format!("box-{}", name));

        // Assign a click listener
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            Self::run_action_async(config.action.clone());
        });
        container.add_controller(gesture);

        // Build the modules inside this box
        Self::build_modules(&container, &config.modules, app_config);

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

        println!("Building modules in box: {:?}", module_names);

        for name in module_names {
            match name.as_str() {
                "clock" => {
                    let clock_config = ClockConfig::from_config(&config.clock);
                    let clock = ClockWidget::new(clock_config);
                    container.append(clock.widget());
                }
                "hyprland/workspaces" => {
                    let workspaces = HyprWorkspacesWidget::new();
                    container.append(workspaces.widget());
                }
                "mpris" => {
                    let mpris_config = MprisConfig::from_config(&config.mpris);
                    let mpris = MprisWidget::new(mpris_config);
                    container.append(mpris.widget());
                }
                "network" => {
                    let network_config = NetworkConfig::from_config(&config.network);
                    let network = NetworkWidget::new(network_config);
                    container.append(network.widget());
                }
                "audio" => {
                    let audio_config = AudioConfig::from_config(&config.audio);
                    let audio = AudioWidget::new(audio_config);
                    container.append(audio.widget());
                }
                "battery" => {
                    let battery_config = BatteryConfig::from_config(&config.battery);
                    let battery = BatteryWidget::new(battery_config);
                    container.append(battery.widget());
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
                    // Support nested boxes
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
                _ => {
                    eprintln!("Unknown module in box: {}", name);
                }
            }
        }
    }
    fn run_action_async(action: String) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(action.clone())
                    .output()
                    .await;
            });
        });
    }
}
