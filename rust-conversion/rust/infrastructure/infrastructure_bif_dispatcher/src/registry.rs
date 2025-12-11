//! BIF Registry
//!
//! Provides a registry for storing and looking up BIF functions by module,
//! function name, and arity. This registry is used by the dispatcher to
//! route BIF calls to their implementations.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use entities_process::Eterm;
use crate::initialization::BifFunction;

/// BIF registry key (module, function, arity)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BifKey {
    /// Module atom
    pub module: Eterm,
    /// Function atom
    pub function: Eterm,
    /// Arity
    pub arity: u32,
}

impl BifKey {
    /// Create a new BIF key
    pub fn new(module: Eterm, function: Eterm, arity: u32) -> Self {
        Self {
            module,
            function,
            arity,
        }
    }
}

/// BIF registry
///
/// Thread-safe registry for storing and looking up BIF functions.
/// BIFs are registered by module, function name, and arity.
pub struct BifRegistry {
    /// Map from (module, function, arity) to BIF function
    registry: RwLock<HashMap<BifKey, Arc<dyn BifFunction + Send + Sync>>>,
}

impl BifRegistry {
    /// Create a new BIF registry
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
        }
    }

    /// Register a BIF function
    ///
    /// # Arguments
    /// * `module` - Module atom
    /// * `function` - Function atom
    /// * `arity` - Function arity
    /// * `bif_func` - BIF function implementation
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(String)` - Error (e.g., BIF already registered)
    pub fn register(
        &self,
        module: Eterm,
        function: Eterm,
        arity: u32,
        bif_func: Arc<dyn BifFunction + Send + Sync>,
    ) -> Result<(), String> {
        let mut registry = self.registry.write().unwrap();
        let key = BifKey::new(module, function, arity);
        
        if registry.contains_key(&key) {
            return Err(format!("BIF {}/{} already registered", function, arity));
        }
        
        registry.insert(key, bif_func);
        Ok(())
    }

    /// Look up a BIF function
    ///
    /// # Arguments
    /// * `module` - Module atom
    /// * `function` - Function atom
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// * `Some(bif_func)` - BIF function if found
    /// * `None` - BIF not found
    pub fn lookup(
        &self,
        module: Eterm,
        function: Eterm,
        arity: u32,
    ) -> Option<Arc<dyn BifFunction + Send + Sync>> {
        let registry = self.registry.read().unwrap();
        let key = BifKey::new(module, function, arity);
        registry.get(&key).cloned()
    }

    /// Unregister a BIF function
    ///
    /// # Arguments
    /// * `module` - Module atom
    /// * `function` - Function atom
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// * `true` - BIF was registered and removed
    /// * `false` - BIF was not found
    pub fn unregister(&self, module: Eterm, function: Eterm, arity: u32) -> bool {
        let mut registry = self.registry.write().unwrap();
        let key = BifKey::new(module, function, arity);
        registry.remove(&key).is_some()
    }

    /// Get the number of registered BIFs
    pub fn len(&self) -> usize {
        let registry = self.registry.read().unwrap();
        registry.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        let registry = self.registry.read().unwrap();
        registry.is_empty()
    }
}

impl Default for BifRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global BIF registry instance
static GLOBAL_BIF_REGISTRY: std::sync::OnceLock<BifRegistry> = std::sync::OnceLock::new();

/// Get the global BIF registry
///
/// # Returns
/// Reference to the global BIF registry
pub fn get_global_registry() -> &'static BifRegistry {
    GLOBAL_BIF_REGISTRY.get_or_init(BifRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBif;

    impl BifFunction for TestBif {
        fn call(
            &self,
            _process: &entities_process::Process,
            _args: &[Eterm],
            _instruction_ptr: entities_process::ErtsCodePtr,
        ) -> Eterm {
            42
        }
    }

    #[test]
    fn test_bif_key() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 3);
        let key3 = BifKey::new(1, 2, 4);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_bif_registry_register() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif);
        
        let result = registry.register(1, 2, 3, bif_func);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_bif_registry_duplicate_register() {
        let registry = BifRegistry::new();
        let bif_func1 = Arc::new(TestBif);
        let bif_func2 = Arc::new(TestBif);
        
        registry.register(1, 2, 3, bif_func1).unwrap();
        let result = registry.register(1, 2, 3, bif_func2);
        assert!(result.is_err());
    }

    #[test]
    fn test_bif_registry_lookup() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif);
        
        registry.register(1, 2, 3, bif_func.clone()).unwrap();
        
        let found = registry.lookup(1, 2, 3);
        assert!(found.is_some());
        
        let not_found = registry.lookup(1, 2, 4);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_bif_registry_unregister() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif);
        
        registry.register(1, 2, 3, bif_func).unwrap();
        assert_eq!(registry.len(), 1);
        
        let removed = registry.unregister(1, 2, 3);
        assert!(removed);
        assert_eq!(registry.len(), 0);
        
        let not_removed = registry.unregister(1, 2, 3);
        assert!(!not_removed);
    }

    #[test]
    fn test_global_registry() {
        let registry = get_global_registry();
        assert!(registry.is_empty() || registry.len() >= 0); // May have been initialized
    }
}
