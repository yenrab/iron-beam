//! Integration tests for infrastructure_ets_tables crate
//!
//! These tests verify that ETS table operations work correctly
//! and test end-to-end workflows for key-value storage.

use infrastructure_ets_tables::EtsTable;

#[test]
fn test_ets_table_creation() {
    let table = EtsTable::new();
    // Table is created (can't verify empty state without size() method)
}

#[test]
fn test_ets_table_insert_lookup() {
    let mut table = EtsTable::new();
    
    // Insert key-value pair
    let previous = table.insert(1, 100);
    assert!(previous.is_none());
    
    // Lookup value
    let found = table.lookup(1);
    assert_eq!(found, Some(100));
}

#[test]
fn test_ets_table_replace() {
    let mut table = EtsTable::new();
    
    // Insert initial value
    table.insert(1, 100);
    assert_eq!(table.lookup(1), Some(100));
    
    // Replace with new value
    let previous = table.insert(1, 200);
    assert_eq!(previous, Some(100));
    assert_eq!(table.lookup(1), Some(200));
}

#[test]
fn test_ets_table_multiple_keys() {
    let mut table = EtsTable::new();
    
    // Insert multiple key-value pairs
    table.insert(1, 100);
    table.insert(2, 200);
    table.insert(3, 300);
    
    // Lookup all values
    assert_eq!(table.lookup(1), Some(100));
    assert_eq!(table.lookup(2), Some(200));
    assert_eq!(table.lookup(3), Some(300));
}

#[test]
fn test_ets_table_lookup_nonexistent() {
    let table = EtsTable::new();
    
    // Lookup non-existent key
    let found = table.lookup(999);
    assert!(found.is_none());
}

#[test]
fn test_ets_table_various_key_values() {
    let mut table = EtsTable::new();
    
    // Test with various key-value combinations
    let test_cases = vec![
        (0u64, 0u64),
        (1u64, 1u64),
        (42u64, 100u64),
        (u64::MAX, u64::MAX),
        (1000u64, 2000u64),
    ];
    
    for (key, value) in test_cases {
        table.insert(key, value);
        assert_eq!(table.lookup(key), Some(value));
    }
}

#[test]
fn test_ets_table_insert_returns_previous() {
    let mut table = EtsTable::new();
    
    // First insert returns None
    let previous1 = table.insert(1, 100);
    assert!(previous1.is_none());
    
    // Second insert returns previous value
    let previous2 = table.insert(1, 200);
    assert_eq!(previous2, Some(100));
    
    // Third insert returns new previous value
    let previous3 = table.insert(1, 300);
    assert_eq!(previous3, Some(200));
}

#[test]
fn test_ets_table_sequential_operations() {
    let mut table = EtsTable::new();
    
    // Insert, lookup, replace sequence
    table.insert(1, 10);
    assert_eq!(table.lookup(1), Some(10));
    
    table.insert(2, 20);
    assert_eq!(table.lookup(2), Some(20));
    
    table.insert(1, 11); // Replace
    assert_eq!(table.lookup(1), Some(11));
    assert_eq!(table.lookup(2), Some(20)); // Other key unchanged
}
