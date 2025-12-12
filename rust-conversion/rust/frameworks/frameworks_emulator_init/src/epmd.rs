//! epmd Daemon Management Module
//!
//! Provides epmd daemon management functionality to replace erlexec epmd startup.
//! Handles starting epmd daemon before emulator initialization.

use std::process::Command;
use std::path::PathBuf;

/// Start epmd daemon if needed
pub fn start_epmd_daemon(bindir: &str, epmd_path: Option<&str>) -> Result<(), String> {
    // Determine epmd program path
    let epmd_program = epmd_path
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(bindir).join("epmd")
        });

    // Check if epmd exists
    if !epmd_program.exists() {
        return Err(format!("epmd program not found at: {}", epmd_program.display()));
    }

    // Spawn epmd daemon (replaces C system() call)
    let child = Command::new(&epmd_program)
        .arg("-daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn epmd: {}", e))?;

    // Don't wait for epmd - it's a daemon
    // epmd may already be running, which is fine
    // We detach the child process
    drop(child);

    Ok(())
}

/// Check if epmd is already running
pub fn is_epmd_running() -> bool {
    // Try to connect to epmd port (4369) to check if it's running
    // This is a simple check - in production, you might want more robust detection
    use std::net::TcpStream;
    TcpStream::connect("127.0.0.1:4369").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_epmd_running() {
        // This test may fail if epmd is not running
        // It's mainly to ensure the function doesn't panic
        let _result = is_epmd_running();
    }
}

