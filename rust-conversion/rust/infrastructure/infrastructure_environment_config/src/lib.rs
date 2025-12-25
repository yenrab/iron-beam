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

    #[test]
    fn test_erlang_flags() {
        // Test that we can get flags (may be empty)
        let flags = erlang::get_erlang_flags();
        // Just verify it's a valid Vec<String>
        assert!(flags.len() >= 0);
    }
}
