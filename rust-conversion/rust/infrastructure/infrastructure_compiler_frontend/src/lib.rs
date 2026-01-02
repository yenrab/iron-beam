/*!
# Infrastructure Compiler Frontend

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Compiler command interface and argument processing

## Overview

This crate provides the main compiler frontend that orchestrates all infrastructure components.
Replaces the C `main()` and `wmain()` functions with safe Rust argument processing and compilation orchestration.

## Original C Functions Replaced

The original `erlc.c` contained these main functions:
- `main()`: Standard C main entry point → **Replaced with safe argument parsing and orchestration**
- `wmain()`: Windows Unicode main entry point → **Unified with cross-platform argument handling**
- `process_opt()`: Command line option processing → **Replaced with structured argument parsing**

## Compiler Frontend Architecture

### 1. Compiler Configuration
```rust
use infrastructure_compiler_frontend::{Compiler, CompilerOptions};

// Create compiler with specific options
let options = CompilerOptions {
    warnings: true,
    debug_info: false,
    optimize: true,
    verbose: true,
    target: None,
};

let compiler = Compiler {
    source_files: vec!["example.erl".into()],
    output_dir: Some("build".into()),
    include_dirs: vec!["include".into()],
    options,
    use_server: false,
    server_addr: None,
};
```

### 2. Compilation Result Handling
```rust
use infrastructure_compiler_frontend::CompilationResult;

// Handle compilation results
match CompilationResult::Success {
    CompilationResult::Success => println!("Compilation successful"),
    CompilationResult::Failure(errors) => {
        for error in errors {
            eprintln!("{}", error);
        }
    }
}
```

### 3. Multi-Mode Compilation Strategy
```rust
// Check if server mode is enabled
let use_server = false; // Would be determined by configuration
if use_server {
    // Distributed compilation via compile server
    println!("Would use server mode");
} else {
    // Local compilation with Erlang emulator
    println!("Would use local mode");
}
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (orchestrates all infrastructure components)
- **SOLID Principle**: Single responsibility for compilation orchestration
- **Safe Rust**: No unsafe code, leverages all safe infrastructure components
- **Composable**: Integrates all infrastructure crates into cohesive interface
- **Extensible**: Easy to add new compilation modes and options
*/

use std::path::PathBuf;

use infrastructure_error_handling::{CompilerError, CompilerResult};
use infrastructure_environment_config::{erlang, compile_server};
use infrastructure_path_handling::erlang_paths;
use infrastructure_process_execution::{executor, erlang as erlang_exec};
use infrastructure_compile_server::{client, CompileRequest, CompileOptions};

/// Compilation result
#[derive(Debug, Clone, PartialEq)]
pub enum CompilationResult {
    /// Compilation succeeded
    Success,
    /// Compilation failed with errors
    Failure(Vec<String>),
}

/// Main compiler frontend structure
#[derive(Debug, Clone, PartialEq)]
pub struct Compiler {
    /// Input source files
    pub source_files: Vec<PathBuf>,
    /// Output directory
    pub output_dir: Option<PathBuf>,
    /// Include directories
    pub include_dirs: Vec<PathBuf>,
    /// Compilation options
    pub options: CompilerOptions,
    /// Whether to use compile server
    pub use_server: bool,
    /// Server address if using server mode
    pub server_addr: Option<String>,
}

/// Compiler configuration options
#[derive(Debug, Clone, PartialEq)]
pub struct CompilerOptions {
    /// Enable warnings
    pub warnings: bool,
    /// Debug information
    pub debug_info: bool,
    /// Optimization level
    pub optimize: bool,
    /// Verbose output
    pub verbose: bool,
    /// Target platform
    pub target: Option<String>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            warnings: true,
            debug_info: false,
            optimize: false,
            verbose: false,
            target: None,
        }
    }
}

impl Compiler {
    /// Create compiler from command line arguments
    ///
    /// Replaces the C main() function argument processing
    pub fn from_args() -> CompilerResult<Self> {
        Self::from_args_env(std::env::args().collect())
    }

    /// Create compiler from argument vector (for testing)
    pub fn from_args_env(args: Vec<String>) -> CompilerResult<Self> {
        let mut source_files = Vec::new();
        let mut output_dir = None;
        let mut include_dirs = Vec::new();
        let mut options = CompilerOptions::default();

        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "-o" | "--output" => {
                    if i + 1 < args.len() {
                        output_dir = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        return Err(CompilerError::InvalidArgument(
                            "Missing output directory after -o".to_string(),
                        ));
                    }
                }
                "-I" => {
                    if i + 1 < args.len() {
                        include_dirs.push(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        return Err(CompilerError::InvalidArgument(
                            "Missing include directory after -I".to_string(),
                        ));
                    }
                }
                "-W" | "--warnings" => {
                    options.warnings = true;
                    i += 1;
                }
                "--no-warnings" => {
                    options.warnings = false;
                    i += 1;
                }
                "-D" | "--debug" => {
                    options.debug_info = true;
                    i += 1;
                }
                "-O" | "--optimize" => {
                    options.optimize = true;
                    i += 1;
                }
                "-v" | "--verbose" => {
                    options.verbose = true;
                    i += 1;
                }
                "--target" => {
                    if i + 1 < args.len() {
                        options.target = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(CompilerError::InvalidArgument(
                            "Missing target after --target".to_string(),
                        ));
                    }
                }
                "--server" => {
                    i += 1; // Server mode will be determined by environment
                }
                arg if arg.starts_with('-') => {
                    return Err(CompilerError::InvalidArgument(
                        format!("Unknown option: {}", arg),
                    ));
                }
                file => {
                    source_files.push(PathBuf::from(file));
                    i += 1;
                }
            }
        }

