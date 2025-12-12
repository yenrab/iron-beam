//! Integration tests for usecases_process_management crate
//!
//! These tests verify that process management operations work correctly
//! and test end-to-end workflows for process locking, dictionaries, dumps, and code tracking.

use usecases_process_management::{ProcessLock, ProcessDict, ProcessDump};
use usecases_process_management::process_code_tracking::{
    check_process_uses_module,
    any_process_uses_module,
    any_dirty_process_uses_module,
    check_nif_in_module_area,
    check_continuation_pointers_in_module,
    pointer_in_module_area,
    ModuleCodeArea,
};
use entities_process::Process;
use entities_data_handling::term_hashing::Term;
use infrastructure_utilities::process_table::get_global_process_table;
use std::sync::Arc;

#[test]
fn test_process_lock_basic() {
    let lock = ProcessLock::new();
    
    // Acquire lock for process 1, lock 0
    lock.acquire(1, 0);
    
    // Release lock
    lock.release(0);
}

#[test]
fn test_process_lock_multiple_locks() {
    let lock = ProcessLock::new();
    
    // Acquire multiple different locks
    lock.acquire(1, 0);
    lock.acquire(2, 1);
    lock.acquire(3, 2);
    
    // Release them
    lock.release(0);
    lock.release(1);
    lock.release(2);
}

#[test]
fn test_process_lock_release_nonexistent() {
    let lock = ProcessLock::new();
    
    // Release a lock that was never acquired (should not panic)
    lock.release(999);
}

#[test]
fn test_process_dict_basic() {
    let mut dict = ProcessDict::new();
    
    // Test empty dictionary
    assert!(dict.is_empty());
    assert_eq!(dict.keys().len(), 0);
    
    // Put a value
    let key = Term::Atom(1);
    let value = Term::Small(42);
    let old_value = dict.put(key.clone(), value.clone());
    assert!(old_value.is_none());
    
    // Get the value
    assert_eq!(dict.get(&key), Some(&value));
    assert!(!dict.is_empty());
    assert_eq!(dict.keys().len(), 1);
}

#[test]
fn test_process_dict_update() {
    let mut dict = ProcessDict::new();
    
    let key = Term::Atom(1);
    let value1 = Term::Small(42);
    let value2 = Term::Small(100);
    
    // Put initial value
    dict.put(key.clone(), value1.clone());
    
    // Update with new value
    let old_value = dict.put(key.clone(), value2.clone());
    assert_eq!(old_value, Some(value1));
    
    // Verify new value
    assert_eq!(dict.get(&key), Some(&value2));
}

#[test]
fn test_process_dict_erase() {
    let mut dict = ProcessDict::new();
    
    let key = Term::Atom(1);
    let value = Term::Small(42);
    
    // Put value
    dict.put(key.clone(), value.clone());
    assert_eq!(dict.get(&key), Some(&value));
    
    // Erase value
    let erased = dict.erase(&key);
    assert_eq!(erased, Some(value));
    assert_eq!(dict.get(&key), None);
    assert!(dict.is_empty());
}

#[test]
fn test_process_dict_multiple_keys() {
    let mut dict = ProcessDict::new();
    
    let key1 = Term::Atom(1);
    let key2 = Term::Atom(2);
    let key3 = Term::Small(3);
    
    let value1 = Term::Small(10);
    let value2 = Term::Small(20);
    let value3 = Term::Small(30);
    
    // Put multiple values
    dict.put(key1.clone(), value1.clone());
    dict.put(key2.clone(), value2.clone());
    dict.put(key3.clone(), value3.clone());
    
    // Verify all values
    assert_eq!(dict.get(&key1), Some(&value1));
    assert_eq!(dict.get(&key2), Some(&value2));
    assert_eq!(dict.get(&key3), Some(&value3));
    assert_eq!(dict.keys().len(), 3);
}

#[test]
fn test_process_dict_clear() {
    let mut dict = ProcessDict::new();
    
    // Add multiple values
    dict.put(Term::Atom(1), Term::Small(10));
    dict.put(Term::Atom(2), Term::Small(20));
    dict.put(Term::Small(3), Term::Small(30));
    
    assert!(!dict.is_empty());
    assert_eq!(dict.keys().len(), 3);
    
    // Clear dictionary
    dict.clear();
    
    assert!(dict.is_empty());
    assert_eq!(dict.keys().len(), 0);
}

