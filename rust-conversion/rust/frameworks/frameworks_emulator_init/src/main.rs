//! Erlang/OTP Emulator Binary Entry Point
//!
//! This is the main entry point for the Erlang/OTP emulator, replacing both
//! erlexec (C) and erl_main.c (C) with a pure Rust implementation.
//!
//! This binary integrates all erlexec functionality:
//! - Command-line argument parsing
//! - Environment variable setup (ROOTDIR, BINDIR, PROGNAME, PATH)
//! - epmd daemon management
//! - Boot/config path resolution
//! - Signal stack initialization
//! - Direct call to erl_start() (no process replacement)

use std::process;

mod args;
mod env;
mod epmd;
mod signal_stack;

use clap::Parser;
use args::EmulatorArgs;
use env::{determine_paths, manipulate_path, set_env_vars};
use epmd::start_epmd_daemon;
use signal_stack::sys_init_signal_stack;

fn main() {
    eprintln!("[DEBUG] in main");
    // Initialize signal stack before any threads are created
    // This is critical for scheduler thread safety
    unsafe {
        if let Err(e) = sys_init_signal_stack() {
            eprintln!("Warning: Failed to initialize signal stack: {}", e);
            // Continue anyway - signal stack is important but not fatal
        }
    }
    eprintln!("[DEBUG] after signal stack initialization");
    // Parse command-line arguments (replaces erlexec argument processing)
    let args = EmulatorArgs::parse();
    eprintln!("[DEBUG] after args parsing");
    // Handle special modes (must exit early)
    if args.emu_name_exit {
        println!("beam");
        process::exit(0);
    }
    eprintln!("[DEBUG] after emu_name_exit");
    if args.emu_args_exit {
        // Print all arguments
        for arg in std::env::args().skip(1) {
            println!("{}", arg);
        }
        process::exit(0);
    }
    eprintln!("[DEBUG] after emu_args_exit");
    if args.emu_qouted_cmd_exit {
        // Print quoted command line
        print!("\"beam\" ");
        for arg in std::env::args().skip(1) {
            print!("\"{}\" ", arg);
        }
        println!();
        process::exit(0);
    }
    eprintln!("[DEBUG] after emu_qouted_cmd_exit");
    // Validate argument combinations
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
    eprintln!("[DEBUG] after validate");
    // Determine rootdir and bindir
    let (rootdir, bindir) = match determine_paths() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error: Failed to determine paths: {}", e);
            process::exit(1);
        }
    };
    eprintln!("[DEBUG] after determine_paths");
    // Set environment variables (replaces erlexec environment setup)
    set_env_vars(&rootdir, &bindir, "beam");
    manipulate_path(&bindir, &rootdir);
    eprintln!("[DEBUG] after set_env_vars");
    // Start epmd daemon if needed (replaces erlexec epmd management)
    if args.should_start_epmd() {
        if let Err(e) = start_epmd_daemon(&bindir, args.epmd.as_deref()) {
            eprintln!("Warning: Failed to start epmd daemon: {}", e);
            // Continue anyway - epmd may already be running
        }
    }
    eprintln!("[DEBUG] after start_epmd_daemon");
    // Build arguments for erl_start (replaces erlexec argument construction)
    let mut emulator_args = args.build_emulator_args(&rootdir, &bindir);
    let mut argc = emulator_args.len();
    eprintln!("[DEBUG] after build_emulator_args, argc: {}", argc);
    // Call Rust erl_start directly (no execv, no process replacement)
    // This is the key difference from C: we call erl_start() directly instead
    // of using execv() to launch a separate binary
    // erl_start() will now:
    // 1. Start scheduler threads
    // 2. Load boot script
    // 3. Create init process
    // 4. Enter main execution loop (blocks until shutdown)
    eprintln!("[DEBUG] about to call erl_start");
    match frameworks_emulator_init::main_init::erl_start(&mut argc, &mut emulator_args) {
        Ok(()) => {
            // erl_start() returns after shutdown is complete
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Failed to start emulator: {}", e);
            process::exit(1);
        }
    }
    eprintln!("[DEBUG] after erl_start");
}