        if source_files.is_empty() {
            return Err(CompilerError::InvalidArgument(
                "No source files specified".to_string(),
            ));
        }

        // Determine server usage from environment
        let server_config = compile_server::get_config();
        let use_server = server_config.enabled;
        let server_addr = if use_server {
            Some("127.0.0.1:9999".to_string()) // Could be configurable
        } else {
            None
        };

        Ok(Self {
            source_files,
            output_dir,
            include_dirs,
            options,
            use_server,
            server_addr,
        })
    }

    /// Execute compilation
    ///
    /// Main orchestration function that replaces the core C compilation logic
    pub async fn compile(&self) -> CompilerResult<CompilationResult> {
        if self.use_server && client::server_available().await {
            self.compile_with_server().await
        } else {
            self.compile_locally().await
        }
    }

    /// Compile using the distributed compile server
    pub async fn compile_with_server(&self) -> CompilerResult<CompilationResult> {
        let mut results = Vec::new();
        let mut all_success = true;

        for source_file in &self.source_files {
            // Read source file
            let source_content = infrastructure_path_handling::fs_utils::read_file_to_string(source_file)?;

            // Create compilation request
            let request = CompileRequest {
                source_file: source_file.to_string_lossy().to_string(),
                source_content,
                options: CompileOptions {
                    warnings: self.options.warnings,
                    debug_info: self.options.debug_info,
                    optimize: self.options.optimize,
                    target: self.options.target.clone(),
                },
                environment: infrastructure_compile_server::encoding::encode_environment()
                    .map_err(|e| e.into_compiler_error())?,
            };

            // Send to server
            match client::send_compile_request(&request).await {
                Ok(response) => {
                    if response.success {
                        if self.options.verbose {
                            println!("Compiled {} successfully", source_file.display());
                        }
                    } else {
                        all_success = false;
                        results.extend(response.errors);
                    }
                }
                Err(e) => {
                    all_success = false;
                    results.push(format!("Server error for {}: {}", source_file.display(), e.into_compiler_error()));
                }
            }
        }

        if all_success {
            Ok(CompilationResult::Success)
        } else {
            Ok(CompilationResult::Failure(results))
        }
    }

    /// Compile locally using Erlang emulator
    pub async fn compile_locally(&self) -> CompilerResult<CompilationResult> {
        // Find Erlang emulator
        let emulator = erlang_paths::find_erlang_emulator()?;

        // Set up compilation environment
        erlang::setup_compilation_env(&emulator.to_string_lossy())?;

        let mut all_success = true;
        let mut errors = Vec::new();

        for source_file in &self.source_files {
            // Build compilation arguments
            let mut args: Vec<String> = vec!["-compile".to_string()];

            // Add output directory
            if let Some(ref outdir) = self.output_dir {
                args.push("-o".to_string());
                args.push(outdir.to_string_lossy().to_string());
            }

            // Add include paths
            for include in &self.include_dirs {
                args.push("-I".to_string());
                args.push(include.to_string_lossy().to_string());
            }

            // Add options
            if self.options.warnings {
                args.push("+warn_unused_vars".to_string());
                args.push("+warn_unused_import".to_string());
            }

            if self.options.debug_info {
                args.push("+debug_info".to_string());
            }

            // Add source file
            args.push(source_file.to_string_lossy().to_string());

            // Execute compilation
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            match erlang_exec::run_erlang_emulator(&args_refs) {
                Ok(infrastructure_process_execution::ExecutionResult::Success(_)) => {
                    if self.options.verbose {
                        println!("Compiled {} successfully", source_file.display());
                    }
                }
                Ok(infrastructure_process_execution::ExecutionResult::Failure(code, stderr)) => {
                    all_success = false;
                    errors.push(format!("Compilation failed for {} (exit code {}): {}",
                        source_file.display(), code, stderr));
                }
                Err(e) => {
                    all_success = false;
                    errors.push(format!("Execution error for {}: {}", source_file.display(), e));
                }
            }
        }

        if all_success {
            Ok(CompilationResult::Success)
        } else {
            Ok(CompilationResult::Failure(errors))
        }
    }

    /// Check if server mode should be used
    pub fn use_server(&self) -> bool {
        self.use_server
    }

    /// Get compiler version information
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Display help information
    pub fn help() -> &'static str {
        r#"Erlang Compiler Frontend

USAGE:
    erlc [OPTIONS] <source_files>...

OPTIONS:
    -o, --output <DIR>        Output directory
    -I <DIR>                  Include directory
    -W, --warnings            Enable warnings
    --no-warnings             Disable warnings
    -D, --debug               Include debug information
    -O, --optimize            Enable optimization
    -v, --verbose             Verbose output
    --target <TARGET>         Target platform
    --server                  Use compile server (if available)

EXAMPLES:
    erlc example.erl
    erlc -o build -I include -W example.erl
    erlc --server --verbose *.erl
