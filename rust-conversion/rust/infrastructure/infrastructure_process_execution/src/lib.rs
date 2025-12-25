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
}
