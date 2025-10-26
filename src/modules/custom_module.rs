// ============ custom_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;
use tokio::process::Command;

pub struct CustomModuleWidget {
    button: gtk::Button,
}

impl CustomModuleWidget {
    pub fn new(
        name: &str,
        action: String,
        exec: String,
        interval: u64,
        format: Option<String>,
    ) -> Self {
        let button = gtk::Button::with_label("Loading...");
        button.add_css_class("custom-module");
        button.add_css_class(&format!("custom-{}", name));

        let widget = Self {
            button: button.clone(),
        };

        // Left click handler
        button.connect_clicked(move |_| {
            Self::run_action_async(action.clone());
        });

        widget.start_updates(exec, interval, format);

        widget
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    fn start_updates(&self, exec: String, interval: u64, format: Option<String>) {
        let button = self.button.clone();
        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    let output = Command::new("sh").arg("-c").arg(&exec).output().await;

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

                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                }
            });
        });

        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                button.set_label(&msg);
            }
            glib::ControlFlow::Continue
        });
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
