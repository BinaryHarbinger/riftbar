// ============ hyprland_workspaces.rs ============
use gtk4 as gtk;
use gtk4::prelude::*;
use hyprland::data::*;
use hyprland::shared::{HyprData, HyprDataActive};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    sync::{Arc, mpsc},
};

#[derive(Clone)]
pub struct WorkspacesConfig {
    pub format: Option<String>,
    pub icons: Option<HashMap<String, String>>,
    pub min_workspace_count: i32,
    pub workspace_formating: Option<HashMap<u32, String>>,
    pub show_special_workspaces: bool,
    pub widget_orientation: gtk::Orientation,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: None,
            icons: None,
            min_workspace_count: 4,
            workspace_formating: None,
            show_special_workspaces: false,
            widget_orientation: gtk::Orientation::Horizontal,
        }
    }
}

impl WorkspacesConfig {
    pub fn from_config(
        config: &crate::config::WorkspacesConfig,
        container_orientation: gtk::Orientation,
    ) -> Self {
        Self {
            format: config.format.clone(),
            icons: config.icons.clone(),
            min_workspace_count: config.min_workspace_count,
            workspace_formating: config.workspace_formating.clone(),
            show_special_workspaces: config.show_special_workspaces,
            widget_orientation: container_orientation,
        }
    }
}

pub struct HyprWorkspacesWidget {
    pub container: gtk::Box,
}

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

// Events we care about from socket2
#[derive(Debug)]
enum HyprEvent {
    // Active workspace switched — just update the CSS class
    ActiveWorkspace(i32),
    // Workspace list mutated — need a full rebuild
    WorkspaceListChanged,
    // Idle inhibitor toggled — update "idle" CSS class
    Idle(bool),
}

/// Connect to Hyprland's socket2 and forward relevant events down `tx`.
fn start_socket_listener(tx: mpsc::Sender<HyprEvent>) {
    std::thread::spawn(move || {
        let instance = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(v) => v,
            Err(_) => {
                eprintln!("[workspaces] HYPRLAND_INSTANCE_SIGNATURE not set");
                return;
            }
        };
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/hypr/{}/.socket2.sock", runtime_dir, instance);

        let stream = match UnixStream::connect(&socket_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[workspaces] failed to connect to socket2: {e}");
                return;
            }
        };

        let reader = BufReader::new(stream);

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[workspaces] socket2 read error: {e}");
                    break;
                }
            };

            // workspace>>ID  — active workspace switched
            if let Some(rest) = line.strip_prefix("workspace>>") {
                if let Ok(id) = rest.trim().parse::<i32>() {
                    let _ = tx.send(HyprEvent::ActiveWorkspace(id));
                }
                continue;
            }

            // focusedmon>>MONITOR,ID  — active workspace on focused monitor changed
            if let Some(rest) = line.strip_prefix("focusedmon>>") {
                if let Some(id_str) = rest.split(',').nth(1)
                    && let Ok(id) = id_str.trim().parse::<i32>()
                {
                    let _ = tx.send(HyprEvent::ActiveWorkspace(id));
                }
                continue;
            }

            // createworkspace>>NAME  — a new workspace appeared
            if line.starts_with("createworkspace>>") {
                let _ = tx.send(HyprEvent::WorkspaceListChanged);
                continue;
            }

            // destroyworkspace>>NAME  — a workspace was removed
            if line.starts_with("destroyworkspace>>") {
                let _ = tx.send(HyprEvent::WorkspaceListChanged);
                continue;
            }

            // moveworkspace>>NAME,MONITOR  — workspace moved to another monitor
            if line.starts_with("moveworkspace>>") {
                let _ = tx.send(HyprEvent::WorkspaceListChanged);
                continue;
            }

            // screencast>>STATE,OWNER  — idle inhibitor on/off (Hyprland uses
            // this together with the hypridle integration; the canonical event
            // is actually from the "idle" protocol but Hyprland ≥ 0.40 emits
            // "idleinhibitor>>ACTIVATE" / "idleinhibitor>>DEACTIVATE".
            if let Some(rest) = line.strip_prefix("idleinhibitor>>") {
                let active = rest.trim().eq_ignore_ascii_case("activate");
                let _ = tx.send(HyprEvent::Idle(active));
                continue;
            }
        }
    });
}

/// One-shot: ask Hyprland for the current workspace list + active id.
fn fetch_workspaces(show_special: bool) -> (Vec<WorkspaceObject>, i32) {
    let workspaces = match Workspaces::get() {
        Ok(ws) => {
            let mut list: Vec<_> = ws.into_iter().collect();
            list.sort_by_key(|w| w.id);
            list.into_iter()
                .filter(|w| show_special || w.id >= 0)
                .map(|w| WorkspaceObject {
                    id: w.id,
                    name: w.name.clone(),
                })
                .collect()
        }
        Err(e) => {
            eprintln!("[workspaces] failed to fetch workspace list: {e}");
            vec![]
        }
    };

    let active_id = Workspace::get_active().map(|w| w.id).unwrap_or(-1);

    (workspaces, active_id)
}

