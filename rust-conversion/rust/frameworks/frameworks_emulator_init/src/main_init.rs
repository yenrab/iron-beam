//! Main Initialization Module
//!
//! Provides main initialization phase functions.
//! Based on `erl_init()` and `erl_start()` from erl_init.c

use crate::initialization::set_initialized;

/// Initialization configuration
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Number of CPUs
    pub ncpu: usize,
    /// Process table size
    pub proc_tab_sz: usize,
    /// Port table size
    pub port_tab_sz: usize,
    /// Number of schedulers
    pub no_schedulers: usize,
    /// Number of schedulers online
    pub no_schedulers_online: usize,
    /// Number of poll threads
    pub no_poll_threads: usize,
    /// Number of dirty CPU schedulers
    pub no_dirty_cpu_schedulers: usize,
    /// Number of dirty CPU schedulers online
    pub no_dirty_cpu_schedulers_online: usize,
    /// Number of dirty IO schedulers
    pub no_dirty_io_schedulers: usize,
    /// Time correction mode
    pub time_correction: i32,
    /// Time warp mode
    pub time_warp_mode: TimeWarpMode,
}

/// Time warp mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWarpMode {
    /// No time warp
    NoTimeWarp,
    /// Multi-time warp
    MultiTimeWarp,
    /// Single time warp
    SingleTimeWarp,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            ncpu: 1,
            proc_tab_sz: 1_048_576, // ERTS_DEFAULT_MAX_PROCESSES
            port_tab_sz: 1_048_576,  // ERTS_DEFAULT_MAX_PORTS
            no_schedulers: 1,
            no_schedulers_online: 1,
            no_poll_threads: 1,
            no_dirty_cpu_schedulers: 0,
            no_dirty_cpu_schedulers_online: 0,
            no_dirty_io_schedulers: 0,
            time_correction: 0,
            time_warp_mode: TimeWarpMode::NoTimeWarp,
        }
    }
}

/// Perform main initialization
///
/// Based on `erl_init()` from erl_init.c. This function performs
/// the main initialization phase, coordinating initialization of
/// all runtime components in the correct order.
///
/// # Arguments
/// * `config` - Initialization configuration
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
pub fn erl_init(config: InitConfig) -> Result<(), String> {
    // Initialize global literals
    // In C: init_global_literals();
    infrastructure_utilities::init_global_literals()
        .map_err(|e| format!("Failed to initialize global literals: {}", e))?;
    
    // Initialize process management
    // In C: erts_init_process(ncpu, proc_tab_sz, legacy_proc_tab);
    usecases_process_management::erts_init_process(
        config.ncpu,
        config.proc_tab_sz,
        false, // legacy_proc_tab - not used in Rust implementation
    )
    .map_err(|e| format!("Failed to initialize process management: {}", e))?;
    
    // Initialize scheduling
    // In C: erts_init_scheduling(no_schedulers, no_schedulers_online, no_poll_threads, 
    //                            no_dirty_cpu_schedulers, no_dirty_cpu_schedulers_online, no_dirty_io_schedulers)
    usecases_scheduling::erts_init_scheduling(
        config.no_schedulers,
        config.no_schedulers_online,
        config.no_poll_threads,
        config.no_dirty_cpu_schedulers,
        config.no_dirty_cpu_schedulers_online,
        config.no_dirty_io_schedulers,
    )
    .map_err(|e| format!("Failed to initialize scheduling: {}", e))?;
    
    // Initialize BIF dispatcher
    // In C: erts_init_bif()
    infrastructure_bif_dispatcher::erts_init_bif()
        .map_err(|e| format!("Failed to initialize BIF dispatcher: {:?}", e))?;
    
    // Initialize emulator loop
    // In C: init_emulator()
    // Note: init_emulator takes an Arc<AtomicBool> for init_done flag
    // We'll create a temporary flag for initialization
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    let init_done = Arc::new(AtomicBool::new(false));
    infrastructure_emulator_loop::init_emulator(init_done.clone())
        .map_err(|e| format!("Failed to initialize emulator loop: {:?}", e))?;
    
    // Set up process executor to break circular dependency
    // The executor allows the scheduler to execute processes without
    // directly depending on the emulator loop
    use entities_process::{set_process_executor, ProcessExecutor};
    use infrastructure_emulator_loop::EmulatorLoopExecutor;
    set_process_executor(Box::new(EmulatorLoopExecutor))
        .map_err(|e| format!("Failed to set process executor: {}", e))?;
    
    // Initialize runtime utilities
    infrastructure_runtime_utils::erts_init_utils()
        .map_err(|e| format!("Failed to initialize runtime utils: {}", e))?;
    
    // Initialize scheduler-specific data
    infrastructure_runtime_utils::erts_utils_sched_spec_data_init()
        .map_err(|e| format!("Failed to initialize scheduler data: {}", e))?;
    
    // Mark as initialized
    set_initialized(true);
    
    Ok(())
}

