// ============ shared/util.rs ============

use once_cell::sync::Lazy;

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
pub fn run_command_async(action: String) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = tokio::process::Command::new(&*SHELL_NAME)
                .arg("-c")
                .arg(action.clone())
                .output()
                .await;
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
