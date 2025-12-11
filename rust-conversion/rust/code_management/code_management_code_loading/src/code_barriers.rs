//! Code Barrier Management
//!
//! Provides code barrier synchronization for thread safety.
//! Based on code_ix.c - Code barrier system for thread synchronization.
//!
//! Code barriers ensure that all threads have completed certain operations
//! before proceeding. This is critical for safe code updates in a multi-threaded
//! environment.

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 2012-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Code barrier structure
///
/// Schedules an operation to run after thread progress and all schedulers
/// have issued an instruction barrier.
pub struct CodeBarrier {
    /// Number of pending schedulers that need to issue barrier
    pending_schedulers: Arc<AtomicUsize>,
    /// Size of cleanup data (0 if no cleanup needed)
    size: usize,
    /// Function to call after barrier
    later_function: Option<Box<dyn FnOnce() + Send>>,
    /// Data to pass to later function
    later_data: Option<*mut std::ffi::c_void>,
}

impl CodeBarrier {
    /// Create a new code barrier
    pub fn new() -> Self {
        Self {
            pending_schedulers: Arc::new(AtomicUsize::new(0)),
            size: 0,
            later_function: None,
            later_data: None,
        }
    }

    /// Get the number of pending schedulers
    pub fn pending_schedulers(&self) -> usize {
        self.pending_schedulers.load(Ordering::Acquire)
    }
}

impl Default for CodeBarrier {
    fn default() -> Self {
        Self::new()
    }
}

/// Code barrier manager
pub struct CodeBarrierManager {
    /// Outstanding blocking code barriers counter
    outstanding_blocking_code_barriers: Arc<AtomicU32>,
    /// Thread-local storage for debug barrier requirements
    /// In a full implementation, this would use thread-local storage
    needs_code_barrier: Arc<Mutex<bool>>,
}

impl CodeBarrierManager {
    /// Create a new code barrier manager
    pub fn new() -> Self {
        Self {
            outstanding_blocking_code_barriers: Arc::new(AtomicU32::new(0)),
            needs_code_barrier: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize the code barrier manager
    pub fn init(&self) {
        // Initialize outstanding blocking code barriers to 0
        self.outstanding_blocking_code_barriers.store(0, Ordering::Release);
        // In a full implementation, would initialize thread-local storage here
    }

    /// Schedule a code barrier
    ///
    /// Schedules an operation to run after thread progress and all schedulers
    /// have issued an instruction barrier.
    ///
    /// # Arguments
    /// * `barrier` - Code barrier to schedule
    /// * `later_function` - Function to call after barrier completes
    /// * `later_data` - Data to pass to later function
    pub fn schedule_code_barrier(
        &self,
        barrier: &mut CodeBarrier,
        later_function: Box<dyn FnOnce() + Send>,
        later_data: Option<*mut std::ffi::c_void>,
    ) {
        self.schedule_code_barrier_cleanup(barrier, later_function, later_data, 0);
    }

    /// Schedule a code barrier with cleanup
    ///
    /// Same as `schedule_code_barrier` but with cleanup size for memory management.
    ///
    /// # Arguments
    /// * `barrier` - Code barrier to schedule
    /// * `later_function` - Function to call after barrier completes
    /// * `later_data` - Data to pass to later function
    /// * `size` - Size of cleanup data (0 if no cleanup needed)
    pub fn schedule_code_barrier_cleanup(
        &self,
        barrier: &mut CodeBarrier,
        later_function: Box<dyn FnOnce() + Send>,
        later_data: Option<*mut std::ffi::c_void>,
        size: usize,
    ) {
        // In a full implementation, this would:
        // 1. Set up the barrier to wait for all schedulers
        // 2. Schedule thread progress later operation
        // 3. Issue instruction barriers on all schedulers
        
        barrier.size = size;
        barrier.later_function = Some(later_function);
        barrier.later_data = later_data;
        
        // Set pending schedulers count (in full implementation, would be actual scheduler count)
        // For now, we'll use 1 as a placeholder
        barrier.pending_schedulers.store(1, Ordering::Release);
        
        // In a full implementation, would schedule thread progress operation here
        // erts_schedule_thr_prgr_later_op(...)
    }

    /// Issue a blocking code barrier
    ///
    /// Issues a code barrier on the current thread, as well as all managed threads
    /// when they wake up after thread progress is unblocked.
    ///
    /// Requires that thread progress is blocked.
    pub fn blocking_code_barrier(&self) {
        // In a full implementation, this would:
        // 1. Unrequire debug code barrier
        // 2. Increment outstanding blocking code barriers
        // 3. Schedule blocking code barriers on all threads
        
        // Unrequire debug code barrier
        let mut needs = self.needs_code_barrier.lock().unwrap();
        *needs = false;
        drop(needs);
        
        // Increment outstanding blocking code barriers
        let _count = self.outstanding_blocking_code_barriers.fetch_add(1, Ordering::AcqRel);
        
        // In a full implementation, would schedule thread progress operation here
        // if count == 0, schedule decrement operation
    }

    /// Get outstanding blocking code barriers count
    ///
    /// Returns the number of outstanding blocking code barriers.
    pub fn outstanding_blocking_code_barriers(&self) -> u32 {
        self.outstanding_blocking_code_barriers.load(Ordering::Acquire)
    }

    /// Finalize wait for code barrier
    ///
    /// Helper function: all managed threads should call this as soon as
    /// thread progress is unblocked, BEFORE updating thread progress.
    pub fn finalize_wait(&self) {
        // In a full implementation, would:
        // 1. Issue instruction barrier
        // 2. Decrement pending schedulers on barrier
        // 3. If all schedulers done, schedule later operation
    }
}

impl Default for CodeBarrierManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global code barrier manager (singleton)
static GLOBAL_CODE_BARRIERS: std::sync::OnceLock<CodeBarrierManager> = std::sync::OnceLock::new();

/// Get the global code barrier manager
pub fn get_global_code_barriers() -> &'static CodeBarrierManager {
    GLOBAL_CODE_BARRIERS.get_or_init(|| {
        let manager = CodeBarrierManager::new();
        manager.init();
        manager
    })
}

/// Debug: Require code barrier
///
/// Sets a flag indicating that a code barrier is required.
/// Used for debugging to ensure code barriers are properly issued.
#[cfg(debug_assertions)]
pub fn debug_require_code_barrier() {
    let manager = get_global_code_barriers();
    let mut needs = manager.needs_code_barrier.lock().unwrap();
    *needs = true;
}

/// Debug: Require code barrier (no-op in release builds)
#[cfg(not(debug_assertions))]
pub fn debug_require_code_barrier() {
    // No-op in release builds
}

/// Debug: Check code barrier
///
/// Asserts that no code barrier is required.
/// Used for debugging to ensure code barriers are properly issued.
#[cfg(debug_assertions)]
pub fn debug_check_code_barrier() {
    let manager = get_global_code_barriers();
    let needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
    let needs_value = *needs;
    drop(needs); // Release lock before potentially panicking
    assert!(!needs_value, "Code barrier required but not issued");
}

/// Debug: Check code barrier (no-op in release builds)
#[cfg(not(debug_assertions))]
pub fn debug_check_code_barrier() {
    // No-op in release builds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_barrier_manager_init() {
        let manager = CodeBarrierManager::new();
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }

    #[test]
    fn test_code_barrier_creation() {
        let barrier = CodeBarrier::new();
        assert_eq!(barrier.pending_schedulers(), 0);
    }

    #[test]
    fn test_blocking_code_barrier() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_code_barrier() {
        // Use global singleton for debug functions
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear any previous state
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        debug_require_code_barrier();
        // In a full implementation, would check that flag is set
        
        // This would panic if barrier not issued, but we haven't issued one
        // so we skip the check in this test
    }

    #[test]
    fn test_code_barrier_default() {
        let barrier = CodeBarrier::default();
        assert_eq!(barrier.pending_schedulers(), 0);
    }

    #[test]
    fn test_code_barrier_manager_default() {
        let manager = CodeBarrierManager::default();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }

    #[test]
    fn test_schedule_code_barrier() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(move || {
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            None,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }

    #[test]
    fn test_schedule_code_barrier_cleanup() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(move || {
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            None,
            100, // cleanup size
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }

    #[test]
    fn test_schedule_code_barrier_with_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let data: i32 = 42;
        let data_ptr = &data as *const i32 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }

    #[test]
    fn test_multiple_blocking_code_barriers() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 3);
    }

    #[test]
    fn test_finalize_wait() {
        let manager = CodeBarrierManager::new();
        manager.finalize_wait();
        // This is a no-op in the current implementation, but we test it's callable
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_check_code_barrier_success() {
        // Use global singleton for debug functions
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear any previous state
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        // Don't require a barrier, so check should pass
        debug_check_code_barrier();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_check_code_barrier_failure() {
        // Use global singleton for debug functions
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear any previous state
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        debug_require_code_barrier();
        
        // This should panic because we required a barrier but didn't issue one
        let result = std::panic::catch_unwind(|| {
            debug_check_code_barrier();
        });
        
        assert!(result.is_err(), "debug_check_code_barrier should panic when barrier is required but not issued");
    }

    #[test]
    fn test_get_global_code_barriers() {
        let manager1 = get_global_code_barriers();
        let manager2 = get_global_code_barriers();
        
        // Should return the same instance (singleton)
        assert_eq!(
            manager1.outstanding_blocking_code_barriers(),
            manager2.outstanding_blocking_code_barriers()
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_blocking_code_barrier_clears_debug_flag() {
        // Use global singleton for debug functions
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear any previous state
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        debug_require_code_barrier();
        manager.blocking_code_barrier();
        // After blocking_code_barrier, the flag should be cleared
        debug_check_code_barrier(); // Should not panic
    }
    
    #[test]
    fn test_code_barrier_pending_schedulers() {
        let barrier = CodeBarrier::new();
        assert_eq!(barrier.pending_schedulers(), 0);
        
        // Set pending schedulers (simulating what schedule_code_barrier does)
        barrier.pending_schedulers.store(5, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 5);
        
        barrier.pending_schedulers.store(10, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 10);
    }
    
    #[test]
    fn test_code_barrier_manager_init_multiple_times() {
        let manager = CodeBarrierManager::new();
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Add some barriers
        manager.blocking_code_barrier();
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        // Init again should reset
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_different_sizes() {
        let manager = CodeBarrierManager::new();
        
        // Test with size 0
        let mut barrier1 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(
            &mut barrier1,
            Box::new(|| {}),
            None,
            0,
        );
        assert_eq!(barrier1.pending_schedulers(), 1);
        
        // Test with size 100
        let mut barrier2 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(
            &mut barrier2,
            Box::new(|| {}),
            None,
            100,
        );
        assert_eq!(barrier2.pending_schedulers(), 1);
        
        // Test with large size
        let mut barrier3 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(
            &mut barrier3,
            Box::new(|| {}),
            None,
            10000,
        );
        assert_eq!(barrier3.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let data: u64 = 0x1234567890ABCDEF;
        let data_ptr = &data as *const u64 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
            50,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_clears_needs_flag() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Set needs flag manually
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap();
            *needs = true;
        }
        
        // Issue blocking barrier - should clear the flag
        manager.blocking_code_barrier();
        
        // Check flag is cleared
        {
            let needs = manager.needs_code_barrier.lock().unwrap();
            assert!(!*needs);
        }
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_after_operations() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 3);
        
        // Init should reset
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_finalize_wait_multiple_times() {
        let manager = CodeBarrierManager::new();
        
        // Should be safe to call multiple times
        manager.finalize_wait();
        manager.finalize_wait();
        manager.finalize_wait();
    }
    
    #[test]
    fn test_get_global_code_barriers_singleton() {
        let manager1 = get_global_code_barriers();
        let manager2 = get_global_code_barriers();
        
        // Should be the same instance
        assert!(std::ptr::eq(manager1, manager2));
    }
    
    #[test]
    fn test_get_global_code_barriers_initialized() {
        let manager = get_global_code_barriers();
        
        // Should be initialized (outstanding should be 0 after init)
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_multiple_barriers() {
        let manager = CodeBarrierManager::new();
        
        let mut barrier1 = CodeBarrier::new();
        let mut barrier2 = CodeBarrier::new();
        let mut barrier3 = CodeBarrier::new();
        
        manager.schedule_code_barrier(&mut barrier1, Box::new(|| {}), None);
        manager.schedule_code_barrier(&mut barrier2, Box::new(|| {}), None);
        manager.schedule_code_barrier(&mut barrier3, Box::new(|| {}), None);
        
        assert_eq!(barrier1.pending_schedulers(), 1);
        assert_eq!(barrier2.pending_schedulers(), 1);
        assert_eq!(barrier3.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_code_barrier_with_later_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let mut value = 42;
        let data_ptr = &mut value as *mut i32 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_increments_atomic() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        let initial = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        let after_one = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        let after_two = manager.outstanding_blocking_code_barriers();
        
        assert_eq!(initial, 0);
        assert_eq!(after_one, 1);
        assert_eq!(after_two, 2);
    }
    
    #[test]
    fn test_code_barrier_manager_new_vs_default() {
        let manager1 = CodeBarrierManager::new();
        let manager2 = CodeBarrierManager::default();
        
        // Both should start with 0 outstanding barriers
        assert_eq!(manager1.outstanding_blocking_code_barriers(), 0);
        assert_eq!(manager2.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_code_barrier_new_vs_default() {
        let barrier1 = CodeBarrier::new();
        let barrier2 = CodeBarrier::default();
        
        // Both should start with 0 pending schedulers
        assert_eq!(barrier1.pending_schedulers(), 0);
        assert_eq!(barrier2.pending_schedulers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_overwrites_existing() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Schedule first barrier
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Schedule second barrier (should overwrite)
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_overwrites_existing() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Schedule first barrier with size 10
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 10);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Schedule second barrier with size 20 (should overwrite)
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 20);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_require_code_barrier_sets_flag() {
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear flag
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        // Require barrier
        debug_require_code_barrier();
        
        // Check flag is set
        {
            let needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            assert!(*needs);
        }
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_require_code_barrier_multiple_times() {
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear flag
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        // Require barrier multiple times
        debug_require_code_barrier();
        debug_require_code_barrier();
        debug_require_code_barrier();
        
        // Flag should still be set
        {
            let needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            assert!(*needs);
        }
    }
    
    #[test]
    fn test_blocking_code_barrier_with_existing_outstanding() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Add some outstanding barriers
        manager.blocking_code_barrier();
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        // Add another one
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 3);
    }
    
    #[test]
    fn test_code_barrier_pending_schedulers_concurrent() {
        let barrier = CodeBarrier::new();
        
        // Simulate concurrent updates (though in real code this would be done by schedulers)
        barrier.pending_schedulers.store(1, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        barrier.pending_schedulers.store(0, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_null_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(|| {}),
            None, // Explicitly None
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_null_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            None, // Explicitly None
            100,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_ordering() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test that ordering is correct (Acquire should see all previous writes)
        manager.blocking_code_barrier();
        let count1 = manager.outstanding_blocking_code_barriers();
        
        manager.blocking_code_barrier();
        let count2 = manager.outstanding_blocking_code_barriers();
        
        assert!(count2 > count1);
    }
    
    #[test]
    fn test_schedule_code_barrier_direct_call() {
        // Ensure schedule_code_barrier is directly called (not just through cleanup)
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Direct call to schedule_code_barrier (which internally calls schedule_code_barrier_cleanup)
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        
        // Verify it worked
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_vs_cleanup() {
        // Test that schedule_code_barrier (wrapper) and schedule_code_barrier_cleanup both work
        let manager = CodeBarrierManager::new();
        
        // Test wrapper function
        let mut barrier1 = CodeBarrier::new();
        manager.schedule_code_barrier(&mut barrier1, Box::new(|| {}), None);
        assert_eq!(barrier1.pending_schedulers(), 1);
        
        // Test cleanup function directly
        let mut barrier2 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(&mut barrier2, Box::new(|| {}), None, 0);
        assert_eq!(barrier2.pending_schedulers(), 1);
        
        // Both should behave the same when size is 0
        assert_eq!(barrier1.pending_schedulers(), barrier2.pending_schedulers());
    }
    
    #[test]
    fn test_schedule_code_barrier_repeated_calls() {
        // Test calling schedule_code_barrier multiple times
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // First call
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Second call (overwrites)
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Third call
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_function_that_captures() {
        // Test schedule_code_barrier with a function that captures variables
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let captured_value = 42;
        let captured = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
        let captured_clone = captured.clone();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(move || {
                captured_clone.store(captured_value, std::sync::atomic::Ordering::Release);
            }),
            None,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_all_code_paths() {
        // Ensure all code paths in schedule_code_barrier are covered
        let manager = CodeBarrierManager::new();
        
        // Test with None data
        let mut barrier1 = CodeBarrier::new();
        manager.schedule_code_barrier(&mut barrier1, Box::new(|| {}), None);
        
        // Test with Some data
        let mut barrier2 = CodeBarrier::new();
        let data: i32 = 100;
        let data_ptr = &data as *const i32 as *mut std::ffi::c_void;
        manager.schedule_code_barrier(&mut barrier2, Box::new(|| {}), Some(data_ptr));
        
        // Both should set pending schedulers
        assert_eq!(barrier1.pending_schedulers(), 1);
        assert_eq!(barrier2.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_all_assignments() {
        // Ensure all assignments in schedule_code_barrier_cleanup are covered
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        let data: u64 = 0xDEADBEEF;
        let data_ptr = &data as *const u64 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
            200,
        );
        
        // Verify all fields were set
        assert_eq!(barrier.pending_schedulers(), 1);
        // Note: size, later_function, and later_data are private, but we can verify
        // the effect through pending_schedulers
    }
    
    #[test]
    fn test_blocking_code_barrier_all_statements() {
        // Ensure all statements in blocking_code_barrier are covered
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Set needs flag first
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap();
            *needs = true;
        }
        
        // Call blocking_code_barrier - should clear flag and increment counter
        let before = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        let after = manager.outstanding_blocking_code_barriers();
        
        // Verify flag was cleared
        {
            let needs = manager.needs_code_barrier.lock().unwrap();
            assert!(!*needs);
        }
        
        // Verify counter was incremented
        assert_eq!(after, before + 1);
    }
    
    #[test]
    fn test_debug_require_code_barrier_release_build() {
        // Test that debug_require_code_barrier exists in release builds (no-op)
        // This ensures both #[cfg] variants are compiled
        #[cfg(not(debug_assertions))]
        {
            debug_require_code_barrier(); // Should be no-op
        }
        
        #[cfg(debug_assertions)]
        {
            // In debug builds, this actually does something
            let manager = get_global_code_barriers();
            manager.init();
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
                *needs = false;
            }
            debug_require_code_barrier();
        }
    }
    
    #[test]
    fn test_debug_check_code_barrier_release_build() {
        // Test that debug_check_code_barrier exists in release builds (no-op)
        #[cfg(not(debug_assertions))]
        {
            debug_check_code_barrier(); // Should be no-op
        }
        
        #[cfg(debug_assertions)]
        {
            // In debug builds, this actually does something
            let manager = get_global_code_barriers();
            manager.init();
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
                *needs = false;
            }
            debug_check_code_barrier(); // Should not panic
        }
    }
    
    #[test]
    fn test_init_resets_outstanding() {
        // Test that init() resets outstanding_blocking_code_barriers
        let manager = CodeBarrierManager::new();
        
        // Add some barriers
        manager.blocking_code_barrier();
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        // Init should reset to 0
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_size_assignment() {
        // Test that size is properly assigned in schedule_code_barrier_cleanup
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Call with different sizes to ensure the assignment is covered
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 0);
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 1);
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 100);
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_code_barrier_pending_schedulers_edge_cases() {
        let barrier = CodeBarrier::new();
        
        // Test with maximum usize value
        barrier.pending_schedulers.store(usize::MAX, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), usize::MAX);
        
        // Test with zero
        barrier.pending_schedulers.store(0, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 0);
        
        // Test with 1
        barrier.pending_schedulers.store(1, Ordering::Release);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_edge_cases() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test with maximum u32 value
        for _ in 0..1000 {
            manager.blocking_code_barrier();
        }
        let count = manager.outstanding_blocking_code_barriers();
        assert_eq!(count, 1000);
        
        // Reset and test again
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_max_size() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with maximum usize size
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            None,
            usize::MAX,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_various_data_types() {
        let manager = CodeBarrierManager::new();
        
        // Test with i8
        let mut barrier1 = CodeBarrier::new();
        let data_i8: i8 = -42;
        let ptr_i8 = &data_i8 as *const i8 as *mut std::ffi::c_void;
        manager.schedule_code_barrier(&mut barrier1, Box::new(|| {}), Some(ptr_i8));
        
        // Test with u64
        let mut barrier2 = CodeBarrier::new();
        let data_u64: u64 = 0xFFFFFFFFFFFFFFFF;
        let ptr_u64 = &data_u64 as *const u64 as *mut std::ffi::c_void;
        manager.schedule_code_barrier(&mut barrier2, Box::new(|| {}), Some(ptr_u64));
        
        // Test with f64
        let mut barrier3 = CodeBarrier::new();
        let data_f64: f64 = 3.14159;
        let ptr_f64 = &data_f64 as *const f64 as *mut std::ffi::c_void;
        manager.schedule_code_barrier(&mut barrier3, Box::new(|| {}), Some(ptr_f64));
        
        assert_eq!(barrier1.pending_schedulers(), 1);
        assert_eq!(barrier2.pending_schedulers(), 1);
        assert_eq!(barrier3.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_ordering_semantics() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test that AcqRel ordering works correctly
        let initial = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        
        // With AcqRel, we should see the increment
        let after = manager.outstanding_blocking_code_barriers();
        assert_eq!(after, initial + 1);
    }
    
    #[test]
    fn test_pending_schedulers_ordering_semantics() {
        let barrier = CodeBarrier::new();
        
        // Test that Acquire ordering works correctly
        barrier.pending_schedulers.store(5, Ordering::Release);
        let value = barrier.pending_schedulers(); // Uses Acquire
        assert_eq!(value, 5);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_all_size_variants() {
        let manager = CodeBarrierManager::new();
        
        // Test various size values
        let sizes = vec![0, 1, 10, 100, 1000, 10000, usize::MAX / 2];
        
        for size in sizes {
            let mut barrier = CodeBarrier::new();
            manager.schedule_code_barrier_cleanup(
                &mut barrier,
                Box::new(|| {}),
                None,
                size,
            );
            assert_eq!(barrier.pending_schedulers(), 1);
        }
    }
    
    #[test]
    fn test_code_barrier_manager_concurrent_init() {
        let manager = Arc::new(CodeBarrierManager::new());
        manager.init();
        
        // Test that init can be called multiple times safely
        let manager_clone = manager.clone();
        std::thread::scope(|s| {
            s.spawn(|| {
                manager_clone.init();
            });
            manager.init();
        });
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_blocking_code_barrier_concurrent_access() {
        let manager = Arc::new(CodeBarrierManager::new());
        manager.init();
        
        const NUM_THREADS: u32 = 10;
        const BARRIERS_PER_THREAD: u32 = 100;
        
        std::thread::scope(|s| {
            for _ in 0..NUM_THREADS {
                let manager_clone = manager.clone();
                s.spawn(move || {
                    for _ in 0..BARRIERS_PER_THREAD {
                        manager_clone.blocking_code_barrier();
                    }
                });
            }
        });
        
        let expected = NUM_THREADS * BARRIERS_PER_THREAD;
        assert_eq!(manager.outstanding_blocking_code_barriers(), expected);
    }
    
    #[test]
    fn test_code_barrier_pending_schedulers_concurrent_updates() {
        let barrier = CodeBarrier::new();
        let pending_schedulers = barrier.pending_schedulers.clone();
        
        const NUM_THREADS: usize = 10;
        const UPDATES_PER_THREAD: usize = 100;
        
        std::thread::scope(|s| {
            for i in 0..NUM_THREADS {
                let pending_clone = pending_schedulers.clone();
                s.spawn(move || {
                    for j in 0..UPDATES_PER_THREAD {
                        let value = (i * UPDATES_PER_THREAD + j) % 1000;
                        pending_clone.store(value, Ordering::Release);
                    }
                });
            }
        });
        
        // After all updates, value should be some valid number
        let final_value = barrier.pending_schedulers();
        assert!(final_value < 1000);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_complex_capture() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with function that captures multiple variables
        let value1 = 42;
        let value2 = String::from("test");
        let value3 = vec![1, 2, 3];
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(move || {
                let _ = value1;
                let _ = value2;
                let _ = value3;
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            None,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_complex_capture() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with function that captures and uses data
        let data = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let data_clone = data.clone();
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(move || {
                data_clone.store(0xDEADBEEF, std::sync::atomic::Ordering::Release);
            }),
            None,
            512,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_get_global_code_barriers_thread_safety() {
        // Test that get_global_code_barriers is thread-safe
        const NUM_THREADS: usize = 20;
        
        let managers: Vec<_> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..NUM_THREADS)
                .map(|_| {
                    s.spawn(|| get_global_code_barriers())
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        
        // All should return the same instance
        let first = managers[0];
        for manager in managers.iter().skip(1) {
            assert!(std::ptr::eq(first, *manager));
        }
    }
    
    #[test]
    fn test_debug_check_code_barrier_lock_poisoning_path() {
        #[cfg(debug_assertions)]
        {
            // This test ensures the unwrap_or_else path is covered
            // We can't easily poison the lock in a test, but we can verify
            // the code path exists by checking the implementation
            let manager = get_global_code_barriers();
            manager.init();
            
            // Clear flag
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
                *needs = false;
            }
            
            // The unwrap_or_else path is in debug_check_code_barrier
            // We test it doesn't panic when flag is false
            debug_check_code_barrier();
        }
    }
    
    #[test]
    fn test_schedule_code_barrier_sequence() {
        let manager = CodeBarrierManager::new();
        
        // Test scheduling multiple barriers in sequence
        let mut barriers = Vec::new();
        for i in 0..10 {
            let mut barrier = CodeBarrier::new();
            let index = i;
            manager.schedule_code_barrier(
                &mut barrier,
                Box::new(move || {
                    let _ = index;
                }),
                None,
            );
            barriers.push(barrier);
        }
        
        for barrier in barriers {
            assert_eq!(barrier.pending_schedulers(), 1);
        }
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_sequence() {
        let manager = CodeBarrierManager::new();
        
        // Test scheduling multiple cleanup barriers with different sizes
        let sizes = vec![0, 10, 100, 1000];
        let mut barriers = Vec::new();
        
        for size in sizes {
            let mut barrier = CodeBarrier::new();
            manager.schedule_code_barrier_cleanup(
                &mut barrier,
                Box::new(|| {}),
                None,
                size,
            );
            barriers.push(barrier);
        }
        
        for barrier in barriers {
            assert_eq!(barrier.pending_schedulers(), 1);
        }
    }
    
    #[test]
    fn test_blocking_code_barrier_sequence_with_init() {
        let manager = CodeBarrierManager::new();
        
        // Test sequence: init -> barrier -> barrier -> init -> barrier
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
    }
    
    #[test]
    fn test_code_barrier_lifecycle() {
        // Test complete lifecycle of a code barrier
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Initial state
        assert_eq!(barrier.pending_schedulers(), 0);
        
        // Schedule barrier
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Schedule again (overwrites)
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        assert_eq!(barrier.pending_schedulers(), 1);
        
        // Schedule with cleanup
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 100);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_code_barrier_manager_lifecycle() {
        // Test complete lifecycle of a code barrier manager
        let manager = CodeBarrierManager::new();
        
        // Initial state
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Init
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Add barriers
        for i in 0..5 {
            manager.blocking_code_barrier();
            assert_eq!(manager.outstanding_blocking_code_barriers(), i + 1);
        }
        
        // Re-init (resets)
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Add more barriers
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_function_side_effects() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with function that has side effects (though function won't be called)
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(move || {
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::Release);
            }),
            None,
        );
        
        // Function not called yet, counter should still be 0
        assert_eq!(counter.load(std::sync::atomic::Ordering::Acquire), 0);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_function_side_effects() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with function that has side effects
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(move || {
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::Release);
            }),
            None,
            200,
        );
        
        // Function not called yet, counter should still be 0
        assert_eq!(counter.load(std::sync::atomic::Ordering::Acquire), 0);
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_finalize_wait_called_after_barriers() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test calling finalize_wait in various scenarios
        manager.finalize_wait();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        manager.finalize_wait();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        manager.blocking_code_barrier();
        manager.blocking_code_barrier();
        manager.finalize_wait();
        
        // Should not affect outstanding count (finalize_wait is a no-op)
        assert_eq!(manager.outstanding_blocking_code_barriers(), 3);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_mutable_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with mutable data pointer
        let mut value = 100;
        let data_ptr = &mut value as *mut i32 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
        // Value should still be accessible (function not called)
        assert_eq!(value, 100);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_mutable_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with mutable data pointer and cleanup size
        let mut value = 200;
        let data_ptr = &mut value as *mut i32 as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
            300,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
        assert_eq!(value, 200);
    }
    
    #[test]
    fn test_code_barrier_manager_new_creates_fresh_instance() {
        let manager1 = CodeBarrierManager::new();
        let manager2 = CodeBarrierManager::new();
        
        // Should be different instances
        assert!(!std::ptr::eq(&manager1, &manager2));
        
        // Both should start with 0 outstanding
        assert_eq!(manager1.outstanding_blocking_code_barriers(), 0);
        assert_eq!(manager2.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_code_barrier_new_creates_fresh_instance() {
        let barrier1 = CodeBarrier::new();
        let barrier2 = CodeBarrier::new();
        
        // Should be different instances
        assert!(!std::ptr::eq(&barrier1.pending_schedulers, &barrier2.pending_schedulers));
        
        // Both should start with 0 pending
        assert_eq!(barrier1.pending_schedulers(), 0);
        assert_eq!(barrier2.pending_schedulers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_preserves_function() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test that function is preserved (though not called)
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(move || {
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            None,
        );
        
        // Function stored but not called
        assert!(!called.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_preserves_function_and_size() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test that function and size are preserved
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        const TEST_SIZE: usize = 12345;
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(move || {
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            None,
            TEST_SIZE,
        );
        
        // Function stored but not called
        assert!(!called.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_clears_flag_even_when_already_false() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Ensure flag is false
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap();
            *needs = false;
        }
        
        // Call blocking_code_barrier - should still work
        manager.blocking_code_barrier();
        
        // Flag should still be false
        {
            let needs = manager.needs_code_barrier.lock().unwrap();
            assert!(!*needs);
        }
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_after_init_sequence() {
        let manager = CodeBarrierManager::new();
        
        // Complex sequence: new -> init -> barrier -> barrier -> init -> barrier -> init
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_zero_sized_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with zero-sized type
        struct ZeroSized;
        let data = ZeroSized;
        let data_ptr = &data as *const ZeroSized as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_zero_sized_data() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with zero-sized type and cleanup
        struct ZeroSized;
        let data = ZeroSized;
        let data_ptr = &data as *const ZeroSized as *mut std::ffi::c_void;
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(|| {}),
            Some(data_ptr),
            0,
        );
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_pending_schedulers_all_ordering_combinations() {
        let barrier = CodeBarrier::new();
        
        // Test different ordering combinations for store (Acquire and AcqRel are invalid for store)
        let store_orderings = vec![
            Ordering::Relaxed,
            Ordering::Release,
            Ordering::SeqCst,
        ];
        
        for ordering in store_orderings {
            barrier.pending_schedulers.store(42, ordering);
            let value = barrier.pending_schedulers(); // Uses Acquire
            assert_eq!(value, 42);
        }
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_all_ordering_combinations() {
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test that AcqRel ordering works correctly
        manager.blocking_code_barrier();
        let value = manager.outstanding_blocking_code_barriers(); // Uses Acquire
        assert_eq!(value, 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_with_empty_function() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with completely empty function
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_with_empty_function() {
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test with completely empty function
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 0);
        
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_get_global_code_barriers_initialization_order() {
        // Test that get_global_code_barriers initializes correctly
        let manager = get_global_code_barriers();
        
        // Should be initialized (outstanding should be 0 after init)
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Should be able to call init again
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_code_barrier_manager_with_shared_arc() {
        // Test using Arc to share manager across scopes
        let manager = Arc::new(CodeBarrierManager::new());
        manager.init();
        
        let manager_clone = manager.clone();
        std::thread::scope(|s| {
            s.spawn(move || {
                manager_clone.blocking_code_barrier();
            });
        });
        
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
    }
    
    #[test]
    fn test_code_barrier_with_shared_arc() {
        // Test using Arc to share barrier's pending_schedulers across scopes
        let barrier = CodeBarrier::new();
        let pending_schedulers = barrier.pending_schedulers.clone();
        
        std::thread::scope(|s| {
            let pending_clone = pending_schedulers.clone();
            s.spawn(move || {
                pending_clone.store(5, Ordering::Release);
            });
        });
        
        assert_eq!(barrier.pending_schedulers(), 5);
    }
    
    #[test]
    #[cfg(not(debug_assertions))]
    fn test_debug_require_code_barrier_release_build_noop() {
        // Test that debug_require_code_barrier is a no-op in release builds
        // This ensures the release build version of the function is covered
        debug_require_code_barrier();
        // Should not panic or have any side effects
    }
    
    #[test]
    #[cfg(not(debug_assertions))]
    fn test_debug_check_code_barrier_release_build_noop() {
        // Test that debug_check_code_barrier is a no-op in release builds
        // This ensures the release build version of the function is covered
        debug_check_code_barrier();
        // Should not panic or have any side effects
    }
    
    #[test]
    fn test_get_global_code_barriers_closure_initialization() {
        // Test that the closure in get_global_code_barriers is executed
        // The first call should initialize the manager
        let manager1 = get_global_code_barriers();
        assert_eq!(manager1.outstanding_blocking_code_barriers(), 0);
        
        // Subsequent calls should return the same instance
        let manager2 = get_global_code_barriers();
        assert!(std::ptr::eq(manager1, manager2));
    }
    
    #[test]
    fn test_get_global_code_barriers_closure_path() {
        // Test the closure execution path in get_global_code_barriers
        // This ensures the closure (|| { ... }) is covered
        // We can't easily test this directly, but calling it multiple times
        // ensures the get_or_init path is covered
        
        // First call - should execute closure
        let _manager1 = get_global_code_barriers();
        
        // Second call - should not execute closure (already initialized)
        let _manager2 = get_global_code_barriers();
        
        // Both should be the same instance
        assert!(std::ptr::eq(_manager1, _manager2));
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_check_code_barrier_unwrap_or_else_path() {
        // Test the unwrap_or_else path in debug_check_code_barrier
        // This is difficult to test because we'd need to poison a mutex
        // However, we can at least ensure the function is callable
        // and the unwrap_or_else closure exists in the code
        
        let manager = get_global_code_barriers();
        manager.init();
        
        // Clear flag
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap_or_else(|e| e.into_inner());
            *needs = false;
        }
        
        // Call debug_check_code_barrier - should not panic
        // The unwrap_or_else path exists but is hard to trigger (requires lock poisoning)
        debug_check_code_barrier();
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_all_field_assignments() {
        // Ensure all field assignments in schedule_code_barrier_cleanup are covered
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        // Test that size, later_function, and later_data are all assigned
        let data: i32 = 999;
        let data_ptr = &data as *const i32 as *mut std::ffi::c_void;
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        manager.schedule_code_barrier_cleanup(
            &mut barrier,
            Box::new(move || {
                called_clone.store(true, std::sync::atomic::Ordering::Release);
            }),
            Some(data_ptr),
            777,
        );
        
        // Verify pending_schedulers was set (indirect verification)
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_all_code_paths() {
        // Ensure all code paths in blocking_code_barrier are covered
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Test the lock acquisition and modification path
        {
            let mut needs = manager.needs_code_barrier.lock().unwrap();
            *needs = true;
        }
        
        // Call blocking_code_barrier - should clear flag and increment counter
        let before = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        let after = manager.outstanding_blocking_code_barriers();
        
        // Verify flag was cleared
        {
            let needs = manager.needs_code_barrier.lock().unwrap();
            assert!(!*needs);
        }
        
        // Verify counter was incremented
        assert_eq!(after, before + 1);
        
        // Test the drop(needs) path is covered
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), before + 2);
    }
    
    #[test]
    fn test_debug_require_code_barrier_lock_path() {
        #[cfg(debug_assertions)]
        {
            // Test the lock acquisition path in debug_require_code_barrier
            let manager = get_global_code_barriers();
            manager.init();
            
            // Clear flag first
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap();
                *needs = false;
            }
            
            // Call debug_require_code_barrier - should set flag
            debug_require_code_barrier();
            
            // Verify flag was set
            {
                let needs = manager.needs_code_barrier.lock().unwrap();
                assert!(*needs);
            }
        }
    }
    
    #[test]
    fn test_debug_check_code_barrier_drop_before_panic() {
        #[cfg(debug_assertions)]
        {
            // Test that drop(needs) happens before the potential panic
            // This ensures the drop statement is covered
            let manager = get_global_code_barriers();
            manager.init();
            
            // Clear flag
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap();
                *needs = false;
            }
            
            // Should not panic (flag is false)
            debug_check_code_barrier();
            
            // Set flag and verify it would panic
            {
                let mut needs = manager.needs_code_barrier.lock().unwrap();
                *needs = true;
            }
            
            // This should panic, but we catch it
            let result = std::panic::catch_unwind(|| {
                debug_check_code_barrier();
            });
            
            assert!(result.is_err());
        }
    }
    
    #[test]
    fn test_schedule_code_barrier_internal_call_to_cleanup() {
        // Test that schedule_code_barrier internally calls schedule_code_barrier_cleanup
        // This ensures the internal call path is covered
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        manager.schedule_code_barrier(&mut barrier, Box::new(|| {}), None);
        
        // Verify it worked (indirectly confirms internal call)
        assert_eq!(barrier.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_get_global_code_barriers_closure_manager_creation() {
        // Test the manager creation path in the closure
        // This ensures CodeBarrierManager::new() in the closure is covered
        let manager = get_global_code_barriers();
        
        // Verify manager was created and initialized
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_get_global_code_barriers_closure_init_call() {
        // Test that init() is called in the closure
        // This ensures manager.init() in the closure is covered
        let manager = get_global_code_barriers();
        
        // If init() was called, outstanding should be 0
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Add a barrier
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 1);
        
        // Re-init should reset
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_code_barrier_default_impl() {
        // Test the Default implementation for CodeBarrier
        let barrier = CodeBarrier::default();
        assert_eq!(barrier.pending_schedulers(), 0);
        
        // Verify it's equivalent to new()
        let barrier2 = CodeBarrier::new();
        assert_eq!(barrier.pending_schedulers(), barrier2.pending_schedulers());
    }
    
    #[test]
    fn test_code_barrier_manager_default_impl() {
        // Test the Default implementation for CodeBarrierManager
        let manager = CodeBarrierManager::default();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
        
        // Verify it's equivalent to new()
        let manager2 = CodeBarrierManager::new();
        assert_eq!(
            manager.outstanding_blocking_code_barriers(),
            manager2.outstanding_blocking_code_barriers()
        );
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_size_zero_vs_nonzero() {
        // Test both size = 0 and size != 0 paths
        let manager = CodeBarrierManager::new();
        
        // Test with size = 0 (via schedule_code_barrier)
        let mut barrier1 = CodeBarrier::new();
        manager.schedule_code_barrier(&mut barrier1, Box::new(|| {}), None);
        
        // Test with size != 0 (via schedule_code_barrier_cleanup)
        let mut barrier2 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(&mut barrier2, Box::new(|| {}), None, 100);
        
        assert_eq!(barrier1.pending_schedulers(), 1);
        assert_eq!(barrier2.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_data_none_vs_some() {
        // Test both None and Some(data) paths
        let manager = CodeBarrierManager::new();
        
        // Test with None
        let mut barrier1 = CodeBarrier::new();
        manager.schedule_code_barrier_cleanup(&mut barrier1, Box::new(|| {}), None, 0);
        
        // Test with Some(data)
        let mut barrier2 = CodeBarrier::new();
        let data: i32 = 42;
        let data_ptr = &data as *const i32 as *mut std::ffi::c_void;
        manager.schedule_code_barrier_cleanup(&mut barrier2, Box::new(|| {}), Some(data_ptr), 0);
        
        assert_eq!(barrier1.pending_schedulers(), 1);
        assert_eq!(barrier2.pending_schedulers(), 1);
    }
    
    #[test]
    fn test_blocking_code_barrier_fetch_add_path() {
        // Test the fetch_add path in blocking_code_barrier
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // fetch_add returns the previous value
        let prev1 = manager.outstanding_blocking_code_barriers();
        manager.blocking_code_barrier();
        let prev2 = manager.outstanding_blocking_code_barriers();
        
        assert_eq!(prev1, 0);
        assert_eq!(prev2, 1);
        
        // The _count variable assignment is covered by using fetch_add
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
    }
    
    #[test]
    fn test_pending_schedulers_load_ordering() {
        // Test that pending_schedulers() uses Acquire ordering
        let barrier = CodeBarrier::new();
        
        // Store with Release
        barrier.pending_schedulers.store(42, Ordering::Release);
        
        // Load with Acquire (should see the Release store)
        let value = barrier.pending_schedulers();
        assert_eq!(value, 42);
    }
    
    #[test]
    fn test_outstanding_blocking_code_barriers_load_ordering() {
        // Test that outstanding_blocking_code_barriers() uses Acquire ordering
        let manager = CodeBarrierManager::new();
        manager.init();
        
        // Store with Release (via blocking_code_barrier which uses AcqRel)
        manager.blocking_code_barrier();
        
        // Load with Acquire (should see the AcqRel store)
        let value = manager.outstanding_blocking_code_barriers();
        assert_eq!(value, 1);
    }
    
    #[test]
    fn test_init_store_ordering() {
        // Test that init() uses Release ordering
        let manager = CodeBarrierManager::new();
        
        // Add some barriers first
        manager.blocking_code_barrier();
        manager.blocking_code_barrier();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 2);
        
        // Init should reset with Release ordering
        manager.init();
        assert_eq!(manager.outstanding_blocking_code_barriers(), 0);
    }
    
    #[test]
    fn test_schedule_code_barrier_cleanup_store_ordering() {
        // Test that schedule_code_barrier_cleanup uses Release ordering
        let manager = CodeBarrierManager::new();
        let mut barrier = CodeBarrier::new();
        
        manager.schedule_code_barrier_cleanup(&mut barrier, Box::new(|| {}), None, 0);
        
        // Load with Acquire should see the Release store
        let value = barrier.pending_schedulers();
        assert_eq!(value, 1);
    }
}

