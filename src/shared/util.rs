#[inline]
pub fn run_command_async(action: String) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(action.clone())
                .output()
                .await;
        });
    });
}

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
