//! Code Index Management
//!
//! Provides code index management for atomic code updates.
//! Based on code_ix.c - Code index and staging management.

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
}

