// ============ modules/action.rs ============
use std::process::Command;

/// Execute a shell command asynchronously in a separate thread
pub fn run_command_async(command: &str) {
    if command.is_empty() {
        return;
    }

    let command = command.to_string();
    std::thread::spawn(move || {
        match Command::new("sh").arg("-c").arg(&command).output() {
            Ok(output) => {
                if output.status.success() {
                    println!("Executed command: {}", command);
                } else {
                    eprintln!(
                        "Command failed: {}\nError: {}",
                        command,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command '{}': {}", command, e);
            }
        }
    });
}

/// Execute a shell command synchronously and return the output
pub fn run_command_sync(command: &str) -> Option<String> {
    if command.is_empty() {
        return None;
    }

    match Command::new("sh").arg("-c").arg(command).output() {
        Ok(output) => {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                eprintln!(
                    "Command failed: {}\nError: {}",
                    command,
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command '{}': {}", command, e);
            None
        }
    }
}
