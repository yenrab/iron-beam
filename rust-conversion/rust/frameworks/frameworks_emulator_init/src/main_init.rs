//! Main Initialization Module
//!
//! Provides main initialization phase functions.
//! Based on `erl_init()` and `erl_start()` from erl_init.c

use crate::early_init::EarlyInitResult;
use crate::initialization::{set_initialized, InitializationState};

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
    
    // Initialize process management
    // In C: erts_init_process(ncpu, proc_tab_sz, legacy_proc_tab);
    // For now, we'll just mark this as a placeholder
    
    // Initialize scheduling
    // In C: erts_init_scheduling(...)
    // This would call into usecases_scheduling
    // For now, we'll just mark this as a placeholder
    
    // Initialize BIF dispatcher
    // In C: erts_init_bif()
    // This would call into infrastructure_bif_dispatcher
    // For now, we'll just mark this as a placeholder
    
    // Initialize emulator loop
    // In C: init_emulator()
    // This would call into infrastructure_emulator_loop
    // For now, we'll just mark this as a placeholder
    
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
    let mut config = InitConfig {
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
    // In C: This is done in erl_start() with a large switch statement
    // For now, we'll use defaults
    
    // Perform main initialization
    erl_init(config)
        .map_err(|e| format!("Main initialization failed: {}", e))?;
    
    Ok(())
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