impl HyprWorkspacesWidget {
    pub fn new(config: Arc<WorkspacesConfig>) -> Self {
        let container = gtk::Box::new(config.widget_orientation, 5);
        container.set_css_classes(&["workspaces"]);

        let widget = Self { container };
        widget.start_updates(config);
        widget
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    fn start_updates(&self, config: Arc<WorkspacesConfig>) {
        let container = self.container.clone();
        let (tx, rx) = mpsc::channel::<HyprEvent>();

        // Kick off the socket2 listener thread
        start_socket_listener(tx);

        // Initial full build — do it once before the event loop starts
        {
            let (workspaces, active_id) = fetch_workspaces(config.show_special_workspaces);
            Self::rebuild_buttons(
                &container,
                &workspaces,
                active_id,
                config.format.as_deref().unwrap_or("{id}"),
                config.icons.clone(),
                config.min_workspace_count,
                &config.workspace_formating,
            );
            // Defer the active class update by one frame so GTK has a chance
            // to realise the new buttons before we try to style them.
            let container_init = container.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(16), move || {
                Self::update_active_class(&container_init, active_id);
            });
        }

        // State we track across idle_add callbacks
        let mut current_active_id: i32 = Workspace::get_active().map(|w| w.id).unwrap_or(-1);
        let mut is_idle = false;

        // Use idle_add so we only wake when there is actually something in the
        // channel, rather than hammering every 100 ms.
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Drain all pending events in one go
            let mut needs_rebuild = false;
            let mut new_active: Option<i32> = None;
            let mut new_idle: Option<bool> = None;

            while let Ok(event) = rx.try_recv() {
                match event {
                    HyprEvent::ActiveWorkspace(id) => {
                        new_active = Some(id);
                    }
                    HyprEvent::WorkspaceListChanged => {
                        needs_rebuild = true;
                        // Also re-fetch the active id so the rebuild is correct
                        new_active = Some(
                            Workspace::get_active()
                                .map(|w| w.id)
                                .unwrap_or(current_active_id),
                        );
                    }
                    HyprEvent::Idle(active) => {
                        new_idle = Some(active);
                    }
                }
            }

            // Apply idle CSS change
            if let Some(idle) = new_idle
                && idle != is_idle
            {
                is_idle = idle;
                if idle {
                    container.add_css_class("idle");
                } else {
                    container.remove_css_class("idle");
                }
            }

            // Full rebuild only when workspace list changed
            if needs_rebuild {
                let (workspaces, fetched_active) = fetch_workspaces(config.show_special_workspaces);

                // The new active id (from the socket event or a fresh fetch).
                // We deliberately do NOT pass this into rebuild_buttons — the
                // buttons are built without any active class so that GTK gets
                // one clean frame to render them before the active style lands.
                let next_active_id = if fetched_active != -1 {
                    fetched_active
                } else {
                    new_active.unwrap_or(current_active_id)
                };

                // Rebuild with the *old* active id so no button starts life
                // already marked active (avoids the instant-highlight glitch).
                Self::rebuild_buttons(
                    &container,
                    &workspaces,
                    current_active_id,
                    config.format.as_deref().unwrap_or("{id}"),
                    config.icons.clone(),
                    config.min_workspace_count,
                    &config.workspace_formating,
                );

                // One frame later: apply the active class to the correct button.
                let container_rebuild = container.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(16), move || {
                    Self::update_active_class(&container_rebuild, next_active_id);
                });

                current_active_id = next_active_id;
            }
            // Cheap active-class-only update — no rebuild needed
            else if let Some(id) = new_active
                && id != current_active_id
            {
                current_active_id = id;
                Self::update_active_class(&container, id);
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

        // Build workspace array, padding up to min_workspace_count
        let mut workspace_id_array: Vec<WorkspaceObject> = workspace_ids.to_vec();
        for i in 1..=min_workspace_count {
            let ws = WorkspaceObject {
                id: i,
                ..Default::default()
            };
            if !workspace_id_array.contains(&ws) {
                workspace_id_array.push(ws);
            }
        }
        workspace_id_array.sort_unstable();

        // Compute labels (can be done in parallel with scoped threads)
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

                        let pre_format = match workspace_formating {
                            Some(fmt) => fmt.get(&(ws_id as u32)).cloned().unwrap_or_else(|| {
                                if !name.is_empty() && workspace.is_special_workspace() {
                                    name.clone()
                                } else {
                                    ws_id.to_string()
                                }
                            }),
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
                .map(|h| h.join().unwrap())
                .collect()
        });

        // Create GTK widgets on the main thread
        for (workspace, label_text) in workspace_id_array.iter().zip(label_texts.iter()) {
            let ws_id = workspace.id;

            let gtk_label = gtk::Label::new(Some(label_text));
            let button = gtk::Button::new();
            button.set_child(Some(&gtk_label));
            button.set_widget_name(&ws_id.to_string());

            if ws_id == prev_active_id {
                button.set_css_classes(&["workspace-button", "active"]);
            } else {
                button.set_css_classes(&["workspace-button"]);
            }

            button.connect_clicked(move |_| {
                Self::switch_workspace(ws_id);
            });

            container.append(&button);
        }
    }

    fn update_active_class(container: &gtk::Box, active_id: i32) {
        let mut child = container.first_child();
        while let Some(widget) = child {
            if let Some(btn) = widget.downcast_ref::<gtk::Button>() {
                let ws_id = btn
                    .widget_name()
                    .as_str()
                    .parse::<i32>()
                    .unwrap_or(i32::MIN);
                if ws_id == active_id {
                    btn.set_css_classes(&["workspace-button", "active"]);
                } else {
                    btn.set_css_classes(&["workspace-button"]);
                }
            }
            child = widget.next_sibling();
        }
    }

    fn switch_workspace(workspace_id: i32) {
        use hyprland::dispatch::*;
        if let Err(e) = Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            workspace_id,
        ))) {
            eprintln!("[workspaces] failed to switch to workspace {workspace_id}: {e:?}");
        }
    }
}
