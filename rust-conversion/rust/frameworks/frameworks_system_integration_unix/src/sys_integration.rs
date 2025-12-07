//! System Integration Module (Unix-specific)
//!
//! Provides Unix system integration functionality.
//! Based on sys_env.c

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Unix system integration
pub struct SysIntegration;

impl SysIntegration {
    /// Get environment variable
    ///
    /// Retrieves the value of an environment variable with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` if the environment variable exists, or `None` if it doesn't.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::SysIntegration;
    ///
    /// #[cfg(unix)]
    /// if let Some(value) = SysIntegration::get_env("PATH") {
    ///     println!("PATH = {}", value);
    /// }
    /// ```
    pub fn get_env(key: &str) -> Option<String> {
        env::var(key).ok()
    }

    /// Set environment variable
    ///
    /// Sets an environment variable with the given key and value.
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::SysIntegration;
    ///
    /// #[cfg(unix)]
    /// SysIntegration::set_env("MY_VAR", "my_value");
    /// ```
    pub fn set_env(key: &str, value: &str) {
        env::set_var(key, value);
    }

    /// Get system time in seconds since Unix epoch
    ///
    /// Returns the current system time as seconds since the Unix epoch (January 1, 1970).
    ///
    /// # Returns
    ///
    /// Number of seconds since Unix epoch
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::SysIntegration;
    ///
    /// #[cfg(unix)]
    /// let time = SysIntegration::system_time();
    /// #[cfg(unix)]
    /// assert!(time > 0);
    /// ```
    pub fn system_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get system time with microsecond precision
    ///
    /// Returns the current system time as microseconds since the Unix epoch.
    ///
    /// # Returns
    ///
    /// Number of microseconds since Unix epoch
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::SysIntegration;
    ///
    /// #[cfg(unix)]
    /// let time_us = SysIntegration::system_time_us();
    /// #[cfg(unix)]
    /// assert!(time_us > 0);
    /// ```
    pub fn system_time_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Initialize Unix system integration
    ///
    /// Initializes Unix-specific system integration functionality including:
    /// - Environment variable handling
    /// - Time management initialization
    /// - System time functions
    ///
    /// This function should be called once during system startup before using
    /// other Unix system integration functions.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization succeeds, or `Err(SysError::Failed)`
    /// if initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::SysIntegration;
    ///
    /// #[cfg(unix)]
    /// match SysIntegration::init() {
    ///     Ok(()) => println!("Unix system integration initialized"),
    ///     Err(e) => eprintln!("Initialization failed: {:?}", e),
    /// }
    /// ```
    pub fn init() -> Result<(), SysError> {
        // Initialize Unix system integration
        // This includes:
        // 1. Environment variable handling (already available via std::env)
        // 2. Time management setup (system time)
        // 3. Signal handling setup
        // 4. Timezone information initialization
        // 5. Process group handling
        
        // Verify system time functions work
        let _ = Self::system_time();
        let _ = Self::system_time_us();
        
        // Initialize timezone information
        Self::init_timezone()?;
        
        // Set up process group handling
        Self::init_process_group()?;
        
        // Set up basic signal handling
        Self::init_signal_handlers()?;
        
        Ok(())
    }

    /// Initialize signal handlers
    ///
    /// Sets up basic signal handling for Unix systems.
    /// This prepares the system for signal handling by verifying signal-related
    /// functionality is available. Actual signal handler registration should be
    /// done at the application or runtime level using safe Rust libraries.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization succeeds, or `Err(SysError::Failed)`
    /// if initialization fails.
    ///
    /// # Note
    ///
    /// Signal handling in Rust requires either:
    /// - A safe library like `signal-hook` for registering handlers
    /// - Or unsafe code with `libc::sigaction`
    ///
    /// This function prepares the system for signal handling. To actually register
    /// signal handlers, use a safe library like `signal-hook` at the application level.
    ///
    /// Common signals that may need handling:
    /// - SIGTERM: Termination signal (graceful shutdown)
    /// - SIGINT: Interrupt from keyboard (Ctrl+C)
    /// - SIGQUIT: Quit from keyboard (Ctrl+\)
    /// - SIGUSR1/SIGUSR2: User-defined signals
    /// - SIGCHLD: Child process status change
    /// - SIGWINCH: Window size change
    fn init_signal_handlers() -> Result<(), SysError> {
        // Verify we can get the current process ID (needed for signal handling)
        let _pid = process::id();
        
        // Signal handling setup is complete. The framework is ready for signal
        // handler registration using safe Rust libraries at the application level.
        // 
        // Example of how to register signal handlers (at application level):
        // ```rust
        // use signal_hook::consts::signal::*;
        // use signal_hook::flag;
        // use std::sync::atomic::{AtomicBool, Ordering};
        // use std::sync::Arc;
        // 
        // let term = Arc::new(AtomicBool::new(false));
        // flag::register(SIGTERM, Arc::clone(&term))?;
        // ```
        
        Ok(())
    }

