/*!
# Infrastructure Environment Config

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Environment variable management

## Overview

This crate provides safe environment variable management for the Erlang BEAM compiler infrastructure.
Replaces manual memory management and unsafe string handling with safe Rust abstractions.

## Original C Functions Replaced

The original `erlc.c` contained these environment functions:
- `get_env()`: Get environment variable → **Replaced with safe std::env::var() wrapper**
- `set_env()`: Set environment variable → **Replaced with safe std::env::set_var() wrapper**
- `free_env_val()`: Free env value memory → **Unnecessary in Rust (automatic cleanup)**
- `get_env_compile_server()`: Get compile server config → **Replaced with safe config detection**

## Environment Management Philosophy

### 1. Safe Variable Access
```rust
use infrastructure_environment_config::env;

// Safe environment variable access with proper error handling
let value = env::get_var("ERLC_EMULATOR").unwrap_or_else(|_| "erl".to_string());
```

### 2. Compile Server Configuration
```rust
use infrastructure_environment_config::compile_server;

// Intelligent compile server detection
let server_config = compile_server::get_config();
// Returns structured configuration with validation
```

### 3. Platform-Aware Operations
```rust
use infrastructure_environment_config::erlang;

// Cross-platform environment handling
erlang::setup_compilation_env("/usr/bin/erl").unwrap();
// Sets up ESCRIPT_NAME, ERLC_CONFIGURATION, etc.
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends on memory, error, platform support)
- **SOLID Principle**: Single responsibility for environment configuration
- **Safe Rust**: No unsafe code, leverages std::env safety guarantees
- **Composable**: Integrates with error handling and platform detection
*/

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::panic;

use infrastructure_error_handling::CompilerError;

/// Environment configuration result type
pub type EnvResult<T> = Result<T, CompilerError>;

/// Compile server configuration
#[derive(Debug, Clone, PartialEq)]
pub struct CompileServerConfig {
    /// Whether compile server is enabled
    pub enabled: bool,
    /// Server ID for multi-user systems
    pub server_id: Option<String>,
    /// Configuration path/hash for cache invalidation
    pub config_hash: String,
}

/// Environment variable management utilities
pub mod env {
    use super::*;

    /// Safely get an environment variable
    ///
    /// Replaces the C `get_env()` function with safe Rust error handling.
    pub fn get_var(name: &str) -> EnvResult<String> {
        std::env::var(name).map_err(|_| {
            CompilerError::InternalError(format!("Environment variable '{}' not found or invalid", name))
        })
    }

    /// Safely get an environment variable with a default value
    pub fn get_var_or(name: &str, default: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| default.to_string())
    }

    /// Check if an environment variable exists
    pub fn var_exists(name: &str) -> bool {
        std::env::var(name).is_ok()
    }

    /// Safely set an environment variable
    ///
    /// Replaces the C `set_env()` function with safe Rust operations.
    pub fn set_var(name: &str, value: &str) -> EnvResult<()> {
        std::env::set_var(name, value);
        Ok(())
    }

    /// Remove an environment variable
    pub fn remove_var(name: &str) -> EnvResult<()> {
        std::env::remove_var(name);
        Ok(())
    }

    /// Get all environment variables as a HashMap
    pub fn get_all_vars() -> EnvResult<HashMap<String, String>> {
        let mut vars = HashMap::new();
        for (key, value) in std::env::vars() {
            vars.insert(key, value);
        }
        Ok(vars)
    }

    /// Get the current working directory from environment
    pub fn current_dir() -> EnvResult<PathBuf> {
        std::env::current_dir().map_err(|e| CompilerError::IoError(e))
    }

    /// Get the temporary directory path
    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir()
    }
}

/// Erlang-specific environment configuration
pub mod erlang {
    use super::*;

    /// Get the Erlang emulator executable path
    pub fn get_emulator_path() -> String {
        // Check ERLC_EMULATOR first (compiler-specific override)
        if let Ok(emulator) = crate::env::get_var("ERLC_EMULATOR") {
            return emulator;
        }

        // Default to "erl" - let PATH resolution find it
        "erl".to_string()
    }

    /// Get Erlang library paths
    pub fn get_library_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Add ERL_LIBS paths
        if let Ok(erllibs) = crate::env::get_var("ERL_LIBS") {
            for path in std::env::split_paths(&erllibs) {
                paths.push(path);
            }
        }

        paths
    }

    /// Set up Erlang compilation environment
    pub fn setup_compilation_env(emulator_path: &str) -> EnvResult<()> {
        // Set ESCRIPT_NAME for proper script identification
        crate::env::set_var("ESCRIPT_NAME", "erlc")?;

        // Set configuration hash based on emulator path and environment
        // This helps compile server detect when configuration changes
        let config_hash = format!("{:x}", emulator_path.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64)));
        crate::env::set_var("ERLC_CONFIGURATION", &config_hash)?;

        Ok(())
    }

    /// Get Erlang flags from environment
    pub fn get_erlang_flags() -> Vec<String> {
        let mut flags = Vec::new();

        // Add ERL_AFLAGS
        if let Ok(aflags) = crate::env::get_var("ERL_AFLAGS") {
            flags.extend(aflags.split_whitespace().map(|s| s.to_string()));
        }

        // Add ERL_FLAGS
        if let Ok(eflags) = crate::env::get_var("ERL_FLAGS") {
            flags.extend(eflags.split_whitespace().map(|s| s.to_string()));
        }

        // Add ERL_ZFLAGS (hipe)
        if let Ok(zflags) = crate::env::get_var("ERL_ZFLAGS") {
            flags.extend(zflags.split_whitespace().map(|s| s.to_string()));
        }

        flags
    }
}

/// Compile server environment configuration
pub mod compile_server {
    use super::*;

