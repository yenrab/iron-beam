//! Early Initialization Module
//!
//! Provides early initialization phase functions.
//! Based on `early_init()` from erl_init.c

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

/// Early initialization result
#[derive(Debug, Clone)]
pub struct EarlyInitResult {
    /// Number of CPUs detected
    pub ncpu: usize,
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
}

/// Global flag to track if early init is complete
static EARLY_INIT_DONE: AtomicBool = AtomicBool::new(false);

/// Perform early initialization
///
/// Based on `early_init()` from erl_init.c. This function performs
/// initialization tasks that must be done before the main initialization
/// phase, including:
/// - Parsing command line arguments
/// - Initializing memory allocators
/// - Setting up thread progress
/// - Detecting CPU topology
///
/// # Arguments
/// * `argc` - Number of command line arguments (mutable, may be modified)
/// * `argv` - Command line arguments (mutable, may be modified)
///
/// # Returns
/// * `Ok(EarlyInitResult)` - Early initialization result with system configuration
/// * `Err(String)` - Initialization error
pub fn early_init(argc: &mut usize, argv: &mut Vec<String>) -> Result<EarlyInitResult, String> {
    // Check if already initialized
    if EARLY_INIT_DONE.load(Ordering::Acquire) {
        return Err("Early initialization already completed".to_string());
    }
    
    // Save emulator arguments (for later retrieval)
    // In C: erts_save_emu_args(*argc, argv);
    
    // Initialize term system
    // In C: erts_term_init();
    
    // Detect CPU topology
    let ncpu = detect_cpu_count();
    let no_schedulers = calculate_schedulers(ncpu);
    let no_schedulers_online = no_schedulers; // For now, all schedulers online
    let no_poll_threads = 1; // Default
    let no_dirty_cpu_schedulers = 0; // Default
    let no_dirty_cpu_schedulers_online = 0; // Default
    let no_dirty_io_schedulers = 0; // Default
    
    // Initialize runtime utilities
    infrastructure_runtime_utils::erts_init_utils()
        .map_err(|e| format!("Failed to initialize runtime utils: {}", e))?;
    
    // Initialize memory utilities
    infrastructure_runtime_utils::erts_init_utils_mem()
        .map_err(|e| format!("Failed to initialize memory utils: {}", e))?;
    
    // Mark early init as complete
    EARLY_INIT_DONE.store(true, Ordering::Release);
    
    Ok(EarlyInitResult {
        ncpu,
        no_schedulers,
        no_schedulers_online,
        no_poll_threads,
        no_dirty_cpu_schedulers,
        no_dirty_cpu_schedulers_online,
        no_dirty_io_schedulers,
    })
}

/// Detect CPU count
fn detect_cpu_count() -> usize {
    // Use num_cpus crate or std::thread::available_parallelism
    // For now, use a simple approach
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Calculate number of schedulers based on CPU count
fn calculate_schedulers(ncpu: usize) -> usize {
    // Default: one scheduler per CPU
    ncpu.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_early_init() {
        // Reset state for testing
        EARLY_INIT_DONE.store(false, Ordering::Release);
        
        let mut argc = 1;
        let mut argv = vec!["test".to_string()];
        let result = early_init(&mut argc, &mut argv);
        
        assert!(result.is_ok());
        let init_result = result.unwrap();
        assert!(init_result.ncpu > 0);
        assert!(init_result.no_schedulers > 0);
    }
    
    #[test]
    fn test_early_init_idempotent() {
        // Reset state for testing
        EARLY_INIT_DONE.store(false, Ordering::Release);
        
        let mut argc1 = 1;
        let mut argv1 = vec!["test".to_string()];
        let _result1 = early_init(&mut argc1, &mut argv1).unwrap();
        
        // Second call should fail
        let mut argc2 = 1;
        let mut argv2 = vec!["test".to_string()];
        let result2 = early_init(&mut argc2, &mut argv2);
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_detect_cpu_count() {
        let ncpu = detect_cpu_count();
        assert!(ncpu > 0);
    }
    
    #[test]
    fn test_calculate_schedulers() {
        assert_eq!(calculate_schedulers(1), 1);
        assert_eq!(calculate_schedulers(4), 4);
        assert_eq!(calculate_schedulers(0), 1); // Minimum 1
    }
}


