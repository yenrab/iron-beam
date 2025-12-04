//! Code Index Management
//!
//! Provides code index management for atomic code updates in the Erlang/OTP runtime system.
//! This module implements the code index mechanism that enables hot code loading and
//! atomic code updates without stopping the runtime.
//!
//! ## Overview
//!
//! The code index system manages multiple code versions simultaneously:
//! - **Active Code Index**: Currently running code version
//! - **Staging Code Index**: Code being prepared for activation
//! - **Spare Index**: Available for future code updates
//!
//! This allows atomic code updates where new code can be loaded and staged before
//! switching to it, ensuring zero-downtime code updates.
//!
//! ## Code Update Process
//!
//! 1. Load new code into the staging index
//! 2. Validate and prepare the new code
//! 3. Atomically switch from active to staging index
//! 4. Old code remains available for processes still using it
//!
//! ## Examples
//!
//! ```rust
//! use code_management_code_loading::code_index::CodeIndexManager;
//!
//! let manager = CodeIndexManager::new();
//! manager.init();
//! let active = manager.active_code_ix();
//! let staging = manager.staging_code_ix();
//! // Load code into staging...
//! manager.start_staging(0);
//! manager.commit_staging();
//! ```
//!
//! ## See Also
//!
//! - [`code_barriers`](super::code_barriers/index.html): Code barriers for safe code updates
//! - [`code_loader`](super::code_loader/index.html): Code loading functionality
//! - [`module_management`](super::module_management/index.html): Module management
//!
//! Based on `code_ix.c` - Code index and staging management

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

use std::sync::atomic::{AtomicU32, Ordering};

/// Number of code indices (active, staging, and one spare)
pub const NUM_CODE_IX: usize = 3;

/// Code index type
pub type CodeIndex = u32;

/// Code index manager - manages active and staging code indices
pub struct CodeIndexManager {
    /// Active code index (currently running code)
    active_code_index: AtomicU32,
    /// Staging code index (code being prepared for activation)
    staging_code_index: AtomicU32,
}

impl CodeIndexManager {
    /// Create a new code index manager
    pub fn new() -> Self {
        Self {
            active_code_index: AtomicU32::new(0),
            staging_code_index: AtomicU32::new(0),
        }
    }

    /// Initialize code indices (called at system startup)
    ///
    /// Both active and staging start at 0 during preloading.
    /// After preloading, a commit sets them to their proper values.
    pub fn init(&self) {
        // Both start at 0 during initialization
        self.active_code_index.store(0, Ordering::Release);
        self.staging_code_index.store(0, Ordering::Release);
    }

    /// Get the active code index
    ///
    /// This is guaranteed to be valid until the calling function returns.
    /// For consistency, only one call should be made and the result reused.
    pub fn active_code_ix(&self) -> CodeIndex {
        self.active_code_index.load(Ordering::Acquire)
    }

    /// Get the staging code index
    ///
    /// Only used by a process performing code loading/upgrading/deleting/purging.
    /// Code staging permission must be seized.
    pub fn staging_code_ix(&self) -> CodeIndex {
        self.staging_code_index.load(Ordering::Acquire)
    }

    /// Start staging code index
    ///
    /// Prepares the staging area to be a complete copy of the active code.
    /// Code staging permission must have been seized.
    ///
    /// # Arguments
    /// * `_num_new` - Number of new modules expected (for future use)
    pub fn start_staging(&self, _num_new: usize) {
        // Calculate next staging index
        let current_active = self.active_code_index.load(Ordering::Acquire);
        let next_staging = (current_active + 1) % NUM_CODE_IX as u32;
        
        // Set staging index
        self.staging_code_index.store(next_staging, Ordering::Release);
        
        // In a full implementation, this would also:
        // - Start staging for beam_catches
        // - Start staging for functions
        // - Start staging for exports
        // - Start staging for modules
        // - Start staging for ranges
    }

    /// End staging code index
    ///
    /// Must be preceded by `start_staging` and followed by `commit_staging`.
    pub fn end_staging(&self) {
        // In a full implementation, this would:
        // - End staging for beam_catches
        // - End staging for functions
        // - End staging for exports
        // - End staging for modules
        // - End staging for ranges
    }

    /// Commit staging code index
    ///
    /// Sets the staging code index as the new active code index.
    /// Must be preceded by `end_staging`.
    pub fn commit_staging(&self) {
        let staging_ix = self.staging_code_index.load(Ordering::Acquire);
        
        // Make staging the new active
        self.active_code_index.store(staging_ix, Ordering::Release);
        
        // Calculate next staging index
        let next_staging = (staging_ix + 1) % NUM_CODE_IX as u32;
        self.staging_code_index.store(next_staging, Ordering::Release);
    }

