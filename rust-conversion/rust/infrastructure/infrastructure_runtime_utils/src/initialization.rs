//! Initialization Module
//!
//! Provides runtime initialization functions.
//! Based on erts_init_utils() and related functions from utils.c

use std::sync::atomic::{AtomicBool, Ordering};

/// Initialization state
static INIT_DONE: AtomicBool = AtomicBool::new(false);
static MEM_INIT_DONE: AtomicBool = AtomicBool::new(false);

/// Initialize runtime utilities
///
/// Based on `erts_init_utils()` from utils.c. This function performs
/// one-time initialization of runtime utilities.
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
pub fn erts_init_utils() -> Result<(), String> {
    // Check if already initialized
    if INIT_DONE.load(Ordering::Acquire) {
        return Ok(());
    }
    
    // Perform initialization tasks
    // In the C code, this sets up thread-specific data keys for debugging
    // For now, we just mark initialization as done
    
    INIT_DONE.store(true, Ordering::Release);
    
    Ok(())
}

/// Initialize runtime utilities memory
///
/// Based on `erts_init_utils_mem()` from utils.c. This function initializes
/// memory-related utilities.
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
pub fn erts_init_utils_mem() -> Result<(), String> {
    // Check if already initialized
    if MEM_INIT_DONE.load(Ordering::Acquire) {
        return Ok(());
    }
    
    // Perform memory initialization tasks
    // In the C code, this initializes memory allocation thresholds:
    // - trim_threshold = -1
    // - top_pad = -1
    // - mmap_threshold = -1
    // - mmap_max = -1
    // For now, we just mark initialization as done
    
    MEM_INIT_DONE.store(true, Ordering::Release);
    
    Ok(())
}

/// Initialize scheduler-specific data for utilities
///
/// Based on `erts_utils_sched_spec_data_init()` from utils.c. This function
/// initializes scheduler-specific data for utilities.
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
pub fn erts_utils_sched_spec_data_init() -> Result<(), String> {
    // In the C code, this sets thread-specific data for debugging
    // For now, we just ensure main initialization is done
    erts_init_utils()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_erts_init_utils() {
        // Reset state for testing
        INIT_DONE.store(false, Ordering::Release);
        
        let result = erts_init_utils();
        assert!(result.is_ok());
        assert!(INIT_DONE.load(Ordering::Acquire));
        
        // Second call should also succeed (idempotent)
        let result2 = erts_init_utils();
        assert!(result2.is_ok());
    }
    
    #[test]
    fn test_erts_init_utils_mem() {
        // Reset state for testing
        MEM_INIT_DONE.store(false, Ordering::Release);
        
        let result = erts_init_utils_mem();
        assert!(result.is_ok());
        assert!(MEM_INIT_DONE.load(Ordering::Acquire));
        
        // Second call should also succeed (idempotent)
        let result2 = erts_init_utils_mem();
        assert!(result2.is_ok());
    }
    
    #[test]
    fn test_erts_utils_sched_spec_data_init() {
        let result = erts_utils_sched_spec_data_init();
        assert!(result.is_ok());
    }
}


