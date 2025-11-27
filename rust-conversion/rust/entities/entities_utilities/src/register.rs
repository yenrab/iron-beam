//! Register Operations
//!
//! Provides register handling functionality for processes and ports.
//! Based on register.c
//!
//! This module manages a table mapping atom names to process/port IDs.
//! In the entities layer, we use simplified types (String for names, u64 for IDs).
//! The actual Eterm/Process/Port types will be integrated in higher layers.

use std::collections::HashMap;

/// Register table mapping names to process/port IDs
///
/// In Erlang, processes and ports can be registered with atom names
/// for easy lookup. This table maintains those mappings.
pub struct Register {
    /// Maps registered name (atom) to process/port ID
    /// Key: atom name (as String for now, will be Eterm later)
    /// Value: process or port ID (as u64 for now, will be proper ID type later)
    table: HashMap<String, u64>,
}

/// Result of a register operation
#[derive(Debug, PartialEq, Eq)]
pub enum RegisterResult {
    /// Successfully registered
    Success,
    /// Name is already registered to a different process/port
    AlreadyRegistered,
    /// Invalid name (not an atom or undefined)
    InvalidName,
    /// Process/port is already registered with a different name
    AlreadyHasName,
    /// Process/port is not alive
    NotAlive,
}

impl Register {
    /// Create a new empty register table
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Register a name with a process/port ID
    ///
    /// # Arguments
    /// * `name` - The atom name to register (as String)
    /// * `id` - The process or port ID to register (as u64)
    ///
    /// # Returns
    /// * `RegisterResult::Success` if registration succeeded
    /// * `RegisterResult::AlreadyRegistered` if name is already registered to different ID
    /// * `RegisterResult::InvalidName` if name is invalid
    /// * `RegisterResult::AlreadyHasName` if ID is already registered with different name
    ///
    /// # Note
    /// This is a simplified version. The full C implementation checks:
    /// - If name is a valid atom (not undefined)
    /// - If process/port is alive
    /// - If process/port already has a registered name
    pub fn register_name(&mut self, name: &str, id: u64) -> RegisterResult {
        // Check for invalid name (empty string represents undefined in simplified version)
        if name.is_empty() {
            return RegisterResult::InvalidName;
        }

        // Check if name is already registered to a different ID
        if let Some(&existing_id) = self.table.get(name) {
            if existing_id != id {
                return RegisterResult::AlreadyRegistered;
            }
            // Same name and ID - already registered, return success
            return RegisterResult::Success;
        }

        // Check if ID is already registered with a different name
        for (existing_name, &existing_id) in &self.table {
            if existing_id == id && existing_name != name {
                return RegisterResult::AlreadyHasName;
            }
        }

        // Register the name
        self.table.insert(name.to_string(), id);
        RegisterResult::Success
    }

    /// Find the ID for a registered name
    ///
    /// # Arguments
    /// * `name` - The atom name to look up
    ///
    /// # Returns
    /// * `Some(id)` if name is registered
    /// * `None` if name is not registered
    ///
    /// This corresponds to `erts_whereis_name_to_id` in the C code.
    pub fn whereis_name(&self, name: &str) -> Option<u64> {
        self.table.get(name).copied()
    }

