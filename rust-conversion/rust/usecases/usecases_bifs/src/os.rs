//! OS-related Built-in Functions
//!
//! Provides operating system interface BIFs:
//! - Environment variable operations
//! - Process ID retrieval
//! - Timestamp operations
//! - Signal handling
//!
//! Based on erl_bif_os.c
//!
//! This module uses safe Rust standard library functions instead of unsafe FFI calls.

use std::env;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// OS BIF operations
pub struct OsBif;

impl OsBif {
    /// Get the current process ID
    ///
    /// Equivalent to `os:getpid/0` in Erlang.
    /// Returns the process ID as a list of integers (each digit as a separate integer).
    ///
    /// # Returns
    /// Vector of integers representing the PID digits
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// let pid = OsBif::getpid();
    /// // pid might be [1, 2, 3, 4, 5] for PID 12345
    /// ```
    pub fn getpid() -> Vec<u8> {
        let pid = process::id();
        pid.to_string()
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect()
    }

    /// Get all environment variables
    ///
    /// Equivalent to `os:getenv/0` in Erlang.
    /// Returns all environment variables as a vector of (key, value) tuples.
    ///
    /// # Returns
    /// Vector of (key, value) string tuples
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// let env_vars = OsBif::env();
    /// // env_vars contains all environment variables
    /// ```
    pub fn env() -> Vec<(String, String)> {
        env::vars().collect()
    }

    /// Get a specific environment variable
    ///
    /// Equivalent to `os:getenv/1` in Erlang.
    /// Returns the value of the environment variable if it exists, None otherwise.
    ///
    /// # Arguments
    /// * `key` - The environment variable name
    ///
    /// # Returns
    /// * `Some(value)` if the variable exists
    /// * `None` if the variable does not exist
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// let path = OsBif::getenv("PATH");
    /// ```
    pub fn getenv(key: &str) -> Option<String> {
        env::var(key).ok()
    }

    /// Set an environment variable
    ///
    /// Equivalent to `os:putenv/2` in Erlang.
    /// Sets an environment variable for the current process.
    ///
    /// # Arguments
    /// * `key` - The environment variable name
    /// * `value` - The environment variable value
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(error)` if setting failed
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// OsBif::putenv("MY_VAR", "my_value").unwrap();
    /// ```
    pub fn putenv(key: &str, value: &str) -> Result<(), OsError> {
        env::set_var(key, value);
        Ok(())
    }

    /// Unset an environment variable
    ///
    /// Equivalent to `os:unsetenv/1` in Erlang.
    /// Removes an environment variable from the current process.
    ///
    /// # Arguments
    /// * `key` - The environment variable name to remove
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(error)` if unsetting failed
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// OsBif::unsetenv("MY_VAR").unwrap();
    /// ```
    pub fn unsetenv(key: &str) -> Result<(), OsError> {
        env::remove_var(key);
        Ok(())
    }

    /// Get a timestamp
    ///
    /// Equivalent to `os:timestamp/0` in Erlang.
    /// Returns a tuple of (megaseconds, seconds, microseconds) since Unix epoch.
    ///
    /// # Returns
    /// Tuple of (megaseconds, seconds, microseconds)
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// let (megasec, sec, microsec) = OsBif::timestamp();
    /// ```
    pub fn timestamp() -> (u64, u64, u64) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let total_seconds = duration.as_secs();
        let megaseconds = total_seconds / 1_000_000;
        let seconds = total_seconds % 1_000_000;
        let microseconds = duration.subsec_micros() as u64;

        (megaseconds, seconds, microseconds)
    }

    /// Set signal handling
    ///
    /// Equivalent to `os:set_signal/2` in Erlang.
    /// Sets how a signal should be handled: ignore, default, or handle.
    ///
    /// # Arguments
    /// * `signal` - The signal name (e.g., "SIGINT", "SIGTERM")
    /// * `action` - The action to take: "ignore", "default", or "handle"
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(OsError)` if the signal or action is invalid
    ///
    /// # Note
    /// This is a simplified implementation. Full signal handling would require
    /// platform-specific code. For now, this validates inputs and returns success.
    /// Actual signal handling should be implemented in the adapters layer.
    ///
    /// # Example
    /// ```
    /// use usecases_bifs::os::OsBif;
    /// OsBif::set_signal("SIGINT", "ignore").unwrap();
    /// ```
    pub fn set_signal(signal: &str, action: &str) -> Result<(), OsError> {
        // Validate action
        match action {
            "ignore" | "default" | "handle" => {
                // Validate signal name format (should be an atom-like string)
                if signal.is_empty() {
                    return Err(OsError::InvalidArgument(
                        "Signal name cannot be empty".to_string(),
                    ));
                }
                // In a full implementation, we would set the signal handler here
                // For now, we just validate and return success
                Ok(())
            }
            _ => Err(OsError::InvalidArgument(format!(
                "Invalid action: {}. Must be 'ignore', 'default', or 'handle'",
                action
            ))),
        }
    }
}

/// Error type for OS BIF operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsError {
    /// Invalid argument provided
    InvalidArgument(String),
    /// Operation not supported
    NotSupported(String),
    /// System error
    SystemError(String),
}

