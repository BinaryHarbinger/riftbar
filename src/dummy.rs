// ============ dummy.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;

pub struct DummyWidget {
    pub container: gtk::Box,
}

impl DummyWidget {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let button = gtk::Button::with_label("Action");

        button.connect_clicked(|_| {
            println!("Dummy button clicked!");
            // TODO: Implement your feature here
        });

        container.append(&button);

        Self { container }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    // Dummy methods for future implementation
    pub fn update_data(&self, _data: String) {
        // TODO: Update widget with new data
    }

    pub fn refresh(&self) {
        // TODO: Refresh module state
    }
}
