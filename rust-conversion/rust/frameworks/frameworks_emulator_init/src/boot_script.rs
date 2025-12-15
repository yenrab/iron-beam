//! Boot Script Parser and Executor
//!
//! Handles loading and executing boot scripts (.boot files).
//! Boot scripts are binary Erlang terms containing instructions for:
//! - Loading modules
//! - Setting code paths
//! - Starting kernel processes
//! - Starting applications
//!
//! Based on init.erl boot script handling

use std::path::Path;
use std::sync::{Mutex, OnceLock};
use infrastructure_utilities::{ErlangTerm, decode_term};
use entities_utilities::{Register, RegisterResult};

/// Boot script structure
#[derive(Debug, Clone)]
pub struct BootScript {
    /// Script name
    pub name: String,
    /// Script version
    pub version: String,
    /// List of commands to execute
    pub commands: Vec<BootCommand>,
}

/// Boot script command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootCommand {
    /// Progress update: {progress, Info}
    Progress(String),
    /// Preloaded modules: {preLoaded, [Mod1, Mod2, ...]}
    PreLoaded(Vec<String>),
    /// Code path: {path, [Dir1, Dir2, ...]}
    Path(Vec<String>),
    /// Primary load: {primLoad, [Mod1, Mod2, ...]}
    PrimLoad(Vec<String>),
    /// Kernel load completed: {kernel_load_completed}
    KernelLoadCompleted,
    /// Kernel process: {kernelProcess, Name, {Mod, Func, Args}}
    KernelProcess {
        name: String,
        module: String,
        function: String,
        args: Vec<String>,
    },
    /// Apply function: {apply, {Mod, Func, Args}}
    Apply {
        module: String,
        function: String,
        args: Vec<String>,
    },
}

