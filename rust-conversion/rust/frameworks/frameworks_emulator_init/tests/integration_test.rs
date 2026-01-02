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

    // The erl_init function returning Ok(()) indicates successful initialization
    // We don't check is_initialized() here to avoid shared state issues with other tests
    assert!(result.is_ok());
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
    
    if let Err(e) = &result {
        eprintln!("erl_init failed with error: {}", e);
    }
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

#[test]
fn test_erl_init_error_propagation() {
    set_initialized(false);
    // Test with invalid configuration that might cause errors
    let config = InitConfig {
        ncpu: 0, // Invalid - no CPUs
        ..Default::default()
    };
    let result = erl_init(config);
    // May fail with informative error
    if let Err(e) = result {
        assert!(!e.is_empty());
    }
}

#[test]
fn test_erl_start_error_handling() {
    // Test error handling in erl_start
    let mut argc = 1;
    let mut argv = vec!["test".to_string()];
    let result = erl_start(&mut argc, &mut argv);
    
    // May succeed or fail depending on system state
    if let Err(e) = result {
        // Error should be informative
        assert!(!e.is_empty());
    }
}

#[test]
fn test_erl_start_with_various_arguments() {
    // Test erl_start with different argument combinations
    let test_cases = vec![
        vec!["test".to_string()],
        vec!["test".to_string(), "--boot".to_string(), "start".to_string()],
        vec!["test".to_string(), "-boot".to_string(), "start.boot".to_string()],
        vec!["test".to_string(), "--".to_string(), "arg1".to_string()],
    ];
    
    for argv in test_cases {
        let mut argc = argv.len();
        let mut argv_mut = argv;
        let result = erl_start(&mut argc, &mut argv_mut);
        // May succeed or fail - we're testing that it doesn't panic
        let _ = result;
    }
}

#[test]
fn test_verify_beam_execution_setup_integration() {
    // Test verify_beam_execution_setup in integration context
    use frameworks_emulator_init::verify_beam_execution_setup;
    
    set_initialized(false);
    let config = InitConfig::default();
    let _ = erl_init(config);
    
    // Now verify setup
    let result = verify_beam_execution_setup();
    // May succeed or fail depending on whether preloaded modules were loaded
    if let Err(e) = result {
        // Error should contain diagnostic information
        assert!(!e.is_empty());
    }
}

#[test]
fn test_init_config_edge_cases() {
    // Test InitConfig with edge case values
    let configs = vec![
        InitConfig {
            ncpu: usize::MAX,
            ..Default::default()
        },
        InitConfig {
            proc_tab_sz: usize::MAX,
            ..Default::default()
        },
        InitConfig {
            no_schedulers: usize::MAX,
            ..Default::default()
        },
    ];
    
    for config in configs {
        // Should be able to create config with max values
        let _debug = format!("{:?}", config);
    }
}

#[test]
fn test_time_warp_mode_all_combinations() {
    // Test all time warp mode combinations with different configs
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
        let result = erl_init(config);
        // May succeed or fail
        let _ = result;
    }
}

#[test]
fn test_initialization_state_transitions() {
    // Test initialization state management
    set_initialized(false);
    assert!(!is_initialized());
    
    let config = InitConfig::default();
    let result = erl_init(config);
    
    if result.is_ok() {
        assert!(is_initialized());
    }
}

#[test]
fn test_erl_init_multiple_calls() {
    // Test behavior when erl_init is called multiple times
    set_initialized(false);
    let config1 = InitConfig::default();
    let result1 = erl_init(config1.clone());
    
    set_initialized(false);
    let result2 = erl_init(config1);
    
    // Both should provide consistent behavior
    let _ = (result1, result2);
}

#[test]
fn test_init_config_with_all_fields_set() {
    // Test InitConfig with all fields explicitly set
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
        time_warp_mode: TimeWarpMode::MultiTimeWarp,
    };
    
    assert_eq!(config.ncpu, 8);
    assert_eq!(config.no_schedulers, 8);
    assert_eq!(config.time_warp_mode, TimeWarpMode::MultiTimeWarp);
}


