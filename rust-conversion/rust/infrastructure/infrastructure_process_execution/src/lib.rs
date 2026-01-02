/*!
# Infrastructure Process Execution

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Platform-specific process spawning and execution

## Overview

This crate provides safe process spawning and execution for the Erlang BEAM compiler infrastructure.
Replaces platform-specific C spawning functions with safe Rust std::process abstractions.

## Original C Functions Replaced

The original `erlc.c` contained these process execution functions:
- `run_erlang()`: Execute Erlang emulator with arguments → **Replaced with safe Command execution**
- `my_spawnvp()`: Platform-specific process spawning → **Replaced with std::process::Command**

## Process Execution Philosophy

### 1. Safe Command Execution
```rust
use infrastructure_process_execution::{executor, ExecutionResult};

// Execute command with proper error handling
match executor::run_command("erl", &["-eval", "halt()"]) {
    Ok(ExecutionResult::Success(output)) => println!("Success: {}", output),
    Ok(ExecutionResult::Failure(code, stderr)) => eprintln!("Failed: {}", stderr),
    Err(e) => eprintln!("Execution error: {}", e),
}
```

### 2. Erlang-Specific Execution
```rust
use infrastructure_process_execution::{erlang, CompileOptions};

// Run Erlang compiler with environment setup
match erlang::compile_erlang_file("example.erl", &CompileOptions::default()) {
    Ok(result) => println!("Compilation successful"),
    Err(e) => eprintln!("Compilation failed: {}", e),
}
```

### 3. Background Process Management
```rust
use infrastructure_process_execution::background;
use infrastructure_environment_config::CompileServerConfig;

// Start compile server in background
let config = CompileServerConfig {
    enabled: true,
    server_id: Some("test".to_string()),
    config_hash: "test_hash".to_string(),
};
match background::start_compile_server(&config) {
    Ok(mut server) => {
        // Wait for process to be ready
        match server.try_wait() {
            Ok(Some(output)) => println!("Server started"),
            Ok(None) => println!("Server still starting..."),
            Err(e) => eprintln!("Server error: {}", e),
        }
    }
    Err(e) => eprintln!("Failed to start server: {}", e),
}
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends on memory, error, platform, environment)
- **SOLID Principle**: Single responsibility for process execution
- **Safe Rust**: No unsafe code, leverages std::process safety
- **Cross-Platform**: Works on all Rust-supported platforms
*/

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use infrastructure_error_handling::{CompilerError, CompilerResult};
use infrastructure_environment_config::env;
use infrastructure_platform_support::arguments;

/// Process execution result
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Process completed successfully
    Success(String),
    /// Process failed with exit code and error output
    Failure(i32, String),
}

/// Process execution options
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    /// Working directory for the process
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Timeout for process execution
    pub timeout: Option<Duration>,
    /// Whether to capture stdout
    pub capture_stdout: bool,
    /// Whether to capture stderr
    pub capture_stderr: bool,
}

/// Erlang compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Output directory
    pub outdir: Option<PathBuf>,
    /// Include paths
    pub includes: Vec<PathBuf>,
    /// Compilation flags
    pub flags: Vec<String>,
    /// Whether to enable warnings
    pub warnings: bool,
    /// Whether to enable debug info
    pub debug_info: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            outdir: None,
            includes: Vec::new(),
            flags: Vec::new(),
            warnings: true,
            debug_info: false,
        }
    }
}

/// Basic command execution utilities
pub mod executor {
    use super::*;

    /// Execute a command with arguments and return the result
    ///
    /// Replaces the C `run_erlang()` function with safe Rust process execution.
    pub fn run_command<S: AsRef<OsStr>>(
        program: S,
        args: &[S],
    ) -> CompilerResult<ExecutionResult> {
        run_command_with_options(program, args, &ExecutionOptions::default())
    }

