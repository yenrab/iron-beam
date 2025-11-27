//! Export Operations
//!
//! Provides export functionality for Erlang terms.
//!
//! Export entries represent functions that can be called (MFA - Module, Function, Arity).
//! The export table manages all export entries in the system.

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
use std::sync::RwLock;

/// MFA (Module, Function, Arity) - uniquely identifies a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mfa {
    /// Module atom index
    pub module: u32,
    /// Function atom index
    pub function: u32,
    /// Function arity
    pub arity: u32,
}

impl Mfa {
    /// Create a new MFA
    pub fn new(module: u32, function: u32, arity: u32) -> Self {
        Self {
            module,
            function,
            arity,
        }
    }

    /// Compute hash value for MFA (matches C export_hash function)
    pub fn hash(&self) -> u64 {
        // C: (atom_val(module) * atom_val(function)) ^ arity
        (self.module as u64)
            .wrapping_mul(self.function as u64)
            ^ (self.arity as u64)
    }
}

/// Export entry representing a callable function
#[derive(Debug, Clone)]
pub struct Export {
    /// MFA (Module, Function, Arity)
    pub mfa: Mfa,
    /// BIF number (-1 if not a BIF)
    pub bif_number: i32,
    /// Whether this is a traced BIF
    pub is_bif_traced: bool,
}

impl Export {
    /// Create a new export entry
    pub fn new(module: u32, function: u32, arity: u32) -> Self {
        Self {
            mfa: Mfa::new(module, function, arity),
            bif_number: -1,
            is_bif_traced: false,
        }
    }

    /// Create a new BIF export entry
    pub fn new_bif(module: u32, function: u32, arity: u32, bif_number: i32) -> Self {
        Self {
            mfa: Mfa::new(module, function, arity),
            bif_number,
            is_bif_traced: false,
        }
    }

    /// Check if this is a BIF
    pub fn is_bif(&self) -> bool {
        self.bif_number >= 0
    }

    /// Compute hash value for export (based on MFA)
    pub fn hash(&self) -> u64 {
        self.mfa.hash()
    }
}

impl PartialEq for Export {
    fn eq(&self, other: &Self) -> bool {
        self.mfa == other.mfa
    }
}

impl Eq for Export {}

/// Export table for managing all export entries
pub struct ExportTable {
    /// Map from MFA hash to export entry
    exports: RwLock<HashMap<u64, Export>>,
    /// List of exports (for iteration)
    export_list: RwLock<Vec<Export>>,
    /// Current number of exports
    size: RwLock<usize>,
    /// Maximum number of exports
    limit: usize,
}

impl ExportTable {
    /// Initial size for export table (matches C EXPORT_INITIAL_SIZE)
    pub const INITIAL_SIZE: usize = 4000;
    /// Maximum size for export table (matches C EXPORT_LIMIT)
    pub const LIMIT: usize = 512 * 1024;

    /// Create a new export table
    pub fn new() -> Self {
        Self {
            exports: RwLock::new(HashMap::with_capacity(Self::INITIAL_SIZE)),
            export_list: RwLock::new(Vec::with_capacity(Self::INITIAL_SIZE)),
            size: RwLock::new(0),
            limit: Self::LIMIT,
        }
    }

    /// Create a new export table with custom limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            exports: RwLock::new(HashMap::with_capacity(Self::INITIAL_SIZE)),
            export_list: RwLock::new(Vec::with_capacity(Self::INITIAL_SIZE)),
            size: RwLock::new(0),
            limit,
        }
    }

    /// Get export entry for MFA, or None if not found
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Some(Export) if found, None otherwise
    pub fn get(&self, module: u32, function: u32, arity: u32) -> Option<Export> {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();
        let exports = self.exports.read().unwrap();
        exports.get(&hash).cloned()
    }

    /// Create or get export entry for MFA
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Export entry (existing or newly created)
    pub fn put(&self, module: u32, function: u32, arity: u32) -> Export {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();

        // Check if already exists
        {
            let exports = self.exports.read().unwrap();
            if let Some(export) = exports.get(&hash) {
                return export.clone();
            }
        }

        // Create new export
        let export = Export::new(module, function, arity);
        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        // Check limit
        if *size >= self.limit {
            // Return existing if at limit (shouldn't happen in practice)
            return exports.get(&hash).cloned().unwrap_or(export.clone());
        }

        exports.insert(hash, export.clone());
        export_list.push(export.clone());
        *size += 1;

        export
    }

    /// Get existing export entry or create a stub entry
    ///
    /// Stub entries are used when a function is referenced but not yet loaded.
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Export entry (existing or newly created stub)
    pub fn get_or_make_stub(&self, module: u32, function: u32, arity: u32) -> Export {
        // Same as put for now - in full implementation, stub would have special handling
        self.put(module, function, arity)
    }

    /// List all exports
    ///
    /// # Returns
    /// Vector of all export entries
    pub fn list(&self) -> Vec<Export> {
        let export_list = self.export_list.read().unwrap();
        export_list.clone()
    }

    /// Get the number of exports
    ///
    /// # Returns
    /// Number of export entries
    pub fn list_size(&self) -> usize {
        *self.size.read().unwrap()
    }

    /// Get the table size (number of entries)
    ///
    /// # Returns
    /// Number of export entries
    pub fn table_size(&self) -> usize {
        *self.size.read().unwrap()
    }

    /// Get the size in bytes of all export entries
    ///
    /// # Returns
    /// Approximate size in bytes
    pub fn entry_bytes(&self) -> usize {
        // Approximate size: MFA (12 bytes) + bif_number (4 bytes) + is_bif_traced (1 byte) + overhead
        const ENTRY_SIZE: usize = 20; // Approximate
        self.table_size() * ENTRY_SIZE
    }

    /// Remove an export entry
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Some(Export) if removed, None if not found
    pub fn remove(&self, module: u32, function: u32, arity: u32) -> Option<Export> {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();

        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        if let Some(export) = exports.remove(&hash) {
            // Remove from list
            export_list.retain(|e| e.mfa != mfa);
            *size -= 1;
            Some(export)
        } else {
            None
        }
    }

    /// Check if an export exists
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// true if exists, false otherwise
    pub fn contains(&self, module: u32, function: u32, arity: u32) -> bool {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();
        let exports = self.exports.read().unwrap();
        exports.contains_key(&hash)
    }

    /// Clear all exports
    pub fn clear(&self) {
        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();
        exports.clear();
        export_list.clear();
        *size = 0;
    }
}

