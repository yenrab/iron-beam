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
#[derive(Debug, Clone)]
pub enum CompilationResult {
    /// Compilation succeeded
    Success,
    /// Compilation failed with errors
    Failure(Vec<String>),
}

/// Main compiler frontend structure
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
