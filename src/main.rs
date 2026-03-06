// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::{
    cell::RefCell,
    env, fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    sync::mpsc,
};

mod config;
mod modules;
mod shared;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut config_path = config::Config::get_config_path();

    let args: Vec<String> = env::args().collect();

    let mut use_gpu = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i].as_str();
        if matches!(*arg, "-c" | "--config") {
            if i + 1 < args.len() {
                config_path = expand_tilde(&args[i + 1]);
                i += 2;
            } else {
                std::process::exit(1);
            }
        } else if matches!(*arg, "-v" | "--version") {
            println!("Riftbar v{}", VERSION);
            std::process::exit(1);
        } else if *arg == "--use-gpu" {
            use_gpu = true;
        } else if matches!(*arg, "--ipc" | "-i") {
            if i + 1 < args.len() {
                println!("[IPC]: Triggered ipc command: {}", args[i + 1]);
                let _ = ipc_writer(&args[i + 1], &args[i + 2]);
                std::process::exit(0)
            } else {
                println!(
                    "[IPC]: Error, IPC needs at least two arguments\n    toggle <bar.name> \n      open <bar.name> \n     close <bar.name>"
                );
                std::process::exit(1)
            }
        } else if arg.starts_with("-") {
            eprintln!("Unknown option: {}", args[i]);
            std::process::exit(1);
        } else {
            i += 1;
        }
    }

    let config = config::Config::load(config_path);

    if !use_gpu || !config.general.use_gpu {
        unsafe {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
    }

    let app = gtk::Application::new(Some("com.binaryharb.RiftBar"), Default::default());

    // Stays on the GTK main thread only — Rc<RefCell> is fine here
    let window_map: Rc<RefCell<Vec<(String, gtk::Window, bool)>>> =
        Rc::new(RefCell::new(Vec::new()));

    app.connect_activate({
        let window_map = Rc::clone(&window_map);
        move |app| {
            for (name, bar_config) in &config.bars {
                let window = gtk::Window::new();

                // Initialize layer shell
                window.init_layer_shell();

                window.set_namespace(Some(&bar_config.namespace));
                if bar_config.reserve_space {
                    window.auto_exclusive_zone_enable();
                } else {
                    LayerShell::set_exclusive_zone(&window, 0);
                }
                window.set_application(Some(app));
                window.add_css_class(name);
                window.add_css_class("bar-container");

                // Set layer from config
                let layer = match bar_config.layer.as_str() {
                    "background" => gtk4_layer_shell::Layer::Background,
                    "bottom" => gtk4_layer_shell::Layer::Bottom,
                    "overlay" => gtk4_layer_shell::Layer::Overlay,
                    _ => gtk4_layer_shell::Layer::Top,
                };
                window.set_layer(layer);

                // Set anchors based on position
                match bar_config.position.as_str() {
                    "bottom" => {
                        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
                    }
                    "left" => {
                        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                    }
                    "right" => {
                        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
                    }
                    _ => {
                        // top
                        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
                    }
                }

                let orientation = match bar_config.position.as_str() {
                    "right" | "left" => gtk::Orientation::Vertical,
                    _ => gtk::Orientation::Horizontal,
                };

                if bar_config.position.as_str() != "right" && bar_config.position.as_str() != "left"
                {
                    // Use a center box for proper three-column layout
                    let layout_container = gtk::CenterBox::new();
                    layout_container.add_css_class("riftbar");

                    // Left section
                    let left_box = gtk::Box::new(orientation, bar_config.spacing);
                    left_box.set_halign(gtk::Align::Start);
                    left_box.set_hexpand(true);
                    left_box.set_vexpand(false);
                    left_box.add_css_class("left-section");
                    if bar_config.modules_left.is_some() {
                        build_modules(
                            &left_box,
                            &bar_config.modules_left.clone().unwrap_or_default(),
                            &config,
                            0,
                        );
                    }

                    // Center section
                    let center_box = gtk::Box::new(orientation, bar_config.spacing);
                    center_box.set_halign(gtk::Align::Center);
                    center_box.set_hexpand(true);
                    center_box.set_vexpand(false);
                    center_box.add_css_class("center-section");
                    if bar_config.modules_center.is_some() {
                        build_modules(
                            &center_box,
                            &bar_config.modules_center.clone().unwrap_or_default(),
                            &config,
                            0,
                        );
                    }

                    // Right section
                    let right_box = gtk::Box::new(orientation, bar_config.spacing);
                    right_box.set_halign(gtk::Align::End);
                    right_box.set_hexpand(true);
                    right_box.set_vexpand(false);
                    right_box.add_css_class("right-section");
                    if bar_config.modules_right.is_some() {
                        build_modules(
                            &right_box,
                            &bar_config.modules_right.clone().unwrap_or_default(),
                            &config,
                            0,
                        );
                    }

                    // Attach to center box - each section gets equal width
                    layout_container.set_start_widget(Some(&left_box));
                    layout_container.set_center_widget(Some(&center_box));
                    layout_container.set_end_widget(Some(&right_box));

                    // Set css class
                    layout_container.add_css_class(name);

                    window.set_child(Some(&layout_container));
                } else {
                    let layout_container = gtk::Overlay::new();
                    layout_container.add_css_class("riftbar");

                    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

                    let start_box = gtk::Box::new(orientation, bar_config.spacing);
                    start_box.set_halign(gtk::Align::Fill);
                    start_box.set_hexpand(true);
                    start_box.add_css_class("left-section");
                    build_modules(
                        &start_box,
                        &bar_config.modules_left.clone().unwrap_or_default(),
                        &config,
                        0,
                    );

                    main_vbox.append(&start_box);

                    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    spacer.set_vexpand(true);
                    main_vbox.append(&spacer);

                    let end_box = gtk::Box::new(orientation, bar_config.spacing);
                    end_box.set_halign(gtk::Align::Fill);
                    end_box.set_hexpand(true);
                    end_box.add_css_class("right-section");
                    build_modules(
                        &end_box,
                        &bar_config.modules_right.clone().unwrap_or_default(),
                        &config,
                        0,
                    );

                    main_vbox.append(&end_box);

                    layout_container.set_child(Some(&main_vbox));

                    let center_box = gtk::Box::new(orientation, bar_config.spacing);
                    center_box.add_css_class("center-section");
                    build_modules(
                        &center_box,
                        &bar_config.modules_center.clone().unwrap_or_default(),
                        &config,
                        0,
                    );

                    layout_container.add_overlay(&center_box);
                    center_box.set_halign(gtk::Align::Center);
                    center_box.set_valign(gtk::Align::Center);

                    // Set css class
                    layout_container.add_css_class(name);

                    window.set_child(Some(&layout_container));
                }

                window_map
                    .borrow_mut()
                    .push((name.clone(), window, bar_config.open_on_launch));
            }

            // Load CSS after window is set up
            apply_css_to_gtk();

            for (_, window, open_on_launch) in window_map.borrow().iter() {
                if *open_on_launch {
                    window.present();
                }
            }

            if config.general.enable_ipc {
                println!("Starting IPC...");
                let (tx, rx) = mpsc::channel::<(String, String)>();
                ipc_listener(tx);

                // Poll the channel on the GTK main thread every 50ms
                let window_map = Rc::clone(&window_map);
                gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                    while let Ok((command, target)) = rx.try_recv() {
                        let map = window_map.borrow();
                        let matched: Vec<&gtk::Window> = if target == "*" {
                            map.iter().map(|(_, w, _)| w).collect()
                        } else {
                            map.iter()
                                .filter(|(name, _, _)| name == &target)
                                .map(|(_, w, _)| w)
                                .collect()
                        };

                        if matched.is_empty() {
                            eprintln!("IPC: no bar named '{}'", target);
                            continue;
                        }

                        for window in matched {
                            match command.as_str() {
                                "toggle" => {
                                    if window.is_visible() {
                                        window.set_visible(false);
                                    } else {
                                        window.present();
                                    }
                                }
                                "open" => {
                                    window.present();
                                }
                                "close" => {
                                    window.set_visible(false);
                                }
                                other => {
                                    eprintln!("IPC: unknown command '{}'", other);
                                }
                            }
                        }
                    }
                    gtk::glib::ControlFlow::Continue
                });
            }
        }
    });

    app.run_with_args::<String>(&[]);
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

    let container_orientation = container.orientation();

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
                let tray = modules::TrayWidget::new(tray_config, container_orientation);
                container.append(tray.widget());
            }
            "hyprland/workspaces" => {
                let workspaces_config = Arc::new(modules::WorkspacesConfig::from_config(
                    &config.workspaces,
                    container_orientation,
                ));
                let workspaces = modules::HyprWorkspacesWidget::new(workspaces_config);
                container.append(workspaces.widget());
            }
            "active_window" => {
                let act_win_config =
                    modules::ActiveWindowConfig::from_config(&config.active_window);
                let act_win = modules::ActiveWindowWidget::new(act_win_config);
                container.append(act_win.widget());
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
                    let custom = modules::CustomModuleWidget::new(modules::CustomModuleConfig {
                        name: custom_name,
                        on_click: custom_config.on_click.clone(),
                        on_click_right: custom_config.on_click_right.clone(),
                        on_click_middle: custom_config.on_click_middle.clone(),
                        scroll_up: custom_config.scroll_up.clone(),
                        scroll_down: custom_config.scroll_down.clone(),
                        exec: custom_config.exec.clone(),
                        interval: custom_config.interval,
                        format: custom_config.format.clone(),
                    });
                    container.append(custom.widget());
                }
            }
            name if name.starts_with("box/") => {
                let box_name = name.strip_prefix("box/").unwrap();
                if let Some(box_config) = config.boxes.get(box_name) {
                    let box_widget_config = modules::BoxWidgetConfig {
                        modules: box_config.modules.clone(),
                        gestures: crate::shared::Gestures {
                            on_click: box_config.on_click.clone(),
                            on_click_middle: box_config.on_click_middle.clone(),
                            on_click_right: box_config.on_click_right.clone(),
                            scroll_up: box_config.scroll_up.clone(),
                            scroll_down: box_config.scroll_down.clone(),
                        },
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

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(stripped)
    } else if path == "~" {
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from(path)
    }
}

#[derive(Debug)]
struct IpcMessage {
    command: String,
    target: String,
}

impl IpcMessage {
    fn parse(line: &str) -> Option<Self> {
        let mut parts = line.splitn(2, ' ');
        let command = parts.next()?.to_string();
        let target = parts.next()?.to_string();
        Some(IpcMessage { command, target })
    }
}

/// Spawns a background thread that listens on the Unix socket and forwards
/// (command, target) string pairs through an mpsc channel.
/// Only plain Strings cross the thread boundary — no GTK types involved.
fn ipc_listener(tx: mpsc::Sender<(String, String)>) {
    let path = "/tmp/riftbar.sock";
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path).expect("Failed to bind IPC socket");

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("IPC accept error: {}", e);
                    continue;
                }
            };

            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("IPC read error: {}", e);
                        break;
                    }
                };

                match IpcMessage::parse(&line) {
                    Some(msg) => {
                        println!("IPC: command='{}' target='{}'", msg.command, msg.target);
                        // Only Strings cross the thread boundary — Send safe
                        let _ = tx.send((msg.command, msg.target));
                    }
                    None => eprintln!("Malformed IPC message: {:?}", line),
                }
            }
        }
    });
}

fn ipc_writer(command: &str, target: &str) -> std::io::Result<()> {
    let mut stream = UnixStream::connect("/tmp/riftbar.sock")?;
    writeln!(stream, "{} {}", command, target)?;
    Ok(())
}
