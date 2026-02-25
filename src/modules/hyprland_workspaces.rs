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
    pub icons: Option<HashMap<String, String>>,
    pub min_workspace_count: i32,
    pub workspace_formating: Option<HashMap<u32, String>>,
    pub show_special_workspaces: bool,
    pub orientation: bool,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: None,
            icons: None,
            min_workspace_count: 4,
            workspace_formating: None,
            show_special_workspaces: false,
            orientation: true,
        }
    }
}

impl WorkspacesConfig {
    pub fn from_config(config: &crate::config::WorkspacesConfig, orientation_bool: bool) -> Self {
        Self {
            format: config.format.clone(),
            icons: config.icons.clone(),
            min_workspace_count: config.min_workspace_count,
            workspace_formating: config.workspace_formating.clone(),
            show_special_workspaces: config.show_special_workspaces,
            orientation: orientation_bool,
        }
    }
}

pub struct HyprWorkspacesWidget {
    pub container: gtk::Box,
}

// TODO: I dont know what to call this struct
#[derive(Clone, Debug, Default)]
struct WorkspaceObject {
    id: i32,
    name: String,
}

impl WorkspaceObject {
    pub fn is_special_workspace(&self) -> bool {
        self.id < 0
    }
}

impl PartialEq for WorkspaceObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for WorkspaceObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorkspaceObject {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Eq for WorkspaceObject {}

impl HyprWorkspacesWidget {
    pub fn new(config: Arc<WorkspacesConfig>) -> Self {
        let widget_orientation = if config.orientation {
            gtk::Orientation::Horizontal
        } else {
            gtk::Orientation::Vertical
        };
        let container = gtk::Box::new(widget_orientation, 5);
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
        let (sender, receiver) = mpsc::channel::<(Vec<WorkspaceObject>, i32)>();
        let show_special_workspaces = config.show_special_workspaces;

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

                            let workspace_ids = workspaces.iter().map(|w| WorkspaceObject {
                                id: w.id,
                                name: w.name.clone(),
                            });
                            let workspace_ids: Vec<WorkspaceObject> = if show_special_workspaces {
                                workspace_ids.collect()
                            } else {
                                workspace_ids.filter(|w| w.id >= 0).collect()
                            };

                            let active_id = match Workspace::get_active() {
                                Ok(active) => active.id,
                                Err(_) => -1,
                            };

                            (workspace_ids, active_id)
                        }
                        Err(_) => ((vec![]), -1),
                    };

                    let _ = sender.send(result);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
        });

        // Track previous state
        let mut prev_workspaces: Vec<WorkspaceObject> = vec![];
        // FIXME: -1 can't be a sentinel value for prev_active_id as in theory this could be a
        // valid special_workspace_id
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
                        config.format.as_deref().unwrap_or("{id}"),
                        config.icons.clone(),
                        config.min_workspace_count,
                        &config.workspace_formating, // Pass as reference
                    );

                    // Schedule the class update after the next frame so buttons render first
                    let container_clone = container.clone();
                    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                        Self::update_active_class(&container_clone, active_id);
                        glib::ControlFlow::Break
                    });

                    // NOTE: Workspace id does not change for special workspaces, ie, special
                    // workspace does not get the active style
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
        workspace_ids: &[WorkspaceObject],
        prev_active_id: i32,
        format: &str,
        icons: Option<HashMap<String, String>>,
        min_workspace_count: i32,
        workspace_formating: &Option<HashMap<u32, String>>,
    ) {
        // Clear existing buttons
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        // Build workspace array
        let mut workspace_id_array: Vec<WorkspaceObject> = workspace_ids.to_vec();
        for i in 1..=min_workspace_count {
            let workspace = WorkspaceObject {
                id: i,
                ..Default::default()
            };
            if !workspace_id_array.contains(&workspace) {
                workspace_id_array.push(workspace);
            }
        }
        workspace_id_array.sort_unstable();

        // Compute all labels in parallel using scoped threads
        let label_texts: Vec<String> = std::thread::scope(|s| {
            workspace_id_array
                .iter()
                .map(|workspace| {
                    s.spawn(|| {
                        let ws_id = workspace.id;
                        let name = if workspace.name.starts_with("special") {
                            workspace
                                .name
                                .split(':')
                                .next_back()
                                .unwrap_or("magic")
                                .to_string()
                        } else {
                            String::new()
                        };

                        // Determine workspace label in parallel
                        let pre_format = match workspace_formating {
                            Some(formatting) => {
                                formatting.get(&(ws_id as u32)).cloned().unwrap_or_else(|| {
                                    if !name.is_empty() && workspace.is_special_workspace() {
                                        name.clone()
                                    } else {
                                        ws_id.to_string()
                                    }
                                })
                            }
                            None => {
                                if !name.is_empty() && workspace.is_special_workspace() {
                                    name.clone()
                                } else {
                                    ws_id.to_string()
                                }
                            }
                        };

                        let mut label = format.replace("{}", "{id}").replace("{id}", &pre_format);

                        if let Some(ref icon_map) = icons {
                            let key = if ws_id == prev_active_id {
                                "active"
                            } else {
                                "normal"
                            };
                            let icon = icon_map.get(key).map(|s| s.as_str()).unwrap_or("");
                            label = label.replace("{icon}", icon);
                        }

                        label
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect()
        });

        // Create GTK widgets in main thread with pre-computed labels
        for (workspace, label_text) in workspace_id_array.iter().zip(label_texts.iter()) {
            let ws_id = workspace.id;

            // Create GTK widgets (must be done in main thread)
            let gtk_label = gtk::Label::new(Some(label_text));
            let button = gtk::Button::new();
            button.set_child(Some(&gtk_label));
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
