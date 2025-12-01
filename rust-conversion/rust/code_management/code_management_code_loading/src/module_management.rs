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
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

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

impl Default for ModuleInstance {
    fn default() -> Self {
        Self {
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
        }
    }
}

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

/// Module table manager - manages module tables for all code indices
pub struct ModuleTableManager {
    /// Module tables for each code index (3 indices total)
    tables: [ModuleTable; 3],
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
}

impl Default for ModuleTableManager {
    fn default() -> Self {
        Self::new()
    }
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
}

