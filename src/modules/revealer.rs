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
        crate::build_modules(&content_box, &config.modules, app_config, 2);
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
}