    /// Abort staging code index
    ///
    /// Abandons the staging changes.
    /// Must be preceded by `start_staging`.
    pub fn abort_staging(&self) {
        // In a full implementation, this would:
        // - End staging for beam_catches (with commit=0)
        // - End staging for functions (with commit=0)
        // - End staging for exports (with commit=0)
        // - End staging for modules (with commit=0)
        // - End staging for ranges (with commit=0)
        
        // Reset staging to match active (abandon changes)
        let active_ix = self.active_code_index.load(Ordering::Acquire);
        self.staging_code_index.store(active_ix, Ordering::Release);
    }

    /// Get outstanding blocking code barriers count
    ///
    /// Returns the number of outstanding blocking code barriers.
    /// This is used for thread synchronization.
    pub fn outstanding_blocking_code_barriers(&self) -> u32 {
        // In a full implementation, this would track actual barriers
        // For now, return 0
        0
    }
}

impl Default for CodeIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global code index manager (singleton)
static GLOBAL_CODE_IX: std::sync::OnceLock<CodeIndexManager> = std::sync::OnceLock::new();

/// Get the global code index manager
pub fn get_global_code_ix() -> &'static CodeIndexManager {
    GLOBAL_CODE_IX.get_or_init(|| {
        let manager = CodeIndexManager::new();
        manager.init();
        manager
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_index_manager_init() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        assert_eq!(manager.active_code_ix(), 0);
        assert_eq!(manager.staging_code_ix(), 0);
    }

    #[test]
    fn test_code_index_staging_cycle() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Start staging
        manager.start_staging(0);
        let staging_ix = manager.staging_code_ix();
        assert_eq!(staging_ix, 1); // Next index after 0
        
        // End staging
        manager.end_staging();
        
        // Commit staging
        manager.commit_staging();
        assert_eq!(manager.active_code_ix(), 1);
        assert_eq!(manager.staging_code_ix(), 2); // Next index after 1
    }

    #[test]
    fn test_code_index_abort() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        let initial_active = manager.active_code_ix();
        
        // Start staging
        manager.start_staging(0);
        assert_ne!(manager.staging_code_ix(), initial_active);
        
        // Abort staging
        manager.abort_staging();
        assert_eq!(manager.staging_code_ix(), initial_active);
    }

    #[test]
    fn test_code_index_wraparound() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Simulate multiple commits to test wraparound
        for _ in 0..5 {
            manager.start_staging(0);
            manager.end_staging();
            manager.commit_staging();
        }
        
        // Should have wrapped around (5 commits: 0->1->2->0->1->2)
        let active = manager.active_code_ix();
        assert!(active < NUM_CODE_IX as u32);
    }

    #[test]
    fn test_global_code_ix() {
        let manager1 = get_global_code_ix();
        let manager2 = get_global_code_ix();
        
        // Should return the same instance
        assert_eq!(manager1.active_code_ix(), manager2.active_code_ix());
    }

    #[test]
    fn test_code_index_manager_default() {
        // Test Default trait implementation
        let manager = CodeIndexManager::default();
        
        // Verify it works the same as new()
        assert_eq!(manager.active_code_ix(), 0);
        assert_eq!(manager.staging_code_ix(), 0);
        
        // Test that it can be used
        manager.start_staging(0);
        assert_eq!(manager.staging_code_ix(), 1);
    }

    #[test]
    fn test_code_index_new() {
        // Test new() method explicitly
        let manager = CodeIndexManager::new();
        
        // Should start at 0 for both
        assert_eq!(manager.active_code_ix(), 0);
        assert_eq!(manager.staging_code_ix(), 0);
    }

    #[test]
    fn test_code_index_init_multiple_times() {
        let manager = CodeIndexManager::new();
        
        // Set to different values
        manager.start_staging(0);
        manager.commit_staging();
        assert_eq!(manager.active_code_ix(), 1);
        
        // Init should reset to 0
        manager.init();
        assert_eq!(manager.active_code_ix(), 0);
        assert_eq!(manager.staging_code_ix(), 0);
    }

    #[test]
    fn test_code_index_end_staging() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Start staging
        manager.start_staging(0);
        let staging_before = manager.staging_code_ix();
        
        // End staging (should not change staging index)
        manager.end_staging();
        let staging_after = manager.staging_code_ix();
        
        // end_staging doesn't change the staging index
        assert_eq!(staging_before, staging_after);
    }

    #[test]
    fn test_code_index_commit_without_end() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Start staging
        manager.start_staging(0);
        let staging_ix = manager.staging_code_ix();
        
        // Commit without end_staging (should still work)
        manager.commit_staging();
        assert_eq!(manager.active_code_ix(), staging_ix);
    }

    #[test]
    fn test_code_index_abort_after_commit() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Start and commit
        manager.start_staging(0);
        manager.commit_staging();
        let active_after_commit = manager.active_code_ix();
        
        // Start staging again
        manager.start_staging(0);
        let staging_before_abort = manager.staging_code_ix();
        assert_ne!(staging_before_abort, active_after_commit);
        
        // Abort should reset to active
        manager.abort_staging();
        assert_eq!(manager.staging_code_ix(), active_after_commit);
    }

    #[test]
    fn test_code_index_wraparound_edge_cases() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Test wraparound from 2 to 0
        // Set to index 2
        manager.start_staging(0); // staging = 1
        manager.commit_staging(); // active = 1, staging = 2
        manager.start_staging(0); // staging = 2
        manager.commit_staging(); // active = 2, staging = 0
        
        assert_eq!(manager.active_code_ix(), 2);
        assert_eq!(manager.staging_code_ix(), 0);
        
        // Next commit should wrap to 1
        manager.start_staging(0); // staging = 0
        manager.commit_staging(); // active = 0, staging = 1
        assert_eq!(manager.active_code_ix(), 0);
        assert_eq!(manager.staging_code_ix(), 1);
    }

    #[test]
    fn test_code_index_start_staging_with_different_num_new() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Test with different num_new values (parameter is currently unused but should be callable)
        manager.start_staging(0);
        let staging1 = manager.staging_code_ix();
        
        manager.commit_staging();
        manager.start_staging(5);
        let staging2 = manager.staging_code_ix();
        
        manager.commit_staging();
        manager.start_staging(100);
        let staging3 = manager.staging_code_ix();
        
        // All should work the same way (num_new is for future use)
        assert!(staging1 < NUM_CODE_IX as u32);
        assert!(staging2 < NUM_CODE_IX as u32);
        assert!(staging3 < NUM_CODE_IX as u32);
    }

    #[test]
    fn test_code_index_multiple_abort_cycles() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Multiple abort cycles
        for _ in 0..10 {
            let active_before = manager.active_code_ix();
            manager.start_staging(0);
            manager.abort_staging();
            assert_eq!(manager.staging_code_ix(), active_before);
        }
    }

    #[test]
    fn test_code_index_full_cycle_with_abort() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Full cycle: start -> end -> commit
        manager.start_staging(0);
        manager.end_staging();
        manager.commit_staging();
        let active1 = manager.active_code_ix();
        
        // Another cycle with abort
        manager.start_staging(0);
        manager.abort_staging();
        assert_eq!(manager.active_code_ix(), active1); // Should not change
        
        // Another full cycle
        manager.start_staging(0);
        manager.end_staging();
        manager.commit_staging();
        let active2 = manager.active_code_ix();
        assert_ne!(active1, active2);
    }

    #[test]
    fn test_code_index_constants() {
        // Test that NUM_CODE_IX constant is accessible
        assert_eq!(NUM_CODE_IX, 3);
        
        // Test CodeIndex type
        let _index: CodeIndex = 0;
        let _index2: CodeIndex = NUM_CODE_IX as u32;
    }

    #[test]
    fn test_code_index_global_initialization() {
        // Test that global code ix is properly initialized
        let global = get_global_code_ix();
        
        // Should be initialized to 0
        assert_eq!(global.active_code_ix(), 0);
        assert_eq!(global.staging_code_ix(), 0);
        
        // Should be usable
        global.start_staging(0);
        assert_eq!(global.staging_code_ix(), 1);
    }

    #[test]
    fn test_code_index_concurrent_operations() {
        let manager = CodeIndexManager::new();
        manager.init();
        
        // Simulate rapid operations
        for i in 0..20 {
            if i % 3 == 0 {
                manager.start_staging(0);
            } else if i % 3 == 1 {
                manager.end_staging();
            } else {
                manager.commit_staging();
            }
            
            // Verify indices are always valid
            let active = manager.active_code_ix();
            let staging = manager.staging_code_ix();
            assert!(active < NUM_CODE_IX as u32);
            assert!(staging < NUM_CODE_IX as u32);
        }
    }
}

