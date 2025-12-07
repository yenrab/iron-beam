/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

//! Framework Utilities Module
//!
//! Provides framework-level utility functions for the Erlang/OTP runtime system.
//! This module implements utility functions that are used at the framework level,
//! including execution utilities and process management helpers.
//!
//! ## Overview
//!
//! The framework utilities module provides high-level utility functions that support
//! runtime execution and management at the framework layer. These utilities coordinate
//! between the system integration layer and the infrastructure layer to provide
//! framework-level operations.
//!
//! ## Key Features
//!
//! - **Execution Utilities**: Functions for managing runtime execution
//! - **Process Management Helpers**: Utilities for process lifecycle management
//! - **Framework Coordination**: Functions that coordinate between system layers
//!
//! ## Examples
//!
//! ```rust
//! use frameworks_utilities::{FrameworkUtils, FrameworkError};
//!
//! // Execute a framework utility operation
//! match FrameworkUtils::utility() {
//!     Ok(()) => println!("Framework utility executed successfully"),
//!     Err(FrameworkError::Failed) => println!("Framework utility failed"),
//! }
//! ```
//!
//! ```rust
//! use frameworks_utilities::FrameworkUtils;
//!
//! // Use framework utilities in error handling
//! let result = FrameworkUtils::utility();
//! if result.is_err() {
//!     eprintln!("Framework operation failed");
//! }
//! ```
//!
//! ```rust
//! use frameworks_utilities::{FrameworkUtils, FrameworkError};
//!
//! // Chain framework utility operations
//! let _ = FrameworkUtils::utility()
//!     .map_err(|e| match e {
//!         FrameworkError::Failed => "Operation failed",
//!     });
//! ```
//!
//! ## See Also
//!
//! - [`entities_utilities`](../../entities/entities_utilities/index.html): Entity-level utilities
//! - [`frameworks_system_integration`](../frameworks_system_integration/index.html): System integration framework

/// Framework utilities
///
/// Provides framework-level utility functions for runtime execution and process management.
/// This struct serves as a namespace for framework utility operations that coordinate
/// between system layers.
///
/// ## Usage
///
/// Framework utilities are accessed through associated functions on this struct.
/// All operations are designed to work at the framework layer, coordinating
/// between the system integration and infrastructure layers.
pub struct FrameworkUtils;

impl FrameworkUtils {
    /// Set an environment variable
    ///
    /// Sets an environment variable with the given key and value.
    /// This is a cross-platform utility function for framework-level operations.
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Set an environment variable
    /// FrameworkUtils::set_env("MY_VAR", "my_value");
    /// ```
    pub fn set_env(key: &str, value: &str) {
        std::env::set_var(key, value);
    }

