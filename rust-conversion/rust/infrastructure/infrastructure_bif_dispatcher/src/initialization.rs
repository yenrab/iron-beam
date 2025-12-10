//! BIF Dispatcher Initialization
//!
//! Provides initialization functions for the BIF dispatcher system, including
//! trap export setup. Based on erts_init_bif() and erts_init_trap_export()
//! from bif.c

use std::sync::{Arc, Mutex};
use entities_process::Eterm;

/// Trap export structure
///
/// Represents a trap export entry that routes BIF calls. Based on Export
/// structure from export.h, but simplified for Rust implementation.
#[derive(Clone)]
pub struct TrapExport {
    /// Module name (atom)
    module: Eterm,
    /// Function name (atom)
    function: Eterm,
    /// Arity
    arity: u32,
    /// BIF function pointer (as a trait object for type safety)
    bif_func: Option<Arc<dyn BifFunction + Send + Sync>>,
    /// BIF number (-1 if not a BIF)
    bif_number: i32,
}

/// Trait for BIF functions
///
/// BIF functions take a process, arguments array, and instruction pointer,
/// and return an Eterm result.
pub trait BifFunction {
    /// Call the BIF function
    ///
    /// # Arguments
    /// * `process` - Process calling the BIF
    /// * `args` - BIF arguments (up to 4 arguments)
    /// * `instruction_ptr` - Instruction pointer
    ///
    /// # Returns
    /// Result term or error indicator
    fn call(&self, process: &entities_process::Process, args: &[Eterm], instruction_ptr: entities_process::ErtsCodePtr) -> Eterm;
}

impl TrapExport {
    /// Create a new trap export
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    /// * `function` - Function name (atom)
    /// * `arity` - Function arity
    /// * `bif_func` - Optional BIF function implementation
    pub fn new(module: Eterm, function: Eterm, arity: u32, bif_func: Option<Arc<dyn BifFunction + Send + Sync>>) -> Self {
        Self {
            module,
            function,
            arity,
            bif_func,
            bif_number: -1, // -1 means not a BIF
        }
    }

    /// Get module name
    pub fn module(&self) -> Eterm {
        self.module
    }

    /// Get function name
    pub fn function(&self) -> Eterm {
        self.function
    }

    /// Get arity
    pub fn arity(&self) -> u32 {
        self.arity
    }

    /// Get BIF number
    pub fn bif_number(&self) -> i32 {
        self.bif_number
    }

    /// Set BIF number
    pub fn set_bif_number(&mut self, bif_number: i32) {
        self.bif_number = bif_number;
    }

    /// Get BIF function
    pub fn bif_func(&self) -> Option<&Arc<dyn BifFunction + Send + Sync>> {
        self.bif_func.as_ref()
    }
}

/// Initialize a trap export
///
/// Based on erts_init_trap_export() from bif.c
///
/// Sets up a trap export entry that routes BIF calls. The export entry
/// is initialized with module, function, arity, and BIF function pointer.
///
/// # Arguments
/// * `ep` - Trap export to initialize
/// * `module` - Module name (atom)
/// * `function` - Function name (atom)
/// * `arity` - Function arity
/// * `bif_func` - Optional BIF function implementation
///
/// # Note
/// In the C implementation, this also sets up trampolines for each code index
/// and configures the dispatch addresses. In Rust, we use a simpler approach
/// with trait objects for BIF functions.
pub fn erts_init_trap_export(
    ep: &mut TrapExport,
    module: Eterm,
    function: Eterm,
    arity: u32,
    bif_func: Option<Arc<dyn BifFunction + Send + Sync>>,
) {
    // Initialize trap export
    *ep = TrapExport::new(module, function, arity, bif_func);
    
    // In the C implementation, this would:
    // 1. Zero out the Export structure
    // 2. Activate export trampolines for each code index
    // 3. Set up dispatch addresses
    // 4. Set bif_number to -1 (not a BIF)
    // 5. Set module, function, arity
    // 6. Set trampoline opcode and BIF address
    
    // For Rust, we've already set these in TrapExport::new()
}

/// Global trap exports (initialized by erts_init_bif)
static BIF_RETURN_TRAP_EXPORT: Mutex<Option<TrapExport>> = Mutex::new(None);
static BIF_HANDLE_SIGNALS_RETURN_EXPORT: Mutex<Option<TrapExport>> = Mutex::new(None);
static AWAIT_EXIT_TRAP_EXPORT: Mutex<Option<TrapExport>> = Mutex::new(None);