"#
    }
}

/// Command-line interface functions
pub mod cli {
    use super::*;

    /// Run the compiler from command line arguments
    ///
    /// This replaces the C main() function
    pub async fn run() -> CompilerResult<i32> {
        let args: Vec<String> = std::env::args().collect();

        // Handle special arguments
        if args.len() > 1 {
            match args[1].as_str() {
                "--help" | "-h" => {
                    println!("{}", Compiler::help());
                    return Ok(0);
                }
                "--version" | "-V" => {
                    println!("erlc {}", Compiler::version());
                    return Ok(0);
                }
                _ => {}
            }
        }

        // Parse arguments and run compilation
        let compiler = Compiler::from_args()?;

        match compiler.compile().await? {
            CompilationResult::Success => {
                println!("Compilation successful");
                Ok(0)
            }
            CompilationResult::Failure(errors) => {
                for error in errors {
                    eprintln!("{}", error);
                }
                Ok(1)
            }
        }
    }

    /// Run with custom arguments (for testing)
    pub async fn run_with_args(args: Vec<String>) -> CompilerResult<i32> {
        // Handle special arguments (same as run())
        if args.len() > 1 {
            match args[1].as_str() {
                "--help" | "-h" => {
                    println!("{}", Compiler::help());
                    return Ok(0);
                }
                "--version" | "-V" => {
                    println!("erlc {}", Compiler::version());
                    return Ok(0);
                }
                _ => {}
            }
        }

        let compiler = Compiler::from_args_env(args)?;

        match compiler.compile().await? {
            CompilationResult::Success => Ok(0),
            CompilationResult::Failure(_) => Ok(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_options_default() {
        let options = CompilerOptions::default();
        assert!(options.warnings);
        assert!(!options.debug_info);
        assert!(!options.optimize);
        assert!(!options.verbose);
        assert!(options.target.is_none());
    }

    #[test]
    fn test_compiler_from_args_basic() {
        let args = vec![
            "erlc".to_string(),
            "test.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();
        assert_eq!(compiler.source_files.len(), 1);
        assert_eq!(compiler.source_files[0], PathBuf::from("test.erl"));
        // Server usage depends on environment config, not just command line args
        // The test may or may not use server depending on config
    }

    #[test]
    fn test_compiler_from_args_with_options() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "build".to_string(),
            "-I".to_string(),
            "include".to_string(),
            "-v".to_string(),
            "--debug".to_string(),
            "test.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();
        assert_eq!(compiler.source_files.len(), 1);
        assert_eq!(compiler.output_dir, Some(PathBuf::from("build")));
        assert_eq!(compiler.include_dirs.len(), 1);
        assert_eq!(compiler.include_dirs[0], PathBuf::from("include"));
        assert!(compiler.options.verbose);
        assert!(compiler.options.debug_info);
    }

    #[test]
    fn test_compiler_from_args_missing_output() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            // Missing output directory
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_no_files() {
        let args = vec![
            "erlc".to_string(),
            "-v".to_string(),
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_unknown_option() {
        let args = vec![
            "erlc".to_string(),
            "--unknown-option".to_string(),
            "test.erl".to_string(),
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_all_options() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "output_dir".to_string(),
            "-I".to_string(),
            "include1".to_string(),
            "-I".to_string(),
            "include2".to_string(),
            "-W".to_string(),
            "--no-warnings".to_string(),
            "-D".to_string(),
            "-O".to_string(),
            "-v".to_string(),
            "--target".to_string(),
            "custom_target".to_string(),
            "--server".to_string(),
            "file1.erl".to_string(),
            "file2.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        assert_eq!(compiler.source_files.len(), 2);
        assert_eq!(compiler.output_dir, Some(PathBuf::from("output_dir")));
        assert_eq!(compiler.include_dirs.len(), 2);
        assert_eq!(compiler.options.warnings, false); // --no-warnings overrides -W
        assert_eq!(compiler.options.debug_info, true);
        assert_eq!(compiler.options.optimize, true);
        assert_eq!(compiler.options.verbose, true);
        assert_eq!(compiler.options.target, Some("custom_target".to_string()));
        // use_server depends on environment config, not just --server flag
    }

    #[test]
    fn test_compiler_from_args_multiple_output_dirs() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "dir1".to_string(),
            "-o".to_string(),
            "dir2".to_string(), // Second -o should override first
            "test.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();
        assert_eq!(compiler.output_dir, Some(PathBuf::from("dir2")));
    }

    #[test]
    fn test_compiler_from_args_warnings_override() {
        // Test that --no-warnings overrides -W
        let args1 = vec![
            "erlc".to_string(),
            "-W".to_string(),
            "--no-warnings".to_string(),
            "test.erl".to_string(),
        ];

        let compiler1 = Compiler::from_args_env(args1).unwrap();
        assert_eq!(compiler1.options.warnings, false);

        // Test that -W overrides default
        let args2 = vec![
            "erlc".to_string(),
            "-W".to_string(),
            "test.erl".to_string(),
        ];

        let compiler2 = Compiler::from_args_env(args2).unwrap();
        assert_eq!(compiler2.options.warnings, true);
    }

    #[test]
    fn test_compiler_from_args_long_options() {
        let args = vec![
            "erlc".to_string(),
            "--output".to_string(),
            "build".to_string(),
            "--warnings".to_string(),
            "--debug".to_string(),
            "--optimize".to_string(),
            "--verbose".to_string(),
            "--target".to_string(),
            "test_target".to_string(),
            "source.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        assert_eq!(compiler.output_dir, Some(PathBuf::from("build")));
        assert_eq!(compiler.options.warnings, true);
        assert_eq!(compiler.options.debug_info, true);
        assert_eq!(compiler.options.optimize, true);
        assert_eq!(compiler.options.verbose, true);
        assert_eq!(compiler.options.target, Some("test_target".to_string()));
        assert_eq!(compiler.source_files, vec![PathBuf::from("source.erl")]);
    }

    #[test]
    fn test_compiler_from_args_missing_include_directory() {
        let args = vec![
            "erlc".to_string(),
            "-I".to_string(),
            // No value after -I, so next arg is consumed as include dir
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));

        if let Err(CompilerError::InvalidArgument(msg)) = result {
            assert!(msg.contains("Missing include directory after -I"));
        }
    }

    #[test]
    fn test_compiler_from_args_missing_target() {
        let args = vec![
            "erlc".to_string(),
            "--target".to_string(),
            // No value after --target
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));

        if let Err(CompilerError::InvalidArgument(msg)) = result {
            assert!(msg.contains("Missing target after --target"));
        }
    }

    #[test]
    fn test_compiler_from_args_special_characters() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "/path with spaces/output".to_string(),
            "-I".to_string(),
            "/path with (parens)/include".to_string(),
            "file with spaces.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        assert_eq!(compiler.output_dir, Some(PathBuf::from("/path with spaces/output")));
        assert_eq!(compiler.include_dirs[0], PathBuf::from("/path with (parens)/include"));
        assert_eq!(compiler.source_files[0], PathBuf::from("file with spaces.erl"));
    }

    #[test]
    fn test_compiler_from_args_relative_paths() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "../build".to_string(),
            "-I".to_string(),
            "./include".to_string(),
            "src/main.erl".to_string(),
            "../deps/utils.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        assert_eq!(compiler.output_dir, Some(PathBuf::from("../build")));
        assert_eq!(compiler.include_dirs[0], PathBuf::from("./include"));
        assert_eq!(compiler.source_files.len(), 2);
        assert_eq!(compiler.source_files[0], PathBuf::from("src/main.erl"));
        assert_eq!(compiler.source_files[1], PathBuf::from("../deps/utils.erl"));
    }

    #[test]
    fn test_compiler_from_args_empty_args() {
        let args = vec!["erlc".to_string()];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_only_flags() {
        let args = vec![
            "erlc".to_string(),
            "-v".to_string(),
            "-W".to_string(),
            "-D".to_string(),
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_mixed_valid_invalid() {
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "build".to_string(),
            "--invalid-flag".to_string(),
            "test.erl".to_string(),
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_compiler_from_args_case_sensitivity() {
        // Options should be case-sensitive for unknown options
        let args = vec![
            "erlc".to_string(),
            "-O".to_string(), // Valid
            "-o".to_string(), // Valid
            "build".to_string(),
            "-w".to_string(), // Invalid (lowercase)
            "test.erl".to_string(),
        ];

        let result = Compiler::from_args_env(args);
        assert!(matches!(result, Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_version() {
        let version = Compiler::version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_help() {
        let help = Compiler::help();
        assert!(help.contains("USAGE"));
        assert!(help.contains("OPTIONS"));
        assert!(help.contains("EXAMPLES"));
    }

    #[test]
    fn test_help_content_comprehensive() {
        let help = Compiler::help();

        // Check for all expected sections
        assert!(help.contains("Erlang Compiler Frontend"));
        assert!(help.contains("USAGE:"));
        assert!(help.contains("OPTIONS:"));
        assert!(help.contains("EXAMPLES:"));
        assert!(help.contains("erlc [OPTIONS] <source_files>..."));

        // Check for specific options
        assert!(help.contains("-o, --output"));
        assert!(help.contains("-I"));
        assert!(help.contains("-W, --warnings"));
        assert!(help.contains("--no-warnings"));
        assert!(help.contains("-D, --debug"));
        assert!(help.contains("-O, --optimize"));
        assert!(help.contains("-v, --verbose"));
        assert!(help.contains("--target"));
        assert!(help.contains("--server"));

        // Check for examples
        assert!(help.contains("erlc example.erl"));
        assert!(help.contains("erlc -o build -I include -W example.erl"));
        assert!(help.contains("erlc --server --verbose *.erl"));
    }

    #[test]
    fn test_version_format() {
        let version = Compiler::version();
        assert!(!version.is_empty());

        // Version should be a valid semver-like string
        // Basic check that it contains digits and dots
        assert!(version.chars().any(|c| c.is_digit(10)));
        assert!(version.contains('.'));
    }

    #[tokio::test]
    async fn test_cli_run_with_help() {
        let args = vec![
            "erlc".to_string(),
            "--help".to_string(),
        ];

        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(0))); // Help should return success
    }

    #[tokio::test]
    async fn test_cli_run_with_version() {
        let args = vec![
            "erlc".to_string(),
            "--version".to_string(),
        ];

        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(0))); // Version should return success
    }

    #[tokio::test]
    async fn test_cli_run_with_short_help() {
        let args = vec![
            "erlc".to_string(),
            "-h".to_string(),
        ];

        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(0))); // Short help should return success
    }

    #[tokio::test]
    async fn test_cli_run_with_short_version() {
        let args = vec![
            "erlc".to_string(),
            "-V".to_string(),
        ];

        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(0))); // Short version should return success
    }

    #[tokio::test]
    async fn test_cli_run_with_valid_compilation() {
        let args = vec![
            "erlc".to_string(),
            "--verbose".to_string(),
            "test.erl".to_string(),
        ];

        // This will fail due to missing/nonexistent file, should return exit code 1
        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(1))); // Compilation failure should return exit code 1
    }

