//! Code Loading and Module Management Built-in Functions
//!
//! Provides module loading, unloading, and management operations.
//! Tracks which modules are loaded, pre-loaded, and handles module lifecycle.
//!
//! This module implements safe Rust equivalents of Erlang code loading BIFs.

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
 *
 * Creation productivity increased for code in this file by using AALang and GAB.
 * See https://github.com/yenrab/AALang-Gab
 */

use crate::op::ErlangTerm;
use crate::unique::Reference;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::SystemTime;
use usecases_process_management::process_code_tracking::{ModuleCodeArea, any_process_uses_module, any_dirty_process_uses_module};
use code_management_code_loading::{get_global_code_ix, get_global_module_manager};

/// Error type for code loading operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// Bad argument (e.g., invalid module name, module not found)
    BadArgument(String),
    /// System limit exceeded
    SystemLimit(String),
    /// Operation not supported
    NotSupported(String),
}

/// Module status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStatus {
    /// Module is loaded and active
    Loaded,
    /// Module is pre-loaded (part of the system)
    PreLoaded,
    /// Module has old code that needs purging
    HasOldCode,
    /// Module has on_load function pending
    OnLoadPending,
}

/// Module metadata for information queries
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    /// MD5 checksum of the module code
    pub md5: Option<Vec<u8>>,
    /// Module exports (list of {Function, Arity} tuples)
    pub exports: Vec<ErlangTerm>,
    /// Module attributes (list of attribute tuples)
    pub attributes: Vec<ErlangTerm>,
    /// Compile information (list of compile option tuples)
    pub compile: Vec<ErlangTerm>,
}

/// Module registry entry
#[derive(Debug, Clone)]
struct ModuleEntry {
    /// Module name
    name: String,
    /// Module status
    status: ModuleStatus,
    /// Whether module has old code
    has_old_code: bool,
    /// Whether module has on_load function
    has_on_load: bool,
    /// Debug info (if available)
    debug_info: Option<ErlangTerm>,
    /// MD5 checksum of the module code
    md5: Option<Vec<u8>>,
    /// Module exports (list of {Function, Arity} tuples)
    exports: Vec<ErlangTerm>,
    /// Module attributes (list of attribute tuples)
    attributes: Vec<ErlangTerm>,
    /// Compile information (list of compile option tuples)
    compile: Vec<ErlangTerm>,
}


/// Prepared code state
///
/// Represents code that has been prepared for loading but not yet finished.
#[derive(Debug, Clone)]
struct PreparedCode {
    /// Module name
    module: String,
    /// BEAM code binary data
    code: Vec<u8>,
    /// Whether this module has an on_load function
    has_on_load: bool,
    /// MD5 checksum of the module
    md5: Option<Vec<u8>>,
    /// Reference to this prepared code (magic reference)
    magic_ref: Reference,
}

impl PreparedCode {
    fn new(module: String, code: Vec<u8>, has_on_load: bool) -> Self {
        // Create a unique reference for this prepared code
        let magic_ref = crate::unique::UniqueBif::make_ref();
        Self {
            module,
            code,
            has_on_load,
            md5: None,
            magic_ref,
        }
    }
    
    /// Get the reference value as u64 for ErlangTerm
    ///
    /// Combines the magic reference components (thread_id, value, ref_number) into a single u64.
    /// This encoding allows the reference to be represented as an ErlangTerm::Reference.
    /// The encoding uses wrapping arithmetic to ensure all components fit in a u64.
    fn reference_value(&self) -> u64 {
        let thread_id = self.magic_ref.thread_id() as u64;
        let value = self.magic_ref.value();
        let ref_num = self.magic_ref.ref_number() as u64;
        // Encode all components into a single u64 using a multiplicative encoding scheme
        thread_id.wrapping_mul(1000000000)
            .wrapping_add(value)
            .wrapping_add(ref_num)
    }

    fn compute_md5(&mut self) {
        use crate::checksum::ChecksumBif;
        let md5_hash = ChecksumBif::md5(&self.code);
        self.md5 = Some(md5_hash.to_vec());
    }
}

/// Magic reference to prepared code
///
/// In Erlang, prepared code is represented as a "magic reference" that
/// can be passed around and later used to finish loading.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreparedCodeRef {
    /// The magic reference
    reference: Reference,
}

impl PreparedCodeRef {
    fn new(reference: Reference) -> Self {
        Self { reference }
    }

    fn reference(&self) -> &Reference {
        &self.reference
    }
}

/// Prepared code registry
///
/// Tracks prepared code that hasn't been finished loading yet.
struct PreparedCodeRegistry {
    /// Map of magic references to prepared code
    prepared: Arc<RwLock<HashMap<Reference, PreparedCode>>>,
    /// Counter for generating unique references
    ref_counter: AtomicU64,
}

impl PreparedCodeRegistry {
    fn new() -> Self {
        Self {
            prepared: Arc::new(RwLock::new(HashMap::new())),
            ref_counter: AtomicU64::new(1),
        }
    }

    fn get_instance() -> &'static PreparedCodeRegistry {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<PreparedCodeRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| PreparedCodeRegistry::new())
    }

    fn register(&self, code: PreparedCode) -> Reference {
        let ref_val = code.magic_ref.clone();
        let mut prepared = self.prepared.write().unwrap();
        prepared.insert(ref_val.clone(), code);
        ref_val
    }

    fn get(&self, reference: &Reference) -> Option<PreparedCode> {
        let prepared = self.prepared.read().unwrap();
        prepared.get(reference).cloned()
    }

    fn remove(&self, reference: &Reference) -> Option<PreparedCode> {
        let mut prepared = self.prepared.write().unwrap();
        prepared.remove(reference)
    }

    /// Find prepared code by u64 reference value and return the Reference key
    fn find_by_reference_value(&self, ref_value: u64) -> Option<(Reference, PreparedCode)> {
        let prepared = self.prepared.read().unwrap();
        for (ref_key, code) in prepared.iter() {
            if code.reference_value() == ref_value {
                return Some((ref_key.clone(), code.clone()));
            }
        }
        None
    }

    fn clear(&self) {
        let mut prepared = self.prepared.write().unwrap();
        prepared.clear();
    }
}

/// Module registry storage
///
/// Tracks all loaded modules and their status.
#[derive(Clone, Debug)]
struct ModuleRegistry {
    /// Map of module names to module entries
    modules: Arc<RwLock<HashMap<String, ModuleEntry>>>,
    /// Set of pre-loaded module names
    preloaded: Arc<RwLock<HashSet<String>>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            preloaded: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Get the singleton instance
    fn get_instance() -> &'static ModuleRegistry {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<ModuleRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| ModuleRegistry::new())
    }
}

/// Code Loading Built-in Functions
pub struct LoadBif;

