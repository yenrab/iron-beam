//! Process Dictionary Module
//!
//! Provides process dictionary functionality.
//! Based on erl_process_dict.c
//!
//! The process dictionary is a key-value store where both keys and values
//! are Erlang terms. Each process has its own dictionary.

use std::collections::HashMap;
use entities_data_handling::term_hashing::Term;

/// Process dictionary
///
/// Stores key-value pairs where both keys and values are Erlang terms.
/// The dictionary is process-local and is cleared when the process terminates.
pub struct ProcessDict {
    dict: HashMap<Term, Term>,
}

impl ProcessDict {
    /// Create a new empty process dictionary
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    /// Put a value in the dictionary
    ///
    /// # Arguments
    /// * `key` - Dictionary key (any Erlang term)
    /// * `value` - Dictionary value (any Erlang term)
    ///
    /// # Returns
    /// Previous value associated with the key, if any
    ///
    /// # Examples
    /// ```
    /// use usecases_process_management::process_dict::ProcessDict;
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let mut dict = ProcessDict::new();
    /// let key = Term::Atom(1); // Atom index 1
    /// let value = Term::Small(42);
    /// dict.put(key.clone(), value);
    /// assert_eq!(dict.get(&key), Some(&Term::Small(42)));
    /// ```
    pub fn put(&mut self, key: Term, value: Term) -> Option<Term> {
        self.dict.insert(key, value)
    }

    /// Get a value from the dictionary
    ///
    /// # Arguments
    /// * `key` - Dictionary key to look up
    ///
    /// # Returns
    /// Value associated with the key, if any
    ///
    /// # Examples
    /// ```
    /// use usecases_process_management::process_dict::ProcessDict;
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let mut dict = ProcessDict::new();
    /// let key = Term::Atom(1);
    /// dict.put(key.clone(), Term::Small(100));
    /// assert_eq!(dict.get(&key), Some(&Term::Small(100)));
    /// ```
    pub fn get(&self, key: &Term) -> Option<&Term> {
        self.dict.get(key)
    }

    /// Remove a value from the dictionary
    ///
    /// # Arguments
    /// * `key` - Dictionary key to remove
    ///
    /// # Returns
    /// Value that was removed, if any
    pub fn erase(&mut self, key: &Term) -> Option<Term> {
        self.dict.remove(key)
    }

    /// Get all keys in the dictionary
    ///
    /// # Returns
    /// Vector of all keys in the dictionary
    pub fn keys(&self) -> Vec<&Term> {
        self.dict.keys().collect()
    }

    /// Clear the entire dictionary
    pub fn clear(&mut self) {
        self.dict.clear();
    }

    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }

    /// Get the number of entries in the dictionary
    pub fn len(&self) -> usize {
        self.dict.len()
    }
}

impl Default for ProcessDict {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_dict() {
        let mut dict = ProcessDict::new();
        let key = Term::Atom(1);
        let value = Term::Small(100);
        dict.put(key.clone(), value.clone());
        assert_eq!(dict.get(&key), Some(&value));
    }

    #[test]
    fn test_process_dict_erase() {
        let mut dict = ProcessDict::new();
        let key = Term::Atom(1);
        dict.put(key.clone(), Term::Small(100));
        assert_eq!(dict.erase(&key), Some(Term::Small(100)));
        assert_eq!(dict.get(&key), None);
    }

    #[test]
    fn test_process_dict_clear() {
        let mut dict = ProcessDict::new();
        dict.put(Term::Atom(1), Term::Small(100));
        dict.put(Term::Atom(2), Term::Small(200));
        assert_eq!(dict.len(), 2);
        dict.clear();
        assert!(dict.is_empty());
    }

    #[test]
    fn test_process_dict_keys() {
        let mut dict = ProcessDict::new();
        let key1 = Term::Atom(1);
        let key2 = Term::Atom(2);
        dict.put(key1.clone(), Term::Small(100));
        dict.put(key2.clone(), Term::Small(200));
        let keys = dict.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&key1));
        assert!(keys.contains(&&key2));
    }
}