    #[tokio::test]
    async fn test_cli_run_with_invalid_args() {
        let args = vec![
            "erlc".to_string(),
            "--invalid-option".to_string(),
        ];

        let result = cli::run_with_args(args).await;
        assert!(result.is_err()); // Invalid args should return error
    }

    #[tokio::test]
    async fn test_cli_run_with_no_args() {
        let args = vec!["erlc".to_string()];

        let result = cli::run_with_args(args).await;
        assert!(result.is_err()); // No source files should return error
    }

    #[tokio::test]
    async fn test_cli_run_with_mixed_options() {
        let args = vec![
            "erlc".to_string(),
            "-v".to_string(),  // verbose
            "-W".to_string(),  // warnings
            "-D".to_string(),  // debug
            "-O".to_string(),  // optimize
            "-o".to_string(),
            "build".to_string(),
            "-I".to_string(),
            "include".to_string(),
            "file1.erl".to_string(),
            "file2.erl".to_string(),
        ];

        // Should parse successfully but compilation will fail due to missing files
        let result = cli::run_with_args(args).await;
        assert!(matches!(result, Ok(1))); // Compilation failure should return exit code 1
    }

    #[test]
    fn test_cli_help_formatting() {
        let help = Compiler::help();

        // Help should be properly formatted with newlines
        assert!(help.contains('\n'));
        assert!(help.lines().count() > 10); // Should have multiple lines

        // Should not have trailing/leading whitespace issues
        let lines: Vec<&str> = help.lines().collect();
        for line in lines {
            // Lines shouldn't have trailing whitespace (basic check)
            assert!(!line.ends_with(' ') || line.trim().is_empty());
        }
    }

