//! Integration tests for infrastructure_utilities crate
//!
//! These tests verify that utility functions work correctly
//! and test end-to-end workflows for process table, atom table, and other utilities.

use infrastructure_utilities::*;
use entities_process::{Process, ProcessId};
use std::sync::Arc;

#[test]
fn test_process_table_creation() {
    let table = ProcessTable::new();
    assert_eq!(table.size(), 0);
    assert_eq!(table.max_size(), None);
}

#[test]
fn test_process_table_with_max_size() {
    let table = ProcessTable::with_max_size(1000);
    assert_eq!(table.size(), 0);
    assert_eq!(table.max_size(), Some(1000));
}

#[test]
fn test_process_table_insert_lookup() {
    let table = ProcessTable::new();
    let process = Arc::new(Process::new(1));
    
    // Insert process
    let previous = table.insert(1, Arc::clone(&process));
    assert!(previous.is_none()); // No previous process
    assert_eq!(table.size(), 1);
    
    // Lookup process
    let found = table.lookup(1);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), 1);
}

#[test]
fn test_process_table_remove() {
    let table = ProcessTable::new();
    let process = Arc::new(Process::new(2));
    
    // Insert and remove
    let previous = table.insert(2, Arc::clone(&process));
    assert!(previous.is_none());
    assert_eq!(table.size(), 1);
    
    let removed = table.remove(2);
    assert!(removed.is_some());
    assert_eq!(table.size(), 0);
    
    // Try to remove again
    let removed_again = table.remove(2);
    assert!(removed_again.is_none());
}

#[test]
fn test_process_table_multiple_processes() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=10 {
        let process = Arc::new(Process::new(i));
        table.insert(i, process);
    }
    
    assert_eq!(table.size(), 10);
    
    // Lookup all processes
    for i in 1..=10 {
        let found = table.lookup(i);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), i);
    }
}

#[test]
fn test_process_table_lookup_nonexistent() {
    let table = ProcessTable::new();
    
    // Lookup non-existent process
    let found = table.lookup(999);
    assert!(found.is_none());
}

#[test]
fn test_process_table_clear() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        table.insert(i, process);
    }
    
    assert_eq!(table.size(), 5);
    
    // Clear table
    table.clear();
    assert_eq!(table.size(), 0);
    
    // Verify all processes are gone
    for i in 1..=5 {
        assert!(table.lookup(i).is_none());
    }
}

#[test]
fn test_process_table_max_size_limit() {
    let table = ProcessTable::with_max_size(3);
    
    // Insert up to max size
    for i in 1..=3 {
        let process = Arc::new(Process::new(i));
        let previous = table.insert(i, process);
        assert!(previous.is_none());
    }
    
    assert_eq!(table.size(), 3);
    
    // Try to insert beyond max size - insert() doesn't check max_size, new_element() does
    // So we test new_element() instead
    let result = table.new_element(|_id| Arc::new(Process::new(0)));
    assert!(result.is_err());
    assert_eq!(table.size(), 3);
}

#[test]
fn test_get_global_process_table() {
    let table1 = get_global_process_table();
    let table2 = get_global_process_table();
    
    // Should return the same reference (singleton)
    assert!(std::ptr::eq(table1, table2));
}

#[test]
fn test_global_process_table_operations() {
    let table = get_global_process_table();
    
    // Insert a process
    let process = Arc::new(Process::new(100));
    let _previous = table.insert(100, Arc::clone(&process));
    
    // Lookup the process
    let found = table.lookup(100);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), 100);
    
    // Clean up
    table.remove(100);
}

#[test]
fn test_get_global_atom_table() {
    use entities_data_handling::AtomEncoding;
    
    let table1 = get_global_atom_table();
    let table2 = get_global_atom_table();
    
    // Should return the same reference (singleton)
    assert!(std::ptr::eq(table1, table2));
    
    // Test atom operations
    let index1 = table1.put_index(b"test_atom", AtomEncoding::Latin1, false).unwrap();
    let index2 = table2.put_index(b"test_atom", AtomEncoding::Latin1, false).unwrap();
    
    // Same atom should get same index
    assert_eq!(index1, index2);
}

