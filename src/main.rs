// ============ main.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

mod clock;
mod hyprlandworkspaces;
mod mpris;

fn main() {
    let app = gtk::Application::new(Some("com.example.AsyncStatusbar"), Default::default());

    app.connect_activate(move |app| {
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

        // Create main horizontal box
        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Left section
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        left_box.set_hexpand(true);
        left_box.set_halign(gtk::Align::Start);
        let mpris = mpris::MprisWidget::new();
        left_box.append(mpris.widget());

        // Center section
        let center_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        center_box.set_hexpand(true);
        center_box.set_halign(gtk::Align::Center);
        let workspaces = hyprlandworkspaces::HyprWorkspacesWidget::new();
        center_box.append(workspaces.widget());
        // Right section
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk::Align::End);
        let clock = clock::ClockWidget::new();
        right_box.append(clock.widget());

        // Pack everything
        main_box.append(&left_box);
        main_box.append(&center_box);
        main_box.append(&right_box);

        window.set_child(Some(&main_box));
        window.present();
    });

    app.run();
}