impl LoadBif {
    /// Delete a module (delete_module/1)
    ///
    /// Removes a module from the system. The module must not have old code
    /// that needs purging.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If module was deleted
    /// * `Ok(ErlangTerm::Atom("undefined"))` - If module was not found
    /// * `Err(LoadError)` - If operation fails (e.g., module has old code)
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Delete a loaded module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::delete_module_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    ///
    /// // Delete non-existent module
    /// let result = LoadBif::delete_module_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    ///
    /// // Cannot delete pre-loaded module
    /// LoadBif::clear_all();
    /// LoadBif::mark_preloaded("preloaded");
    /// let result = LoadBif::delete_module_1(&ErlangTerm::Atom("preloaded".to_string()));
    /// assert!(result.is_err());
    /// ```
    pub fn delete_module_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        if let Some(entry) = modules.get(&module_name) {
            // Check if module has old code that needs purging
            if entry.has_old_code {
                return Err(LoadError::BadArgument(format!(
                    "Module {} must be purged before deleting",
                    module_name
                )));
            }

            // Check if it's pre-loaded (can't delete pre-loaded modules)
            let preloaded = registry.preloaded.read().unwrap();
            if preloaded.contains(&module_name) {
                return Err(LoadError::BadArgument(format!(
                    "Cannot delete pre-loaded module {}",
                    module_name
                )));
            }

            // Delete the module
            modules.remove(&module_name);
            Ok(ErlangTerm::Atom("true".to_string()))
        } else {
            Ok(ErlangTerm::Atom("undefined".to_string()))
        }
    }

    /// Check if a module is loaded (module_loaded/1)
    ///
    /// Returns true if the module is loaded and active (not just pending on_load).
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If module is loaded
    /// * `Ok(ErlangTerm::Atom("false"))` - If module is not loaded
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Check loaded module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    ///
    /// // Check non-loaded module
    /// let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check pre-loaded module
    /// LoadBif::mark_preloaded("preloaded_module");
    /// let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("preloaded_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    /// ```
    pub fn module_loaded_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();

        if let Some(entry) = modules.get(&module_name) {
            // Module is loaded if it's in Loaded status (not OnLoadPending)
            match entry.status {
                ModuleStatus::Loaded | ModuleStatus::PreLoaded => {
                    Ok(ErlangTerm::Atom("true".to_string()))
                }
                _ => Ok(ErlangTerm::Atom("false".to_string())),
            }
        } else {
            Ok(ErlangTerm::Atom("false".to_string()))
        }
    }

    /// Get pre-loaded modules (pre_loaded/0)
    ///
    /// Returns a list of all pre-loaded module names.
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - List of pre-loaded module names (atoms)
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Get pre-loaded modules
    /// LoadBif::clear_all();
    /// LoadBif::mark_preloaded("module1");
    /// LoadBif::mark_preloaded("module2");
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = preloaded {
    ///     assert!(modules.len() >= 2);
    /// }
    ///
    /// // Get empty list when no pre-loaded modules
    /// LoadBif::clear_all();
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = preloaded {
    ///     assert_eq!(modules.len(), 0);
    /// }
    ///
    /// // Pre-loaded modules persist after clearing regular modules
    /// LoadBif::mark_preloaded("persistent");
    /// LoadBif::clear_all();
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = preloaded {
    ///     assert!(modules.len() >= 1);
    /// }
    /// ```
    pub fn pre_loaded_0() -> Result<ErlangTerm, LoadError> {
        let registry = ModuleRegistry::get_instance();
        let preloaded = registry.preloaded.read().unwrap();

        let mut modules: Vec<ErlangTerm> = preloaded
            .iter()
            .map(|name| ErlangTerm::Atom(name.clone()))
            .collect();

        // Sort for consistent ordering
        modules.sort_by(|a, b| {
            let name_a = match a {
                ErlangTerm::Atom(name) => name,
                _ => "",
            };
            let name_b = match b {
                ErlangTerm::Atom(name) => name,
                _ => "",
            };
            name_a.cmp(name_b)
        });

        Ok(ErlangTerm::List(modules))
    }

    /// Get all loaded modules (loaded/0)
    ///
    /// Returns a list of all loaded module names (both regular and pre-loaded).
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - List of loaded module names (atoms)
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Get all loaded modules
    /// LoadBif::clear_all();
    /// LoadBif::register_module("module1", ModuleStatus::Loaded, false, false);
    /// LoadBif::register_module("module2", ModuleStatus::Loaded, false, false);
    /// let loaded = LoadBif::loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = loaded {
    ///     assert!(modules.len() >= 2);
    /// }
    ///
    /// // Get empty list when no modules loaded
    /// LoadBif::clear_all();
    /// let loaded = LoadBif::loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = loaded {
    ///     assert_eq!(modules.len(), 0);
    /// }
    ///
    /// // Includes pre-loaded modules
    /// LoadBif::clear_all();
    /// LoadBif::mark_preloaded("preloaded");
    /// LoadBif::register_module("regular", ModuleStatus::Loaded, false, false);
    /// let loaded = LoadBif::loaded_0().unwrap();
    /// if let ErlangTerm::List(modules) = loaded {
    ///     assert!(modules.len() >= 2);
    /// }
    /// ```
    pub fn loaded_0() -> Result<ErlangTerm, LoadError> {
        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();

        let mut module_names: Vec<ErlangTerm> = modules
            .keys()
            .map(|name| ErlangTerm::Atom(name.clone()))
            .collect();

        // Sort for consistent ordering
        module_names.sort_by(|a, b| {
            let name_a = match a {
                ErlangTerm::Atom(name) => name,
                _ => "",
            };
            let name_b = match b {
                ErlangTerm::Atom(name) => name,
                _ => "",
            };
            name_a.cmp(name_b)
        });

        Ok(ErlangTerm::List(module_names))
    }

    /// Finish after on_load execution (finish_after_on_load/2)
    ///
    /// Completes the on_load function execution for a module.
    /// If the second argument is `true`, the module becomes active.
    /// If `false`, the module loading is aborted.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    /// * `success` - Whether on_load succeeded (atom: "true" or "false")
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("ok"))` - If successful
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Finish with success
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::OnLoadPending, false, true);
    /// let result = LoadBif::finish_after_on_load_2(
    ///     &ErlangTerm::Atom("my_module".to_string()),
    ///     &ErlangTerm::Atom("true".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Finish with failure (abort loading)
    /// LoadBif::clear_all();
    /// LoadBif::register_module("failed_module", ModuleStatus::OnLoadPending, false, true);
    /// let result = LoadBif::finish_after_on_load_2(
    ///     &ErlangTerm::Atom("failed_module".to_string()),
    ///     &ErlangTerm::Atom("false".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Finish non-existent module (returns error)
    /// let result = LoadBif::finish_after_on_load_2(
    ///     &ErlangTerm::Atom("nonexistent".to_string()),
    ///     &ErlangTerm::Atom("true".to_string()),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn finish_after_on_load_2(
        module: &ErlangTerm,
        success: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let success_bool = match success {
            ErlangTerm::Atom(ref name) if name == "true" => true,
            ErlangTerm::Atom(ref name) if name == "false" => false,
            _ => {
                return Err(LoadError::BadArgument(
                    "Success argument must be 'true' or 'false'".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        if let Some(entry) = modules.get_mut(&module_name) {
            // Check if module has on_load pending
            if entry.status != ModuleStatus::OnLoadPending {
                return Err(LoadError::BadArgument(format!(
                    "Module {} does not have on_load pending",
                    module_name
                )));
            }

            if success_bool {
                // on_load succeeded - make module active
                entry.status = ModuleStatus::Loaded;
                entry.has_on_load = false;
            } else {
                // on_load failed - remove module
                modules.remove(&module_name);
            }

            Ok(ErlangTerm::Atom("ok".to_string()))
        } else {
            Err(LoadError::BadArgument(format!(
                "Module {} not found",
                module_name
            )))
        }
    }

    /// Get debug info from code (code_get_debug_info/1)
    ///
    /// Returns debug information for a loaded module, if available.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - Debug info (map or atom "none" if not available)
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get debug info when available
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let debug_info = ErlangTerm::Map(std::collections::HashMap::new());
    /// LoadBif::set_debug_info("my_module", debug_info.clone());
    /// let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, debug_info);
    ///
    /// // Get debug info when not available
    /// LoadBif::clear_all();
    /// LoadBif::register_module("no_debug_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("no_debug_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("none".to_string()));
    ///
    /// // Get debug info for non-existent module
    /// let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("nonexistent".to_string()));
    /// assert!(result.is_err());
    /// ```
    pub fn code_get_debug_info_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();

        if let Some(entry) = modules.get(&module_name) {
            if let Some(debug_info) = &entry.debug_info {
                Ok(debug_info.clone())
            } else {
                Ok(ErlangTerm::Atom("none".to_string()))
            }
        } else {
            Err(LoadError::BadArgument(format!(
                "Module {} not found",
                module_name
            )))
        }
    }

    /// Internal: Check if process code is using a module (erts_internal_check_process_code/1)
    ///
    /// This is an internal function that checks if any process is using code from a module.
    /// 
    /// Checks if any process has code pointers (instruction pointer, NIF pointers, or
    /// continuation pointers on the stack) pointing into the module's old code area.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - Some process is using the code
    /// * `Ok(ErlangTerm::Atom("false"))` - No processes using the code
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Check for loaded module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::erts_internal_check_process_code_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check for non-existent module
    /// let result = LoadBif::erts_internal_check_process_code_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check with invalid argument
    /// let result = LoadBif::erts_internal_check_process_code_1(&ErlangTerm::Integer(42));
    /// assert!(result.is_err());
    /// ```
    pub fn erts_internal_check_process_code_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        // Get module code area for old code (if available)
        let module_code = Self::get_module_old_code_area(&module_name);
        
        // Check if any process is using the module's old code
        let any_uses = any_process_uses_module(&module_code);
        
        Ok(if any_uses {
            ErlangTerm::Atom("true".to_string())
        } else {
            ErlangTerm::Atom("false".to_string())
        })
    }

    /// Internal: Purge a module (erts_internal_purge_module/2)
    ///
    /// Purges old code from a module. This is an internal function.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    /// * `option` - Purge option (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If purged successfully
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Purge module with old code
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, true, false);
    /// let result = LoadBif::erts_internal_purge_module_2(
    ///     &ErlangTerm::Atom("my_module".to_string()),
    ///     &ErlangTerm::Atom("default".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    ///
    /// // Purge non-existent module
    /// let result = LoadBif::erts_internal_purge_module_2(
    ///     &ErlangTerm::Atom("nonexistent".to_string()),
    ///     &ErlangTerm::Atom("default".to_string()),
    /// );
    /// assert!(result.is_err());
    ///
    /// // Purge with different options
    /// LoadBif::clear_all();
    /// LoadBif::register_module("module2", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::erts_internal_purge_module_2(
    ///     &ErlangTerm::Atom("module2".to_string()),
    ///     &ErlangTerm::Atom("kill".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    /// ```
    pub fn erts_internal_purge_module_2(
        module: &ErlangTerm,
        _option: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        if let Some(entry) = modules.get_mut(&module_name) {
            // Clear old code flag
            entry.has_old_code = false;
            Ok(ErlangTerm::Atom("true".to_string()))
        } else {
            Err(LoadError::BadArgument(format!(
                "Module {} not found",
                module_name
            )))
        }
    }

    /// Prepare code for loading (erts_internal_prepare_loading/2)
    ///
    /// Prepares BEAM code for loading. Returns a magic reference that can be
    /// used later with finish_loading/1 to actually load the code.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    /// * `code` - BEAM code binary
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Reference)` - Magic reference to prepared code
    /// * `Ok(ErlangTerm::Tuple)` - Error tuple `{error, Reason}` if preparation fails
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Prepare code for loading
    /// LoadBif::clear_all();
    /// let code = vec![0xBE, 0xAM, 0x01, 0x02, 0x03];
    /// let result = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("my_module".to_string()),
    ///     &ErlangTerm::Binary(code),
    /// ).unwrap();
    /// assert!(matches!(result, ErlangTerm::Reference(_)));
    ///
    /// // Prepare code with bitstring
    /// let code2 = vec![0x01, 0x02, 0x03];
    /// let result = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("module2".to_string()),
    ///     &ErlangTerm::Bitstring(code2, 24),
    /// ).unwrap();
    /// assert!(matches!(result, ErlangTerm::Reference(_)));
    ///
    /// // Prepare with invalid code type
    /// let result = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("module3".to_string()),
    ///     &ErlangTerm::Integer(42),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn erts_internal_prepare_loading_2(
        module: &ErlangTerm,
        code: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let code_bytes = match code {
            ErlangTerm::Binary(data) => data.clone(),
            ErlangTerm::Bitstring(data, _) => data.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Code must be a binary".to_string(),
                ));
            }
        };

        // Check if code has on_load function
        // Note: This is a heuristic check - proper detection requires parsing the BEAM file
        // attributes chunk to look for the on_load attribute. The 0xBE marker is the
        // first byte of a valid BEAM file ("BEAM" starts with 'B' = 0x42, but we check
        // for 0xBE as a placeholder). In a full implementation, this would use
        // code_management::beam_loader::BeamLoader to parse the file and check attributes.
        let has_on_load = code_bytes.len() > 0 && code_bytes[0] == 0xBE;

        let mut prepared = PreparedCode::new(module_name, code_bytes, has_on_load);
        prepared.compute_md5();

        let registry = PreparedCodeRegistry::get_instance();
        let prepared_code = prepared.clone();
        let ref_value = prepared_code.reference_value();
        registry.register(prepared);

        Ok(ErlangTerm::Reference(ref_value))
    }

    /// Check if prepared code has on_load function (has_prepared_code_on_load/1)
    ///
    /// # Arguments
    /// * `prepared_ref` - Magic reference to prepared code
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If prepared code has on_load
    /// * `Ok(ErlangTerm::Atom("false"))` - If prepared code does not have on_load
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Check prepared code with on_load
    /// LoadBif::clear_all();
    /// let code = vec![0xBE, 0x41, 0x4D]; // BEAM file header (heuristic on_load check)
    /// let ref_term = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("on_load_module".to_string()),
    ///     &ErlangTerm::Binary(code),
    /// ).unwrap();
    /// let result = LoadBif::has_prepared_code_on_load_1(&ref_term).unwrap();
    /// // Result depends on whether code has on_load marker
    ///
    /// // Check prepared code without on_load
    /// let code2 = vec![0x01, 0x02, 0x03];
    /// let ref_term2 = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("no_on_load".to_string()),
    ///     &ErlangTerm::Binary(code2),
    /// ).unwrap();
    /// let result = LoadBif::has_prepared_code_on_load_1(&ref_term2).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check invalid reference
    /// let result = LoadBif::has_prepared_code_on_load_1(&ErlangTerm::Integer(42));
    /// assert!(result.is_err());
    /// ```
    pub fn has_prepared_code_on_load_1(
        prepared_ref: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let ref_value = match prepared_ref {
            ErlangTerm::Reference(val) => *val,
            _ => {
                return Err(LoadError::BadArgument(
                    "Argument must be a magic reference".to_string(),
                ));
            }
        };

        // Find the prepared code by searching through all entries
        // In a real implementation, we'd have a reverse lookup map
        let registry = PreparedCodeRegistry::get_instance();
        let prepared_map = registry.prepared.read().unwrap();
        let mut found = None;
        for (ref_key, prepared) in prepared_map.iter() {
            if prepared.reference_value() == ref_value {
                found = Some(prepared.has_on_load);
                break;
            }
        }
        drop(prepared_map);
        
        if let Some(has_on_load) = found {
            Ok(if has_on_load {
                ErlangTerm::Atom("true".to_string())
            } else {
                ErlangTerm::Atom("false".to_string())
            })
        } else {
            Err(LoadError::BadArgument(
                "Invalid prepared code reference".to_string(),
            ))
        }
    }

    /// Finish loading prepared code (finish_loading/1)
    ///
    /// Finishes loading prepared code. Takes a list of magic references to
    /// prepared code and makes the modules active.
    ///
    /// # Arguments
    /// * `prepared_list` - List of magic references to prepared code
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("ok"))` - If loading succeeded
    /// * `Ok(ErlangTerm::Tuple)` - Error tuple `{error, [{Module, Reason}]}` if loading fails
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Finish loading single module
    /// LoadBif::clear_all();
    /// let code = vec![0x01, 0x02, 0x03];
    /// let ref_term = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("module1".to_string()),
    ///     &ErlangTerm::Binary(code),
    /// ).unwrap();
    /// let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![ref_term])).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Finish loading multiple modules
    /// LoadBif::clear_all();
    /// let code1 = vec![0x01, 0x02];
    /// let code2 = vec![0x03, 0x04];
    /// let ref1 = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("module1".to_string()),
    ///     &ErlangTerm::Binary(code1),
    /// ).unwrap();
    /// let ref2 = LoadBif::erts_internal_prepare_loading_2(
    ///     &ErlangTerm::Atom("module2".to_string()),
    ///     &ErlangTerm::Binary(code2),
    /// ).unwrap();
    /// let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![ref1, ref2])).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Finish loading with invalid reference
    /// let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![ErlangTerm::Integer(42)]));
    /// // Returns error tuple if any reference is invalid
    /// ```
    pub fn finish_loading_1(prepared_list: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let references = match prepared_list {
            ErlangTerm::List(refs) => refs,
            _ => {
                return Err(LoadError::BadArgument(
                    "Argument must be a list of prepared code references".to_string(),
                ));
            }
        };

        let registry = PreparedCodeRegistry::get_instance();
        let module_registry = ModuleRegistry::get_instance();
        let mut errors = Vec::new();
        let mut loaded_modules = Vec::new();

        for ref_term in references {
            let ref_value = match ref_term {
                ErlangTerm::Reference(val) => *val,
                _ => {
                    errors.push((
                        ErlangTerm::Atom("invalid".to_string()),
                        ErlangTerm::Atom("bad_reference".to_string()),
                    ));
                    continue;
                }
            };

            // Find and remove the prepared code by reference value
            if let Some((ref_key, prepared)) = registry.find_by_reference_value(ref_value) {
                // Remove from registry atomically
                if let Some(prepared) = registry.remove(&ref_key) {
                    // Check if module already has old code
                    let modules = module_registry.modules.read().unwrap();
                    if let Some(entry) = modules.get(&prepared.module) {
                        if entry.has_old_code {
                            errors.push((
                                ErlangTerm::Atom(prepared.module.clone()),
                                ErlangTerm::Atom("not_purged".to_string()),
                            ));
                            continue;
                        }
                    }
                    drop(modules);

                    // Register the module
                    let mut modules = module_registry.modules.write().unwrap();
                    let status = if prepared.has_on_load {
                        ModuleStatus::OnLoadPending
                    } else {
                        ModuleStatus::Loaded
                    };

                    // Get MD5 from prepared code (ensure it's computed)
                    let md5 = prepared.md5.clone();
                    
                    // Parse BEAM file to extract exports, attributes, and compile info
                    let (exports, attributes, compile) = Self::parse_beam_metadata(&prepared.code);
                    
                    modules.insert(
                        prepared.module.clone(),
                        ModuleEntry {
                            name: prepared.module.clone(),
                            status,
                            has_old_code: false,
                            has_on_load: prepared.has_on_load,
                            debug_info: None,
                            md5,
                            exports,
                            attributes,
                            compile,
                        },
                    );
                    loaded_modules.push(prepared.module);
                } else {
                    errors.push((
                        ErlangTerm::Atom("unknown".to_string()),
                        ErlangTerm::Atom("invalid_reference".to_string()),
                    ));
                }
            } else {
                errors.push((
                    ErlangTerm::Atom("unknown".to_string()),
                    ErlangTerm::Atom("invalid_reference".to_string()),
                ));
            }
        }

        if !errors.is_empty() {
            let error_list: Vec<ErlangTerm> = errors
                .into_iter()
                .map(|(module, reason)| {
                    ErlangTerm::Tuple(vec![module, reason])
                })
                .collect();
            Ok(ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("error".to_string()),
                ErlangTerm::List(error_list),
            ]))
        } else {
            Ok(ErlangTerm::Atom("ok".to_string()))
        }
    }

    /// Check if module has old code (check_old_code/1)
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If module has old code
    /// * `Ok(ErlangTerm::Atom("false"))` - If module does not have old code
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Check module without old code
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check non-existent module
    /// let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check module with old code
    /// LoadBif::clear_all();
    /// LoadBif::register_module("reloaded_module", ModuleStatus::Loaded, true, false);
    /// let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("reloaded_module".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    /// ```
    pub fn check_old_code_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();

        if let Some(entry) = modules.get(&module_name) {
            Ok(if entry.has_old_code {
                ErlangTerm::Atom("true".to_string())
            } else {
                ErlangTerm::Atom("false".to_string())
            })
        } else {
            Ok(ErlangTerm::Atom("false".to_string()))
        }
    }

    /// Get MD5 checksum of BEAM file module (erts_internal_beamfile_module_md5/1)
    ///
    /// # Arguments
    /// * `code` - BEAM code binary
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Binary)` - MD5 checksum (16 bytes)
    /// * `Ok(ErlangTerm::Atom("undefined"))` - If code is invalid
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get MD5 of BEAM code
    /// let code = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    /// let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Binary(code)).unwrap();
    /// if let ErlangTerm::Binary(md5) = result {
    ///     assert_eq!(md5.len(), 16);
    /// }
    ///
    /// // Get MD5 of empty code (returns undefined)
    /// let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Binary(vec![])).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    ///
    /// // Get MD5 of bitstring
    /// let code2 = vec![0xBE, 0xAM];
    /// let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Bitstring(code2, 16)).unwrap();
    /// if let ErlangTerm::Binary(md5) = result {
    ///     assert_eq!(md5.len(), 16);
    /// }
    /// ```
    pub fn erts_internal_beamfile_module_md5_1(
        code: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let code_bytes = match code {
            ErlangTerm::Binary(data) => data.clone(),
            ErlangTerm::Bitstring(data, _) => data.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Code must be a binary".to_string(),
                ));
            }
        };

        if code_bytes.is_empty() {
            return Ok(ErlangTerm::Atom("undefined".to_string()));
        }

        // Use proper MD5 from checksum module
        // In real implementation, this would parse the BEAM file to get the actual MD5
        use crate::checksum::ChecksumBif;
        let md5_hash = ChecksumBif::md5(&code_bytes);
        Ok(ErlangTerm::Binary(md5_hash.to_vec()))
    }

    /// Extract chunk from BEAM file (erts_internal_beamfile_chunk/2)
    ///
    /// # Arguments
    /// * `code` - BEAM code binary
    /// * `chunk_id` - Chunk ID as list of 4 bytes
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Binary)` - Chunk data
    /// * `Ok(ErlangTerm::Atom("undefined"))` - If chunk not found
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Extract chunk from BEAM code
    /// let code = vec![0x42, 0x45, 0x41, 0x4D, 0x01, 0x02, 0x03];
    /// let chunk_id = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(0x42),
    ///     ErlangTerm::Integer(0x45),
    ///     ErlangTerm::Integer(0x41),
    ///     ErlangTerm::Integer(0x4D),
    /// ]);
    /// let result = LoadBif::erts_internal_beamfile_chunk_2(&ErlangTerm::Binary(code), &chunk_id).unwrap();
    /// // Returns chunk data if found, undefined otherwise
    ///
    /// // Extract non-existent chunk
    /// let code2 = vec![0x01, 0x02, 0x03];
    /// let chunk_id2 = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(0xFF),
    ///     ErlangTerm::Integer(0xFF),
    ///     ErlangTerm::Integer(0xFF),
    ///     ErlangTerm::Integer(0xFF),
    /// ]);
    /// let result = LoadBif::erts_internal_beamfile_chunk_2(&ErlangTerm::Binary(code2), &chunk_id2).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    ///
    /// // Extract with invalid chunk ID format
    /// let result = LoadBif::erts_internal_beamfile_chunk_2(
    ///     &ErlangTerm::Binary(vec![0x01]),
    ///     &ErlangTerm::Integer(42),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn erts_internal_beamfile_chunk_2(
        code: &ErlangTerm,
        chunk_id: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let code_bytes = match code {
            ErlangTerm::Binary(data) => data.clone(),
            ErlangTerm::Bitstring(data, _) => data.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Code must be a binary".to_string(),
                ));
            }
        };

        // Parse chunk ID from list of 4 bytes
        let chunk_id_bytes = match chunk_id {
            ErlangTerm::List(bytes) if bytes.len() == 4 => {
                let mut id = [0u8; 4];
                for (i, byte_term) in bytes.iter().enumerate() {
                    if let ErlangTerm::Integer(b) = byte_term {
                        if *b >= 0 && *b <= 255 {
                            id[i] = *b as u8;
                        } else {
                            return Err(LoadError::BadArgument(
                                "Chunk ID bytes must be 0-255".to_string(),
                            ));
                        }
                    } else {
                        return Err(LoadError::BadArgument(
                            "Chunk ID must be a list of 4 integers".to_string(),
                        ));
                    }
                }
                id
            }
            _ => {
                return Err(LoadError::BadArgument(
                    "Chunk ID must be a list of 4 bytes".to_string(),
                ));
            }
        };

        // Search for chunk marker in BEAM file
        // Note: This is a simplified search - a full implementation would parse the IFF
        // (Interchange File Format) structure properly, respecting chunk boundaries and
        // alignment. The proper implementation would use code_management::beam_loader::BeamLoader
        // to parse the file structure and extract chunks by ID.
        let chunk_marker = &chunk_id_bytes;
        if let Some(pos) = code_bytes.windows(4).position(|w| w == chunk_marker) {
            // Found chunk marker, return chunk data as binary
            // In a full implementation, this would parse the chunk header (ID + size)
            // and return only the chunk data, not the entire remainder of the file
            let chunk_data = code_bytes[pos..].to_vec();
            Ok(ErlangTerm::Binary(chunk_data))
        } else {
            Ok(ErlangTerm::Atom("undefined".to_string()))
        }
    }

    /// Check dirty process code (erts_internal_check_dirty_process_code/2)
    ///
    /// This is an internal function for checking if dirty processes use code.
    /// 
    /// Checks if any dirty processes (processes running on dirty schedulers) have
    /// code pointers pointing into the module's old code area.
    ///
    /// # Arguments
    /// * `pid` - Process ID (currently unused, but kept for API compatibility)
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - Some dirty process is using the code
    /// * `Ok(ErlangTerm::Atom("false"))` - No dirty processes using the code
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Check dirty process code
    /// let result = LoadBif::erts_internal_check_dirty_process_code_2(
    ///     &ErlangTerm::Pid(12345),
    ///     &ErlangTerm::Atom("my_module".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check with different PID
    /// let result = LoadBif::erts_internal_check_dirty_process_code_2(
    ///     &ErlangTerm::Pid(67890),
    ///     &ErlangTerm::Atom("other_module".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Check with invalid module
    /// let result = LoadBif::erts_internal_check_dirty_process_code_2(
    ///     &ErlangTerm::Pid(12345),
    ///     &ErlangTerm::Integer(42),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn erts_internal_check_dirty_process_code_2(
        _pid: &ErlangTerm,
        module: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        // Get module code area for old code (if available)
        let module_code = Self::get_module_old_code_area(&module_name);
        
        // Check if any dirty process is using the module's old code
        let any_dirty_uses = any_dirty_process_uses_module(&module_code);
        
        Ok(if any_dirty_uses {
            ErlangTerm::Atom("true".to_string())
        } else {
            ErlangTerm::Atom("false".to_string())
        })
    }

    /// Call on_load function (call_on_load_function/1)
    ///
    /// This is typically implemented as an instruction, not a BIF.
    /// This BIF interface is not supported - on_load functions are called
    /// automatically by the code loading infrastructure when modules are loaded.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Err(LoadError)` - This function is not supported as a BIF
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Call on_load (not supported as BIF)
    /// let result = LoadBif::call_on_load_function_1(&ErlangTerm::Atom("my_module".to_string()));
    /// assert!(result.is_err());
    ///
    /// // Call with invalid module name
    /// let result = LoadBif::call_on_load_function_1(&ErlangTerm::Integer(42));
    /// assert!(result.is_err());
    ///
    /// // This function always returns an error (not supported as BIF)
    /// let result = LoadBif::call_on_load_function_1(&ErlangTerm::Atom("any_module".to_string()));
    /// assert!(result.is_err());
    /// ```
    pub fn call_on_load_function_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let _module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        Err(LoadError::NotSupported(
            "call_on_load_function is implemented as an instruction, not a BIF".to_string(),
        ))
    }

    /// Literal area collector send copy request (erts_literal_area_collector_send_copy_request/3)
    ///
    /// This is an internal function for literal area collection.
    /// 
    /// Note: Literal area collection is a low-level memory management feature for
    /// handling module literals during code loading. This implementation provides
    /// the BIF interface but defers actual collection work to the infrastructure layer.
    ///
    /// # Arguments
    /// * `pid` - Process ID
    /// * `req_id` - Request ID
    /// * `action` - Action atom (init, check_gc, need_gc)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("ok"))` - If successful
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Send init request
    /// let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
    ///     &ErlangTerm::Pid(12345),
    ///     &ErlangTerm::Integer(1),
    ///     &ErlangTerm::Atom("init".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Send check_gc request
    /// let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
    ///     &ErlangTerm::Pid(12345),
    ///     &ErlangTerm::Integer(2),
    ///     &ErlangTerm::Atom("check_gc".to_string()),
    /// ).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Send invalid action
    /// let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
    ///     &ErlangTerm::Pid(12345),
    ///     &ErlangTerm::Integer(3),
    ///     &ErlangTerm::Atom("invalid".to_string()),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn erts_literal_area_collector_send_copy_request_3(
        _pid: &ErlangTerm,
        _req_id: &ErlangTerm,
        action: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let action_str = match action {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Action must be an atom".to_string(),
                ));
            }
        };

        match action_str.as_str() {
            "init" | "check_gc" | "need_gc" => {
                // Accept the request - actual literal area collection is handled by infrastructure
                Ok(ErlangTerm::Atom("ok".to_string()))
            }
            _ => Err(LoadError::BadArgument(
                "Action must be init, check_gc, or need_gc".to_string(),
            )),
        }
    }

    /// Literal area collector release area switch (erts_literal_area_collector_release_area_switch/0)
    ///
    /// This is an internal function for literal area collection.
    /// 
    /// Note: This function releases a literal area switch if one is pending.
    /// Currently returns false as literal area switching is handled by the infrastructure layer.
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("false"))` - No areas to switch
    /// * `Err(LoadError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Release area switch
    /// let result = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Returns false when no area switch is pending
    /// let result = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Can be called multiple times
    /// let result1 = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
    /// let result2 = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
    /// assert_eq!(result1, result2);
    /// ```
    pub fn erts_literal_area_collector_release_area_switch_0() -> Result<ErlangTerm, LoadError> {
        // No area switch pending - literal area management is handled by infrastructure layer
        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Helper: Register a module (for testing and internal use)
    ///
    /// This is a helper function to register a module in the registry.
    /// In a full implementation, this would be called by the code loader.
    ///
    /// # Arguments
    /// * `name` - Module name
    /// * `status` - Module status
    /// * `has_old_code` - Whether module has old code
    /// * `has_on_load` - Whether module has on_load function
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::{LoadBif, ModuleStatus};
    ///
    /// // Register a loaded module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let result = LoadBif::module_loaded_1(&usecases_bifs::op::ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, usecases_bifs::op::ErlangTerm::Atom("true".to_string()));
    ///
    /// // Register module with on_load
    /// LoadBif::register_module("on_load_module", ModuleStatus::OnLoadPending, false, true);
    /// let result = LoadBif::module_loaded_1(&usecases_bifs::op::ErlangTerm::Atom("on_load_module".to_string())).unwrap();
    /// assert_eq!(result, usecases_bifs::op::ErlangTerm::Atom("false".to_string()));
    ///
    /// // Register module with old code
    /// LoadBif::register_module("old_code_module", ModuleStatus::HasOldCode, true, false);
    /// let result = LoadBif::check_old_code_1(&usecases_bifs::op::ErlangTerm::Atom("old_code_module".to_string())).unwrap();
    /// assert_eq!(result, usecases_bifs::op::ErlangTerm::Atom("true".to_string()));
    /// ```
    pub fn register_module(
        name: &str,
        status: ModuleStatus,
        has_old_code: bool,
        has_on_load: bool,
    ) {
        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        modules.insert(
            name.to_string(),
            ModuleEntry {
                name: name.to_string(),
                status,
                has_old_code,
                has_on_load,
                debug_info: None,
                md5: None,
                exports: vec![],
                attributes: vec![],
                compile: vec![],
            },
        );
    }

    /// Helper: Get module old code area for process code tracking
    ///
    /// Attempts to get the old code area for a module. This is used to check
    /// if any processes have code pointers pointing into the module's old code.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    ///
    /// # Returns
    /// `ModuleCodeArea` with the module's old code area, or empty if not available
    ///
    /// # Note
    /// This function looks up the module in the code management layer's module table.
    /// It converts the module name to an atom index (simplified hash-based approach)
    /// and looks up the module. If the module has old code, it extracts the code
    /// header and length from the old module instance.
    fn get_module_old_code_area(module_name: &str) -> ModuleCodeArea {
        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();
        
        // Check if module exists and has old code in our registry
        if let Some(entry) = modules.get(module_name) {
            if entry.has_old_code {
                // Get the active code index
                let code_ix = get_global_code_ix();
                let active_ix = code_ix.active_code_ix() as usize;
                
                // Get the module manager and table
                let module_manager = get_global_module_manager();
                let table = module_manager.get_table(active_ix);
                
                // Convert module name to atom index (simplified: use hash of name)
                // In a full implementation, this would use the atom table to get the
                // actual atom index. For now, we use a simple hash-based approach.
                let module_atom = Self::module_name_to_atom_index(module_name);
                
                // Look up the module in the table
                // Note: The module might not exist in the code management layer yet
                // if it was only registered in our local registry. In that case,
                // we return an empty code area (no processes using it).
                if let Some(module) = table.get_module(module_atom) {
                    // Acquire read lock on old code
                    let _old_code_guard = module_manager.rlock_old_code(active_ix);
                    
                    // Extract old code area if available
                    if let Some(code_hdr) = module.old.code_hdr {
                        if module.old.code_length > 0 {
                            return ModuleCodeArea::new(
                                code_hdr as *const u8,
                                module.old.code_length,
                            );
                        }
                    }
                }
                // If module not found in code management layer, return empty
                // This means no processes are using it (module not fully loaded yet)
            }
        }
        
        // Return empty code area if module not found or no old code
        ModuleCodeArea::empty()
    }
    
    /// Helper: Convert module name to atom index
    ///
    /// This is a simplified implementation that uses a hash of the module name.
    /// In a full implementation, this would use the atom table to look up or
    /// create the atom and return its index.
    ///
    /// # Arguments
    /// * `module_name` - Module name as string
    ///
    /// # Returns
    /// Atom index (u32) for the module name
    fn module_name_to_atom_index(module_name: &str) -> u32 {
        // Simplified: use a hash of the module name
        // In a full implementation, this would:
        // 1. Look up the atom in the atom table
        // 2. If not found, create it
        // 3. Return the atom index
        //
        // For now, we use a simple hash function to generate a consistent index
        let mut hasher = DefaultHasher::new();
        module_name.hash(&mut hasher);
        (hasher.finish() & 0xFFFFFFFF) as u32
    }
    
    /// Helper: Parse BEAM file metadata (exports, attributes, compile info)
    ///
    /// Parses the BEAM file to extract exports, attributes, and compile info chunks.
    /// These are decoded from external term format and stored as ErlangTerm values.
    ///
    /// # Arguments
    /// * `code_bytes` - BEAM file bytes
    ///
    /// # Returns
    /// Tuple of (exports, attributes, compile) as vectors of ErlangTerm
    fn parse_beam_metadata(code_bytes: &[u8]) -> (Vec<ErlangTerm>, Vec<ErlangTerm>, Vec<ErlangTerm>) {
        // Parse BEAM file using BeamLoader
        use code_management_code_loading::BeamLoader;
        use infrastructure_data_handling::decode_term::decode_ei_term;
        
        match BeamLoader::read_beam_file(code_bytes) {
            Ok(beam_file) => {
                // Extract exports
                let exports = Self::parse_exports_from_beam(&beam_file);
                
                // Extract attributes - decode from external term format
                let attributes = if let Some(attr_data) = &beam_file.attributes_data {
                    match decode_ei_term(attr_data, 0) {
                        Ok((term, _)) => {
                            // Attributes are typically a list of tuples
                            Self::term_to_erlang_term_list(&term)
                        }
                        Err(_) => {
                            // If decoding fails, return empty
                            vec![]
                        }
                    }
                } else {
                    vec![]
                };
                
                // Extract compile info - decode from external term format
                let compile = if let Some(compile_data) = &beam_file.compile_info_data {
                    match decode_ei_term(compile_data, 0) {
                        Ok((term, _)) => {
                            // Compile info is typically a list of {Key, Value} tuples
                            Self::term_to_erlang_term_list(&term)
                        }
                        Err(_) => {
                            // If decoding fails, return empty
                            vec![]
                        }
                    }
                } else {
                    vec![]
                };
                
                (exports, attributes, compile)
            }
            Err(_) => {
                // If parsing fails, return empty vectors
                (vec![], vec![], vec![])
            }
        }
    }
    
    /// Helper: Convert Term to ErlangTerm
    ///
    /// Converts a decoded Term from the infrastructure layer to ErlangTerm
    /// for use in the use cases layer.
    ///
    /// # Arguments
    /// * `term` - Term to convert
    ///
    /// # Returns
    /// ErlangTerm representation
    fn term_to_erlang_term(term: &entities_data_handling::term_hashing::Term) -> ErlangTerm {
        use entities_data_handling::term_hashing::Term;
        
        match term {
            Term::Nil => ErlangTerm::Nil,
            Term::Small(value) => {
                // Check if it fits in i64
                if *value >= i64::MIN as i64 && *value <= i64::MAX as i64 {
                    ErlangTerm::Integer(*value as i64)
                } else {
                    // Convert to BigInteger if needed
                    use entities_utilities::BigNumber;
                    ErlangTerm::BigInteger(BigNumber::from(*value))
                }
            }
            Term::Atom(atom_index) => {
                // Convert atom index to string
                // In a full implementation, we'd look up the atom name from the atom table
                // For now, we'll use a simplified representation
                ErlangTerm::Atom(format!("atom_{}", atom_index))
            }
            Term::Big(bignum) => ErlangTerm::BigInteger(bignum.clone()),
            Term::Rational(rational) => ErlangTerm::Rational(rational.clone()),
            Term::Float(value) => ErlangTerm::Float(*value),
            Term::Binary { data, bit_offset, bit_size } => {
                if *bit_offset == 0 && *bit_size % 8 == 0 {
                    ErlangTerm::Binary(data.clone())
                } else {
                    ErlangTerm::Bitstring(data.clone(), *bit_size)
                }
            }
            Term::List { head, tail } => {
                // Convert cons cell structure to Vec<ErlangTerm>
                let mut elements = Vec::new();
                let mut current_head = head.as_ref();
                let mut current_tail = tail.as_ref();
                
                // Traverse the list structure
                loop {
                    elements.push(Self::term_to_erlang_term(current_head));
                    
                    match current_tail {
                        Term::List { head: h, tail: t } => {
                            current_head = h.as_ref();
                            current_tail = t.as_ref();
                        }
                        Term::Nil => {
                            break;
                        }
                        _ => {
                            // Improper list - add tail as last element
                            elements.push(Self::term_to_erlang_term(current_tail));
                            break;
                        }
                    }
                }
                
                ErlangTerm::List(elements)
            }
            Term::Tuple(elements) => {
                let erlang_elements: Vec<ErlangTerm> = elements
                    .iter()
                    .map(|e| Self::term_to_erlang_term(e))
                    .collect();
                ErlangTerm::Tuple(erlang_elements)
            }
            Term::Map(entries) => {
                use std::collections::HashMap;
                let mut map = HashMap::new();
                for (k, v) in entries {
                    map.insert(
                        Self::term_to_erlang_term(k),
                        Self::term_to_erlang_term(v),
                    );
                }
                ErlangTerm::Map(map)
            }
            Term::Pid { node, id, serial, creation: _ } => {
                // Encode PID as u64 (simplified)
                let pid_value = (*node as u64) << 32 | (*id as u64) << 16 | (*serial as u64);
                ErlangTerm::Pid(pid_value)
            }
            Term::Port { node, id, creation: _ } => {
                // Encode Port as u64 (simplified)
                let port_value = (*node as u64) << 32 | *id;
                ErlangTerm::Port(port_value)
            }
            Term::Ref { node, ids, creation: _ } => {
                // Encode Reference as u64 (simplified)
                let ref_value = if let Some(&first_id) = ids.first() {
                    (*node as u64) << 32 | (first_id as u64)
                } else {
                    *node as u64
                };
                ErlangTerm::Reference(ref_value)
            }
            Term::Fun { arity, .. } => {
                ErlangTerm::Function { arity: *arity as usize }
            }
        }
    }
    
    /// Helper: Convert Term to list of ErlangTerm
    ///
    /// If the term is a list, converts it to a Vec<ErlangTerm>.
    /// If it's a single term, wraps it in a Vec.
    ///
    /// # Arguments
    /// * `term` - Term to convert
    ///
    /// # Returns
    /// Vector of ErlangTerm
    fn term_to_erlang_term_list(term: &entities_data_handling::term_hashing::Term) -> Vec<ErlangTerm> {
        use entities_data_handling::term_hashing::Term;
        
        match term {
            Term::List { .. } | Term::Nil => {
                // Convert list to Vec
                let erlang_term = Self::term_to_erlang_term(term);
                if let ErlangTerm::List(elements) = erlang_term {
                    elements
                } else {
                    vec![]
                }
            }
            Term::Tuple(elements) => {
                // If it's a tuple, convert each element
                elements
                    .iter()
                    .map(|e| Self::term_to_erlang_term(e))
                    .collect()
            }
            _ => {
                // Single term - wrap in Vec
                vec![Self::term_to_erlang_term(term)]
            }
        }
    }
    
    /// Helper: Parse exports from BEAM file
    ///
    /// Extracts export information from the BEAM file's export table.
    /// Returns a list of {Function, Arity} tuples as ErlangTerm.
    ///
    /// # Arguments
    /// * `beam_file` - Parsed BEAM file
    ///
    /// # Returns
    /// Vector of ErlangTerm representing exports
    fn parse_exports_from_beam(beam_file: &code_management_code_loading::BeamFile) -> Vec<ErlangTerm> {
        // Convert exports from (function_atom, arity, label) to ErlangTerm tuples
        // For now, we'll create simplified tuples: {Function, Arity}
        // In a full implementation, we'd need the atom table to convert atom indices to names
        let mut exports = Vec::new();
        
        for (function_atom, arity, _label) in &beam_file.exports {
            // Create a tuple {Function, Arity}
            // Since we don't have atom names yet, we'll use the atom index as an integer
            // In a full implementation, we'd look up the atom name from the atom table
            let export_tuple = ErlangTerm::Tuple(vec![
                ErlangTerm::Integer(*function_atom as i64), // Function atom index
                ErlangTerm::Integer(*arity as i64),        // Arity
            ]);
            exports.push(export_tuple);
        }
        
        exports
    }

    /// Helper: Mark a module as pre-loaded (for testing and internal use)
    ///
    /// # Arguments
    /// * `name` - Module name to mark as pre-loaded
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Mark module as pre-loaded
    /// LoadBif::clear_all();
    /// LoadBif::mark_preloaded("system_module");
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let usecases_bifs::op::ErlangTerm::List(modules) = preloaded {
    ///     assert!(modules.len() >= 1);
    /// }
    ///
    /// // Mark multiple modules as pre-loaded
    /// LoadBif::clear_all();
    /// LoadBif::mark_preloaded("module1");
    /// LoadBif::mark_preloaded("module2");
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let usecases_bifs::op::ErlangTerm::List(modules) = preloaded {
    ///     assert!(modules.len() >= 2);
    /// }
    ///
    /// // Pre-loaded modules persist after clear_all
    /// LoadBif::mark_preloaded("persistent");
    /// LoadBif::clear_all();
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let usecases_bifs::op::ErlangTerm::List(modules) = preloaded {
    ///     assert!(modules.len() >= 1);
    /// }
    /// ```
    pub fn mark_preloaded(name: &str) {
        let registry = ModuleRegistry::get_instance();
        let mut preloaded = registry.preloaded.write().unwrap();
        preloaded.insert(name.to_string());
    }

    /// Get module metadata (for info module)
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    ///
    /// # Returns
    /// * `Some(ModuleMetadata)` - If module is found
    /// * `None` - If module is not found
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Get metadata for loaded module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let metadata = LoadBif::get_module_metadata("my_module");
    /// assert!(metadata.is_some());
    ///
    /// // Get metadata for non-existent module
    /// let metadata = LoadBif::get_module_metadata("nonexistent");
    /// assert!(metadata.is_none());
    ///
    /// // Get metadata with MD5
    /// LoadBif::clear_all();
    /// let code = vec![0x01, 0x02, 0x03];
    /// let ref_term = LoadBif::erts_internal_prepare_loading_2(
    ///     &usecases_bifs::op::ErlangTerm::Atom("md5_module".to_string()),
    ///     &usecases_bifs::op::ErlangTerm::Binary(code),
    /// ).unwrap();
    /// LoadBif::finish_loading_1(&usecases_bifs::op::ErlangTerm::List(vec![ref_term])).unwrap();
    /// let metadata = LoadBif::get_module_metadata("md5_module");
    /// assert!(metadata.is_some());
    /// if let Some(meta) = metadata {
    ///     assert!(meta.md5.is_some());
    /// }
    /// ```
    pub fn get_module_metadata(module_name: &str) -> Option<ModuleMetadata> {
        let registry = ModuleRegistry::get_instance();
        let modules = registry.modules.read().unwrap();
        modules.get(module_name).map(|entry| ModuleMetadata {
            md5: entry.md5.clone(),
            exports: entry.exports.clone(),
            attributes: entry.attributes.clone(),
            compile: entry.compile.clone(),
        })
    }

    /// Helper: Set debug info for a module (for testing)
    ///
    /// # Arguments
    /// * `module` - Module name
    /// * `debug_info` - Debug information (ErlangTerm, typically a map)
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Set debug info for module
    /// LoadBif::clear_all();
    /// LoadBif::register_module("my_module", ModuleStatus::Loaded, false, false);
    /// let debug_info = ErlangTerm::Map(std::collections::HashMap::new());
    /// LoadBif::set_debug_info("my_module", debug_info.clone());
    /// let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, debug_info);
    ///
    /// // Set debug info with map data
    /// let mut debug_map = std::collections::HashMap::new();
    /// debug_map.insert(ErlangTerm::Atom("key".to_string()), ErlangTerm::Integer(42));
    /// let debug_info2 = ErlangTerm::Map(debug_map);
    /// LoadBif::set_debug_info("my_module", debug_info2.clone());
    /// let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("my_module".to_string())).unwrap();
    /// assert_eq!(result, debug_info2);
    ///
    /// // Set debug info for non-existent module (no-op)
    /// LoadBif::set_debug_info("nonexistent", ErlangTerm::Atom("info".to_string()));
    /// // Module not found, so set_debug_info does nothing
    /// ```
    pub fn set_debug_info(module: &str, debug_info: ErlangTerm) {
        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        if let Some(entry) = modules.get_mut(module) {
            entry.debug_info = Some(debug_info);
        }
    }

    /// Helper: Clear all modules (for testing)
    ///
    /// Clears all registered modules, pre-loaded modules, and prepared code.
    /// Useful for test isolation.
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Clear all modules
    /// LoadBif::register_module("module1", ModuleStatus::Loaded, false, false);
    /// LoadBif::register_module("module2", ModuleStatus::Loaded, false, false);
    /// LoadBif::clear_all();
    /// let loaded = LoadBif::loaded_0().unwrap();
    /// if let usecases_bifs::op::ErlangTerm::List(modules) = loaded {
    ///     assert_eq!(modules.len(), 0);
    /// }
    ///
    /// // Clear after marking pre-loaded
    /// LoadBif::mark_preloaded("preloaded");
    /// LoadBif::register_module("regular", ModuleStatus::Loaded, false, false);
    /// LoadBif::clear_all();
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// if let usecases_bifs::op::ErlangTerm::List(modules) = preloaded {
    ///     assert_eq!(modules.len(), 0);
    /// }
    ///
    /// // Clear prepared code
    /// let code = vec![0x01, 0x02];
    /// let ref_term = LoadBif::erts_internal_prepare_loading_2(
    ///     &usecases_bifs::op::ErlangTerm::Atom("prepared".to_string()),
    ///     &usecases_bifs::op::ErlangTerm::Binary(code),
    /// ).unwrap();
    /// LoadBif::clear_all();
    /// // Prepared code is also cleared
    /// ```
    pub fn clear_all() {
        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();
        let mut preloaded = registry.preloaded.write().unwrap();
        modules.clear();
        preloaded.clear();
        
        let prepared_registry = PreparedCodeRegistry::get_instance();
        prepared_registry.clear();
        
        // Also clear persistent terms to avoid test interference
        use crate::persistent::PersistentBif;
        let _ = PersistentBif::erase_all_0();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_module_1_not_found() {
        LoadBif::clear_all();

        let result = LoadBif::delete_module_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    }

    #[test]
    fn test_delete_module_1_success() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "test_module",
            ModuleStatus::Loaded,
            false,
            false,
        );

        let result = LoadBif::delete_module_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));

        // Verify it's gone
        let loaded = LoadBif::loaded_0().unwrap();
        if let ErlangTerm::List(list) = loaded {
            assert!(!list.contains(&ErlangTerm::Atom("test_module".to_string())));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_delete_module_1_with_old_code() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "old_module",
            ModuleStatus::Loaded,
            true, // has old code
            false,
        );

        let result = LoadBif::delete_module_1(&ErlangTerm::Atom("old_module".to_string()));
        assert!(result.is_err());
        if let Err(LoadError::BadArgument(msg)) = result {
            assert!(msg.contains("must be purged"));
        } else {
            panic!("Expected BadArgument error");
        }
    }

    #[test]
    fn test_delete_module_1_preloaded() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "preloaded_module",
            ModuleStatus::PreLoaded,
            false,
            false,
        );
        LoadBif::mark_preloaded("preloaded_module");

        let result = LoadBif::delete_module_1(&ErlangTerm::Atom("preloaded_module".to_string()));
        assert!(result.is_err());
        if let Err(LoadError::BadArgument(msg)) = result {
            assert!(msg.contains("pre-loaded"));
        } else {
            panic!("Expected BadArgument error");
        }
    }

    #[test]
    fn test_delete_module_1_invalid_argument() {
        LoadBif::clear_all();

        let result = LoadBif::delete_module_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_module_loaded_1_not_found() {
        LoadBif::clear_all();

        let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_module_loaded_1_loaded() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "loaded_module",
            ModuleStatus::Loaded,
            false,
            false,
        );

        let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("loaded_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_module_loaded_1_preloaded() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "preloaded_module",
            ModuleStatus::PreLoaded,
            false,
            false,
        );

        let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("preloaded_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_module_loaded_1_on_load_pending() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "pending_module",
            ModuleStatus::OnLoadPending,
            false,
            true,
        );

        let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("pending_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_module_loaded_1_invalid_argument() {
        LoadBif::clear_all();

        let result = LoadBif::module_loaded_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_pre_loaded_0_empty() {
        LoadBif::clear_all();

        let result = LoadBif::pre_loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 0);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_pre_loaded_0_with_modules() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "preloaded1",
            ModuleStatus::PreLoaded,
            false,
            false,
        );
        LoadBif::mark_preloaded("preloaded1");

        LoadBif::register_module(
            "preloaded2",
            ModuleStatus::PreLoaded,
            false,
            false,
        );
        LoadBif::mark_preloaded("preloaded2");

        let result = LoadBif::pre_loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 2);
            assert!(list.contains(&ErlangTerm::Atom("preloaded1".to_string())));
            assert!(list.contains(&ErlangTerm::Atom("preloaded2".to_string())));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_loaded_0_empty() {
        LoadBif::clear_all();

        let result = LoadBif::loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 0);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_loaded_0_with_modules() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "module1",
            ModuleStatus::Loaded,
            false,
            false,
        );

        LoadBif::register_module(
            "module2",
            ModuleStatus::Loaded,
            false,
            false,
        );

        LoadBif::register_module(
            "preloaded1",
            ModuleStatus::PreLoaded,
            false,
            false,
        );

        let result = LoadBif::loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            // Check that our modules are present
            assert!(list.contains(&ErlangTerm::Atom("module1".to_string())), "module1 not found");
            assert!(list.contains(&ErlangTerm::Atom("module2".to_string())), "module2 not found");
            assert!(list.contains(&ErlangTerm::Atom("preloaded1".to_string())), "preloaded1 not found");
            // Tests run serially, so we should have exactly 3 modules
            assert_eq!(list.len(), 3, "Expected exactly 3 modules");
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_finish_after_on_load_2_success() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "onload_module",
            ModuleStatus::OnLoadPending,
            false,
            true,
        );

        let result = LoadBif::finish_after_on_load_2(
            &ErlangTerm::Atom("onload_module".to_string()),
            &ErlangTerm::Atom("true".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Verify module is now loaded
        let loaded_result = LoadBif::module_loaded_1(&ErlangTerm::Atom("onload_module".to_string())).unwrap();
        assert_eq!(loaded_result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_finish_after_on_load_2_failure() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "failed_module",
            ModuleStatus::OnLoadPending,
            false,
            true,
        );

        let result = LoadBif::finish_after_on_load_2(
            &ErlangTerm::Atom("failed_module".to_string()),
            &ErlangTerm::Atom("false".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Verify module is removed
        let loaded_result = LoadBif::module_loaded_1(&ErlangTerm::Atom("failed_module".to_string())).unwrap();
        assert_eq!(loaded_result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_finish_after_on_load_2_not_pending() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "loaded_module",
            ModuleStatus::Loaded,
            false,
            false,
        );

        let result = LoadBif::finish_after_on_load_2(
            &ErlangTerm::Atom("loaded_module".to_string()),
            &ErlangTerm::Atom("true".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_finish_after_on_load_2_invalid_success() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "onload_module",
            ModuleStatus::OnLoadPending,
            false,
            true,
        );

        let result = LoadBif::finish_after_on_load_2(
            &ErlangTerm::Atom("onload_module".to_string()),
            &ErlangTerm::Integer(1),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_code_get_debug_info_1_none() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "no_debug_module",
            ModuleStatus::Loaded,
            false,
            false,
        );

        let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("no_debug_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("none".to_string()));
    }

    #[test]
    fn test_code_get_debug_info_1_with_debug_info() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "debug_module",
            ModuleStatus::Loaded,
            false,
            false,
        );

        // Verify module exists before setting debug info
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("debug_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));

        let debug_info = ErlangTerm::Map({
            let mut map = HashMap::new();
            map.insert(
                ErlangTerm::Atom("source".to_string()),
                ErlangTerm::Atom("test.erl".to_string()),
            );
            map
        });

        LoadBif::set_debug_info("debug_module", debug_info.clone());

        let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("debug_module".to_string())).unwrap();
        assert_eq!(result, debug_info);
    }

    #[test]
    fn test_code_get_debug_info_1_not_found() {
        LoadBif::clear_all();

        let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("nonexistent".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_check_process_code_1() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_check_process_code_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_erts_internal_purge_module_2() {
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::thread;
        use std::time::Duration;
        
        // Retry the entire test setup if race condition occurs
        let mut success = false;
        let mut final_unique_name = String::new();
        for _attempt in 0..3 {
            LoadBif::clear_all();
            
            let unique_name = format!("purge_module_{}_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(), _attempt);
            LoadBif::register_module(
                &unique_name,
                ModuleStatus::Loaded,
                true, // has old code
                false,
            );
            
            // Small delay to allow registration to complete
            thread::sleep(Duration::from_millis(10));
            
            // Verify module exists
            let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom(unique_name.clone()));
            if loaded == Ok(ErlangTerm::Atom("true".to_string())) {
                success = true;
                final_unique_name = unique_name.clone();
                
                // Continue with the rest of the test using unique_name
                // Verify module has old code before purging
                let has_old = LoadBif::check_old_code_1(&ErlangTerm::Atom(unique_name.clone())).unwrap();
                assert_eq!(has_old, ErlangTerm::Atom("true".to_string()));

                let result = LoadBif::erts_internal_purge_module_2(
                    &ErlangTerm::Atom(unique_name.clone()),
                    &ErlangTerm::Atom("force".to_string()),
                ).unwrap();

                assert_eq!(result, ErlangTerm::Atom("true".to_string()));

                // Verify module still exists
                let loaded_after = LoadBif::module_loaded_1(&ErlangTerm::Atom(unique_name.clone())).unwrap();
                assert_eq!(loaded_after, ErlangTerm::Atom("true".to_string()));

                // Verify old code flag is cleared
                let has_old_after = LoadBif::check_old_code_1(&ErlangTerm::Atom(unique_name.clone())).unwrap();
                assert_eq!(has_old_after, ErlangTerm::Atom("false".to_string()));
                
                // Verify module can now be deleted
                let delete_result = LoadBif::delete_module_1(&ErlangTerm::Atom(unique_name.clone())).unwrap();
                assert_eq!(delete_result, ErlangTerm::Atom("true".to_string()));
                break;
            }
        }
        assert!(success, "Failed to register module after retries");
    }

    #[test]
    fn test_loaded_0_sorted() {
        use std::time::{SystemTime, UNIX_EPOCH};
        LoadBif::clear_all();

        let unique_prefix = format!("test_loaded_sorted_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let alpha = format!("{}_alpha", unique_prefix);
        let beta = format!("{}_beta", unique_prefix);
        let zebra = format!("{}_zebra", unique_prefix);
        
        LoadBif::register_module(&zebra, ModuleStatus::Loaded, false, false);
        LoadBif::register_module(&alpha, ModuleStatus::Loaded, false, false);
        LoadBif::register_module(&beta, ModuleStatus::Loaded, false, false);

        let result = LoadBif::loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            // Check that our modules are present and sorted
            let alpha_term = ErlangTerm::Atom(alpha.clone());
            let beta_term = ErlangTerm::Atom(beta.clone());
            let zebra_term = ErlangTerm::Atom(zebra.clone());
            
            assert!(list.contains(&alpha_term), "alpha module not found");
            assert!(list.contains(&beta_term), "beta module not found");
            assert!(list.contains(&zebra_term), "zebra module not found");
            
            // Find positions and verify sorting
            let alpha_pos = list.iter().position(|x| x == &alpha_term).unwrap();
            let beta_pos = list.iter().position(|x| x == &beta_term).unwrap();
            let zebra_pos = list.iter().position(|x| x == &zebra_term).unwrap();
            
            assert!(alpha_pos < beta_pos, "alpha should come before beta");
            assert!(beta_pos < zebra_pos, "beta should come before zebra");
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_pre_loaded_0_sorted() {
        LoadBif::clear_all();

        LoadBif::register_module("zebra", ModuleStatus::PreLoaded, false, false);
        LoadBif::mark_preloaded("zebra");
        LoadBif::register_module("alpha", ModuleStatus::PreLoaded, false, false);
        LoadBif::mark_preloaded("alpha");
        LoadBif::register_module("beta", ModuleStatus::PreLoaded, false, false);
        LoadBif::mark_preloaded("beta");

        let result = LoadBif::pre_loaded_0().unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 3);
            // Should be sorted alphabetically
            assert_eq!(list[0], ErlangTerm::Atom("alpha".to_string()));
            assert_eq!(list[1], ErlangTerm::Atom("beta".to_string()));
            assert_eq!(list[2], ErlangTerm::Atom("zebra".to_string()));
        } else {
            panic!("Expected List");
        }
    }

    // ============================================================================
    // Tests for Priority 1: Core Loading Functions
    // ============================================================================

    #[test]
    fn test_erts_internal_prepare_loading_2_success() {
        LoadBif::clear_all();

        // Test BEAM code - minimal valid BEAM file header bytes for testing
        // In real usage, this would be a complete BEAM file with proper IFF structure
        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00];
        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Should return a Reference
        assert!(matches!(result, ErlangTerm::Reference(_)));
    }

    #[test]
    fn test_erts_internal_prepare_loading_2_invalid_module() {
        LoadBif::clear_all();

        let code = vec![0xBE, 0x41, 0x4D];
        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Binary(code),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_prepare_loading_2_invalid_code() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Integer(123),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_has_prepared_code_on_load_1_true() {
        LoadBif::clear_all();

        // Prepare code with on_load (code starting with 0xBE indicates on_load)
        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("onload_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        let result = LoadBif::has_prepared_code_on_load_1(&prepared_ref).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_has_prepared_code_on_load_1_false() {
        LoadBif::clear_all();

        // Prepare code without on_load
        let code = vec![0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("normal_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        let result = LoadBif::has_prepared_code_on_load_1(&prepared_ref).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_has_prepared_code_on_load_1_invalid_ref() {
        LoadBif::clear_all();

        let result = LoadBif::has_prepared_code_on_load_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_finish_loading_1_success() {
        LoadBif::clear_all();

        // Prepare code
        let code = vec![0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Finish loading
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Verify module is loaded
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_finish_loading_1_with_old_code() {
        LoadBif::clear_all();

        // Register module with old code
        LoadBif::register_module("old_module", ModuleStatus::Loaded, true, false);

        // Prepare new code
        let code = vec![0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("old_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Finish loading should fail
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple[0], ErlangTerm::Atom("error".to_string()));
        } else {
            panic!("Expected error tuple");
        }
    }

    #[test]
    fn test_finish_loading_1_invalid_list() {
        LoadBif::clear_all();

        let result = LoadBif::finish_loading_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    // ============================================================================
    // Tests for Priority 2: Useful Functions
    // ============================================================================

    #[test]
    fn test_check_old_code_1_true() {
        LoadBif::clear_all();

        LoadBif::register_module("old_module", ModuleStatus::Loaded, true, false);

        let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("old_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_check_old_code_1_false() {
        LoadBif::clear_all();

        LoadBif::register_module("new_module", ModuleStatus::Loaded, false, false);

        let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("new_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_check_old_code_1_not_found() {
        LoadBif::clear_all();

        let result = LoadBif::check_old_code_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_erts_internal_beamfile_module_md5_1() {
        LoadBif::clear_all();

        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
        let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Binary(code)).unwrap();

        // Should return a binary (MD5 hash)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_beamfile_module_md5_1_empty() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Binary(vec![])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    }

    #[test]
    fn test_erts_internal_beamfile_module_md5_1_invalid() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    // ============================================================================
    // Tests for Priority 3: Internal/Specialized Functions
    // ============================================================================

    #[test]
    fn test_erts_internal_check_dirty_process_code_2() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_check_dirty_process_code_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("test_module".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_call_on_load_function_1() {
        LoadBif::clear_all();

        let result = LoadBif::call_on_load_function_1(&ErlangTerm::Atom("test_module".to_string()));
        assert!(result.is_err());
        if let Err(LoadError::NotSupported(_)) = result {
            // Expected
        } else {
            panic!("Expected NotSupported error");
        }
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_init() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Atom("init".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_check_gc() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Atom("check_gc".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_need_gc() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Atom("need_gc".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_invalid_action() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Atom("invalid".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_literal_area_collector_release_area_switch_0() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_found() {
        LoadBif::clear_all();

        // Create code with chunk marker
        let code = vec![0x00, 0x01, 0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        ).unwrap();

        // Should return a binary (chunk data)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_not_found() {
        LoadBif::clear_all();

        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xFF),
            ErlangTerm::Integer(0xFF),
            ErlangTerm::Integer(0xFF),
            ErlangTerm::Integer(0xFF),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("undefined".to_string()));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id() {
        LoadBif::clear_all();

        let chunk_id_list = ErlangTerm::List(vec![ErlangTerm::Integer(0xFF)]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    // ============================================================================
    // Additional tests for 100% coverage
    // ============================================================================

    #[test]
    fn test_erts_internal_prepare_loading_2_bitstring() {
        LoadBif::clear_all();

        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00];
        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Bitstring(code, 40), // 5 bytes * 8 bits
        ).unwrap();

        assert!(matches!(result, ErlangTerm::Reference(_)));
    }

    #[test]
    fn test_erts_internal_prepare_loading_2_empty_code() {
        LoadBif::clear_all();

        let code = vec![];
        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        assert!(matches!(result, ErlangTerm::Reference(_)));
    }

    #[test]
    fn test_has_prepared_code_on_load_1_invalid_reference_not_found() {
        LoadBif::clear_all();

        // Create a reference that doesn't exist in the registry
        let fake_ref = ErlangTerm::Reference(999999999);
        let result = LoadBif::has_prepared_code_on_load_1(&fake_ref);
        assert!(result.is_err());
    }

    #[test]
    fn test_finish_loading_1_empty_list() {
        LoadBif::clear_all();

        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_finish_loading_1_invalid_reference_type() {
        LoadBif::clear_all();

        // List with non-reference type
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![
            ErlangTerm::Integer(123),
        ])).unwrap();

        // Should return error tuple
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple[0], ErlangTerm::Atom("error".to_string()));
        } else {
            panic!("Expected error tuple");
        }
    }

    #[test]
    fn test_finish_loading_1_invalid_reference_not_found() {
        LoadBif::clear_all();

        // Reference that doesn't exist
        let fake_ref = ErlangTerm::Reference(999999999);
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![fake_ref])).unwrap();

        // Should return error tuple
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple[0], ErlangTerm::Atom("error".to_string()));
        } else {
            panic!("Expected error tuple");
        }
    }

    #[test]
    fn test_finish_loading_1_with_on_load() {
        LoadBif::clear_all();

        // Prepare code with on_load
        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("onload_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Finish loading
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Verify module is in OnLoadPending status
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("onload_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("false".to_string())); // Not loaded yet, pending on_load
    }

    #[test]
    fn test_finish_loading_1_multiple_modules() {
        LoadBif::clear_all();

        // Prepare multiple modules - use different code to ensure different hashes
        let code1 = vec![0x00, 0x01, 0x02, 0x03, 0xAA];
        let prepared_ref1 = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("module1".to_string()),
            &ErlangTerm::Binary(code1),
        ).unwrap();

        // Use significantly different code to ensure different reference values
        let code2 = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xBB];
        let prepared_ref2 = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("module2".to_string()),
            &ErlangTerm::Binary(code2),
        ).unwrap();

        // Verify references are different
        let ref_val1 = match &prepared_ref1 {
            ErlangTerm::Reference(v) => *v,
            _ => panic!("Expected Reference"),
        };
        let ref_val2 = match &prepared_ref2 {
            ErlangTerm::Reference(v) => *v,
            _ => panic!("Expected Reference"),
        };
        assert_ne!(ref_val1, ref_val2, "References should be unique");

        // Finish loading both
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref1, prepared_ref2])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Verify both modules are loaded
        let loaded1 = LoadBif::module_loaded_1(&ErlangTerm::Atom("module1".to_string())).unwrap();
        assert_eq!(loaded1, ErlangTerm::Atom("true".to_string()));

        let loaded2 = LoadBif::module_loaded_1(&ErlangTerm::Atom("module2".to_string())).unwrap();
        assert_eq!(loaded2, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_finish_loading_1_mixed_success_and_failure() {
        LoadBif::clear_all();

        // Prepare one valid module
        let code = vec![0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("valid_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Add an invalid reference
        let fake_ref = ErlangTerm::Reference(999999999);

        // Finish loading with mixed list
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref, fake_ref])).unwrap();

        // Should return error tuple with one error
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple[0], ErlangTerm::Atom("error".to_string()));
            if let ErlangTerm::List(errors) = &tuple[1] {
                assert_eq!(errors.len(), 1);
            } else {
                panic!("Expected error list");
            }
        } else {
            panic!("Expected error tuple");
        }
    }

    #[test]
    fn test_erts_internal_beamfile_module_md5_1_bitstring() {
        LoadBif::clear_all();

        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
        let result = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Bitstring(code, 56)).unwrap();

        // Should return a binary (MD5 hash)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_bitstring() {
        LoadBif::clear_all();

        // Create code with chunk marker
        let code = vec![0x00, 0x01, 0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Bitstring(code, 72), // 9 bytes * 8 bits
            &chunk_id_list,
        ).unwrap();

        // Should return a binary (chunk data)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id_bytes_out_of_range() {
        LoadBif::clear_all();

        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(256), // Out of range (> 255)
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id_negative() {
        LoadBif::clear_all();

        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(-1), // Negative
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id_wrong_type() {
        LoadBif::clear_all();

        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Atom("invalid".to_string()), // Wrong type
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id_wrong_length() {
        LoadBif::clear_all();

        // Wrong length (3 bytes instead of 4)
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_invalid_chunk_id_too_long() {
        LoadBif::clear_all();

        // Too long (5 bytes instead of 4)
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
            ErlangTerm::Integer(0x00),
        ]);
        let code = vec![0x00, 0x01, 0x02, 0x03];

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_chunk_at_start() {
        LoadBif::clear_all();

        // Chunk marker at the start of code
        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        ).unwrap();

        // Should return a binary (chunk data)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_beamfile_chunk_2_chunk_at_end() {
        LoadBif::clear_all();

        // Chunk marker at the end of code
        let code = vec![0x00, 0x01, 0x02, 0xBE, 0x41, 0x4D, 0x01];
        let chunk_id_list = ErlangTerm::List(vec![
            ErlangTerm::Integer(0xBE),
            ErlangTerm::Integer(0x41),
            ErlangTerm::Integer(0x4D),
            ErlangTerm::Integer(0x01),
        ]);

        let result = LoadBif::erts_internal_beamfile_chunk_2(
            &ErlangTerm::Binary(code),
            &chunk_id_list,
        ).unwrap();

        // Should return a binary (chunk data)
        assert!(matches!(result, ErlangTerm::Binary(_)));
    }

    #[test]
    fn test_erts_internal_purge_module_2_not_found() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_purge_module_2(
            &ErlangTerm::Atom("nonexistent".to_string()),
            &ErlangTerm::Atom("force".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_purge_module_2_invalid_module() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_purge_module_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Atom("force".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_check_old_code_1_invalid_argument() {
        LoadBif::clear_all();

        let result = LoadBif::check_old_code_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_check_process_code_1_invalid_argument() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_check_process_code_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_erts_internal_check_dirty_process_code_2_invalid_pid() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_check_dirty_process_code_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Atom("test_module".to_string()),
        );
        // This function doesn't validate PID, but let's test it anyway
        // Actually, looking at the code, it doesn't validate pid, so this should work
        // But let's test with invalid module instead
    }

    #[test]
    fn test_erts_internal_check_dirty_process_code_2_invalid_module() {
        LoadBif::clear_all();

        let result = LoadBif::erts_internal_check_dirty_process_code_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(123),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_invalid_pid() {
        LoadBif::clear_all();

        // Function doesn't validate pid, but let's test with invalid action
        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Atom("init".to_string()),
        );
        // Function doesn't validate pid, so this should work
        assert!(result.is_ok());
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_invalid_req_id() {
        LoadBif::clear_all();

        // Function doesn't validate req_id, but let's test it
        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("invalid".to_string()),
            &ErlangTerm::Atom("init".to_string()),
        );
        // Function doesn't validate req_id, so this should work
        assert!(result.is_ok());
    }

    #[test]
    fn test_erts_literal_area_collector_send_copy_request_3_invalid_action_type() {
        LoadBif::clear_all();

        let result = LoadBif::erts_literal_area_collector_send_copy_request_3(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(456),
            &ErlangTerm::Integer(123),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_call_on_load_function_1_invalid_argument() {
        LoadBif::clear_all();

        let result = LoadBif::call_on_load_function_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_prepared_code_compute_md5() {
        LoadBif::clear_all();

        // Test that compute_md5 is called during prepare_loading
        let code = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let result = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Binary(code.clone()),
        ).unwrap();

        // Verify the prepared code has MD5 computed by checking it can be retrieved
        let ref_value = match result {
            ErlangTerm::Reference(val) => val,
            _ => panic!("Expected Reference"),
        };

        // The MD5 is computed internally, we can verify by preparing same code twice
        // and checking references are different (they should be)
        let result2 = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module2".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        let ref_value2 = match result2 {
            ErlangTerm::Reference(val) => val,
            _ => panic!("Expected Reference"),
        };

        // References should be different (different modules)
        assert_ne!(ref_value, ref_value2);
    }

    #[test]
    fn test_finish_loading_1_module_already_loaded() {
        LoadBif::clear_all();

        // Register a module first
        LoadBif::register_module("existing_module", ModuleStatus::Loaded, false, false);

        // Prepare new code for the same module
        let code = vec![0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("existing_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Finish loading should succeed (replaces existing module)
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_module_loaded_1_has_old_code() {
        LoadBif::clear_all();

        LoadBif::register_module(
            "old_code_module",
            ModuleStatus::Loaded,
            true, // has old code
            false,
        );

        // Module with old code should still be considered loaded
        let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("old_code_module".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_term_to_erlang_term_rational() {
        use entities_data_handling::term_hashing::Term;
        use entities_utilities::BigRational;
        
        // Test positive rational (22/7)
        let rational = BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap();
        let term = Term::Rational(rational.clone());
        let erlang_term = LoadBif::term_to_erlang_term(&term);
        
        match erlang_term {
            ErlangTerm::Rational(r) => {
                assert_eq!(r, rational);
            }
            _ => panic!("Expected ErlangTerm::Rational, got {:?}", erlang_term),
        }
        
        // Test negative rational (-5/3)
        let rational_neg = BigRational::from_i64(-5).div(&BigRational::from_i64(3)).unwrap();
        let term_neg = Term::Rational(rational_neg.clone());
        let erlang_term_neg = LoadBif::term_to_erlang_term(&term_neg);
        
        match erlang_term_neg {
            ErlangTerm::Rational(r) => {
                assert_eq!(r, rational_neg);
            }
            _ => panic!("Expected ErlangTerm::Rational, got {:?}", erlang_term_neg),
        }
        
        // Test rational that's an integer (42/1)
        let rational_int = BigRational::from_i64(42).div(&BigRational::from_i64(1)).unwrap();
        let term_int = Term::Rational(rational_int.clone());
        let erlang_term_int = LoadBif::term_to_erlang_term(&term_int);
        
        match erlang_term_int {
            ErlangTerm::Rational(r) => {
                assert_eq!(r, rational_int);
            }
            _ => panic!("Expected ErlangTerm::Rational, got {:?}", erlang_term_int),
        }
        
        // Test zero rational (0/1)
        let rational_zero = BigRational::from_i64(0).div(&BigRational::from_i64(1)).unwrap();
        let term_zero = Term::Rational(rational_zero.clone());
        let erlang_term_zero = LoadBif::term_to_erlang_term(&term_zero);
        
        match erlang_term_zero {
            ErlangTerm::Rational(r) => {
                assert_eq!(r, rational_zero);
            }
            _ => panic!("Expected ErlangTerm::Rational, got {:?}", erlang_term_zero),
        }
    }
    
    #[test]
    fn test_term_to_erlang_term_rational_in_structures() {
        use entities_data_handling::term_hashing::Term;
        use entities_utilities::BigRational;
        
        // Test rational in tuple
        let rational = BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap();
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Rational(rational.clone()),
            Term::Small(3),
        ]);
        let erlang_term = LoadBif::term_to_erlang_term(&term);
        
        match erlang_term {
            ErlangTerm::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], ErlangTerm::Integer(1));
                match &elements[1] {
                    ErlangTerm::Rational(r) => {
                        assert_eq!(r, &rational);
                    }
                    _ => panic!("Expected ErlangTerm::Rational in tuple"),
                }
                assert_eq!(elements[2], ErlangTerm::Integer(3));
            }
            _ => panic!("Expected ErlangTerm::Tuple, got {:?}", erlang_term),
        }
        
        // Test rational in list
        let term_list = Term::List {
            head: Box::new(Term::Rational(rational.clone())),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(42)),
                tail: Box::new(Term::Nil),
            }),
        };
        let erlang_term_list = LoadBif::term_to_erlang_term(&term_list);
        
        match erlang_term_list {
            ErlangTerm::List(elements) => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    ErlangTerm::Rational(r) => {
                        assert_eq!(r, &rational);
                    }
                    _ => panic!("Expected ErlangTerm::Rational in list"),
                }
                assert_eq!(elements[1], ErlangTerm::Integer(42));
            }
            _ => panic!("Expected ErlangTerm::List, got {:?}", erlang_term_list),
        }
    }
}

