//! Persistent Term Storage Built-in Functions
//!
//! Provides persistent term storage - a global key-value store that survives
//! process restarts. Optimized for frequent reads and infrequent writes.
//!
//! This module implements safe Rust equivalents of Erlang persistent term BIFs.

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
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Error type for persistent term operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentError {
    /// Bad argument (e.g., key not found)
    BadArgument(String),
}

/// Persistent term storage
///
/// Uses a thread-safe hash map to store key-value pairs.
/// The storage is shared across all processes and persists for the lifetime
/// of the runtime system.
#[derive(Clone, Debug)]
pub struct PersistentStorage {
    /// Thread-safe storage for persistent terms
    storage: Arc<RwLock<HashMap<ErlangTerm, ErlangTerm>>>,
}

impl PersistentStorage {
    /// Create a new persistent storage instance
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the singleton instance
    ///
    /// In a full implementation, this would use a global singleton.
    /// For now, we'll use a static instance created on first access.
    fn get_instance() -> &'static PersistentStorage {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<PersistentStorage> = OnceLock::new();
        INSTANCE.get_or_init(|| PersistentStorage::new())
    }
}

/// Persistent Term Built-in Functions
pub struct PersistentBif;

impl PersistentBif {
    /// Store a persistent term (put/2)
    ///
    /// Stores a key-value pair in the persistent term storage.
    /// If the key already exists, the value is updated.
    ///
    /// # Arguments
    /// * `key` - Key to store
    /// * `value` - Value to store
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("ok"))` - If successful
    /// * `Err(PersistentError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Store a simple key-value pair
    /// let key = ErlangTerm::Atom("my_key".to_string());
    /// let value = ErlangTerm::Integer(42);
    /// let result = PersistentBif::put_2(&key, &value).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Update an existing key
    /// let new_value = ErlangTerm::Integer(100);
    /// let result = PersistentBif::put_2(&key, &new_value).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    ///
    /// // Store with different key types
    /// let tuple_key = ErlangTerm::Tuple(vec![
    ///     ErlangTerm::Atom("module".to_string()),
    ///     ErlangTerm::Atom("name".to_string()),
    /// ]);
    /// let result = PersistentBif::put_2(&tuple_key, &ErlangTerm::Integer(1)).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    /// ```
    pub fn put_2(key: &ErlangTerm, value: &ErlangTerm) -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let mut map = storage.storage.write().unwrap();
        
        // Check if key already exists with same value
        if let Some(existing_value) = map.get(key) {
            if existing_value == value {
                // Same value, no need to update
                return Ok(ErlangTerm::Atom("ok".to_string()));
            }
        }
        
        // Store or update the key-value pair
        map.insert(key.clone(), value.clone());
        Ok(ErlangTerm::Atom("ok".to_string()))
    }

    /// Get all persistent terms (get/0)
    ///
    /// Returns a list of all key-value pairs as tuples `{Key, Value}`.
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - List of `{Key, Value}` tuples
    /// * `Err(PersistentError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Clear storage and store some terms
    /// PersistentBif::erase_all_0().unwrap();
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key1".to_string()),
    ///     &ErlangTerm::Integer(1),
    /// ).unwrap();
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key2".to_string()),
    ///     &ErlangTerm::Integer(2),
    /// ).unwrap();
    ///
    /// // Get all stored terms
    /// let all = PersistentBif::get_0().unwrap();
    /// if let ErlangTerm::List(items) = all {
    ///     assert!(items.len() >= 2);
    /// }
    ///
    /// // Get all from empty storage
    /// PersistentBif::erase_all_0().unwrap();
    /// let empty = PersistentBif::get_0().unwrap();
    /// if let ErlangTerm::List(items) = empty {
    ///     assert_eq!(items.len(), 0);
    /// }
    /// ```
    pub fn get_0() -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let map = storage.storage.read().unwrap();
        
        let mut result = Vec::new();
        for (key, value) in map.iter() {
            result.push(ErlangTerm::Tuple(vec![
                key.clone(),
                value.clone(),
            ]));
        }
        
        Ok(ErlangTerm::List(result))
    }

    /// Get a persistent term by key (get/1)
    ///
    /// Returns the value associated with the key, or an error if not found.
    ///
    /// # Arguments
    /// * `key` - Key to look up
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - The value associated with the key
    /// * `Err(PersistentError)` - If key is not found
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get existing key
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("my_key".to_string()),
    ///     &ErlangTerm::Integer(42),
    /// ).unwrap();
    /// let value = PersistentBif::get_1(&ErlangTerm::Atom("my_key".to_string())).unwrap();
    /// assert_eq!(value, ErlangTerm::Integer(42));
    ///
    /// // Get non-existent key (returns error)
    /// let result = PersistentBif::get_1(&ErlangTerm::Atom("nonexistent".to_string()));
    /// assert!(result.is_err());
    ///
    /// // Get with different value types
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("string_key".to_string()),
    ///     &ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]),
    /// ).unwrap();
    /// let list_value = PersistentBif::get_1(&ErlangTerm::Atom("string_key".to_string())).unwrap();
    /// assert!(matches!(list_value, ErlangTerm::List(_)));
    /// ```
    pub fn get_1(key: &ErlangTerm) -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let map = storage.storage.read().unwrap();
        
        map.get(key)
            .cloned()
            .ok_or_else(|| PersistentError::BadArgument(
                format!("Key not found: {:?}", key)
            ))
    }

    /// Get a persistent term by key with default (get/2)
    ///
    /// Returns the value associated with the key, or the default value if not found.
    ///
    /// # Arguments
    /// * `key` - Key to look up
    /// * `default` - Default value to return if key is not found
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - The value associated with the key, or the default
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get non-existent key with default
    /// let value = PersistentBif::get_2(
    ///     &ErlangTerm::Atom("nonexistent".to_string()),
    ///     &ErlangTerm::Integer(0),
    /// ).unwrap();
    /// assert_eq!(value, ErlangTerm::Integer(0));
    ///
    /// // Get existing key (returns actual value, not default)
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("existing".to_string()),
    ///     &ErlangTerm::Integer(100),
    /// ).unwrap();
    /// let value = PersistentBif::get_2(
    ///     &ErlangTerm::Atom("existing".to_string()),
    ///     &ErlangTerm::Integer(0),
    /// ).unwrap();
    /// assert_eq!(value, ErlangTerm::Integer(100));
    ///
    /// // Get with different default types
    /// let value = PersistentBif::get_2(
    ///     &ErlangTerm::Atom("missing".to_string()),
    ///     &ErlangTerm::Atom("default".to_string()),
    /// ).unwrap();
    /// assert_eq!(value, ErlangTerm::Atom("default".to_string()));
    /// ```
    pub fn get_2(key: &ErlangTerm, default: &ErlangTerm) -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let map = storage.storage.read().unwrap();
        
        Ok(map.get(key)
            .cloned()
            .unwrap_or_else(|| default.clone()))
    }

    /// Erase a persistent term (erase/1)
    ///
    /// Removes a key-value pair from the persistent term storage.
    ///
    /// # Arguments
    /// * `key` - Key to erase
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If key was found and erased
    /// * `Ok(ErlangTerm::Atom("false"))` - If key was not found
    /// * `Err(PersistentError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Erase existing key
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("my_key".to_string()),
    ///     &ErlangTerm::Integer(42),
    /// ).unwrap();
    /// let result = PersistentBif::erase_1(&ErlangTerm::Atom("my_key".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    ///
    /// // Erase non-existent key
    /// let result = PersistentBif::erase_1(&ErlangTerm::Atom("nonexistent".to_string())).unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    ///
    /// // Erase and verify it's gone
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("temp".to_string()),
    ///     &ErlangTerm::Integer(1),
    /// ).unwrap();
    /// PersistentBif::erase_1(&ErlangTerm::Atom("temp".to_string())).unwrap();
    /// assert!(PersistentBif::get_1(&ErlangTerm::Atom("temp".to_string())).is_err());
    /// ```
    pub fn erase_1(key: &ErlangTerm) -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let mut map = storage.storage.write().unwrap();
        
        if map.remove(key).is_some() {
            Ok(ErlangTerm::Atom("true".to_string()))
        } else {
            Ok(ErlangTerm::Atom("false".to_string()))
        }
    }

    /// Erase all persistent terms (erts_internal_erase_persistent_terms/0)
    ///
    /// Removes all key-value pairs from the persistent term storage.
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - Always succeeds
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Erase all from populated storage
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key1".to_string()),
    ///     &ErlangTerm::Integer(1),
    /// ).unwrap();
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key2".to_string()),
    ///     &ErlangTerm::Integer(2),
    /// ).unwrap();
    /// let result = PersistentBif::erase_all_0().unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    ///
    /// // Verify all are erased
    /// let all = PersistentBif::get_0().unwrap();
    /// if let ErlangTerm::List(items) = all {
    ///     assert_eq!(items.len(), 0);
    /// }
    ///
    /// // Erase all from empty storage
    /// let result = PersistentBif::erase_all_0().unwrap();
    /// assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    /// ```
    pub fn erase_all_0() -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let mut map = storage.storage.write().unwrap();
        
        map.clear();
        Ok(ErlangTerm::Atom("true".to_string()))
    }

    /// Get information about persistent terms (info/0)
    ///
    /// Returns a map with information about the persistent term storage:
    /// - `count`: Number of stored terms
    /// - `memory`: Approximate memory usage in bytes
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Map)` - Map with info
    /// * `Err(PersistentError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::persistent::PersistentBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get info from empty storage
    /// PersistentBif::erase_all_0().unwrap();
    /// let info = PersistentBif::info_0().unwrap();
    /// // info contains count: 0
    ///
    /// // Get info from populated storage
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key1".to_string()),
    ///     &ErlangTerm::Integer(1),
    /// ).unwrap();
    /// PersistentBif::put_2(
    ///     &ErlangTerm::Atom("key2".to_string()),
    ///     &ErlangTerm::Integer(2),
    /// ).unwrap();
    /// let info = PersistentBif::info_0().unwrap();
    /// // info contains count: 2 and memory information
    ///
    /// // Get info after modifications
    /// PersistentBif::erase_1(&ErlangTerm::Atom("key1".to_string())).unwrap();
    /// let info_after = PersistentBif::info_0().unwrap();
    /// // info_after contains updated count
    /// ```
    pub fn info_0() -> Result<ErlangTerm, PersistentError> {
        let storage = PersistentStorage::get_instance();
        let map = storage.storage.read().unwrap();
        
        let count = map.len();
        
        // Calculate approximate memory usage
        // This is a simplified calculation - in reality, we'd need to
        // recursively calculate the size of each term
        let memory = count * 64; // Rough estimate: 64 bytes per entry
        
        let mut info_map = HashMap::new();
        info_map.insert(
            ErlangTerm::Atom("count".to_string()),
            ErlangTerm::Integer(count as i64),
        );
        info_map.insert(
            ErlangTerm::Atom("memory".to_string()),
            ErlangTerm::Integer(memory as i64),
        );
        
        Ok(ErlangTerm::Map(info_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_2_basic() {
        // Clear storage first
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("test_key".to_string());
        let value = ErlangTerm::Integer(42);
        let result = PersistentBif::put_2(&key, &value).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_put_2_update() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("update_key".to_string());
        let value1 = ErlangTerm::Integer(1);
        let value2 = ErlangTerm::Integer(2);
        
        // First put
        PersistentBif::put_2(&key, &value1).unwrap();
        
        // Update with new value
        let result = PersistentBif::put_2(&key, &value2).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
        
        // Verify update
        let retrieved = PersistentBif::get_1(&key).unwrap();
        assert_eq!(retrieved, value2);
    }

    #[test]
    fn test_put_2_same_value() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("same_key".to_string());
        let value = ErlangTerm::Integer(42);
        
        // First put
        PersistentBif::put_2(&key, &value).unwrap();
        
        // Put same value again (should return ok without error)
        let result = PersistentBif::put_2(&key, &value).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    }

    #[test]
    fn test_get_1_found() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("get_key".to_string());
        let value = ErlangTerm::Integer(100);
        
        PersistentBif::put_2(&key, &value).unwrap();
        
        let result = PersistentBif::get_1(&key).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_get_1_not_found() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("nonexistent".to_string());
        let result = PersistentBif::get_1(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_2_with_default() {
        // Clear storage first to ensure clean state
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("get2_key".to_string());
        let value = ErlangTerm::Integer(200);
        let default = ErlangTerm::Integer(0);
        
        // Get with default when key doesn't exist
        let result1 = PersistentBif::get_2(&key, &default).unwrap();
        assert_eq!(result1, default);
        
        // Store the key
        PersistentBif::put_2(&key, &value).unwrap();
        
        // Get with default when key exists (should return stored value)
        let result2 = PersistentBif::get_2(&key, &default).unwrap();
        assert_eq!(result2, value);
        
        // Clean up
        let _ = PersistentBif::erase_all_0();
    }

    #[test]
    fn test_get_0_empty() {
        let _ = PersistentBif::erase_all_0();
        
        let result = PersistentBif::get_0().unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 0);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_get_0_with_entries() {
        // Clear storage first to ensure clean state
        let _ = PersistentBif::erase_all_0();
        
        let key1 = ErlangTerm::Atom("get0_key1".to_string());
        let value1 = ErlangTerm::Integer(1);
        let key2 = ErlangTerm::Atom("get0_key2".to_string());
        let value2 = ErlangTerm::Integer(2);
        
        PersistentBif::put_2(&key1, &value1).unwrap();
        PersistentBif::put_2(&key2, &value2).unwrap();
        
        let result = PersistentBif::get_0().unwrap();
        if let ErlangTerm::List(list) = result {
            // Should have at least 2 entries (might have more from other tests)
            // If list is empty or has fewer than 2, it means erase_all didn't work or there's a race condition
            // In that case, just verify our specific entries are present
            if list.len() < 2 {
                // Re-put the entries to ensure they exist
                PersistentBif::put_2(&key1, &value1).unwrap();
                PersistentBif::put_2(&key2, &value2).unwrap();
                let result2 = PersistentBif::get_0().unwrap();
                if let ErlangTerm::List(list2) = result2 {
                    assert!(list2.len() >= 2, "After re-putting, should have at least 2 entries");
                } else {
                    panic!("Expected List");
                }
            }
            // Check that both entries are present
            let mut found_key1 = false;
            let mut found_key2 = false;
            for entry in &list {
                if let ErlangTerm::Tuple(tuple) = entry {
                    if tuple.len() == 2 {
                        if tuple[0] == key1 && tuple[1] == value1 {
                            found_key1 = true;
                        }
                        if tuple[0] == key2 && tuple[1] == value2 {
                            found_key2 = true;
                        }
                    }
                }
            }
            assert!(found_key1, "key1 not found in result");
            assert!(found_key2, "key2 not found in result");
        } else {
            panic!("Expected List");
        }
        
        // Clean up
        let _ = PersistentBif::erase_all_0();
    }

    #[test]
    fn test_erase_1_found() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("erase_key".to_string());
        let value = ErlangTerm::Integer(300);
        
        PersistentBif::put_2(&key, &value).unwrap();
        
        let result = PersistentBif::erase_1(&key).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
        
        // Verify it's gone
        let get_result = PersistentBif::get_1(&key);
        assert!(get_result.is_err());
    }

    #[test]
    fn test_erase_1_not_found() {
        let _ = PersistentBif::erase_all_0();
        
        let key = ErlangTerm::Atom("nonexistent_erase".to_string());
        let result = PersistentBif::erase_1(&key).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_erase_all_0() {
        // Clear all first to ensure clean state
        let _ = PersistentBif::erase_all_0();
        
        // Verify it's empty
        let all_before = PersistentBif::get_0().unwrap();
        if let ErlangTerm::List(list) = &all_before {
            let initial_count = list.len();
            
            // Add some entries
            PersistentBif::put_2(
                &ErlangTerm::Atom("key1".to_string()),
                &ErlangTerm::Integer(1),
            ).unwrap();
            PersistentBif::put_2(
                &ErlangTerm::Atom("key2".to_string()),
                &ErlangTerm::Integer(2),
            ).unwrap();
            
            // Verify they exist (should be initial_count + 2)
            let all = PersistentBif::get_0().unwrap();
            if let ErlangTerm::List(list) = all {
                assert_eq!(list.len(), initial_count + 2);
            } else {
                panic!("Expected List");
            }
            
            // Erase all
            let result = PersistentBif::erase_all_0().unwrap();
            assert_eq!(result, ErlangTerm::Atom("true".to_string()));
            
            // Verify they're gone
            let all_after = PersistentBif::get_0().unwrap();
            if let ErlangTerm::List(list) = all_after {
                assert_eq!(list.len(), 0);
            } else {
                panic!("Expected List");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_info_0_empty() {
        let _ = PersistentBif::erase_all_0();
        
        let info = PersistentBif::info_0().unwrap();
        if let ErlangTerm::Map(map) = info {
            let count = map.get(&ErlangTerm::Atom("count".to_string()));
            assert_eq!(count, Some(&ErlangTerm::Integer(0)));
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_info_0_with_entries() {
        let _ = PersistentBif::erase_all_0();
        
        PersistentBif::put_2(
            &ErlangTerm::Atom("info_key1".to_string()),
            &ErlangTerm::Integer(1),
        ).unwrap();
        PersistentBif::put_2(
            &ErlangTerm::Atom("info_key2".to_string()),
            &ErlangTerm::Integer(2),
        ).unwrap();
        
        let info = PersistentBif::info_0().unwrap();
        if let ErlangTerm::Map(map) = info {
            let count = map.get(&ErlangTerm::Atom("count".to_string()));
            assert_eq!(count, Some(&ErlangTerm::Integer(2)));
            
            let memory = map.get(&ErlangTerm::Atom("memory".to_string()));
            assert!(memory.is_some());
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_persistent_terms_isolation() {
        let _ = PersistentBif::erase_all_0();
        
        // Store different types
        PersistentBif::put_2(
            &ErlangTerm::Atom("atom_key".to_string()),
            &ErlangTerm::Atom("atom_value".to_string()),
        ).unwrap();
        
        PersistentBif::put_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::List(vec![
                ErlangTerm::Integer(1),
                ErlangTerm::Integer(2),
            ]),
        ).unwrap();
        
        PersistentBif::put_2(
            &ErlangTerm::Tuple(vec![ErlangTerm::Atom("tuple_key".to_string())]),
            &ErlangTerm::Float(3.14),
        ).unwrap();
        
        // Verify all can be retrieved
        let atom_val = PersistentBif::get_1(&ErlangTerm::Atom("atom_key".to_string())).unwrap();
        assert_eq!(atom_val, ErlangTerm::Atom("atom_value".to_string()));
        
        let list_val = PersistentBif::get_1(&ErlangTerm::Integer(123)).unwrap();
        if let ErlangTerm::List(list) = list_val {
            assert_eq!(list.len(), 2);
        } else {
            panic!("Expected List");
        }
        
        let tuple_key = ErlangTerm::Tuple(vec![ErlangTerm::Atom("tuple_key".to_string())]);
        let float_val = PersistentBif::get_1(&tuple_key).unwrap();
        assert_eq!(float_val, ErlangTerm::Float(3.14));
    }
}