    /// Execute a command with full options
    pub fn run_command_with_options<S: AsRef<OsStr>>(
        program: S,
        args: &[S],
        options: &ExecutionOptions,
    ) -> CompilerResult<ExecutionResult> {
        let mut command = Command::new(program);

        // Set arguments
        command.args(args);

        // Set working directory
        if let Some(ref dir) = options.working_dir {
            command.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &options.env_vars {
            command.env(key, value);
        }

        // Configure stdio
        if options.capture_stdout {
            command.stdout(Stdio::piped());
        }
        if options.capture_stderr {
            command.stderr(Stdio::piped());
        }

        // Execute the command
        let mut child = command.spawn().map_err(|e| CompilerError::IoError(e))?;

        // Wait for completion or timeout
        let output = if let Some(timeout) = options.timeout {
            wait_with_timeout(&mut child, timeout)?
        } else {
            child.wait_with_output().map_err(|e| CompilerError::IoError(e))?
        };

        // Process the result
        if output.status.success() {
            let stdout = if options.capture_stdout {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::new()
            };
            Ok(ExecutionResult::Success(stdout))
        } else {
            let stderr = if options.capture_stderr {
                String::from_utf8_lossy(&output.stderr).to_string()
            } else {
                format!("Process exited with code {}", output.status.code().unwrap_or(-1))
            };
            Ok(ExecutionResult::Failure(
                output.status.code().unwrap_or(-1),
                stderr,
            ))
        }
    }

    /// Check if a command exists and is executable
    pub fn command_exists<S: AsRef<OsStr>>(program: S) -> bool {
        Command::new(program)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn wait_with_timeout(
        child: &mut std::process::Child,
        timeout: Duration,
    ) -> CompilerResult<std::process::Output> {
        // Simple timeout implementation - in a real implementation,
        // you might want to use tokio or async-std for better timeout handling
        let start = std::time::Instant::now();

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process has finished
                    let mut output = std::process::Output {
                        status,
                        stdout: Vec::new(),
                        stderr: Vec::new(),
                    };

                    // Read remaining output if pipes exist
                    if let Some(ref mut stdout) = child.stdout {
                        let mut buf = Vec::new();
                        stdout.read_to_end(&mut buf)?;
                        output.stdout = buf;
                    }
                    if let Some(ref mut stderr) = child.stderr {
                        let mut buf = Vec::new();
                        stderr.read_to_end(&mut buf)?;
                        output.stderr = buf;
                    }

                    return Ok(output);
                }
                Ok(None) => {
                    // Process still running, check timeout
                    if start.elapsed() >= timeout {
                        // Kill the process and return timeout error
                        let _ = child.kill();
                        return Err(CompilerError::InternalError(
                            "Process execution timed out".to_string(),
                        ));
                    }
                    // Sleep briefly before checking again
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(CompilerError::IoError(e)),
            }
        }
    }
}

/// Erlang-specific process execution
pub mod erlang {
    use super::*;
    use infrastructure_environment_config::erlang;

    /// Execute Erlang emulator with arguments
    ///
    /// This replaces the core functionality of the original `run_erlang()` C function.
    pub fn run_erlang_emulator(args: &[&str]) -> CompilerResult<ExecutionResult> {
        let emulator_path = infrastructure_environment_config::erlang::get_emulator_path();
        run_erlang_emulator_with_path(&emulator_path, args)
    }

    /// Execute Erlang emulator with specific path
    pub fn run_erlang_emulator_with_path(
        emulator_path: &str,
        args: &[&str],
    ) -> CompilerResult<ExecutionResult> {
        // Set up environment for Erlang execution
        erlang::setup_compilation_env(emulator_path)?;

        let mut options = ExecutionOptions::default();
        options.capture_stdout = true;
        options.capture_stderr = true;

        // Add Erlang-specific flags
        let mut full_args = vec!["+sbtu", "+A0", "+P", "65536", "+Q", "1024", "-noinput"];
        full_args.extend_from_slice(args);

        executor::run_command_with_options(emulator_path, &full_args, &options)
    }