#[test]
fn test_process_dump_basic() {
    let process = Process::new(123);
    let dump = ProcessDump::dump(&process);
    
    assert!(!dump.is_empty());
    assert!(dump.contains("Process Dump"));
    assert!(dump.contains("Process ID: 123"));
    assert!(dump.contains("Heap Size"));
    assert!(dump.contains("State"));
    assert!(dump.contains("Reductions"));
}

#[test]
fn test_process_dump_by_id() {
    let table = get_global_process_table();
    let process = Arc::new(Process::new(456));
    table.insert(456, Arc::clone(&process));
    
    let dump = ProcessDump::dump_by_id(456);
    assert!(!dump.is_empty());
    assert!(dump.contains("Process ID: 456"));
    
    // Test with non-existent process
    let dump_not_found = ProcessDump::dump_by_id(999);
    assert!(dump_not_found.contains("not found"));
}

#[test]
fn test_process_dump_contains_all_info() {
    let process = Process::new(789);
    let dump = ProcessDump::dump(&process);
    
    // Verify all expected fields are present
    assert!(dump.contains("Process ID"));
    assert!(dump.contains("State"));
    assert!(dump.contains("Heap Size"));
    assert!(dump.contains("Min Heap Size"));
    assert!(dump.contains("Max Heap Size"));
    assert!(dump.contains("Flags"));
    assert!(dump.contains("Reductions"));
    assert!(dump.contains("FCalls"));
    assert!(dump.contains("Arity"));
    assert!(dump.contains("Catches"));
    assert!(dump.contains("Return Trace Frames"));
    assert!(dump.contains("Heap Start Index"));
    assert!(dump.contains("Heap Top Index"));
    assert!(dump.contains("Program Counter"));
    assert!(dump.contains("Unique"));
    assert!(dump.contains("Schedule Count"));
    assert!(dump.contains("Suspend Count"));
}

#[test]
fn test_module_code_area_creation() {
    let code_start = 0x1000 as *const u8;
    let code_size = 4096;
    
    let module_code = ModuleCodeArea::new(code_start, code_size);
    
    assert_eq!(module_code.code_start, code_start);
    assert_eq!(module_code.code_size, code_size);
    assert!(module_code.is_valid());
}

#[test]
fn test_module_code_area_empty() {
    let module_code = ModuleCodeArea::empty();
    
    assert!(module_code.code_start.is_null());
    assert_eq!(module_code.code_size, 0);
    assert!(!module_code.is_valid());
}

#[test]
fn test_pointer_in_module_area() {
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    
    // Pointer inside module area (must be within range)
    // Use a pointer that's actually within the module area
    let ptr_in = unsafe { mod_start.add(100) };
    assert!(pointer_in_module_area(ptr_in, mod_start, mod_size));
    
    // Pointer at start of module area
    assert!(pointer_in_module_area(mod_start, mod_start, mod_size));
    
    // Pointer just before end of module area
    let ptr_near_end = unsafe { mod_start.add(mod_size as usize - 1) };
    assert!(pointer_in_module_area(ptr_near_end, mod_start, mod_size));
    
    // Pointer outside module area (after)
    let ptr_out_after = unsafe { mod_start.add(mod_size as usize) };
    assert!(!pointer_in_module_area(ptr_out_after, mod_start, mod_size));
    
    // Pointer outside module area (before)
    let ptr_out_before = unsafe { mod_start.sub(1) };
    assert!(!pointer_in_module_area(ptr_out_before, mod_start, mod_size));
    
    // Null pointer
    assert!(!pointer_in_module_area(std::ptr::null(), mod_start, mod_size));
    
    // Null module start
    assert!(!pointer_in_module_area(ptr_in, std::ptr::null(), mod_size));
}

#[test]
fn test_check_process_uses_module() {
    let process = Process::new(1);
    
    // Test with empty/invalid module code area
    let empty_module = ModuleCodeArea::empty();
    assert!(!check_process_uses_module(&process, &empty_module));
    
    // Test with valid module code area but process doesn't use it
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    let module_code = ModuleCodeArea::new(mod_start, mod_size);
    
    // Process with null instruction pointer shouldn't use module
    assert!(!check_process_uses_module(&process, &module_code));
}

