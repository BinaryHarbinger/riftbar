// ============ modules/clock.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use libc::{localtime_r, strftime, time};
use std::ffi::{CStr, CString};

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
    pub on_click_middle: String,
    pub on_click_right: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: "%H:%M".to_string(),
            interval: 1,
            tooltip: true,
            tooltip_format: "%A, %B %d, %Y".to_string(),
            on_click: String::new(),
            on_click_middle: String::new(),
            on_click_right: String::new(),
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
            on_click_middle: config.on_click_middle.clone(),
            on_click_right: config.on_click_right.clone(),
        }
    }
}

impl ClockWidget {
    pub fn new(config: ClockConfig) -> Self {
        let button = gtk::Button::with_label("--:--");
        button.set_css_classes(&["clock", "module"]);
        button.set_widget_name("clock");

        // Crate click handlers
        crate::shared::create_gesture_handler(
            &button,
            crate::shared::Gestures {
                on_click: config.on_click,
                on_click_middle: Some(config.on_click_middle),
                on_click_right: Some(config.on_click_right),
                scroll_up: None,
                scroll_down: None,
            },
        );

        // Set up tooltip if enabled
        if config.tooltip {
            let tooltip_format = config.tooltip_format.clone();
            button.set_has_tooltip(true);
            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                let tooltip_text = format_local_time(tooltip_format.as_str());
                tooltip.set_text(Some(&tooltip_text));
                true
            });
        }

        // Set initial label
        button.set_label(&format_local_time(&config.format));

        // Clone button for the closure
        let button_clone = button.clone();

        // Poll for update
        let mut last_label = String::new();
        glib::timeout_add_local(
            std::time::Duration::from_millis(config.interval),
            move || {
                let current_label = &format_local_time(&config.format);
                if last_label.as_str() != current_label {
                    last_label = current_label.clone();
                    button_clone.set_label(current_label);
                }
                glib::ControlFlow::Continue
            },
        );

        Self { button }
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }
}

fn format_local_time(fmt: &str) -> String {
    unsafe {
        let mut t: libc::time_t = 0;
        time(&mut t);

        let mut tm: libc::tm = std::mem::zeroed();
        localtime_r(&t, &mut tm);

        let mut buf = [0u8; 128];
        let c_fmt = CString::new(fmt).unwrap();

        strftime(buf.as_mut_ptr() as *mut _, buf.len(), c_fmt.as_ptr(), &tm);

        CStr::from_ptr(buf.as_ptr() as *const _)
            .to_string_lossy()
            .into_owned()
    }
}
