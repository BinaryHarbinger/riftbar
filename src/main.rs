// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;

mod config;
mod modules;

fn main() {
    let config = config::Config::load();

    let app = gtk::Application::new(Some("com.example.RiftBar"), Default::default());

    app.connect_activate(move |app| {
        let window = gtk::Window::new();

        // Initialize layer shell
        window.init_layer_shell();

        // Set layer from config
        let layer = match config.bar.layer.as_str() {
            "background" => gtk4_layer_shell::Layer::Background,
            "bottom" => gtk4_layer_shell::Layer::Bottom,
            "overlay" => gtk4_layer_shell::Layer::Overlay,
            _ => gtk4_layer_shell::Layer::Top,
        };
        window.set_layer(layer);

        // Set anchors based on position
        match config.bar.position.as_str() {
            "bottom" => {
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }
            _ => {
                // top
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }
        }

        window.set_namespace(Some("riftbar"));
        window.auto_exclusive_zone_enable();
        window.set_application(Some(app));
        window.add_css_class("riftbar");

        // Use a center box for proper three-column layout
        let layout_container = gtk::CenterBox::new();
        layout_container.add_css_class("main-bar");

        // Left section
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        left_box.set_halign(gtk::Align::Start);
        left_box.set_hexpand(true);
        left_box.set_vexpand(false);
        left_box.add_css_class("left-section");
        build_modules(&left_box, &config.modules_left, &config);

        // Center section
        let center_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        center_box.set_halign(gtk::Align::Center);
        center_box.set_hexpand(true);
        center_box.set_vexpand(false);
        center_box.add_css_class("center-section");
        build_modules(&center_box, &config.modules_center, &config);

        // Right section
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        right_box.set_halign(gtk::Align::End);
        right_box.set_hexpand(true);
        right_box.set_vexpand(false);
        right_box.add_css_class("right-section");
        build_modules(&right_box, &config.modules_right, &config);

        // Attach to center box - each section gets equal width
        layout_container.set_start_widget(Some(&left_box));
        layout_container.set_center_widget(Some(&center_box));
        layout_container.set_end_widget(Some(&right_box));

        window.set_child(Some(&layout_container));

        // Load CSS after window is set up
        load_css();
        start_css_watcher();

        window.present();
    });

    app.run();
}

fn build_modules(container: &gtk::Box, module_names: &[String], config: &config::Config) {
    println!("Building modules: {:?}", module_names);
    for name in module_names {
        match name.as_str() {
            "clock" => {
                let clock = modules::ClockWidget::new();
                container.append(clock.widget());
            }
            "hyprland/workspaces" => {
                let workspaces = modules::HyprWorkspacesWidget::new();
                container.append(workspaces.widget());
            }
            "mpris" => {
                let mpris_config = modules::MprisConfig::from_config(&config.mpris);
                let mpris = modules::MprisWidget::new(mpris_config);
                container.append(mpris.widget());
            }
            "network" => {
                let network_config = modules::NetworkConfig::from_config(&config.network);
                let network = modules::NetworkWidget::new(network_config);
                container.append(network.widget());
            }
            "battery" => {
                let battery_config = modules::BatteryConfig::from_config(&config.battery);
                let battery = modules::BatteryWidget::new(battery_config);
                container.append(battery.widget());
            }
            "audio" => {
                let audio_config = modules::AudioConfig::from_config(&config.audio);
                let audio = modules::AudioWidget::new(audio_config);
                container.append(audio.widget());
            }
            name if name.starts_with("custom/") => {
                let custom_name = name.strip_prefix("custom/").unwrap();
                if let Some(custom_config) = config.custom_modules.get(custom_name) {
                    let custom = modules::CustomModuleWidget::new(
                        custom_name,
                        custom_config.action.clone(),
                        custom_config.exec.clone(),
                        custom_config.interval,
                        custom_config.format.clone(),
                    );
                    container.append(custom.widget());
                }
            }
            name if name.starts_with("box/") => {
                let box_name = name.strip_prefix("box/").unwrap();
                if let Some(box_config) = config.boxes.get(box_name) {
                    let box_widget_config = modules::BoxWidgetConfig {
                        modules: box_config.modules.clone(),
                        spacing: box_config.spacing,
                        orientation: box_config
                            .orientation
                            .clone()
                            .unwrap_or_else(|| "horizontal".to_string()),
                    };
                    let box_widget = modules::BoxWidget::new(box_name, box_widget_config, config);
                    container.append(box_widget.widget());
                }
            }
            _ => {
                eprintln!("Unknown module: {}", name);
            }
        }
    }
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
        return;
    }

    // Apply CSS to default display
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &css_provider, 950);
    } else {
        eprintln!("Could not get default display for CSS");
    }
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
