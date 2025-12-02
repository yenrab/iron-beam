//! Module Management
//!
//! Provides module table management, module lookup, and module staging.
//! Based on module.c - Module table and instance management.

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
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

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicU64, Ordering};

/// Module instance - represents a single version of a module's code
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    /// Code header (simplified - will be properly typed later)
    pub code_hdr: Option<*const ()>,
    /// Length of loaded code in bytes
    pub code_length: u32,
    /// Catches (simplified)
    pub catches: u32,
    /// NIF pointer (simplified)
    pub nif: Option<*mut ()>,
    /// Number of breakpoints
    pub num_breakpoints: u32,
    /// Number of traced exports
    pub num_traced_exports: u32,
    /// Executable region (simplified)
    pub executable_region: Option<*const ()>,
    /// Writable region (simplified)
    pub writable_region: Option<*mut ()>,
    /// Metadata (simplified)
    pub metadata: Option<*mut ()>,
    /// Whether module is unsealed (can be modified)
    pub unsealed: bool,
}

impl ModuleInstance {
    /// Initialize a module instance (equivalent to erts_module_instance_init)
    ///
    /// Sets all fields to their default/empty state.
    pub fn init(&mut self) {
        self.code_hdr = None;
        self.code_length = 0;
        self.catches = 0;
        self.nif = None;
        self.num_breakpoints = 0;
        self.num_traced_exports = 0;
        self.executable_region = None;
        self.writable_region = None;
        self.metadata = None;
        self.unsealed = false;
    }

    /// Unseal a module (make it writable for modification)
    ///
    /// Equivalent to erts_unseal_module(). The module must not already be unsealed.
    ///
    /// # Panics
    /// Panics if the module is already unsealed (in debug builds).
    pub fn unseal(&mut self) {
        debug_assert!(!self.unsealed, "Module is already unsealed");
        self.unsealed = true;
    }

    /// Seal a module (make it read-only after modification)
    ///
    /// Equivalent to erts_seal_module(). The module must be unsealed.
    ///
    /// # Panics
    /// Panics if the module is not unsealed (in debug builds).
    pub fn seal(&mut self) {
        debug_assert!(self.unsealed, "Module is not unsealed");
        self.unsealed = false;
    }

    /// Convert a code pointer to a writable pointer
    ///
    /// Equivalent to erts_writable_code_ptr(). The module must be unsealed.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to convert (must point within the module's code)
    ///
    /// # Returns
    /// Writable pointer, or None if pointer is invalid or module is not unsealed
    pub fn writable_code_ptr(&self, ptr: *const ()) -> Option<*mut ()> {
        if !self.unsealed {
            return None;
        }

        // In a full implementation, this would:
        // 1. Check that ptr is within the executable_region
        // 2. Calculate offset from executable_region start
        // 3. Return writable_region + offset
        // For now, simplified implementation
        if let (Some(exec_start), Some(writable_start)) = 
            (self.executable_region, self.writable_region) {
            let exec_start = exec_start as usize;
            let writable_start = writable_start as usize;
            let ptr_addr = ptr as usize;
            
            // Check if ptr is within executable region
            if ptr_addr >= exec_start && 
               ptr_addr < exec_start + self.code_length as usize {
                let offset = ptr_addr - exec_start;
                Some((writable_start + offset) as *mut ())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for ModuleInstance {
    fn default() -> Self {
        let mut inst = Self {
            code_hdr: None,
            code_length: 0,
            catches: 0,
            nif: None,
            num_breakpoints: 0,
            num_traced_exports: 0,
            executable_region: None,
            writable_region: None,
            metadata: None,
            unsealed: false,
        };
        inst.init();
        inst
    }
}

// ModuleInstance is Send + Sync because:
// - All fields are either primitive types or raw pointers
// - Raw pointers are Send + Sync (they don't own the data)
// - The actual memory safety is managed by the VM/runtime
unsafe impl Send for ModuleInstance {}
unsafe impl Sync for ModuleInstance {}

/// Module entry in the module table
#[derive(Debug, Clone)]
pub struct Module {
    /// Module atom index (not tagged)
    pub module: u32,
    /// Seen flag (used by finish_loading)
    pub seen: bool,
    /// Current module instance
    pub curr: ModuleInstance,
    /// Old module instance (for code updates)
    pub old: ModuleInstance,
    /// On-load module instance (if module has on_load function)
    pub on_load: Option<ModuleInstance>,
}

// Module is Send + Sync because:
// - All fields are either primitive types or ModuleInstance (which is Send + Sync)
// - ModuleInstance is already marked as Send + Sync
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

/// Module table - manages modules for a code index
#[derive(Debug)]
pub struct ModuleTable {
    /// Hash map from module atom index to module
    modules: Arc<RwLock<HashMap<u32, Arc<Module>>>>,
    /// Total bytes used by modules
    total_bytes: AtomicU64,
    /// Maximum number of modules
    limit: usize,
}

impl ModuleTable {
    /// Create a new module table
    ///
    /// # Arguments
    /// * `initial_size` - Initial size of the table
    /// * `limit` - Maximum number of modules
    pub fn new(initial_size: usize, limit: usize) -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::with_capacity(initial_size))),
            total_bytes: AtomicU64::new(0),
            limit,
        }
    }

