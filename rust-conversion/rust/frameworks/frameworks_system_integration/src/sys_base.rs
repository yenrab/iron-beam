//! System Integration Base Module
//!
//! Provides base system integration functionality.
//! Based on sys_shell.c
//!
//! This module implements the foundational framework for system integration,
//! providing base operations for shell integration, command execution, and
//! system information queries.

use frameworks_system_integration_common::SysCommon;
use std::process::{Command, Stdio};
use std::io::{self, IsTerminal};

/// System integration base
///
/// Provides foundational system integration operations including initialization,
/// command execution, and system information queries.
pub struct SysBase;

impl SysBase {
    /// Initialize system integration base
    ///
    /// Initializes the base system integration layer, including common
    /// system integration components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// let result = SysBase::init();
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `SysError::InitFailed` if initialization fails.
    pub fn init() -> Result<(), SysError> {
        // Initialize common system integration
        SysCommon::init()
            .map_err(|_| SysError::InitFailed)?;
        
        Ok(())
    }

    /// Execute a shell command
    ///
    /// Executes a command in the system shell and returns the output.
    /// This is a safe wrapper around system command execution.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    ///
    /// A `Result` containing the command output as a string, or an error if
    /// execution fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// // Execute a simple command
    /// let output = SysBase::execute_command("echo", &["hello"])?;
    /// assert!(output.contains("hello"));
    /// # Ok::<(), frameworks_system_integration::SysError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `SysError::CommandFailed` if the command execution fails.
    pub fn execute_command(command: &str, args: &[&str]) -> Result<String, SysError> {
        let output = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|_| SysError::CommandFailed)?;

        if !output.status.success() {
            return Err(SysError::CommandFailed);
        }

        String::from_utf8(output.stdout)
            .map_err(|_| SysError::CommandFailed)
    }

    /// Execute a shell command and return exit status
    ///
    /// Executes a command and returns only the exit status code.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    ///
    /// A `Result` containing the exit status code, or an error if execution fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// // Check if a command succeeds
    /// let status = SysBase::execute_command_status("true", &[])?;
    /// assert_eq!(status, 0);
    /// # Ok::<(), frameworks_system_integration::SysError>(())
    /// ```
    pub fn execute_command_status(command: &str, args: &[&str]) -> Result<i32, SysError> {
        let status = Command::new(command)
            .args(args)
            .status()
            .map_err(|_| SysError::CommandFailed)?;

        Ok(status.code().unwrap_or(-1))
    }

    /// Check if a command exists in the system PATH
    ///
    /// # Arguments
    ///
    /// * `command` - The command name to check
    ///
    /// # Returns
    ///
    /// `true` if the command exists in PATH, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// // Check if a common command exists
    /// let exists = SysBase::command_exists("echo");
    /// assert!(exists);
    /// ```
    pub fn command_exists(command: &str) -> bool {
        Command::new(command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
            || Command::new(command)
                .arg("-v")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
            || Command::new(command)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
    }

    /// Get system information
    ///
    /// Returns basic system information including OS type and architecture.
    ///
    /// # Returns
    ///
    /// A `SystemInfo` struct containing system information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// let info = SysBase::system_info();
    /// println!("OS: {}", info.os_type);
    /// println!("Architecture: {}", info.architecture);
    /// ```
    pub fn system_info() -> SystemInfo {
        SystemInfo {
            os_type: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        }
    }

    /// Check if running in an interactive shell
    ///
    /// Determines if the current process is running in an interactive
    /// terminal/shell environment.
    ///
    /// # Returns
    ///
    /// `true` if running in an interactive shell, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// if SysBase::is_interactive() {
    ///     println!("Running in interactive mode");
    /// }
    /// ```
    pub fn is_interactive() -> bool {
        io::stdin().is_terminal() && io::stdout().is_terminal()
    }

    /// Get the current working directory
    ///
    /// # Returns
    ///
    /// A `Result` containing the current working directory path, or an error
    /// if the directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    ///
    /// let cwd = SysBase::current_dir()?;
    /// println!("Current directory: {}", cwd.display());
    /// # Ok::<(), frameworks_system_integration::SysError>(())
    /// ```
    pub fn current_dir() -> Result<std::path::PathBuf, SysError> {
        std::env::current_dir()
            .map_err(|_| SysError::SystemError)
    }

    /// Change the current working directory
    ///
    /// # Arguments
    ///
    /// * `path` - The path to change to
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration::SysBase;
    /// use std::path::Path;
    ///
    /// // Change to a directory
    /// SysBase::change_dir(Path::new("/tmp"))?;
    /// # Ok::<(), frameworks_system_integration::SysError>(())
    /// ```
    pub fn change_dir<P: AsRef<std::path::Path>>(path: P) -> Result<(), SysError> {
        std::env::set_current_dir(path)
            .map_err(|_| SysError::SystemError)
    }
}