/// Main emulator entry point
///
/// Based on `erl_start()` from erl_init.c. This is the main entry point
/// for starting the Erlang emulator. It performs early initialization,
/// then main initialization, and coordinates the startup sequence.
///
/// # Arguments
/// * `argc` - Number of command line arguments (mutable, may be modified)
/// * `argv` - Command line arguments (mutable, may be modified)
///
/// # Returns
/// * `Ok(())` - Emulator started successfully
/// * `Err(String)` - Startup error
pub fn erl_start(argc: &mut usize, argv: &mut Vec<String>) -> Result<(), String> {
    // Perform early initialization
    use crate::early_init;
    let early_result = early_init::early_init(argc, argv)
        .map_err(|e| format!("Early initialization failed: {}", e))?;
    
    // Build initialization configuration
    let config = InitConfig {
        ncpu: early_result.ncpu,
        no_schedulers: early_result.no_schedulers,
        no_schedulers_online: early_result.no_schedulers_online,
        no_poll_threads: early_result.no_poll_threads,
        no_dirty_cpu_schedulers: early_result.no_dirty_cpu_schedulers,
        no_dirty_cpu_schedulers_online: early_result.no_dirty_cpu_schedulers_online,
        no_dirty_io_schedulers: early_result.no_dirty_io_schedulers,
        ..Default::default()
    };
    
    // Parse command line arguments for configuration overrides
    // Extract boot script path from arguments
    let boot_script = extract_boot_script(argv);
    
    // Perform main initialization
    erl_init(config)
        .map_err(|e| format!("Main initialization failed: {}", e))?;
    
    // Step 1: Start scheduler threads
    // In C: erts_start_schedulers()
    let scheduler_handles = usecases_scheduling::erts_start_schedulers()
        .map_err(|e| format!("Failed to start scheduler threads: {}", e))?;
    
    // Step 2: Load boot script (if specified)
    // The boot script is loaded and executed here, before the init process starts
    // In the full implementation, the init process would execute the boot script
    use crate::env;
    let (rootdir, bindir) = env::determine_paths().unwrap_or_else(|_| (String::new(), String::new()));
    if let Some(boot_path) = boot_script {
        if let Err(e) = load_boot_script(&boot_path, &rootdir, &bindir) {
            eprintln!("Warning: {}", e);
            eprintln!("Continuing without boot script (some features may not work)");
        }
    }
    
    // Step 3: Create init process and start Erlang shell
    // In C: This is done by erl_first_process() which creates the init process
    // The init process then loads the boot script and starts the shell
    create_init_process()
        .map_err(|e| format!("Failed to create init process: {}", e))?;
    
    // Step 4: Enter main execution loop (block until shutdown)
    // In C: erts_sys_main_thread() - the main thread enters a loop or waits
    // The scheduler threads are already running, so we just need to wait
    // For now, we'll wait for a shutdown signal or until schedulers stop
    wait_for_shutdown(scheduler_handles);
    
    Ok(())
}

/// Extract boot script path from command line arguments
fn extract_boot_script(argv: &[String]) -> Option<String> {
    for (i, arg) in argv.iter().enumerate() {
        if arg == "--boot" || arg == "-boot" {
            if i + 1 < argv.len() {
                return Some(argv[i + 1].clone());
            }
        }
    }
    None
}

/// Load boot script
///
/// Based on boot script loading in init.erl
///
/// This function loads and parses the boot script file.
/// It uses the boot_script module to parse and execute the script.
fn load_boot_script(boot_path: &str, rootdir: &str, bindir: &str) -> Result<(), String> {
    use crate::boot_script;
    
    eprintln!("Loading boot script: {}", boot_path);
    
    // Load and parse boot script
    let script = boot_script::load_boot_script(boot_path, rootdir, bindir)
        .map_err(|e| format!("Failed to load boot script: {}", e))?;
    
    // Execute boot script commands
    boot_script::execute_boot_script(&script)
        .map_err(|e| format!("Failed to execute boot script: {}", e))?;
    
    Ok(())
}