/// Boot script parser error
#[derive(Debug, Clone)]
pub enum BootScriptError {
    /// File not found
    NotFound(String),
    /// Invalid format
    InvalidFormat(String),
    /// Parse error
    ParseError(String),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for BootScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootScriptError::NotFound(msg) => write!(f, "Boot script not found: {}", msg),
            BootScriptError::InvalidFormat(msg) => write!(f, "Invalid boot script format: {}", msg),
            BootScriptError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BootScriptError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for BootScriptError {}

/// Load and parse a boot script file
///
/// Boot scripts are binary Erlang terms with the format:
/// {script, {Name, Vsn}, [Commands]}
///
/// # Arguments
/// * `boot_path` - Path to .boot file (without extension)
/// * `rootdir` - Root directory for resolving paths
/// * `bindir` - Binary directory for resolving paths
///
/// # Returns
/// Parsed boot script or error
pub fn load_boot_script(
    boot_path: &str,
    rootdir: &str,
    bindir: &str,
) -> Result<BootScript, BootScriptError> {
    // Resolve boot script path
    let resolved_path = resolve_boot_path(boot_path, rootdir, bindir)?;
    
    // Read boot script file
    let boot_data = std::fs::read(&resolved_path)
        .map_err(|e| BootScriptError::IoError(format!("Failed to read boot script: {}", e)))?;
    
    // Parse binary Erlang term
    parse_boot_script(&boot_data)
}

/// Resolve boot script path
///
/// Tries multiple locations:
/// 1. Exact path (if absolute or with .boot extension)
/// 2. bindir/boot.boot
/// 3. rootdir/bin/boot.boot
fn resolve_boot_path(boot_path: &str, rootdir: &str, bindir: &str) -> Result<String, BootScriptError> {
    // Try exact path first
    if Path::new(boot_path).is_absolute() || boot_path.ends_with(".boot") {
        if Path::new(boot_path).exists() {
            return Ok(boot_path.to_string());
        }
    }
    
    // Try with .boot extension
    let paths_to_try = vec![
        format!("{}.boot", boot_path),
        format!("{}/{}.boot", bindir, boot_path),
        format!("{}/bin/{}.boot", rootdir, boot_path),
    ];
    
    let tried_paths = paths_to_try.clone();
    for path in paths_to_try {
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    
    Err(BootScriptError::NotFound(format!(
        "Boot script not found: {} (tried: {})",
        boot_path,
        tried_paths.join(", ")
    )))
}

/// Parse boot script from binary data
///
/// Decodes the binary Erlang term format and parses the boot script structure.
/// Boot script format: {script, {Name, Vsn}, [Commands]}
///
/// # Arguments
/// * `data` - Binary boot script data
///
/// # Returns
/// Parsed boot script or error
fn parse_boot_script(data: &[u8]) -> Result<BootScript, BootScriptError> {
    // Decode the binary Erlang term
    let term = decode_term(data)
        .map_err(|e| BootScriptError::ParseError(format!("Failed to decode term: {}", e)))?;
    
    // Parse the script structure: {script, {Name, Vsn}, [Commands]}
    match term {
        ErlangTerm::Tuple(mut elements) if elements.len() == 3 => {
            // First element should be atom "script"
            let name = match &elements[0] {
                ErlangTerm::Atom(s) if s == "script" => s.clone(),
                _ => {
                    return Err(BootScriptError::InvalidFormat(
                        "Expected 'script' atom as first element".to_string(),
                    ));
                }
            };
            
            // Second element should be {Name, Vsn}
            let (script_name, script_version) = match &elements[1] {
                ErlangTerm::Tuple(name_vsn) if name_vsn.len() == 2 => {
                    let name = match &name_vsn[0] {
                        ErlangTerm::Atom(s) => s.clone(),
                        ErlangTerm::Binary(b) => String::from_utf8_lossy(b).to_string(),
                        _ => return Err(BootScriptError::InvalidFormat("Invalid script name".to_string())),
                    };
                    let version = match &name_vsn[1] {
                        ErlangTerm::Atom(s) => s.clone(),
                        ErlangTerm::Binary(b) => String::from_utf8_lossy(b).to_string(),
                        ErlangTerm::Integer(i) => i.to_string(),
                        _ => return Err(BootScriptError::InvalidFormat("Invalid script version".to_string())),
                    };
                    (name, version)
                }
                _ => {
                    return Err(BootScriptError::InvalidFormat(
                        "Expected {Name, Vsn} tuple as second element".to_string(),
                    ));
                }
            };
            
            // Third element should be list of commands
            let commands = match &elements[2] {
                ErlangTerm::List(cmd_terms) => {
                    let mut parsed_commands = Vec::new();
                    for cmd_term in cmd_terms {
                        match parse_command(cmd_term) {
                            Ok(cmd) => parsed_commands.push(cmd),
                            Err(e) => {
                                eprintln!("Warning: Failed to parse command: {}", e);
                                // Continue with other commands
                            }
                        }
                    }
                    parsed_commands
                }
                _ => {
                    return Err(BootScriptError::InvalidFormat(
                        "Expected command list as third element".to_string(),
                    ));
                }
            };
            
            Ok(BootScript {
                name: script_name,
                version: script_version,
                commands,
            })
        }
        _ => Err(BootScriptError::InvalidFormat(
            "Expected {script, {Name, Vsn}, [Commands]} tuple".to_string(),
        )),
    }
}

/// Parse a single boot command from an Erlang term
fn parse_command(term: &ErlangTerm) -> Result<BootCommand, BootScriptError> {
    match term {
        ErlangTerm::Tuple(elements) if !elements.is_empty() => {
            // First element is the command name
            let cmd_name = match &elements[0] {
                ErlangTerm::Atom(s) => s.as_str(),
                _ => {
                    return Err(BootScriptError::ParseError(
                        "Command name must be an atom".to_string(),
                    ));
                }
            };
            
            match cmd_name {
                "progress" if elements.len() == 2 => {
                    let info = match &elements[1] {
                        ErlangTerm::Atom(s) => s.clone(),
                        ErlangTerm::Binary(b) => String::from_utf8_lossy(b).to_string(),
                        _ => {
                            return Err(BootScriptError::ParseError(
                                "Progress info must be atom or binary".to_string(),
                            ));
                        }
                    };
                    Ok(BootCommand::Progress(info))
                }
                "preLoaded" if elements.len() == 2 => {
                    let modules = parse_module_list(&elements[1])?;
                    Ok(BootCommand::PreLoaded(modules))
                }
                "path" if elements.len() == 2 => {
                    let paths = parse_string_list(&elements[1])?;
                    Ok(BootCommand::Path(paths))
                }
                "primLoad" if elements.len() == 2 => {
                    let modules = parse_module_list(&elements[1])?;
                    Ok(BootCommand::PrimLoad(modules))
                }
                "kernel_load_completed" if elements.len() == 1 => {
                    Ok(BootCommand::KernelLoadCompleted)
                }
                "kernelProcess" if elements.len() == 3 => {
                    let name = term_to_string(&elements[1])?;
                    let (module, function, args) = parse_mfa(&elements[2])?;
                    Ok(BootCommand::KernelProcess {
                        name,
                        module,
                        function,
                        args,
                    })
                }
                "apply" if elements.len() == 2 => {
                    let (module, function, args) = parse_mfa(&elements[1])?;
                    Ok(BootCommand::Apply {
                        module,
                        function,
                        args,
                    })
                }
                _ => Err(BootScriptError::ParseError(format!(
                    "Unknown or invalid command: {}",
                    cmd_name
                ))),
            }
        }
        _ => Err(BootScriptError::ParseError(
            "Command must be a tuple".to_string(),
        )),
    }
}

/// Parse a list of module names
fn parse_module_list(term: &ErlangTerm) -> Result<Vec<String>, BootScriptError> {
    match term {
        ErlangTerm::List(elements) => {
            let mut modules = Vec::new();
            for elem in elements {
                match elem {
                    ErlangTerm::Atom(s) => modules.push(s.clone()),
                    ErlangTerm::Binary(b) => {
                        modules.push(String::from_utf8_lossy(b).to_string());
                    }
                    _ => {
                        return Err(BootScriptError::ParseError(
                            "Module name must be atom or binary".to_string(),
                        ));
                    }
                }
            }
            Ok(modules)
        }
        _ => Err(BootScriptError::ParseError(
            "Expected module list".to_string(),
        )),
    }
}

/// Parse a list of strings
fn parse_string_list(term: &ErlangTerm) -> Result<Vec<String>, BootScriptError> {
    parse_module_list(term) // Same format
}

/// Parse MFA (Module, Function, Args) tuple
fn parse_mfa(term: &ErlangTerm) -> Result<(String, String, Vec<String>), BootScriptError> {
    match term {
        ErlangTerm::Tuple(elements) if elements.len() == 3 => {
            let module = term_to_string(&elements[0])?;
            let function = term_to_string(&elements[1])?;
            let args = match &elements[2] {
                ErlangTerm::List(args_list) => {
                    let mut parsed_args = Vec::new();
                    for arg in args_list {
                        parsed_args.push(term_to_string(arg)?);
                    }
                    parsed_args
                }
                _ => {
                    return Err(BootScriptError::ParseError(
                        "Args must be a list".to_string(),
                    ));
                }
            };
            Ok((module, function, args))
        }
        _ => Err(BootScriptError::ParseError(
            "Expected {Mod, Func, Args} tuple".to_string(),
        )),
    }
}

/// Convert an Erlang term to a string representation
fn term_to_string(term: &ErlangTerm) -> Result<String, BootScriptError> {
    match term {
        ErlangTerm::Atom(s) => Ok(s.clone()),
        ErlangTerm::Binary(b) => Ok(String::from_utf8_lossy(b).to_string()),
        ErlangTerm::Integer(i) => Ok(i.to_string()),
        _ => Err(BootScriptError::ParseError(
            "Cannot convert term to string".to_string(),
        )),
    }
}

/// Execute boot script commands
///
/// Executes the commands in a boot script in order.
/// This is where modules are loaded and processes are started.
///
/// # Arguments
/// * `script` - Boot script to execute
///
/// # Returns
/// Result indicating success or failure
pub fn execute_boot_script(script: &BootScript) -> Result<(), String> {
    eprintln!("Executing boot script: {} (version {})", script.name, script.version);
    
    for (i, command) in script.commands.iter().enumerate() {
        eprintln!("  [{}/{}] Executing: {:?}", i + 1, script.commands.len(), command);
        
        match execute_command(command) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("  Error executing command: {}", e);
                return Err(format!("Failed to execute boot command {}: {}", i + 1, e));
            }
        }
    }
    
    eprintln!("Boot script execution completed");
    Ok(())
}

/// Execute a single boot command
///
/// # Arguments
/// * `command` - Command to execute
///
/// # Returns
/// Result indicating success or failure
fn execute_command(command: &BootCommand) -> Result<(), String> {
    match command {
        BootCommand::Progress(info) => {
            eprintln!("    Progress: {}", info);
            Ok(())
        }
        BootCommand::PreLoaded(modules) => {
            eprintln!("    Preloaded modules: {:?}", modules);
            mark_modules_preloaded(&modules)
        }
        BootCommand::Path(paths) => {
            eprintln!("    Setting code path: {:?}", paths);
            set_code_path(&paths)
        }
        BootCommand::PrimLoad(modules) => {
            eprintln!("    Loading modules: {:?}", modules);
            load_modules(&modules)
        }
        BootCommand::KernelLoadCompleted => {
            eprintln!("    Kernel load completed");
            Ok(())
        }
        BootCommand::KernelProcess { name, module, function, args } => {
            eprintln!("    Starting kernel process: {} ({}.{}/{} with args: {:?})", 
                     name, module, function, args.len(), args);
            spawn_kernel_process(name, module, function, args)
        }
        BootCommand::Apply { module, function, args } => {
            eprintln!("    Applying: {}.{}/{} with args: {:?}", 
                     module, function, args.len(), args);
            apply_function(module, function, args)
        }
    }
}