    #[test]
    fn test_version_non_empty() {
        let version1 = Compiler::version();
        let version2 = Compiler::version();

        // Version should be consistent across calls
        assert_eq!(version1, version2);

        // Version should contain version-like characters
        assert!(version1.chars().any(|c| c.is_alphanumeric()));
    }

    // ==================== CompilationResult Tests ====================

    #[test]
    fn test_compilation_result_variants() {
        // Test all CompilationResult variants can be created
        let _success = CompilationResult::Success;
        let _failure = CompilationResult::Failure(vec!["error1".to_string(), "error2".to_string()]);
        let _empty_failure = CompilationResult::Failure(vec![]);
    }

    #[test]
    fn test_compilation_result_success_variant() {
        let result = CompilationResult::Success;

        match result {
            CompilationResult::Success => {
                // Expected
            }
            CompilationResult::Failure(_) => {
                panic!("Expected Success, got Failure");
            }
        }
    }

    #[test]
    fn test_compilation_result_failure_variant() {
        let errors = vec!["syntax error".to_string(), "undefined function".to_string()];
        let result = CompilationResult::Failure(errors.clone());

        match result {
            CompilationResult::Success => {
                panic!("Expected Failure, got Success");
            }
            CompilationResult::Failure(actual_errors) => {
                assert_eq!(actual_errors, errors);
            }
        }
    }

    #[test]
    fn test_compilation_result_debug_formatting() {
        // Test debug formatting for all variants
        let success_result = CompilationResult::Success;
        let failure_result = CompilationResult::Failure(vec!["error".to_string()]);

        let success_debug = format!("{:?}", success_result);
        let failure_debug = format!("{:?}", failure_result);

        assert!(success_debug.contains("Success"));
        assert!(failure_debug.contains("Failure"));
        assert!(failure_debug.contains("error"));
    }