    /// Get a module by atom index
    ///
    /// # Arguments
    /// * `module` - Module atom index
    ///
    /// # Returns
    /// Reference to module if found, None otherwise
    pub fn get_module(&self, module: u32) -> Option<Arc<Module>> {
        let modules = self.modules.read().unwrap();
        modules.get(&module).map(|m| Arc::clone(m))
    }

    /// Put a module into the table (insert or update)
    ///
    /// # Arguments
    /// * `module` - Module atom index
    ///
    /// # Returns
    /// Reference to the module (existing or newly created)
    pub fn put_module(&self, module: u32) -> Arc<Module> {
        let mut modules = self.modules.write().unwrap();
        
        if let Some(existing) = modules.get(&module) {
            Arc::clone(existing)
        } else {
            // Check limit
            if modules.len() >= self.limit {
                // In a full implementation, this would return an error
                // For now, we'll allow it but log a warning
            }
            
            let new_module = Arc::new(Module {
                module,
                seen: false,
                curr: ModuleInstance::default(),
                old: ModuleInstance::default(),
                on_load: None,
            });
            
            // Update total bytes (simplified - actual size calculation would be more complex)
            self.total_bytes.fetch_add(std::mem::size_of::<Module>() as u64, Ordering::Relaxed);
            
            modules.insert(module, Arc::clone(&new_module));
            new_module
        }
    }

    /// Get the number of modules in the table
    pub fn size(&self) -> usize {
        let modules = self.modules.read().unwrap();
        modules.len()
    }

    /// Get total bytes used by modules
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes.load(Ordering::Acquire)
    }

    /// Get module by index (for iteration)
    ///
    /// # Arguments
    /// * `index` - Index in the table
    ///
    /// # Returns
    /// Module if index is valid, None otherwise
    pub fn get_module_by_index(&self, index: usize) -> Option<Arc<Module>> {
        let modules = self.modules.read().unwrap();
        let keys: Vec<u32> = modules.keys().copied().collect();
        if index < keys.len() {
            let key = keys[index];
            modules.get(&key).map(|m| Arc::clone(m))
        } else {
            None
        }
    }

    /// Clear all modules (for testing/cleanup)
    pub fn clear(&self) {
        let mut modules = self.modules.write().unwrap();
        modules.clear();
        self.total_bytes.store(0, Ordering::Release);
    }
}

/// Old code read-write lock
///
/// Protects access to old module instances. Old code can be read concurrently,
/// but purging requires exclusive access.
pub struct OldCodeLock {
    /// Read-write lock for old code
    lock: RwLock<()>,
}