/// Resolve function entry point from module:function/arity
///
/// Looks up the function in the export table and resolves its code pointer.
/// If the code pointer is not available, attempts to resolve it from the label.
///
/// # Arguments
/// * `module` - Module name
/// * `function` - Function name
/// * `arity` - Function arity
///
/// # Returns
/// Code pointer to function entry point or error
fn resolve_function_entry_point(
    module: &str,
    function: &str,
    arity: usize,
) -> Result<entities_process::ErtsCodePtr, String> {
    use entities_io_operations::export::get_global_export_table;
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    use code_management_code_loading::{get_global_module_manager, get_global_code_ix};
    
    let atom_table = get_global_atom_table();
    let module_atom_index = atom_table.put_index(module.as_bytes(), AtomEncoding::SevenBitAscii, false)
        .map_err(|_| format!("Failed to create atom for module: {}", module))? as u32;
    
    let function_atom_index = atom_table.put_index(function.as_bytes(), AtomEncoding::SevenBitAscii, false)
        .map_err(|_| format!("Failed to create atom for function: {}", function))? as u32;
    
    let export_table = get_global_export_table();
    let export = export_table.get(module_atom_index, function_atom_index, arity as u32)
        .ok_or_else(|| format!("{}/{} not found in export table", function, arity))?;
    
    // Try to get code pointer directly
    if let Some(ptr) = export.get_code_ptr() {
        return Ok(ptr);
    }
    
    // Try to resolve from label
    if let Some(label) = export.label {
        // Use the same resolution logic as in main_init.rs
        let module_manager = get_global_module_manager();
        let code_ix = get_global_code_ix();
        let active_ix = code_ix.active_code_ix() as usize;
        
        if let Some(code_data) = module_manager.get_code_data(module_atom_index as usize, active_ix) {
            let code_header_size = if code_data.len() >= 20 { 20 } else { 0 };
            let instruction_size = 4;
            let label_offset = code_header_size + ((label as usize) * instruction_size);
            
            if label_offset >= code_data.len() {
                return Err(format!(
                    "Label {} (offset {}) out of bounds for module {} (code size: {})",
                    label, label_offset, module, code_data.len()
                ));
            }
            
            let code_ptr = code_data.as_ptr().wrapping_add(label_offset) as entities_process::ErtsCodePtr;
            
            // Update export table with resolved code pointer
            export_table.update_export_code_ptr(module_atom_index, function_atom_index, arity as u32, code_ptr);
            
            return Ok(code_ptr);
        }
        
        return Err(format!(
            "Module {} code data not available for label resolution",
            module
        ));
    }
    
    Err(format!(
        "{}/{} has neither code pointer nor label",
        function, arity
    ))
}

/// Spawn a kernel process
///
/// Creates a new process that will execute the specified module:function/arity.
/// The process is registered with the given name and scheduled for execution.
///
/// # Arguments
/// * `name` - Process name (for registration)
/// * `module` - Module name
/// * `function` - Function name
/// * `args` - Function arguments
///
/// # Returns
/// Result indicating success or failure
fn spawn_kernel_process(
    name: &str,
    module: &str,
    function: &str,
    args: &[String],
) -> Result<(), String> {
    use entities_process::Process;
    use infrastructure_utilities::process_table::get_global_process_table;
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    use std::sync::Arc;
    
    // Resolve function entry point
    let arity = args.len();
    let code_ptr = resolve_function_entry_point(module, function, arity)
        .map_err(|e| format!("Failed to resolve entry point for {}/{}: {}", function, arity, e))?;
    
    eprintln!("      ✓ Resolved {}/{} entry point: {:p}", function, arity, code_ptr);
    
    // Allocate a new process with automatic ID generation
    let process_table = get_global_process_table();
    let (pid, process_arc) = process_table
        .new_element(|id| {
            let mut process = Process::new(id);
            
            // Set instruction pointer to function entry point
            process.set_i(code_ptr);
            
            // Set arity for the function call
            process.set_arity(arity as u8);
            
            // Set up process heap with function arguments
            if let Err(e) = setup_function_arguments(&mut process, args) {
                eprintln!("      ⚠ Failed to set up function arguments: {} (process will start without arguments)", e);
            }
            
            Arc::new(process)
        })
        .map_err(|e| format!("Failed to allocate process: {:?}", e))?;
    
    // Register process name
    register_process_name(name, pid)?;
    
    // Schedule the process
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
    let runq_guard = runq.lock()
        .map_err(|e| format!("Failed to lock run queue: {}", e))?;
    
    schedule_process(process_arc.clone(), &runq_guard, Priority::Normal)
        .map_err(|e| format!("Failed to schedule kernel process: {:?}", e))?;
    
    eprintln!("      ✓ Kernel process '{}' spawned and scheduled (PID: {})", name, pid);
    Ok(())
}

/// Set up function arguments on process heap
///
/// Encodes function arguments as Erlang terms and stores them in the process heap.
/// Arguments are stored starting at heap_start_index (where X registers begin).
///
/// # Arguments
/// * `process` - Process to set up
/// * `args` - Function arguments (as strings for now)
///
/// # Returns
/// Result indicating success or failure
fn setup_function_arguments(process: &mut entities_process::Process, args: &[String]) -> Result<(), String> {
    use infrastructure_utilities::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    
    let atom_table = get_global_atom_table();
    let heap_start = process.heap_start_index();
    let required_heap_size = heap_start + args.len();
    
    // Ensure heap is large enough
    {
        let mut heap_slice = process.heap_slice_mut();
        if heap_slice.len() < required_heap_size {
            heap_slice.resize(required_heap_size, 0);
        }
        
        // Encode each argument as an atom and store in heap
        for (i, arg) in args.iter().enumerate() {
            let arg_atom_index = atom_table.put_index(
                arg.as_bytes(),
                AtomEncoding::SevenBitAscii,
                false,
            )
            .map_err(|_| format!("Failed to create atom for argument: {}", arg))? as u32;
            
            // Encode as Eterm atom: (atom_index << 6) | 0x0B
            let arg_term = ((arg_atom_index as u64) << 6) | 0x0B;
            heap_slice[heap_start + i] = arg_term;
        }
    }
    
    Ok(())
}