#[test]
fn test_global_atom_table_operations() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Put atoms
    let index1 = table.put_index(b"atom1", AtomEncoding::Latin1, false).unwrap();
    let index2 = table.put_index(b"atom2", AtomEncoding::Latin1, false).unwrap();
    
    assert_ne!(index1, index2);
    
    // Get atom names
    let name1 = table.get_name(index1);
    assert!(name1.is_some());
    assert_eq!(name1.unwrap(), b"atom1");
    
    let name2 = table.get_name(index2);
    assert!(name2.is_some());
    assert_eq!(name2.unwrap(), b"atom2");
}

#[test]
fn test_process_table_error_cases() {
    let table = ProcessTable::with_max_size(1);
    
    // Insert first process using new_element (which checks max_size)
    let (id1, _process1) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
    assert_eq!(table.size(), 1);
    
    // Try to insert second process (should fail due to max size)
    let result = table.new_element(|id| Arc::new(Process::new(id)));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProcessTableError::TableFull);
    assert_eq!(table.size(), 1);
}

#[test]
fn test_process_table_concurrent_access() {
    use std::thread;
    use std::sync::Arc;
    
    let table = Arc::new(ProcessTable::new());
    
    // Spawn multiple threads inserting processes
    let mut handles = vec![];
    for i in 0..10 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            let process = Arc::new(Process::new(i));
            let _previous = table_clone.insert(i, process);
            i
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        let _ = handle.join().unwrap();
    }
    
    // Verify all processes were inserted
    assert_eq!(table.size(), 10);
    for i in 0..10 {
        assert!(table.lookup(i).is_some());
    }
}

#[test]
fn test_process_table_iteration() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    // Get all IDs and verify count
    let all_ids = table.get_all_ids();
    assert_eq!(all_ids.len(), 5);
}

#[test]
fn test_atom_table_encoding_variants() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Test different encodings
    let index1 = table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
    let index2 = table.put_index(b"test", AtomEncoding::Latin1, false).unwrap();
    let index3 = table.put_index(b"test", AtomEncoding::Utf8, false).unwrap();
    
    // Same atom name with different encodings may or may not get same index
    // depending on implementation
    let _ = (index1, index2, index3);
}

#[test]
fn test_process_table_replace() {
    let table = ProcessTable::new();
    
    // Insert initial process (should return None for new entry)
    let process1 = Arc::new(Process::new(1));
    let first_insert = table.insert(1, Arc::clone(&process1));
    assert!(first_insert.is_none()); // First insert returns None
    
    // Replace with new process
    let process2 = Arc::new(Process::new(1));
    let old_process = table.insert(1, Arc::clone(&process2));
    
    // Should return old process (replacement)
    assert!(old_process.is_some());
    assert_eq!(table.size(), 1);
    
    // Lookup should return new process
    let found = table.lookup(1);
    assert!(found.is_some());
}

#[test]
fn test_process_table_size_after_operations() {
    let table = ProcessTable::new();
    
    assert_eq!(table.size(), 0);
    
    // Insert processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
        assert_eq!(table.size(), i as usize);
    }
    
    // Remove processes
    for i in 1..=5 {
        table.remove(i);
        assert_eq!(table.size(), (5 - i) as usize);
    }
}

#[test]
fn test_process_table_empty_after_clear() {
    let table = ProcessTable::new();
    
    // Insert processes
    for i in 1..=10 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    assert_eq!(table.size(), 10);
    assert!(!table.is_empty());
    
    // Clear
    table.clear();
    
    assert_eq!(table.size(), 0);
    assert!(table.is_empty());
}

#[test]
fn test_process_table_max_size_none() {
    let table = ProcessTable::new();
    assert_eq!(table.max_size(), None);
    
    // Should be able to insert many processes
    for i in 1..=100 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    assert_eq!(table.size(), 100);
}

#[test]
fn test_atom_table_get_nonexistent() {
    let table = get_global_atom_table();
    
    // Try to get non-existent atom
    let name = table.get_name(99999);
    assert!(name.is_none());
}

#[test]
fn test_atom_table_duplicate_atoms() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Put same atom multiple times
    let index1 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    let index2 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    let index3 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    
    // All should get the same index
    assert_eq!(index1, index2);
    assert_eq!(index2, index3);
}

#[test]
fn test_process_table_error_enum() {
    // Test ProcessTableError enum
    let error = ProcessTableError::TableFull;
    let _ = format!("{:?}", error);
}
