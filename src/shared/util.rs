// ============ shared/util.rs ============

use gtk4 as gtk;
use gtk4::prelude::*;
use once_cell::sync::Lazy;
use std::process::Stdio;

// Detect dash if installed as static variable
static SHELL_NAME: Lazy<String> = Lazy::new(|| {
    let is_dash_installed = std::path::Path::new("/bin/dash").exists();

    if is_dash_installed {
        "/bin/dash".to_string()
    } else {
        "/bin/sh".to_string()
    }
});

// Run Async Shell Commands
#[inline]
pub fn run_shell_command(command: &str) {
    if command.is_empty() {
        return;
    }

    let _ = std::process::Command::new(&*SHELL_NAME)
        .arg("-c")
        .arg(format!("`{}`", command))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut child| {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        });
}

// Cut strings to given limit
#[inline]
pub fn take_chars(s: &str, x: u64) -> &str {
    if x == 0 {
        return "";
    }

    for (count, (byte_idx, _)) in s.char_indices().enumerate() {
        if count as u64 == x {
            return &s[..byte_idx];
        }
    }

    s
}

pub struct Gestures {
    pub on_click: String,
    pub on_click_middle: Option<String>,
    pub on_click_right: Option<String>,
    pub scroll_up: Option<String>,
    pub scroll_down: Option<String>,
}

pub fn create_gesture_handler<W: IsA<gtk::Widget>>(gtk_object: &W, gestures: Gestures) {
    let widget: &gtk::Widget = gtk_object.upcast_ref();

    // Left Click Gesture
    if !gestures.on_click.is_empty() {
        let on_click = gestures.on_click.clone();
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            button.connect_clicked(move |_| {
                run_shell_command(&on_click);
            });
        } else {
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_pressed(move |_, _, _, _| {
                run_shell_command(&on_click);
            });
            widget.add_controller(gesture);
        }
    }

    // Middle and Right Click Gestures
    if gestures.on_click_middle.is_some() || gestures.on_click_right.is_some() {
        let gesture = gtk::GestureClick::new();
        gesture.set_button(0);
        let on_click_middle = gestures.on_click_middle.unwrap_or_default();
        let on_click_right = gestures.on_click_right.unwrap_or_default();
        gesture.connect_released(move |gesture, _, _, _| match gesture.current_button() {
            2 => run_shell_command(&on_click_middle),
            3 => run_shell_command(&on_click_right),
            _ => {}
        });
        widget.add_controller(gesture);
    }

    // Scroll gestures
    if gestures.scroll_up.is_some() || gestures.scroll_down.is_some() {
        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let scroll_up = gestures.scroll_up;
        let scroll_down = gestures.scroll_down;
        scroll_controller.connect_scroll(move |_, _, dy| {
            if dy < 0.0 {
                if let Some(cmd) = scroll_up.as_ref() {
                    run_shell_command(cmd);
                }
            } else if let Some(cmd) = scroll_down.as_ref() {
                run_shell_command(cmd);
            }
            gtk4::glib::Propagation::Stop
        });
        widget.add_controller(scroll_controller);
    }
}