impl OldCodeLock {
    /// Create a new old code lock
    pub fn new() -> Self {
        Self {
            lock: RwLock::new(()),
        }
    }

    /// Acquire read lock for old code
    pub fn read(&self) -> RwLockReadGuard<'_, ()> {
        self.lock.read().unwrap()
    }

    /// Acquire write lock for old code
    pub fn write(&self) -> RwLockWriteGuard<'_, ()> {
        self.lock.write().unwrap()
    }
}

impl Default for OldCodeLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Module table manager - manages module tables for all code indices
pub struct ModuleTableManager {
    /// Module tables for each code index (3 indices total)
    tables: [ModuleTable; 3],
    /// Old code locks for each code index
    old_code_locks: [OldCodeLock; 3],
}

impl ModuleTableManager {
    /// Create a new module table manager
    pub fn new() -> Self {
        const MODULE_SIZE: usize = 50;
        const MODULE_LIMIT: usize = 64 * 1024;
        
        Self {
            tables: [
                ModuleTable::new(MODULE_SIZE, MODULE_LIMIT),
                ModuleTable::new(MODULE_SIZE, MODULE_LIMIT),
                ModuleTable::new(MODULE_SIZE, MODULE_LIMIT),
            ],
            old_code_locks: [
                OldCodeLock::new(),
                OldCodeLock::new(),
                OldCodeLock::new(),
            ],
        }
    }

    /// Get module table for a specific code index
    ///
    /// # Arguments
    /// * `code_ix` - Code index (0, 1, or 2)
    ///
    /// # Returns
    /// Reference to the module table
    pub fn get_table(&self, code_ix: usize) -> &ModuleTable {
        &self.tables[code_ix % 3]
    }

    /// Initialize module tables (called at system startup)
    pub fn init(&self) {
        // Tables are already initialized in new()
        // This method exists for API compatibility with C code
    }

    /// Start staging modules (copy from active to staging table)
    ///
    /// Equivalent to module_start_staging(). Copies all modules from the active
    /// code index to the staging code index.
    ///
    /// # Arguments
    /// * `active_ix` - Active code index
    /// * `staging_ix` - Staging code index
    pub fn start_staging(&self, active_ix: usize, staging_ix: usize) {
        let src_table = self.get_table(active_ix);
        let dst_table = self.get_table(staging_ix);
        
        let src_modules = src_table.modules.read().unwrap();
        let mut dst_modules = dst_table.modules.write().unwrap();
        
        // Copy all modules from source to destination
        for (module_id, src_module) in src_modules.iter() {
            // Create a copy of the module
            let dst_module = Arc::new(Module {
                module: src_module.module,
                seen: src_module.seen,
                curr: src_module.curr.clone(),
                old: src_module.old.clone(),
                on_load: src_module.on_load.clone(),
            });
            
            // Update total bytes
            let module_size = std::mem::size_of::<Module>() as u64;
            dst_table.total_bytes.fetch_add(module_size, Ordering::Relaxed);
            
            dst_modules.insert(*module_id, dst_module);
        }
    }

    /// End staging modules (finalize staging operation)
    ///
    /// Equivalent to module_end_staging(). If commit is false, removes modules
    /// added during staging. If commit is true, keeps them.
    ///
    /// # Arguments
    /// * `staging_ix` - Staging code index
    /// * `commit` - Whether to commit (true) or abort (false) the staging
    /// * `entries_at_start` - Number of entries at start of staging
    pub fn end_staging(&self, staging_ix: usize, commit: bool, entries_at_start: usize) {
        let table = self.get_table(staging_ix);
        
        if !commit {
            // Abort: remove modules added during staging
            let mut modules = table.modules.write().unwrap();
            let keys_to_remove: Vec<u32> = modules
                .keys()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .skip(entries_at_start)
                .collect();
            
            for key in keys_to_remove {
                if modules.remove(&key).is_some() {
                    let module_size = std::mem::size_of::<Module>() as u64;
                    table.total_bytes.fetch_sub(module_size, Ordering::Relaxed);
                }
            }
        }
        // If commit is true, modules are kept (no action needed)
    }

