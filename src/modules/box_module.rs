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

        // Crate click handlers
        crate::shared::create_gesture_handler(
            &container,
            crate::shared::Gestures {
                on_click: config.on_click,
                on_click_middle: None,
                on_click_right: None,
                scroll_up: None,
                scroll_down: None,
            },
        );

        // Build the modules inside this box
        crate::build_modules(&container, &config.modules, app_config, 1);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
}
