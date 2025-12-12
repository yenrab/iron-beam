//! Integration tests for entities_process crate
//!
//! These tests verify that the Process entity works correctly
//! and test end-to-end process lifecycle operations.
//!
//! Note: Tests for unsafe code (unsafe impl Send/Sync) are excluded
//! as they require manual testing.

use entities_process::{Process, ProcessId, ProcessState};

#[test]
fn test_process_creation() {
    // Test creating a new process
    let process_id: ProcessId = 1;
    let process = Process::new(process_id);
    
    // Verify process ID
    assert_eq!(process.id(), process_id);
    assert_eq!(process.get_id(), process_id);
    
    // Verify initial state (flags: 0 maps to Unknown(0))
    let state = process.get_state();
    // Process starts with flags: 0, which maps to Unknown(0)
    assert!(matches!(state, ProcessState::Unknown(0)));
    
    // Verify initial heap size
    assert!(process.heap_sz() >= 233); // Default minimum heap size
    assert_eq!(process.min_heap_size(), 233);
    assert_eq!(process.max_heap_size(), 0); // 0 = unlimited
}

#[test]
fn test_process_heap_operations() {
    let process = Process::new(1);
    
    // Test heap size getters
    let heap_sz = process.heap_sz();
    assert!(heap_sz > 0);
    
    let min_heap = process.min_heap_size();
    assert!(min_heap > 0);
    
    let max_heap = process.max_heap_size();
    assert!(max_heap == 0 || max_heap >= min_heap);
    
    // Test heap index getters
    let heap_start = process.heap_start_index();
    assert_eq!(heap_start, 0);
    
    let heap_top = process.heap_top_index();
    assert!(heap_top >= heap_start);
    
    // Test heap slice access
    let heap_slice = process.heap_slice();
    assert_eq!(heap_slice.len(), heap_sz);
    
    // Test heap allocation
    let allocated_index = process.allocate_heap_words(10);
    assert!(allocated_index.is_some());
    let new_heap_top = process.heap_top_index();
    assert!(new_heap_top >= heap_top);
}

#[test]
fn test_process_stack_operations() {
    let process = Process::new(1);
    
    // Test stack top index (may be None initially)
    let _stack_top = process.stack_top_index();
    // Stack top can be None or Some(index)
    
    // Test stack size calculation
    let _stack_size = process.stack_size_words();
    // Stack size can be None if stack_top_index is None
}

#[test]
fn test_process_state_flags() {
    let process = Process::new(1);
    
    // Test state getter (initial state is Unknown(0) when flags: 0)
    let state = process.get_state();
    assert!(matches!(state, ProcessState::Unknown(0)));
    
    // Test flags getter
    let flags = process.flags();
    // Flags value depends on initial state
    assert!(flags >= 0);
}

#[test]
fn test_process_reductions() {
    let process = Process::new(1);
    
    // Test reductions getters
    let reds = process.reds();
    assert_eq!(reds, 0); // Initial reductions should be 0
    
    let fcalls = process.fcalls();
    assert_eq!(fcalls, 0); // Initial function calls should be 0
}

#[test]
fn test_process_registers() {
    let process = Process::new(1);
    
    // Test arity (number of live argument registers)
    let arity = process.arity();
    assert_eq!(arity, 0); // Initial arity should be 0
}

#[test]
fn test_process_exception_handling() {
    let process = Process::new(1);
    
    // Test catch count
    let catches = process.catches();
    assert_eq!(catches, 0); // Initial catch count should be 0
    
    // Test return trace frames
    let return_trace_frames = process.return_trace_frames();
    assert_eq!(return_trace_frames, 0); // Initial return trace frames should be 0
}

#[test]
fn test_process_code_pointer() {
    let process = Process::new(1);
    
    // Test instruction pointer getter
    let i = process.i();
    // Instruction pointer can be null initially
    // Just verify we can get it without panicking
    let _ = i;
}

#[test]
fn test_process_unique_integer() {
    let process = Process::new(1);
    
    // Test unique integer getter
    let uniq = process.uniq();
    // Unique integer can be any value
    let _ = uniq;
}

#[test]
fn test_process_scheduling() {
    let process = Process::new(1);
    
    // Test schedule count
    let schedule_count = process.schedule_count();
    assert_eq!(schedule_count, 0); // Initial schedule count should be 0
    
    // Test suspend count (rcount)
    let rcount = process.rcount();
    assert_eq!(rcount, 0); // Initial suspend count should be 0
}

#[test]
fn test_process_heap_accessors() {
    let process = Process::new(1);
    
    // Test heap() getter (returns heap start index) - deprecated but still testable
    #[allow(deprecated)]
    let heap = process.heap();
    assert_eq!(heap, Some(0)); // Heap start should be 0
    
    // Test htop() getter (returns heap top index) - deprecated but still testable
    #[allow(deprecated)]
    let htop = process.htop();
    assert!(htop.is_some());
    assert!(htop.unwrap() >= 0);
    
    // Test stop() getter (returns stack top index) - deprecated but still testable
    #[allow(deprecated)]
    let _stop = process.stop();
    // Stack top can be None or Some(index)
}