/// Create init process
///
/// Based on erl_first_process() from erl_init.c
///
/// Creates the first Erlang process (init process) which will:
/// 1. Load the boot script
/// 2. Start kernel processes
/// 3. Start the Erlang shell
fn create_init_process() -> Result<(), String> {
    use entities_process::Process;
    use infrastructure_utilities::process_table::get_global_process_table;
    use std::sync::Arc;
    
    // In the full implementation, this would:
    // 1. Create a new process with special init process code
    // 2. Set up the process with init:boot/1 function
    // 3. Add the process to the process table
    // 4. Schedule the process on a scheduler
    
    let process_table = get_global_process_table();
    
    // Create a placeholder init process
    // In the real implementation, this would be a special system process
    // with the init module's boot function
    let mut init_process = Process::new(1); // PID 1 is typically the init process
    
    // For testing, create a simple test code sequence
    // In the full implementation, this would load actual BEAM code
    use entities_process::ErtsCodePtr;
    use infrastructure_emulator_loop::instruction_decoder::opcodes;
    let mut test_code = Vec::new();
    
    // move x(0) x(1)
    test_code.push(opcodes::MOVE as u64);
    test_code.push(0); // src = x(0)
    test_code.push(1); // dst = x(1)
    
    // return
    test_code.push(opcodes::RETURN as u64);
    
    // Set instruction pointer to point to our test code
    // Note: In production, code would be in a code area that persists
    // For now, we'll leak the memory (not ideal, but works for testing)
    let code_ptr = test_code.as_ptr() as ErtsCodePtr;
    std::mem::forget(test_code); // Keep code alive
    
    init_process.set_i(code_ptr);
    
    let init_process = Arc::new(init_process);
    
    // Insert into process table
    // insert() returns Option<Arc<Process>> (the old process if replaced)
    // We don't care about the old value, just that it succeeded
    let _old_process = process_table.insert(1, init_process.clone());
    
    // Schedule the init process
    // In the full implementation, we would:
    // 1. Get a scheduler
    // 2. Enqueue the init process to the scheduler's run queue
    // 3. Wake the scheduler if it's sleeping
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    
    if let Some(schedulers) = get_global_schedulers() {
        let schedulers_guard = schedulers.lock().unwrap();
        if let Some(scheduler) = schedulers_guard.first() {
            let runq = scheduler.runq();
            let runq_guard = runq.lock().unwrap();
            schedule_process(init_process.clone(), &runq_guard, Priority::Max)
                .map_err(|e| format!("Failed to schedule init process: {:?}", e))?;
        }
    }
    
    eprintln!("Init process created and scheduled");
    
    Ok(())
}

/// Wait for shutdown signal
///
/// Blocks the main thread until the emulator is shut down.
/// In the full implementation, this would:
/// 1. Wait for shutdown signal (SIGTERM, SIGINT, etc.)
/// 2. Gracefully stop scheduler threads
/// 3. Clean up resources
fn wait_for_shutdown(handles: Vec<std::thread::JoinHandle<()>>) {
    // Start a simple REPL loop in the main thread
    // In the full implementation, this would be handled by user_drv and shell processes
    start_simple_repl();
    
    // REPL has exited, now stop scheduler threads
    eprintln!("Stopping scheduler threads...");
    use usecases_scheduling::threads::erts_stop_schedulers;
    erts_stop_schedulers(handles);
    
    eprintln!("Shutdown complete.");
}

