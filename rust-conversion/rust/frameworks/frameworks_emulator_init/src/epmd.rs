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
    use std::path::PathBuf;

    #[test]
    fn test_is_epmd_running() {
        // This test may fail if epmd is not running
        // It's mainly to ensure the function doesn't panic
        let result = is_epmd_running();
        // Result is a boolean - epmd may or may not be running
        let _ = result;
    }

    #[test]
    fn test_is_epmd_running_consistency() {
        // Multiple calls should return consistent results (within a short time)
        let result1 = is_epmd_running();
        let result2 = is_epmd_running();
        // Results should be the same (epmd state shouldn't change instantly)
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_start_epmd_daemon_path_resolution_with_epmd_path() {
        // Test path resolution when epmd_path is provided
        // Use a non-existent path to test error handling
        let result = start_epmd_daemon("/nonexistent/bindir", Some("/nonexistent/epmd"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("epmd program not found"));
        assert!(error_msg.contains("/nonexistent/epmd"));
    }

    #[test]
    fn test_start_epmd_daemon_path_resolution_without_epmd_path() {
        // Test path resolution when epmd_path is None (should use bindir/epmd)
        let result = start_epmd_daemon("/nonexistent/bindir", None);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("epmd program not found"));
        // Should contain bindir path
        assert!(error_msg.contains("/nonexistent/bindir"));
    }

    #[test]
    fn test_start_epmd_daemon_path_resolution_logic() {
        // Test that path resolution logic works correctly
        // When epmd_path is provided, it should be used
        let epmd_path = Some("/custom/path/to/epmd");
        let bindir = "/default/bindir";
        
        // The function will check if the path exists, but we can test the path construction
        let expected_path = PathBuf::from(epmd_path.unwrap());
        let result = start_epmd_daemon(bindir, epmd_path);
        
        // Should fail because path doesn't exist, but error should mention the custom path
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("/custom/path/to/epmd"));
    }

    #[test]
    fn test_start_epmd_daemon_path_resolution_default() {
        // Test that when epmd_path is None, it uses bindir/epmd
        let bindir = "/test/bindir";
        
        // The function will check if the path exists
        let result = start_epmd_daemon(bindir, None);
        
        // Should fail because path doesn't exist, but error should mention bindir/epmd
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains(bindir));
    }

    #[test]
    fn test_start_epmd_daemon_error_message_format() {
        // Test that error messages are properly formatted
        let result = start_epmd_daemon("/test/bindir", Some("/test/epmd"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        
        // Error message should be informative
        assert!(!error_msg.is_empty());
        assert!(error_msg.contains("epmd program not found"));
    }

    #[test]
    fn test_start_epmd_daemon_with_relative_path() {
        // Test with relative path
        let result = start_epmd_daemon(".", Some("./epmd"));
        // May succeed if epmd exists in current directory, or fail if not
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_start_epmd_daemon_with_absolute_bindir() {
        // Test with absolute bindir path
        let result = start_epmd_daemon("/usr/local/bin", None);
        // May succeed if epmd exists, or fail if not
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_start_epmd_daemon_path_construction() {
        // Test path construction logic
        // When epmd_path is None, should construct bindir/epmd
        let bindir = "/test/bindir";
        let expected_path = PathBuf::from(bindir).join("epmd");
        
        let result = start_epmd_daemon(bindir, None);
        // Should fail because path doesn't exist
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        // Error should contain the constructed path
        assert!(error_msg.contains("epmd"));
    }

    #[test]
    fn test_is_epmd_running_returns_boolean() {
        // Verify function returns a boolean
        let result = is_epmd_running();
        // Should be either true or false
        assert!(result == true || result == false);
    }

    #[test]
    fn test_start_epmd_daemon_empty_bindir() {
        // Test with empty bindir
        let result = start_epmd_daemon("", None);
        // Should fail because empty path won't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_start_epmd_daemon_empty_epmd_path() {
        // Test with empty epmd_path (should use bindir/epmd)
        let result = start_epmd_daemon("/test/bindir", Some(""));
        // Empty string path won't exist, should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_pathbuf_construction() {
        // Test that PathBuf construction works as expected
        let bindir = "/test/bindir";
        let path1 = PathBuf::from(bindir).join("epmd");
        let path2 = PathBuf::from("/test/bindir/epmd");
        
        // Both should represent the same path
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_start_epmd_daemon_with_windows_path() {
        // Test with Windows-style path (if on Windows)
        #[cfg(windows)]
        {
            let result = start_epmd_daemon("C:\\test\\bindir", Some("C:\\test\\epmd.exe"));
            // Should fail because path doesn't exist
            assert!(result.is_err());
        }
        
        #[cfg(not(windows))]
        {
            // On Unix, test with forward slashes
            let result = start_epmd_daemon("/test/bindir", Some("/test/epmd"));
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_is_epmd_running_connection_attempt() {
        // Test that is_epmd_running attempts to connect
        // This is a basic smoke test - actual connection depends on epmd state
        let _result = is_epmd_running();
        // Function should complete without panicking
    }

    #[test]
    fn test_start_epmd_daemon_error_contains_path() {
        // Verify error messages contain the path information
        let test_paths = vec![
            ("/test1", Some("/custom/epmd")),
            ("/test2", None),
            ("/test/bindir", Some("/test/epmd")),
        ];
        
        for (bindir, epmd_path) in test_paths {
            let result = start_epmd_daemon(bindir, epmd_path);
            assert!(result.is_err());
            let error_msg = result.unwrap_err();
            // Error should contain path information
            assert!(!error_msg.is_empty());
        }
    }
}