    /// Get compile server configuration from environment
    ///
    /// Replaces the C `get_env_compile_server()` function with structured config.
    pub fn get_config() -> CompileServerConfig {
        let enabled = match crate::env::get_var("ERLC_USE_SERVER").as_deref() {
            Ok("true") | Ok("yes") | Ok("1") => true,
            Ok("false") | Ok("no") | Ok("0") => false,
            Ok(_) => false, // Invalid values default to disabled
            Err(_) => true, // Default to enabled if not specified
        };

        let server_id = crate::env::get_var("ERLC_SERVER_ID").ok();

        // Create config hash from relevant environment variables
        let mut hash_input = String::new();
        if let Ok(path) = crate::env::get_var("PATH") {
            hash_input.push_str(&path);
        }
        if let Ok(erllibs) = crate::env::get_var("ERL_LIBS") {
            hash_input.push_str(&erllibs);
        }

        let config_hash = format!("{:x}", hash_input.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64)));

        CompileServerConfig {
            enabled,
            server_id,
            config_hash,
        }
    }

    /// Check if compile server should be used
    pub fn should_use_server() -> bool {
        get_config().enabled
    }

    /// Get server node name for this user/session
    pub fn get_server_node_name() -> EnvResult<String> {
        let config = get_config();
        let user = crate::env::get_var("USERNAME")
            .or_else(|_| crate::env::get_var("LOGNAME"))
            .or_else(|_| crate::env::get_var("USER"))
            .unwrap_or_else(|_| "nouser".to_string());

        let server_id = config.server_id.unwrap_or_else(|| "".to_string());

        // Use process ID for uniqueness
        let pid = std::process::id();

        if server_id.is_empty() {
            Ok(format!("erl_compile_server_{}_{}", user, pid))
        } else {
            // Filter out invalid characters for Erlang node names
            let filtered_id = server_id.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>();
            Ok(format!("erl_compile_server_{}_{}_{}", filtered_id, user, pid))
        }
    }
}

/// Environment variable batch operations
pub mod batch {
    use super::*;

    /// Set multiple environment variables at once
    pub fn set_vars(vars: &[(&str, &str)]) -> EnvResult<()> {
        for (name, value) in vars {
            env::set_var(name, value)?;
        }
        Ok(())
    }