    /// Get module by code index and numeric index
    ///
    /// Equivalent to module_code(). Returns module at numeric index in the table.
    ///
    /// # Arguments
    /// * `index` - Numeric index in the table
    /// * `code_ix` - Code index
    ///
    /// # Returns
    /// Module if found, None otherwise
    pub fn module_code(&self, index: usize, code_ix: usize) -> Option<Arc<Module>> {
        let table = self.get_table(code_ix);
        table.get_module_by_index(index)
    }

    /// Get module code size for a code index
    ///
    /// Equivalent to module_code_size(). Returns number of modules in the table.
    ///
    /// # Arguments
    /// * `code_ix` - Code index
    ///
    /// # Returns
    /// Number of modules
    pub fn module_code_size(&self, code_ix: usize) -> usize {
        let table = self.get_table(code_ix);
        table.size()
    }

    /// Get total module table size in bytes
    ///
    /// Equivalent to module_table_sz(). Returns total bytes used by all modules.
    ///
    /// # Returns
    /// Total bytes
    pub fn module_table_sz(&self) -> u64 {
        // Sum total bytes from all tables
        self.tables.iter().map(|t| t.total_bytes()).sum()
    }

    /// Get module information (for debugging)
    ///
    /// Equivalent to module_info(). Returns information about the module table.
    ///
    /// # Arguments
    /// * `code_ix` - Code index
    ///
    /// # Returns
    /// String containing module table information
    pub fn module_info(&self, code_ix: usize) -> String {
        let table = self.get_table(code_ix);
        let size = table.size();
        let total_bytes = table.total_bytes();
        format!("Module table (code_ix={}): {} modules, {} bytes", code_ix, size, total_bytes)
    }

    /// Lock old code for reading
    ///
    /// Equivalent to erts_rlock_old_code(). Acquires read lock on old code.
    ///
    /// # Arguments
    /// * `code_ix` - Code index
    ///
    /// # Returns
    /// Read guard that must be dropped to release the lock
    pub fn rlock_old_code(&self, code_ix: usize) -> RwLockReadGuard<'_, ()> {
        self.old_code_locks[code_ix % 3].read()
    }

    /// Lock old code for writing
    ///
    /// Equivalent to erts_rwlock_old_code(). Acquires write lock on old code.
    ///
    /// # Arguments
    /// * `code_ix` - Code index
    ///
    /// # Returns
    /// Write guard that must be dropped to release the lock
    pub fn rwlock_old_code(&self, code_ix: usize) -> RwLockWriteGuard<'_, ()> {
        self.old_code_locks[code_ix % 3].write()
    }
}

impl Default for ModuleTableManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global module table manager (singleton)
static GLOBAL_MODULE_MANAGER: std::sync::OnceLock<ModuleTableManager> = std::sync::OnceLock::new();

