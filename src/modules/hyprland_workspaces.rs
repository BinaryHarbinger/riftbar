use gtk4::subclass::widget;
// ============ hyprlandworkspaces.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use hyprland::data::*;
use hyprland::shared::{HyprData, HyprDataActive};
use std::sync::{Arc, mpsc};

#[derive(Clone)]
pub struct WorkspacesConfig {
    pub min_workspace_count: i32,
    pub tooltip: bool,
    pub tooltip_format: String,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            min_workspace_count: 4,
            tooltip: true,
            tooltip_format: "Workspaces".to_string(),
        }
    }
}

impl WorkspacesConfig {
    pub fn from_config(config: &crate::config::WorkspacesConfig) -> Self {
        Self {
            min_workspace_count: config.min_workspace_count.clone(),
            tooltip: config.tooltip,
            tooltip_format: config.tooltip_format.clone(),
        }
    }
}

pub struct HyprWorkspacesWidget {
    pub container: gtk::Box,
}

impl HyprWorkspacesWidget {
    pub fn new(config: Arc<WorkspacesConfig>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        container.set_css_classes(&["workspaces"]);
        
        let widget = Self { container };

        // Start the update loop
        widget.start_updates(config);

        widget
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn start_updates(&self, config: Arc<WorkspacesConfig> ) {
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
                    Self::rebuild_buttons(&container, &workspace_ids, prev_active_id, config.min_workspace_count);

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
                    Self::update_active_class(&container, active_id);
                    prev_active_id = active_id;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn rebuild_buttons(container: &gtk::Box, workspace_ids: &[i32], prev_active_id: i32, min_workspace_count: i32) {
        // Clear existing buttons
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        // Add up to minimum workspace count in config.toml
        let mut workspace_id_array: Vec<i32>  = workspace_ids.to_vec();

        for i in 1..=min_workspace_count {
            if !workspace_id_array.contains(&i) {
                workspace_id_array.push(i);
            }
        }

        workspace_id_array.sort();

        // Create button for each workspace
        for &ws_id in &workspace_id_array {
            let button = gtk::Button::with_label(&ws_id.to_string());

            // Set CSS classes based on PREVIOUS active state
            if ws_id == prev_active_id {
                button.set_css_classes(&["workspace-button", "active"]);
            } else {
                button.set_css_classes(&["workspace-button"]);
            }

            // Handle click to switch workspace
            button.connect_clicked(move |_| {
                Self::switch_workspace(ws_id);
            });

            container.append(&button);
        }
    }

    fn update_active_class(container: &gtk::Box, active_id: i32) {
        let mut child = container.first_child();
        let mut _index = 0;

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