#[test]
fn test_process_nif_pointer_management() {
    let mut process = Process::new(1);
    
    // Test initial NIF pointers (should be empty)
    let nif_pointers = process.get_nif_pointers();
    assert_eq!(nif_pointers.len(), 0);
    
    // Test adding a NIF pointer (must be non-null)
    let test_value: u8 = 42;
    let test_pointer: *const u8 = &test_value as *const u8;
    let result = process.add_nif_pointer(test_pointer);
    assert!(result.is_ok());
    
    // Verify pointer was added
    let nif_pointers = process.get_nif_pointers();
    assert_eq!(nif_pointers.len(), 1);
    assert_eq!(nif_pointers[0], test_pointer);
    
    // Test removing a NIF pointer
    let result = process.remove_nif_pointer(test_pointer);
    assert!(result.is_ok());
    
    // Verify pointer was removed
    let nif_pointers = process.get_nif_pointers();
    assert_eq!(nif_pointers.len(), 0);
}

#[test]
fn test_process_nif_library_management() {
    let mut process = Process::new(1);
    
    // Test initial NIF libraries (should be empty)
    let nif_libraries = process.get_nif_libraries();
    assert_eq!(nif_libraries.len(), 0);
    
    // Test adding a NIF library
    let test_library: std::sync::Arc<dyn std::any::Any + Send + Sync> = 
        std::sync::Arc::new(42u64);
    let result = process.add_nif_library(test_library.clone());
    assert!(result.is_ok());
    
    // Verify library was added
    let nif_libraries = process.get_nif_libraries();
    assert_eq!(nif_libraries.len(), 1);
    
    // Test removing a NIF library
    let result = process.remove_nif_library(&test_library);
    assert!(result.is_ok());
    
    // Verify library was removed
    let nif_libraries = process.get_nif_libraries();
    assert_eq!(nif_libraries.len(), 0);
}

#[test]
fn test_process_multiple_heap_allocations() {
    let process = Process::new(1);
    
    // Allocate multiple heap blocks
    let index1 = process.allocate_heap_words(5);
    assert!(index1.is_some());
    
    let index2 = process.allocate_heap_words(10);
    assert!(index2.is_some());
    
    let index3 = process.allocate_heap_words(15);
    assert!(index3.is_some());
    
    // Verify heap top increased
    let final_heap_top = process.heap_top_index();
    assert!(final_heap_top >= 30); // At least 5 + 10 + 15 words
}

#[test]
fn test_process_heap_slice_mut() {
    let process = Process::new(1);
    
    // Test mutable heap slice access
    let mut heap_slice = process.heap_slice_mut();
    assert!(heap_slice.len() > 0);
    
    // Modify heap data
    if heap_slice.len() > 10 {
        heap_slice[10] = 42;
        assert_eq!(heap_slice[10], 42);
    }
}

#[test]
fn test_process_state_enum() {
    // Test ProcessState enum variants
    let states = vec![
        ProcessState::Free,
        ProcessState::Exiting,
        ProcessState::Active,
        ProcessState::Running,
        ProcessState::Suspended,
        ProcessState::Gc,
        ProcessState::SysTasks,
        ProcessState::RunningSys,
        ProcessState::Proxy,
        ProcessState::DelayedSys,
        ProcessState::DirtyRunning,
        ProcessState::DirtyRunningSys,
        ProcessState::Unknown(999),
    ];
    
    // Verify all states can be created and compared
    for state in &states {
        let _ = format!("{:?}", state);
    }
    
    // Test equality
    assert_eq!(ProcessState::Active, ProcessState::Active);
    assert_ne!(ProcessState::Active, ProcessState::Running);
}

#[test]
fn test_process_error_cases() {
    let mut process = Process::new(1);
    
    // Test removing non-existent NIF pointer
    let test_value: u8 = 99;
    let non_existent_pointer: *const u8 = &test_value as *const u8;
    let result = process.remove_nif_pointer(non_existent_pointer);
    // Should succeed (remove_nif_pointer always returns Ok)
    assert!(result.is_ok());
    
    // Test removing non-existent NIF library
    let non_existent_library: std::sync::Arc<dyn std::any::Any + Send + Sync> = 
        std::sync::Arc::new(999u64);
    let result = process.remove_nif_library(&non_existent_library);
    // Should return error or handle gracefully
    let _ = result;
}

#[test]
fn test_process_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let process = Arc::new(Process::new(1));
    
    // Test concurrent heap access
    let process1 = process.clone();
    let process2 = process.clone();
    
    let handle1 = thread::spawn(move || {
        let _ = process1.heap_slice();
        process1.heap_top_index()
    });
    
    let handle2 = thread::spawn(move || {
        let _ = process2.heap_slice();
        process2.heap_top_index()
    });
    
    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    
    // Both threads should get the same heap top index
    assert_eq!(result1, result2);
}
