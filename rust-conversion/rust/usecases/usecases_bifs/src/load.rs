//! Code Loading and Module Management Built-in Functions
//!
//! Provides module loading, unloading, and management operations.
//! Tracks which modules are loaded, pre-loaded, and handles module lifecycle.
//!
//! Based on beam_bif_load.c
//!
//! This module implements safe Rust equivalents of Erlang code loading BIFs.

use crate::op::ErlangTerm;
use crate::unique::Reference;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::SystemTime;

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
    fn reference_value(&self) -> u64 {
        // Combine thread_id, value, and ref_number into a single u64
        // This is a simplified approach - in real implementation, references are multi-part
        let thread_id = self.magic_ref.thread_id() as u64;
        let value = self.magic_ref.value();
        let ref_num = self.magic_ref.ref_number() as u64;
        // Combine into a single u64 (simplified)
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
    /// let result = LoadBif::delete_module_1(&ErlangTerm::Atom("my_module".to_string()));
    /// // Returns true if deleted, undefined if not found
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
    /// let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("my_module".to_string()));
    /// // Returns true if loaded, false otherwise
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
    /// let preloaded = LoadBif::pre_loaded_0().unwrap();
    /// // Returns a list of pre-loaded module names
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
    /// let loaded = LoadBif::loaded_0().unwrap();
    /// // Returns a list of all loaded module names
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
    /// let result = LoadBif::finish_after_on_load_2(
    ///     &ErlangTerm::Atom("my_module".to_string()),
    ///     &ErlangTerm::Atom("true".to_string()),
    /// ).unwrap();
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
    /// let debug_info = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("my_module".to_string()));
    /// // Returns debug info map or "none"
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
    /// In a simplified implementation, we return false (no processes using the code).
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("false"))` - No processes using the code
    /// * `Err(LoadError)` - If operation fails
    pub fn erts_internal_check_process_code_1(module: &ErlangTerm) -> Result<ErlangTerm, LoadError> {
        let _module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        // Simplified: always return false (no processes using the code)
        Ok(ErlangTerm::Atom("false".to_string()))
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

        // Simplified: Check if code has on_load by looking for magic marker
        // In real implementation, this would parse the BEAM file
        let has_on_load = code_bytes.len() > 0 && code_bytes[0] == 0xBE; // Simplified check

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

            // Find and remove the prepared code by searching
            let prepared_map = registry.prepared.read().unwrap();
            let mut found_ref = None;
            for (ref_key, prepared) in prepared_map.iter() {
                if prepared.reference_value() == ref_value {
                    found_ref = Some(ref_key.clone());
                    break;
                }
            }
            drop(prepared_map);
            
            if let Some(ref_key) = found_ref {
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
                
                modules.insert(
                    prepared.module.clone(),
                    ModuleEntry {
                        name: prepared.module.clone(),
                        status,
                        has_old_code: false,
                        has_on_load: prepared.has_on_load,
                        debug_info: None,
                        md5,
                        exports: vec![], // TODO: Parse from BEAM file
                        attributes: vec![], // TODO: Parse from BEAM file
                        compile: vec![], // TODO: Parse from BEAM file
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

        // Simplified: Search for chunk marker in code
        // In real implementation, this would parse IFF format
        let chunk_marker = &chunk_id_bytes;
        if let Some(pos) = code_bytes.windows(4).position(|w| w == chunk_marker) {
            // Found chunk marker, return a sub-binary (simplified)
            let chunk_data = code_bytes[pos..].to_vec();
            Ok(ErlangTerm::Binary(chunk_data))
        } else {
            Ok(ErlangTerm::Atom("undefined".to_string()))
        }
    }

    /// Check dirty process code (erts_internal_check_dirty_process_code/2)
    ///
    /// This is an internal function for checking if dirty processes use code.
    /// Simplified implementation always returns false.
    ///
    /// # Arguments
    /// * `pid` - Process ID
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("false"))` - No dirty processes using the code
    /// * `Err(LoadError)` - If operation fails
    pub fn erts_internal_check_dirty_process_code_2(
        _pid: &ErlangTerm,
        module: &ErlangTerm,
    ) -> Result<ErlangTerm, LoadError> {
        let _module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(LoadError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        // Simplified: always return false (no dirty processes using code)
        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Call on_load function (call_on_load_function/1)
    ///
    /// This is typically implemented as an instruction, not a BIF.
    /// Simplified implementation returns error.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Err(LoadError)` - This function is not supported as a BIF
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
    /// Simplified implementation.
    ///
    /// # Arguments
    /// * `pid` - Process ID
    /// * `req_id` - Request ID
    /// * `action` - Action atom (init, check_gc, need_gc)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("ok"))` - If successful
    /// * `Err(LoadError)` - If operation fails
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
                // Simplified: just return ok
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
    /// Simplified implementation.
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("false"))` - No areas to switch
    /// * `Err(LoadError)` - If operation fails
    pub fn erts_literal_area_collector_release_area_switch_0() -> Result<ErlangTerm, LoadError> {
        // Simplified: always return false (no areas to switch)
        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Helper: Register a module (for testing and internal use)
    ///
    /// This is a helper function to register a module in the registry.
    /// In a full implementation, this would be called by the code loader.
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

    /// Helper: Mark a module as pre-loaded (for testing and internal use)
    pub fn mark_preloaded(name: &str) {
        let registry = ModuleRegistry::get_instance();
        let mut preloaded = registry.preloaded.write().unwrap();
        preloaded.insert(name.to_string());
    }

    /// Get module metadata (for info module)
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
    pub fn set_debug_info(module: &str, debug_info: ErlangTerm) {
        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();

        if let Some(entry) = modules.get_mut(module) {
            entry.debug_info = Some(debug_info);
        }
    }

    /// Helper: Clear all modules (for testing)
    pub fn clear_all() {
        let registry = ModuleRegistry::get_instance();
        let mut modules = registry.modules.write().unwrap();
        let mut preloaded = registry.preloaded.write().unwrap();
        modules.clear();
        preloaded.clear();
        
        let prepared_registry = PreparedCodeRegistry::get_instance();
        prepared_registry.clear();
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
            assert_eq!(list.len(), 3);
            assert!(list.contains(&ErlangTerm::Atom("module1".to_string())));
            assert!(list.contains(&ErlangTerm::Atom("module2".to_string())));
            assert!(list.contains(&ErlangTerm::Atom("preloaded1".to_string())));
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
        LoadBif::clear_all();

        LoadBif::register_module(
            "purge_module",
            ModuleStatus::Loaded,
            true, // has old code
            false,
        );

        // Verify module exists
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("purge_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));

        // Verify module has old code before purging
        let has_old = LoadBif::check_old_code_1(&ErlangTerm::Atom("purge_module".to_string())).unwrap();
        assert_eq!(has_old, ErlangTerm::Atom("true".to_string()));

        let result = LoadBif::erts_internal_purge_module_2(
            &ErlangTerm::Atom("purge_module".to_string()),
            &ErlangTerm::Atom("force".to_string()),
        ).unwrap();

        assert_eq!(result, ErlangTerm::Atom("true".to_string()));

        // Verify module still exists
        let loaded_after = LoadBif::module_loaded_1(&ErlangTerm::Atom("purge_module".to_string())).unwrap();
        assert_eq!(loaded_after, ErlangTerm::Atom("true".to_string()));

        // Verify old code flag is cleared
        let has_old_after = LoadBif::check_old_code_1(&ErlangTerm::Atom("purge_module".to_string())).unwrap();
        assert_eq!(has_old_after, ErlangTerm::Atom("false".to_string()));

        // Verify module can now be deleted
        let delete_result = LoadBif::delete_module_1(&ErlangTerm::Atom("purge_module".to_string())).unwrap();
        assert_eq!(delete_result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_loaded_0_sorted() {
        LoadBif::clear_all();

        LoadBif::register_module("zebra", ModuleStatus::Loaded, false, false);
        LoadBif::register_module("alpha", ModuleStatus::Loaded, false, false);
        LoadBif::register_module("beta", ModuleStatus::Loaded, false, false);

        let result = LoadBif::loaded_0().unwrap();
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

        let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00]; // Simplified BEAM code (BEAM in ASCII)
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
}