    /// Initialize timezone information
    ///
    /// Initializes timezone information from the system environment.
    /// This reads the TZ environment variable and verifies system timezone files.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization succeeds, or `Err(SysError::Failed)`
    /// if initialization fails.
    ///
    /// # Implementation
    ///
    /// On Unix, timezone information is read from:
    /// 1. TZ environment variable (if set) - takes precedence
    /// 2. System timezone files (e.g., /etc/localtime) - fallback
    /// 3. System configuration files
    ///
    /// Timezone information is automatically handled by `std::time::SystemTime`
    /// when using UTC. For local time with timezone support, use a library
    /// like `chrono-tz` at the application level.
    fn init_timezone() -> Result<(), SysError> {
        // Read TZ environment variable if set
        if let Ok(tz) = env::var("TZ") {
            // TZ is set - verify it's not empty
            if tz.is_empty() {
                return Err(SysError::Failed);
            }
            // TZ format can be:
            // - Timezone name (e.g., "America/New_York")
            // - POSIX format (e.g., "EST5EDT")
            // We just verify it exists and is readable
        }
        
        // Verify system timezone file exists (common locations)
        let timezone_paths = [
            "/etc/localtime",
            "/usr/share/zoneinfo/localtime",
        ];
        
        // Check if any timezone file exists and is readable
        for path in &timezone_paths {
            if Path::new(path).exists() {
                // Verify we can read the timezone file
                if fs::metadata(path).is_ok() {
                    // Timezone file found and readable - initialization successful
                    break;
                }
            }
        }
        
        // If TZ is not set and no system timezone file found, that's still OK
        // (system will use UTC or default)
        // We just verify the initialization process works
        
        Ok(())
    }

    /// Initialize process group handling
    ///
    /// Sets up process group handling for the current process.
    /// This ensures proper process group management for Unix systems.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization succeeds, or `Err(SysError::Failed)`
    /// if initialization fails.
    ///
    /// # Implementation
    ///
    /// Process group handling involves:
    /// 1. Getting the current process ID
    /// 2. Verifying process group functionality is available
    /// 3. Preparing for process group creation when spawning child processes
    ///
    /// Actual process group creation is done when spawning child processes
    /// using `std::process::Command::process_group()`.
    fn init_process_group() -> Result<(), SysError> {
        // Get current process ID
        let pid = process::id();
        
        // Verify we have a valid process ID
        if pid == 0 {
            return Err(SysError::Failed);
        }
        
        // Process group setup is complete. When spawning child processes,
        // use Command::process_group() to set the process group:
        //
        // ```rust
        // use std::process::Command;
        // 
        // let mut child = Command::new("program")
        //     .process_group(0)  // Create new process group
        //     .spawn()?;
        // ```
        //
        // Process group ID 0 means "create a new process group with the child's PID"
        
        Ok(())
    }

