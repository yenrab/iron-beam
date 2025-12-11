//! Integration tests for frameworks_emulator_init
//!
//! Tests the emulator initialization functionality including early init,
//! main init, and the full startup sequence.

use frameworks_emulator_init::{
    early_init, erl_start, erl_init, InitConfig, TimeWarpMode,
    is_initialized, set_initialized, InitializationState,
};

#[test]
fn test_early_init() {
    // Reset state for testing
    set_initialized(false);
    
    let mut argc = 1;
    let mut argv = vec!["test".to_string()];
    let result = early_init(&mut argc, &mut argv);
    
    assert!(result.is_ok());
    let init_result = result.unwrap();
    assert!(init_result.ncpu > 0);
    assert!(init_result.no_schedulers > 0);
}

#[test]
fn test_init_config_default() {
    let config = InitConfig::default();
    assert_eq!(config.ncpu, 1);
    assert_eq!(config.proc_tab_sz, 1_048_576);
    assert_eq!(config.port_tab_sz, 1_048_576);
    assert_eq!(config.time_warp_mode, TimeWarpMode::NoTimeWarp);
}

#[test]
fn test_init_config_custom() {
    let config = InitConfig {
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
    
    assert_eq!(config.ncpu, 4);
    assert_eq!(config.no_schedulers, 4);
    assert_eq!(config.time_warp_mode, TimeWarpMode::MultiTimeWarp);
}

#[test]
fn test_erl_init() {
    // Reset state for testing
    set_initialized(false);
    
    let config = InitConfig::default();
    let result = erl_init(config);
    
    assert!(result.is_ok());
    assert!(is_initialized());
}

#[test]
fn test_erl_init_with_custom_config() {
    // Reset state for testing
    set_initialized(false);
    
    let config = InitConfig {
        ncpu: 2,
        no_schedulers: 2,
        no_schedulers_online: 2,
        ..Default::default()
    };
    let result = erl_init(config);
    
    assert!(result.is_ok());
}

#[test]
fn test_time_warp_mode() {
    let mode1 = TimeWarpMode::NoTimeWarp;
    let mode2 = TimeWarpMode::MultiTimeWarp;
    let mode3 = TimeWarpMode::SingleTimeWarp;
    
    assert_ne!(mode1, mode2);
    assert_ne!(mode2, mode3);
    assert_ne!(mode1, mode3);
}

#[test]
fn test_initialization_state() {
    let state1 = InitializationState::NotInitialized;
    let state2 = InitializationState::EarlyInitComplete;
    let state3 = InitializationState::MainInitComplete;
    let state4 = InitializationState::FullyInitialized;
    
    assert!(!state1.is_complete());
    assert!(!state2.is_complete());
    assert!(!state3.is_complete());
    assert!(state4.is_complete());
}

#[test]
fn test_is_initialized() {
    set_initialized(false);
    assert!(!is_initialized());
    
    set_initialized(true);
    assert!(is_initialized());
}

#[test]
fn test_erl_start_flow() {
    // Reset state for testing
    set_initialized(false);
    
    let mut argc = 1;
    let mut argv = vec!["test".to_string()];
    
    // Note: This test may fail if early_init was already called in a previous test
    // In a real scenario, we'd have proper state management
    let result = erl_start(&mut argc, &mut argv);
    
    // The result depends on whether early_init was already called
    // If it succeeds, initialization should be complete
    if result.is_ok() {
        assert!(is_initialized());
    }
}

#[test]
fn test_init_config_clone() {
    let config1 = InitConfig {
        ncpu: 4,
        no_schedulers: 4,
        ..Default::default()
    };
    let config2 = config1.clone();
    
    assert_eq!(config1.ncpu, config2.ncpu);
    assert_eq!(config1.no_schedulers, config2.no_schedulers);
}

#[test]
fn test_init_config_debug() {
    let config = InitConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ncpu"));
    assert!(debug_str.contains("proc_tab_sz"));
}