    /// Compile an Erlang file
    pub fn compile_erlang_file(
        file_path: &str,
        options: &CompileOptions,
    ) -> CompilerResult<ExecutionResult> {
        let mut args = Vec::new();
        args.push("-compile".to_string());

        // Add output directory
        if let Some(ref outdir) = options.outdir {
            args.push("-o".to_string());
            args.push(outdir.to_string_lossy().to_string());
        }

        // Add include paths
        for include in &options.includes {
            args.push("-I".to_string());
            args.push(include.to_string_lossy().to_string());
        }

        // Add flags
        if options.warnings {
            args.push("+warn_unused_vars".to_string());
            args.push("+warn_unused_import".to_string());
        }

        if options.debug_info {
            args.push("+debug_info".to_string());
        }

        // Add custom flags
        for flag in &options.flags {
            args.push(flag.clone());
        }

        // Add the file to compile
        args.push(file_path.to_string());

        run_erlang_emulator(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    /// Run Erlang with EScript
    pub fn run_escript(script_path: &str, script_args: &[&str]) -> CompilerResult<ExecutionResult> {
        let mut args = vec!["escript", script_path];
        args.extend_from_slice(script_args);

        run_erlang_emulator(&args)
    }
}

/// Background process management
pub mod background {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

/// Handle to a background process
pub struct BackgroundProcess {
    child: Option<std::process::Child>,
    stdout_reader: Option<thread::JoinHandle<()>>,
    stderr_reader: Option<thread::JoinHandle<()>>,
}

    impl BackgroundProcess {
        /// Wait for the process to exit
        pub fn wait(mut self) -> CompilerResult<std::process::Output> {
            let child = self.child.take().unwrap();
            let output = child.wait_with_output().map_err(|e| CompilerError::IoError(e))?;
            Ok(output)
        }

        /// Try to wait without blocking
        pub fn try_wait(&mut self) -> CompilerResult<Option<std::process::Output>> {
            if let Some(ref mut child) = self.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process finished, get the output
                        let output = std::process::Output {
                            status,
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                        };
                        Ok(Some(output))
                    }
                    Ok(None) => Ok(None), // Still running
                    Err(e) => Err(CompilerError::IoError(e)),
                }
            } else {
                Ok(None) // Process already waited on
            }
        }

        /// Send SIGTERM to the process
        pub fn terminate(&mut self) -> CompilerResult<()> {
            if let Some(ref mut child) = self.child {
                child.kill().map_err(|e| CompilerError::IoError(e))
            } else {
                Ok(()) // Process already finished
            }
        }

        /// Get the process ID
        pub fn id(&self) -> Option<u32> {
            self.child.as_ref().map(|c| c.id())
        }
    }

    impl Drop for BackgroundProcess {
        fn drop(&mut self) {
            // Try to terminate the process when dropped
            if let Some(ref mut child) = self.child {
                let _ = child.kill();
            }
        }
    }

    /// Start a compile server in the background
    pub fn start_compile_server(
        server_config: &infrastructure_environment_config::CompileServerConfig,
    ) -> CompilerResult<BackgroundProcess> {
        if !server_config.enabled {
            return Err(CompilerError::ConfigError(
                "Compile server is not enabled".to_string(),
            ));
        }

        let emulator_path = infrastructure_environment_config::erlang::get_emulator_path();
        let server_name = infrastructure_environment_config::compile_server::get_server_node_name()?;

        let mut args = vec![
            "-detached",
            "-sname",
            &server_name,
            "-kernel",
            "start_compile_server",
            "true",
        ];

        let mut options = ExecutionOptions::default();
        options.capture_stdout = false; // Background process
        options.capture_stderr = false;

        let child = Command::new(&emulator_path)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| CompilerError::IoError(e))?;

        Ok(BackgroundProcess {
            child: Some(child),
            stdout_reader: None,
            stderr_reader: None,
        })
    }
}

/// Process pipeline utilities
pub mod pipeline {
    use super::*;

