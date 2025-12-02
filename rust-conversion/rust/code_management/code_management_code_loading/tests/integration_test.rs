//! Integration tests for code_management_code_loading crate
//!
//! These tests verify that multiple modules work together correctly
//! and test end-to-end workflows for code loading, permissions, barriers, and debugging.

use code_management_code_loading::*;
use code_management_code_loading::code_permissions::{CodePermissionManager, ProcessId};
use code_management_code_loading::code_barriers::{CodeBarrierManager, CodeBarrier};
use code_management_code_loading::beam_debug::{BeamDebugTracer, Mfa};
use code_management_code_loading::code_index::CodeIndexManager;
use code_management_code_loading::module_management::ModuleTableManager;

#[test]
fn test_code_permission_and_barrier_integration() {
    // Test that code permissions and barriers work together
    let perm_manager = CodePermissionManager::new();
    let barrier_manager = CodeBarrierManager::new();
    
    perm_manager.init();
    barrier_manager.init();
    
    let process_id: ProcessId = 1;
    
    // Seize code modification permission
    assert!(perm_manager.try_seize_code_mod_permission(process_id));
    assert!(perm_manager.has_code_mod_permission(process_id));
    
    // Create and schedule a code barrier
    let mut barrier = CodeBarrier::new();
    let barrier_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let barrier_called_clone = barrier_called.clone();
    
    barrier_manager.schedule_code_barrier(
        &mut barrier,
        Box::new(move || {
            barrier_called_clone.store(true, std::sync::atomic::Ordering::Release);
        }),
        None,
    );
    
    // Release permission
    perm_manager.release_code_mod_permission();
    assert!(!perm_manager.has_code_mod_permission(process_id));
    
    // In a full implementation, the barrier would be executed here
    // For now, we just verify the barrier was set up correctly
    assert_eq!(barrier.pending_schedulers(), 1);
}

#[test]
fn test_code_loading_workflow_integration() {
    // Test a complete code loading workflow:
    // 1. Seize code load permission
    // 2. Start staging code index
    // 3. Load module
    // 4. Commit staging
    // 5. Release permission
    
    let perm_manager = CodePermissionManager::new();
    let code_ix = CodeIndexManager::new();
    let module_table = ModuleTableManager::new();
    let code_loader = code_management_code_loading::code_loader::CodeLoader;
    
    perm_manager.init();
    code_ix.init();
    module_table.init();
    
    let process_id: ProcessId = 1;
    
    // Step 1: Seize code load permission (both staging and modification)
    assert!(perm_manager.try_seize_code_load_permission(process_id));
    assert!(perm_manager.has_code_load_permission(process_id));
    
    // Step 2: Start staging code index
    code_ix.start_staging(0);
    // After start_staging, staging index should be the next index (1)
    assert_eq!(code_ix.staging_code_ix(), 1);
    
    // Step 3: Load a module (simplified - just verify the loader works)
    let test_code = b"test module code";
    let result = code_management_code_loading::code_loader::CodeLoader::verify_module(test_code);
    assert!(result); // Non-empty code should verify
    
    // Step 4: End and commit staging
    code_ix.end_staging();
    code_ix.commit_staging();
    
    // Step 5: Release permission
    perm_manager.release_code_load_permission();
    assert!(!perm_manager.has_code_load_permission(process_id));
}

#[test]
fn test_debug_tracing_with_code_loading_integration() {
    // Test that debug tracing works with code loading operations
    let tracer = BeamDebugTracer::new();
    let perm_manager = CodePermissionManager::new();
    
    perm_manager.init();
    
    let process_id: ProcessId = 1;
    
    // Set up debug tracing for a module
    let module_name = "my_module";
    let function_name = "my_function";
    let arity = 2;
    
    let trace_index = tracer.set_traced_mfa(module_name, function_name, arity);
    assert!(trace_index.is_some());
    let trace_index = trace_index.unwrap();
    
    // Seize code modification permission (needed for tracing)
    assert!(perm_manager.try_seize_code_mod_permission(process_id));
    
    // Check if MFA is traced
    let module_atom = tracer.string_to_atom(module_name);
    let function_atom = tracer.string_to_atom(function_name);
    let is_traced = tracer.is_traced_mfa(module_atom, Some(function_atom), arity);
    assert_eq!(is_traced, trace_index);
    
    // Format a trace message
    let trace_msg = tracer.vtrace_mfa(trace_index, "Calling {0} with {1}", &[&function_name, &"args"]);
    assert!(trace_msg.is_some());
    
    // Release permission
    perm_manager.release_code_mod_permission();
}

#[test]
fn test_code_barrier_with_permissions_integration() {
    // Test that code barriers coordinate with permissions
    let perm_manager = CodePermissionManager::new();
    let barrier_manager = CodeBarrierManager::new();
    
    perm_manager.init();
    barrier_manager.init();
    
    let process_id: ProcessId = 1;
    
    // Seize permission
    assert!(perm_manager.try_seize_code_mod_permission(process_id));
    
    // Issue blocking code barrier
    barrier_manager.blocking_code_barrier();
    assert_eq!(barrier_manager.outstanding_blocking_code_barriers(), 1);
    
    // Release permission
    perm_manager.release_code_mod_permission();
    
    // In a full implementation, the barrier would ensure all threads
    // have seen the permission release before proceeding
}

