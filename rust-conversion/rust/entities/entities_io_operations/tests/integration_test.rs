//! Integration tests for entities_io_operations crate
//!
//! These tests verify export table operations end-to-end.

use entities_io_operations::*;

#[test]
fn test_export_table_lifecycle() {
    // Test complete lifecycle: create, add, query, remove, clear
    let table = ExportTable::new();

    // Initially empty
    assert_eq!(table.table_size(), 0);
    assert!(table.list().is_empty());

    // Add some exports
    let export1 = table.put(1, 2, 3); // module=1, function=2, arity=3
    let export2 = table.put(4, 5, 6);
    let export3 = table.put(7, 8, 9);

    // Verify size
    assert_eq!(table.table_size(), 3);
    assert_eq!(table.list_size(), 3);

    // Verify we can get them
    assert_eq!(table.get(1, 2, 3), Some(export1.clone()));
    assert_eq!(table.get(4, 5, 6), Some(export2.clone()));
    assert_eq!(table.get(7, 8, 9), Some(export3.clone()));

    // Verify list contains all
    let exports = table.list();
    assert_eq!(exports.len(), 3);

    // Remove one
    let removed = table.remove(4, 5, 6);
    assert!(removed.is_some());
    assert_eq!(table.table_size(), 2);
    assert_eq!(table.get(4, 5, 6), None);

    // Clear all
    table.clear();
    assert_eq!(table.table_size(), 0);
    assert!(table.list().is_empty());
}

#[test]
fn test_export_table_duplicate_handling() {
    // Test that duplicate MFAs are handled correctly
    let table = ExportTable::new();

    // Put same MFA twice
    let export1 = table.put(1, 2, 3);
    let export2 = table.put(1, 2, 3);

    // Should only have one entry
    assert_eq!(table.table_size(), 1);
    assert_eq!(export1.mfa, export2.mfa);

    // Both should return the same export
    assert_eq!(table.get(1, 2, 3), Some(export1.clone()));
}

#[test]
fn test_export_table_different_arities() {
    // Test that same module/function with different arities are different exports
    let table = ExportTable::new();

    // Same module and function, different arities
    table.put(1, 2, 0);
    table.put(1, 2, 1);
    table.put(1, 2, 2);
    table.put(1, 2, 3);

    // Should have 4 different exports
    assert_eq!(table.table_size(), 4);

    // All should be retrievable
    assert!(table.contains(1, 2, 0));
    assert!(table.contains(1, 2, 1));
    assert!(table.contains(1, 2, 2));
    assert!(table.contains(1, 2, 3));
}

#[test]
fn test_export_table_get_or_make_stub() {
    // Test stub creation and retrieval
    let table = ExportTable::new();

    // Get or make stub for non-existent export
    let stub1 = table.get_or_make_stub(10, 20, 30);
    assert_eq!(table.table_size(), 1);
    assert_eq!(stub1.mfa.module, 10);
    assert_eq!(stub1.mfa.function, 20);
    assert_eq!(stub1.mfa.arity, 30);
    assert!(stub1.is_stub); // Should be marked as stub

    // Get or make stub for existing export - should return existing stub
    let stub2 = table.get_or_make_stub(10, 20, 30);
    assert_eq!(table.table_size(), 1); // Still only one
    assert_eq!(stub1.mfa, stub2.mfa);
    assert!(stub2.is_stub); // Should still be a stub
}

#[test]
fn test_export_table_large_scale() {
    // Test with many exports
    let table = ExportTable::new();

    // Add many exports
    for i in 0..100 {
        table.put(i, i + 100, i + 200);
    }

    assert_eq!(table.table_size(), 100);

    // Verify we can retrieve them
    for i in 0..100 {
        assert!(table.contains(i, i + 100, i + 200));
        let export = table.get(i, i + 100, i + 200);
        assert!(export.is_some());
        let export = export.unwrap();
        assert_eq!(export.mfa.module, i);
        assert_eq!(export.mfa.function, i + 100);
        assert_eq!(export.mfa.arity, i + 200);
    }

    // Verify list contains all
    let exports = table.list();
    assert_eq!(exports.len(), 100);
}

#[test]
fn test_export_table_entry_bytes() {
    // Test entry bytes calculation
    let table = ExportTable::new();

    // Empty table
    assert_eq!(table.entry_bytes(), 0);

    // Add some exports
    table.put(1, 2, 3);
    table.put(4, 5, 6);

    // Should have some bytes
    let bytes = table.entry_bytes();
    assert!(bytes > 0);

    // Adding more should increase bytes
    table.put(7, 8, 9);
    let new_bytes = table.entry_bytes();
    assert!(new_bytes > bytes);
}

#[test]
fn test_export_table_with_bif() {
    // Test BIF exports
    let table = ExportTable::new();

    // Create a BIF export manually (since put creates regular exports)
    // We'll test that regular exports work and can be queried
    let export = table.put(1, 2, 3);
    assert!(!export.is_bif()); // Regular export is not a BIF

    // Verify we can still use it
    assert_eq!(table.get(1, 2, 3), Some(export.clone()));
}

#[test]
fn test_export_table_mfa_hash_consistency() {
    // Test that MFA hashing is consistent
    let table = ExportTable::new();

    // Add exports
    let export1 = table.put(1, 2, 3);
    let export2 = table.put(4, 5, 6);

    // Hash values should be different for different MFAs
    assert_ne!(export1.hash(), export2.hash());

    // Same MFA should have same hash
    let export3 = table.put(1, 2, 3);
    assert_eq!(export1.hash(), export3.hash());
}