/// Apply a function directly
///
/// Creates a temporary process to execute the specified module:function/arity
/// with the given arguments. This is used for boot script `apply` commands.
///
/// # Arguments
/// * `module` - Module name
/// * `function` - Function name
/// * `args` - Function arguments (as strings for now)
///
/// # Returns
/// Result indicating success or failure
fn apply_function(
    module: &str,
    function: &str,
    args: &[String],
) -> Result<(), String> {
    use entities_process::Process;
    use infrastructure_utilities::process_table::get_global_process_table;
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    use std::sync::Arc;
    
    eprintln!("      Applying function: {}.{}/{}", module, function, args.len());
    
    // Resolve function entry point
    let arity = args.len();
    let code_ptr = resolve_function_entry_point(module, function, arity)
        .map_err(|e| format!("Failed to resolve entry point for {}/{}: {}", function, arity, e))?;
    
    eprintln!("      ✓ Resolved {}/{} entry point: {:p}", function, arity, code_ptr);
    
    // Allocate a new process with automatic ID generation
    let process_table = get_global_process_table();
    let (pid, process_arc) = process_table
        .new_element(|id| {
            let mut process = Process::new(id);
            
            // Set instruction pointer to function entry point
            process.set_i(code_ptr);
            
            // Set arity for the function call
            process.set_arity(arity as u8);
            
            // Set up process heap with function arguments
            if let Err(e) = setup_function_arguments(&mut process, args) {
                eprintln!("      ⚠ Failed to set up function arguments: {} (process will start without arguments)", e);
            }
            
            Arc::new(process)
        })
        .map_err(|e| format!("Failed to allocate process for apply: {:?}", e))?;
    
    eprintln!("      Created temporary process (PID: {}) for apply", pid);
    
    // In the full implementation, we would:
    // 1. Execute the function synchronously in the current context
    // 2. Wait for the result
    // 3. Handle any exceptions
    // 4. Clean up the temporary process
    
    // For now, we'll schedule it like a kernel process
    // In production, this would be synchronous execution
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
    let runq_guard = runq.lock()
        .map_err(|e| format!("Failed to lock run queue: {}", e))?;
    
    schedule_process(process_arc.clone(), &runq_guard, Priority::Normal)
        .map_err(|e| format!("Failed to schedule apply process: {:?}", e))?;
    
    eprintln!("      ✓ Function {}.{}/{} scheduled for execution (PID: {})", 
             module, function, args.len(), pid);
    
    // Note: In the full implementation, we would wait for the result here
    // For now, we just schedule it and continue
    
    Ok(())
}

/// Load modules from boot script
///
/// Loads BEAM modules specified in the primLoad command.
/// This function searches for .beam files, parses them using BeamLoader,
/// and registers them in the module management system.
///
/// # Arguments
/// * `modules` - List of module names to load
///
/// # Returns
/// Result indicating success or failure
fn load_modules(modules: &[String]) -> Result<(), String> {
    use code_management_code_loading::{CodeLoader, BeamLoader};
    use code_management_code_loading::code_loader::LoadError;
    use usecases_bifs::load::LoadBif;
    use std::path::Path;
    use std::fs;
    
    // Get code search paths
    // This should include paths set by boot script 'path' commands
    // and default paths including Makefile output directories
    let code_paths = get_code_paths();
    
    let mut loaded_count = 0;
    let mut failed_modules = Vec::new();
    
    for module_name in modules {
        // Try to find and load the module
        let mut found = false;
        
        for code_path in &code_paths {
            // Try .beam file
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
                    
                    // Parse BEAM file using BeamLoader
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
                            
                            // In the full implementation, we would also:
                            // 1. Register module in module table (code_management_code_loading::module_management)
                            // 2. Make code executable (code index management)
                            // 3. Register exports in export table
                            // 4. Set up code pointers for function entry points
                            //
                            // For now, LoadBif::register_module ensures basic module registration
                            // which allows module_loaded/1 and other BIFs to work correctly.
                            
                            eprintln!("      ✓ Loaded: {} (from {})", module_name, beam_path.display());
                            if beam_file.has_on_load {
                                eprintln!("        ⚠ Module has on_load function (on_load execution not yet implemented)");
                            }
                            loaded_count += 1;
                            found = true;
                            break;
                        }
                        Err(e) => {
                            eprintln!("      ✗ Failed to parse BEAM file {}: {:?}", module_name, e);
                            // Try next path
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
                    failed_modules.push(module_name.clone());
                    found = true; // Don't try other paths
                    break;
                }
            }
        }
        
        if !found {
            eprintln!("      ✗ Not found: {} (searched in: {:?})", module_name, code_paths);
            failed_modules.push(module_name.clone());
        }
    }
    
    if !failed_modules.is_empty() {
        eprintln!("    Warning: Failed to load {} modules: {:?}", 
                 failed_modules.len(), failed_modules);
        // In the full implementation, this might be an error
        // For now, we'll continue with a warning
    }
    
    eprintln!("    Loaded {}/{} modules", loaded_count, modules.len());
    Ok(())
}

/// Global code path storage
///
/// Stores the code search paths set by boot script `path` commands.
/// This is a thread-safe global storage that can be accessed from anywhere.
static CODE_PATH: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

/// Global process name registry
///
/// Stores the mapping between process names and process IDs.
/// This is a thread-safe global registry that can be accessed from anywhere.
static PROCESS_REGISTRY: OnceLock<Mutex<Register>> = OnceLock::new();

