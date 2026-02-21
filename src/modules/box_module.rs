// ============ modules/box_widget.rs ============

use gtk4 as gtk;
use gtk4::prelude::*;
// use std::sync::Arc;

pub struct BoxWidget {
    container: gtk::Box,
}

#[derive(Clone)]
pub struct BoxWidgetConfig {
    pub modules: Vec<String>,
    pub on_click: String,
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
        if !config.on_click.is_empty() {
            let gesture = gtk::GestureClick::new();
            gesture.connect_released(move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                crate::shared::run_shell_command(&config.on_click);
            });
            container.add_controller(gesture);
        }
        // Build the modules inside this box
        crate::build_modules(&container, &config.modules, app_config, 1);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
}
