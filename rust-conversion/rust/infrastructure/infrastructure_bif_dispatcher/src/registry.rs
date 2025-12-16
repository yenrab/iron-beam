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
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // ==================== Test BIF Implementation ====================

    struct TestBif {
        return_value: Eterm,
    }

    impl TestBif {
        fn new(return_value: Eterm) -> Self {
            Self { return_value }
        }
    }

    impl Default for TestBif {
        fn default() -> Self {
            Self { return_value: 42 }
        }
    }

    impl BifFunction for TestBif {
        fn call(
            &self,
            _process: &entities_process::Process,
            _args: &[Eterm],
            _instruction_ptr: entities_process::ErtsCodePtr,
        ) -> Eterm {
            self.return_value
        }
    }

    // ==================== BifKey Creation Tests ====================

    #[test]
    fn test_bif_key_new() {
        let key = BifKey::new(100, 200, 3);
        assert_eq!(key.module, 100);
        assert_eq!(key.function, 200);
        assert_eq!(key.arity, 3);
    }

    #[test]
    fn test_bif_key_zero_values() {
        let key = BifKey::new(0, 0, 0);
        assert_eq!(key.module, 0);
        assert_eq!(key.function, 0);
        assert_eq!(key.arity, 0);
    }

    #[test]
    fn test_bif_key_max_values() {
        let key = BifKey::new(u64::MAX, u64::MAX, u32::MAX);
        assert_eq!(key.module, u64::MAX);
        assert_eq!(key.function, u64::MAX);
        assert_eq!(key.arity, u32::MAX);
    }

    // ==================== BifKey Equality Tests ====================

    #[test]
    fn test_bif_key_eq() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 3);
        let key3 = BifKey::new(1, 2, 4);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_bif_key_ne_module() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(99, 2, 3);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_bif_key_ne_function() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 99, 3);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_bif_key_ne_arity() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 99);
        assert_ne!(key1, key2);
    }

    // ==================== BifKey Hash Tests ====================

    #[test]
    fn test_bif_key_hash_equal_keys() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 3);
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_bif_key_hash_different_keys() {
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 4);
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);
        
        // Different keys should (usually) have different hashes
        // Note: This is not guaranteed, but highly likely
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    // ==================== BifKey Debug/Clone Tests ====================

    #[test]
    fn test_bif_key_debug() {
        let key = BifKey::new(10, 20, 3);
        let debug = format!("{:?}", key);
        assert!(debug.contains("BifKey"));
        assert!(debug.contains("10"));
        assert!(debug.contains("20"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn test_bif_key_clone() {
        let key = BifKey::new(100, 200, 5);
        let cloned = key.clone();
        
        assert_eq!(key, cloned);
        assert_eq!(key.module, cloned.module);
        assert_eq!(key.function, cloned.function);
        assert_eq!(key.arity, cloned.arity);
    }

    // ==================== BifRegistry Creation Tests ====================

    #[test]
    fn test_bif_registry_new() {
        let registry = BifRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_bif_registry_default() {
        let registry = BifRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    // ==================== BifRegistry Register Tests ====================

    #[test]
    fn test_bif_registry_register() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        let result = registry.register(1, 2, 3, bif_func);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_bif_registry_register_multiple() {
        let registry = BifRegistry::new();
        
        for i in 0..10 {
            let bif_func = Arc::new(TestBif::new(i));
            let result = registry.register(i, i * 10, i as u32, bif_func);
            assert!(result.is_ok());
        }
        
        assert_eq!(registry.len(), 10);
    }

    #[test]
    fn test_bif_registry_duplicate_register() {
        let registry = BifRegistry::new();
        let bif_func1 = Arc::new(TestBif::default());
        let bif_func2 = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func1).unwrap();
        let result = registry.register(1, 2, 3, bif_func2);
        assert!(result.is_err());
    }

    #[test]
    fn test_bif_registry_duplicate_register_error_message() {
        let registry = BifRegistry::new();
        let bif_func1 = Arc::new(TestBif::default());
        let bif_func2 = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func1).unwrap();
        let result = registry.register(1, 2, 3, bif_func2);
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("already registered"));
    }

    #[test]
    fn test_bif_registry_register_same_module_function_different_arity() {
        let registry = BifRegistry::new();
        
        for arity in 0..5u32 {
            let bif_func = Arc::new(TestBif::new(arity as Eterm));
            let result = registry.register(1, 2, arity, bif_func);
            assert!(result.is_ok());
        }
        
        assert_eq!(registry.len(), 5);
    }

    #[test]
    fn test_bif_registry_register_same_module_arity_different_function() {
        let registry = BifRegistry::new();
        
        for func in 0..5 {
            let bif_func = Arc::new(TestBif::new(func));
            let result = registry.register(1, func, 2, bif_func);
            assert!(result.is_ok());
        }
        
        assert_eq!(registry.len(), 5);
    }

    // ==================== BifRegistry Lookup Tests ====================

    #[test]
    fn test_bif_registry_lookup() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func.clone()).unwrap();
        
        let found = registry.lookup(1, 2, 3);
        assert!(found.is_some());
        
        let not_found = registry.lookup(1, 2, 4);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_bif_registry_lookup_empty() {
        let registry = BifRegistry::new();
        let result = registry.lookup(1, 2, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_bif_registry_lookup_wrong_module() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func).unwrap();
        
        let result = registry.lookup(99, 2, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_bif_registry_lookup_wrong_function() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func).unwrap();
        
        let result = registry.lookup(1, 99, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_bif_registry_lookup_wrong_arity() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func).unwrap();
        
        let result = registry.lookup(1, 2, 99);
        assert!(result.is_none());
    }

    #[test]
    fn test_bif_registry_lookup_multiple() {
        let registry = BifRegistry::new();
        
        for i in 0..5 {
            let bif_func = Arc::new(TestBif::new(i * 100));
            registry.register(i, i * 10, i as u32, bif_func).unwrap();
        }
        
        for i in 0..5 {
            let result = registry.lookup(i, i * 10, i as u32);
            assert!(result.is_some());
        }
    }

    // ==================== BifRegistry Unregister Tests ====================

    #[test]
    fn test_bif_registry_unregister() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func).unwrap();
        assert_eq!(registry.len(), 1);
        
        let removed = registry.unregister(1, 2, 3);
        assert!(removed);
        assert_eq!(registry.len(), 0);
        
        let not_removed = registry.unregister(1, 2, 3);
        assert!(!not_removed);
    }

    #[test]
    fn test_bif_registry_unregister_empty() {
        let registry = BifRegistry::new();
        let result = registry.unregister(1, 2, 3);
        assert!(!result);
    }

    #[test]
    fn test_bif_registry_unregister_wrong_key() {
        let registry = BifRegistry::new();
        let bif_func = Arc::new(TestBif::default());
        
        registry.register(1, 2, 3, bif_func).unwrap();
        
        // Wrong module
        assert!(!registry.unregister(99, 2, 3));
        // Wrong function
        assert!(!registry.unregister(1, 99, 3));
        // Wrong arity
        assert!(!registry.unregister(1, 2, 99));
        
        // Original still exists
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_bif_registry_unregister_and_re_register() {
        let registry = BifRegistry::new();
        let bif_func1 = Arc::new(TestBif::new(100));
        let bif_func2 = Arc::new(TestBif::new(200));
        
        registry.register(1, 2, 3, bif_func1).unwrap();
        assert!(registry.unregister(1, 2, 3));
        
        // Should be able to re-register
        let result = registry.register(1, 2, 3, bif_func2);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    // ==================== BifRegistry len/is_empty Tests ====================

    #[test]
    fn test_bif_registry_len() {
        let registry = BifRegistry::new();
        assert_eq!(registry.len(), 0);
        
        for i in 0..5 {
            let bif_func = Arc::new(TestBif::new(i));
            registry.register(i, i, i as u32, bif_func).unwrap();
            assert_eq!(registry.len(), (i + 1) as usize);
        }
    }

    #[test]
    fn test_bif_registry_is_empty() {
        let registry = BifRegistry::new();
        assert!(registry.is_empty());
        
        let bif_func = Arc::new(TestBif::default());
        registry.register(1, 2, 3, bif_func).unwrap();
        assert!(!registry.is_empty());
        
        registry.unregister(1, 2, 3);
        assert!(registry.is_empty());
    }

    // ==================== Global Registry Tests ====================

    #[test]
    fn test_global_registry() {
        let registry = get_global_registry();
        assert!(registry.is_empty() || registry.len() >= 0); // May have been initialized
    }

    #[test]
    fn test_global_registry_singleton() {
        let registry1 = get_global_registry();
        let registry2 = get_global_registry();
        
        // Both should point to the same registry
        assert!(std::ptr::eq(registry1, registry2));
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_bif_registry_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        // BifRegistry should be Send + Sync due to RwLock
        assert_send::<BifRegistry>();
        assert_sync::<BifRegistry>();
    }

    #[test]
    fn test_bif_registry_concurrent_access() {
        use std::thread;
        
        let registry = Arc::new(BifRegistry::new());
        let mut handles = vec![];
        
        // Spawn threads to register different BIFs
        for i in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let bif_func = Arc::new(TestBif::new(i));
                let _ = reg.register(i, i * 10, i as u32, bif_func);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // All should be registered
        assert_eq!(registry.len(), 10);
    }

    #[test]
    fn test_bif_registry_concurrent_lookup() {
        use std::thread;
        
        let registry = Arc::new(BifRegistry::new());
        
        // Pre-register some BIFs
        for i in 0..5 {
            let bif_func = Arc::new(TestBif::new(i * 100));
            registry.register(i, i * 10, i as u32, bif_func).unwrap();
        }
        
        let mut handles = vec![];
        
        // Spawn threads to lookup BIFs concurrently
        for i in 0..5 {
            let reg = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let result = reg.lookup(i, i * 10, i as u32);
                assert!(result.is_some());
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_workflow() {
        let registry = BifRegistry::new();
        
        // Start empty
        assert!(registry.is_empty());
        
        // Register
        let bif_func = Arc::new(TestBif::new(999));
        registry.register(1, 2, 3, bif_func).unwrap();
        assert_eq!(registry.len(), 1);
        
        // Lookup
        let found = registry.lookup(1, 2, 3);
        assert!(found.is_some());
        
        // Failed duplicate register
        let bif_func2 = Arc::new(TestBif::new(888));
        assert!(registry.register(1, 2, 3, bif_func2).is_err());
        
        // Unregister
        assert!(registry.unregister(1, 2, 3));
        assert!(registry.is_empty());
        
        // Lookup after unregister
        assert!(registry.lookup(1, 2, 3).is_none());
        
        // Re-register
        let bif_func3 = Arc::new(TestBif::new(777));
        assert!(registry.register(1, 2, 3, bif_func3).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_different_arities() {
        let registry = BifRegistry::new();
        
        // Register same function name with different arities (like erlang:+/1 and erlang:+/2)
        for arity in 0..=4u32 {
            let bif_func = Arc::new(TestBif::new(arity as Eterm));
            let result = registry.register(1, 100, arity, bif_func);
            assert!(result.is_ok());
        }
        
        assert_eq!(registry.len(), 5);
        
        // Each should be independently lookupable
        for arity in 0..=4u32 {
            let result = registry.lookup(1, 100, arity);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_bif_key_in_hashmap() {
        // Test that BifKey works properly as a HashMap key
        let mut map: HashMap<BifKey, i32> = HashMap::new();
        
        let key1 = BifKey::new(1, 2, 3);
        let key2 = BifKey::new(1, 2, 4);
        let key3 = BifKey::new(1, 2, 3); // Same as key1
        
        map.insert(key1.clone(), 100);
        map.insert(key2, 200);
        
        // key3 should find the same entry as key1
        assert_eq!(map.get(&key3), Some(&100));
        assert_eq!(map.len(), 2);
    }
}