impl Default for ExportTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Export operations - convenience functions
pub mod export_ops {
    use super::ExportTable;

    /// Create a new export table
    pub fn new_table() -> ExportTable {
        ExportTable::new()
    }

    /// Create a new export table with custom limit
    pub fn new_table_with_limit(limit: usize) -> ExportTable {
        ExportTable::with_limit(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfa_creation() {
        let mfa = Mfa::new(1, 2, 3);
        assert_eq!(mfa.module, 1);
        assert_eq!(mfa.function, 2);
        assert_eq!(mfa.arity, 3);
    }

    #[test]
    fn test_mfa_hash() {
        let mfa1 = Mfa::new(1, 2, 3);
        let mfa2 = Mfa::new(1, 2, 3);
        let mfa3 = Mfa::new(1, 2, 4);

        assert_eq!(mfa1.hash(), mfa2.hash());
        assert_ne!(mfa1.hash(), mfa3.hash());
    }

    #[test]
    fn test_mfa_equality() {
        let mfa1 = Mfa::new(1, 2, 3);
        let mfa2 = Mfa::new(1, 2, 3);
        let mfa3 = Mfa::new(1, 2, 4);

        assert_eq!(mfa1, mfa2);
        assert_ne!(mfa1, mfa3);
    }

    #[test]
    fn test_export_creation() {
        let export = Export::new(1, 2, 3);
        assert_eq!(export.mfa.module, 1);
        assert_eq!(export.mfa.function, 2);
        assert_eq!(export.mfa.arity, 3);
        assert_eq!(export.bif_number, -1);
        assert!(!export.is_bif());
    }

    #[test]
    fn test_export_bif() {
        let export = Export::new_bif(1, 2, 3, 42);
        assert_eq!(export.mfa.module, 1);
        assert_eq!(export.mfa.function, 2);
        assert_eq!(export.mfa.arity, 3);
        assert_eq!(export.bif_number, 42);
        assert!(export.is_bif());
    }

    #[test]
    fn test_export_table_new() {
        let table = ExportTable::new();
        assert_eq!(table.table_size(), 0);
        assert_eq!(table.list_size(), 0);
    }

    #[test]
    fn test_export_table_put() {
        let table = ExportTable::new();
        
        let export1 = table.put(1, 2, 3);
        assert_eq!(table.table_size(), 1);
        assert_eq!(export1.mfa.module, 1);
        assert_eq!(export1.mfa.function, 2);
        assert_eq!(export1.mfa.arity, 3);

        let export2 = table.put(4, 5, 6);
        assert_eq!(table.table_size(), 2);
        assert_ne!(export1.mfa, export2.mfa);
    }

    #[test]
    fn test_export_table_put_duplicate() {
        let table = ExportTable::new();
        
        let export1 = table.put(1, 2, 3);
        let export2 = table.put(1, 2, 3);
        
        assert_eq!(table.table_size(), 1);
        assert_eq!(export1.mfa, export2.mfa);
    }

    #[test]
    fn test_export_table_get() {
        let table = ExportTable::new();
        
        table.put(1, 2, 3);
        
        let export = table.get(1, 2, 3);
        assert!(export.is_some());
        let export = export.unwrap();
        assert_eq!(export.mfa.module, 1);
        assert_eq!(export.mfa.function, 2);
        assert_eq!(export.mfa.arity, 3);

        let missing = table.get(4, 5, 6);
        assert!(missing.is_none());
    }

    #[test]
    fn test_export_table_get_or_make_stub() {
        let table = ExportTable::new();
        
        // Get non-existent - should create stub
        let stub = table.get_or_make_stub(1, 2, 3);
        assert_eq!(table.table_size(), 1);
        assert_eq!(stub.mfa.module, 1);
        assert_eq!(stub.mfa.function, 2);
        assert_eq!(stub.mfa.arity, 3);

        // Get existing - should return existing
        let existing = table.get_or_make_stub(1, 2, 3);
        assert_eq!(table.table_size(), 1);
        assert_eq!(stub.mfa, existing.mfa);
    }

    #[test]
    fn test_export_table_list() {
        let table = ExportTable::new();
        
        table.put(1, 2, 3);
        table.put(4, 5, 6);
        table.put(7, 8, 9);

        let exports = table.list();
        assert_eq!(exports.len(), 3);
    }

    #[test]
    fn test_export_table_list_size() {
        let table = ExportTable::new();
        
        assert_eq!(table.list_size(), 0);
        
        table.put(1, 2, 3);
        assert_eq!(table.list_size(), 1);
        
        table.put(4, 5, 6);
        assert_eq!(table.list_size(), 2);
    }

    #[test]
    fn test_export_table_table_size() {
        let table = ExportTable::new();
        
        assert_eq!(table.table_size(), 0);
        
        table.put(1, 2, 3);
        assert_eq!(table.table_size(), 1);
    }

    #[test]
    fn test_export_table_entry_bytes() {
        let table = ExportTable::new();
        
        assert_eq!(table.entry_bytes(), 0);
        
        table.put(1, 2, 3);
        assert!(table.entry_bytes() > 0);
    }

    #[test]
    fn test_export_table_remove() {
        let table = ExportTable::new();
        
        table.put(1, 2, 3);
        table.put(4, 5, 6);
        
        assert_eq!(table.table_size(), 2);
        
        let removed = table.remove(1, 2, 3);
        assert!(removed.is_some());
        assert_eq!(table.table_size(), 1);
        
        let missing = table.remove(1, 2, 3);
        assert!(missing.is_none());
    }

    #[test]
    fn test_export_table_contains() {
        let table = ExportTable::new();
        
        assert!(!table.contains(1, 2, 3));
        
        table.put(1, 2, 3);
        assert!(table.contains(1, 2, 3));
        assert!(!table.contains(4, 5, 6));
    }

    #[test]
    fn test_export_table_clear() {
        let table = ExportTable::new();
        
        table.put(1, 2, 3);
        table.put(4, 5, 6);
        assert_eq!(table.table_size(), 2);
        
        table.clear();
        assert_eq!(table.table_size(), 0);
        assert_eq!(table.list().len(), 0);
    }

    #[test]
    fn test_export_table_different_arities() {
        let table = ExportTable::new();
        
        // Same module and function, different arities are different exports
        table.put(1, 2, 0);
        table.put(1, 2, 1);
        table.put(1, 2, 2);
        
        assert_eq!(table.table_size(), 3);
        assert!(table.contains(1, 2, 0));
        assert!(table.contains(1, 2, 1));
        assert!(table.contains(1, 2, 2));
    }

    #[test]
    fn test_export_table_different_modules() {
        let table = ExportTable::new();
        
        // Same function name, different modules are different exports
        table.put(1, 2, 3);
        table.put(4, 2, 3);
        
        assert_eq!(table.table_size(), 2);
        assert!(table.contains(1, 2, 3));
        assert!(table.contains(4, 2, 3));
    }

    #[test]
    fn test_export_table_with_limit() {
        let table = ExportTable::with_limit(10);
        
        // Add unique exports
        for i in 0..10 {
            table.put(i, i + 100, i); // Use different function values to ensure uniqueness
        }
        
        assert_eq!(table.table_size(), 10);
        
        // Adding one more when at limit - the current implementation allows it
        // (limit check happens but doesn't prevent insertion)
        table.put(10, 110, 10);
        // The limit check prevents insertion, so size stays at 10
        // In a full implementation, this might raise an error or handle differently
        assert_eq!(table.table_size(), 10);
    }

    #[test]
    fn test_export_equality() {
        let export1 = Export::new(1, 2, 3);
        let export2 = Export::new(1, 2, 3);
        let export3 = Export::new(1, 2, 4);

        assert_eq!(export1, export2);
        assert_ne!(export1, export3);
    }

    #[test]
    fn test_export_hash() {
        let export1 = Export::new(1, 2, 3);
        let export2 = Export::new(1, 2, 3);
        let export3 = Export::new(1, 2, 4);

        assert_eq!(export1.hash(), export2.hash());
        assert_ne!(export1.hash(), export3.hash());
    }

    #[test]
    fn test_export_convenience_functions() {
        use super::export_ops;
        let table1 = export_ops::new_table();
        assert_eq!(table1.table_size(), 0);

        let table2 = export_ops::new_table_with_limit(100);
        assert_eq!(table2.table_size(), 0);
    }

    #[test]
    fn test_export_table_default() {
        let table = ExportTable::default();
        assert_eq!(table.table_size(), 0);
    }
}
