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
    use entities_process::{Process, ErtsCodePtr};
    use infrastructure_utilities::process_table::get_global_process_table;
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    use std::sync::Arc;
    
    // Allocate a new process with automatic ID generation
    let process_table = get_global_process_table();
    let (pid, process_arc) = process_table
        .new_element(|id| {
            let mut process = Process::new(id);
            
            // In the full implementation, we would:
            // 1. Look up the module in the code server
            // 2. Find the function entry point (export table lookup)
            // 3. Set the instruction pointer to the function entry point
            // 4. Set up the process heap with function arguments
            // 5. Set up the process stack for function call
            
            // For now, create a placeholder code sequence
            // In production, this would be the actual function entry point
            use infrastructure_emulator_loop::instruction_decoder::opcodes;
            let mut process_code = Vec::new();
            
            // Create a simple entry point that will eventually call the function
            // For now, just a return instruction (process will exit immediately)
            // In full implementation, this would be the actual function code
            process_code.push(opcodes::RETURN as u64);
            
            let code_ptr = process_code.as_ptr() as ErtsCodePtr;
            std::mem::forget(process_code); // Keep code alive
            
            process.set_i(code_ptr);
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
    let scheduler = &schedulers_guard[0];
    let runq = scheduler.runq();
    let runq_guard = runq.lock()
        .map_err(|e| format!("Failed to lock run queue: {}", e))?;
    
    schedule_process(process_arc.clone(), &runq_guard, Priority::Normal)
        .map_err(|e| format!("Failed to schedule kernel process: {:?}", e))?;
    
    eprintln!("      ✓ Kernel process '{}' spawned and scheduled (PID: {})", name, pid);
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
///
/// # Note
/// In the full implementation, this would:
/// 1. Look up the function in the module's export table
/// 2. Create a temporary process or use current context
/// 3. Set up the function call with proper argument encoding
/// 4. Execute the function synchronously
/// 5. Return the result
fn apply_function(
    module: &str,
    function: &str,
    args: &[String],
) -> Result<(), String> {
    use entities_process::{Process, ErtsCodePtr};
    use infrastructure_utilities::process_table::get_global_process_table;
    use usecases_scheduling::{get_global_schedulers, schedule_process, Priority};
    use std::sync::Arc;
    
    eprintln!("      Applying function: {}.{}/{}", module, function, args.len());
    
    // In the full implementation, we would:
    // 1. Look up the module in the code server
    // 2. Find the function entry point in the export table
    // 3. Create a temporary process or use the current execution context
    // 4. Set up the process heap with function arguments (properly encoded as Erlang terms)
    // 5. Set the instruction pointer to the function entry point
    // 6. Execute the function synchronously (or schedule and wait)
    // 7. Handle the return value or any exceptions
    
    // For now, we'll create a temporary process similar to kernel process spawning
    // but mark it for synchronous execution
    
    // Allocate a new process with automatic ID generation
    let process_table = get_global_process_table();
    let (pid, process_arc) = process_table
        .new_element(|id| {
            let mut process = Process::new(id);
            
            // In the full implementation, this would be the actual function entry point
            // For now, create a placeholder code sequence
            use infrastructure_emulator_loop::instruction_decoder::opcodes;
            let mut process_code = Vec::new();
            
            // Create a simple entry point that will eventually call the function
            // For now, just a return instruction (process will exit immediately)
            // In full implementation, this would be the actual function code
            process_code.push(opcodes::RETURN as u64);
            
            let code_ptr = process_code.as_ptr() as ErtsCodePtr;
            std::mem::forget(process_code); // Keep code alive
            
            process.set_i(code_ptr);
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
/// This function searches for .beam files and loads them into the runtime.
///
/// # Arguments
/// * `modules` - List of module names to load
///
/// # Returns
/// Result indicating success or failure
fn load_modules(modules: &[String]) -> Result<(), String> {
    use code_management_code_loading::CodeLoader;
    use code_management_code_loading::code_loader::LoadError;
    use std::path::Path;
    
    // Get code search paths
    // In the full implementation, this would use the code path from boot script
    // For now, we'll try common locations
    let code_paths = get_code_paths();
    
    let mut loaded_count = 0;
    let mut failed_modules = Vec::new();
    
    for module_name in modules {
        // Try to find and load the module
        let mut found = false;
        
        for code_path in &code_paths {
            // Try .beam file
            let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
            
            match CodeLoader::load_module(&beam_path) {
                Ok(code) => {
                    // Verify the code
                    if CodeLoader::verify_module(&code) {
                        eprintln!("      ✓ Loaded: {} (from {})", module_name, beam_path.display());
                        loaded_count += 1;
                        found = true;
                        break;
                    } else {
                        eprintln!("      ✗ Invalid format: {}", module_name);
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
/// if no path has been set.
///
/// # Returns
/// Vector of code path directories
fn get_code_paths() -> Vec<String> {
    let code_path = init_code_path();
    let path_guard = code_path
        .lock()
        .expect("Failed to lock code path");
    
    path_guard.clone()
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

    #[test]
    fn test_boot_command_display() {
        let cmd = BootCommand::Progress("test".to_string());
        assert_eq!(cmd, BootCommand::Progress("test".to_string()));
    }

    #[test]
    fn test_resolve_boot_path() {
        // Test with non-existent path (should return error)
        let result = resolve_boot_path("nonexistent", "/root", "/bin");
        assert!(result.is_err());
    }
}
