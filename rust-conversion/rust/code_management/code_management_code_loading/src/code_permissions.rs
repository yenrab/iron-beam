//! Code Permission Management
//!
//! Provides code modification and staging permission management.
//! Based on code_ix.c - Code permission locking system.
//!
//! Code permissions ensure exclusive access to code modification operations:
//! - Code modification permission: For tracing, breakpoints, etc.
//! - Code staging permission: For code loading and purging
//! - Code load permission: Both staging and modification (for full code loading)

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

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Process ID type (simplified - in full implementation would be Process*)
pub type ProcessId = u64;

/// Code permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePermissionType {
    /// Code modification permission (for tracing, breakpoints)
    Modification,
    /// Code staging permission (for code loading and purging)
    Staging,
    /// Code load permission (both staging and modification)
    Load,
}

/// Code permission queue item
struct CodePermissionQueueItem {
    /// Process waiting for permission (if process-based)
    process_id: Option<ProcessId>,
    /// Auxiliary function to call (if aux-based)
    aux_func: Option<Box<dyn FnOnce() + Send>>,
}

/// Code permission structure
struct CodePermission {
    /// Mutex for synchronization
    lock: Mutex<()>,
    /// Current owner process ID (if any)
    owner: Option<ProcessId>,
    /// Whether permission is currently seized
    seized: bool,
    /// Queue of waiting processes/functions
    queue: VecDeque<CodePermissionQueueItem>,
}

/// Code permission manager
pub struct CodePermissionManager {
    /// Code modification permission
    mod_permission: Arc<Mutex<CodePermission>>,
    /// Code staging permission
    stage_permission: Arc<Mutex<CodePermission>>,
}

impl CodePermission {
    fn new() -> Self {
        Self {
            lock: Mutex::new(()),
            owner: None,
            seized: false,
            queue: VecDeque::new(),
        }
    }
}

impl CodePermissionManager {
    /// Create a new code permission manager
    pub fn new() -> Self {
        Self {
            mod_permission: Arc::new(Mutex::new(CodePermission::new())),
            stage_permission: Arc::new(Mutex::new(CodePermission::new())),
        }
    }

    /// Initialize the code permission manager
    pub fn init(&self) {
        // Initialization is done in new()
        // In a full implementation, this would set up thread-local storage for debug checks
    }

    /// Try to seize code modification permission
    ///
    /// # Arguments
    /// * `process_id` - Process ID requesting permission
    ///
    /// # Returns
    /// `true` if permission was granted, `false` if already seized (process should yield)
    pub fn try_seize_code_mod_permission(&self, process_id: ProcessId) -> bool {
        self.try_seize_permission(&self.mod_permission, Some(process_id), None)
    }

    /// Release code modification permission
    ///
    /// Resumes any waiting processes
    pub fn release_code_mod_permission(&self) {
        self.release_permission(&self.mod_permission);
    }

    /// Try to seize code staging permission
    ///
    /// # Arguments
    /// * `process_id` - Process ID requesting permission
    ///
    /// # Returns
    /// `true` if permission was granted, `false` if already seized (process should yield)
    pub fn try_seize_code_stage_permission(&self, process_id: ProcessId) -> bool {
        self.try_seize_permission(&self.stage_permission, Some(process_id), None)
    }

    /// Release code staging permission
    ///
    /// Resumes any waiting processes
    pub fn release_code_stage_permission(&self) {
        self.release_permission(&self.stage_permission);
    }

    /// Try to seize code load permission (both staging and modification)
    ///
    /// # Arguments
    /// * `process_id` - Process ID requesting permission
    ///
    /// # Returns
    /// `true` if both permissions were granted, `false` otherwise
    pub fn try_seize_code_load_permission(&self, process_id: ProcessId) -> bool {
        // First try staging permission
        if self.try_seize_code_stage_permission(process_id) {
            // Then try modification permission
            if self.try_seize_code_mod_permission(process_id) {
                return true;
            }
            // If modification failed, release staging
            self.release_code_stage_permission();
        }
        false
    }

    /// Release code load permission (releases both staging and modification)
    pub fn release_code_load_permission(&self) {
        self.release_code_mod_permission();
        self.release_code_stage_permission();
    }

