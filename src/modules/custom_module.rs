// ============ custom_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::process::Command;
use std::sync::mpsc;
use std::thread::sleep;

pub struct CustomModuleWidget {
    button: gtk::Button,
    label: gtk::Label,
}

pub struct CustomModuleConfig<'a> {
    pub name: &'a str,
    pub on_click: String,
    pub on_click_right: String,
    pub on_click_middle: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub exec: String,
    pub interval: u64,
    pub format: Option<String>,
}

impl CustomModuleWidget {
    pub fn new(config: CustomModuleConfig) -> Self {
        let label = gtk::Label::new(None);
        let button = gtk::Button::new();
        button.set_child(Some(&label));
        button.add_css_class("custom-module");
        button.add_css_class(&format!("custom-{}", config.name));

        let widget = Self {
            button: button.clone(),
            label: label.clone(),
        };

        // Left click handler
        if !config.on_click.is_empty() {
            button.connect_clicked(move |_| {
                crate::shared::run_shell_command(config.on_click.clone());
            });
        }

        // Middle and right click handler
        if !config.on_click_middle.is_empty() && !config.on_click_right.is_empty() {
            let gesture = gtk::GestureClick::new();
            gesture.set_button(0); // Listen to all buttons

            gesture.connect_released(move |gesture, _, _, _| {
                let button_num = gesture.current_button();
                match button_num {
                    2 => {
                        // Middle Click
                        crate::shared::run_shell_command(config.on_click_middle.clone());
                    }
                    3 => {
                        // Right Click
                        crate::shared::run_shell_command(config.on_click_right.clone());
                    }
                    _ => {}
                }
            });
            button.add_controller(gesture);
        }

        // Scroll handler
        if !config.scroll_up.is_empty() || !config.scroll_down.is_empty() {
            let scroll_controller =
                gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
            scroll_controller.connect_scroll(move |_, _, dy| {
                if dy < 0.0 {
                    // Scroll up
                    if !config.scroll_up.is_empty() {
                        crate::shared::run_shell_command(config.scroll_up.clone());
                    }
                } else {
                    // Scroll down
                    if !config.scroll_down.is_empty() {
                        crate::shared::run_shell_command(config.scroll_down.clone());
                    }
                }
                gtk4::glib::Propagation::Stop
            });
            button.add_controller(scroll_controller);
        }

        widget.start_updates(config.exec, config.interval, config.format);

        widget
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn start_updates(&self, exec: String, interval: u64, format: Option<String>) {
        let label = self.label.clone();
        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            loop {
                if !exec.is_empty() {
                    let output = Command::new("sh").arg("-c").arg(&exec).output();

                    match output {
                        Ok(output) => {
                            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            let formatted = if let Some(ref fmt) = format {
                                fmt.replace("{}", &result)
                            } else {
                                result
                            };
                            let _ = sender.send(formatted);
                        }
                        Err(e) => {
                            eprintln!("Custom module exec failed: {}", e);
                        }
                    }
                } else {
                    let _ = sender.send(format.unwrap_or_default());
                    break;
                }

                sleep(std::time::Duration::from_secs(interval));
            }
        });

        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                label.set_markup(&msg);
            }
            glib::ControlFlow::Continue
        });
    }
}
