//! Export Operations Module
//!
//! Provides comprehensive export functionality for managing callable functions in the
//! Erlang/OTP runtime system. Export entries represent functions that can be called,
//! identified by their MFA (Module, Function, Arity) tuple.
//!
//! ## Overview
//!
//! The export table is a fundamental data structure in the Erlang runtime that maps
//! function identifiers (MFA) to export entries. This enables efficient function
//! lookup and call resolution, supporting both regular functions and BIFs (Built-In
//! Functions).
//!
//! ## Key Concepts
//!
//! - **MFA (Module, Function, Arity)**: A tuple that uniquely identifies a function
//!   in the system. Consists of module atom index, function atom index, and arity.
//!
//! - **Export Entry**: Represents a callable function, containing the MFA, BIF number
//!   (if applicable), and metadata about the function (traced, stub, etc.).
//!
//! - **Export Table**: Thread-safe table that manages all export entries, providing
//!   efficient lookup by MFA hash.
//!
//! ## Features
//!
//! - **Function Registration**: Register functions with their MFA identifiers
//! - **BIF Support**: Special handling for Built-In Functions
//! - **Stub Entries**: Entries for functions that are referenced but not yet loaded
//! - **Thread-Safe**: All operations use `RwLock` for concurrent access
//! - **Efficient Lookup**: Hash-based lookup for O(1) average case performance
//!
//! ## Examples
//!
//! ```rust
//! use entities_io_operations::{ExportTable, Export, Mfa};
//!
//! // Create an export table
//! let table = ExportTable::new();
//!
//! // Create and register an export (module=1, function=2, arity=2)
//! let export = table.put(1, 2, 2);
//!
//! // Look up an export by MFA components
//! let found = table.get(1, 2, 2);
//!
//! // Or use MFA struct for convenience
//! let mfa = Mfa::new(1, 2, 2);
//! let found = table.get(mfa.module, mfa.function, mfa.arity);
//! ```
//!
//! ## See Also
//!
//! - [`atom`](../entities_data_handling/atom/index.html): Atom table used for module/function names

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
        (self.module as u64).wrapping_mul(self.function as u64) ^ (self.arity as u64)
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
    /// Whether this is a stub entry (represents code not yet loaded)
    pub is_stub: bool,
}

impl Export {
    /// Create a new export entry
    pub fn new(module: u32, function: u32, arity: u32) -> Self {
        Self {
            mfa: Mfa::new(module, function, arity),
            bif_number: -1,
            is_bif_traced: false,
            is_stub: false,
        }
    }

    /// Create a new BIF export entry
    pub fn new_bif(module: u32, function: u32, arity: u32, bif_number: i32) -> Self {
        Self {
            mfa: Mfa::new(module, function, arity),
            bif_number,
            is_bif_traced: false,
            is_stub: false,
        }
    }

    /// Create a new stub export entry
    ///
    /// Stub entries represent functions that are referenced but not yet loaded.
    /// Calling a stub will trigger an error handler.
    pub fn new_stub(module: u32, function: u32, arity: u32) -> Self {
        Self {
            mfa: Mfa::new(module, function, arity),
            bif_number: -1,
            is_bif_traced: false,
            is_stub: true,
        }
    }

    /// Check if this is a BIF
    pub fn is_bif(&self) -> bool {
        self.bif_number >= 0
    }