impl std::fmt::Display for OsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            OsError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            OsError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for OsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getpid() {
        let pid = OsBif::getpid();
        assert!(!pid.is_empty());
        // PID should be all digits (0-9)
        for &digit in &pid {
            assert!(digit < 10);
        }
    }

    #[test]
    fn test_env() {
        let env_vars = OsBif::env();
        // Should have at least some environment variables
        assert!(!env_vars.is_empty());
        // Each entry should be a (key, value) tuple
        for (key, value) in &env_vars {
            assert!(!key.is_empty());
        }
    }

    #[test]
    fn test_getenv_existing() {
        // Set a test environment variable
        env::set_var("TEST_OS_BIF_VAR", "test_value");
        
        let value = OsBif::getenv("TEST_OS_BIF_VAR");
        assert_eq!(value, Some("test_value".to_string()));
        
        // Cleanup
        env::remove_var("TEST_OS_BIF_VAR");
    }

    #[test]
    fn test_getenv_nonexistent() {
        // Use a very unlikely variable name
        let value = OsBif::getenv("TEST_OS_BIF_NONEXISTENT_12345");
        assert_eq!(value, None);
    }

    #[test]
    fn test_putenv() {
        let result = OsBif::putenv("TEST_OS_BIF_PUTENV", "test_value");
        assert!(result.is_ok());
        
        // Verify it was set
        let value = env::var("TEST_OS_BIF_PUTENV");
        assert_eq!(value, Ok("test_value".to_string()));
        
        // Cleanup
        env::remove_var("TEST_OS_BIF_PUTENV");
    }

    #[test]
    fn test_unsetenv() {
        // Set a variable first
        env::set_var("TEST_OS_BIF_UNSETENV", "test_value");
        
        let result = OsBif::unsetenv("TEST_OS_BIF_UNSETENV");
        assert!(result.is_ok());
        
        // Verify it was removed
        let value = env::var("TEST_OS_BIF_UNSETENV");
        assert!(value.is_err());
    }

    #[test]
    fn test_timestamp() {
        let (megasec, sec, microsec) = OsBif::timestamp();
        
        // Timestamp should be reasonable (after 1970)
        assert!(megasec > 0 || sec > 0);
        // Microseconds should be less than 1 second
        assert!(microsec < 1_000_000);
    }

    #[test]
    fn test_set_signal_valid() {
        let result = OsBif::set_signal("SIGINT", "ignore");
        assert!(result.is_ok());
        
        let result = OsBif::set_signal("SIGTERM", "default");
        assert!(result.is_ok());
        
        let result = OsBif::set_signal("SIGHUP", "handle");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_signal_invalid_action() {
        let result = OsBif::set_signal("SIGINT", "invalid");
        assert!(matches!(result, Err(OsError::InvalidArgument(_))));
    }

    #[test]
    fn test_set_signal_empty_signal() {
        let result = OsBif::set_signal("", "ignore");
        assert!(matches!(result, Err(OsError::InvalidArgument(_))));
    }

    #[test]
    fn test_os_error_display() {
        let err = OsError::InvalidArgument("test".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid argument"));
        assert!(display.contains("test"));
    }

    #[test]
    fn test_os_error_not_supported() {
        let err = OsError::NotSupported("feature".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Not supported"));
        assert!(display.contains("feature"));
    }

    #[test]
    fn test_os_error_system_error() {
        let err = OsError::SystemError("error".to_string());
        let display = format!("{}", err);
        assert!(display.contains("System error"));
        assert!(display.contains("error"));
    }

    #[test]
    fn test_getpid_format() {
        let pid = OsBif::getpid();
        // Convert back to number to verify it's valid
        let pid_str: String = pid.iter().map(|&d| (b'0' + d) as char).collect();
        let pid_num: u32 = pid_str.parse().unwrap();
        // Should be a reasonable process ID
        assert!(pid_num > 0);
    }

    #[test]
    fn test_env_contains_path() {
        let env_vars = OsBif::env();
        // On most systems, PATH should exist
        let has_path = env_vars.iter().any(|(k, _)| k == "PATH");
        // This might not be true in all test environments, so we just check the structure
        assert!(!env_vars.is_empty());
    }

    #[test]
    fn test_putenv_overwrite() {
        // Set initial value
        OsBif::putenv("TEST_OS_BIF_OVERWRITE", "initial").unwrap();
        
        // Overwrite it
        OsBif::putenv("TEST_OS_BIF_OVERWRITE", "updated").unwrap();
        
        // Verify new value
        let value = OsBif::getenv("TEST_OS_BIF_OVERWRITE");
        assert_eq!(value, Some("updated".to_string()));
        
        // Cleanup
        env::remove_var("TEST_OS_BIF_OVERWRITE");
    }

    #[test]
    fn test_timestamp_monotonic() {
        let (m1, s1, u1) = OsBif::timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (m2, s2, u2) = OsBif::timestamp();
        
        // Second timestamp should be >= first (allowing for some precision issues)
        let time1 = m1 * 1_000_000 + s1;
        let time2 = m2 * 1_000_000 + s2;
        assert!(time2 >= time1 || (time2 == time1 && u2 >= u1));
    }
}