#[test]
fn test_any_process_uses_module() {
    // Test with empty module code area
    let empty_module = ModuleCodeArea::empty();
    assert!(!any_process_uses_module(&empty_module));
    
    // Test with valid module code area
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    let module_code = ModuleCodeArea::new(mod_start, mod_size);
    
    // With no processes using the module, should return false
    let uses = any_process_uses_module(&module_code);
    // Result depends on process table state, just verify it doesn't panic
    let _ = uses;
}

#[test]
fn test_any_dirty_process_uses_module() {
    // Test with empty module code area
    let empty_module = ModuleCodeArea::empty();
    assert!(!any_dirty_process_uses_module(&empty_module));
    
    // Test with valid module code area
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    let module_code = ModuleCodeArea::new(mod_start, mod_size);
    
    // With no dirty processes using the module, should return false
    let uses = any_dirty_process_uses_module(&module_code);
    // Result depends on process table state, just verify it doesn't panic
    let _ = uses;
}

#[test]
fn test_check_nif_in_module_area() {
    let process = Process::new(1);
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    
    // Process with no NIF pointers shouldn't have NIFs in module area
    assert!(!check_nif_in_module_area(&process, mod_start, mod_size));
}

#[test]
fn test_check_continuation_pointers_in_module() {
    let process = Process::new(1);
    let mod_start = 0x1000 as *const u8;
    let mod_size = 4096;
    
    // Process with no continuation pointers shouldn't have pointers in module area
    assert!(!check_continuation_pointers_in_module(&process, mod_start, mod_size));
}

#[test]
fn test_process_dict_with_various_term_types() {
    let mut dict = ProcessDict::new();
    
    // Test with different term types as keys and values
    dict.put(Term::Atom(1), Term::Small(42));
    dict.put(Term::Small(2), Term::Atom(3));
    dict.put(Term::Nil, Term::Small(0));
    
    let tuple_key = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
    let list_value = Term::List {
        head: Box::new(Term::Small(10)),
        tail: Box::new(Term::Nil),
    };
    dict.put(tuple_key.clone(), list_value.clone());
    
    // Verify all entries
    assert_eq!(dict.get(&Term::Atom(1)), Some(&Term::Small(42)));
    assert_eq!(dict.get(&Term::Small(2)), Some(&Term::Atom(3)));
    assert_eq!(dict.get(&Term::Nil), Some(&Term::Small(0)));
    assert_eq!(dict.get(&tuple_key), Some(&list_value));
}

#[test]
fn test_process_lock_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let lock = Arc::new(ProcessLock::new());
    let mut handles = vec![];
    
    // Spawn multiple threads trying to acquire the same lock
    for i in 0..5 {
        let lock_clone = lock.clone();
        let handle = thread::spawn(move || {
            lock_clone.acquire(i, 0);
            // Simulate some work
            thread::sleep(std::time::Duration::from_millis(10));
            lock_clone.release(0);
            i
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        let _ = handle.join().unwrap();
    }
    
    // Lock should be released now
    lock.release(0); // Should not panic
}

#[test]
fn test_process_dump_with_heap_data() {
    let process = Process::new(1);
    
    // Allocate some heap space
    let _ = process.allocate_heap_words(10);
    
    let dump = ProcessDump::dump(&process);
    
    // Verify heap information is in dump
    assert!(dump.contains("Heap Data Length"));
    assert!(dump.contains("Heap Top Index"));
}

#[test]
fn test_module_code_area_edge_cases() {
    // Test with zero size
    let mod_start = 0x1000 as *const u8;
    let module_code = ModuleCodeArea::new(mod_start, 0);
    assert!(!module_code.is_valid());
    
    // Test with very small size
    let module_code_small = ModuleCodeArea::new(mod_start, 1);
    assert!(module_code_small.is_valid());
    
    // Test pointer at exact boundary
    let ptr_at_start = mod_start;
    assert!(pointer_in_module_area(ptr_at_start, mod_start, 100));
    
    let ptr_just_after = unsafe { mod_start.add(100) };
    assert!(!pointer_in_module_area(ptr_just_after, mod_start, 100));
}