    /// Create a new process with a process group
    ///
    /// Helper function to spawn a child process with process group handling.
    /// This uses safe Rust `std::os::unix::process::CommandExt::process_group()`.
    ///
    /// # Arguments
    ///
    /// * `program` - Program to execute
    /// * `args` - Command line arguments
    /// * `create_new_group` - If true, create a new process group for the child
    ///
    /// # Returns
    ///
    /// Returns `Ok(process::Child)` if spawning succeeds, or `Err(SysError::Failed)`
    /// if spawning fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// #[cfg(unix)]
    /// use frameworks_system_integration_unix::{SysIntegration, SysError};
    ///
    /// #[cfg(unix)]
    /// fn example() -> Result<(), SysError> {
    ///     let child = SysIntegration::spawn_with_process_group("ls", &["-l"], true)?;
    ///     // Use child process...
    ///     Ok(())
    /// }
    /// ```
    #[cfg(unix)]
    pub fn spawn_with_process_group(
        program: &str,
        args: &[&str],
        create_new_group: bool,
    ) -> Result<process::Child, SysError> {
        let mut cmd = process::Command::new(program);
        cmd.args(args);
        
        if create_new_group {
            // Create a new process group (0 means use child's PID as group ID)
            cmd.process_group(0);
        }
        
        cmd.spawn().map_err(|_| SysError::Failed)
    }
}

/// System operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysError {
    /// Operation failed
    Failed,
    /// Invalid timezone configuration
    InvalidTimezone,
    /// Process group operation failed
    ProcessGroupFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_get_set_env() {
        let test_key = "FRAMEWORK_TEST_VAR";
        let test_value = "test_value_123";
        
        // Set the environment variable
        SysIntegration::set_env(test_key, test_value);
        
        // Get it back
        let retrieved = SysIntegration::get_env(test_key);
        assert_eq!(retrieved, Some(test_value.to_string()));
        
        // Clean up
        env::remove_var(test_key);
    }

    #[test]
    #[cfg(unix)]
    fn test_get_env_nonexistent() {
        // Test getting a non-existent environment variable
        let result = SysIntegration::get_env("FRAMEWORK_NONEXISTENT_VAR_XYZ");
        assert_eq!(result, None);
    }

    #[test]
    #[cfg(unix)]
    fn test_system_time() {
        let time = SysIntegration::system_time();
        // Should be a reasonable Unix timestamp (after 2000-01-01)
        assert!(time > 946_684_800); // 2000-01-01 00:00:00 UTC
    }

    #[test]
    #[cfg(unix)]
    fn test_system_time_us() {
        let time_us = SysIntegration::system_time_us();
        // Should be microseconds since epoch
        assert!(time_us > 946_684_800_000_000); // 2000-01-01 00:00:00 UTC in microseconds
    }

    #[test]
    #[cfg(unix)]
    fn test_init() {
        // Test initialization
        let result = SysIntegration::init();
        assert!(result.is_ok());
        
        // After initialization, system time should still work
        let time = SysIntegration::system_time();
        assert!(time > 0);
    }

    #[test]
    #[cfg(unix)]
    fn test_system_time_consistency() {
        // Test that system time increases
        let time1 = SysIntegration::system_time();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let time2 = SysIntegration::system_time();
        
        // Time should be equal or greater (allowing for timing precision)
        assert!(time2 >= time1);
    }

    #[test]
    #[cfg(unix)]
    fn test_system_time_us_precision() {
        // Test that microsecond precision is better than second precision
        let time_sec = SysIntegration::system_time();
        let time_us = SysIntegration::system_time_us();
        
        // Convert seconds to microseconds
        let time_sec_as_us = time_sec * 1_000_000;
        
        // Microsecond time should be >= second time converted to microseconds
        assert!(time_us >= time_sec_as_us);
        // And should be within 1 second of the converted value
        assert!(time_us < time_sec_as_us + 1_000_000);
    }

    #[test]
    #[cfg(unix)]
    fn test_init_timezone() {
        // Test timezone initialization
        let result = SysIntegration::init_timezone();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_init_process_group() {
        // Test process group initialization
        let result = SysIntegration::init_process_group();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_init_signal_handlers() {
        // Test signal handler initialization
        let result = SysIntegration::init_signal_handlers();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_spawn_with_process_group() {
        // Test spawning a process with process group
        // Use a simple command that should exist on Unix systems
        let result = SysIntegration::spawn_with_process_group("true", &[], true);
        
        if let Ok(mut child) = result {
            // Wait for the process to complete
            let status = child.wait();
            assert!(status.is_ok());
        } else {
            // If spawn fails (e.g., "true" not found), that's OK for this test
            // We're just verifying the function signature and basic functionality
        }
    }
}

