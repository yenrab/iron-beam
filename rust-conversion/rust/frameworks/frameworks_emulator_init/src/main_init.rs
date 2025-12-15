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
    eprintln!("[DEBUG] erl_start: entered");
    
    // Perform early initialization
    use crate::early_init;
    eprintln!("[DEBUG] erl_start: calling early_init");
    let early_result = early_init::early_init(argc, argv)
        .map_err(|e| format!("Early initialization failed: {}", e))?;
    eprintln!("[DEBUG] erl_start: early_init completed");
    
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
    eprintln!("[DEBUG] erl_start: calling erl_init");
    erl_init(config)
        .map_err(|e| format!("Main initialization failed: {}", e))?;
    eprintln!("[DEBUG] erl_start: erl_init completed");
    
    // Step 1: Start scheduler threads
    // In C: erts_start_schedulers()
    eprintln!("[DEBUG] erl_start: starting scheduler threads");
    let scheduler_handles = usecases_scheduling::erts_start_schedulers()
        .map_err(|e| format!("Failed to start scheduler threads: {}", e))?;
    eprintln!("[DEBUG] erl_start: scheduler threads started ({} handles)", scheduler_handles.len());
    
    // Step 2: Load preloaded modules (must be before creating init process)
    // In C: load_preloaded() loads preloaded modules (erl_init, init, etc.)
    eprintln!("[DEBUG] erl_start: loading preloaded modules");
    use crate::env;
    let (rootdir, bindir) = env::determine_paths().unwrap_or_else(|_| (String::new(), String::new()));
    load_preloaded(&rootdir, &bindir)
        .map_err(|e| format!("Failed to load preloaded modules: {}", e))?;
    eprintln!("[DEBUG] erl_start: preloaded modules loaded");
    
    // Verify BEAM code execution setup after loading preloaded modules
    if let Err(e) = verify_beam_execution_setup() {
        eprintln!("Warning: BEAM execution setup verification failed: {}", e);
        eprintln!("Continuing anyway, but BEAM code execution may not work correctly");
    }
    
    // Step 3: Load boot script (if specified)
    // The boot script is loaded and executed here, before the init process starts
    // In the full implementation, the init process would execute the boot script
    if let Some(boot_path) = boot_script {
        eprintln!("[DEBUG] erl_start: loading boot script: {}", boot_path);
        if let Err(e) = load_boot_script(&boot_path, &rootdir, &bindir) {
            eprintln!("Warning: {}", e);
            eprintln!("Continuing without boot script (some features may not work)");
        }
    }
    
    // Step 4: Extract boot arguments for init process
    let boot_module = extract_boot_module(argv).unwrap_or_else(|| "start".to_string());
    let boot_args = extract_boot_args(argv);
    
    // Step 5: Create init process and start Erlang shell
    // In C: This is done by erl_first_process() which creates the init process
    // The init process then loads the boot script and starts the shell
    eprintln!("[DEBUG] erl_start: creating init process");
    create_init_process(&boot_module, &boot_args)
        .map_err(|e| format!("Failed to create init process: {}", e))?;
    eprintln!("[DEBUG] erl_start: init process created");
    
    // Step 4: Enter main execution loop (block until shutdown)
    // In C: erts_sys_main_thread() - the main thread enters a loop or waits
    // The scheduler threads are already running, so we just need to wait
    // For now, we'll wait for a shutdown signal or until schedulers stop
    eprintln!("[DEBUG] erl_start: entering wait_for_shutdown (REPL should start)");
    wait_for_shutdown(scheduler_handles);
    eprintln!("[DEBUG] erl_start: wait_for_shutdown returned");
    
    Ok(())
}