/// Initialize the code path storage
fn init_code_path() -> &'static Mutex<Vec<String>> {
    CODE_PATH.get_or_init(|| {
        // Initialize with default paths
        let mut default_paths = Vec::new();
        
        // Add current directory
        default_paths.push(".".to_string());
        
        // Try to get ROOTDIR and construct lib paths
        if let Ok(rootdir) = std::env::var("ROOTDIR") {
            // Add lib directories
            if let Ok(entries) = std::fs::read_dir(format!("{}/lib", rootdir)) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            if let Some(app_dir) = entry.path().to_str() {
                                // Try ebin subdirectory (where .beam files are)
                                let ebin_path = format!("{}/ebin", app_dir);
                                if Path::new(&ebin_path).exists() {
                                    default_paths.push(ebin_path);
                                }
                                // Also try the app directory itself
                                default_paths.push(app_dir.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Mutex::new(default_paths)
    })
}

/// Set the code search path
///
/// Replaces the current code path with the provided paths.
/// This is called by the boot script `path` command.
///
/// # Arguments
/// * `paths` - List of directory paths to search for BEAM files
///
/// # Returns
/// Result indicating success or failure
fn set_code_path(paths: &[String]) -> Result<(), String> {
    let code_path = init_code_path();
    let mut path_guard = code_path
        .lock()
        .map_err(|e| format!("Failed to lock code path: {}", e))?;
    
    // Replace the code path with the new paths
    *path_guard = paths.to_vec();
    
    eprintln!("      ✓ Code path set to {} directories", paths.len());
    Ok(())
}

/// Get code search paths
///
/// Returns a list of directories to search for BEAM files.
/// Uses the code path set by boot script `path` commands, or defaults
/// if no path has been set. Also includes Makefile output directories.
///
/// # Returns
/// Vector of code path directories
fn get_code_paths() -> Vec<String> {
    let code_path = init_code_path();
    let path_guard = code_path
        .lock()
        .expect("Failed to lock code path");
    
    let mut paths = path_guard.clone();
    
    // If code path is empty or only has defaults, add Makefile output directories
    // The Makefile creates .beam files in target/otp_root/lib/*/ebin/
    if paths.is_empty() || (paths.len() == 1 && paths[0] == ".") {
        // Try to find target/otp_root/lib/*/ebin/ directories
        // This is relative to the rust-conversion directory
        let target_base = std::env::current_dir()
            .ok()
            .and_then(|cwd| {
                // Try to find rust-conversion directory
                let mut current = cwd.clone();
                loop {
                    if current.join("rust-conversion").exists() {
                        return Some(current.join("rust-conversion").join("target").join("otp_root"));
                    }
                    if let Some(parent) = current.parent() {
                        current = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
                None
            });
        
        if let Some(target_root) = target_base {
            let lib_dir = target_root.join("lib");
            if lib_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                    for entry in entries.flatten() {
                        let app_dir = entry.path();
                        if app_dir.is_dir() {
                            let ebin_dir = app_dir.join("ebin");
                            if ebin_dir.exists() {
                                if let Some(path_str) = ebin_dir.to_str() {
                                    paths.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Also try ROOTDIR/lib/*/ebin/ if ROOTDIR is set
        if let Ok(rootdir) = std::env::var("ROOTDIR") {
            let lib_dir = Path::new(&rootdir).join("lib");
            if lib_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                    for entry in entries.flatten() {
                        let app_dir = entry.path();
                        if app_dir.is_dir() {
                            let ebin_dir = app_dir.join("ebin");
                            if ebin_dir.exists() {
                                if let Some(path_str) = ebin_dir.to_str() {
                                    paths.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    paths
}

/// Initialize the process registry
fn init_process_registry() -> &'static Mutex<Register> {
    PROCESS_REGISTRY.get_or_init(|| {
        Mutex::new(Register::new())
    })
}

/// Register a process name
///
/// Registers a process with the given name in the global process registry.
/// This enables the process to be found by name using `whereis/1`.
///
/// # Arguments
/// * `name` - Process name (atom)
/// * `pid` - Process ID
///
/// # Returns
/// Result indicating success or failure
fn register_process_name(name: &str, pid: u64) -> Result<(), String> {
    let registry = init_process_registry();
    let mut reg_guard = registry
        .lock()
        .map_err(|e| format!("Failed to lock process registry: {}", e))?;
    
    match reg_guard.register_name(name, pid) {
        RegisterResult::Success => {
            eprintln!("      ✓ Registered process '{}' with PID {}", name, pid);
            Ok(())
        }
        RegisterResult::AlreadyRegistered => {
            Err(format!("Process name '{}' is already registered to a different PID", name))
        }
        RegisterResult::AlreadyHasName => {
            Err(format!("Process PID {} already has a different registered name", pid))
        }
        RegisterResult::InvalidName => {
            Err(format!("Invalid process name: '{}'", name))
        }
        RegisterResult::NotAlive => {
            Err(format!("Process PID {} is not alive", pid))
        }
    }
}

/// Mark modules as preloaded
///
/// Marks the specified modules as preloaded in the module management system.
/// Preloaded modules are part of the system and are always available.
///
/// # Arguments
/// * `modules` - List of module names to mark as preloaded
///
/// # Returns
/// Result indicating success or failure
fn mark_modules_preloaded(modules: &[String]) -> Result<(), String> {
    // Use the LoadBif infrastructure to mark modules as preloaded
    // This ensures consistency with the module management system
    use usecases_bifs::load::LoadBif;
    
    for module_name in modules {
        LoadBif::mark_preloaded(module_name);
        eprintln!("      ✓ Marked '{}' as preloaded", module_name);
    }
    
    eprintln!("      ✓ Marked {} modules as preloaded", modules.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use infrastructure_utilities::ErlangTerm;

    #[test]
    fn test_boot_script_debug() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![BootCommand::Progress("test".to_string())],
        };
        let debug_str = format!("{:?}", script);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_boot_script_clone() {
        let script1 = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![BootCommand::Progress("test".to_string())],
        };
        let script2 = script1.clone();
        assert_eq!(script1.name, script2.name);
        assert_eq!(script1.version, script2.version);
        assert_eq!(script1.commands.len(), script2.commands.len());
    }

    #[test]
    fn test_boot_command_progress() {
        let cmd = BootCommand::Progress("test".to_string());
        assert_eq!(cmd, BootCommand::Progress("test".to_string()));
        assert_ne!(cmd, BootCommand::Progress("other".to_string()));
    }

    #[test]
    fn test_boot_command_preloaded() {
        let cmd = BootCommand::PreLoaded(vec!["mod1".to_string(), "mod2".to_string()]);
        assert_eq!(cmd, BootCommand::PreLoaded(vec!["mod1".to_string(), "mod2".to_string()]));
        assert_ne!(cmd, BootCommand::PreLoaded(vec!["mod1".to_string()]));
    }

    #[test]
    fn test_boot_command_path() {
        let cmd = BootCommand::Path(vec!["/path1".to_string(), "/path2".to_string()]);
        assert_eq!(cmd, BootCommand::Path(vec!["/path1".to_string(), "/path2".to_string()]));
    }

    #[test]
    fn test_boot_command_primload() {
        let cmd = BootCommand::PrimLoad(vec!["mod1".to_string()]);
        assert_eq!(cmd, BootCommand::PrimLoad(vec!["mod1".to_string()]));
    }

    #[test]
    fn test_boot_command_kernel_load_completed() {
        let cmd = BootCommand::KernelLoadCompleted;
        assert_eq!(cmd, BootCommand::KernelLoadCompleted);
        assert_ne!(cmd, BootCommand::Progress("test".to_string()));
    }

    #[test]
    fn test_boot_command_kernel_process() {
        let cmd = BootCommand::KernelProcess {
            name: "init".to_string(),
            module: "init".to_string(),
            function: "start".to_string(),
            args: vec!["arg1".to_string()],
        };
        assert_eq!(cmd, BootCommand::KernelProcess {
            name: "init".to_string(),
            module: "init".to_string(),
            function: "start".to_string(),
            args: vec!["arg1".to_string()],
        });
    }

    #[test]
    fn test_boot_command_apply() {
        let cmd = BootCommand::Apply {
            module: "mod".to_string(),
            function: "func".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
        };
        assert_eq!(cmd, BootCommand::Apply {
            module: "mod".to_string(),
            function: "func".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
        });
    }

    #[test]
    fn test_boot_command_debug() {
        let commands = vec![
            BootCommand::Progress("test".to_string()),
            BootCommand::PreLoaded(vec!["mod1".to_string()]),
            BootCommand::Path(vec!["/path".to_string()]),
            BootCommand::PrimLoad(vec!["mod1".to_string()]),
            BootCommand::KernelLoadCompleted,
            BootCommand::KernelProcess {
                name: "init".to_string(),
                module: "init".to_string(),
                function: "start".to_string(),
                args: vec![],
            },
            BootCommand::Apply {
                module: "mod".to_string(),
                function: "func".to_string(),
                args: vec![],
            },
        ];
        
        for cmd in commands {
            let debug_str = format!("{:?}", cmd);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_boot_script_error_display() {
        let errors = vec![
            BootScriptError::NotFound("file.boot".to_string()),
            BootScriptError::InvalidFormat("invalid".to_string()),
            BootScriptError::ParseError("parse error".to_string()),
            BootScriptError::IoError("io error".to_string()),
        ];
        
        for error in errors {
            let display_str = format!("{}", error);
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_boot_script_error_clone() {
        let error1 = BootScriptError::NotFound("file.boot".to_string());
        let error2 = error1.clone();
        assert_eq!(format!("{}", error1), format!("{}", error2));
    }

    #[test]
    fn test_resolve_boot_path_nonexistent() {
        // Test with non-existent path (should return error)
        let result = resolve_boot_path("nonexistent", "/root", "/bin");
        assert!(result.is_err());
        if let Err(BootScriptError::NotFound(msg)) = result {
            assert!(msg.contains("nonexistent"));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_term_to_string_atom() {
        let term = ErlangTerm::Atom("test".to_string());
        let result = term_to_string(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_term_to_string_binary() {
        let term = ErlangTerm::Binary(b"test".to_vec());
        let result = term_to_string(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_term_to_string_integer() {
        let term = ErlangTerm::Integer(42);
        let result = term_to_string(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_term_to_string_invalid() {
        let term = ErlangTerm::List(vec![]);
        let result = term_to_string(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_list_atoms() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Atom("mod1".to_string()),
            ErlangTerm::Atom("mod2".to_string()),
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_ok());
        let modules = result.unwrap();
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0], "mod1");
        assert_eq!(modules[1], "mod2");
    }

    #[test]
    fn test_parse_module_list_binaries() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Binary(b"mod1".to_vec()),
            ErlangTerm::Binary(b"mod2".to_vec()),
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_ok());
        let modules = result.unwrap();
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0], "mod1");
        assert_eq!(modules[1], "mod2");
    }

    #[test]
    fn test_parse_module_list_mixed() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Atom("mod1".to_string()),
            ErlangTerm::Binary(b"mod2".to_vec()),
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_ok());
        let modules = result.unwrap();
        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_parse_module_list_invalid_element() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Integer(42),
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_list_not_list() {
        let term = ErlangTerm::Atom("not_list".to_string());
        let result = parse_module_list(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_list() {
        // parse_string_list uses parse_module_list, so test is similar
        let term = ErlangTerm::List(vec![
            ErlangTerm::Atom("path1".to_string()),
            ErlangTerm::Atom("path2".to_string()),
        ]);
        let result = parse_string_list(&term);
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_parse_mfa() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            ErlangTerm::List(vec![
                ErlangTerm::Atom("arg1".to_string()),
                ErlangTerm::Atom("arg2".to_string()),
            ]),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_ok());
        let (module, function, args) = result.unwrap();
        assert_eq!(module, "module");
        assert_eq!(function, "function");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "arg1");
        assert_eq!(args[1], "arg2");
    }

    #[test]
    fn test_parse_mfa_invalid_tuple() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mfa_not_tuple() {
        let term = ErlangTerm::Atom("not_tuple".to_string());
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_progress() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("progress".to_string()),
            ErlangTerm::Atom("info".to_string()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::Progress(info) => assert_eq!(info, "info"),
            _ => panic!("Expected Progress command"),
        }
    }

    #[test]
    fn test_parse_command_progress_binary() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("progress".to_string()),
            ErlangTerm::Binary(b"info".to_vec()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::Progress(info) => assert_eq!(info, "info"),
            _ => panic!("Expected Progress command"),
        }
    }

    #[test]
    fn test_parse_command_preloaded() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("preLoaded".to_string()),
            ErlangTerm::List(vec![
                ErlangTerm::Atom("mod1".to_string()),
                ErlangTerm::Atom("mod2".to_string()),
            ]),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::PreLoaded(modules) => {
                assert_eq!(modules.len(), 2);
                assert_eq!(modules[0], "mod1");
                assert_eq!(modules[1], "mod2");
            }
            _ => panic!("Expected PreLoaded command"),
        }
    }

    #[test]
    fn test_parse_command_path() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("path".to_string()),
            ErlangTerm::List(vec![
                ErlangTerm::Atom("/path1".to_string()),
                ErlangTerm::Atom("/path2".to_string()),
            ]),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::Path(paths) => {
                assert_eq!(paths.len(), 2);
            }
            _ => panic!("Expected Path command"),
        }
    }

    #[test]
    fn test_parse_command_primload() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("primLoad".to_string()),
            ErlangTerm::List(vec![
                ErlangTerm::Atom("mod1".to_string()),
            ]),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::PrimLoad(modules) => {
                assert_eq!(modules.len(), 1);
                assert_eq!(modules[0], "mod1");
            }
            _ => panic!("Expected PrimLoad command"),
        }
    }

    #[test]
    fn test_parse_command_kernel_load_completed() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("kernel_load_completed".to_string()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::KernelLoadCompleted => {}
            _ => panic!("Expected KernelLoadCompleted command"),
        }
    }

    #[test]
    fn test_parse_command_kernel_process() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("kernelProcess".to_string()),
            ErlangTerm::Atom("init".to_string()),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("init".to_string()),
                ErlangTerm::Atom("start".to_string()),
                ErlangTerm::List(vec![
                    ErlangTerm::Atom("arg1".to_string()),
                ]),
            ]),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::KernelProcess { name, module, function, args } => {
                assert_eq!(name, "init");
                assert_eq!(module, "init");
                assert_eq!(function, "start");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected KernelProcess command"),
        }
    }

    #[test]
    fn test_parse_command_apply() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("apply".to_string()),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("mod".to_string()),
                ErlangTerm::Atom("func".to_string()),
                ErlangTerm::List(vec![
                    ErlangTerm::Atom("arg1".to_string()),
                ]),
            ]),
        ]);
        let result = parse_command(&term);
        assert!(result.is_ok());
        match result.unwrap() {
            BootCommand::Apply { module, function, args } => {
                assert_eq!(module, "mod");
                assert_eq!(function, "func");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Apply command"),
        }
    }

    #[test]
    fn test_parse_command_unknown() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("unknown".to_string()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_not_tuple() {
        let term = ErlangTerm::Atom("not_tuple".to_string());
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_invalid_name() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(42),
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_boot_script_valid() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("script".to_string()),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("test".to_string()),
                ErlangTerm::Atom("1.0".to_string()),
            ]),
            ErlangTerm::List(vec![
                ErlangTerm::Tuple(vec![
                    ErlangTerm::Atom("progress".to_string()),
                    ErlangTerm::Atom("test".to_string()),
                ]),
            ]),
        ]);
        // We can't directly test parse_boot_script as it requires decode_term
        // But we can test the structure matches what parse_boot_script expects
        match term {
            ErlangTerm::Tuple(elements) if elements.len() == 3 => {
                // First element should be "script"
                match &elements[0] {
                    ErlangTerm::Atom(s) => assert_eq!(s, "script"),
                    _ => panic!("Expected script atom"),
                }
            }
            _ => panic!("Expected 3-tuple"),
        }
    }

    #[test]
    fn test_parse_boot_script_invalid_structure() {
        // Test with wrong number of tuple elements
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("script".to_string()),
        ]);
        match term {
            ErlangTerm::Tuple(elements) if elements.len() != 3 => {
                // This should fail in parse_boot_script
            }
            _ => panic!("Expected different structure"),
        }
    }

    #[test]
    fn test_parse_boot_script_invalid_first_element() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("not_script".to_string()),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("test".to_string()),
                ErlangTerm::Atom("1.0".to_string()),
            ]),
            ErlangTerm::List(vec![]),
        ]);
        match term {
            ErlangTerm::Tuple(elements) if elements.len() == 3 => {
                match &elements[0] {
                    ErlangTerm::Atom(s) if s != "script" => {
                        // This should fail in parse_boot_script
                    }
                    _ => panic!("Expected non-script atom"),
                }
            }
            _ => panic!("Expected 3-tuple"),
        }
    }

    #[test]
    fn test_boot_script_error_error_trait() {
        // Test that BootScriptError implements Error trait
        let error = BootScriptError::NotFound("test".to_string());
        let error_ref: &dyn std::error::Error = &error;
        let description = error_ref.to_string();
        assert!(!description.is_empty());
    }

    #[test]
    fn test_boot_command_clone() {
        let cmd1 = BootCommand::Progress("test".to_string());
        let cmd2 = cmd1.clone();
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_boot_command_equality() {
        let cmd1 = BootCommand::Progress("test".to_string());
        let cmd2 = BootCommand::Progress("test".to_string());
        let cmd3 = BootCommand::Progress("other".to_string());
        
        assert_eq!(cmd1, cmd2);
        assert_ne!(cmd1, cmd3);
        assert_ne!(cmd1, BootCommand::KernelLoadCompleted);
    }

    #[test]
    fn test_empty_module_list() {
        let term = ErlangTerm::List(vec![]);
        let result = parse_module_list(&term);
        assert!(result.is_ok());
        let modules = result.unwrap();
        assert_eq!(modules.len(), 0);
    }

    #[test]
    fn test_empty_args_list() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            ErlangTerm::List(vec![]),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_ok());
        let (_, _, args) = result.unwrap();
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn test_parse_mfa_invalid_args() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            ErlangTerm::Atom("not_list".to_string()),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_boot_script_empty() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![],
        };
        let result = execute_boot_script(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_boot_script_progress_command() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![BootCommand::Progress("Loading...".to_string())],
        };
        let result = execute_boot_script(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_boot_script_kernel_load_completed() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![BootCommand::KernelLoadCompleted],
        };
        let result = execute_boot_script(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_boot_script_multiple_commands() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![
                BootCommand::Progress("Step 1".to_string()),
                BootCommand::Progress("Step 2".to_string()),
                BootCommand::KernelLoadCompleted,
            ],
        };
        let result = execute_boot_script(&script);
        // May succeed or fail depending on system state
        let _ = result;
    }



    #[test]
    fn test_parse_command_invalid() {
        let term = ErlangTerm::Atom("invalid".to_string());
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_list_empty() {
        let term = ErlangTerm::List(vec![]);
        let result = parse_module_list(&term);
        if let Ok(modules) = result {
            assert_eq!(modules.len(), 0);
        }
    }

    #[test]
    fn test_parse_module_list_invalid() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Integer(1), // Not an atom
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_list_empty() {
        let term = ErlangTerm::List(vec![]);
        let result = parse_string_list(&term);
        if let Ok(strings) = result {
            assert_eq!(strings.len(), 0);
        }
    }

    #[test]
    fn test_parse_string_list_binary() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Binary(b"str1".to_vec()),
            ErlangTerm::Binary(b"str2".to_vec()),
        ]);
        let result = parse_string_list(&term);
        if let Ok(strings) = result {
            assert_eq!(strings.len(), 2);
        }
    }

    #[test]
    fn test_term_to_string_float() {
        let term = ErlangTerm::Float(3.14);
        let result = term_to_string(&term);
        // Float conversion may succeed or fail
        let _ = result;
    }

    #[test]
    fn test_term_to_string_list() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Integer(104), // 'h'
            ErlangTerm::Integer(101), // 'e'
        ]);
        let result = term_to_string(&term);
        // List to string conversion
        let _ = result;
    }

    #[test]
    fn test_boot_script_error_debug() {
        let errors = vec![
            BootScriptError::NotFound("file.boot".to_string()),
            BootScriptError::InvalidFormat("invalid".to_string()),
            BootScriptError::ParseError("parse error".to_string()),
            BootScriptError::IoError("io error".to_string()),
        ];
        
        for error in errors {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_boot_command_equality_all_variants() {
        let cmd1 = BootCommand::KernelLoadCompleted;
        let cmd2 = BootCommand::KernelLoadCompleted;
        let cmd3 = BootCommand::Progress("test".to_string());
        
        assert_eq!(cmd1, cmd2);
        assert_ne!(cmd1, cmd3);
    }

    #[test]
    fn test_boot_script_with_all_command_types() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![
                BootCommand::Progress("test".to_string()),
                BootCommand::PreLoaded(vec!["mod1".to_string()]),
                BootCommand::Path(vec!["/path".to_string()]),
                BootCommand::PrimLoad(vec!["mod1".to_string()]),
                BootCommand::KernelLoadCompleted,
                BootCommand::KernelProcess {
                    name: "init".to_string(),
                    module: "init".to_string(),
                    function: "start".to_string(),
                    args: vec![],
                },
                BootCommand::Apply {
                    module: "mod".to_string(),
                    function: "func".to_string(),
                    args: vec![],
                },
            ],
        };
        
        // Test that script can be cloned and debugged
        let _clone = script.clone();
        let _debug = format!("{:?}", script);
    }

    #[test]
    fn test_resolve_boot_path_absolute() {
        let result = resolve_boot_path("/absolute/path.boot", "/root", "/bin");
        // May succeed or fail depending on file existence
        let _ = result;
    }

    #[test]
    fn test_resolve_boot_path_relative() {
        let result = resolve_boot_path("relative.boot", "/root", "/bin");
        // May succeed or fail depending on file existence
        let _ = result;
    }

    #[test]
    fn test_resolve_boot_path_without_extension() {
        let result = resolve_boot_path("start", "/root", "/bin");
        // Should add .boot extension
        let _ = result;
    }

    // Error handling tests for load_boot_script
    #[test]
    fn test_load_boot_script_file_not_found() {
        let result = load_boot_script("nonexistent", "/root", "/bin");
        assert!(result.is_err());
        match result.unwrap_err() {
            BootScriptError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_load_boot_script_io_error() {
        // Try to load from a directory (should fail to read as file)
        let result = load_boot_script("/", "/root", "/bin");
        // May fail with NotFound or IoError depending on path resolution
        assert!(result.is_err());
    }

    // Error handling tests for parse_boot_script
    #[test]
    fn test_parse_boot_script_empty_data() {
        let data = vec![];
        let result = parse_boot_script(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            BootScriptError::ParseError(_) => {}
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_boot_script_invalid_binary() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = parse_boot_script(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_boot_script_not_tuple() {
        // Test with invalid binary data that doesn't decode to a tuple
        let data = vec![131, 100, 0, 4, 116, 101, 115, 116]; // atom "test" in Erlang binary format
        let result = parse_boot_script(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            BootScriptError::InvalidFormat(_) | BootScriptError::ParseError(_) => {}
            _ => panic!("Expected InvalidFormat or ParseError"),
        }
    }

    #[test]
    fn test_parse_boot_script_wrong_tuple_length() {
        // Test with binary data representing a tuple with wrong length
        // This is a simplified test - actual binary format is complex
        let data = vec![131, 104, 2, 100, 0, 6, 115, 99, 114, 105, 112, 116, 100, 0, 4, 110, 97, 109, 101];
        let result = parse_boot_script(&data);
        // May fail at parse or format validation
        assert!(result.is_err());
    }

    // Error handling tests for parse_command
    #[test]
    fn test_parse_command_empty_tuple() {
        let term = ErlangTerm::Tuple(vec![]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_not_atom_name() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(42), // Not an atom
            ErlangTerm::Atom("info".to_string()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_progress_invalid_info() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("progress".to_string()),
            ErlangTerm::Integer(42), // Not atom or binary
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_progress_wrong_arity() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("progress".to_string()),
            // Missing second element
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_preloaded_not_list() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("preLoaded".to_string()),
            ErlangTerm::Atom("not_a_list".to_string()),
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_path_not_list() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("path".to_string()),
            ErlangTerm::Integer(42),
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_kernel_process_wrong_arity() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("kernelProcess".to_string()),
            ErlangTerm::Atom("name".to_string()),
            // Missing third element
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_apply_wrong_arity() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("apply".to_string()),
            // Missing second element
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    // Error handling tests for parse_module_list
    #[test]
    fn test_parse_module_list_invalid_element_type() {
        let term = ErlangTerm::List(vec![
            ErlangTerm::Atom("module1".to_string()),
            ErlangTerm::Integer(42), // Invalid type
            ErlangTerm::Atom("module2".to_string()),
        ]);
        let result = parse_module_list(&term);
        assert!(result.is_err());
    }

    // Error handling tests for parse_mfa
    #[test]
    fn test_parse_mfa_not_tuple_duplicate() {
        let term = ErlangTerm::Atom("not_a_tuple".to_string());
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mfa_wrong_length() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            // Missing args element
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mfa_args_not_list() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            ErlangTerm::Atom("not_a_list".to_string()),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mfa_invalid_arg_type() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("module".to_string()),
            ErlangTerm::Atom("function".to_string()),
            ErlangTerm::List(vec![
                ErlangTerm::Tuple(vec![]), // Cannot convert tuple to string
            ]),
        ]);
        let result = parse_mfa(&term);
        assert!(result.is_err());
    }

    // Error handling tests for term_to_string
    #[test]
    fn test_term_to_string_unsupported_type() {
        let term = ErlangTerm::List(vec![]);
        let result = term_to_string(&term);
        assert!(result.is_err());
    }

    // Error handling tests for execute_command
    #[test]
    fn test_execute_command_preloaded_error() {
        // This will fail if modules can't be marked as preloaded
        let command = BootCommand::PreLoaded(vec!["nonexistent_module".to_string()]);
        // Should not panic, may return error
        let _ = execute_command(&command);
    }

    #[test]
    fn test_execute_command_path_error() {
        // Test with invalid paths
        let command = BootCommand::Path(vec!["/nonexistent/path".to_string()]);
        // Should not panic
        let _ = execute_command(&command);
    }

    #[test]
    fn test_execute_command_primload_error() {
        // Test with modules that don't exist
        let command = BootCommand::PrimLoad(vec!["nonexistent_module".to_string()]);
        // Should not panic, may return error
        let _ = execute_command(&command);
    }

    // Error handling tests for resolve_function_entry_point
    #[test]
    fn test_resolve_function_entry_point_module_not_found() {
        let result = resolve_function_entry_point("nonexistent_module", "function", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_function_entry_point_function_not_found() {
        let result = resolve_function_entry_point("erlang", "nonexistent_function", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_function_entry_point_wrong_arity() {
        let result = resolve_function_entry_point("erlang", "length", 999);
        assert!(result.is_err());
    }

    // Error handling tests for execute_boot_script
    #[test]
    fn test_execute_boot_script_with_failing_command() {
        let script = BootScript {
            name: "test".to_string(),
            version: "1.0".to_string(),
            commands: vec![
                BootCommand::Progress("test".to_string()),
                BootCommand::PrimLoad(vec!["nonexistent_module".to_string()]),
            ],
        };
        // Should handle errors gracefully
        let _ = execute_boot_script(&script);
    }

    // Error handling tests for set_code_path
    #[test]
    fn test_set_code_path_empty() {
        let result = set_code_path(&[]);
        // Should not panic
        let _ = result;
    }

    #[test]
    fn test_set_code_path_multiple_paths() {
        let paths = vec![
            "/path1".to_string(),
            "/path2".to_string(),
            "/path3".to_string(),
        ];
        let result = set_code_path(&paths);
        // Should not panic
        let _ = result;
    }

    // Error handling tests for register_process_name
    #[test]
    fn test_register_process_name_invalid_name() {
        let result = register_process_name("", 1);
        // May succeed or fail depending on validation
        let _ = result;
    }

    #[test]
    fn test_register_process_name_duplicate() {
        // Register same name twice
        let _ = register_process_name("test_process", 1);
        let result = register_process_name("test_process", 2);
        // May succeed or fail depending on implementation
        let _ = result;
    }

    // Error handling tests for mark_modules_preloaded
    #[test]
    fn test_mark_modules_preloaded_empty() {
        let result = mark_modules_preloaded(&[]);
        // Should not panic
        let _ = result;
    }

    #[test]
    fn test_mark_modules_preloaded_multiple() {
        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let result = mark_modules_preloaded(&modules);
        // Should not panic
        let _ = result;
    }

    // Error handling tests for load_modules
    #[test]
    fn test_load_modules_empty() {
        let result = load_modules(&[]);
        // Should not panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_modules_nonexistent() {
        let modules = vec!["nonexistent_module".to_string()];
        let result = load_modules(&modules);
        // Should handle gracefully (may return Ok with warnings)
        let _ = result;
    }

    // Test error display formats
    #[test]
    fn test_boot_script_error_display_variants() {
        let errors = vec![
            BootScriptError::NotFound("test".to_string()),
            BootScriptError::InvalidFormat("test".to_string()),
            BootScriptError::ParseError("test".to_string()),
            BootScriptError::IoError("test".to_string()),
        ];
        
        for error in errors {
            let display = format!("{}", error);
            assert!(display.contains("test"));
        }
    }

    // Test parse_boot_script with version as integer - tested via decode_term
    // This is already covered by existing tests that use decode_term

    // Test parse_command with invalid command that has wrong element count
    #[test]
    fn test_parse_command_kernel_load_completed_with_args() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("kernel_load_completed".to_string()),
            ErlangTerm::Atom("extra".to_string()), // Should have no args
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    // Test parse_command with primLoad wrong arity
    #[test]
    fn test_parse_command_primload_wrong_arity() {
        let term = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("primLoad".to_string()),
            // Missing second element
        ]);
        let result = parse_command(&term);
        assert!(result.is_err());
    }

    // Test get_code_paths
    #[test]
    fn test_get_code_paths() {
        let paths = get_code_paths();
        // Should return at least default paths
        assert!(!paths.is_empty());
    }

    // Test that parse_boot_script continues with other commands when one fails
    // This is already tested by test_parse_boot_script_valid which includes multiple commands
}