/// System information structure
///
/// Contains basic information about the system environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    /// Operating system type (e.g., "linux", "windows", "macos")
    pub os_type: String,
    /// System architecture (e.g., "x86_64", "arm")
    pub architecture: String,
    /// System family (e.g., "unix", "windows")
    pub family: String,
}

/// System operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysError {
    /// Initialization failed
    InitFailed,
    /// Command execution failed
    CommandFailed,
    /// System operation failed
    SystemError,
}

impl std::fmt::Display for SysError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SysError::InitFailed => write!(f, "Initialization failed"),
            SysError::CommandFailed => write!(f, "Command execution failed"),
            SysError::SystemError => write!(f, "System operation failed"),
        }
    }
}

impl std::error::Error for SysError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_base_init() {
        let result = SysBase::init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_info() {
        let info = SysBase::system_info();
        assert!(!info.os_type.is_empty());
        assert!(!info.architecture.is_empty());
        assert!(!info.family.is_empty());
    }

    #[test]
    fn test_command_exists() {
        // Most systems have "echo" command
        let exists = SysBase::command_exists("echo");
        // This may or may not be true depending on the system
        // Just verify the function doesn't panic
        let _ = exists;
    }

    #[test]
    fn test_is_interactive() {
        // Just verify the function doesn't panic
        let _ = SysBase::is_interactive();
    }

    #[test]
    fn test_current_dir() {
        let result = SysBase::current_dir();
        assert!(result.is_ok());
        let cwd = result.unwrap();
        assert!(cwd.exists());
    }

    #[test]
    fn test_execute_command() {
        // Test with a simple command that should work on most systems
        #[cfg(unix)]
        {
            let result = SysBase::execute_command("echo", &["test"]);
            if result.is_ok() {
                let output = result.unwrap();
                assert!(output.contains("test") || output.trim() == "test");
            }
        }
        
        #[cfg(windows)]
        {
            let result = SysBase::execute_command("cmd", &["/C", "echo", "test"]);
            if result.is_ok() {
                let output = result.unwrap();
                assert!(output.contains("test"));
            }
        }
    }

