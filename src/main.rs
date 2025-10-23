// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::path::PathBuf;

mod clock;
mod hyprlandworkspaces;
mod mpris;
mod network;

fn main() {
    let app = gtk::Application::new(Some("com.example.AsyncStatusbar"), Default::default());

    app.connect_activate(move |app| {
        load_css();

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

        // Create main grid with 3 equal columns
        let main_grid = gtk::Grid::new();
        main_grid.set_column_homogeneous(true); // All columns same width

        // Left section
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        left_box.set_halign(gtk::Align::Start);

        let mpris = mpris::MprisWidget::new();
        left_box.append(mpris.widget());

        // Center section
        let center_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        center_box.set_halign(gtk::Align::Center);

        let workspaces = hyprlandworkspaces::HyprWorkspacesWidget::new();
        center_box.append(workspaces.widget());

        // Right section
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        right_box.set_halign(gtk::Align::End);

        let network = network::NetworkWidget::new();
        right_box.append(network.widget());
        let clock = clock::ClockWidget::new();
        right_box.append(clock.widget());

        // Attach to grid (column, row, width, height)
        main_grid.attach(&left_box, 0, 0, 1, 1);
        main_grid.attach(&center_box, 1, 0, 1, 1);
        main_grid.attach(&right_box, 2, 0, 1, 1);

        window.set_child(Some(&main_grid));
        window.present();
    });

    app.run();
}

fn load_css() {
    let css_provider = gtk::CssProvider::new();

    // Get CSS path from config directory
    let mut css_path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
    css_path.push(".config/riftbar/style.css");

    if css_path.exists() {
        css_provider.load_from_path(&css_path);
        println!("Loaded CSS from: {:?}", css_path);
    } else {
        println!("CSS file not found at: {:?}", css_path);
    }

    // Apply CSS to default display
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to display"),
        &css_provider,
        900,
    );
}