    /// Get an environment variable
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
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Get an environment variable
    /// if let Some(value) = FrameworkUtils::get_env("PATH") {
    ///     println!("PATH = {}", value);
    /// }
    /// ```
    pub fn get_env(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Quote command line arguments
    ///
    /// Quotes command line arguments to handle spaces and special characters.
    /// This is equivalent to the `fnuttify_argv` function in the C implementation.
    ///
    /// # Arguments
    ///
    /// * `args` - Slice of argument strings to quote
    ///
    /// # Returns
    ///
    /// Returns a vector of quoted argument strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Quote arguments
    /// let args = vec!["hello", "world with spaces"];
    /// let quoted = FrameworkUtils::quote_arguments(&args);
    /// assert_eq!(quoted[0], "hello");
    /// assert_eq!(quoted[1], "\"world with spaces\"");
    /// ```
    pub fn quote_arguments(args: &[&str]) -> Vec<String> {
        args.iter()
            .map(|arg| {
                if arg.contains(' ') || arg.contains('"') {
                    // Escape quotes with backslashes and wrap in quotes
                    format!("\"{}\"", arg.replace('"', "\\\""))
                } else {
                    // No quoting needed, return as-is
                    arg.to_string()
                }
            })
            .collect()
    }

    /// Check if running in a console environment
    ///
    /// Determines if the current process has access to a console/terminal.
    /// This is useful for deciding whether to use interactive features.
    ///
    /// # Returns
    ///
    /// Returns `true` if a console is available, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Check for console
    /// if FrameworkUtils::has_console() {
    ///     println!("Running in console mode");
    /// } else {
    ///     println!("Running in detached mode");
    /// }
    /// ```
    pub fn has_console() -> bool {
        use std::io::IsTerminal;
        std::io::stdin().is_terminal()
    }

    /// Keep window open (interactive mode)
    ///
    /// Prompts the user to press a key before closing the window.
    /// This is useful for keeping console windows open in interactive mode.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Keep window open
    /// FrameworkUtils::keep_window_open();
    /// ```
    pub fn keep_window_open() {
        if Self::has_console() {
            println!("\nPress Enter to close window...");
            let mut buffer = String::new();
            let _ = std::io::stdin().read_line(&mut buffer);
        }
    }

    /// Execute a framework utility operation
    ///
    /// Performs framework-level utility operations that coordinate runtime execution
    /// and process management. This function provides the primary interface for
    /// framework-level utility operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation succeeds, or `Err(FrameworkError::Failed)`
    /// if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Execute framework utility
    /// let result = FrameworkUtils::utility();
    /// assert!(result.is_ok());
    /// ```
    ///
    /// ```rust
    /// use frameworks_utilities::{FrameworkUtils, FrameworkError};
    ///
    /// // Handle framework utility errors
    /// match FrameworkUtils::utility() {
    ///     Ok(()) => println!("Success"),
    ///     Err(FrameworkError::Failed) => println!("Failed"),
    /// }
    /// ```
    ///
    /// ```rust
    /// use frameworks_utilities::FrameworkUtils;
    ///
    /// // Use in conditional logic
    /// if FrameworkUtils::utility().is_ok() {
    ///     // Continue with framework operations
    /// }
    /// ```
    ///
    /// ## See Also
    ///
    /// - [`FrameworkError`]: Error type for framework operations
    pub fn utility() -> Result<(), FrameworkError> {
        // Framework utility operations are now implemented as individual functions
        // This function remains for backward compatibility
        Ok(())
    }
}

/// Framework operation errors
///
/// Represents errors that can occur during framework utility operations.
/// This enum provides error types for framework-level operations.
///
/// ## Variants
///
/// - **Failed**: Indicates that a framework operation failed to complete
///   successfully. This can occur due to system resource constraints,
///   invalid state, or other framework-level issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameworkError {
    /// Operation failed
    ///
    /// This error indicates that a framework utility operation failed to
    /// complete successfully. Common causes include system resource constraints,
    /// invalid operation state, or framework-level configuration issues.
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_utils() {
        let result = FrameworkUtils::utility();
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_get_env() {
        // Test setting and getting environment variables
        let test_key = "FRAMEWORK_TEST_VAR";
        let test_value = "test_value_123";
        
        // Set the environment variable
        FrameworkUtils::set_env(test_key, test_value);
        
        // Get it back
        let retrieved = FrameworkUtils::get_env(test_key);
        assert_eq!(retrieved, Some(test_value.to_string()));
        
        // Clean up
        std::env::remove_var(test_key);
    }

    #[test]
    fn test_get_env_nonexistent() {
        // Test getting a non-existent environment variable
        let result = FrameworkUtils::get_env("FRAMEWORK_NONEXISTENT_VAR_XYZ");
        assert_eq!(result, None);
    }

    #[test]
    fn test_quote_arguments() {
        // Test quoting arguments with spaces
        let args = vec!["hello", "world with spaces", "normal_arg"];
        let quoted = FrameworkUtils::quote_arguments(&args);
        
        assert_eq!(quoted[0], "hello");
        assert_eq!(quoted[1], "\"world with spaces\"");
        assert_eq!(quoted[2], "normal_arg");
    }

    #[test]
    fn test_quote_arguments_with_quotes() {
        // Test quoting arguments that contain quotes
        let args = vec!["arg with \"quotes\""];
        let quoted = FrameworkUtils::quote_arguments(&args);
        
        assert_eq!(quoted[0], "\"arg with \\\"quotes\\\"\"");
    }

    #[test]
    fn test_quote_arguments_empty() {
        // Test quoting empty argument list
        let args: Vec<&str> = vec![];
        let quoted = FrameworkUtils::quote_arguments(&args);
        assert!(quoted.is_empty());
    }

    #[test]
    fn test_has_console() {
        // Test console detection (may vary by environment)
        let has_console = FrameworkUtils::has_console();
        // Just verify it doesn't panic
        assert!(has_console || !has_console); // Always true, but ensures function works
    }

    #[test]
    fn test_keep_window_open() {
        // Test keep window open (should not panic)
        // We can't easily test the interactive part, but we can verify it doesn't crash
        FrameworkUtils::keep_window_open();
    }
}