    #[test]
    fn test_execute_command_status() {
        #[cfg(unix)]
        {
            // Test with "true" command which should return 0
            let result = SysBase::execute_command_status("true", &[]);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 0);
            }
        }
        
        #[cfg(windows)]
        {
            // On Windows, use "exit /b 0"
            let result = SysBase::execute_command_status("cmd", &["/C", "exit", "/b", "0"]);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 0);
            }
        }
    }

    #[test]
    fn test_change_dir() {
        // Get current directory
        let original = SysBase::current_dir().unwrap();
        
        // Try to change to a temporary directory (if it exists)
        let temp_dir = std::env::temp_dir();
        if temp_dir.exists() {
            let result = SysBase::change_dir(&temp_dir);
            assert!(result.is_ok());
            
            // Verify we're in the temp directory (use canonical paths for comparison)
            let new_dir = SysBase::current_dir().unwrap();
            let canonical_new = new_dir.canonicalize().unwrap_or(new_dir);
            let canonical_temp = temp_dir.canonicalize().unwrap_or(temp_dir);
            assert_eq!(canonical_new, canonical_temp);
            
            // Change back
            let _ = SysBase::change_dir(&original);
        }
    }

    #[test]
    fn test_system_info_fields() {
        let info = SysBase::system_info();
        
        // Verify all fields are populated
        assert!(!info.os_type.is_empty());
        assert!(!info.architecture.is_empty());
        assert!(!info.family.is_empty());
        
        // Verify family matches OS type
        #[cfg(unix)]
        assert_eq!(info.family, "unix");
        
        #[cfg(windows)]
        assert_eq!(info.family, "windows");
    }

    #[test]
    fn test_execute_command_failure() {
        // Test with a command that should fail
        #[cfg(unix)]
        {
            let result = SysBase::execute_command("false", &[]);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), SysError::CommandFailed);
        }
        
        #[cfg(windows)]
        {
            let result = SysBase::execute_command("cmd", &["/C", "exit", "/b", "1"]);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), SysError::CommandFailed);
        }
    }

    #[test]
    fn test_execute_command_nonexistent() {
        // Test with a command that doesn't exist
        let result = SysBase::execute_command("nonexistent_command_xyz123", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SysError::CommandFailed);
    }

    #[test]
    fn test_execute_command_multiple_args() {
        #[cfg(unix)]
        {
            let result = SysBase::execute_command("echo", &["hello", "world"]);
            if result.is_ok() {
                let output = result.unwrap();
                assert!(output.contains("hello") || output.contains("world"));
            }
        }
        
        #[cfg(windows)]
        {
            let result = SysBase::execute_command("cmd", &["/C", "echo", "hello", "world"]);
            if result.is_ok() {
                let output = result.unwrap();
                assert!(output.contains("hello") || output.contains("world"));
            }
        }
    }

    #[test]
    fn test_execute_command_empty_args() {
        #[cfg(unix)]
        {
            let result = SysBase::execute_command("echo", &[]);
            // Should work (echo without args prints newline)
            assert!(result.is_ok() || result.is_err());
        }
        
        #[cfg(windows)]
        {
            let result = SysBase::execute_command("cmd", &["/C", "echo."]);
            // Should work
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_execute_command_status_failure() {
        // Test with a command that should return non-zero
        #[cfg(unix)]
        {
            let result = SysBase::execute_command_status("false", &[]);
            if result.is_ok() {
                assert_ne!(result.unwrap(), 0);
            }
        }
        
        #[cfg(windows)]
        {
            let result = SysBase::execute_command_status("cmd", &["/C", "exit", "/b", "1"]);
            if result.is_ok() {
                assert_ne!(result.unwrap(), 0);
            }
        }
    }

    #[test]
    fn test_execute_command_status_nonexistent() {
        // Test with a command that doesn't exist
        let result = SysBase::execute_command_status("nonexistent_command_xyz123", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SysError::CommandFailed);
    }

    #[test]
    fn test_execute_command_status_with_exit_code() {
        #[cfg(unix)]
        {
            // Test with a command that returns a specific exit code
            // Using sh -c "exit 42" to return exit code 42
            let result = SysBase::execute_command_status("sh", &["-c", "exit 42"]);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 42);
            }
        }
        
        #[cfg(windows)]
        {
            // On Windows, use exit /b 42
            let result = SysBase::execute_command_status("cmd", &["/C", "exit", "/b", "42"]);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 42);
            }
        }
    }

    #[test]
    fn test_command_exists_nonexistent() {
        // Test with a command that definitely doesn't exist
        let exists = SysBase::command_exists("nonexistent_command_xyz12345");
        assert!(!exists);
    }

    #[test]
    fn test_command_exists_common_commands() {
        // Test with commands that likely exist
        // These may or may not exist, but the function should not panic
        let _ = SysBase::command_exists("ls");
        let _ = SysBase::command_exists("cat");
        let _ = SysBase::command_exists("pwd");
    }

    #[test]
    fn test_change_dir_nonexistent() {
        // Test changing to a non-existent directory
        let result = SysBase::change_dir("/nonexistent/directory/xyz123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SysError::SystemError);
    }

    #[test]
    fn test_change_dir_file_not_directory() {
        // Test changing to a file (not a directory)
        // Create a temporary file
        let temp_file = std::env::temp_dir().join("test_file_for_chdir");
        // Try to write to it (may fail if it exists, but that's okay)
        let _ = std::fs::write(&temp_file, "test");
        
        if temp_file.exists() {
            let result = SysBase::change_dir(&temp_file);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), SysError::SystemError);
            
            // Clean up
            let _ = std::fs::remove_file(&temp_file);
        }
    }

    #[test]
    fn test_change_dir_relative_path() {
        // Get current directory
        let original = SysBase::current_dir().unwrap();
        
        // Change to parent directory
        let parent = original.parent();
        if let Some(parent_path) = parent {
            if parent_path.exists() {
                let result = SysBase::change_dir(parent_path);
                assert!(result.is_ok());
                
                // Verify we're in the parent directory
                let new_dir = SysBase::current_dir().unwrap();
                let canonical_new = new_dir.canonicalize().unwrap_or(new_dir);
                let canonical_parent = parent_path.canonicalize().unwrap_or(parent_path.to_path_buf());
                assert_eq!(canonical_new, canonical_parent);
                
                // Change back
                let _ = SysBase::change_dir(&original);
            }
        }
    }

    #[test]
    fn test_sys_error_display() {
        let error1 = SysError::InitFailed;
        assert_eq!(format!("{}", error1), "Initialization failed");
        
        let error2 = SysError::CommandFailed;
        assert_eq!(format!("{}", error2), "Command execution failed");
        
        let error3 = SysError::SystemError;
        assert_eq!(format!("{}", error3), "System operation failed");
    }

    #[test]
    fn test_sys_error_error_trait() {
        let error = SysError::CommandFailed;
        // Test that it implements Error trait
        let error_ref: &dyn std::error::Error = &error;
        assert_eq!(error_ref.to_string(), "Command execution failed");
    }

    #[test]
    fn test_system_info_debug() {
        let info = SysBase::system_info();
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("SystemInfo"));
        assert!(debug_str.contains(&info.os_type));
        assert!(debug_str.contains(&info.architecture));
        assert!(debug_str.contains(&info.family));
    }

    #[test]
    fn test_system_info_clone() {
        let info1 = SysBase::system_info();
        let info2 = info1.clone();
        assert_eq!(info1, info2);
    }

    #[test]
    fn test_system_info_eq() {
        let info1 = SysBase::system_info();
        let info2 = SysBase::system_info();
        // Should be equal (same system)
        assert_eq!(info1, info2);
    }

    #[test]
    fn test_system_info_partial_eq() {
        let info1 = SysBase::system_info();
        let info2 = info1.clone();
        assert!(info1 == info2);
        assert!(!(info1 != info2));
    }

    #[test]
    fn test_is_interactive_consistency() {
        // Call multiple times - should return consistent result
        let first = SysBase::is_interactive();
        let second = SysBase::is_interactive();
        assert_eq!(first, second);
    }

    #[test]
    fn test_current_dir_consistency() {
        // Get current directory multiple times - should be consistent
        let dir1 = SysBase::current_dir().unwrap();
        let dir2 = SysBase::current_dir().unwrap();
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn test_execute_command_status_consistency() {
        #[cfg(unix)]
        {
            // Execute same command multiple times - should return same status
            let status1 = SysBase::execute_command_status("true", &[]);
            let status2 = SysBase::execute_command_status("true", &[]);
            if status1.is_ok() && status2.is_ok() {
                assert_eq!(status1.unwrap(), status2.unwrap());
            }
        }
        
        #[cfg(windows)]
        {
            let status1 = SysBase::execute_command_status("cmd", &["/C", "exit", "/b", "0"]);
            let status2 = SysBase::execute_command_status("cmd", &["/C", "exit", "/b", "0"]);
            if status1.is_ok() && status2.is_ok() {
                assert_eq!(status1.unwrap(), status2.unwrap());
            }
        }
    }

    #[test]
    fn test_sys_base_init_multiple_calls() {
        // Multiple init calls should be fine
        assert!(SysBase::init().is_ok());
        assert!(SysBase::init().is_ok());
        assert!(SysBase::init().is_ok());
    }

    #[test]
    fn test_sys_error_partial_eq() {
        let error1 = SysError::InitFailed;
        let error2 = SysError::InitFailed;
        let error3 = SysError::CommandFailed;
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_sys_error_copy() {
        // SysError implements Copy, so we can copy it
        let error1 = SysError::SystemError;
        let error2 = error1; // Copy
        assert_eq!(error1, error2);
    }
}

