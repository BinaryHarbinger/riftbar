// ============ hyprland_workspaces.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use hyprland::data::*;
use hyprland::shared::{HyprData, HyprDataActive};
use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
};

#[derive(Clone)]
pub struct WorkspacesConfig {
    pub format: Option<String>,
    pub min_workspace_count: i32,
    pub workspace_formating: Option<HashMap<u32, String>>,
    // pub tooltip: bool,
    // pub tooltip_format: String,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: None,
            min_workspace_count: 4,
            workspace_formating: None,
            // tooltip: true,
            // tooltip_format: "Workspaces".to_string(),
        }
    }
}

impl WorkspacesConfig {
    pub fn from_config(config: &crate::config::WorkspacesConfig) -> Self {
        Self {
            format: config.format.clone(),
            min_workspace_count: config.min_workspace_count,
            workspace_formating: config.workspace_formating.clone(),
            // tooltip: config.tooltip,
            // tooltip_format: config.tooltip_format.clone(),
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

    fn start_updates(&self, config: Arc<WorkspacesConfig>) {
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
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok((workspace_ids, active_id)) = receiver.try_recv() {
                // Check if workspaces changed
                if workspace_ids != prev_workspaces {
                    Self::rebuild_buttons(
                        &container,
                        &workspace_ids,
                        prev_active_id,
                        config.format.as_deref().unwrap_or("{}"),
                        config.min_workspace_count,
                        &config.workspace_formating, // Pass as reference
                    );

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

    fn rebuild_buttons(
        container: &gtk::Box,
        workspace_ids: &[i32],
        prev_active_id: i32,
        format: &str,
        min_workspace_count: i32,
        workspace_formating: &Option<HashMap<u32, String>>,
    ) {
        // Clear existing buttons
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        // Build workspace array
        let mut workspace_id_array: Vec<i32> = workspace_ids.to_vec();
        for i in 1..=min_workspace_count {
            if !workspace_id_array.contains(&i) {
                workspace_id_array.push(i);
            }
        }
        workspace_id_array.sort_unstable(); // Slightly faster than sort()

        // Pre-allocate string for reuse
        let mut id_string = String::with_capacity(4);

        // Create button for each workspace
        for &ws_id in &workspace_id_array {
            // Determine workspace label
            let pre_format = match workspace_formating {
                Some(formatting) => {
                    // Only do HashMap lookup if formatting exists
                    formatting
                        .get(&(ws_id as u32))
                        .map(|s| s.as_str())
                        .unwrap_or_else(|| {
                            id_string.clear();
                            use std::fmt::Write;
                            let _ = write!(&mut id_string, "{}", ws_id);
                            &id_string
                        })
                }
                None => {
                    // No formatting - just convert ID to string
                    id_string.clear();
                    use std::fmt::Write;
                    let _ = write!(&mut id_string, "{}", ws_id);
                    &id_string
                }
            };

            let label = format.replace("{}", pre_format);

            let button = gtk::Button::with_label(&label);
            button.set_widget_name(&ws_id.to_string());
            container.append(&button);

            // Set CSS classes based on previous active state
            if ws_id == prev_active_id {
                button.set_css_classes(&["workspace-button", "active"]);
            } else {
                button.set_css_classes(&["workspace-button"]);
            }

            // Handle click to switch workspace
            button.connect_clicked(move |_| {
                Self::switch_workspace(ws_id);
            });
        }
    }
    fn update_active_class(container: &gtk::Box, active_id: i32) {
        let mut child = container.first_child();

        while let Some(button) = child {
            if let Some(btn) = button.downcast_ref::<gtk::Button>() {
                // Get workspace ID from widget name instead of label
                let ws_id = btn.widget_name().as_str().parse::<i32>().unwrap_or(-1);

                if ws_id == active_id {
                    btn.set_css_classes(&["workspace-button", "active"]);
                } else {
                    btn.set_css_classes(&["workspace-button"]);
                }
            }
            child = button.next_sibling();
        }
    }
    fn switch_workspace(workspace_id: i32) {
        use hyprland::dispatch::*;

        let result = Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            workspace_id,
        )));

        if let Err(e) = result {
            println!("Failed to switch workspace: {:?}", e);
        }
    }
}
