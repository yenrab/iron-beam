//! Map Operations Module
//!
//! Provides map data structure operations.
//! Based on erl_map.c
//!
//! This module implements a persistent map data structure for Erlang terms.
//! Maps are key-value stores where both keys and values are Terms.

use crate::term_hashing::Term;

/// Map data structure
///
/// Internally stores key-value pairs as a vector. For efficient lookup,
/// we use hash-based indexing. This is a simplified implementation for
/// the entities layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    /// Key-value pairs stored in insertion order
    /// For efficient lookup, we maintain that keys are unique
    pairs: Vec<(Term, Term)>,
}

/// Map operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    /// Key not found in map
    KeyNotFound,
    /// Key already exists (for update operations that require key to exist)
    KeyExists,
}

impl Map {
    /// Create a new empty map
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
        }
    }

    /// Get the size of the map (number of key-value pairs)
    pub fn size(&self) -> usize {
        self.pairs.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Check if a key exists in the map
    pub fn is_key(&self, key: &Term) -> bool {
        self.find_index(key).is_some()
    }

    /// Get a value by key, returning None if key doesn't exist
    pub fn get(&self, key: &Term) -> Option<&Term> {
        self.find_index(key).map(|idx| &self.pairs[idx].1)
    }

    /// Find a key-value pair, returning Some((key, value)) if found, None otherwise
    pub fn find(&self, key: &Term) -> Option<(&Term, &Term)> {
        self.find_index(key).map(|idx| {
            let (k, v) = &self.pairs[idx];
            (k, v)
        })
    }

    /// Put a key-value pair into the map
    ///
    /// If the key already exists, the value is updated.
    /// Returns the previous value if the key existed, None otherwise.
    pub fn put(&mut self, key: Term, value: Term) -> Option<Term> {
        if let Some(idx) = self.find_index(&key) {
            let old_value = std::mem::replace(&mut self.pairs[idx].1, value);
            Some(old_value)
        } else {
            self.pairs.push((key, value));
            None
        }
    }

    /// Update a key-value pair in the map
    ///
    /// Returns Ok(previous_value) if the key exists, Err(MapError::KeyNotFound) otherwise.
    pub fn update(&mut self, key: &Term, value: Term) -> Result<Term, MapError> {
        if let Some(idx) = self.find_index(key) {
            let old_value = std::mem::replace(&mut self.pairs[idx].1, value);
            Ok(old_value)
        } else {
            Err(MapError::KeyNotFound)
        }
    }

    /// Remove a key from the map
    ///
    /// Returns the value if the key existed, None otherwise.
    pub fn remove(&mut self, key: &Term) -> Option<Term> {
        if let Some(idx) = self.find_index(key) {
            Some(self.pairs.remove(idx).1)
        } else {
            None
        }
    }

    /// Take a key-value pair from the map
    ///
    /// Returns Some((key, value)) if the key existed, None otherwise.
    pub fn take(&mut self, key: &Term) -> Option<(Term, Term)> {
        if let Some(idx) = self.find_index(key) {
            Some(self.pairs.remove(idx))
        } else {
            None
        }
    }

    /// Get all keys in the map
    pub fn keys(&self) -> Vec<&Term> {
        self.pairs.iter().map(|(k, _)| k).collect()
    }

    /// Get all values in the map
    pub fn values(&self) -> Vec<&Term> {
        self.pairs.iter().map(|(_, v)| v).collect()
    }

    /// Convert the map to a list of (key, value) pairs
    pub fn to_list(&self) -> Vec<(Term, Term)> {
        self.pairs.clone()
    }

    /// Create a map from a list of (key, value) pairs
    ///
    /// If duplicate keys exist, the last value for each key is kept.
    pub fn from_list(pairs: Vec<(Term, Term)>) -> Self {
        let mut map = Self::new();
        for (key, value) in pairs {
            map.put(key, value);
        }
        map
    }

    /// Merge two maps
    ///
    /// Keys from `other` take precedence over keys in `self`.
    /// Returns a new map containing all key-value pairs.
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (key, value) in &other.pairs {
            result.put(key.clone(), value.clone());
        }
        result
    }

    /// Find the index of a key in the pairs vector
    ///
    /// Uses linear search through the pairs. For small maps (typical in entities layer),
    /// this is efficient. For larger maps, hash-based optimization could be added.
    fn find_index(&self, key: &Term) -> Option<usize> {
        for (idx, (k, _)) in self.pairs.iter().enumerate() {
            if k == key {
                return Some(idx);
            }
        }
        None
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term_hashing::Term;

    #[test]
    fn test_map_creation() {
        let map = Map::new();
        assert!(map.is_empty());
        assert_eq!(map.size(), 0);
    }

    #[test]
    fn test_map_put_and_get() {
        let mut map = Map::new();
        let key = Term::Small(42);
        let value = Term::Small(100);

        // Put a new key-value pair
        assert_eq!(map.put(key.clone(), value.clone()), None);
        assert_eq!(map.size(), 1);
        assert!(map.is_key(&key));

        // Get the value
        assert_eq!(map.get(&key), Some(&value));

        // Update the value
        let new_value = Term::Small(200);
        assert_eq!(map.put(key.clone(), new_value.clone()), Some(value));
        assert_eq!(map.get(&key), Some(&new_value));
    }

    #[test]
    fn test_map_remove() {
        let mut map = Map::new();
        let key = Term::Small(42);
        let value = Term::Small(100);

        map.put(key.clone(), value.clone());
        assert_eq!(map.size(), 1);

        // Remove existing key
        assert_eq!(map.remove(&key), Some(value));
        assert_eq!(map.size(), 0);
        assert!(!map.is_key(&key));

        // Remove non-existent key
        assert_eq!(map.remove(&key), None);
    }

    #[test]
    fn test_map_update() {
        let mut map = Map::new();
        let key = Term::Small(42);
        let value = Term::Small(100);
        let new_value = Term::Small(200);

        // Update non-existent key should fail
        assert_eq!(map.update(&key, new_value.clone()), Err(MapError::KeyNotFound));

        // Put key first
        map.put(key.clone(), value.clone());

        // Update existing key should succeed
        assert_eq!(map.update(&key, new_value.clone()), Ok(value));
        assert_eq!(map.get(&key), Some(&new_value));
    }

    #[test]
    fn test_map_take() {
        let mut map = Map::new();
        let key = Term::Small(42);
        let value = Term::Small(100);

        map.put(key.clone(), value.clone());

        // Take existing key
        assert_eq!(map.take(&key), Some((key.clone(), value.clone())));
        assert_eq!(map.size(), 0);

        // Take non-existent key
        assert_eq!(map.take(&key), None);
    }

    #[test]
    fn test_map_find() {
        let mut map = Map::new();
        let key = Term::Small(42);
        let value = Term::Small(100);

        // Find non-existent key
        assert_eq!(map.find(&key), None);

        // Put and find
        map.put(key.clone(), value.clone());
        let found = map.find(&key);
        assert!(found.is_some());
        let (k, v) = found.unwrap();
        assert_eq!(*k, key);
        assert_eq!(*v, value);
    }

    #[test]
    fn test_map_keys_and_values() {
        let mut map = Map::new();
        let key1 = Term::Small(1);
        let value1 = Term::Small(10);
        let key2 = Term::Small(2);
        let value2 = Term::Small(20);

        map.put(key1.clone(), value1.clone());
        map.put(key2.clone(), value2.clone());

        let keys = map.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&key1));
        assert!(keys.contains(&&key2));

        let values = map.values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&&value1));
        assert!(values.contains(&&value2));
    }

    #[test]
    fn test_map_to_list() {
        let mut map = Map::new();
        let key1 = Term::Small(1);
        let value1 = Term::Small(10);
        let key2 = Term::Small(2);
        let value2 = Term::Small(20);

        map.put(key1.clone(), value1.clone());
        map.put(key2.clone(), value2.clone());

        let list = map.to_list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&(key1.clone(), value1.clone())));
        assert!(list.contains(&(key2.clone(), value2.clone())));
    }

    #[test]
    fn test_map_from_list() {
        let key1 = Term::Small(1);
        let value1 = Term::Small(10);
        let key2 = Term::Small(2);
        let value2 = Term::Small(20);

        let pairs = vec![
            (key1.clone(), value1.clone()),
            (key2.clone(), value2.clone()),
        ];

        let map = Map::from_list(pairs);
        assert_eq!(map.size(), 2);
        assert_eq!(map.get(&key1), Some(&value1));
        assert_eq!(map.get(&key2), Some(&value2));
    }

    #[test]
    fn test_map_from_list_duplicates() {
        let key = Term::Small(1);
        let value1 = Term::Small(10);
        let value2 = Term::Small(20);

        // Last value should win
        let pairs = vec![
            (key.clone(), value1.clone()),
            (key.clone(), value2.clone()),
        ];

        let map = Map::from_list(pairs);
        assert_eq!(map.size(), 1);
        assert_eq!(map.get(&key), Some(&value2));
    }

    #[test]
    fn test_map_merge() {
        let mut map1 = Map::new();
        let key1 = Term::Small(1);
        let value1 = Term::Small(10);
        let key2 = Term::Small(2);
        let value2 = Term::Small(20);

        map1.put(key1.clone(), value1.clone());
        map1.put(key2.clone(), value2.clone());

        let mut map2 = Map::new();
        let key3 = Term::Small(3);
        let value3 = Term::Small(30);
        let value2_new = Term::Small(200); // Override key2

        map2.put(key3.clone(), value3.clone());
        map2.put(key2.clone(), value2_new.clone());

        let merged = map1.merge(&map2);
        assert_eq!(merged.size(), 3);
        assert_eq!(merged.get(&key1), Some(&value1));
        assert_eq!(merged.get(&key2), Some(&value2_new)); // map2's value wins
        assert_eq!(merged.get(&key3), Some(&value3));
    }

    #[test]
    fn test_map_atom_keys() {
        let mut map = Map::new();
        let key = Term::Atom(1);
        let value = Term::Small(100);

        map.put(key.clone(), value.clone());
        assert_eq!(map.get(&key), Some(&value));
        assert!(map.is_key(&key));
    }

    #[test]
    fn test_map_tuple_keys() {
        let mut map = Map::new();
        let key = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        let value = Term::Small(100);

        map.put(key.clone(), value.clone());
        assert_eq!(map.get(&key), Some(&value));
        assert!(map.is_key(&key));
    }

    #[test]
    fn test_map_multiple_operations() {
        let mut map = Map::new();

        // Add multiple keys
        for i in 0..10 {
            let key = Term::Small(i);
            let value = Term::Small(i * 10);
            map.put(key, value);
        }

        assert_eq!(map.size(), 10);

        // Remove some keys
        for i in 0..5 {
            let key = Term::Small(i);
            map.remove(&key);
        }

        assert_eq!(map.size(), 5);

        // Verify remaining keys
        for i in 5..10 {
            let key = Term::Small(i);
            assert!(map.is_key(&key));
            assert_eq!(map.get(&key), Some(&Term::Small(i * 10)));
        }
    }

    #[test]
    fn test_map_default() {
        let map = Map::default();
        assert!(map.is_empty());
        assert_eq!(map.size(), 0);
    }
}

