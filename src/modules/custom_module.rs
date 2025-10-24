// ============ custom_module.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use std::sync::mpsc;
use tokio::process::Command;

pub struct CustomModuleWidget {
    container: gtk::Box,
    label: gtk::Label,
}

impl CustomModuleWidget {
    pub fn new(name: &str, exec: String, interval: u64, format: Option<String>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add_css_class("custom-module");
        container.add_css_class(&format!("custom-{}", name));

        let label = gtk::Label::new(Some("Loading..."));
        container.append(&label);

        let widget = Self { container, label };

        widget.start_updates(exec, interval, format);

        widget
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn start_updates(&self, exec: String, interval: u64, format: Option<String>) {
        let label = self.label.clone();
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
                label.set_label(&msg);
            }
            glib::ControlFlow::Continue
        });
    }
}