#[test]
fn test_module_management_with_permissions_integration() {
    // Test module management operations with code permissions
    let perm_manager = CodePermissionManager::new();
    let module_table = ModuleTableManager::new();
    
    perm_manager.init();
    module_table.init();
    
    let process_id: ProcessId = 1;
    
    // Seize code modification permission
    assert!(perm_manager.try_seize_code_mod_permission(process_id));
    
    // In a full implementation, would use module_table to create module
    // For now, just verify permissions are held
    
    // Release permission
    perm_manager.release_code_mod_permission();
}

#[test]
fn test_code_index_with_barriers_integration() {
    // Test code index operations with barriers
    let code_ix = CodeIndexManager::new();
    let barrier_manager = CodeBarrierManager::new();
    
    code_ix.init();
    barrier_manager.init();
    
    // Start staging
    code_ix.start_staging(1);
    
    // Issue a barrier to ensure all threads see the staging change
    barrier_manager.blocking_code_barrier();
    
    // End and commit staging
    code_ix.end_staging();
    code_ix.commit_staging();
    
    // Verify active index was updated
    // After commit, active should be the staging index (1)
    assert_eq!(code_ix.active_code_ix(), 1);
}

#[test]
fn test_multiple_processes_code_permissions_integration() {
    // Test that multiple processes can queue for permissions
    let perm_manager = CodePermissionManager::new();
    perm_manager.init();
    
    let process1: ProcessId = 1;
    let process2: ProcessId = 2;
    let process3: ProcessId = 3;
    
    // Process 1 seizes permission
    assert!(perm_manager.try_seize_code_mod_permission(process1));
    assert!(perm_manager.has_code_mod_permission(process1));
    
    // Process 2 tries to seize (should fail and queue)
    assert!(!perm_manager.try_seize_code_mod_permission(process2));
    assert!(!perm_manager.has_code_mod_permission(process2));
    
    // Process 3 tries to seize (should also fail and queue)
    assert!(!perm_manager.try_seize_code_mod_permission(process3));
    assert!(!perm_manager.has_code_mod_permission(process3));
    
    // Process 1 releases - should allow next process to get permission
    perm_manager.release_code_mod_permission();
    
    // In a full implementation, process 2 would now get the permission
    // For now, we just verify the queueing mechanism works
}

#[test]
fn test_debug_tracing_multiple_mfas_integration() {
    // Test tracing multiple MFAs
    let tracer = BeamDebugTracer::new();
    
    // Set up tracing for multiple functions
    let index1 = tracer.set_traced_mfa("module1", "func1", 1);
    let index2 = tracer.set_traced_mfa("module1", "func2", 2);
    let index3 = tracer.set_traced_mfa("module2", "func1", 1);
    
    assert!(index1.is_some());
    assert!(index2.is_some());
    assert!(index3.is_some());
    
    // Verify all are traced
    let module1_atom = tracer.string_to_atom("module1");
    let module2_atom = tracer.string_to_atom("module2");
    let func1_atom = tracer.string_to_atom("func1");
    let func2_atom = tracer.string_to_atom("func2");
    
    assert_eq!(tracer.is_traced_mfa(module1_atom, Some(func1_atom), 1), index1.unwrap());
    assert_eq!(tracer.is_traced_mfa(module1_atom, Some(func2_atom), 2), index2.unwrap());
    assert_eq!(tracer.is_traced_mfa(module2_atom, Some(func1_atom), 1), index3.unwrap());
    
    // Test module-level tracing (any function in module)
    // When function is None, it returns the first matching trace index (could be any of them)
    let module_trace = tracer.is_traced_mfa(module1_atom, None, 0);
    assert!(module_trace > 0); // Should match at least one function
}

#[test]
fn test_code_loader_with_module_management_integration() {
    // Test code loader working with module management
    let module_table = ModuleTableManager::new();
    
    module_table.init();
    
    // Create test code
    let test_code = b"test module code for integration test";
    
    // Verify code
    assert!(code_management_code_loading::code_loader::CodeLoader::verify_module(test_code));
    
    // In a full implementation, would:
    // 1. Load code using code_loader
    // 2. Create module instance using module_table
    // 3. Register module in module table
    // 4. Verify module can be looked up
}

#[test]
fn test_global_singletons_integration() {
    // Test that global singletons work correctly
    let perm_manager1 = get_global_code_permissions();
    let perm_manager2 = get_global_code_permissions();
    
    // Should be the same instance
    assert!(std::ptr::eq(perm_manager1, perm_manager2));
    
    let barrier_manager1 = get_global_code_barriers();
    let barrier_manager2 = get_global_code_barriers();
    
    // Should be the same instance
    assert!(std::ptr::eq(barrier_manager1, barrier_manager2));
    
    let tracer1 = get_global_debug_tracer();
    let tracer2 = get_global_debug_tracer();
    
    // Should be the same instance
    assert!(std::ptr::eq(tracer1, tracer2));
    
    // Test that they can be used together
    let process_id: ProcessId = 1;
    perm_manager1.init();
    assert!(perm_manager1.try_seize_code_mod_permission(process_id));
    
    barrier_manager1.init();
    barrier_manager1.blocking_code_barrier();
    
    let index = tracer1.set_traced_mfa("test", "func", 1);
    assert!(index.is_some());
}