    /// Check if a name is registered
    ///
    /// # Arguments
    /// * `name` - The atom name to check
    ///
    /// # Returns
    /// `true` if the name is registered, `false` otherwise
    pub fn is_registered(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    /// Check if an ID is registered
    ///
    /// # Arguments
    /// * `id` - The process/port ID to check
    ///
    /// # Returns
    /// `Some(name)` if the ID is registered, `None` otherwise
    pub fn get_name_for_id(&self, id: u64) -> Option<String> {
        for (name, &registered_id) in &self.table {
            if registered_id == id {
                return Some(name.clone());
            }
        }
        None
    }

    /// Unregister a name
    ///
    /// # Arguments
    /// * `name` - The atom name to unregister
    ///
    /// # Returns
    /// * `true` if the name was registered and has been removed
    /// * `false` if the name was not registered
    ///
    /// This corresponds to `erts_unregister_name` in the C code.
    pub fn unregister_name(&mut self, name: &str) -> bool {
        self.table.remove(name).is_some()
    }

    /// Unregister by ID
    ///
    /// # Arguments
    /// * `id` - The process/port ID to unregister
    ///
    /// # Returns
    /// * `Some(name)` if the ID was registered and has been removed
    /// * `None` if the ID was not registered
    pub fn unregister_id(&mut self, id: u64) -> Option<String> {
        let name_to_remove = self.get_name_for_id(id);
        if let Some(ref name) = name_to_remove {
            self.table.remove(name);
        }
        name_to_remove
    }

    /// Get the number of registered names
    ///
    /// # Returns
    /// The number of registered names in the table
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Check if the register table is empty
    ///
    /// # Returns
    /// `true` if there are no registered names, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Clear all registrations
    ///
    /// Removes all registered names from the table.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// Get all registered names
    ///
    /// # Returns
    /// A vector of all registered names
    pub fn get_all_names(&self) -> Vec<String> {
        self.table.keys().cloned().collect()
    }

    /// Get all registered IDs
    ///
    /// # Returns
    /// A vector of all registered IDs
    pub fn get_all_ids(&self) -> Vec<u64> {
        self.table.values().copied().collect()
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creation() {
        let reg = Register::new();
        assert!(reg.is_empty());
        assert_eq!(reg.size(), 0);
    }

    #[test]
    fn test_register_name() {
        let mut reg = Register::new();
        
        // Register a name
        assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
        assert!(reg.is_registered("my_process"));
        assert_eq!(reg.whereis_name("my_process"), Some(123));
        assert_eq!(reg.size(), 1);
    }

    #[test]
    fn test_register_duplicate_name_same_id() {
        let mut reg = Register::new();
        
        // Register same name and ID twice - should succeed
        assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
        assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
        assert_eq!(reg.size(), 1);
    }

    #[test]
    fn test_register_duplicate_name_different_id() {
        let mut reg = Register::new();
        
        // Register name with one ID
        assert_eq!(reg.register_name("my_process", 123), RegisterResult::Success);
        
        // Try to register same name with different ID - should fail
        assert_eq!(
            reg.register_name("my_process", 456),
            RegisterResult::AlreadyRegistered
        );
        assert_eq!(reg.whereis_name("my_process"), Some(123)); // Original ID still registered
    }

    #[test]
    fn test_register_same_id_different_name() {
        let mut reg = Register::new();
        
        // Register ID with one name
        assert_eq!(reg.register_name("name1", 123), RegisterResult::Success);
        
        // Try to register same ID with different name - should fail
        assert_eq!(
            reg.register_name("name2", 123),
            RegisterResult::AlreadyHasName
        );
        assert_eq!(reg.get_name_for_id(123), Some("name1".to_string()));
    }

    #[test]
    fn test_register_invalid_name() {
        let mut reg = Register::new();
        
        // Empty string represents undefined/invalid name
        assert_eq!(
            reg.register_name("", 123),
            RegisterResult::InvalidName
        );
        assert!(reg.is_empty());
    }

    #[test]
    fn test_whereis_name() {
        let mut reg = Register::new();
        
        // Name not registered
        assert_eq!(reg.whereis_name("nonexistent"), None);
        
        // Register and find
        reg.register_name("my_process", 123);
        assert_eq!(reg.whereis_name("my_process"), Some(123));
    }

    #[test]
    fn test_get_name_for_id() {
        let mut reg = Register::new();
        
        // ID not registered
        assert_eq!(reg.get_name_for_id(999), None);
        
        // Register and find
        reg.register_name("my_process", 123);
        assert_eq!(reg.get_name_for_id(123), Some("my_process".to_string()));
    }

    #[test]
    fn test_unregister_name() {
        let mut reg = Register::new();
        
        // Unregister non-existent name
        assert!(!reg.unregister_name("nonexistent"));
        
        // Register and unregister
        reg.register_name("my_process", 123);
        assert!(reg.is_registered("my_process"));
        assert!(reg.unregister_name("my_process"));
        assert!(!reg.is_registered("my_process"));
        assert_eq!(reg.whereis_name("my_process"), None);
    }

    #[test]
    fn test_unregister_id() {
        let mut reg = Register::new();
        
        // Unregister non-existent ID
        assert_eq!(reg.unregister_id(999), None);
        
        // Register and unregister by ID
        reg.register_name("my_process", 123);
        assert_eq!(reg.unregister_id(123), Some("my_process".to_string()));
        assert!(!reg.is_registered("my_process"));
    }

    #[test]
    fn test_multiple_registrations() {
        let mut reg = Register::new();
        
        reg.register_name("process1", 1);
        reg.register_name("process2", 2);
        reg.register_name("process3", 3);
        
        assert_eq!(reg.size(), 3);
        assert_eq!(reg.whereis_name("process1"), Some(1));
        assert_eq!(reg.whereis_name("process2"), Some(2));
        assert_eq!(reg.whereis_name("process3"), Some(3));
    }

    #[test]
    fn test_clear() {
        let mut reg = Register::new();
        
        reg.register_name("process1", 1);
        reg.register_name("process2", 2);
        assert_eq!(reg.size(), 2);
        
        reg.clear();
        assert!(reg.is_empty());
        assert_eq!(reg.size(), 0);
        assert_eq!(reg.whereis_name("process1"), None);
    }

    #[test]
    fn test_get_all_names() {
        let mut reg = Register::new();
        
        reg.register_name("process1", 1);
        reg.register_name("process2", 2);
        reg.register_name("process3", 3);
        
        let names = reg.get_all_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"process1".to_string()));
        assert!(names.contains(&"process2".to_string()));
        assert!(names.contains(&"process3".to_string()));
    }

    #[test]
    fn test_get_all_ids() {
        let mut reg = Register::new();
        
        reg.register_name("process1", 1);
        reg.register_name("process2", 2);
        reg.register_name("process3", 3);
        
        let ids = reg.get_all_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }

    #[test]
    fn test_default() {
        let reg = Register::default();
        assert!(reg.is_empty());
    }
}
