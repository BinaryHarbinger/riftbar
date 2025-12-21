// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::sync::Arc;

mod config;
mod modules;

fn main() {
    let config = config::Config::load();

    let app = gtk::Application::new(Some("com.binaryharb.RiftBar"), Default::default());

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
        layout_container.add_css_class("you-should-not-use-this-class");

        // Left section
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        left_box.set_halign(gtk::Align::Start);
        left_box.set_hexpand(true);
        left_box.set_vexpand(false);
        left_box.add_css_class("left-section");
        build_modules(&left_box, &config.modules_left, &config, 0);

        // Center section
        let center_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        center_box.set_halign(gtk::Align::Center);
        center_box.set_hexpand(true);
        center_box.set_vexpand(false);
        center_box.add_css_class("center-section");
        build_modules(&center_box, &config.modules_center, &config, 0);

        // Right section
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, config.bar.spacing);
        right_box.set_halign(gtk::Align::End);
        right_box.set_hexpand(true);
        right_box.set_vexpand(false);
        right_box.add_css_class("right-section");
        build_modules(&right_box, &config.modules_right, &config, 0);

        // Attach to center box - each section gets equal width
        layout_container.set_start_widget(Some(&left_box));
        layout_container.set_center_widget(Some(&center_box));
        layout_container.set_end_widget(Some(&right_box));

        window.set_child(Some(&layout_container));

        // Load CSS after window is set up
        apply_css_to_gtk();

        window.present();
    });

    app.run();
}

fn build_modules(
    container: &gtk::Box,
    module_names: &[String],
    config: &config::Config,
    container_type: i32,
) {
    let container_name = match container_type {
        0 => "",
        1 => " in box",
        2 => " in revealer",
        _ => "",
    };

    println!("Building modules{}: {:?}", container_name, module_names);

    for name in module_names {
        match name.as_str() {
            "clock" => {
                let clock_config = modules::ClockConfig::from_config(&config.clock);
                let clock = modules::ClockWidget::new(clock_config);
                container.append(clock.widget());
            }
            "tray" => {
                let tray_config = modules::TrayConfig {
                    spacing: config.tray.spacing,
                    icon_size: config.tray.icon_size,
                };
                let tray = modules::TrayWidget::new(tray_config);
                container.append(tray.widget());
            }
            "hyprland/workspaces" => {
                let workspaces_config =
                    Arc::new(modules::WorkspacesConfig::from_config(&config.workspaces));
                let workspaces = modules::HyprWorkspacesWidget::new(workspaces_config);
                container.append(workspaces.widget());
            }
            "mpris" => {
                let mpris_config = modules::MprisConfig::from_config(&config.mpris);
                let mpris = modules::MprisWidget::new(mpris_config);
                container.append(mpris.widget());
            }
            "network" => {
                let network_config = Arc::new(modules::NetworkConfig::from_config(&config.network));
                let network = modules::NetworkWidget::new(network_config);
                container.append(network.widget());
            }
            "battery" => {
                let battery_config = Arc::new(modules::BatteryConfig::from_config(&config.battery));
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
                        action: box_config.action.clone(),
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
            name if name.starts_with("revealer/") => {
                let revealer_name = name.strip_prefix("revealer/").unwrap();
                if let Some(revealer_config) = config.revealers.get(revealer_name) {
                    let revealer_widget_config = modules::RevealerConfig {
                        modules: revealer_config.modules.clone(),
                        spacing: revealer_config.spacing,
                        orientation: revealer_config
                            .orientation
                            .clone()
                            .unwrap_or_else(|| "horizontal".to_string()),
                        trigger: revealer_config.trigger.clone().unwrap_or_default(),
                        transition: revealer_config
                            .transition
                            .clone()
                            .unwrap_or_else(|| "slide_left".to_string()),
                        transition_duration: revealer_config.transition_duration.unwrap_or(200),
                        reveal_on_hover: revealer_config.reveal_on_hover.unwrap_or(false),
                    };
                    let revealer_widget =
                        modules::RevealerWidget::new(revealer_name, revealer_widget_config, config);
                    container.append(revealer_widget.widget());
                }
            }
            _ => {
                eprintln!("Unknown module: {}", name);
            }
        }
    }
}

fn apply_css_to_gtk() {
    let css = match config::load_css_string() {
        Some(css) => css,
        None => {
            println!("No style.scss or style.css found, skipping CSS");
            return;
        }
    };

    let provider = gtk::CssProvider::new();
    provider.load_from_data(&css);

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &provider, 950);
        println!("CSS applied to GTK");
    } else {
        eprintln!("Failed to get default GTK display");
    }
}
