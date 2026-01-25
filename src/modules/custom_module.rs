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

impl CustomModuleWidget {
    pub fn new(
        name: &str,
        on_click: String,
        on_click_right: String,
        on_click_middle: String,
        exec: String,
        interval: u64,
        format: Option<String>,
    ) -> Self {
        let label = gtk::Label::new(None);
        let button = gtk::Button::new();
        button.set_child(Some(&label));
        button.add_css_class("custom-module");
        button.add_css_class(&format!("custom-{}", name));

        let widget = Self {
            button: button.clone(),
            label: label.clone(),
        };

        // Left click handler
        if !on_click.is_empty() {
            button.connect_clicked(move |_| {
                crate::shared::run_command_async(on_click.clone());
            });
        }

        // Middle and right click handler
        if !on_click_middle.is_empty() && !on_click_right.is_empty() {
            let gesture = gtk::GestureClick::new();
            gesture.set_button(0); // Listen to all buttons

            gesture.connect_released(move |gesture, _, _, _| {
                let button_num = gesture.current_button();
                match button_num {
                    2 => {
                        // Middle Click
                        crate::shared::run_command_async(on_click_middle.clone());
                    }
                    3 => {
                        // Right Click
                        crate::shared::run_command_async(on_click_right.clone());
                    }
                    _ => {}
                }
            });
            button.add_controller(gesture);
        }

        widget.start_updates(exec, interval, format);

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
                    let _ = sender.send(format.unwrap_or(String::new()));
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