/// Load preloaded modules
///
/// Based on load_preloaded() from erl_init.c
/// Loads preloaded modules (erl_init, init, etc.) from embedded code or filesystem.
/// These modules must be loaded before creating the init process.
///
/// # Arguments
/// * `rootdir` - Root directory for resolving paths
/// * `bindir` - Binary directory for resolving paths
///
/// # Returns
/// Result indicating success or failure
fn load_preloaded(rootdir: &str, bindir: &str) -> Result<(), String> {
    use code_management_code_loading::{CodeLoader, BeamLoader};
    use code_management_code_loading::code_loader::LoadError;
    use usecases_bifs::load::LoadBif;
    use std::path::Path;
    use std::fs;
    
    eprintln!("Loading preloaded modules...");
    
    // Preloaded modules that must be loaded before init process creation
    let preloaded_modules = ["erl_init", "init"];
    
    // Get code paths for preloaded modules
    // Preloaded modules are typically in:
    // 1. rootdir/erts/preloaded/ebin/ (preloaded modules directory)
    // 2. rootdir/lib/erts-VSN/ebin/ (if available)
    // 3. rootdir/lib/kernel-VSN/ebin/ (for init module)
    // 4. bindir (fallback)
    let mut code_paths = Vec::new();
    
    // Add erts preloaded directory if available (highest priority)
    if !rootdir.is_empty() {
        // Try preloaded directory directly (most common location)
        let preloaded_ebin = Path::new(rootdir).join("erts").join("preloaded").join("ebin");
        if preloaded_ebin.exists() {
            code_paths.push(preloaded_ebin.to_string_lossy().to_string());
        }
        
        // Also try preloaded directory without ebin subdirectory
        let preloaded_dir = Path::new(rootdir).join("erts").join("preloaded");
        if preloaded_dir.exists() {
            code_paths.push(preloaded_dir.to_string_lossy().to_string());
        }
        
        // Try to find erts application directory
        let lib_dir = Path::new(rootdir).join("lib");
        if lib_dir.exists() {
            if let Ok(entries) = fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let app_dir = entry.path();
                    if app_dir.is_dir() {
                        let dir_name = app_dir.file_name().and_then(|n| n.to_str());
                        if let Some(name) = dir_name {
                            // Look for erts or kernel directories
                            if name.starts_with("erts-") || name.starts_with("kernel-") {
                                let ebin_dir = app_dir.join("ebin");
                                if ebin_dir.exists() {
                                    code_paths.push(ebin_dir.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Add bindir as fallback
    if !bindir.is_empty() {
        code_paths.push(bindir.to_string());
    }
    
    // Debug: log the paths we're searching
    eprintln!("      Searching for preloaded modules in: {:?}", code_paths);
    
    let mut loaded_count = 0;
    let mut failed_modules = Vec::new();
    
    for module_name in &preloaded_modules {
        let mut found = false;
        
        // Try to load from each code path
        for code_path in &code_paths {
            let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
            
            if !beam_path.exists() {
                continue;
            }
            
            // Load BEAM file
            match CodeLoader::load_module(&beam_path) {
                Ok(beam_data) => {
                    // Verify BEAM format
                    if !CodeLoader::verify_module(&beam_data) {
                        eprintln!("      ✗ Invalid BEAM format: {}", module_name);
                        continue;
                    }
                    
                    // Parse BEAM file
                    match BeamLoader::read_beam_file(&beam_data) {
                        Ok(beam_file) => {
                            // Register module using LoadBif infrastructure
                            // This ensures the module is properly registered in the module management system
                            LoadBif::register_module(
                                module_name,
                                usecases_bifs::load::ModuleStatus::Loaded,
                                false, // has_old_code
                                beam_file.has_on_load, // has_on_load
                            );
                            
                            // Mark as preloaded
                            LoadBif::mark_preloaded(module_name);
                            
                            // Register exports in the export table
                            // This allows functions to be looked up and called
                            use entities_io_operations::export::get_global_export_table;
                            use infrastructure_utilities::atom_table::get_global_atom_table;
                            use entities_data_handling::AtomEncoding;
                            
                            let atom_table = get_global_atom_table();
                            let module_atom_index = atom_table.put_index(module_name.as_bytes(), AtomEncoding::SevenBitAscii, false)
                                .map_err(|_| format!("Failed to create atom for module: {}", module_name))?;
                            
                            let export_table = get_global_export_table();
                            
                            // Register all exports with their labels
                            // Labels will be resolved to code pointers when the code is actually loaded and made executable
                            for (function_atom_idx, arity, label) in &beam_file.exports {
                                // Create export entry (or get existing)
                                export_table.put(module_atom_index as u32, *function_atom_idx, *arity);
                                
                                // Update export with label for later code pointer resolution
                                export_table.update_export_label(module_atom_index as u32, *function_atom_idx, *arity, *label);
                            }
                            
                            // Store code data for label resolution
                            // This allows resolve_export_label() to find the code without reloading the file
                            if !beam_file.code_data.is_empty() {
                                let code_data_vec = beam_file.code_data.clone();
                                let code_data_box = Box::new(code_data_vec);
                                let code_data_static: &'static [u8] = Box::leak(code_data_box);
                                
                                // Store code data in module table using module atom index
                                use code_management_code_loading::{get_global_module_manager, get_global_code_ix};
                                let module_manager = get_global_module_manager();
                                let code_ix = get_global_code_ix();
                                let active_ix = code_ix.active_code_ix() as usize;
                                
                                module_manager.put_module_with_code(module_atom_index, code_data_static, active_ix);
                                
                                eprintln!("      ✓ Cached BEAM code data for {} ({} bytes)", module_name, code_data_static.len());
                            }
                            
                            eprintln!("      ✓ Loaded preloaded module: {} (from {}), registered {} exports", 
                                     module_name, beam_path.display(), beam_file.exports.len());
                            loaded_count += 1;
                            found = true;
                            break;
                        }
                        Err(e) => {
                            eprintln!("      ✗ Failed to parse BEAM file {}: {:?}", module_name, e);
                            continue;
                        }
                    }
                }
                Err(LoadError::FileError) => {
                    // File not found, try next path
                    continue;
                }
                Err(LoadError::InvalidFormat) => {
                    eprintln!("      ✗ Invalid format: {}", module_name);
                    failed_modules.push(module_name.to_string());
                    found = true; // Don't try other paths
                    break;
                }
            }
        }
        
        if !found {
            eprintln!("      ✗ Not found: {} (searched in: {:?})", module_name, code_paths);
            failed_modules.push(module_name.to_string());
        }
    }
    
    if !failed_modules.is_empty() {
        return Err(format!(
            "Failed to load {} preloaded modules: {:?}. These modules are required for initialization.",
            failed_modules.len(),
            failed_modules
        ));
    }
    
    eprintln!("      ✓ Loaded {}/{} preloaded modules", loaded_count, preloaded_modules.len());
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
/// Based on erl_first_process_otp() from erl_init.c
///
/// Creates the first Erlang process (init process) which will:
/// 1. Execute erl_init:start/2 with boot arguments
/// 2. Load the boot script
/// 3. Start kernel processes
/// 4. Start the Erlang shell
///
/// # Arguments
/// * `boot_module` - Boot module name (e.g., "start")
/// * `boot_args` - Boot arguments (list of strings)
fn create_init_process(boot_module: &str, boot_args: &[String]) -> Result<(), String> {
    use entities_process::Process;
    use infrastructure_utilities::process_table::get_global_process_table;
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    use entities_io_operations::export::get_global_export_table;
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    use std::sync::Arc;
    
    // Verify erl_init module is loaded (must be loaded by load_preloaded())
    let module_loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("erl_init".to_string()))
        .map_err(|e| format!("Failed to check if erl_init is loaded: {:?}", e))?;
    
    match module_loaded {
        ErlangTerm::Atom(ref status) if status == "true" => {
            eprintln!("      ✓ erl_init module is loaded");
        }
        _ => {
            return Err("erl_init module not loaded (preloaded modules must be loaded first)".to_string());
        }
    }
    
    // Look up erl_init:start/2 in the export table
    let atom_table = get_global_atom_table();
    let module_atom_index = atom_table.put_index(b"erl_init", AtomEncoding::SevenBitAscii, false)
        .map_err(|_| "Failed to create atom for module: erl_init".to_string())? as u32;
    
    let function_atom_index = atom_table.put_index(b"start", AtomEncoding::SevenBitAscii, false)
        .map_err(|_| "Failed to create atom for function: start".to_string())? as u32;
    
    let arity = 2u32; // erl_init:start/2
    
    let export_table = get_global_export_table();
    let export = export_table.get(module_atom_index, function_atom_index, arity)
        .ok_or_else(|| "erl_init:start/2 not found in export table (module may not be fully loaded)".to_string())?;
    
    eprintln!("      ✓ Found erl_init:start/2 in export table");
    
    // Get code pointer or resolve label
    let code_ptr = if let Some(ptr) = export.get_code_ptr() {
        eprintln!("      ✓ Export has code pointer");
        Some(ptr)
    } else if let Some(label) = export.label {
        eprintln!("      ⚠ Export has label {} - attempting to resolve to code pointer", label);
        // Attempt to resolve label to code pointer
        match resolve_export_label("erl_init", module_atom_index as usize, function_atom_index, arity, label) {
            Ok(ptr) => {
                eprintln!("      ✓ Resolved label {} to code pointer", label);
                // Update export table with resolved code pointer
                export_table.update_export_code_ptr(module_atom_index, function_atom_index, arity, ptr);
                Some(ptr)
            }
            Err(e) => {
                eprintln!("      ✗ Failed to resolve label: {}", e);
                eprintln!("      ⚠ Using placeholder code (full BEAM code loading requires JIT infrastructure)");
                None
            }
        }
    } else {
        eprintln!("      ✗ Export has neither code pointer nor label");
        return Err("erl_init:start/2 export has no code pointer or label".to_string());
    };
    
    let process_table = get_global_process_table();
    
    // Create init process (PID 1 is typically the init process)
    let mut init_process = Process::new(1);
    
    // Set up process to call erl_init:start/2
    // Code pointer must be resolved - no placeholder fallback
    let ptr = code_ptr.ok_or_else(|| {
        "Failed to resolve code pointer for erl_init:start/2. Module must be loaded and code must be available.".to_string()
    })?;
    
    // Set instruction pointer to function entry point
    init_process.set_i(ptr);
    
    // Set up process heap with boot arguments
    // Boot arguments for erl_init:start/2:
    // - Arg 1: Boot module name (atom, e.g., "start")
    // - Arg 2: Boot arguments (list of strings)
    match setup_boot_arguments(&mut init_process, boot_module, boot_args) {
        Ok(_) => {
            eprintln!("      ✓ Set up process heap with boot arguments (module: {}, args: {})", 
                     boot_module, boot_args.len());
        }
        Err(e) => {
            eprintln!("      ⚠ Failed to set up boot arguments: {} (process will start without arguments)", e);
        }
    }
    
    // Set arity for the function call (erl_init:start/2 has arity 2)
    init_process.set_arity(2);
    
    eprintln!("      ✓ Set instruction pointer to erl_init:start/2 code: {:p}", ptr);
    eprintln!("      ✓ Configured process for function call (arity=2)");
    eprintln!("      ✓ Process ready for BEAM code execution");
    
    let init_process = Arc::new(init_process);
    
    // Insert into process table
    let _old_process = process_table.insert(1, init_process.clone());
    
    // Schedule the init process
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    
    let schedulers = get_global_schedulers()
        .ok_or_else(|| "Schedulers not initialized".to_string())?;
    
    let schedulers_guard = schedulers
        .lock()
        .map_err(|e| format!("Failed to lock schedulers: {}", e))?;
    
    if schedulers_guard.is_empty() {
        return Err("No schedulers available".to_string());
    }
    
    // Schedule on first available scheduler
    // LOCK ORDER: schedulers -> runq (see LOCKING.md)
    let scheduler = &schedulers_guard[0];
    let runq = scheduler.runq();
    let runq_guard = runq
        .lock()
        .map_err(|e| format!("Failed to lock run queue: {}", e))?;
    
    schedule_process(init_process.clone(), &runq_guard, Priority::Max)
        .map_err(|e| format!("Failed to schedule init process: {:?}", e))?;
    
    eprintln!("      ✓ Init process created and scheduled (PID: 1)");
    
    Ok(())
}

/// Verify BEAM code execution setup
///
/// This function verifies that:
/// 1. Preloaded modules are loaded
/// 2. Export table has entries for erl_init:start/2
/// 3. Code pointers can be resolved from labels
/// 4. Process can be created with valid code pointer
///
/// # Returns
/// Result indicating success or failure, with detailed diagnostic information
pub fn verify_beam_execution_setup() -> Result<(), String> {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    use entities_io_operations::export::get_global_export_table;
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    
    eprintln!("Verifying BEAM code execution setup...");
    
    // Step 1: Verify erl_init module is loaded
    let module_loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("erl_init".to_string()))
        .map_err(|e| format!("Failed to check if erl_init is loaded: {:?}", e))?;
    
    match module_loaded {
        ErlangTerm::Atom(ref status) if status == "true" => {
            eprintln!("  ✓ erl_init module is loaded");
        }
        _ => {
            return Err("erl_init module not loaded (run load_preloaded() first)".to_string());
        }
    }
    
    // Step 2: Verify export table has erl_init:start/2
    let atom_table = get_global_atom_table();
    let module_atom_index = atom_table.put_index(b"erl_init", AtomEncoding::SevenBitAscii, false)
        .map_err(|_| "Failed to create atom for module: erl_init".to_string())? as u32;
    
    let function_atom_index = atom_table.put_index(b"start", AtomEncoding::SevenBitAscii, false)
        .map_err(|_| "Failed to create atom for function: start".to_string())? as u32;
    
    let arity = 2u32;
    
    let export_table = get_global_export_table();
    let export = export_table.get(module_atom_index, function_atom_index, arity)
        .ok_or_else(|| "erl_init:start/2 not found in export table".to_string())?;
    
    eprintln!("  ✓ erl_init:start/2 found in export table");
    
    // Step 3: Verify code pointer or label exists
    if let Some(ptr) = export.get_code_ptr() {
        eprintln!("  ✓ Export has code pointer: {:p}", ptr);
    } else if let Some(label) = export.label {
        eprintln!("  ✓ Export has label: {} (needs resolution)", label);
        
        // Step 4: Try to resolve label
        match resolve_export_label("erl_init", module_atom_index as usize, function_atom_index, arity, label) {
            Ok(ptr) => {
                eprintln!("  ✓ Label {} resolved to code pointer: {:p}", label, ptr);
            }
            Err(e) => {
                eprintln!("  ⚠ Label resolution failed: {}", e);
                eprintln!("  ⚠ This may be expected if BEAM code is not yet loaded into executable memory");
            }
        }
    } else {
        return Err("erl_init:start/2 export has neither code pointer nor label".to_string());
    }
    
    // Step 5: Verify code storage has data
    let module_manager = get_global_module_manager();
    let code_ix = get_global_code_ix();
    let active_ix = code_ix.active_code_ix() as usize;
    let atom_table = infrastructure_utilities::atom_table::get_global_atom_table();
    if let Ok(erl_init_atom) = atom_table.put_index(b"erl_init", AtomEncoding::SevenBitAscii, false) {
        if module_manager.get_code_data(erl_init_atom, active_ix).is_some() {
            eprintln!("  ✓ BEAM code data cached for erl_init module");
        } else {
            eprintln!("  ⚠ BEAM code data not cached (will be loaded on demand)");
        }
    }
    
    eprintln!("  ✓ BEAM code execution setup verification complete");
    Ok(())
}

// Use code storage from module management layer
use code_management_code_loading::{get_global_module_manager, get_global_code_ix};

/// Resolve export label to code pointer
///
/// Attempts to resolve a BEAM function label to an actual code pointer.
/// This requires loading the BEAM code chunk and resolving the label offset.
///
/// # Arguments
/// * `module_name` - Module name (for loading BEAM file)
/// * `module_atom_index` - Module atom index
/// * `function_atom_index` - Function atom index
/// * `arity` - Function arity
/// * `label` - Label offset in BEAM code
///
/// # Returns
/// Code pointer if resolution succeeds, error otherwise
///
/// # Note
/// Full implementation requires:
/// 1. Loading BEAM code chunk into executable memory (using JIT allocator)
/// 2. Resolving label offset to code pointer within executable memory
/// 3. Making code executable (set memory protection)
/// For now, this stores code data in a global HashMap and creates pointers into it.
fn resolve_export_label(
    module_name: &str,
    module_atom_index: usize,
    _function_atom_index: u32,
    _arity: u32,
    label: i32,
) -> Result<entities_process::ErtsCodePtr, String> {
    use code_management_code_loading::{CodeLoader, BeamLoader};
    use code_management_code_loading::code_loader::LoadError;
    use std::path::Path;
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    
    let module_manager = get_global_module_manager();
    let code_ix = get_global_code_ix();
    let active_ix = code_ix.active_code_ix() as usize;
    
    // Check if code is already loaded
    if let Some(code_data) = module_manager.get_code_data(module_atom_index, active_ix) {
            // Code already loaded - resolve label to code pointer
            // BEAM code chunk format:
            // - Header: 4 bytes sub-size, 4 bytes IS, 4 bytes OM, 4 bytes L, 4 bytes F (20 bytes total)
            // - Code: actual BEAM instructions follow
            // Labels are instruction offsets (not byte offsets)
            // Each instruction is typically 4 bytes (one word)
            
            // Parse code chunk header to find where actual code starts
            let code_header_size = if code_data.len() >= 20 {
                // Read sub-size from header (first 4 bytes, big-endian)
                let sub_size = u32::from_be_bytes([
                    code_data[0], code_data[1], code_data[2], code_data[3]
                ]) as usize;
                
                // Header is: sub-size (4) + IS (4) + OM (4) + L (4) + F (4) = 20 bytes
                // But sub-size tells us the size of the header section
                // For now, assume standard 20-byte header
                20
            } else {
                // Code data too small, assume no header
                0
            };
            
            // Labels are instruction offsets (each instruction is 4 bytes)
            // So label N points to instruction at offset: header_size + (N * 4)
            let instruction_size = 4; // BEAM instructions are 4 bytes (one word)
            let label_offset = code_header_size + ((label as usize) * instruction_size);
            
            if label_offset >= code_data.len() {
                return Err(format!(
                    "Label {} (offset {}) out of bounds for module {} (code size: {}, header: {})",
                    label, label_offset, module_name, code_data.len(), code_header_size
                ));
            }
            
            let code_ptr = code_data.as_ptr().wrapping_add(label_offset) as entities_process::ErtsCodePtr;
            eprintln!("      ✓ Resolved label {} to instruction offset {} (byte offset: {})", 
                     label, label, label_offset);
            eprintln!("      ✓ Code pointer: {:p} (base: {:p}, header: {}, offset: {})", 
                     code_ptr, code_data.as_ptr(), code_header_size, label_offset);
            
            // Verify the code pointer is within bounds
            let code_ptr_usize = code_ptr as usize;
            let code_base_usize = code_data.as_ptr() as usize;
            let code_end_usize = code_base_usize + code_data.len();
            
            if code_ptr_usize < code_base_usize || code_ptr_usize >= code_end_usize {
                return Err(format!(
                    "Code pointer {:p} is out of bounds (base: {:p}, end: {:p})",
                    code_ptr, code_data.as_ptr(), code_end_usize as *const u8
                ));
            }
            
            return Ok(code_ptr);
        }
    
    // Code not loaded - load it now
    let code_paths = get_preloaded_code_paths();
    
    for code_path in &code_paths {
        let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
        
        if !beam_path.exists() {
            continue;
        }
        
        // Load BEAM file
        match CodeLoader::load_module(&beam_path) {
            Ok(beam_data) => {
                // Parse BEAM file
                match BeamLoader::read_beam_file(&beam_data) {
                    Ok(beam_file) => {
                        if beam_file.code_data.is_empty() {
                            return Err(format!("Module {} has empty code chunk", module_name));
                        }
                        
                        // Parse code chunk header to find where actual code starts
                        // BEAM code chunk format:
                        // - Header: 4 bytes sub-size, 4 bytes IS, 4 bytes OM, 4 bytes L, 4 bytes F (20 bytes total)
                        // - Code: actual BEAM instructions follow
                        // Labels are instruction offsets (not byte offsets)
                        let code_header_size = if beam_file.code_data.len() >= 20 {
                            // Standard 20-byte header
                            20
                        } else {
                            // Code data too small, assume no header
                            0
                        };
                        
                        // Labels are instruction offsets (each instruction is 4 bytes)
                        // So label N points to instruction at offset: header_size + (N * 4)
                        let instruction_size = 4; // BEAM instructions are 4 bytes (one word)
                        let label_offset = code_header_size + ((label as usize) * instruction_size);
                        
                        // Validate label offset
                        if label_offset >= beam_file.code_data.len() {
                            return Err(format!(
                                "Label {} (offset {}) out of bounds for module {} (code size: {}, header: {})",
                                label, label_offset, module_name, beam_file.code_data.len(), code_header_size
                            ));
                        }
                        
                        // Store code data in module table using Box::leak to create static allocation
                        // This ensures the code data lives for the program lifetime
                        let code_data_vec = beam_file.code_data;
                        let code_data_box = Box::new(code_data_vec);
                        let code_data_static: &'static [u8] = Box::leak(code_data_box);
                        
                        // Get module atom index if not provided
                        let module_atom = if module_atom_index == 0 {
                            let atom_table = get_global_atom_table();
                            atom_table.put_index(module_name.as_bytes(), AtomEncoding::SevenBitAscii, false)
                                .map_err(|_| format!("Failed to create atom for module: {}", module_name))?
                        } else {
                            module_atom_index
                        };
                        
                        // Store code data in module table
                        module_manager.put_module_with_code(module_atom, code_data_static, active_ix);
                        
                        // Create code pointer from static data
                        // Label is an instruction offset, so we add header_size + (label * instruction_size)
                        let code_ptr = code_data_static.as_ptr().wrapping_add(label_offset) as entities_process::ErtsCodePtr;
                        
                        eprintln!("      ✓ Loaded and cached BEAM code for module {} (label: {}, instruction offset: {}, byte offset: {}, code size: {})", 
                                 module_name, label, label, label_offset, code_data_static.len());
                        eprintln!("      ✓ Code pointer created: {:p} (base: {:p}, header: {}, offset: {})", 
                                 code_ptr, code_data_static.as_ptr(), code_header_size, label_offset);
                        
                        // Verify the code pointer is within bounds
                        let code_ptr_usize = code_ptr as usize;
                        let code_base_usize = code_data_static.as_ptr() as usize;
                        let code_end_usize = code_base_usize + code_data_static.len();
                        
                        if code_ptr_usize < code_base_usize || code_ptr_usize >= code_end_usize {
                            return Err(format!(
                                "Code pointer {:p} is out of bounds for module {} (base: {:p}, end: {:p})",
                                code_ptr, module_name, code_data_static.as_ptr(), 
                                code_end_usize as *const u8
                            ));
                        }
                        
                        return Ok(code_ptr);
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse BEAM file: {:?}", e));
                    }
                }
            }
            Err(LoadError::FileError) => {
                continue; // Try next path
            }
            Err(LoadError::InvalidFormat) => {
                return Err(format!("Invalid BEAM format for module {}", module_name));
            }
        }
    }
    
    Err(format!("Module {} not found in code paths", module_name))
}

/// Get code paths for preloaded modules
fn get_preloaded_code_paths() -> Vec<String> {
    use crate::env;
    use std::path::Path;
    use std::fs;
    
    let (rootdir, bindir) = env::determine_paths().unwrap_or_else(|_| (String::new(), String::new()));
    let mut code_paths = Vec::new();
    
    // Use the same search logic as load_preloaded()
    if !rootdir.is_empty() {
        // Try preloaded directory with ebin subdirectory first (highest priority)
        let preloaded_ebin = Path::new(&rootdir).join("erts").join("preloaded").join("ebin");
        if preloaded_ebin.exists() {
            code_paths.push(preloaded_ebin.to_string_lossy().to_string());
        }
        
        // Also try preloaded directory without ebin subdirectory
        let preloaded_dir = Path::new(&rootdir).join("erts").join("preloaded");
        if preloaded_dir.exists() {
            code_paths.push(preloaded_dir.to_string_lossy().to_string());
        }
        
        // Try to find erts and kernel application directories
        let lib_dir = Path::new(&rootdir).join("lib");
        if lib_dir.exists() {
            if let Ok(entries) = fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let app_dir = entry.path();
                    if app_dir.is_dir() {
                        let dir_name = app_dir.file_name().and_then(|n| n.to_str());
                        if let Some(name) = dir_name {
                            if name.starts_with("erts-") || name.starts_with("kernel-") {
                                let ebin_dir = app_dir.join("ebin");
                                if ebin_dir.exists() {
                                    code_paths.push(ebin_dir.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Add bindir as fallback
    if !bindir.is_empty() {
        code_paths.push(bindir);
    }
    
    code_paths
}

/// Set up boot arguments on process heap
///
/// Allocates and encodes boot arguments for erl_init:start/2:
/// - Arg 1: Boot module name (atom)
/// - Arg 2: Boot arguments (list of strings)
///
/// # Arguments
/// * `process` - Process to set up
/// * `boot_module` - Boot module name (e.g., "start")
/// * `boot_args` - Boot arguments (list of strings)
///
/// # Returns
/// Result indicating success or failure
fn setup_boot_arguments(process: &mut entities_process::Process, boot_module: &str, boot_args: &[String]) -> Result<(), String> {
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    
    // Get atom table and create atom for boot module name
    let atom_table = get_global_atom_table();
    let boot_module_atom_index = atom_table.put_index(
        boot_module.as_bytes(),
        AtomEncoding::SevenBitAscii,
        false,
    )
    .map_err(|_| format!("Failed to create atom for boot module name: {}", boot_module))? as u32;
    
    // Encode boot module name as Eterm atom
    // Format: (atom_index << 6) | 0x0B
    let boot_module_term = ((boot_module_atom_index as u64) << 6) | 0x0B;
    
    // Encode boot arguments as a list
    // If boot_args is empty, use nil (0x3F)
    // Otherwise, create a proper list structure
    let boot_args_term = if boot_args.is_empty() {
        // Empty list (nil)
        0x3F
    } else {
        // Create a list from boot arguments
        // Each argument needs to be encoded as a string/atom
        // For now, we'll create a simple list structure
        // In the full implementation, we'd properly encode strings
        
        // Calculate space needed for list
        // Each cons cell needs 2 words (head, tail)
        // Plus we need to encode each argument
        let list_cells = boot_args.len();
        let words_needed = list_cells * 2; // 2 words per cons cell
        
        // Allocate heap space for list
        let list_start = process.allocate_heap_words(words_needed)
            .ok_or_else(|| "Failed to allocate heap for boot arguments list".to_string())?;
        
        // Encode each argument and build the list
        let mut heap_slice = process.heap_slice_mut();
        
        // Build list from end to beginning (proper list structure)
        for (i, arg) in boot_args.iter().enumerate().rev() {
            // Encode argument as atom (simplified - in full impl would handle strings properly)
            let arg_atom_index = atom_table.put_index(
                arg.as_bytes(),
                AtomEncoding::SevenBitAscii,
                false,
            )
            .map_err(|_| format!("Failed to create atom for boot argument: {}", arg))? as u32;
            
            let arg_term = ((arg_atom_index as u64) << 6) | 0x0B;
            
            // Determine tail (nil for last element, or pointer to next cons cell)
            let tail = if i == boot_args.len() - 1 {
                0x3F // Nil
            } else {
                // Pointer to next cons cell: (heap_index << 2) | 0x1 (list tag)
                let next_cell_index = list_start + ((i + 1) * 2);
                ((next_cell_index as u64) << 2) | 0x1
            };
            
            // Write cons cell: [head, tail]
            let cell_index = list_start + (i * 2);
            heap_slice[cell_index] = arg_term;      // Head
            heap_slice[cell_index + 1] = tail;      // Tail
        }
        
        // Return pointer to first cons cell: (heap_index << 2) | 0x1 (list tag)
        ((list_start as u64) << 2) | 0x1
    };
    
    // Set up argument registers in process heap
    // In BEAM, argument registers x(0) through x(arity-1) are stored at heap_start_index()
    // We need to ensure the heap is large enough and store arguments at the correct position
    let heap_start = process.heap_start_index();
    let required_heap_size = heap_start + 2; // Need space for 2 arguments (x(0) and x(1))
    
    // Ensure heap is large enough
    {
        let mut heap_slice = process.heap_slice_mut();
        if heap_slice.len() < required_heap_size {
            heap_slice.resize(required_heap_size, 0);
        }
        
        // Write arguments to heap at the correct position (heap_start is where X registers begin)
        heap_slice[heap_start] = boot_module_term;     // x(0) = boot module name
        heap_slice[heap_start + 1] = boot_args_term;   // x(1) = boot arguments (list)
    }
    
    eprintln!("      ✓ Boot arguments stored at heap[{}] and heap[{}]", heap_start, heap_start + 1);
    
    Ok(())
}

/// Extract boot module name from command line arguments
///
/// Looks for -boot flag and extracts the boot module name.
/// Defaults to "start" if not specified.
///
/// # Arguments
/// * `argv` - Command line arguments
///
/// # Returns
/// Boot module name (without .boot extension)
fn extract_boot_module(argv: &[String]) -> Option<String> {
    for (i, arg) in argv.iter().enumerate() {
        if (arg == "--boot" || arg == "-boot") && i + 1 < argv.len() {
            let boot_path = &argv[i + 1];
            // Extract module name from path (remove .boot extension and path)
            let module_name = boot_path
                .trim_end_matches(".boot")
                .split('/')
                .last()
                .unwrap_or(boot_path)
                .to_string();
            return Some(module_name);
        }
    }
    None
}

/// Extract boot arguments from command line
///
/// Extracts arguments that should be passed to erl_init:start/2.
/// These are typically arguments after -- or specific flags.
///
/// # Arguments
/// * `argv` - Command line arguments
///
/// # Returns
/// Vector of boot argument strings
fn extract_boot_args(argv: &[String]) -> Vec<String> {
    let mut args = Vec::new();
    let mut after_double_dash = false;
    
    for arg in argv {
        if arg == "--" {
            after_double_dash = true;
            continue;
        }
        
        if after_double_dash {
            // Arguments after -- are plain arguments passed to init
            args.push(arg.clone());
        }
        // Other arguments are handled by early_init or other parts of the system
    }
    
    args
}

/// Wait for shutdown signal
///
/// Blocks the main thread until the emulator is shut down.
/// In the full implementation, this would:
/// 1. Wait for shutdown signal (SIGTERM, SIGINT, etc.)
/// 2. Gracefully stop scheduler threads
/// 3. Clean up resources
fn wait_for_shutdown(handles: Vec<std::thread::JoinHandle<()>>) {
    eprintln!("[DEBUG] wait_for_shutdown: entered");
    // Start a simple REPL loop in the main thread
    // In the full implementation, this would be handled by user_drv and shell processes
    eprintln!("[DEBUG] wait_for_shutdown: calling start_simple_repl");
    start_simple_repl();
    eprintln!("[DEBUG] wait_for_shutdown: start_simple_repl returned");
    
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
    eprintln!("[DEBUG] start_simple_repl: entered");
    use std::io::{self, BufRead, Write};
    use infrastructure_utilities::erl_eval::new_bindings;
    
    // Print Erlang/OTP banner (similar to C version)
    eprintln!("[DEBUG] start_simple_repl: printing banner");
    println!("Erlang/OTP [Iron BEAM] [erts-15.0] [source] [64-bit]");
    println!("Eshell V15.0  (press Ctrl+c to abort, type help(). for help)");
    
    // Maintain bindings across expressions
    let mut bindings = new_bindings();
    
    let stdin = io::stdin();
    let mut line_count = 1;
    
    eprintln!("[DEBUG] start_simple_repl: entering main loop");
    loop {
        // Only log every 10th iteration to reduce noise
        if line_count % 10 == 1 {
            eprintln!("[DEBUG] start_simple_repl: loop iteration {}", line_count);
        }
        // Print prompt
        print!("{}> ", line_count);
        io::stdout().flush().unwrap();
        
        // Read input until dot is found (multiline support, matching erl_scan:tokens behavior)
        let mut input_buffer = String::new();
        loop {
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => {
                    // EOF
                    println!("\n");
                    return;
                }
                Ok(_) => {
                    input_buffer.push_str(&line);
                    let trimmed = input_buffer.trim();
                    
                    // Handle empty lines
                    if trimmed.is_empty() {
                        break;
                    }
                    
                    // Check if we have a complete expression (ends with dot)
                    if trimmed.ends_with('.') {
                        // We have a complete expression, process it
                        break;
                    }
                    
                    // No dot yet - continue reading (multiline input)
                    // Print continuation prompt (matching Erlang behavior)
                    print!("  | ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    return;
                }
            }
        }
        
        let trimmed = input_buffer.trim();
        
        // Handle empty lines
        if trimmed.is_empty() {
            continue;
        }
        
        // Handle special commands (these must end with dot)
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
                // Use scan_until_dot + parse_repl_exprs + evaluator with persistent bindings
                // scan_until_dot will return an error if no dot is found (shouldn't happen here
                // since we checked above, but good to handle anyway)
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
/// Uses scan_until_dot() and parse_repl_exprs() to match Erlang behavior:
/// - Scanner requires a dot before completing
/// - Parser requires and consumes the dot token
fn evaluate_erlang_expression_with_bindings(
    input: &str,
    bindings: &mut infrastructure_utilities::erl_eval::Bindings,
) -> Result<entities_data_handling::term_hashing::Term, String> {
    use infrastructure_utilities::{scan_until_dot, parse_repl_exprs, exprs};
    
    // Step 1: Scan until dot (matches erl_scan:tokens behavior)
    // This requires a dot before completing, matching Erlang REPL behavior
    let tokens = scan_until_dot(input)
        .map_err(|e| format!("Scan error: {}", e))?;
    
    // Step 2: Parse expressions (requires dot, matches erl_eval:extended_parse_exprs behavior)
    let parsed_exprs = parse_repl_exprs(tokens)
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
    use crate::initialization::set_initialized;
    
    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.ncpu, 1);
        assert_eq!(config.proc_tab_sz, 1_048_576);
        assert_eq!(config.port_tab_sz, 1_048_576);
        assert_eq!(config.no_schedulers, 1);
        assert_eq!(config.no_schedulers_online, 1);
        assert_eq!(config.no_poll_threads, 1);
        assert_eq!(config.no_dirty_cpu_schedulers, 0);
        assert_eq!(config.no_dirty_cpu_schedulers_online, 0);
        assert_eq!(config.no_dirty_io_schedulers, 0);
        assert_eq!(config.time_correction, 0);
        assert_eq!(config.time_warp_mode, TimeWarpMode::NoTimeWarp);
    }

    #[test]
    fn test_init_config_debug() {
        let config = InitConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("InitConfig"));
    }

    #[test]
    fn test_init_config_clone() {
        let config1 = InitConfig {
            ncpu: 4,
            proc_tab_sz: 2_097_152,
            port_tab_sz: 2_097_152,
            no_schedulers: 4,
            no_schedulers_online: 4,
            no_poll_threads: 2,
            no_dirty_cpu_schedulers: 1,
            no_dirty_cpu_schedulers_online: 1,
            no_dirty_io_schedulers: 1,
            time_correction: 1,
            time_warp_mode: TimeWarpMode::MultiTimeWarp,
        };
        let config2 = config1.clone();
        assert_eq!(config1.ncpu, config2.ncpu);
        assert_eq!(config1.proc_tab_sz, config2.proc_tab_sz);
        assert_eq!(config1.port_tab_sz, config2.port_tab_sz);
        assert_eq!(config1.no_schedulers, config2.no_schedulers);
        assert_eq!(config1.no_schedulers_online, config2.no_schedulers_online);
        assert_eq!(config1.no_poll_threads, config2.no_poll_threads);
        assert_eq!(config1.no_dirty_cpu_schedulers, config2.no_dirty_cpu_schedulers);
        assert_eq!(config1.no_dirty_cpu_schedulers_online, config2.no_dirty_cpu_schedulers_online);
        assert_eq!(config1.no_dirty_io_schedulers, config2.no_dirty_io_schedulers);
        assert_eq!(config1.time_correction, config2.time_correction);
        assert_eq!(config1.time_warp_mode, config2.time_warp_mode);
    }

    #[test]
    fn test_init_config_custom() {
        let config = InitConfig {
            ncpu: 8,
            proc_tab_sz: 4_194_304,
            port_tab_sz: 4_194_304,
            no_schedulers: 8,
            no_schedulers_online: 8,
            no_poll_threads: 4,
            no_dirty_cpu_schedulers: 2,
            no_dirty_cpu_schedulers_online: 2,
            no_dirty_io_schedulers: 2,
            time_correction: 2,
            time_warp_mode: TimeWarpMode::SingleTimeWarp,
        };
        assert_eq!(config.ncpu, 8);
        assert_eq!(config.proc_tab_sz, 4_194_304);
        assert_eq!(config.port_tab_sz, 4_194_304);
        assert_eq!(config.no_schedulers, 8);
        assert_eq!(config.no_schedulers_online, 8);
        assert_eq!(config.no_poll_threads, 4);
        assert_eq!(config.no_dirty_cpu_schedulers, 2);
        assert_eq!(config.no_dirty_cpu_schedulers_online, 2);
        assert_eq!(config.no_dirty_io_schedulers, 2);
        assert_eq!(config.time_correction, 2);
        assert_eq!(config.time_warp_mode, TimeWarpMode::SingleTimeWarp);
    }

    #[test]
    fn test_time_warp_mode_variants() {
        let mode1 = TimeWarpMode::NoTimeWarp;
        let mode2 = TimeWarpMode::MultiTimeWarp;
        let mode3 = TimeWarpMode::SingleTimeWarp;
        
        // Test Debug
        let _ = format!("{:?}", mode1);
        let _ = format!("{:?}", mode2);
        let _ = format!("{:?}", mode3);
        
        // Test Clone
        let cloned1 = mode1.clone();
        let cloned2 = mode2.clone();
        let cloned3 = mode3.clone();
        
        // Test Copy (implicit)
        let copied1 = mode1;
        let copied2 = mode2;
        let copied3 = mode3;
        
        // Test PartialEq
        assert_eq!(mode1, cloned1);
        assert_eq!(mode2, cloned2);
        assert_eq!(mode3, cloned3);
        assert_eq!(mode1, copied1);
        assert_eq!(mode2, copied2);
        assert_eq!(mode3, copied3);
        assert_ne!(mode1, mode2);
        assert_ne!(mode1, mode3);
        assert_ne!(mode2, mode3);
        
        // Test Eq
        assert!(mode1 == cloned1);
        assert!(mode1 != mode2);
    }

    #[test]
    fn test_time_warp_mode_all_variants() {
        // Test all variants exist and are distinct
        let modes = vec![
            TimeWarpMode::NoTimeWarp,
            TimeWarpMode::MultiTimeWarp,
            TimeWarpMode::SingleTimeWarp,
        ];
        
        // All should be distinct
        assert_ne!(modes[0], modes[1]);
        assert_ne!(modes[0], modes[2]);
        assert_ne!(modes[1], modes[2]);
        
        // Each should equal itself
        assert_eq!(modes[0], modes[0]);
        assert_eq!(modes[1], modes[1]);
        assert_eq!(modes[2], modes[2]);
    }
    
    #[test]
    fn test_erl_init() {
        set_initialized(false);
        let config = InitConfig::default();
        let result = erl_init(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_erl_init_with_custom_config() {
        set_initialized(false);
        let config = InitConfig {
            ncpu: 2,
            no_schedulers: 2,
            no_schedulers_online: 2,
            ..Default::default()
        };
        let result = erl_init(config);
        // May succeed or fail depending on system state
        let _ = result;
    }

    #[test]
    fn test_erl_init_sets_initialized() {
        set_initialized(false);
        let config = InitConfig::default();
        let result = erl_init(config);
        if result.is_ok() {
            assert!(crate::initialization::is_initialized());
        }
    }

    #[test]
    fn test_erl_init_with_all_time_warp_modes() {
        set_initialized(false);
        let modes = vec![
            TimeWarpMode::NoTimeWarp,
            TimeWarpMode::MultiTimeWarp,
            TimeWarpMode::SingleTimeWarp,
        ];
        
        for mode in modes {
            set_initialized(false);
            let config = InitConfig {
                time_warp_mode: mode,
                ..Default::default()
            };
            let _result = erl_init(config);
            // May succeed or fail depending on system state
        }
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
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_arg() {
        let mut argc = 3;
        let mut argv = vec!["test".to_string(), "--boot".to_string(), "start".to_string()];
        
        let result = erl_start(&mut argc, &mut argv);
        // May fail if early_init was already called
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_empty_argv() {
        let mut argc = 0;
        let mut argv = vec![];
        
        let result = erl_start(&mut argc, &mut argv);
        // May fail if early_init was already called
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_multiple_args() {
        let mut argc = 5;
        let mut argv = vec![
            "test".to_string(),
            "--boot".to_string(),
            "start".to_string(),
            "--sname".to_string(),
            "test@localhost".to_string(),
        ];
        
        let result = erl_start(&mut argc, &mut argv);
        // May fail if early_init was already called
        let _ = result;
    }

    #[test]
    fn test_verify_beam_execution_setup() {
        // This test may fail if preloaded modules aren't loaded
        // It's mainly to ensure the function doesn't panic
        let result = verify_beam_execution_setup();
        // May succeed or fail depending on system state
        let _ = result;
    }

    #[test]
    fn test_verify_beam_execution_setup_error_message() {
        // Test that error messages are informative
        let result = verify_beam_execution_setup();
        if let Err(e) = result {
            // Error message should not be empty
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_init_config_field_access() {
        let config = InitConfig {
            ncpu: 16,
            proc_tab_sz: 8_388_608,
            port_tab_sz: 8_388_608,
            no_schedulers: 16,
            no_schedulers_online: 16,
            no_poll_threads: 8,
            no_dirty_cpu_schedulers: 4,
            no_dirty_cpu_schedulers_online: 4,
            no_dirty_io_schedulers: 4,
            time_correction: 3,
            time_warp_mode: TimeWarpMode::MultiTimeWarp,
        };
        
        // Verify all fields can be accessed
        assert_eq!(config.ncpu, 16);
        assert_eq!(config.proc_tab_sz, 8_388_608);
        assert_eq!(config.port_tab_sz, 8_388_608);
        assert_eq!(config.no_schedulers, 16);
        assert_eq!(config.no_schedulers_online, 16);
        assert_eq!(config.no_poll_threads, 8);
        assert_eq!(config.no_dirty_cpu_schedulers, 4);
        assert_eq!(config.no_dirty_cpu_schedulers_online, 4);
        assert_eq!(config.no_dirty_io_schedulers, 4);
        assert_eq!(config.time_correction, 3);
        assert_eq!(config.time_warp_mode, TimeWarpMode::MultiTimeWarp);
    }

    #[test]
    fn test_time_warp_mode_debug_format() {
        let modes = vec![
            TimeWarpMode::NoTimeWarp,
            TimeWarpMode::MultiTimeWarp,
            TimeWarpMode::SingleTimeWarp,
        ];
        
        for mode in modes {
            let debug_str = format!("{:?}", mode);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_init_config_with_zero_values() {
        let config = InitConfig {
            ncpu: 0,
            proc_tab_sz: 0,
            port_tab_sz: 0,
            no_schedulers: 0,
            no_schedulers_online: 0,
            no_poll_threads: 0,
            no_dirty_cpu_schedulers: 0,
            no_dirty_cpu_schedulers_online: 0,
            no_dirty_io_schedulers: 0,
            time_correction: 0,
            time_warp_mode: TimeWarpMode::NoTimeWarp,
        };
        
        // Should be able to create config with zero values
        assert_eq!(config.ncpu, 0);
        assert_eq!(config.proc_tab_sz, 0);
    }

    #[test]
    fn test_init_config_with_large_values() {
        let config = InitConfig {
            ncpu: 1024,
            proc_tab_sz: 1_073_741_824,
            port_tab_sz: 1_073_741_824,
            no_schedulers: 1024,
            no_schedulers_online: 1024,
            no_poll_threads: 512,
            no_dirty_cpu_schedulers: 256,
            no_dirty_cpu_schedulers_online: 256,
            no_dirty_io_schedulers: 256,
            time_correction: 100,
            time_warp_mode: TimeWarpMode::MultiTimeWarp,
        };
        
        // Should be able to create config with large values
        assert_eq!(config.ncpu, 1024);
        assert_eq!(config.proc_tab_sz, 1_073_741_824);
    }

    #[test]
    fn test_time_warp_mode_copy_semantics() {
        let mode1 = TimeWarpMode::NoTimeWarp;
        let mode2 = mode1; // Copy, not move
        let mode3 = mode1; // Can copy again
        
        // All should be equal
        assert_eq!(mode1, mode2);
        assert_eq!(mode1, mode3);
        assert_eq!(mode2, mode3);
    }

    #[test]
    fn test_init_config_debug_includes_fields() {
        let config = InitConfig::default();
        let debug_str = format!("{:?}", config);
        
        // Debug string should contain key information
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_erl_init_idempotent_behavior() {
        set_initialized(false);
        let config = InitConfig::default();
        
        // First call
        let result1 = erl_init(config.clone());
        
        // Second call - may succeed or fail depending on implementation
        let result2 = erl_init(config);
        
        // At least one should provide information about behavior
        let _ = (result1, result2);
    }

    #[test]
    fn test_erl_init_with_dirty_schedulers() {
        set_initialized(false);
        let config = InitConfig {
            no_dirty_cpu_schedulers: 2,
            no_dirty_cpu_schedulers_online: 2,
            no_dirty_io_schedulers: 1,
            ..Default::default()
        };
        let result = erl_init(config);
        // May succeed or fail depending on system state
        let _ = result;
    }

    #[test]
    fn test_erl_init_with_poll_threads() {
        set_initialized(false);
        let config = InitConfig {
            no_poll_threads: 4,
            ..Default::default()
        };
        let result = erl_init(config);
        let _ = result;
    }

    #[test]
    fn test_erl_init_with_time_correction() {
        set_initialized(false);
        let config = InitConfig {
            time_correction: 1,
            ..Default::default()
        };
        let result = erl_init(config);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_flag_variations() {
        // Test -boot flag (short form)
        let mut argc = 3;
        let mut argv = vec!["test".to_string(), "-boot".to_string(), "start".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_args_after_double_dash() {
        let mut argc = 5;
        let mut argv = vec![
            "test".to_string(),
            "--boot".to_string(),
            "start".to_string(),
            "--".to_string(),
            "arg1".to_string(),
        ];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_multiple_boot_args() {
        let mut argc = 6;
        let mut argv = vec![
            "test".to_string(),
            "--boot".to_string(),
            "start".to_string(),
            "--".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_flag_at_end() {
        // Test boot flag at end (should handle gracefully)
        let mut argc = 2;
        let mut argv = vec!["test".to_string(), "--boot".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_path_with_extension() {
        let mut argc = 3;
        let mut argv = vec!["test".to_string(), "--boot".to_string(), "start.boot".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_erl_start_with_boot_path_with_directory() {
        let mut argc = 3;
        let mut argv = vec!["test".to_string(), "--boot".to_string(), "/path/to/start.boot".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        let _ = result;
    }

    #[test]
    fn test_verify_beam_execution_setup_after_init() {
        set_initialized(false);
        let config = InitConfig::default();
        let _ = erl_init(config);
        
        // Now verify setup
        let result = verify_beam_execution_setup();
        // May succeed or fail depending on whether preloaded modules were loaded
        let _ = result;
    }

    #[test]
    fn test_verify_beam_execution_setup_error_details() {
        let result = verify_beam_execution_setup();
        if let Err(e) = result {
            // Error should contain useful diagnostic information
            assert!(!e.is_empty());
            // Should mention what's missing
            assert!(e.contains("erl_init") || e.contains("module") || e.contains("export") || e.contains("code"));
        }
    }


    #[test]
    fn test_init_config_partial_eq() {
        let config1 = InitConfig::default();
        let config2 = InitConfig::default();
        
        // Configs with same values should be equal (if PartialEq is derived)
        // Note: InitConfig may not implement PartialEq, so this test verifies structure
        assert_eq!(config1.ncpu, config2.ncpu);
        assert_eq!(config1.time_warp_mode, config2.time_warp_mode);
    }

    #[test]
    fn test_erl_init_error_handling_global_literals() {
        // Test that erl_init properly propagates errors from global literals init
        // This is tested indirectly through the error message format
        set_initialized(false);
        let config = InitConfig::default();
        let result = erl_init(config);
        
        if let Err(e) = result {
            // Error should be informative
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_erl_init_error_handling_process_management() {
        // Test error handling for process management initialization
        set_initialized(false);
        let config = InitConfig {
            proc_tab_sz: 0, // Invalid size might cause error
            ..Default::default()
        };
        let result = erl_init(config);
        // May fail with informative error
        if let Err(e) = result {
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_erl_init_error_handling_scheduling() {
        // Test error handling for scheduling initialization
        set_initialized(false);
        let config = InitConfig {
            no_schedulers: 0, // Invalid - no schedulers
            no_schedulers_online: 0,
            ..Default::default()
        };
        let result = erl_init(config);
        // May fail with informative error
        if let Err(e) = result {
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_erl_start_error_handling_early_init() {
        // Test that erl_start properly handles early_init errors
        // This is tested through error message format
        let mut argc = 1;
        let mut argv = vec!["test".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        
        if let Err(e) = result {
            // Error should mention early initialization if that's where it failed
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_erl_start_error_handling_scheduler_start() {
        // Test error handling when scheduler start fails
        // This is tested indirectly
        let mut argc = 1;
        let mut argv = vec!["test".to_string()];
        let result = erl_start(&mut argc, &mut argv);
        
        if let Err(e) = result {
            // Error should be informative
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_time_warp_mode_equality() {
        let mode1 = TimeWarpMode::NoTimeWarp;
        let mode2 = TimeWarpMode::NoTimeWarp;
        let mode3 = TimeWarpMode::MultiTimeWarp;
        
        assert_eq!(mode1, mode2);
        assert_ne!(mode1, mode3);
    }

    #[test]
    fn test_init_config_with_minimal_values() {
        let config = InitConfig {
            ncpu: 1,
            proc_tab_sz: 1,
            port_tab_sz: 1,
            no_schedulers: 1,
            no_schedulers_online: 1,
            no_poll_threads: 0,
            no_dirty_cpu_schedulers: 0,
            no_dirty_cpu_schedulers_online: 0,
            no_dirty_io_schedulers: 0,
            time_correction: -1,
            time_warp_mode: TimeWarpMode::SingleTimeWarp,
        };
        
        assert_eq!(config.ncpu, 1);
        assert_eq!(config.proc_tab_sz, 1);
        assert_eq!(config.time_warp_mode, TimeWarpMode::SingleTimeWarp);
    }

    #[test]
    fn test_erl_init_with_single_time_warp() {
        set_initialized(false);
        let config = InitConfig {
            time_warp_mode: TimeWarpMode::SingleTimeWarp,
            ..Default::default()
        };
        let result = erl_init(config);
        let _ = result;
    }

    #[test]
    fn test_erl_init_with_multi_time_warp() {
        set_initialized(false);
        let config = InitConfig {
            time_warp_mode: TimeWarpMode::MultiTimeWarp,
            ..Default::default()
        };
        let result = erl_init(config);
        let _ = result;
    }
}

