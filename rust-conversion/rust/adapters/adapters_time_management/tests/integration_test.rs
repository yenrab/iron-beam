//! Integration tests for adapters_time_management crate
//!
//! These tests verify that time management adapters work correctly.

use adapters_time_management::*;
use std::time::Duration;

#[test]
fn test_timer_new() {
    let timer = Timer::new(Duration::from_millis(100));
    assert!(!timer.expired());
    assert!(timer.remaining() > Duration::ZERO);
}

#[test]
fn test_timer_expired() {
    let timer = Timer::new(Duration::from_millis(1));
    
    // Wait for timer to expire
    std::thread::sleep(Duration::from_millis(10));
    
    assert!(timer.expired());
    assert_eq!(timer.remaining(), Duration::ZERO);
}

#[test]
fn test_timer_remaining() {
    let timer = Timer::new(Duration::from_millis(100));
    
    let remaining = timer.remaining();
    assert!(remaining <= Duration::from_millis(100));
    assert!(remaining > Duration::ZERO);
}

#[test]
fn test_timer_not_expired() {
    let timer = Timer::new(Duration::from_secs(1));
    assert!(!timer.expired());
}

#[test]
fn test_timer_multiple_checks() {
    let timer = Timer::new(Duration::from_millis(50));
    
    // Check multiple times
    assert!(!timer.expired());
    let remaining1 = timer.remaining();
    
    std::thread::sleep(Duration::from_millis(10));
    
    assert!(!timer.expired());
    let remaining2 = timer.remaining();
    
    // Remaining time should decrease
    assert!(remaining2 < remaining1);
}

#[test]
fn test_timer_zero_duration() {
    let timer = Timer::new(Duration::ZERO);
    assert!(timer.expired());
    assert_eq!(timer.remaining(), Duration::ZERO);
}

#[test]
fn test_timer_large_duration() {
    let timer = Timer::new(Duration::from_secs(3600));
    assert!(!timer.expired());
    assert!(timer.remaining() > Duration::ZERO);
}

#[test]
fn test_timeslice_operations() {
    // Test timeslice operations if available
    // Note: Check if timeslice module has public API
}