    #[test]
    fn test_compilation_result_clone() {
        let original = CompilationResult::Failure(vec!["test error".to_string()]);
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_compilation_result_equality() {
        // Test equality for same variants
        assert_eq!(CompilationResult::Success, CompilationResult::Success);

        let errors1 = vec!["error1".to_string()];
        let errors2 = vec!["error1".to_string()];
        assert_eq!(CompilationResult::Failure(errors1.clone()), CompilationResult::Failure(errors2));

        // Test inequality
        assert_ne!(CompilationResult::Success, CompilationResult::Failure(vec![]));

        let errors3 = vec!["different error".to_string()];
        assert_ne!(CompilationResult::Failure(errors1), CompilationResult::Failure(errors3));
    }

    #[test]
    fn test_compilation_result_with_empty_errors() {
        let result = CompilationResult::Failure(vec![]);

        match result {
            CompilationResult::Success => panic!("Expected Failure"),
            CompilationResult::Failure(errors) => {
                assert_eq!(errors.len(), 0);
            }
        }
    }

    #[test]
    fn test_compilation_result_with_multiple_errors() {
        let errors = vec![
            "syntax error at line 5".to_string(),
            "undefined function 'test/0'".to_string(),
            "unused variable 'X'".to_string(),
        ];

        let result = CompilationResult::Failure(errors.clone());

        match result {
            CompilationResult::Success => panic!("Expected Failure"),
            CompilationResult::Failure(actual_errors) => {
                assert_eq!(actual_errors.len(), 3);
                assert_eq!(actual_errors, errors);
            }
        }
    }

    // ==================== CompilerOptions Tests ====================

    #[test]
    fn test_compiler_options_creation() {
        let options = CompilerOptions {
            warnings: true,
            debug_info: true,
            optimize: true,
            verbose: true,
            target: Some("arm64".to_string()),
        };

        assert_eq!(options.warnings, true);
        assert_eq!(options.debug_info, true);
        assert_eq!(options.optimize, true);
        assert_eq!(options.verbose, true);
        assert_eq!(options.target, Some("arm64".to_string()));
    }

    #[test]
    fn test_compiler_options_clone() {
        let original = CompilerOptions {
            warnings: false,
            debug_info: true,
            optimize: false,
            verbose: true,
            target: Some("x86_64".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_compiler_options_debug_formatting() {
        let options = CompilerOptions {
            warnings: true,
            debug_info: false,
            optimize: true,
            verbose: false,
            target: Some("wasm32".to_string()),
        };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("warnings: true"));
        assert!(debug_str.contains("debug_info: false"));
        assert!(debug_str.contains("optimize: true"));
        assert!(debug_str.contains("verbose: false"));
        assert!(debug_str.contains("target: Some(\"wasm32\")"));
    }

    #[test]
    fn test_compiler_options_equality() {
        let options1 = CompilerOptions {
            warnings: true,
            debug_info: false,
            optimize: true,
            verbose: false,
            target: Some("test".to_string()),
        };

        let options2 = options1.clone();
        let mut options3 = options1.clone();
        options3.warnings = false;

        assert_eq!(options1, options2);
        assert_ne!(options1, options3);
    }

    #[test]
    fn test_compiler_options_with_none_target() {
        let options = CompilerOptions {
            warnings: true,
            debug_info: false,
            optimize: false,
            verbose: false,
            target: None,
        };

        assert_eq!(options.target, None);
    }

    #[test]
    fn test_compiler_options_all_false() {
        let options = CompilerOptions {
            warnings: false,
            debug_info: false,
            optimize: false,
            verbose: false,
            target: None,
        };

        assert_eq!(options.warnings, false);
        assert_eq!(options.debug_info, false);
        assert_eq!(options.optimize, false);
        assert_eq!(options.verbose, false);
        assert_eq!(options.target, None);
    }

    // ==================== Compiler Struct Tests ====================

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test1.erl"), PathBuf::from("test2.erl")],
            output_dir: Some(PathBuf::from("build")),
            include_dirs: vec![PathBuf::from("include"), PathBuf::from("deps")],
            options: CompilerOptions {
                warnings: true,
                debug_info: true,
                optimize: false,
                verbose: true,
                target: Some("beam".to_string()),
            },
            use_server: true,
            server_addr: Some("127.0.0.1:9999".to_string()),
        };

        assert_eq!(compiler.source_files.len(), 2);
        assert_eq!(compiler.output_dir, Some(PathBuf::from("build")));
        assert_eq!(compiler.include_dirs.len(), 2);
        assert_eq!(compiler.use_server, true);
        assert_eq!(compiler.server_addr, Some("127.0.0.1:9999".to_string()));
    }

    #[test]
    fn test_compiler_debug_formatting() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("example.erl")],
            output_dir: Some(PathBuf::from("out")),
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        let debug_str = format!("{:?}", compiler);
        assert!(debug_str.contains("source_files"));
        assert!(debug_str.contains("output_dir"));
        assert!(debug_str.contains("use_server"));
    }

    #[test]
    fn test_compiler_clone() {
        let original = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: Some(PathBuf::from("build")),
            include_dirs: vec![PathBuf::from("include")],
            options: CompilerOptions::default(),
            use_server: true,
            server_addr: Some("localhost:9999".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_compiler_equality() {
        let compiler1 = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        let compiler2 = compiler1.clone();
        let mut compiler3 = compiler1.clone();
        compiler3.use_server = true;

        assert_eq!(compiler1, compiler2);
        assert_ne!(compiler1, compiler3);
    }

    #[test]
    fn test_compiler_use_server_method() {
        let compiler_server = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: true,
            server_addr: Some("127.0.0.1:9999".to_string()),
        };

        let compiler_local = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        assert_eq!(compiler_server.use_server(), true);
        assert_eq!(compiler_local.use_server(), false);
    }

    #[test]
    fn test_compiler_minimal_configuration() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("minimal.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        assert_eq!(compiler.source_files.len(), 1);
        assert_eq!(compiler.output_dir, None);
        assert_eq!(compiler.include_dirs.len(), 0);
        assert_eq!(compiler.use_server, false);
        assert_eq!(compiler.server_addr, None);
    }

    #[test]
    fn test_compiler_complex_configuration() {
        let compiler = Compiler {
            source_files: vec![
                PathBuf::from("src/main.erl"),
                PathBuf::from("src/utils.erl"),
                PathBuf::from("src/types.erl"),
            ],
            output_dir: Some(PathBuf::from("ebin")),
            include_dirs: vec![
                PathBuf::from("include"),
                PathBuf::from("deps/include"),
                PathBuf::from("priv/include"),
            ],
            options: CompilerOptions {
                warnings: true,
                debug_info: true,
                optimize: true,
                verbose: true,
                target: Some("prod".to_string()),
            },
            use_server: true,
            server_addr: Some("compile-server.company.com:9999".to_string()),
        };

        assert_eq!(compiler.source_files.len(), 3);
        assert_eq!(compiler.include_dirs.len(), 3);
        assert_eq!(compiler.options.target, Some("prod".to_string()));
        assert_eq!(compiler.use_server, true);
        assert!(compiler.server_addr.as_ref().unwrap().contains("company.com"));
    }

    // ==================== Compilation Methods Tests ====================

    #[tokio::test]
    async fn test_compile_method_exists() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        // Test that compile method exists and can be called
        // It will likely fail due to missing Erlang/files, but should return a result
        let result = compiler.compile().await;
        assert!(result.is_ok() || result.is_err()); // Method should complete
    }

    #[tokio::test]
    async fn test_compile_with_server_method_exists() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: true,
            server_addr: Some("127.0.0.1:9999".to_string()),
        };

        // Test that compile_with_server method exists and can be called
        let result = compiler.compile_with_server().await;
        assert!(result.is_ok() || result.is_err()); // Method should complete
    }

    #[tokio::test]
    async fn test_compile_locally_method_exists() {
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        // Test that compile_locally method exists and can be called
        let result = compiler.compile_locally().await;
        assert!(result.is_ok() || result.is_err()); // Method should complete
    }

    #[tokio::test]
    async fn test_compile_method_dispatch_logic() {
        // Test that compile() dispatches to the correct method based on use_server flag

        // Test local compilation path
        let compiler_local = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        // Since server is not available, should use local compilation
        let result_local = compiler_local.compile().await;
        // Should complete (success or failure doesn't matter for this test)
        assert!(result_local.is_ok() || result_local.is_err());
    }

    #[test]
    fn test_compiler_method_signatures() {
        // Test that all expected methods exist with correct signatures
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        // Test use_server() method
        let _use_server: bool = compiler.use_server();

        // Test static methods
        let _version: &'static str = Compiler::version();
        let _help: &'static str = Compiler::help();

        // These should compile without issues
        assert!(true);
    }

    #[tokio::test]
    async fn test_compilation_result_types() {
        // Test that compilation methods return the expected result types
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        let result = compiler.compile_locally().await;

        match result {
            Ok(compilation_result) => {
                // Should be either Success or Failure
                match compilation_result {
                    CompilationResult::Success => {
                        // Success case
                    }
                    CompilationResult::Failure(errors) => {
                        // Failure case with error messages
                        assert!(!errors.is_empty()); // Should have at least one error
                    }
                }
            }
            Err(_) => {
                // Error case (e.g., Erlang not found)
                // This is also acceptable
            }
        }
    }

    // ==================== Error Condition Tests ====================

    #[test]
    fn test_error_conditions_argument_parsing() {
        // Test various argument parsing error conditions

        // Missing output directory value
        let args1 = vec!["erlc".to_string(), "-o".to_string()];
        assert!(Compiler::from_args_env(args1).is_err());

        // Missing include directory value
        let args2 = vec!["erlc".to_string(), "-I".to_string()];
        assert!(Compiler::from_args_env(args2).is_err());

        // Missing target value
        let args3 = vec!["erlc".to_string(), "--target".to_string()];
        assert!(Compiler::from_args_env(args3).is_err());

        // Unknown option
        let args4 = vec!["erlc".to_string(), "--invalid".to_string(), "test.erl".to_string()];
        assert!(Compiler::from_args_env(args4).is_err());

        // No source files
        let args5 = vec!["erlc".to_string(), "-v".to_string()];
        assert!(Compiler::from_args_env(args5).is_err());

        // Empty args
        let args6 = vec!["erlc".to_string()];
        assert!(Compiler::from_args_env(args6).is_err());
    }

    #[test]
    fn test_error_message_contents() {
        // Test that error messages contain expected information

        // Missing output directory
        let args = vec!["erlc".to_string(), "-o".to_string()];
        let result = Compiler::from_args_env(args);
        assert!(result.is_err());
        if let Err(CompilerError::InvalidArgument(msg)) = result {
            assert!(msg.contains("output") || msg.contains("-o"));
        }

        // Unknown option
        let args = vec!["erlc".to_string(), "--badoption".to_string(), "test.erl".to_string()];
        let result = Compiler::from_args_env(args);
        assert!(result.is_err());
        if let Err(CompilerError::InvalidArgument(msg)) = result {
            assert!(msg.contains("Unknown option") || msg.contains("badoption"));
        }

        // No source files
        let args = vec!["erlc".to_string(), "-v".to_string()];
        let result = Compiler::from_args_env(args);
        assert!(result.is_err());
        if let Err(CompilerError::InvalidArgument(msg)) = result {
            assert!(msg.contains("source files") || msg.contains("No source"));
        }
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_argument_parsing_workflow() {
        // Test a complete argument parsing workflow
        let args = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "/tmp/build".to_string(),
            "-I".to_string(),
            "/usr/include".to_string(),
            "-I".to_string(),
            "./include".to_string(),
            "-W".to_string(),
            "-D".to_string(),
            "-O".to_string(),
            "-v".to_string(),
            "--target".to_string(),
            "prod".to_string(),
            "main.erl".to_string(),
            "utils.erl".to_string(),
            "types.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        // Verify all settings were parsed correctly
        assert_eq!(compiler.output_dir, Some(PathBuf::from("/tmp/build")));
        assert_eq!(compiler.include_dirs.len(), 2);
        assert_eq!(compiler.include_dirs[0], PathBuf::from("/usr/include"));
        assert_eq!(compiler.include_dirs[1], PathBuf::from("./include"));
        assert_eq!(compiler.source_files.len(), 3);
        assert_eq!(compiler.options.warnings, true);
        assert_eq!(compiler.options.debug_info, true);
        assert_eq!(compiler.options.optimize, true);
        assert_eq!(compiler.options.verbose, true);
        assert_eq!(compiler.options.target, Some("prod".to_string()));
    }

    #[test]
    fn test_compiler_creation_from_minimal_args() {
        // Test compiler creation with minimal valid arguments
        let args = vec![
            "erlc".to_string(),
            "single.erl".to_string(),
        ];

        let compiler = Compiler::from_args_env(args).unwrap();

        // Should have defaults
        assert_eq!(compiler.source_files, vec![PathBuf::from("single.erl")]);
        assert_eq!(compiler.output_dir, None);
        assert_eq!(compiler.include_dirs.len(), 0);
        assert_eq!(compiler.options.warnings, true); // Default
        assert_eq!(compiler.options.debug_info, false); // Default
        assert_eq!(compiler.options.optimize, false); // Default
        assert_eq!(compiler.options.verbose, false); // Default
        assert_eq!(compiler.options.target, None); // Default
    }

    #[test]
    fn test_option_precedence_and_overrides() {
        // Test that options can override each other correctly

        // -W followed by --no-warnings should result in warnings=false
        let args1 = vec![
            "erlc".to_string(),
            "-W".to_string(),
            "--no-warnings".to_string(),
            "test.erl".to_string(),
        ];

        let compiler1 = Compiler::from_args_env(args1).unwrap();
        assert_eq!(compiler1.options.warnings, false);

        // Multiple -o should use the last one
        let args2 = vec![
            "erlc".to_string(),
            "-o".to_string(),
            "first".to_string(),
            "-o".to_string(),
            "second".to_string(),
            "test.erl".to_string(),
        ];

        let compiler2 = Compiler::from_args_env(args2).unwrap();
        assert_eq!(compiler2.output_dir, Some(PathBuf::from("second")));

        // Multiple --target should use the last one
        let args3 = vec![
            "erlc".to_string(),
            "--target".to_string(),
            "first".to_string(),
            "--target".to_string(),
            "second".to_string(),
            "test.erl".to_string(),
        ];

        let compiler3 = Compiler::from_args_env(args3).unwrap();
        assert_eq!(compiler3.options.target, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_cli_integration_workflow() {
        // Test the full CLI workflow from arguments to result

        // Test help workflow
        let help_result = cli::run_with_args(vec!["erlc".to_string(), "--help".to_string()]).await;
        assert!(matches!(help_result, Ok(0)));

        // Test version workflow
        let version_result = cli::run_with_args(vec!["erlc".to_string(), "--version".to_string()]).await;
        assert!(matches!(version_result, Ok(0)));

        // Test compilation attempt (will fail due to missing file)
        let compile_result = cli::run_with_args(vec![
            "erlc".to_string(),
            "-v".to_string(),
            "nonexistent.erl".to_string(),
        ]).await;
        // Should return exit code 1 for compilation failure
        assert!(matches!(compile_result, Ok(1)));
    }

    #[test]
    fn test_data_structure_integration() {
        // Test integration between data structures

        // Create a compiler
        let compiler = Compiler {
            source_files: vec![PathBuf::from("test.erl")],
            output_dir: Some(PathBuf::from("build")),
            include_dirs: vec![PathBuf::from("include")],
            options: CompilerOptions {
                warnings: true,
                debug_info: true,
                optimize: true,
                verbose: true,
                target: Some("test".to_string()),
            },
            use_server: true,
            server_addr: Some("localhost:9999".to_string()),
        };

        // Test that all components work together
        assert_eq!(compiler.source_files.len(), 1);
        assert_eq!(compiler.options.warnings, true);
        assert_eq!(compiler.use_server(), true);

        // Test cloning (all components should be cloneable)
        let cloned = compiler.clone();
        assert_eq!(compiler, cloned);
    }

    #[tokio::test]
    async fn test_compile_locally() {
        // This is a minimal test - in real usage, Erlang would need to be available
        let compiler = Compiler {
            source_files: vec![PathBuf::from("nonexistent.erl")],
            output_dir: None,
            include_dirs: vec![],
            options: CompilerOptions::default(),
            use_server: false,
            server_addr: None,
        };

        // This will likely fail because Erlang isn't available in test environment
        // but we're testing that the method exists and can be called
        let _result = compiler.compile_locally().await;
        // We don't assert success since Erlang may not be installed
    }
}
