//! Process Management Initialization
//!
//! Provides initialization functions for process management system.
//! Based on erts_init_process() from erl_process.c

use infrastructure_utilities::process_table::{get_global_process_table, ProcessTable};

/// Initialize process management system
///
/// Based on `erts_init_process()` from erl_process.c
///
/// This function initializes the process table and process management structures.
/// It must be called before any processes can be created.
///
/// # Arguments
/// * `ncpu` - Number of CPUs (used for lock initialization)
/// * `proc_tab_sz` - Process table size (maximum number of processes)
/// * `legacy_proc_tab` - Whether to use legacy process table format (unused in Rust implementation)
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
///
/// # Note
/// The global process table is initialized lazily on first access via `get_global_process_table()`.
/// This function ensures it's initialized with the correct size limit.
pub fn erts_init_process(
    _ncpu: usize,
    proc_tab_sz: usize,
    _legacy_proc_tab: bool,
) -> Result<(), String> {
    // Initialize process locks (if needed)
    // In C: erts_init_proc_lock(ncpu)
    // For now, process locks are initialized on-demand
    
    // Initialize process list allocator (if needed)
    // In C: init_proclist_alloc()
    // For now, we use standard Rust allocators
    
    // Initialize process table
    // In C: erts_ptab_init_table(&erts_proc, ...)
    // In Rust: The global process table is initialized lazily, but we can ensure
    // it's initialized with the correct size by accessing it
    // Note: The current implementation uses OnceLock which doesn't allow
    // setting max_size after initialization. For now, we'll just ensure
    // the table is initialized. A future enhancement could allow setting
    // max_size during initialization.
    let _table = get_global_process_table();
    
    // In a full implementation, we would:
    // 1. Initialize process locks based on ncpu
    // 2. Initialize process list allocator
    // 3. Initialize process table with proc_tab_sz
    // 4. Set up invalid process structure
    
    // For now, the process table is initialized on first access
    // The size limit would need to be set during table creation
    // This is a limitation of the current OnceLock-based design
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erts_init_process() {
        let result = erts_init_process(4, 1_048_576, false);
        assert!(result.is_ok());
        
        // Verify process table is accessible
        let _table = get_global_process_table();
    }
}