/// Initialize BIF dispatcher system
///
/// Based on erts_init_bif() from bif.c
///
/// Initializes the BIF dispatcher system, setting up trap exports for:
/// - bif_return_trap/2 - BIF return trap handler
/// - bif_handle_signals_return/2 - Signal return handler
/// - await_exit_trap/0 - Await exit trap handler
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(BifInitError)` - Initialization error
///
/// # Note
/// In the C implementation, this also sets up various other trap exports
/// and initializes atomic counters for scheduler wall time and microstate
/// accounting. This is a simplified version focusing on the core dispatcher
/// functionality.
pub fn erts_init_bif() -> Result<(), BifInitError> {
    // Initialize bif_return_trap export
    // In C: erts_init_trap_export(&bif_return_trap_export, am_erlang, am_bif_return_trap, 2, &bif_return_trap);
    {
        let mut export = BIF_RETURN_TRAP_EXPORT.lock().unwrap();
        *export = Some(TrapExport::new(
            0, // am_erlang (would be actual atom in full implementation)
            0, // am_bif_return_trap (would be actual atom in full implementation)
            2,
            None, // bif_return_trap function would be set here
        ));
    }

    // Initialize bif_handle_signals_return export
    // In C: erts_init_trap_export(&erts_bif_handle_signals_return_export, am_erlang, am_bif_handle_signals_return, 2, &bif_handle_signals_return);
    {
        let mut export = BIF_HANDLE_SIGNALS_RETURN_EXPORT.lock().unwrap();
        *export = Some(TrapExport::new(
            0, // am_erlang
            0, // am_bif_handle_signals_return
            2,
            None, // bif_handle_signals_return function would be set here
        ));
    }

    // Initialize await_exit_trap export
    // In C: erts_init_trap_export(&await_exit_trap, am_erts_internal, am_await_exit, 0, erts_internal_await_exit_trap);
    {
        let mut export = AWAIT_EXIT_TRAP_EXPORT.lock().unwrap();
        *export = Some(TrapExport::new(
            0, // am_erts_internal
            0, // am_await_exit
            0,
            None, // erts_internal_await_exit_trap function would be set here
        ));
    }

    // In the C implementation, this would also:
    // - Set up dsend_continue_trap_export
    // - Set up various other trap exports
    // - Initialize atomic counters for scheduler wall time
    // - Initialize atomic counters for microstate accounting

    Ok(())
}

/// Get bif_return_trap export
pub fn get_bif_return_trap_export() -> Option<TrapExport> {
    BIF_RETURN_TRAP_EXPORT.lock().unwrap().as_ref().cloned()
}

/// Get bif_handle_signals_return export
pub fn get_bif_handle_signals_return_export() -> Option<TrapExport> {
    BIF_HANDLE_SIGNALS_RETURN_EXPORT.lock().unwrap().as_ref().cloned()
}

/// Get await_exit_trap export
pub fn get_await_exit_trap_export() -> Option<TrapExport> {
    AWAIT_EXIT_TRAP_EXPORT.lock().unwrap().as_ref().cloned()
}

/// BIF initialization errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BifInitError {
    /// Initialization failed
    InitFailed(String),
    /// Already initialized
    AlreadyInitialized,
}

impl std::fmt::Display for BifInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BifInitError::InitFailed(msg) => write!(f, "BIF dispatcher initialization failed: {}", msg),
            BifInitError::AlreadyInitialized => write!(f, "BIF dispatcher already initialized"),
        }
    }
}

impl std::error::Error for BifInitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trap_export_creation() {
        let export = TrapExport::new(1, 2, 3, None);
        assert_eq!(export.module(), 1);
        assert_eq!(export.function(), 2);
        assert_eq!(export.arity(), 3);
        assert_eq!(export.bif_number(), -1);
    }

    #[test]
    fn test_erts_init_trap_export() {
        let mut export = TrapExport::new(0, 0, 0, None);
        erts_init_trap_export(&mut export, 10, 20, 30, None);
        
        assert_eq!(export.module(), 10);
        assert_eq!(export.function(), 20);
        assert_eq!(export.arity(), 30);
    }

    #[test]
    fn test_erts_init_bif() {
        let result = erts_init_bif();
        assert!(result.is_ok());
        
        // Verify trap exports were created
        assert!(get_bif_return_trap_export().is_some());
        assert!(get_bif_handle_signals_return_export().is_some());
        assert!(get_await_exit_trap_export().is_some());
    }
}

