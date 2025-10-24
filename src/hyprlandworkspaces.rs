// ============ hyprland-workspaces.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use hyprland::data::*;
use hyprland::shared::{HyprData, HyprDataActive};
use std::sync::mpsc;

pub struct HyprWorkspacesWidget {
    pub container: gtk::Box,
}

impl HyprWorkspacesWidget {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        container.set_css_classes(&["workspaces"]);

        let widget = Self { container };

        // Start the update loop
        widget.start_updates();

        widget
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn start_updates(&self) {
        let container = self.container.clone();
        let (sender, receiver) = mpsc::channel::<(Vec<i32>, i32)>();

        // Spawn thread to get workspace info
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    // Get workspaces and active workspace
                    let result = match Workspaces::get() {
                        Ok(ws) => {
                            let mut workspaces: Vec<_> = ws.into_iter().collect();
                            workspaces.sort_by_key(|w| w.id);

                            let workspace_ids: Vec<i32> = workspaces.iter().map(|w| w.id).collect();

                            let active_id = match Workspace::get_active() {
                                Ok(active) => active.id,
                                Err(_) => -1,
                            };

                            (workspace_ids, active_id)
                        }
                        Err(_) => (vec![], -1),
                    };

                    let _ = sender.send(result);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
        });

        // Track previous state
        let mut prev_workspaces: Vec<i32> = vec![];
        let mut prev_active_id: i32 = -1;

        // Poll for updates
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if let Ok((workspace_ids, active_id)) = receiver.try_recv() {
                // Check if workspaces changed
                if workspace_ids != prev_workspaces {
                    println!("Workspaces changed: {:?}", workspace_ids);
                    Self::rebuild_buttons(&container, &workspace_ids, prev_active_id);

                    // Schedule the class update after the next frame so buttons render first
                    let container_clone = container.clone();
                    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                        Self::update_active_class(&container_clone, active_id);
                        glib::ControlFlow::Break
                    });

                    prev_workspaces = workspace_ids;
                    prev_active_id = active_id;
                }
                // Only active workspace changed
                else if active_id != prev_active_id {
                    println!("Active workspace changed: {}", active_id);
                    Self::update_active_class(&container, active_id);
                    prev_active_id = active_id;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn rebuild_buttons(container: &gtk::Box, workspace_ids: &[i32], prev_active_id: i32) {
        // Clear existing buttons
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        println!("Rebuilding buttons for workspaces: {:?}", workspace_ids);

        // Create button for each workspace
        for &ws_id in workspace_ids {
            let button = gtk::Button::with_label(&ws_id.to_string());

            // Set CSS classes based on PREVIOUS active state
            if ws_id == prev_active_id {
                button.set_css_classes(&["workspace-button", "active"]);
            } else {
                button.set_css_classes(&["workspace-button"]);
            }

            // Handle click to switch workspace
            button.connect_clicked(move |btn| {
                println!(
                    "Button clicked! Workspace ID: {}, Label: {:?}",
                    ws_id,
                    btn.label()
                );
                Self::switch_workspace(ws_id);
            });

            container.append(&button);
        }
    }

    fn update_active_class(container: &gtk::Box, active_id: i32) {
        let mut child = container.first_child();
        let mut _index = 0;

        println!("Active workspace set to {}", active_id);

        while let Some(button) = child {
            if let Some(btn) = button.downcast_ref::<gtk::Button>() {
                let ws_id = btn
                    .label()
                    .and_then(|l| l.parse::<i32>().ok())
                    .unwrap_or(-1);

                if ws_id == active_id {
                    btn.set_css_classes(&["workspace-button", "active"]);
                } else {
                    btn.set_css_classes(&["workspace-button"]);
                }
            }
            child = button.next_sibling();
            _index += 1;
        }
    }

    fn switch_workspace(workspace_id: i32) {
        use hyprland::dispatch::*;

        println!("Switching to workspace: {}", workspace_id);

        let result = Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            workspace_id,
        )));

        if let Err(e) = result {
            println!("Failed to switch workspace: {:?}", e);
        } else {
            println!("Successfully switched to workspace {}", workspace_id);
        }
    }
}