    /// Check if current process has code modification permission
    ///
    /// # Arguments
    /// * `process_id` - Process ID to check
    ///
    /// # Returns
    /// `true` if process has permission
    pub fn has_code_mod_permission(&self, process_id: ProcessId) -> bool {
        self.has_permission(&self.mod_permission, process_id)
    }

    /// Check if current process has code staging permission
    ///
    /// # Arguments
    /// * `process_id` - Process ID to check
    ///
    /// # Returns
    /// `true` if process has permission
    pub fn has_code_stage_permission(&self, process_id: ProcessId) -> bool {
        self.has_permission(&self.stage_permission, process_id)
    }

    /// Check if current process has code load permission
    ///
    /// # Arguments
    /// * `process_id` - Process ID to check
    ///
    /// # Returns
    /// `true` if process has both permissions
    pub fn has_code_load_permission(&self, process_id: ProcessId) -> bool {
        self.has_code_stage_permission(process_id) && self.has_code_mod_permission(process_id)
    }

    /// Internal: Try to seize a permission
    fn try_seize_permission(
        &self,
        permission: &Arc<Mutex<CodePermission>>,
        process_id: Option<ProcessId>,
        _aux_func: Option<Box<dyn FnOnce() + Send>>,
    ) -> bool {
        let mut perm = permission.lock().unwrap();
        
        if !perm.seized {
            // Permission available - grant it
            perm.owner = process_id;
            perm.seized = true;
            true
        } else {
            // Permission already seized - add to queue
            // In a full implementation, this would suspend the process
            // For now, we just add to queue and return false
            perm.queue.push_back(CodePermissionQueueItem {
                process_id,
                aux_func: _aux_func,
            });
            false
        }
    }

    /// Internal: Release a permission
    fn release_permission(&self, permission: &Arc<Mutex<CodePermission>>) {
        let mut perm = permission.lock().unwrap();
        
        // Resume all waiting processes/functions
        while let Some(item) = perm.queue.pop_front() {
            if let Some(_process_id) = item.process_id {
                // In a full implementation, would resume the process here
                // For now, we just remove from queue
            }
            if let Some(func) = item.aux_func {
                // In a full implementation, would schedule aux work here
                // For now, we just drop the function
                drop(func);
            }
        }

        perm.owner = None;
        perm.seized = false;
    }

    /// Internal: Check if process has permission
    fn has_permission(&self, permission: &Arc<Mutex<CodePermission>>, process_id: ProcessId) -> bool {
        let perm = permission.lock().unwrap();
        perm.seized && perm.owner == Some(process_id)
    }
}

impl Default for CodePermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global code permission manager (singleton)
static GLOBAL_CODE_PERMISSIONS: std::sync::OnceLock<CodePermissionManager> = std::sync::OnceLock::new();

/// Get the global code permission manager
pub fn get_global_code_permissions() -> &'static CodePermissionManager {
    GLOBAL_CODE_PERMISSIONS.get_or_init(|| {
        let manager = CodePermissionManager::new();
        manager.init();
        manager
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_permission_manager_init() {
        let manager = CodePermissionManager::new();
        manager.init();
    }

    #[test]
    fn test_try_seize_code_mod_permission() {
        let manager = CodePermissionManager::new();
        let process_id = 1;

        // First seize should succeed
        assert!(manager.try_seize_code_mod_permission(process_id));
        assert!(manager.has_code_mod_permission(process_id));

        // Second seize should fail (already seized)
        assert!(!manager.try_seize_code_mod_permission(process_id));

        // Release and try again
        manager.release_code_mod_permission();
        assert!(!manager.has_code_mod_permission(process_id));
        assert!(manager.try_seize_code_mod_permission(process_id));
    }

    #[test]
    fn test_try_seize_code_load_permission() {
        let manager = CodePermissionManager::new();
        let process_id = 1;

        // Should succeed
        assert!(manager.try_seize_code_load_permission(process_id));
        assert!(manager.has_code_load_permission(process_id));
        assert!(manager.has_code_stage_permission(process_id));
        assert!(manager.has_code_mod_permission(process_id));

        // Release
        manager.release_code_load_permission();
        assert!(!manager.has_code_load_permission(process_id));
    }
}

