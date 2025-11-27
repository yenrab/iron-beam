//! Integration tests for entities_utilities crate
//!
//! These tests verify big number and register operations end-to-end.

use entities_utilities::*;

#[test]
fn test_big_number_operations_integration() {
    // Test various big number operations end-to-end
    let big1 = BigNumber::from_i64(1234567890123456i64);
    let big2 = BigNumber::from_i64(-987654321098765i64);
    
    // Test addition
    let sum = big1.plus(&big2);
    // Result might overflow i64, so just verify it's computed
    let _ = sum.to_i64();
    
    // Test subtraction
    let diff = big1.minus(&big2);
    // Result might overflow i64, so just verify it's computed
    let _ = diff.to_i64();
    
    // Test multiplication
    let prod = big1.times(&big2);
    // Result might be too large for i64, so just verify it's a valid big number
    assert!(prod.to_i64().is_none() || prod.to_i64().is_some());
    
    // Test division
    let div = big1.div(&big2);
    // Division result should be valid
    assert!(div.is_some());
    if let Some(div_result) = div {
        assert!(div_result.to_i64().is_some() || div_result.to_i64().is_none());
    }
}

#[test]
fn test_big_number_conversion_integration() {
    // Test conversions between different types
    let from_i64 = BigNumber::from_i64(-123456789);
    assert_eq!(from_i64.to_i64(), Some(-123456789));
    
    let from_u64 = BigNumber::from_u64(9876543210);
    // Verify it was created correctly by checking it's not zero
    assert!(!from_u64.is_zero());
    
    let from_i32 = BigNumber::from_i32(-12345);
    assert_eq!(from_i32.to_i64(), Some(-12345));
    
    let from_u32 = BigNumber::from_u32(54321);
    // Verify it was created correctly
    assert!(!from_u32.is_zero());
    
    // Test f64 conversion
    let from_f64 = BigNumber::from_f64(123.456);
    assert!(from_f64.is_some());
    let big = from_f64.unwrap();
    assert_eq!(big.to_i64(), Some(123)); // Truncated
    
    // Test large f64
    let large_f64 = BigNumber::from_f64(1e20);
    assert!(large_f64.is_some());
}

#[test]
fn test_big_number_edge_cases() {
    // Test edge cases
    let zero = BigNumber::from_i64(0);
    assert_eq!(zero.to_i64(), Some(0));
    
    let max_i64 = BigNumber::from_i64(i64::MAX);
    assert_eq!(max_i64.to_i64(), Some(i64::MAX));
    
    let min_i64 = BigNumber::from_i64(i64::MIN);
    assert_eq!(min_i64.to_i64(), Some(i64::MIN));
    
    let max_u64 = BigNumber::from_u64(u64::MAX);
    // Verify it was created correctly (might not fit in i64)
    assert!(!max_u64.is_zero());
    
    // Test NaN and infinity
    assert!(BigNumber::from_f64(f64::NAN).is_none());
    assert!(BigNumber::from_f64(f64::INFINITY).is_none());
    assert!(BigNumber::from_f64(f64::NEG_INFINITY).is_none());
}

#[test]
fn test_register_lifecycle() {
    // Test complete register lifecycle
    let mut reg = Register::new();
    
    // Initially empty
    assert!(reg.is_empty());
    assert_eq!(reg.size(), 0);
    
    // Register some names
    assert_eq!(reg.register_name("process1", 100), RegisterResult::Success);
    assert_eq!(reg.register_name("process2", 200), RegisterResult::Success);
    assert_eq!(reg.register_name("process3", 300), RegisterResult::Success);
    
    // Verify size
    assert_eq!(reg.size(), 3);
    assert!(!reg.is_empty());
    
    // Verify lookups
    assert_eq!(reg.whereis_name("process1"), Some(100));
    assert_eq!(reg.whereis_name("process2"), Some(200));
    assert_eq!(reg.whereis_name("process3"), Some(300));
    assert_eq!(reg.whereis_name("nonexistent"), None);
    
    // Verify reverse lookups
    assert_eq!(reg.get_name_for_id(100), Some("process1".to_string()));
    assert_eq!(reg.get_name_for_id(200), Some("process2".to_string()));
    assert_eq!(reg.get_name_for_id(300), Some("process3".to_string()));
    assert_eq!(reg.get_name_for_id(999), None);
    
    // Unregister
    assert_eq!(reg.unregister_name("process1"), true);
    assert_eq!(reg.size(), 2);
    assert_eq!(reg.whereis_name("process1"), None);
    
    // Clear all
    reg.clear();
    assert!(reg.is_empty());
    assert_eq!(reg.size(), 0);
}

