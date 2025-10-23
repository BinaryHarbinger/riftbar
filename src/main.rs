use gtk4::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::LayerShell;
use tokio::process::Command;
use std::sync::mpsc;

fn main() {
    // Create GTK application
    let app = gtk::Application::new(
        Some("com.example.AsyncStatusbar"),
        Default::default()
    );

    app.connect_activate(move |app| {
        // Create a regular Window
        let window = gtk::Window::new();
        
        // Initialize layer shell using trait methods
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Top);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_namespace(Some("async-statusbar"));
        
        // Reserve space so other windows don't overlap
        window.auto_exclusive_zone_enable();

        // Set the application for the window
        window.set_application(Some(app));

        // Create label
        let label = gtk::Label::new(Some("Loading..."));
        window.set_child(Some(&label));
        
        // Create channel for communication
        let (sender, receiver) = mpsc::channel::<String>();
        
        // Clone label for the closure
        let label_clone = label.clone();
        
        // Spawn tokio runtime in a separate thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    // Run your system command here
                    let output = Command::new("date")
                        .arg("+%H:%M")
                        .output()
                        .await
                        .unwrap();
                    
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let _ = sender.send(result);
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        });
        
        // Poll for messages using glib timeout
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(msg) = receiver.try_recv() {
                label_clone.set_label(&msg);
            }
            glib::ControlFlow::Continue
        });

        window.present();
    });

    // Run the application
    app.run();
}
