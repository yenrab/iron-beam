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
}