    /// Check if this is a stub entry
    ///
    /// Stub entries represent functions that are referenced but not yet loaded.
    /// When a stub is called, it triggers the error handler to attempt module loading.
    pub fn is_stub_entry(&self) -> bool {
        self.is_stub
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
    /// If a stub exists for this MFA, it will be replaced with a regular export.
    /// This is used when loading a module - stubs are upgraded to regular exports.
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Export entry (existing regular export, or newly created/upgraded from stub)
    pub fn put(&self, module: u32, function: u32, arity: u32) -> Export {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();

        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        // Check if already exists
        if let Some(existing) = exports.get(&hash) {
            // If it's already a regular export (not a stub), return it
            if !existing.is_stub {
                return existing.clone();
            }
            // If it's a stub, we need to replace it with a regular export
            // Remove the stub from the list first
            export_list.retain(|e| e.mfa != mfa);
        }

        // Create new regular export (or upgrade from stub)
        let export = Export::new(module, function, arity);

        // Check limit before inserting (only if we're adding a new entry, not replacing)
        let is_new_entry = !exports.contains_key(&hash);
        if is_new_entry && *size >= self.limit {
            // Limit reached and export doesn't exist - cannot add new entry
            // Return the export we would have created (caller should handle limit error)
            return export;
        }

        // Insert or replace the export
        if is_new_entry {
            *size += 1;
        }
        exports.insert(hash, export.clone());
        export_list.push(export.clone());

        export
    }

    /// Get existing export entry or create a stub entry
    ///
    /// Stub entries are used when a function is referenced but not yet loaded.
    /// Stubs represent code not yet loaded and will trigger an error handler if called.
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Export entry (existing or newly created stub)
    pub fn get_or_make_stub(&self, module: u32, function: u32, arity: u32) -> Export {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();

        // Check if already exists
        {
            let exports = self.exports.read().unwrap();
            if let Some(export) = exports.get(&hash) {
                return export.clone();
            }
        }

        // Create new stub export
        let stub = Export::new_stub(module, function, arity);
        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        // Check limit before inserting
        if *size >= self.limit {
            // At limit - check if this exact export already exists
            if let Some(existing) = exports.get(&hash) {
                return existing.clone();
            }
            // Limit reached and export doesn't exist - cannot add new stub
            // Return the stub we would have created (caller should handle limit error)
            return stub;
        }

        exports.insert(hash, stub.clone());
        export_list.push(stub.clone());
        *size += 1;

        stub
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
    /// Size in bytes of all export entries (excluding HashMap and Vec overhead)
    pub fn entry_bytes(&self) -> usize {
        // Calculate actual size of Export struct including alignment
        const ENTRY_SIZE: usize = std::mem::size_of::<Export>();
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

    /// Check if an export is a stub entry
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Some(true) if the export exists and is a stub, Some(false) if it exists and is not a stub, None if it doesn't exist
    pub fn is_stub(&self, module: u32, function: u32, arity: u32) -> Option<bool> {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();
        let exports = self.exports.read().unwrap();
        exports.get(&hash).map(|export| export.is_stub)
    }

    /// List all stub entries
    ///
    /// # Returns
    /// Vector of all stub export entries
    pub fn list_stubs(&self) -> Vec<Export> {
        let export_list = self.export_list.read().unwrap();
        export_list
            .iter()
            .filter(|export| export.is_stub)
            .cloned()
            .collect()
    }

    /// Get the number of stub entries
    ///
    /// # Returns
    /// Number of stub export entries
    pub fn stub_count(&self) -> usize {
        let export_list = self.export_list.read().unwrap();
        export_list.iter().filter(|export| export.is_stub).count()
    }

    /// Get the number of regular (non-stub) export entries
    ///
    /// # Returns
    /// Number of regular export entries
    pub fn regular_count(&self) -> usize {
        let export_list = self.export_list.read().unwrap();
        export_list.iter().filter(|export| !export.is_stub).count()
    }

    /// Remove all stub entries
    ///
    /// This removes all stub entries from the table, leaving only regular exports.
    /// This is useful for cleanup when modules are loaded and stubs are no longer needed.
    ///
    /// # Returns
    /// Number of stubs removed
    pub fn remove_all_stubs(&self) -> usize {
        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        let mut removed_count = 0;
        let stubs_to_remove: Vec<Mfa> = export_list
            .iter()
            .filter(|export| export.is_stub)
            .map(|export| export.mfa)
            .collect();

        for mfa in &stubs_to_remove {
            let hash = mfa.hash();
            if exports.remove(&hash).is_some() {
                removed_count += 1;
            }
        }

        export_list.retain(|e| !e.is_stub);
        *size -= removed_count;

        removed_count
    }

    /// Remove a stub entry
    ///
    /// This removes a stub entry if it exists and is a stub. Regular exports are not removed.
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// Some(Export) if removed (and was a stub), None if not found or not a stub
    pub fn remove_stub(&self, module: u32, function: u32, arity: u32) -> Option<Export> {
        let mfa = Mfa::new(module, function, arity);
        let hash = mfa.hash();

        let mut exports = self.exports.write().unwrap();
        let mut export_list = self.export_list.write().unwrap();
        let mut size = self.size.write().unwrap();

        // Check if it exists and is a stub
        if let Some(export) = exports.get(&hash) {
            if !export.is_stub {
                // Not a stub, don't remove
                return None;
            }
        } else {
            // Doesn't exist
            return None;
        }

        // Remove the stub
        if let Some(export) = exports.remove(&hash) {
            export_list.retain(|e| e.mfa != mfa);
            *size -= 1;
            Some(export)
        } else {
            None
        }
    }

    /// Check if an export exists and is a stub
    ///
    /// # Arguments
    /// * `module` - Module atom index
    /// * `function` - Function atom index
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// true if the export exists and is a stub, false otherwise
    pub fn contains_stub(&self, module: u32, function: u32, arity: u32) -> bool {
        self.is_stub(module, function, arity).unwrap_or(false)
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
        assert!(!export.is_stub);
    }

    #[test]
    fn test_export_bif() {
        let export = Export::new_bif(1, 2, 3, 42);
        assert_eq!(export.mfa.module, 1);
        assert_eq!(export.mfa.function, 2);
        assert_eq!(export.mfa.arity, 3);
        assert_eq!(export.bif_number, 42);
        assert!(export.is_bif());
        assert!(!export.is_stub);
    }

    #[test]
    fn test_export_stub() {
        let stub = Export::new_stub(1, 2, 3);
        assert_eq!(stub.mfa.module, 1);
        assert_eq!(stub.mfa.function, 2);
        assert_eq!(stub.mfa.arity, 3);
        assert_eq!(stub.bif_number, -1);
        assert!(!stub.is_bif());
        assert!(stub.is_stub);
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
        assert!(stub.is_stub);

        // Get existing - should return existing stub
        let existing = table.get_or_make_stub(1, 2, 3);
        assert_eq!(table.table_size(), 1);
        assert_eq!(stub.mfa, existing.mfa);
        assert!(existing.is_stub);
    }

    #[test]
    fn test_export_table_stub_vs_regular() {
        let table = ExportTable::new();

        // Create a stub
        let stub = table.get_or_make_stub(1, 2, 3);
        assert!(stub.is_stub);
        assert_eq!(table.table_size(), 1);

        // Create a regular export with same MFA - should replace the stub
        let regular = table.put(1, 2, 3);
        // Should replace stub with regular export
        assert_eq!(table.table_size(), 1); // Size unchanged (replaced, not added)
        assert!(!regular.is_stub); // Now a regular export

        // Verify the table now has a regular export, not a stub
        let retrieved = table.get(1, 2, 3).unwrap();
        assert!(!retrieved.is_stub);

        // Create a regular export with different MFA
        let regular2 = table.put(4, 5, 6);
        assert_eq!(table.table_size(), 2);
        assert!(!regular2.is_stub);
    }

    #[test]
    fn test_export_table_stub_replacement() {
        let table = ExportTable::new();

        // Create a stub for a function that will be loaded later
        let stub = table.get_or_make_stub(10, 20, 30);
        assert!(stub.is_stub);
        assert_eq!(table.table_size(), 1);

        // Simulate module loading - put() should replace stub with regular export
        let regular = table.put(10, 20, 30);
        assert!(!regular.is_stub);
        assert_eq!(table.table_size(), 1); // Still one entry (replaced)

        // Verify stub is gone and regular export is in place
        let retrieved = table.get(10, 20, 30).unwrap();
        assert!(!retrieved.is_stub);
        assert_eq!(retrieved.mfa.module, 10);
        assert_eq!(retrieved.mfa.function, 20);
        assert_eq!(retrieved.mfa.arity, 30);
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

        // Adding one more when at limit - should not increase size
        let export_at_limit = table.put(10, 110, 10);
        // The limit check prevents insertion, so size stays at 10
        assert_eq!(table.table_size(), 10);
        // The export was created but not inserted
        assert_eq!(export_at_limit.mfa.module, 10);
        // Verify it's not in the table
        assert!(!table.contains(10, 110, 10));

        // But we can still get existing exports at limit
        let _existing = table.put(0, 100, 0);
        assert_eq!(table.table_size(), 10);
        assert!(table.contains(0, 100, 0));
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

    #[test]
    fn test_export_is_stub_entry() {
        let regular = Export::new(1, 2, 3);
        let stub = Export::new_stub(1, 2, 3);

        assert!(!regular.is_stub_entry());
        assert!(stub.is_stub_entry());
    }

    #[test]
    fn test_export_table_is_stub() {
        let table = ExportTable::new();

        // Non-existent export
        assert_eq!(table.is_stub(1, 2, 3), None);

        // Regular export
        table.put(1, 2, 3);
        assert_eq!(table.is_stub(1, 2, 3), Some(false));

        // Stub export
        table.remove(1, 2, 3);
        table.get_or_make_stub(1, 2, 3);
        assert_eq!(table.is_stub(1, 2, 3), Some(true));
    }

    #[test]
    fn test_export_table_list_stubs() {
        let table = ExportTable::new();

        // Initially no stubs
        assert_eq!(table.list_stubs().len(), 0);

        // Add some regular exports
        table.put(1, 2, 3);
        table.put(4, 5, 6);

        // Add some stubs
        table.get_or_make_stub(10, 20, 30);
        table.get_or_make_stub(11, 21, 31);

        let stubs = table.list_stubs();
        assert_eq!(stubs.len(), 2);
        assert!(stubs.iter().all(|s| s.is_stub));
        assert!(stubs.iter().any(|s| s.mfa == Mfa::new(10, 20, 30)));
        assert!(stubs.iter().any(|s| s.mfa == Mfa::new(11, 21, 31)));
    }

    #[test]
    fn test_export_table_stub_count() {
        let table = ExportTable::new();

        assert_eq!(table.stub_count(), 0);

        // Add regular exports
        table.put(1, 2, 3);
        table.put(4, 5, 6);
        assert_eq!(table.stub_count(), 0);

        // Add stubs
        table.get_or_make_stub(10, 20, 30);
        assert_eq!(table.stub_count(), 1);

        table.get_or_make_stub(11, 21, 31);
        assert_eq!(table.stub_count(), 2);

        // Replace stub with regular export
        table.put(10, 20, 30);
        assert_eq!(table.stub_count(), 1);
    }

    #[test]
    fn test_export_table_regular_count() {
        let table = ExportTable::new();

        assert_eq!(table.regular_count(), 0);

        // Add regular exports
        table.put(1, 2, 3);
        assert_eq!(table.regular_count(), 1);

        table.put(4, 5, 6);
        assert_eq!(table.regular_count(), 2);

        // Add stubs (shouldn't affect regular count)
        table.get_or_make_stub(10, 20, 30);
        assert_eq!(table.regular_count(), 2);

        // Replace stub with regular export
        table.put(10, 20, 30);
        assert_eq!(table.regular_count(), 3);
    }

    #[test]
    fn test_export_table_remove_all_stubs() {
        let table = ExportTable::new();

        // Add mix of regular exports and stubs
        table.put(1, 2, 3);
        table.put(4, 5, 6);
        table.get_or_make_stub(10, 20, 30);
        table.get_or_make_stub(11, 21, 31);
        table.get_or_make_stub(12, 22, 32);

        assert_eq!(table.table_size(), 5);
        assert_eq!(table.stub_count(), 3);
        assert_eq!(table.regular_count(), 2);

        // Remove all stubs
        let removed = table.remove_all_stubs();
        assert_eq!(removed, 3);
        assert_eq!(table.table_size(), 2);
        assert_eq!(table.stub_count(), 0);
        assert_eq!(table.regular_count(), 2);

        // Regular exports should still exist
        assert!(table.contains(1, 2, 3));
        assert!(table.contains(4, 5, 6));

        // Stubs should be gone
        assert!(!table.contains(10, 20, 30));
        assert!(!table.contains(11, 21, 31));
        assert!(!table.contains(12, 22, 32));
    }

    #[test]
    fn test_export_table_remove_stub() {
        let table = ExportTable::new();

        // Add a stub
        table.get_or_make_stub(10, 20, 30);
        assert_eq!(table.table_size(), 1);
        assert!(table.contains_stub(10, 20, 30));

        // Remove the stub
        let removed = table.remove_stub(10, 20, 30);
        assert!(removed.is_some());
        assert!(removed.unwrap().is_stub);
        assert_eq!(table.table_size(), 0);
        assert!(!table.contains(10, 20, 30));

        // Try to remove non-existent stub
        let removed = table.remove_stub(10, 20, 30);
        assert!(removed.is_none());

        // Add a regular export and try to remove it as stub
        table.put(1, 2, 3);
        let removed = table.remove_stub(1, 2, 3);
        assert!(removed.is_none()); // Should not remove regular export
        assert!(table.contains(1, 2, 3)); // Should still exist
    }

    #[test]
    fn test_export_table_contains_stub() {
        let table = ExportTable::new();

        // Non-existent
        assert!(!table.contains_stub(1, 2, 3));

        // Regular export
        table.put(1, 2, 3);
        assert!(!table.contains_stub(1, 2, 3));

        // Stub export
        table.remove(1, 2, 3);
        table.get_or_make_stub(1, 2, 3);
        assert!(table.contains_stub(1, 2, 3));

        // Replace stub with regular
        table.put(1, 2, 3);
        assert!(!table.contains_stub(1, 2, 3));
    }

    #[test]
    fn test_export_table_stub_lifecycle() {
        let table = ExportTable::new();

        // Create stub for not-yet-loaded function
        let stub = table.get_or_make_stub(100, 200, 300);
        assert!(stub.is_stub);
        assert_eq!(table.stub_count(), 1);
        assert_eq!(table.regular_count(), 0);

        // Function gets loaded - replace stub with regular export
        let regular = table.put(100, 200, 300);
        assert!(!regular.is_stub);
        assert_eq!(table.stub_count(), 0);
        assert_eq!(table.regular_count(), 1);

        // Verify the export is now regular
        let retrieved = table.get(100, 200, 300).unwrap();
        assert!(!retrieved.is_stub);
        assert_eq!(retrieved.mfa, Mfa::new(100, 200, 300));
    }

    #[test]
    fn test_export_table_mixed_stubs_and_regulars() {
        let table = ExportTable::new();

        // Add mix
        table.put(1, 2, 3); // regular
        table.put(4, 5, 6); // regular
        table.get_or_make_stub(10, 20, 30); // stub
        table.get_or_make_stub(11, 21, 31); // stub
        table.put(7, 8, 9); // regular

        assert_eq!(table.table_size(), 5);
        assert_eq!(table.regular_count(), 3);
        assert_eq!(table.stub_count(), 2);

        // List should include all
        let all = table.list();
        assert_eq!(all.len(), 5);

        // List stubs should only include stubs
        let stubs = table.list_stubs();
        assert_eq!(stubs.len(), 2);
        assert!(stubs.iter().all(|s| s.is_stub));

        // Remove all stubs
        table.remove_all_stubs();
        assert_eq!(table.table_size(), 3);
        assert_eq!(table.regular_count(), 3);
        assert_eq!(table.stub_count(), 0);
    }
}
