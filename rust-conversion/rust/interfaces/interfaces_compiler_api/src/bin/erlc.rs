//! Erlang Compiler (erlc) Binary
//!
//! Command-line interface for the Rust Erlang compiler.
//! This implements the standard erlc options and behavior.

use clap::Parser;
use interfaces_compiler_api::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Erlang compiler command-line arguments
#[derive(Parser, Debug)]
#[command(name = "erlc")]
#[command(about = "Erlang Compiler")]
#[command(version)]
#[command(author)]
struct ErlcArgs {
    /// Input Erlang source files (.erl)
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output directory for .beam files
    #[arg(short = 'o', long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Add include directory to search path
    #[arg(short = 'I', long, value_name = "DIR")]
    include_dirs: Vec<PathBuf>,

    /// Add path to search for modules
    #[arg(long = "path_add", value_name = "DIR")]
    path_add: Vec<PathBuf>,

    /// Enable warnings
    #[arg(short = 'W', long)]
    warnings: bool,

    /// Warning level (0-2)
    #[arg(long, value_name = "LEVEL")]
    warning_level: Option<u8>,

    /// Enable debug info
    #[arg(short = 'g', long)]
    debug_info: bool,

    /// Compile only (don't load)
    #[arg(short = 'c', long)]
    compile_only: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Suppress progress reports
    #[arg(short = 's', long)]
    silent: bool,

    /// Treat warnings as errors
    #[arg(long)]
    warnings_as_errors: bool,

    /// Time compilation
    #[arg(long)]
    time: bool,

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ErlcArgs::parse();


    // Initialize compiler API
    let compiler = CompilerAPI::new();

    let mut success_count = 0;
    let mut error_count = 0;
    let total_files = args.files.len();

    // Create output directory if specified
    if let Some(ref out_dir) = args.output_dir {
        fs::create_dir_all(out_dir)?;
    }

    for input_file in &args.files {
        if !args.silent {
            eprintln!("Compiling {}", input_file.display());
        }

        match compile_file(&compiler, input_file, &args).await {
            Ok(_) => {
                success_count += 1;
                if args.verbose {
                    eprintln!("✓ Compiled {}", input_file.display());
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("✗ Failed to compile {}: {}", input_file.display(), e);
                if args.warnings_as_errors {
                    std::process::exit(1);
                }
            }
        }
    }

    if !args.silent {
        eprintln!("Compiled {} files: {} succeeded, {} failed",
                 total_files, success_count, error_count);
    }

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Compile a single Erlang source file
async fn compile_file(
    compiler: &CompilerAPI,
    input_path: &Path,
    args: &ErlcArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read source file
    let source_code = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read {}: {}", input_path.display(), e))?;

    // Extract module name from filename
    let module_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Invalid filename: {}", input_path.display()))?;

    // Compile the source
    let output = compiler.compile_source(module_name, &source_code).await
        .map_err(|e| format!("Compilation failed: {}", e))?;

    // Handle compilation result
    if !output.success {
        let error_msg = if output.errors.is_empty() {
            "Unknown compilation error".to_string()
        } else {
            output.errors.join("\n")
        };
        return Err(error_msg.into());
    }

    // Report warnings if enabled
    if args.warnings && !output.warnings.is_empty() {
        for warning in &output.warnings {
            eprintln!("Warning: {}:{}:{}: {}",
                     input_path.display(),
                     warning.line,
                     warning.column,
                     warning.message);
        }
    }

    // Write bytecode to output file
    if let Some(bytecode) = output.bytecode {
        let output_path = determine_output_path(input_path, module_name, &args.output_dir);
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, bytecode)
            .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;

        if args.verbose {
            eprintln!("  → {}", output_path.display());
        }
    }

    // Report timing if requested
    if args.time {
        eprintln!("Compilation time: {}ms", output.compilation_time_ms);
    }

    Ok(())
}

/// Determine the output path for a compiled module
fn determine_output_path(input_path: &Path, module_name: &str, output_dir: &Option<PathBuf>) -> PathBuf {
    let beam_filename = format!("{}.beam", module_name);

    if let Some(ref out_dir) = output_dir {
        out_dir.join(beam_filename)
    } else {
        // Default: same directory as input file
        input_path.with_file_name(beam_filename)
    }
}
