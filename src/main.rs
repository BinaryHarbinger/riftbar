// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;

mod clock;
mod hyprlandworkspaces;
mod mpris;
mod network;

fn main() {
    let app = gtk::Application::new(Some("com.example.AsyncStatusbar"), Default::default());

    app.connect_activate(move |app| {
        load_css();
        start_css_watcher();

        let window = gtk::Window::new();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Top);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_namespace(Some("riftbar"));
        window.auto_exclusive_zone_enable();
        window.set_application(Some(app));

        // Add css class to main window
        window.add_css_class("riftbar");

        // Create main grid with 3 equal columns
        let main_grid = gtk::Grid::new();
        main_grid.set_column_homogeneous(true);

        // Left section
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        left_box.set_halign(gtk::Align::Start);
        let mpris = mpris::MprisWidget::new();
        left_box.append(mpris.widget());

        // Center section
        let center_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        center_box.set_halign(gtk::Align::Center);
        let workspaces = hyprlandworkspaces::HyprWorkspacesWidget::new();
        center_box.append(workspaces.widget());

        // Right section
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        right_box.set_halign(gtk::Align::End);
        let network = network::NetworkWidget::new();
        right_box.append(network.widget());
        let clock = clock::ClockWidget::new();
        right_box.append(clock.widget());

        // Attach to grid
        main_grid.attach(&left_box, 0, 0, 1, 1);
        main_grid.attach(&center_box, 1, 0, 1, 1);
        main_grid.attach(&right_box, 2, 0, 1, 1);

        window.set_child(Some(&main_grid));
        window.present();
    });

    app.run();
}

fn get_config_dir() -> PathBuf {
    let mut config_path =
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
    config_path.push(".config/riftbar");
    config_path
}

fn compile_scss_if_needed() -> Option<PathBuf> {
    let config_dir = get_config_dir();
    let scss_path = config_dir.join("style.scss");
    let css_path = config_dir.join("style.css");

    // If SCSS exists, compile it
    if scss_path.exists() {
        println!("Compiling SCSS: {:?}", scss_path);

        let output = Command::new("sass").arg(&scss_path).arg(&css_path).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("SCSS compiled successfully");
                    return Some(css_path);
                } else {
                    eprintln!(
                        "SCSS compilation failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    // Fall back to CSS if it exists
                    if css_path.exists() {
                        return Some(css_path);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to run sass command: {}. Make sure 'sass' is installed.",
                    e
                );
                // Fall back to CSS if it exists
                if css_path.exists() {
                    return Some(css_path);
                }
            }
        }
    } else if css_path.exists() {
        return Some(css_path);
    }

    None
}

fn load_css() {
    let css_provider = gtk::CssProvider::new();

    if let Some(css_path) = compile_scss_if_needed() {
        css_provider.load_from_path(&css_path);
        println!("Loaded CSS from: {:?}", css_path);
    } else {
        println!("No CSS or SCSS file found in config directory");
    }

    // Apply CSS to default display
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to display"),
        &css_provider,
        950,
    );
}

fn start_css_watcher() {
    let (sender, receiver) = mpsc::channel::<()>();

    // Watch for file changes in a separate thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            use tokio::time::{Duration, sleep};

            let config_dir = get_config_dir();
            let scss_path = config_dir.join("style.scss");
            let css_path = config_dir.join("style.css");

            let mut last_scss_modified = scss_path.metadata().and_then(|m| m.modified()).ok();
            let mut last_css_modified = css_path.metadata().and_then(|m| m.modified()).ok();

            loop {
                sleep(Duration::from_millis(500)).await;

                let scss_modified = scss_path.metadata().and_then(|m| m.modified()).ok();
                let css_modified = css_path.metadata().and_then(|m| m.modified()).ok();

                if scss_modified != last_scss_modified || css_modified != last_css_modified {
                    println!("CSS/SCSS file changed, reloading...");
                    let _ = sender.send(());
                    last_scss_modified = scss_modified;
                    last_css_modified = css_modified;
                }
            }
        });
    });

    // Reload CSS when notified
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if receiver.try_recv().is_ok() {
            load_css();
        }
        glib::ControlFlow::Continue
    });
}