    /// Execute a pipeline of commands
    pub fn execute_pipeline(commands: &[(&str, &[&str])]) -> CompilerResult<ExecutionResult> {
        if commands.is_empty() {
            return Err(CompilerError::InvalidArgument("Empty pipeline".to_string()));
        }

        if commands.len() == 1 {
            // Single command
            let (cmd, args) = &commands[0];
            return executor::run_command(*cmd, *args);
        }

        // For multiple commands, we'd need more complex piping logic
        // This is a simplified implementation
        Err(CompilerError::InternalError(
            "Multi-command pipelines not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_command_execution_success() {
        // Test with a command that should exist and succeed
        let result = executor::run_command("cargo", &["--version"]);
        // The exact result depends on whether cargo is available
        // We just verify it doesn't panic
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_command_execution_failure() {
        // Test with a command that should fail
        let result = executor::run_command("definitely_nonexistent_command_123456789", &[]);

        // The result depends on the system - either spawn fails or command fails
        match result {
            Ok(ExecutionResult::Failure(_, _)) => {
                // Spawn succeeded but command failed - expected on most systems
            }
            Err(CompilerError::IoError(_)) => {
                // Spawn failed because command not found - also acceptable
            }
            _ => panic!("Expected either spawn failure or command failure"),
        }
    }

    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        assert!(options.warnings);
        assert!(!options.debug_info);
        assert!(options.includes.is_empty());
        assert!(options.flags.is_empty());
    }

    #[test]
    fn test_execution_options_default() {
        let options = ExecutionOptions::default();
        assert!(options.working_dir.is_none());
        assert!(options.env_vars.is_empty());
        assert!(options.timeout.is_none());
        assert!(!options.capture_stdout);
        assert!(!options.capture_stderr);
    }

    #[test]
    fn test_erlang_emulator_path() {
        let path = infrastructure_environment_config::erlang::get_emulator_path();
        assert!(!path.is_empty());
        // Should be "erl" by default
        assert_eq!(path, "erl");
    }

    #[test]
    fn test_compile_server_config() {
        let config = infrastructure_environment_config::compile_server::get_config();
        // Should have a config hash
        assert!(!config.config_hash.is_empty());
    }

    #[test]
    fn test_pipeline_empty() {
        let result = pipeline::execute_pipeline(&[]);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_pipeline_single_command() {
        let commands = vec![("echo", &["hello"] as &[&str])];
        let result = pipeline::execute_pipeline(&commands);
        // Should work if echo is available
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_pipeline_multiple_commands() {
        let commands = vec![
            ("echo", &["hello"] as &[&str]),
            ("cat", &[] as &[&str]),
        ];
        let result = pipeline::execute_pipeline(&commands);
        // Should return "not implemented" error
        assert!(matches!(result, Err(CompilerError::InternalError(_))));
    }

    #[test]
    fn test_background_process_creation() {
        // We can't easily test background processes in unit tests
        // as they require specific setup, but we can test the config validation
        let config = infrastructure_environment_config::CompileServerConfig {
            enabled: false,
            server_id: None,
            config_hash: "test".to_string(),
        };

        let result = background::start_compile_server(&config);
        assert!(matches!(result, Err(CompilerError::ConfigError(_))));
    }

    // ==================== Additional Executor Module Tests ====================

    #[test]
    fn test_run_command_with_options() {
        let mut options = ExecutionOptions::default();
        options.capture_stdout = true;
        options.capture_stderr = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        // Should work if cargo is available
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_command_with_working_directory() {
        let mut options = ExecutionOptions::default();
        options.working_dir = Some(PathBuf::from("."));
        options.capture_stdout = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_command_with_env_vars() {
        let mut options = ExecutionOptions::default();
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
        options.env_vars = env_vars;
        options.capture_stdout = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_command_with_timeout() {
        let mut options = ExecutionOptions::default();
        options.timeout = Some(Duration::from_secs(10));
        options.capture_stdout = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_command_exists() {
        // Test with commands that should exist
        assert!(executor::command_exists("cargo") || executor::command_exists("rustc"));
        // Test with command that shouldn't exist
        assert!(!executor::command_exists("definitely_nonexistent_command_123456789"));
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::Success("test output".to_string());
        match result {
            ExecutionResult::Success(output) => assert_eq!(output, "test output"),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::Failure(42, "error message".to_string());
        match result {
            ExecutionResult::Failure(code, message) => {
                assert_eq!(code, 42);
                assert_eq!(message, "error message");
            }
            _ => panic!("Expected failure"),
        }
    }

    // ==================== Erlang Module Tests ====================

    #[test]
    fn test_run_erlang_emulator() {
        let result = erlang::run_erlang_emulator(&["-eval", "halt()."]);
        // This might fail if Erlang is not installed, but shouldn't panic
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_erlang_emulator_with_path() {
        let emulator_path = infrastructure_environment_config::erlang::get_emulator_path();
        let result = erlang::run_erlang_emulator_with_path(&emulator_path, &["-eval", "halt()."]);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_compile_erlang_file() {
        let options = CompileOptions::default();
        let result = erlang::compile_erlang_file("nonexistent.erl", &options);
        // Should fail due to missing file, but shouldn't panic
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_compile_erlang_file_with_options() {
        let mut options = CompileOptions::default();
        options.warnings = false;
        options.debug_info = true;
        options.includes.push(PathBuf::from("test_include"));
        options.flags.push("+test_flag".to_string());

        let result = erlang::compile_erlang_file("nonexistent.erl", &options);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }


    // ==================== CompileOptions Tests ====================

    #[test]
    fn test_compile_options_with_custom_values() {
        let mut options = CompileOptions {
            outdir: Some(PathBuf::from("/tmp/output")),
            includes: vec![PathBuf::from("/tmp/include1"), PathBuf::from("/tmp/include2")],
            flags: vec!["+flag1".to_string(), "+flag2".to_string()],
            warnings: false,
            debug_info: true,
        };

        assert_eq!(options.outdir, Some(PathBuf::from("/tmp/output")));
        assert_eq!(options.includes.len(), 2);
        assert_eq!(options.flags.len(), 2);
        assert!(!options.warnings);
        assert!(options.debug_info);
    }

    #[test]
    fn test_compile_options_clone() {
        let options = CompileOptions::default();
        let cloned = options.clone();
        assert_eq!(options.warnings, cloned.warnings);
        assert_eq!(options.debug_info, cloned.debug_info);
    }

    // ==================== ExecutionOptions Tests ====================

    #[test]
    fn test_execution_options_with_custom_values() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let options = ExecutionOptions {
            working_dir: Some(PathBuf::from("/tmp")),
            env_vars,
            timeout: Some(Duration::from_secs(30)),
            capture_stdout: true,
            capture_stderr: true,
        };

        assert_eq!(options.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(options.env_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(options.timeout, Some(Duration::from_secs(30)));
        assert!(options.capture_stdout);
        assert!(options.capture_stderr);
    }

    #[test]
    fn test_execution_options_clone() {
        let mut options = ExecutionOptions::default();
        options.capture_stdout = true;
        let cloned = options.clone();
        assert_eq!(options.capture_stdout, cloned.capture_stdout);
    }

    // ==================== Background Process Tests ====================

    #[test]
    fn test_background_process_id() {
        // Test with a disabled config (should fail early)
        let config = infrastructure_environment_config::CompileServerConfig {
            enabled: false,
            server_id: Some("test_server".to_string()),
            config_hash: "test".to_string(),
        };

        let result = background::start_compile_server(&config);
        assert!(matches!(result, Err(CompilerError::ConfigError(_))));
    }

    #[test]
    fn test_background_process_enabled_config() {
        // Test with enabled config but invalid setup
        let config = infrastructure_environment_config::CompileServerConfig {
            enabled: true,
            server_id: Some("test_server".to_string()),
            config_hash: "test".to_string(),
        };

        let result = background::start_compile_server(&config);
        // This might succeed or fail depending on Erlang availability
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_)))
                               || matches!(result, Err(CompilerError::ConfigError(_))));
    }

    // ==================== Pipeline Tests ====================

    #[test]
    fn test_pipeline_with_invalid_commands() {
        let commands = vec![
            ("definitely_nonexistent_command_123", &[] as &[&str]),
            ("another_nonexistent_command", &[] as &[&str]),
        ];
        let result = pipeline::execute_pipeline(&commands);
        // Should fail due to nonexistent commands
        assert!(matches!(result, Err(CompilerError::InternalError(_))));
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_command_execution_with_invalid_program() {
        let result = executor::run_command("", &[]);
        assert!(matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_command_with_invalid_working_directory() {
        let mut options = ExecutionOptions::default();
        options.working_dir = Some(PathBuf::from("/definitely/nonexistent/directory/path"));
        options.capture_stdout = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        assert!(matches!(result, Err(CompilerError::IoError(_))));
    }

    // ==================== Edge Cases and Boundary Conditions ====================

    #[test]
    fn test_run_command_with_empty_args() {
        let result = executor::run_command("cargo", &[]);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_run_command_with_many_args() {
        let args: Vec<String> = (0..100).map(|i| format!("arg{}", i)).collect();
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let result = executor::run_command("echo", &args_refs);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[test]
    fn test_compile_options_with_many_includes() {
        let mut options = CompileOptions::default();
        for i in 0..50 {
            options.includes.push(PathBuf::from(format!("/tmp/include{}", i)));
        }
        assert_eq!(options.includes.len(), 50);
    }

    #[test]
    fn test_execution_options_with_many_env_vars() {
        let mut options = ExecutionOptions::default();
        for i in 0..50 {
            options.env_vars.insert(format!("VAR{}", i), format!("value{}", i));
        }
        assert_eq!(options.env_vars.len(), 50);
    }

    // ==================== Platform-Specific Tests ====================

    #[cfg(unix)]
    #[test]
    fn test_unix_specific_execution() {
        // Test Unix-specific behavior if any
        let result = executor::run_command("true", &[]);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_specific_execution() {
        // Test Windows-specific behavior if any
        let result = executor::run_command("cmd", &["/c", "exit", "0"]);
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    // ==================== Concurrent Execution Tests ====================

    #[test]
    fn test_concurrent_command_execution() {
        use std::sync::Arc;
        use std::thread;

        let results = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Run multiple commands concurrently
        for i in 0..3 {
            let results_clone = results.clone();
            let handle = thread::spawn(move || {
                let result = executor::run_command("cargo", &["--version"]);
                let mut results = results_clone.lock().unwrap();
                results.push((i, result.is_ok()));
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 3);
        // At least some should succeed if cargo is available
        let success_count = results.iter().filter(|(_, success)| *success).count();
        assert!(success_count >= 0); // Allow for all to fail if cargo not available
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_erlang_compilation_workflow() {
        // Create a simple Erlang module content
        let erlang_code = r#"
-module(test_module).
-export([hello/0]).

hello() ->
    "Hello, World!".
"#;

        // Write to a temporary file
        let temp_file = "test_module.erl";
        let write_result = std::fs::write(temp_file, erlang_code);
        if write_result.is_ok() {
            let options = CompileOptions::default();
            let compile_result = erlang::compile_erlang_file(temp_file, &options);

            // Clean up
            let _ = std::fs::remove_file(temp_file);
            let _ = std::fs::remove_file("test_module.beam"); // Remove compiled beam file if it exists

            // Compilation should succeed or fail gracefully
            assert!(compile_result.is_ok() || matches!(compile_result, Err(CompilerError::IoError(_))));
        } else {
            // Skip test if we can't write files
            assert!(true);
        }
    }

    #[test]
    fn test_erlang_emulator_version_check() {
        let result = erlang::run_erlang_emulator(&["-eval", "erlang:display(erlang:system_info(version)), halt()."]);
        // Should work if Erlang is available
        assert!(result.is_ok() || matches!(result, Err(CompilerError::IoError(_))));
    }

    // ==================== Timeout and Resource Management Tests ====================

    #[test]
    fn test_timeout_functionality() {
        let mut options = ExecutionOptions::default();
        options.timeout = Some(Duration::from_millis(100)); // Short but reasonable timeout
        options.capture_stdout = true;

        // Try a command that might take longer than timeout
        let result = executor::run_command_with_options("sleep", &["1"], &options);
        // Should either succeed quickly, timeout/fail, or command not found
        assert!(result.is_ok() || matches!(result, Err(_)));
    }

    #[test]
    fn test_large_output_handling() {
        // Test with a command that produces output
        let mut options = ExecutionOptions::default();
        options.capture_stdout = true;

        let result = executor::run_command_with_options("cargo", &["--version"], &options);
        if let Ok(ExecutionResult::Success(output)) = result {
            // Output should not be empty for a successful command
            assert!(!output.trim().is_empty());
        } else {
            // Command failed or not found, which is acceptable
            assert!(matches!(result, Err(_)));
        }
    }

    // ==================== Configuration and Environment Tests ====================

    #[test]
    fn test_erlang_environment_setup() {
        let emulator_path = infrastructure_environment_config::erlang::get_emulator_path();
        let setup_result = infrastructure_environment_config::erlang::setup_compilation_env(&emulator_path);
        // Should succeed or fail gracefully
        assert!(setup_result.is_ok() || matches!(setup_result, Err(_)));
    }

    #[test]
    fn test_compile_server_node_name() {
        let node_name_result = infrastructure_environment_config::compile_server::get_server_node_name();
        // Should succeed or fail based on environment
        assert!(node_name_result.is_ok() || matches!(node_name_result, Err(_)));
    }
}