/// Get the global module table manager
pub fn get_global_module_manager() -> &'static ModuleTableManager {
    GLOBAL_MODULE_MANAGER.get_or_init(|| {
        let manager = ModuleTableManager::new();
        manager.init();
        manager
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_instance_default() {
        let inst = ModuleInstance::default();
        assert_eq!(inst.code_length, 0);
        assert_eq!(inst.catches, 0);
        assert!(!inst.unsealed);
    }

    #[test]
    fn test_module_table_creation() {
        let table = ModuleTable::new(10, 100);
        assert_eq!(table.size(), 0);
        assert_eq!(table.total_bytes(), 0);
    }

    #[test]
    fn test_module_table_put_get() {
        let table = ModuleTable::new(10, 100);
        let module_ix = 42;
        
        let module = table.put_module(module_ix);
        assert_eq!(module.module, module_ix);
        assert_eq!(table.size(), 1);
        
        let retrieved = table.get_module(module_ix);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().module, module_ix);
    }

    #[test]
    fn test_module_table_put_existing() {
        let table = ModuleTable::new(10, 100);
        let module_ix = 100;
        
        let module1 = table.put_module(module_ix);
        let module2 = table.put_module(module_ix);
        
        // Should return the same module
        assert!(Arc::ptr_eq(&module1, &module2));
        assert_eq!(table.size(), 1);
    }

    #[test]
    fn test_module_table_manager() {
        let manager = ModuleTableManager::new();
        
        // Test getting tables for different code indices
        let table0 = manager.get_table(0);
        let table1 = manager.get_table(1);
        let table2 = manager.get_table(2);
        
        // Add modules to different tables to verify they're separate
        table0.put_module(1);
        table1.put_module(2);
        table2.put_module(3);
        
        assert_eq!(table0.size(), 1);
        assert_eq!(table1.size(), 1);
        assert_eq!(table2.size(), 1);
        
        // Verify they're different tables (different modules)
        assert!(table0.get_module(1).is_some());
        assert!(table0.get_module(2).is_none());
        
        // Test wrapping around
        let table3 = manager.get_table(3);
        assert_eq!(table3.size(), table0.size()); // Should wrap to index 0
        assert!(table3.get_module(1).is_some()); // Should have same module as table0
    }

    #[test]
    fn test_module_table_by_index() {
        let table = ModuleTable::new(10, 100);
        
        // Add some modules
        table.put_module(1);
        table.put_module(2);
        table.put_module(3);
        
        assert_eq!(table.size(), 3);
        
        // Get by index
        let module0 = table.get_module_by_index(0);
        assert!(module0.is_some());
        
        let module2 = table.get_module_by_index(2);
        assert!(module2.is_some());
        
        let module_invalid = table.get_module_by_index(10);
        assert!(module_invalid.is_none());
    }

    #[test]
    fn test_module_table_get_module_not_found() {
        let table = ModuleTable::new(10, 100);
        
        // Try to get a module that doesn't exist
        let result = table.get_module(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_module_table_get_module_by_index_empty() {
        let table = ModuleTable::new(10, 100);
        
        // Try to get from empty table
        let result = table.get_module_by_index(0);
        assert!(result.is_none());
    }

    #[test]
    fn test_module_table_clear() {
        let table = ModuleTable::new(10, 100);
        
        // Add some modules
        table.put_module(1);
        table.put_module(2);
        assert_eq!(table.size(), 2);
        assert!(table.total_bytes() > 0);
        
        // Clear the table
        table.clear();
        assert_eq!(table.size(), 0);
        assert_eq!(table.total_bytes(), 0);
        
        // Verify modules are gone
        assert!(table.get_module(1).is_none());
        assert!(table.get_module(2).is_none());
    }

    #[test]
    fn test_module_table_total_bytes() {
        let table = ModuleTable::new(10, 100);
        
        // Initially zero
        assert_eq!(table.total_bytes(), 0);
        
        // Add a module
        table.put_module(1);
        let bytes_after_one = table.total_bytes();
        assert!(bytes_after_one > 0);
        
        // Add another module
        table.put_module(2);
        let bytes_after_two = table.total_bytes();
        assert!(bytes_after_two > bytes_after_one);
    }

    #[test]
    fn test_module_table_limit() {
        let table = ModuleTable::new(10, 2);
        
        // Add modules up to limit
        table.put_module(1);
        assert_eq!(table.size(), 1);
        
        table.put_module(2);
        assert_eq!(table.size(), 2);
        
        // Try to add beyond limit - should still work (current implementation allows it)
        table.put_module(3);
        assert_eq!(table.size(), 3);
    }

    #[test]
    fn test_module_table_manager_init() {
        let manager = ModuleTableManager::new();
        
        // Test init method (should not panic)
        manager.init();
        
        // Verify tables are still accessible after init
        let table = manager.get_table(0);
        assert_eq!(table.size(), 0);
    }

    #[test]
    fn test_module_table_manager_default() {
        // Test Default trait implementation
        let manager = ModuleTableManager::default();
        
        // Verify it works the same as new()
        let table = manager.get_table(0);
        assert_eq!(table.size(), 0);
        
        table.put_module(1);
        assert_eq!(table.size(), 1);
    }

    #[test]
    fn test_module_table_manager_wraparound() {
        let manager = ModuleTableManager::new();
        
        // Test various code indices that wrap around
        let table0 = manager.get_table(0);
        let table3 = manager.get_table(3);  // Should wrap to 0
        let table6 = manager.get_table(6);  // Should wrap to 0
        let table9 = manager.get_table(9);  // Should wrap to 0
        
        // All should be the same table
        assert!(std::ptr::eq(table0, table3));
        assert!(std::ptr::eq(table0, table6));
        assert!(std::ptr::eq(table0, table9));
        
        // Test wrapping to index 1
        let table1 = manager.get_table(1);
        let table4 = manager.get_table(4);  // Should wrap to 1
        assert!(std::ptr::eq(table1, table4));
        
        // Test wrapping to index 2
        let table2 = manager.get_table(2);
        let table5 = manager.get_table(5);  // Should wrap to 2
        assert!(std::ptr::eq(table2, table5));
    }

    #[test]
    fn test_module_instance_init() {
        let mut inst = ModuleInstance::default();
        inst.code_length = 100;
        inst.unsealed = true;
        
        // Re-initialize
        inst.init();
        
        assert_eq!(inst.code_length, 0);
        assert!(!inst.unsealed);
    }

    #[test]
    fn test_module_instance_unseal_seal() {
        let mut inst = ModuleInstance::default();
        assert!(!inst.unsealed);
        
        inst.unseal();
        assert!(inst.unsealed);
        
        inst.seal();
        assert!(!inst.unsealed);
    }

    #[test]
    fn test_module_instance_writable_code_ptr() {
        let mut inst = ModuleInstance::default();
        inst.code_length = 100;
        inst.executable_region = Some(0x1000 as *const ());
        inst.writable_region = Some(0x2000 as *mut ());
        
        // Should fail if not unsealed
        assert!(inst.writable_code_ptr(0x1050 as *const ()).is_none());
        
        inst.unseal();
        
        // Should succeed if unsealed and pointer is within range
        let writable = inst.writable_code_ptr(0x1050 as *const ());
        assert!(writable.is_some());
        assert_eq!(writable.unwrap() as usize, 0x2050);
        
        // Should fail if pointer is out of range
        assert!(inst.writable_code_ptr(0x2000 as *const ()).is_none());
    }

    #[test]
    fn test_module_start_staging() {
        let manager = ModuleTableManager::new();
        let active_table = manager.get_table(0);
        let staging_table = manager.get_table(1);
        
        // Add modules to active table
        active_table.put_module(1);
        active_table.put_module(2);
        assert_eq!(active_table.size(), 2);
        assert_eq!(staging_table.size(), 0);
        
        // Start staging
        manager.start_staging(0, 1);
        
        // Staging table should now have the modules
        assert_eq!(staging_table.size(), 2);
        assert!(staging_table.get_module(1).is_some());
        assert!(staging_table.get_module(2).is_some());
    }

    #[test]
    fn test_module_end_staging_commit() {
        let manager = ModuleTableManager::new();
        let staging_table = manager.get_table(1);
        
        // Add modules during staging
        staging_table.put_module(1);
        staging_table.put_module(2);
        let entries_at_start = 0;
        
        // End staging with commit
        manager.end_staging(1, true, entries_at_start);
        
        // Modules should still be there
        assert_eq!(staging_table.size(), 2);
    }

    #[test]
    fn test_module_end_staging_abort() {
        let manager = ModuleTableManager::new();
        let staging_table = manager.get_table(1);
        
        // Add modules during staging
        staging_table.put_module(1);
        staging_table.put_module(2);
        let entries_at_start = 0;
        
        // End staging with abort
        manager.end_staging(1, false, entries_at_start);
        
        // Modules should be removed
        assert_eq!(staging_table.size(), 0);
    }

    #[test]
    fn test_module_code() {
        let manager = ModuleTableManager::new();
        let table = manager.get_table(0);
        
        table.put_module(1);
        table.put_module(2);
        
        let module = manager.module_code(0, 0);
        assert!(module.is_some());
    }

    #[test]
    fn test_module_code_size() {
        let manager = ModuleTableManager::new();
        let table = manager.get_table(0);
        
        table.put_module(1);
        table.put_module(2);
        table.put_module(3);
        
        assert_eq!(manager.module_code_size(0), 3);
    }

    #[test]
    fn test_module_table_sz() {
        let manager = ModuleTableManager::new();
        let table = manager.get_table(0);
        
        table.put_module(1);
        let bytes1 = manager.module_table_sz();
        
        table.put_module(2);
        let bytes2 = manager.module_table_sz();
        
        assert!(bytes2 > bytes1);
    }

    #[test]
    fn test_module_info() {
        let manager = ModuleTableManager::new();
        let table = manager.get_table(0);
        
        table.put_module(1);
        table.put_module(2);
        
        let info = manager.module_info(0);
        assert!(info.contains("code_ix=0"));
        assert!(info.contains("2 modules"));
    }

    #[test]
    fn test_old_code_locking() {
        let manager = ModuleTableManager::new();
        
        // Test read lock
        let _read_guard = manager.rlock_old_code(0);
        // Lock is held until guard is dropped
        
        // Test write lock
        let _write_guard = manager.rwlock_old_code(1);
        // Lock is held until guard is dropped
    }

    #[test]
    fn test_module_instance_fields() {
        let mut inst = ModuleInstance::default();
        
        // Test all fields are accessible
        inst.code_length = 100;
        inst.catches = 5;
        inst.num_breakpoints = 2;
        inst.num_traced_exports = 3;
        inst.unsealed = true;
        
        assert_eq!(inst.code_length, 100);
        assert_eq!(inst.catches, 5);
        assert_eq!(inst.num_breakpoints, 2);
        assert_eq!(inst.num_traced_exports, 3);
        assert!(inst.unsealed);
    }

    #[test]
    fn test_module_fields() {
        let mut module = Module {
            module: 42,
            seen: false,
            curr: ModuleInstance::default(),
            old: ModuleInstance::default(),
            on_load: None,
        };
        
        // Test all fields are accessible
        module.seen = true;
        module.curr.code_length = 100;
        module.old.code_length = 50;
        module.on_load = Some(ModuleInstance::default());
        
        assert_eq!(module.module, 42);
        assert!(module.seen);
        assert_eq!(module.curr.code_length, 100);
        assert_eq!(module.old.code_length, 50);
        assert!(module.on_load.is_some());
    }

    #[test]
    fn test_module_table_multiple_operations() {
        let table = ModuleTable::new(10, 100);
        
        // Add multiple modules
        for i in 1..=10 {
            table.put_module(i);
        }
        
        assert_eq!(table.size(), 10);
        
        // Get all modules
        for i in 1..=10 {
            let module = table.get_module(i);
            assert!(module.is_some());
            assert_eq!(module.unwrap().module, i);
        }
        
        // Get by index for all
        for i in 0..10 {
            let module = table.get_module_by_index(i);
            assert!(module.is_some());
        }
        
        // Clear and verify
        table.clear();
        assert_eq!(table.size(), 0);
        for i in 1..=10 {
            assert!(table.get_module(i).is_none());
        }
    }
}

