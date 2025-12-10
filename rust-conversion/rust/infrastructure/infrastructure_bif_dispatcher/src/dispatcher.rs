//! BIF Call Dispatcher
//!
//! Provides the core BIF call dispatcher functions that route BIF calls
//! from the emulator to BIF implementations. Based on call_bif() and
//! erts_call_dirty_bif() from bif.c

use std::sync::Arc;
use entities_process::{Process, ErtsCodePtr, Eterm};
use crate::initialization::BifFunction;

/// BIF dispatcher
///
/// Manages BIF call dispatching and routing. The dispatcher routes calls
/// from the emulator to appropriate BIF implementations.
pub struct BifDispatcher {
    /// Whether dispatcher is initialized
    initialized: bool,
}

impl BifDispatcher {
    /// Create a new BIF dispatcher
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize the dispatcher
    pub fn init(&mut self) -> Result<(), BifDispatcherError> {
        if self.initialized {
            return Err(BifDispatcherError::AlreadyInitialized);
        }
        self.initialized = true;
        Ok(())
    }

    /// Check if dispatcher is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for BifDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Call a BIF function
///
/// Based on call_bif() from bif.c
///
/// This is the main dispatcher function that routes BIF calls. It extracts
/// the BIF function pointer from the native function structure and calls it.
///
/// # Arguments
/// * `process` - Process calling the BIF
/// * `reg` - Register array (BIF arguments)
/// * `instruction_ptr` - Instruction pointer (points to native function structure)
///
/// # Returns
/// * `Ok(result)` - BIF result term
/// * `Err(BifDispatcherError)` - Dispatch error
///
/// # Note
/// In the C implementation, this:
/// 1. Extracts ErtsNativeFunc from instruction pointer
/// 2. Gets BIF function pointer
/// 3. Sets trap marker
/// 4. Calls BIF function
/// 5. Handles result (value, trap, or error)
/// 6. Restores native function if needed
pub fn call_bif(
    process: &Process,
    reg: &[Eterm],
    instruction_ptr: ErtsCodePtr,
) -> Result<Eterm, BifDispatcherError> {
    // In the C implementation:
    // ErtsNativeFunc *nep = ERTS_I_BEAM_OP_TO_NFUNC(I);
    // ErtsBifFunc bif = (ErtsBifFunc) nep->func;
    // 
    // nep->func = ERTS_SCHED_BIF_TRAP_MARKER;
    // ret = (*bif)(c_p, reg, I);
    // 
    // if (is_value(ret))
    //     erts_nfunc_restore(c_p, nep, ret);
    // else if (c_p->freason != TRAP)
    //     c_p->freason |= EXF_RESTORE_NFUNC;
    // else if (nep->func == ERTS_SCHED_BIF_TRAP_MARKER) {
    //     erts_nfunc_restore(c_p, nep, ret);
    // }

    // For Rust, we need to:
    // 1. Extract BIF function from instruction pointer (would need native function structure)
    // 2. Call the BIF function
    // 3. Handle result

    // This is a simplified version - full implementation would need:
    // - Native function structure access
    // - BIF function table lookup
    // - Proper error handling and trap restoration

    // For now, return an error indicating this needs full implementation
    Err(BifDispatcherError::NotImplemented("call_bif requires native function structure access".to_string()))
}

/// Call a dirty BIF function
///
/// Based on erts_call_dirty_bif() from bif.c
///
/// Calls a BIF that runs on a dirty scheduler (dirty CPU or dirty I/O scheduler).
/// Dirty BIFs are BIFs that perform blocking operations and need to run on
/// dedicated schedulers to avoid blocking normal schedulers.
///
/// # Arguments
/// * `process` - Process calling the dirty BIF
/// * `instruction_ptr` - Instruction pointer
/// * `reg` - Register array (BIF arguments)
///
/// # Returns
/// * `Ok(result)` - BIF result term
/// * `Err(BifDispatcherError)` - Dispatch error
///
/// # Note
/// In the C implementation, this:
/// 1. Checks if process is exiting
/// 2. Gets dirty shadow process if needed
/// 3. Calls the dirty BIF function
/// 4. Handles result and process state
/// 5. Returns result or error indicator
pub fn erts_call_dirty_bif(
    process: &Process,
    instruction_ptr: ErtsCodePtr,
    reg: &[Eterm],
) -> Result<Eterm, BifDispatcherError> {
    // In the C implementation:
    // BIF_RETTYPE result;
    // int exiting;
    // Process *dirty_shadow_proc;
    // 
    // // Check if process is exiting
    // // Get dirty shadow process
    // // Call dirty BIF function
    // // Handle result

    // This is a simplified version - full implementation would need:
    // - Dirty scheduler integration
    // - Dirty shadow process management
    // - Proper error handling

    // For now, return an error indicating this needs full implementation
    Err(BifDispatcherError::NotImplemented("erts_call_dirty_bif requires dirty scheduler integration".to_string()))
}

/// BIF dispatcher errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BifDispatcherError {
    /// Dispatcher not initialized
    NotInitialized,
    /// Already initialized
    AlreadyInitialized,
    /// BIF function not found
    BifNotFound(String),
    /// Invalid arguments
    InvalidArguments(String),
    /// Process error
    ProcessError(String),
    /// Not implemented (for stubbed functions)
    NotImplemented(String),
}

