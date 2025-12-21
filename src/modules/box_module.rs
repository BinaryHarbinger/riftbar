// ============ modules/box_widget.rs ============

use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::Arc;
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
        crate::build_modules(&container, &config.modules, app_config, 1);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
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
