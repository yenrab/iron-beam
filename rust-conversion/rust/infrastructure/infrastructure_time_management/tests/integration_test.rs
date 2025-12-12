//! Integration tests for infrastructure_time_management crate
//!
//! These tests verify that time management functions work correctly
//! and test end-to-end workflows for time operations.

use infrastructure_time_management::TimeSup;

#[test]
fn test_time_sup_now_micros_integration() {
    let time1 = TimeSup::now_micros();
    assert!(time1 > 0);
    
    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    let time2 = TimeSup::now_micros();
    assert!(time2 >= time1);
}

#[test]
fn test_time_sup_now_millis_integration() {
    let time1 = TimeSup::now_millis();
    assert!(time1 > 0);
    
    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    let time2 = TimeSup::now_millis();
    assert!(time2 >= time1);
}

#[test]
fn test_time_sup_consistency_integration() {
    // Test that micros and millis are consistent
    let micros = TimeSup::now_micros();
    let millis = TimeSup::now_millis();
    let micros2 = TimeSup::now_micros();
    
    // Millis should be approximately micros / 1000 (within 1ms tolerance)
    let millis_from_micros = micros / 1000;
    assert!((millis as i64 - millis_from_micros as i64).abs() <= 1);
    
    // All times should be in order
    assert!(micros2 >= micros);
}

#[test]
fn test_time_sup_multiple_calls() {
    let mut times_micros = Vec::new();
    let mut times_millis = Vec::new();
    
    for _ in 0..10 {
        times_micros.push(TimeSup::now_micros());
        times_millis.push(TimeSup::now_millis());
    }
    
    // Verify times are non-decreasing
    for i in 1..times_micros.len() {
        assert!(times_micros[i] >= times_micros[i-1]);
    }
    
    for i in 1..times_millis.len() {
        assert!(times_millis[i] >= times_millis[i-1]);
    }
}

#[test]
fn test_time_sup_realistic_delay() {
    let start_micros = TimeSup::now_micros();
    let start_millis = TimeSup::now_millis();
    
    // Sleep for 10ms
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let end_micros = TimeSup::now_micros();
    let end_millis = TimeSup::now_millis();
    
    // Should have advanced by at least 10ms
    let diff_micros = end_micros - start_micros;
    let diff_millis = end_millis - start_millis;
    
    assert!(diff_micros >= 10000); // At least 10ms in microseconds
    assert!(diff_millis >= 10); // At least 10ms in milliseconds
}

#[test]
fn test_time_sup_concurrent_access() {
    use std::thread;
    
    let mut handles = vec![];
    
    // Spawn multiple threads accessing time
    for _ in 0..5 {
        let handle = thread::spawn(|| {
            let mut times = Vec::new();
            for _ in 0..10 {
                times.push(TimeSup::now_micros());
            }
            times
        });
        handles.push(handle);
    }
    
    // Collect all times
    let mut all_times = Vec::new();
    for handle in handles {
        let times = handle.join().unwrap();
        all_times.extend(times);
    }
    
    // Verify all times are positive
    for time in all_times {
        assert!(time > 0);
    }
}