impl std::fmt::Display for BifDispatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BifDispatcherError::NotInitialized => write!(f, "BIF dispatcher not initialized"),
            BifDispatcherError::AlreadyInitialized => write!(f, "BIF dispatcher already initialized"),
            BifDispatcherError::BifNotFound(name) => write!(f, "BIF not found: {}", name),
            BifDispatcherError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            BifDispatcherError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            BifDispatcherError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
        }
    }
}

impl std::error::Error for BifDispatcherError {}

#[cfg(test)]
mod tests {
    use super::*;
    use entities_process::ProcessId;

    #[test]
    fn test_bif_dispatcher_creation() {
        let dispatcher = BifDispatcher::new();
        assert!(!dispatcher.is_initialized());
    }

    #[test]
    fn test_bif_dispatcher_init() {
        let mut dispatcher = BifDispatcher::new();
        assert!(!dispatcher.is_initialized());
        
        let result = dispatcher.init();
        assert!(result.is_ok());
        assert!(dispatcher.is_initialized());
    }

    #[test]
    fn test_bif_dispatcher_double_init() {
        let mut dispatcher = BifDispatcher::new();
        dispatcher.init().unwrap();
        
        let result = dispatcher.init();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BifDispatcherError::AlreadyInitialized));
    }

    #[test]
    fn test_call_bif_not_implemented() {
        let process = Process::new(1);
        let reg = vec![100, 200];
        let instruction_ptr = std::ptr::null();
        
        let result = call_bif(&process, &reg, instruction_ptr);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BifDispatcherError::NotImplemented(_)));
    }

    #[test]
    fn test_erts_call_dirty_bif_not_implemented() {
        let process = Process::new(1);
        let reg = vec![100, 200];
        let instruction_ptr = std::ptr::null();
        
        let result = erts_call_dirty_bif(&process, instruction_ptr, &reg);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BifDispatcherError::NotImplemented(_)));
    }

    #[test]
    fn test_bif_dispatcher_error_display() {
        let error1 = BifDispatcherError::NotInitialized;
        let error2 = BifDispatcherError::BifNotFound("test_bif".to_string());
        let error3 = BifDispatcherError::NotImplemented("test".to_string());
        
        let str1 = format!("{}", error1);
        let str2 = format!("{}", error2);
        let str3 = format!("{}", error3);
        
        assert!(str1.contains("not initialized"));
        assert!(str2.contains("BIF not found"));
        assert!(str2.contains("test_bif"));
        assert!(str3.contains("Not implemented"));
        assert!(str3.contains("test"));
    }
}