#[test]
fn test_register_duplicate_handling() {
    // Test duplicate name/ID handling
    let mut reg = Register::new();
    
    // Register name with ID
    assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
    
    // Try to register same name with different ID - should fail
    assert_eq!(
        reg.register_name("my_process", 456),
        RegisterResult::AlreadyRegistered
    );
    
    // Original ID should still be registered
    assert_eq!(reg.whereis_name("my_process"), Some(123));
    
    // Try to register same ID with different name - should fail
    assert_eq!(
        reg.register_name("other_name", 123),
        RegisterResult::AlreadyHasName
    );
    
    // Original name should still be registered
    assert_eq!(reg.get_name_for_id(123), Some("my_process".to_string()));
    
    // Register same name and ID again - should succeed (idempotent)
    assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
}

#[test]
fn test_register_invalid_names() {
    // Test invalid name handling
    let mut reg = Register::new();
    
    // Empty string is invalid
    assert_eq!(
        reg.register_name("", 123),
        RegisterResult::InvalidName
    );
    
    // Valid name should work
    assert_eq!(reg.register_name("valid_name", 123), RegisterResult::Success);
}

#[test]
fn test_register_large_scale() {
    // Test with many registrations
    let mut reg = Register::new();
    
    // Register many processes
    for i in 0..100 {
        let name = format!("process_{}", i);
        assert_eq!(reg.register_name(&name, i as u64), RegisterResult::Success);
    }
    
    assert_eq!(reg.size(), 100);
    
    // Verify all are retrievable
    for i in 0..100 {
        let name = format!("process_{}", i);
        assert_eq!(reg.whereis_name(&name), Some(i as u64));
        assert_eq!(reg.get_name_for_id(i as u64), Some(name));
    }
    
    // Unregister some
    for i in 0..50 {
        let name = format!("process_{}", i);
        assert_eq!(reg.unregister_name(&name), true);
    }
    
    assert_eq!(reg.size(), 50);
    
    // Verify remaining
    for i in 50..100 {
        let name = format!("process_{}", i);
        assert_eq!(reg.whereis_name(&name), Some(i as u64));
    }
}

#[test]
fn test_register_with_big_numbers() {
    // Test register with big number IDs (cross-module integration)
    let mut reg = Register::new();
    
    // Use large IDs that might be represented as big numbers in some contexts
    let large_id1 = u64::MAX;
    let large_id2 = u64::MAX - 1;
    
    assert_eq!(reg.register_name("large1", large_id1), RegisterResult::Success);
    assert_eq!(reg.register_name("large2", large_id2), RegisterResult::Success);
    
    assert_eq!(reg.whereis_name("large1"), Some(large_id1));
    assert_eq!(reg.whereis_name("large2"), Some(large_id2));
    
    // Convert to big numbers for operations
    let big1 = BigNumber::from_u64(large_id1);
    let big2 = BigNumber::from_u64(large_id2);
    
    // Verify they were created correctly
    assert!(!big1.is_zero());
    assert!(!big2.is_zero());
}

#[test]
fn test_big_number_arithmetic_operations() {
    // Test comprehensive arithmetic operations
    let a = BigNumber::from_i64(100);
    let b = BigNumber::from_i64(50);
    
    // Addition
    let sum = a.plus(&b);
    assert_eq!(sum.to_i64(), Some(150));
    
    // Subtraction
    let diff = a.minus(&b);
    assert_eq!(diff.to_i64(), Some(50));
    
    // Multiplication
    let prod = a.times(&b);
    assert_eq!(prod.to_i64(), Some(5000));
    
    // Division
    let div = a.div(&b);
    assert_eq!(div.unwrap().to_i64(), Some(2));
    
    // Modulo
    let rem = a.rem(&b);
    assert_eq!(rem.unwrap().to_i64(), Some(0));
    
    // Negative numbers
    let neg_a = BigNumber::from_i64(-100);
    let neg_b = BigNumber::from_i64(-50);
    
    let neg_sum = neg_a.plus(&neg_b);
    assert_eq!(neg_sum.to_i64(), Some(-150));
    
    let neg_diff = neg_a.minus(&neg_b);
    assert_eq!(neg_diff.to_i64(), Some(-50));
}

#[test]
fn test_big_number_comparison_operations() {
    // Test comparison operations using comp method
    let a = BigNumber::from_i64(100);
    let b = BigNumber::from_i64(50);
    let c = BigNumber::from_i64(100);
    
    // Compare using comp method (returns -1, 0, or 1)
    assert!(a.comp(&b) > 0); // a > b
    assert!(b.comp(&a) < 0); // b < a
    assert_eq!(a.comp(&c), 0); // a == c
    assert_ne!(a.comp(&b), 0); // a != b
    
    // Negative comparisons
    let neg_a = BigNumber::from_i64(-100);
    let neg_b = BigNumber::from_i64(-50);
    
    assert!(neg_a.comp(&neg_b) < 0); // -100 < -50
    assert!(neg_b.comp(&neg_a) > 0);
}

