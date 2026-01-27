// ============ shared/util.rs ============

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
pub fn run_shell_command(command: String) {
    if command.is_empty() {
        return;
    }
    let _ = std::process::Command::new(&*SHELL_NAME)
        .arg("-c")
        .arg(format!("`{}`", command))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
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