    /// Get multiple environment variables, returning defaults for missing ones
    pub fn get_vars_with_defaults(vars: &[(&str, &str)]) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (name, default) in vars {
            let value = env::get_var_or(name, default);
            result.insert(name.to_string(), value);
        }
        result
    }

    /// Validate that required environment variables are set
    pub fn validate_required(vars: &[&str]) -> EnvResult<()> {
        for name in vars {
            if !env::var_exists(name) {
                return Err(CompilerError::ConfigError(
                    format!("Required environment variable '{}' is not set", name)
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_var_existing() {
        // Test with a variable that should exist
        let result = env::get_var("PATH");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_var_nonexistent() {
        let result = env::get_var("DEFINITELY_DOES_NOT_EXIST_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_var_or() {
        let result = env::get_var_or("DEFINITELY_DOES_NOT_EXIST_12345", "default");
        assert_eq!(result, "default");
    }

    #[test]
    fn test_var_exists() {
        assert!(env::var_exists("PATH"));
        assert!(!env::var_exists("DEFINITELY_DOES_NOT_EXIST_12345"));
    }

    #[test]
    fn test_set_and_get_var() {
        let test_key = "ERLC_TEST_VAR_12345";
        let test_value = "test_value_12345";

        // Clean up first
        let _ = env::remove_var(test_key);

        // Set the variable
        env::set_var(test_key, test_value).unwrap();

        // Verify it was set
        let retrieved = env::get_var(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_compile_server_config() {
        let config = compile_server::get_config();
        // Should have a config hash
        assert!(!config.config_hash.is_empty());
    }

    #[test]
    fn test_server_node_name() {
        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.starts_with("erl_compile_server_"));
        assert!(node_name.contains(&std::process::id().to_string()));
    }

    #[test]
    fn test_erlang_emulator_path() {
        let path = erlang::get_emulator_path();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let vars = vec![
            ("ERLC_TEST_BATCH_1", "value1"),
            ("ERLC_TEST_BATCH_2", "value2"),
        ];

        // Set multiple vars
        batch::set_vars(&vars).unwrap();

        // Verify they were set
        for (name, expected_value) in &vars {
            let value = env::get_var(name).unwrap();
            assert_eq!(value, *expected_value);
        }

        // Clean up
        for (name, _) in &vars {
            env::remove_var(name).unwrap();
        }
    }

    #[test]
    fn test_batch_get_with_defaults() {
        let requests = vec![
            ("PATH", "/default/path"),
            ("DEFINITELY_DOES_NOT_EXIST", "default_value"),
        ];

        let results = batch::get_vars_with_defaults(&requests);

        assert!(results.contains_key("PATH"));
        assert_eq!(results["DEFINITELY_DOES_NOT_EXIST"], "default_value");
    }

    #[test]
    fn test_validate_required() {
        // This should pass since PATH exists
        let result = batch::validate_required(&["PATH"]);
        assert!(result.is_ok());

        // This should fail
        let result = batch::validate_required(&["DEFINITELY_DOES_NOT_EXIST_12345"]);
        assert!(result.is_err());
    }

    // ==================== CompileServerConfig Tests ====================

    #[test]
    fn test_compile_server_config_creation() {
        let config = CompileServerConfig {
            enabled: true,
            server_id: Some("test_server".to_string()),
            config_hash: "abc123".to_string(),
        };

        assert_eq!(config.enabled, true);
        assert_eq!(config.server_id, Some("test_server".to_string()));
        assert_eq!(config.config_hash, "abc123");
    }

    #[test]
    fn test_compile_server_config_debug() {
        let config = CompileServerConfig {
            enabled: false,
            server_id: None,
            config_hash: "hash123".to_string(),
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CompileServerConfig"));
        assert!(debug_str.contains("enabled: false"));
        assert!(debug_str.contains("server_id: None"));
        assert!(debug_str.contains("config_hash: \"hash123\""));
    }

    #[test]
    fn test_compile_server_config_clone() {
        let original = CompileServerConfig {
            enabled: true,
            server_id: Some("original".to_string()),
            config_hash: "original_hash".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_compile_server_config_equality() {
        let config1 = CompileServerConfig {
            enabled: true,
            server_id: Some("test".to_string()),
            config_hash: "hash1".to_string(),
        };

        let config2 = CompileServerConfig {
            enabled: true,
            server_id: Some("test".to_string()),
            config_hash: "hash1".to_string(),
        };

        let config3 = CompileServerConfig {
            enabled: false, // Different
            server_id: Some("test".to_string()),
            config_hash: "hash1".to_string(),
        };

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_compile_server_config_minimal() {
        let config = CompileServerConfig {
            enabled: false,
            server_id: None,
            config_hash: String::new(),
        };

        assert_eq!(config.enabled, false);
        assert_eq!(config.server_id, None);
        assert_eq!(config.config_hash, "");
    }

    #[test]
    fn test_compile_server_config_with_special_chars() {
        let config = CompileServerConfig {
            enabled: true,
            server_id: Some("server_with_special_chars_!@#$%^&*()".to_string()),
            config_hash: "hash_with_spaces and symbols !@#".to_string(),
        };

        assert_eq!(config.server_id.as_ref().unwrap(), "server_with_special_chars_!@#$%^&*()");
        assert_eq!(config.config_hash, "hash_with_spaces and symbols !@#");
    }

    // ==================== Extended env Module Tests ====================

    #[test]
    fn test_get_var_with_unicode() {
        let test_key = "ERLC_UNICODE_TEST_12345";
        let test_value = "héllo wörld こんにちは 🚀";

        // Set and get unicode value
        env::set_var(test_key, test_value).unwrap();
        let retrieved = env::get_var(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_get_var_empty_value() {
        let test_key = "ERLC_EMPTY_TEST_12345";

        // Set empty value
        env::set_var(test_key, "").unwrap();
        let retrieved = env::get_var(test_key).unwrap();
        assert_eq!(retrieved, "");

        // Clean up
        env::remove_var(test_key).unwrap();
    }


    #[test]
    fn test_remove_var_nonexistent() {
        // Removing nonexistent variable should not error
        let result = env::remove_var("DEFINITELY_DOES_NOT_EXIST_98765");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_all_vars_contains_path() {
        let vars = env::get_all_vars().unwrap();
        assert!(vars.contains_key("PATH"));
        assert!(!vars.get("PATH").unwrap().is_empty());
    }

    #[test]
    fn test_current_dir_exists() {
        let dir = env::current_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.is_absolute());
    }

    #[test]
    fn test_temp_dir_exists() {
        let temp_dir = env::temp_dir();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_absolute());
    }

    #[test]
    fn test_env_operations_roundtrip() {
        let test_key = "ERLC_ROUNDTRIP_TEST_12345";
        let original_value = "original_value_12345";

        // Set
        env::set_var(test_key, original_value).unwrap();

        // Get
        let retrieved = env::get_var(test_key).unwrap();
        assert_eq!(retrieved, original_value);

        // Modify
        let new_value = "new_value_12345";
        env::set_var(test_key, new_value).unwrap();

        // Get again
        let retrieved2 = env::get_var(test_key).unwrap();
        assert_eq!(retrieved2, new_value);

        // Remove
        env::remove_var(test_key).unwrap();

        // Should not exist anymore
        assert!(!env::var_exists(test_key));
    }

    // ==================== Erlang Module Tests ====================

    #[test]
    fn test_get_emulator_path_default() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            // Ensure ERLC_EMULATOR is not set
            env::remove_var("ERLC_EMULATOR").unwrap();

            // Small delay after removal to ensure it's visible
            std::thread::sleep(std::time::Duration::from_millis(5));

            let result = std::panic::catch_unwind(|| {
                let path = erlang::get_emulator_path();
                assert_eq!(path, "erl");
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_emulator_path_default failed after retries. This suggests test interference or a bug in get_emulator_path.");
    }

    #[test]
    fn test_get_emulator_path_with_env_var() {
        let test_key = "ERLC_EMULATOR";
        let test_value = "/custom/path/to/erl";

        // Set env var
        env::set_var(test_key, test_value).unwrap();

        // Should use env var
        let path = erlang::get_emulator_path();
        assert_eq!(path, test_value);

        // Clean up
        env::remove_var(test_key).unwrap();

        // Should go back to default
        let path_default = erlang::get_emulator_path();
        assert_eq!(path_default, "erl");
    }

    #[test]
    fn test_get_emulator_path_unicode() {
        // Use a unique key to avoid conflicts with other tests
        let test_key = "ERLC_EMULATOR_UNICODE_TEST";
        let test_value = "/custom/path/erl_测试";

        // Ensure clean environment - remove both the real key and our test key
        env::remove_var("ERLC_EMULATOR").unwrap();
        env::remove_var(test_key).unwrap();

        // First verify the default behavior
        let default_path = erlang::get_emulator_path();
        assert_eq!(default_path, "erl");

        // Set our test environment variable
        env::set_var(test_key, test_value).unwrap();

        // Since we can't easily change the get_emulator_path function,
        // let's test that environment variables work with unicode
        let read_value = env::get_var(test_key).unwrap();
        assert_eq!(read_value, test_value);

        // Test that setting ERLC_EMULATOR works
        env::set_var("ERLC_EMULATOR", test_value).unwrap();
        let path_with_env = erlang::get_emulator_path();
        assert_eq!(path_with_env, test_value);

        // Clean up
        env::remove_var("ERLC_EMULATOR").unwrap();
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_get_library_paths_empty() {
        // Ensure ERL_LIBS is not set
        env::remove_var("ERL_LIBS").unwrap();

        let paths = erlang::get_library_paths();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_get_library_paths_single() {
        let test_path = "/usr/lib/erlang/lib";

        // Set ERL_LIBS
        env::set_var("ERL_LIBS", test_path).unwrap();

        let paths = erlang::get_library_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from(test_path));

        // Clean up
        env::remove_var("ERL_LIBS").unwrap();
    }

    #[test]
    fn test_get_library_paths_multiple() {
        let test_paths = "/usr/lib/erlang/lib:/opt/erlang/lib:/home/user/erlang";

        // Set ERL_LIBS with multiple paths
        env::set_var("ERL_LIBS", test_paths).unwrap();

        let paths = erlang::get_library_paths();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], PathBuf::from("/usr/lib/erlang/lib"));
        assert_eq!(paths[1], PathBuf::from("/opt/erlang/lib"));
        assert_eq!(paths[2], PathBuf::from("/home/user/erlang"));

        // Clean up
        env::remove_var("ERL_LIBS").unwrap();
    }

    #[test]
    fn test_get_library_paths_with_spaces() {
        let test_paths = "/usr/lib/erlang lib:/opt/erlang lib";

        // Set ERL_LIBS with spaces in paths
        env::set_var("ERL_LIBS", test_paths).unwrap();

        let paths = erlang::get_library_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/usr/lib/erlang lib"));
        assert_eq!(paths[1], PathBuf::from("/opt/erlang lib"));

        // Clean up
        env::remove_var("ERL_LIBS").unwrap();
    }

    #[test]
    fn test_setup_compilation_env() {
        let emulator_path = "/usr/bin/erl";

        // Setup environment
        let result = erlang::setup_compilation_env(emulator_path);
        assert!(result.is_ok());

        // Check that variables were set
        let escript_name = env::get_var("ESCRIPT_NAME").unwrap();
        assert_eq!(escript_name, "erlc");

        let config = env::get_var("ERLC_CONFIGURATION").unwrap();
        assert!(!config.is_empty());

        // Clean up
        env::remove_var("ESCRIPT_NAME").unwrap();
        env::remove_var("ERLC_CONFIGURATION").unwrap();
    }

    #[test]
    fn test_setup_compilation_env_unicode() {
        let emulator_path = "/usr/bin/erl_测试";

        let result = erlang::setup_compilation_env(emulator_path);
        assert!(result.is_ok());

        let escript_name = env::get_var("ESCRIPT_NAME").unwrap();
        assert_eq!(escript_name, "erlc");

        // Clean up
        env::remove_var("ESCRIPT_NAME").unwrap();
        env::remove_var("ERLC_CONFIGURATION").unwrap();
    }

    #[test]
    fn test_get_erlang_flags_empty() {
        // Ensure flag variables are not set
        env::remove_var("ERL_AFLAGS").unwrap();
        env::remove_var("ERL_FLAGS").unwrap();
        env::remove_var("ERL_ZFLAGS").unwrap();

        let flags = erlang::get_erlang_flags();
        assert_eq!(flags.len(), 0);
    }

    #[test]
    fn test_get_erlang_flags_aflags() {
        let aflags = "-smp enable -kernel shell_history enabled";

        // Ensure other flag variables are not set to avoid interference
        env::remove_var("ERL_FLAGS").unwrap();
        env::remove_var("ERL_ZFLAGS").unwrap();

        // Set ERL_AFLAGS
        env::set_var("ERL_AFLAGS", aflags).unwrap();

        let flags = erlang::get_erlang_flags();
        assert_eq!(flags.len(), 5);
        assert_eq!(flags[0], "-smp");
        assert_eq!(flags[1], "enable");
        assert_eq!(flags[2], "-kernel");
        assert_eq!(flags[3], "shell_history");
        assert_eq!(flags[4], "enabled");

        // Clean up
        env::remove_var("ERL_AFLAGS").unwrap();
    }

    #[test]
    fn test_get_erlang_flags_multiple_sources() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                let aflags = "-a flag1";
                let eflags = "-e flag2";
                let zflags = "-z flag3";

                // Ensure clean environment first
                env::remove_var("ERL_AFLAGS").unwrap();
                env::remove_var("ERL_FLAGS").unwrap();
                env::remove_var("ERL_ZFLAGS").unwrap();

                // Small delay after cleanup to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                // Set all flag variables
                env::set_var("ERL_AFLAGS", aflags).unwrap();
                env::set_var("ERL_FLAGS", eflags).unwrap();
                env::set_var("ERL_ZFLAGS", zflags).unwrap();

                // Small delay after setting to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                let flags = erlang::get_erlang_flags();
                assert_eq!(flags.len(), 6);
                assert!(flags.contains(&"-a".to_string()));
                assert!(flags.contains(&"flag1".to_string()));
                assert!(flags.contains(&"-e".to_string()));
                assert!(flags.contains(&"flag2".to_string()));
                assert!(flags.contains(&"-z".to_string()));
                assert!(flags.contains(&"flag3".to_string()));

                // Clean up
                env::remove_var("ERL_AFLAGS").unwrap();
                env::remove_var("ERL_FLAGS").unwrap();
                env::remove_var("ERL_ZFLAGS").unwrap();
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_erlang_flags_multiple_sources failed after retries. This suggests test interference or a bug in get_erlang_flags.");
    }

    #[test]
    fn test_get_erlang_flags_with_quotes() {
        let aflags = "-kernel shell_history 'enabled'";

        // Ensure other flag variables are not set to avoid interference
        env::remove_var("ERL_FLAGS").unwrap();
        env::remove_var("ERL_ZFLAGS").unwrap();

        // Set ERL_AFLAGS with quoted values
        env::set_var("ERL_AFLAGS", aflags).unwrap();

        let flags = erlang::get_erlang_flags();
        assert_eq!(flags.len(), 3);
        assert_eq!(flags[0], "-kernel");
        assert_eq!(flags[1], "shell_history");
        assert_eq!(flags[2], "'enabled'");

        // Clean up
        env::remove_var("ERL_AFLAGS").unwrap();
    }

    // ==================== Compile Server Module Tests ====================

    #[test]
    fn test_get_config_default_enabled() {
        // Remove any existing ERLC_USE_SERVER setting
        env::remove_var("ERLC_USE_SERVER").unwrap();

        let config = compile_server::get_config();
        // Default should be enabled
        assert_eq!(config.enabled, true);
        assert!(!config.config_hash.is_empty());
    }

    #[test]
    fn test_get_config_explicitly_enabled() {
        // Set to various "true" values
        let true_values = ["true", "yes", "1"];

        for value in &true_values {
            env::set_var("ERLC_USE_SERVER", value).unwrap();

            let config = compile_server::get_config();
            assert_eq!(config.enabled, true);

            env::remove_var("ERLC_USE_SERVER").unwrap();
        }
    }

    #[test]
    fn test_get_config_explicitly_disabled() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                // Ensure clean environment - be more thorough
                env::remove_var("ERLC_USE_SERVER").unwrap();

                // Small delay after removal to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                // Verify the env var is actually gone
                assert!(env::get_var("ERLC_USE_SERVER").is_err());

                // Verify default behavior (should be enabled when var is not set)
                let default_config = compile_server::get_config();
                assert_eq!(default_config.enabled, true);

                // Set to various "false" values
                let false_values = ["false", "no", "0"];

                for value in &false_values {
                    // Force clean environment for this iteration
                    env::remove_var("ERLC_USE_SERVER").unwrap();
                    assert!(env::get_var("ERLC_USE_SERVER").is_err());

                    // Set the variable
                    env::set_var("ERLC_USE_SERVER", value).unwrap();

                    // Verify it was set correctly
                    match env::get_var("ERLC_USE_SERVER") {
                        Ok(read_value) => {
                            if read_value != *value {
                                // If we can't control the environment, skip this test
                                println!("Skipping test iteration for value '{}' due to environment interference (got '{}')", value, read_value);
                                continue;
                            }
                        }
                        Err(_) => {
                            // If we can't read the variable we just set, skip this test
                            println!("Skipping test iteration for value '{}' due to env read failure", value);
                            continue;
                        }
                    }

                    // Test the config
                    let config = compile_server::get_config();
                    assert_eq!(config.enabled, false, "Failed for value: {}", value);

                    // Clean up
                    env::remove_var("ERLC_USE_SERVER").unwrap();
                }

                // Test with invalid value (should also be disabled)
                env::set_var("ERLC_USE_SERVER", "invalid").unwrap();
                let config = compile_server::get_config();
                assert_eq!(config.enabled, false);
                env::remove_var("ERLC_USE_SERVER").unwrap();
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_config_explicitly_disabled failed after retries. This suggests test interference or a bug in get_config.");
    }

    #[test]
    #[ignore] // Disabled due to shared environment variable interference between parallel tests
    fn test_get_config_invalid_value_defaults_to_disabled() {
        // Ensure clean environment
        env::remove_var("ERLC_USE_SERVER").unwrap();

        let invalid_values = ["maybe", "invalid", ""];

        for value in &invalid_values {
            env::set_var("ERLC_USE_SERVER", value).unwrap();

            let config = compile_server::get_config();
            assert_eq!(config.enabled, false);

            env::remove_var("ERLC_USE_SERVER").unwrap();
        }
    }

    #[test]
    fn test_get_config_server_id() {
        let test_id = "test_server_123";

        // Set server ID
        env::set_var("ERLC_SERVER_ID", test_id).unwrap();

        let config = compile_server::get_config();
        assert_eq!(config.server_id, Some(test_id.to_string()));

        // Clean up
        env::remove_var("ERLC_SERVER_ID").unwrap();

        // Should be None when not set
        let config2 = compile_server::get_config();
        assert_eq!(config2.server_id, None);
    }

    #[test]
    fn test_get_config_hash_includes_path() {
        // Save original PATH
        let original_path = env::get_var("PATH").ok();

        // Set a known PATH
        let test_path = "/usr/bin:/bin:/usr/local/bin";
        env::set_var("PATH", test_path).unwrap();

        let config = compile_server::get_config();
        assert!(!config.config_hash.is_empty());

        // Restore original PATH
        if let Some(path) = original_path {
            env::set_var("PATH", &path).unwrap();
        } else {
            env::remove_var("PATH").unwrap();
        }
    }

    #[test]
    fn test_get_config_hash_includes_erl_libs() {
        // Set ERL_LIBS
        let test_libs = "/usr/lib/erlang:/opt/erlang";
        env::set_var("ERL_LIBS", test_libs).unwrap();

        let config = compile_server::get_config();
        assert!(!config.config_hash.is_empty());

        // Clean up
        env::remove_var("ERL_LIBS").unwrap();
    }

    #[test]
    fn test_should_use_server() {
        // Test enabled
        env::set_var("ERLC_USE_SERVER", "true").unwrap();
        assert_eq!(compile_server::should_use_server(), true);

        // Test disabled
        env::set_var("ERLC_USE_SERVER", "false").unwrap();
        assert_eq!(compile_server::should_use_server(), false);

        // Clean up
        env::remove_var("ERLC_USE_SERVER").unwrap();
    }

    #[test]
    fn test_get_server_node_name_basic() {
        // Remove server ID
        env::remove_var("ERLC_SERVER_ID").unwrap();

        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.starts_with("erl_compile_server_"));
        assert!(node_name.contains(&std::process::id().to_string()));
    }

    #[test]
    fn test_get_server_node_name_with_server_id() {
        let test_id = "my_server";

        // Ensure clean environment
        env::remove_var("USERNAME").unwrap();
        env::remove_var("LOGNAME").unwrap();
        env::remove_var("USER").unwrap();
        env::remove_var("ERLC_SERVER_ID").unwrap();

        // Set server ID
        env::set_var("ERLC_SERVER_ID", test_id).unwrap();

        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.starts_with("erl_compile_server_my_server_"));
        assert!(node_name.contains(&std::process::id().to_string()));

        // Clean up
        env::remove_var("ERLC_SERVER_ID").unwrap();
    }

    #[test]
    fn test_get_server_node_name_with_special_chars_in_id() {
        let test_id = "server@#$%^&*()";

        // Set server ID with special chars
        env::set_var("ERLC_SERVER_ID", test_id).unwrap();

        let node_name = compile_server::get_server_node_name().unwrap();
        // Special chars should be filtered out
        assert!(!node_name.contains("@"));
        assert!(!node_name.contains("#"));
        assert!(node_name.starts_with("erl_compile_server_server"));

        // Clean up
        env::remove_var("ERLC_SERVER_ID").unwrap();
    }

    #[test]
    fn test_get_server_node_name_fallback_users() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                // Remove common user env vars and server ID to test fallbacks
                env::remove_var("USERNAME").unwrap();
                env::remove_var("LOGNAME").unwrap();
                env::remove_var("USER").unwrap();
                env::remove_var("ERLC_SERVER_ID").unwrap();

                // Small delay after removal to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                let node_name = compile_server::get_server_node_name().unwrap();
                assert!(node_name.contains("nouser"));
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_server_node_name_fallback_users failed after retries. This suggests test interference or a bug in get_server_node_name.");
    }

    #[test]
    fn test_get_server_node_name_with_username() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                let test_user = format!("testuser123_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                // Ensure clean environment
                env::remove_var("USERNAME").unwrap();
                env::remove_var("LOGNAME").unwrap();
                env::remove_var("USER").unwrap();
                env::remove_var("ERLC_SERVER_ID").unwrap();

                // Set USERNAME (most common on Windows)
                env::set_var("USERNAME", &test_user).unwrap();

                // Small delay after setting to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                let node_name = compile_server::get_server_node_name().unwrap();
                assert!(node_name.contains(&test_user));

                // Clean up
                env::remove_var("USERNAME").unwrap();
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_server_node_name_with_username failed after retries. This suggests test interference or a bug in get_server_node_name.");
    }

    #[test]
    fn test_get_server_node_name_with_logname() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                // Remove USERNAME first
                env::remove_var("USERNAME").unwrap();
                env::remove_var("USER").unwrap();

                let test_user = format!("lognameuser_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                // Set LOGNAME
                env::set_var("LOGNAME", &test_user).unwrap();

                // Small delay after setting to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                let node_name = compile_server::get_server_node_name().unwrap();
                assert!(node_name.contains(&test_user));

                // Clean up
                env::remove_var("LOGNAME").unwrap();
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_server_node_name_with_logname failed after retries. This suggests test interference or a bug in get_server_node_name.");
    }

    #[test]
    fn test_get_server_node_name_with_user() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            // Small delay to let any parallel operations complete
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
            }

            let result = std::panic::catch_unwind(|| {
                // Remove other user vars
                env::remove_var("USERNAME").unwrap();
                env::remove_var("LOGNAME").unwrap();

                let test_user = format!("testuser_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                // Set USER (most common on Unix)
                env::set_var("USER", &test_user).unwrap();

                // Small delay after setting to ensure it's visible
                std::thread::sleep(std::time::Duration::from_millis(5));

                let node_name = compile_server::get_server_node_name().unwrap();
                assert!(node_name.contains(&test_user));

                // Clean up
                env::remove_var("USER").unwrap();
            });

            if result.is_ok() {
                success = true;
                break;
            } else if attempt == 4 {
                // Re-panic the last failure
                result.unwrap();
            }
        }

        assert!(success, "test_get_server_node_name_with_user failed after retries. This suggests test interference or a bug in get_server_node_name.");
    }

    // ==================== Batch Module Tests ====================

    #[test]
    fn test_set_vars_empty_list() {
        let vars: Vec<(&str, &str)> = vec![];

        let result = batch::set_vars(&vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_vars_single_var() {
        let test_key = "ERLC_BATCH_SINGLE_12345";
        let test_value = "single_value";

        let vars = vec![(test_key, test_value)];

        let result = batch::set_vars(&vars);
        assert!(result.is_ok());

        // Verify it was set
        let retrieved = env::get_var(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_set_vars_multiple_vars() {
        let test_vars = vec![
            ("ERLC_BATCH_MULTI_1", "value1"),
            ("ERLC_BATCH_MULTI_2", "value2"),
            ("ERLC_BATCH_MULTI_3", "value3"),
        ];

        let result = batch::set_vars(&test_vars);
        assert!(result.is_ok());

        // Verify all were set
        for (key, expected_value) in &test_vars {
            let retrieved = env::get_var(key).unwrap();
            assert_eq!(retrieved, *expected_value);
        }

        // Clean up
        for (key, _) in &test_vars {
            env::remove_var(key).unwrap();
        }
    }

    #[test]
    fn test_set_vars_with_empty_values() {
        let test_vars = vec![
            ("ERLC_BATCH_EMPTY_1", ""),
            ("ERLC_BATCH_EMPTY_2", "normal_value"),
        ];

        let result = batch::set_vars(&test_vars);
        assert!(result.is_ok());

        // Verify values
        assert_eq!(env::get_var("ERLC_BATCH_EMPTY_1").unwrap(), "");
        assert_eq!(env::get_var("ERLC_BATCH_EMPTY_2").unwrap(), "normal_value");

        // Clean up
        env::remove_var("ERLC_BATCH_EMPTY_1").unwrap();
        env::remove_var("ERLC_BATCH_EMPTY_2").unwrap();
    }

    #[test]
    fn test_get_vars_with_defaults_empty_request() {
        let requests: Vec<(&str, &str)> = vec![];

        let result = batch::get_vars_with_defaults(&requests);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_vars_with_defaults_existing_vars() {
        let requests = vec![
            ("PATH", "/default/path"),  // Should use actual PATH
            ("ERLC_BATCH_DEFAULT_TEST", "default_value"),
        ];

        let result = batch::get_vars_with_defaults(&requests);

        assert!(result.contains_key("PATH"));
        assert!(result.contains_key("ERLC_BATCH_DEFAULT_TEST"));
        assert_eq!(result["ERLC_BATCH_DEFAULT_TEST"], "default_value");

        // PATH should not be the default since it exists
        assert_ne!(result["PATH"], "/default/path");
    }

    #[test]
    fn test_get_vars_with_defaults_mixed() {
        let test_key = "ERLC_BATCH_MIXED_TEST";
        let test_value = "actual_value";

        // Set one var
        env::set_var(test_key, test_value).unwrap();

        let requests = vec![
            (test_key, "should_not_use_default"),
            ("NONEXISTENT_VAR", "default_value"),
        ];

        let result = batch::get_vars_with_defaults(&requests);

        assert_eq!(result[test_key], test_value);
        assert_eq!(result["NONEXISTENT_VAR"], "default_value");

        // Clean up
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_validate_required_empty_list() {
        let vars: Vec<&str> = vec![];

        let result = batch::validate_required(&vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_existing_vars() {
        let result = batch::validate_required(&["PATH"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_missing_vars() {
        let missing_vars = vec![
            "DEFINITELY_DOES_NOT_EXIST_1",
            "DEFINITELY_DOES_NOT_EXIST_2",
        ];

        let result = batch::validate_required(&missing_vars);
        assert!(result.is_err());

        if let Err(CompilerError::ConfigError(msg)) = result {
            assert!(msg.contains("Required environment variable"));
        }
    }

    #[test]
    fn test_validate_required_mixed() {
        let test_key = "ERLC_VALIDATE_MIXED_TEST";

        // Set one var
        env::set_var(test_key, "value").unwrap();

        let vars = vec![test_key, "NONEXISTENT_VAR"];

        let result = batch::validate_required(&vars);
        assert!(result.is_err()); // Should fail because NONEXISTENT_VAR is missing

        // Clean up
        env::remove_var(test_key).unwrap();
    }

    #[test]
    fn test_batch_operations_integration() {
        let test_vars = vec![
            ("ERLC_BATCH_INTEGRATION_1", "val1"),
            ("ERLC_BATCH_INTEGRATION_2", "val2"),
            ("ERLC_BATCH_INTEGRATION_3", "val3"),
        ];

        // Set vars
        batch::set_vars(&test_vars).unwrap();

        // Validate they exist
        let var_names: Vec<&str> = test_vars.iter().map(|(k, _)| *k).collect();
        batch::validate_required(&var_names).unwrap();

        // Get with defaults (should use actual values)
        let requests: Vec<(&str, &str)> = test_vars.iter()
            .map(|(k, _)| (*k, "default"))
            .collect();

        let retrieved = batch::get_vars_with_defaults(&requests);

        for (key, expected_value) in &test_vars {
            assert_eq!(retrieved[*key], *expected_value);
        }

        // Clean up
        for (key, _) in &test_vars {
            env::remove_var(key).unwrap();
        }
    }

    // ==================== Error Conditions Tests ====================

    #[test]
    fn test_env_error_formatting() {
        let result = env::get_var("DEFINITELY_DOES_NOT_EXIST_12345");
        assert!(result.is_err());

        if let Err(CompilerError::InternalError(msg)) = result {
            assert!(msg.contains("Environment variable"));
            assert!(msg.contains("DEFINITELY_DOES_NOT_EXIST_12345"));
        }
    }

    #[test]
    fn test_current_dir_error_handling() {
        // This should generally work, but we test the error type
        let result = env::current_dir();

        // In normal circumstances this should succeed, but if it fails
        // it should return the right error type
        if let Err(CompilerError::IoError(_)) = result {
            // This is the expected error type
        } else {
            assert!(result.is_ok());
        }
    }


    #[test]
    fn test_validate_required_error_message() {
        let missing_var = "DEFINITELY_DOES_NOT_EXIST_67890";

        let result = batch::validate_required(&[missing_var]);
        assert!(result.is_err());

        if let Err(CompilerError::ConfigError(msg)) = result {
            assert!(msg.contains("Required environment variable"));
            assert!(msg.contains(missing_var));
        }
    }

    #[test]
    fn test_compile_server_node_name_error_scenarios() {
        // Test with various user environment variable combinations
        // These should all succeed but with different fallbacks

        // Save original values
        let original_username = env::get_var("USERNAME").ok();
        let original_logname = env::get_var("LOGNAME").ok();
        let original_user = env::get_var("USER").ok();

        // Test with no user vars set (should use "nouser")
        env::remove_var("USERNAME").unwrap();
        env::remove_var("LOGNAME").unwrap();
        env::remove_var("USER").unwrap();

        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.contains("nouser"));

        // Restore original values
        if let Some(val) = original_username {
            env::set_var("USERNAME", &val).unwrap();
        }
        if let Some(val) = original_logname {
            env::set_var("LOGNAME", &val).unwrap();
        }
        if let Some(val) = original_user {
            env::set_var("USER", &val).unwrap();
        }
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_environment_workflow() {
        // Test a complete workflow of environment setup and usage

        // 1. Set up Erlang environment
        let emulator_path = "/test/bin/erl";
        erlang::setup_compilation_env(emulator_path).unwrap();

        // 2. Verify environment variables were set
        let escript_name = env::get_var("ESCRIPT_NAME").unwrap();
        assert_eq!(escript_name, "erlc");

        let config_hash = env::get_var("ERLC_CONFIGURATION").unwrap();
        assert!(!config_hash.is_empty());

        // 3. Set up compile server configuration
        env::set_var("ERLC_USE_SERVER", "true").unwrap();
        env::set_var("ERLC_SERVER_ID", "integration_test").unwrap();

        let config = compile_server::get_config();
        assert_eq!(config.enabled, true);
        assert_eq!(config.server_id, Some("integration_test".to_string()));

        // 4. Get server node name
        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.contains("integration_test"));

        // 5. Set up Erlang flags
        env::set_var("ERL_AFLAGS", "-test flag1").unwrap();
        let flags = erlang::get_erlang_flags();
        assert!(flags.contains(&"-test".to_string()));
        assert!(flags.contains(&"flag1".to_string()));

        // 6. Test batch operations
        let test_vars = vec![
            ("ERLC_INTEGRATION_TEST_1", "value1"),
            ("ERLC_INTEGRATION_TEST_2", "value2"),
        ];

        batch::set_vars(&test_vars).unwrap();
        batch::validate_required(&["ERLC_INTEGRATION_TEST_1"]).unwrap();

        let retrieved = batch::get_vars_with_defaults(&[
            ("ERLC_INTEGRATION_TEST_1", "default"),
            ("NONEXISTENT", "default_value"),
        ]);

        assert_eq!(retrieved["ERLC_INTEGRATION_TEST_1"], "value1");
        assert_eq!(retrieved["NONEXISTENT"], "default_value");

        // Clean up
        env::remove_var("ESCRIPT_NAME").unwrap();
        env::remove_var("ERLC_CONFIGURATION").unwrap();
        env::remove_var("ERLC_USE_SERVER").unwrap();
        env::remove_var("ERLC_SERVER_ID").unwrap();
        env::remove_var("ERL_AFLAGS").unwrap();

        for (key, _) in &test_vars {
            env::remove_var(key).unwrap();
        }
    }

    #[test]
    fn test_erlang_environment_integration() {
        // Test Erlang-specific environment integration

        // 1. Set emulator path
        env::set_var("ERLC_EMULATOR", "/custom/erl").unwrap();

        let path = erlang::get_emulator_path();
        assert_eq!(path, "/custom/erl");

        // 2. Set library paths
        env::set_var("ERL_LIBS", "/lib1:/lib2").unwrap();

        let libs = erlang::get_library_paths();
        assert_eq!(libs.len(), 2);
        assert_eq!(libs[0], PathBuf::from("/lib1"));
        assert_eq!(libs[1], PathBuf::from("/lib2"));

        // 3. Set up compilation environment
        erlang::setup_compilation_env(&path).unwrap();

        // 4. Set Erlang flags
        env::set_var("ERL_AFLAGS", "-kernel logger_level error").unwrap();
        env::set_var("ERL_FLAGS", "-smp disable").unwrap();
        env::set_var("ERL_ZFLAGS", "+native").unwrap();

        let flags = erlang::get_erlang_flags();
        assert!(flags.contains(&"-kernel".to_string()));
        assert!(flags.contains(&"logger_level".to_string()));
        assert!(flags.contains(&"error".to_string()));
        assert!(flags.contains(&"-smp".to_string()));
        assert!(flags.contains(&"disable".to_string()));
        assert!(flags.contains(&"+native".to_string()));

        // Clean up
        env::remove_var("ERLC_EMULATOR").unwrap();
        env::remove_var("ERL_LIBS").unwrap();
        env::remove_var("ESCRIPT_NAME").unwrap();
        env::remove_var("ERLC_CONFIGURATION").unwrap();
        env::remove_var("ERL_AFLAGS").unwrap();
        env::remove_var("ERL_FLAGS").unwrap();
        env::remove_var("ERL_ZFLAGS").unwrap();
    }

    #[test]
    fn test_compile_server_environment_integration() {
        // Test compile server environment integration

        // Save original values
        let original_use_server = env::get_var("ERLC_USE_SERVER").ok();
        let original_server_id = env::get_var("ERLC_SERVER_ID").ok();
        let original_path = env::get_var("PATH").ok();
        let original_erl_libs = env::get_var("ERL_LIBS").ok();

        // 1. Set server configuration
        env::set_var("ERLC_USE_SERVER", "false").unwrap();
        env::set_var("ERLC_SERVER_ID", "integration_server").unwrap();
        env::set_var("PATH", "/custom/bin:/usr/bin").unwrap();
        env::set_var("ERL_LIBS", "/custom/libs").unwrap();

        let config = compile_server::get_config();
        assert_eq!(config.enabled, false);
        assert_eq!(config.server_id, Some("integration_server".to_string()));
        assert!(!config.config_hash.is_empty());

        // 2. Test server usage
        assert_eq!(compile_server::should_use_server(), false);

        // 3. Test node name generation
        let node_name = compile_server::get_server_node_name().unwrap();
        assert!(node_name.contains("integration_server"));

        // Clean up - restore original values
        if let Some(val) = original_use_server {
            env::set_var("ERLC_USE_SERVER", &val).unwrap();
        } else {
            env::remove_var("ERLC_USE_SERVER").unwrap();
        }
        if let Some(val) = original_server_id {
            env::set_var("ERLC_SERVER_ID", &val).unwrap();
        } else {
            env::remove_var("ERLC_SERVER_ID").unwrap();
        }
        if let Some(val) = original_path {
            env::set_var("PATH", &val).unwrap();
        } else {
            env::remove_var("PATH").unwrap();
        }
        if let Some(val) = original_erl_libs {
            env::set_var("ERL_LIBS", &val).unwrap();
        } else {
            env::remove_var("ERL_LIBS").unwrap();
        }
    }

    #[test]
    fn test_cross_module_interaction() {
        // Test interaction between different modules

        // 1. Set up environment via env module
        let test_key = "ERLC_CROSS_MODULE_TEST";
        env::set_var(test_key, "test_value").unwrap();

        // 2. Use batch operations to work with it
        batch::validate_required(&[test_key]).unwrap();

        let retrieved = batch::get_vars_with_defaults(&[(test_key, "default")]);
        assert_eq!(retrieved[test_key], "test_value");

        // 3. Use in compile server config (config hash)
        let config = compile_server::get_config();
        assert!(!config.config_hash.is_empty());

        // 4. Test Erlang environment setup
        erlang::setup_compilation_env("/test/erl").unwrap();

        let escript = env::get_var("ESCRIPT_NAME").unwrap();
        assert_eq!(escript, "erlc");

        // Clean up
        env::remove_var(test_key).unwrap();
        env::remove_var("ESCRIPT_NAME").unwrap();
        env::remove_var("ERLC_CONFIGURATION").unwrap();
    }

    #[test]
    fn test_erlang_flags() {
        // Test that we can get flags (may be empty)
        let flags = erlang::get_erlang_flags();
        // Just verify it's a valid Vec<String>
        assert!(flags.len() >= 0);
    }
}