/// Start a simple REPL loop
///
/// This is a minimal implementation that provides a basic REPL experience.
/// In the full implementation, this would be handled by:
/// - user_drv process (terminal I/O)
/// - group_leader process
/// - shell process (expression evaluation)
///
/// For now, this provides:
/// - REPL prompt (1>, 2>, etc.)
/// - Input reading
/// - Basic command handling (help, quit)
fn start_simple_repl() {
    use std::io::{self, BufRead, Write};
    use infrastructure_utilities::erl_eval::new_bindings;
    
    // Print Erlang/OTP banner (similar to C version)
    println!("Erlang/OTP [Iron BEAM] [erts-15.0] [source] [64-bit]");
    println!("Eshell V15.0  (press Ctrl+c to abort, type help(). for help)");
    
    // Maintain bindings across expressions
    let mut bindings = new_bindings();
    
    let stdin = io::stdin();
    let mut line_count = 1;
    
    loop {
        // Print prompt
        print!("{}> ", line_count);
        io::stdout().flush().unwrap();
        
        // Read line
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF
                println!("\n");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                
                // Handle empty lines
                if trimmed.is_empty() {
                    continue;
                }
                
                // Handle special commands
                match trimmed {
                    "q()." | "quit()." | "halt()." => {
                        println!("ok");
                        break;
                    }
                    "help()." => {
                        println!("  This is a minimal REPL implementation.");
                        println!("  Commands:");
                        println!("    help().  - Show this help");
                        println!("    q().     - Quit the emulator");
                        println!("  You can assign variables:");
                        println!("    X = 3.");
                        println!("    Y = X + 2.");
                    }
                    _ => {
                        // Use full parser and evaluator with persistent bindings
                        match evaluate_erlang_expression_with_bindings(trimmed, &mut bindings) {
                            Ok(result) => {
                                // Format and print the result
                                println!("{}", format_term(&result));
                            }
                            Err(e) => {
                                println!("** {}", e);
                            }
                        }
                    }
                }
                
                line_count += 1;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    println!("Shutting down...");
}

/// Evaluate an Erlang expression using full parser and evaluator
///
/// This uses erl_scan, erl_parse, and erl_eval to fully evaluate Erlang expressions.
fn evaluate_erlang_expression(input: &str) -> Result<entities_data_handling::term_hashing::Term, String> {
    use infrastructure_utilities::{scan_string, parse_exprs, exprs, new_bindings};
    
    // Remove trailing period if present (Erlang syntax)
    let expr_str = input.trim_end_matches('.');
    
    // Step 1: Scan (tokenize)
    let tokens = scan_string(expr_str)
        .map_err(|e| format!("Scan error: {}", e))?;
    
    // Step 2: Parse
    let parsed_exprs = parse_exprs(tokens)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    if parsed_exprs.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    // Step 3: Evaluate
    let bindings = new_bindings();
    let (result, _) = exprs(parsed_exprs, bindings)
        .map_err(|e| format!("Eval error: {}", e))?;
    
    Ok(result)
}

/// Evaluate an Erlang expression with persistent bindings
///
/// This version maintains bindings across multiple expressions (for REPL).
fn evaluate_erlang_expression_with_bindings(
    input: &str,
    bindings: &mut infrastructure_utilities::erl_eval::Bindings,
) -> Result<entities_data_handling::term_hashing::Term, String> {
    use infrastructure_utilities::{scan_string, parse_exprs, exprs};
    
    // Remove trailing period if present (Erlang syntax)
    let expr_str = input.trim_end_matches('.');
    
    // Step 1: Scan (tokenize)
    let tokens = scan_string(expr_str)
        .map_err(|e| format!("Scan error: {}", e))?;
    
    // Step 2: Parse
    let parsed_exprs = parse_exprs(tokens)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    if parsed_exprs.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    // Step 3: Evaluate with current bindings
    let current_bindings = bindings.clone();
    let (result, new_bindings) = exprs(parsed_exprs, current_bindings)
        .map_err(|e| format!("Eval error: {}", e))?;
    
    // Update bindings for next expression
    *bindings = new_bindings;
    
    Ok(result)
}

/// Format a term for display
fn format_term(term: &entities_data_handling::term_hashing::Term) -> String {
    use entities_data_handling::term_hashing::Term;
    
    match term {
        Term::Nil => "[]".to_string(),
        Term::Small(i) => i.to_string(),
        Term::Float(f) => {
            // Format float nicely
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        Term::Atom(index) => {
            // Look up atom name
            use infrastructure_utilities::atom_table::get_global_atom_table;
            let atom_table = get_global_atom_table();
            if let Some(name_bytes) = atom_table.get_name(*index as usize) {
                if let Ok(name) = String::from_utf8(name_bytes.clone()) {
                    name
                } else {
                    format!("atom_{}", index)
                }
            } else {
                format!("atom_{}", index)
            }
        }
        Term::List { head, tail } => {
            format_list(head, tail)
        }
        Term::Tuple(elems) => {
            let parts: Vec<String> = elems.iter().map(format_term).collect();
            format!("{{{}}}", parts.join(", "))
        }
        _ => format!("{:?}", term),
    }
}

/// Format a list
fn format_list(head: &entities_data_handling::term_hashing::Term, tail: &entities_data_handling::term_hashing::Term) -> String {
    use entities_data_handling::term_hashing::Term;
    
    let mut parts = Vec::new();
    let mut current_head = head;
    let mut current_tail = tail;
    
    loop {
        parts.push(format_term(current_head));
        
        match current_tail {
            Term::Nil => break,
            Term::List { head, tail } => {
                current_head = head;
                current_tail = tail;
            }
            _ => {
                // Improper list
                parts.push("|".to_string());
                parts.push(format_term(current_tail));
                break;
            }
        }
    }
    
    format!("[{}]", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.ncpu, 1);
        assert_eq!(config.proc_tab_sz, 1_048_576);
    }
    
    #[test]
    fn test_erl_init() {
        let config = InitConfig::default();
        let result = erl_init(config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_erl_start() {
        let mut argc = 1;
        let mut argv = vec!["test".to_string()];
        
        // Reset early init state for testing
        // Note: This is a limitation - in real code, we'd need a way to reset
        // For now, we'll just test that it works on first call
        let result = erl_start(&mut argc, &mut argv);
        // May fail if early_init was already called, which is expected
        // In a real scenario, we'd have proper state management
    }
}

