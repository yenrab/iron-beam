//! Register Operations Module
//!
//! This module provides register handling functionality for processes and ports
//! in the Erlang/OTP runtime system, providing a table that maps atom names to process/port
//! identifiers.
//!
//! # Purpose
//!
//! Erlang allows processes and ports to be registered with atom names, enabling
//! them to be found by name using functions like `whereis/1` and `register/2`.
//! This module provides the core functionality for managing these registrations:
//!
//! - **Name Registration**: Register a process or port with an atom name, ensuring
//!   that each name maps to exactly one ID and each ID maps to at most one name.
//!   This bidirectional constraint prevents name conflicts and ensures consistent
//!   lookup behavior.
//!
//! - **Name Lookup**: Find the process/port ID associated with a registered name,
//!   enabling the `whereis/1` BIF functionality. This is essential for Erlang's
//!   distributed programming model where processes are often referenced by name.
//!
//! - **Reverse Lookup**: Find the name associated with a process/port ID, enabling
//!   queries about whether a process is registered and what name it uses.
//!
//! - **Registration Management**: Unregister names, clear the entire table, and query
//!   the state of the registration table. This supports process lifecycle management
//!   and cleanup operations.
//!
//! # Implementation Details
//!
//! This module uses Rust's standard `HashMap` for efficient name-to-ID lookups.
//! In the entities layer, we use simplified types (`String` for names, `u64` for IDs)
//! to maintain the layer's independence. The actual `Eterm`/`Process`/`Port` types
//! will be integrated in higher layers of the CLEAN architecture.
//!
//! The implementation enforces the constraint that each name maps to exactly one ID
//! and each ID maps to at most one name. Attempts to register a name with a different
//! ID, or an ID with a different name, will fail with appropriate error codes.
//!
//! # Examples
//!
//! ## Basic Registration
//!
//! ```rust
//! use entities_utilities::{Register, RegisterResult};
//!
//! let mut reg = Register::new();
//!
//! // Register a process with a name
//! reg.register_name("my_process", 123);
//!
//! // Look up the process by name
//! let id = reg.whereis_name("my_process");
//! assert_eq!(id, Some(123));
//! ```
//!
//! ## Reverse Lookup
//!
//! ```rust
//! use entities_utilities::Register;
//!
//! let mut reg = Register::new();
//! reg.register_name("my_process", 123);
//!
//! // Find the name for an ID
//! let name = reg.get_name_for_id(123);
//! assert_eq!(name, Some("my_process".to_string()));
//! ```
//!
//! ## Error Handling
//!
//! ```rust
//! use entities_utilities::{Register, RegisterResult};
//!
//! let mut reg = Register::new();
//! reg.register_name("process1", 100);
//!
//! // Try to register same name with different ID - fails
//! assert_eq!(
//!     reg.register_name("process1", 200),
//!     RegisterResult::AlreadyRegistered
//! );
//! ```

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

/// Register table mapping atom names to process/port IDs.
///
/// This struct maintains a bidirectional mapping between atom names and
/// process/port identifiers. In Erlang, processes and ports can be registered
/// with atom names for easy lookup, enabling the `whereis/1` and `register/2`
/// BIF functionality.
///
/// # Purpose
///
/// The register table is essential for Erlang's distributed programming model,
/// where processes are often referenced by name rather than by PID. This enables
/// processes to find each other by name, which is crucial for supervisor trees,
/// registered processes, and distributed Erlang applications.
///
/// The table enforces the constraint that each name maps to exactly one ID and
/// each ID maps to at most one name. This prevents name conflicts and ensures
/// consistent lookup behavior.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use entities_utilities::Register;
///
/// let mut reg = Register::new();
/// reg.register_name("my_process", 123);
/// assert_eq!(reg.whereis_name("my_process"), Some(123));
/// ```
///
/// ## Multiple Registrations
///
/// ```rust
/// use entities_utilities::Register;
///
/// let mut reg = Register::new();
/// reg.register_name("process1", 100);
/// reg.register_name("process2", 200);
/// reg.register_name("process3", 300);
///
/// assert_eq!(reg.size(), 3);
/// ```
///
/// ## Lifecycle Management
///
/// ```rust
/// use entities_utilities::Register;
///
/// let mut reg = Register::new();
/// reg.register_name("temp_process", 123);
/// // ... use the process ...
/// reg.unregister_name("temp_process");
/// assert_eq!(reg.whereis_name("temp_process"), None);
/// ```
pub struct Register {
    /// Maps registered name (atom) to process/port ID
    /// Key: atom name (as String for now, will be Eterm later)
    /// Value: process or port ID (as u64 for now, will be proper ID type later)
    table: HashMap<String, u64>,
}

/// Result of a register operation.
///
/// This enum represents the possible outcomes when attempting to register a
/// process or port with a name. It provides detailed error information to
/// help diagnose registration failures.
///
/// # Variants
///
/// - `Success`: The registration succeeded. The name is now associated with
///   the specified ID.
///
/// - `AlreadyRegistered`: The name is already registered to a different
///   process/port ID. Each name can only be associated with one ID.
///
/// - `InvalidName`: The provided name is invalid (e.g., empty string
///   representing undefined). In the full implementation, this would check
///   if the name is a valid atom.
///
/// - `AlreadyHasName`: The process/port ID is already registered with a
///   different name. Each ID can only be associated with one name.
///
/// - `NotAlive`: The process/port is not alive. In the full implementation,
///   this would check if the process/port exists and is running.
///
/// # Examples
///
/// ## Success Case
///
/// ```rust
/// use entities_utilities::{Register, RegisterResult};
///
/// let mut reg = Register::new();
/// let result = reg.register_name("my_process", 123);
/// assert_eq!(result, RegisterResult::Success);
/// ```
///
/// ## Error Cases
///
/// ```rust
/// use entities_utilities::{Register, RegisterResult};
///
/// let mut reg = Register::new();
/// reg.register_name("process1", 100);
///
/// // Try to register same name with different ID
/// let result = reg.register_name("process1", 200);
/// assert_eq!(result, RegisterResult::AlreadyRegistered);
///
/// // Try to register same ID with different name
/// let result = reg.register_name("process2", 100);
/// assert_eq!(result, RegisterResult::AlreadyHasName);
///
/// // Try to register with invalid name
/// let result = reg.register_name("", 300);
/// assert_eq!(result, RegisterResult::InvalidName);
/// ```
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
    /// Create a new empty register table.
    //
    /// This function creates a new `Register` instance with no registered
    /// names. The table is ready to accept registrations immediately.
    //
    /// # Purpose
    //
    /// This is the primary constructor for the `Register` type. It initializes
    /// an empty hash map that will store the name-to-ID mappings as processes
    /// and ports are registered.
    //
    /// # Returns
    //
    /// A new `Register` instance with an empty registration table.
    //
    /// # Examples
    //
    /// ## Basic Creation
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// assert!(reg.is_empty());
    /// assert_eq!(reg.size(), 0);
    /// ```
    //
    /// ## Using Default
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::default(); // Also creates empty table
    /// assert!(reg.is_empty());
    /// ```
    //
    /// ## Ready for Registration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// // Immediately ready to register processes
    /// reg.register_name("my_process", 123);
    /// ```
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Register a name with a process/port ID.
    //
    /// This function associates an atom name with a process or port ID in the
    /// registration table. The registration enforces the constraint that each
    /// name maps to exactly one ID and each ID maps to at most one name.
    //
    /// # Purpose
    //
    /// Name registration is the core functionality that enables Erlang's
    /// `register/2` BIF. It allows processes to be found by name, which is
    /// essential for supervisor trees, registered processes, and distributed
    /// Erlang applications. The bidirectional constraint prevents name conflicts
    /// and ensures consistent lookup behavior.
    //
    /// # Arguments
    //
    /// * `name` - The atom name to register (as `String` in the entities layer)
    /// * `id` - The process or port ID to register (as `u64` in the entities layer)
    //
    /// # Returns
    //
    /// * `RegisterResult::Success` if registration succeeded
    /// * `RegisterResult::AlreadyRegistered` if name is already registered to a different ID
    /// * `RegisterResult::InvalidName` if name is invalid (e.g., empty string)
    /// * `RegisterResult::AlreadyHasName` if ID is already registered with a different name
    //
    /// # Examples
    //
    /// ## Successful Registration
    //
    /// ```rust
    /// use entities_utilities::{Register, RegisterResult};
    //
    /// let mut reg = Register::new();
    /// let result = reg.register_name("my_process", 123);
    /// assert_eq!(result, RegisterResult::Success);
    /// assert_eq!(reg.whereis_name("my_process"), Some(123));
    /// ```
    //
    /// ## Duplicate Name Registration
    //
    /// ```rust
    /// use entities_utilities::{Register, RegisterResult};
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// // Registering same name with different ID fails
    /// let result = reg.register_name("my_process", 456);
    /// assert_eq!(result, RegisterResult::AlreadyRegistered);
    /// // Original registration remains
    /// assert_eq!(reg.whereis_name("my_process"), Some(123));
    /// ```
    //
    /// ## Idempotent Registration
    //
    /// ```rust
    /// use entities_utilities::{Register, RegisterResult};
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// // Registering same name and ID again succeeds (idempotent)
    /// let result = reg.register_name("my_process", 123);
    /// assert_eq!(result, RegisterResult::Success);
    /// ```
    //
    /// # Note
    //
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

    /// Find the process/port ID for a registered name.
    //
    /// This function performs a lookup in the registration table to find the
    /// process or port ID associated with the given atom name. This is the
    /// core functionality that enables Erlang's `whereis/1` BIF.
    //
    /// # Purpose
    //
    /// Name lookup is essential for finding processes by name, which is a
    /// fundamental operation in Erlang. Processes often need to find other
    /// processes (such as supervisors, registered servers, or named processes)
    /// by their registered names rather than by PID.
    //
    /// # Arguments
    //
    /// * `name` - The atom name to look up
    //
    /// # Returns
    //
    /// * `Some(id)` if the name is registered
    /// * `None` if the name is not registered
    //
    /// # Examples
    //
    /// ## Successful Lookup
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// let id = reg.whereis_name("my_process");
    /// assert_eq!(id, Some(123));
    /// ```
    //
    /// ## Name Not Found
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// let id = reg.whereis_name("nonexistent");
    /// assert_eq!(id, None);
    /// ```
    //
    /// ## Multiple Lookups
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    //
    /// assert_eq!(reg.whereis_name("process1"), Some(100));
    /// assert_eq!(reg.whereis_name("process2"), Some(200));
    /// assert_eq!(reg.whereis_name("process3"), None);
    /// ```
    //
    pub fn whereis_name(&self, name: &str) -> Option<u64> {
        self.table.get(name).copied()
    }

    /// Check if a name is registered in the table.
    //
    /// This function provides a boolean check for whether a name is registered,
    /// which is more efficient than calling `whereis_name` and checking for `None`
    /// when you only need to know if a name exists, not its associated ID.
    //
    /// # Purpose
    //
    /// This function is useful for conditional logic where you need to check
    /// if a name is registered before performing operations on it. It's more
    /// efficient than `whereis_name` when you don't need the actual ID value.
    //
    /// # Arguments
    //
    /// * `name` - The atom name to check
    //
    /// # Returns
    //
    /// * `true` if the name is registered
    /// * `false` if the name is not registered
    //
    /// # Examples
    //
    /// ## Check Registered Name
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// assert!(reg.is_registered("my_process"));
    /// assert!(!reg.is_registered("nonexistent"));
    /// ```
    //
    /// ## Conditional Logic
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("server", 100);
    //
    /// if reg.is_registered("server") {
    ///     // Server is registered, proceed with operations
    ///     let id = reg.whereis_name("server").unwrap();
    ///     println!("Server ID: {}", id);
    /// } else {
    ///     println!("Server not registered yet");
    /// }
    /// ```
    //
    /// ## After Unregistration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("temp_process", 123);
    /// assert!(reg.is_registered("temp_process"));
    //
    /// reg.unregister_name("temp_process");
    /// assert!(!reg.is_registered("temp_process"));
    /// ```
    pub fn is_registered(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    /// Find the registered name for a process/port ID (reverse lookup).
    //
    /// This function performs a reverse lookup in the registration table to find
    /// the atom name associated with a given process or port ID. This enables
    /// queries about whether a process is registered and what name it uses.
    //
    /// # Purpose
    //
    /// Reverse lookup is useful for debugging, monitoring, and process management.
    /// It allows you to determine if a process/port is registered and what name
    /// it's using, which is helpful for understanding the system state and for
    /// implementing process introspection features.
    //
    /// # Arguments
    //
    /// * `id` - The process/port ID to look up
    //
    /// # Returns
    //
    /// * `Some(name)` if the ID is registered
    /// * `None` if the ID is not registered
    //
    /// # Examples
    //
    /// ## Successful Reverse Lookup
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// let name = reg.get_name_for_id(123);
    /// assert_eq!(name, Some("my_process".to_string()));
    /// ```
    //
    /// ## ID Not Found
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// let name = reg.get_name_for_id(999);
    /// assert_eq!(name, None);
    /// ```
    //
    /// ## Bidirectional Lookup
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("server", 100);
    //
    /// // Forward lookup: name -> ID
    /// let id = reg.whereis_name("server");
    /// assert_eq!(id, Some(100));
    //
    /// // Reverse lookup: ID -> name
    /// let name = reg.get_name_for_id(100);
    /// assert_eq!(name, Some("server".to_string()));
    /// ```
    pub fn get_name_for_id(&self, id: u64) -> Option<String> {
        for (name, &registered_id) in &self.table {
            if registered_id == id {
                return Some(name.clone());
            }
        }
        None
    }

    /// Unregister a name from the registration table.
    //
    /// This function removes the association between a name and its process/port ID
    /// from the registration table. After unregistration, the name can be registered
    /// again with a different ID, and the ID can be registered with a different name.
    //
    /// # Purpose
    //
    /// Unregistration is essential for process lifecycle management. When a process
    /// terminates or no longer needs to be registered, its name should be unregistered
    /// to free up the name for other processes and to maintain accurate registration
    /// state. This corresponds to Erlang's `unregister/1` functionality.
    //
    /// # Arguments
    //
    /// * `name` - The atom name to unregister
    //
    /// # Returns
    //
    /// * `true` if the name was registered and has been removed
    /// * `false` if the name was not registered
    //
    /// # Examples
    //
    /// ## Successful Unregistration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    /// assert!(reg.is_registered("my_process"));
    //
    /// let removed = reg.unregister_name("my_process");
    /// assert!(removed);
    /// assert!(!reg.is_registered("my_process"));
    /// assert_eq!(reg.whereis_name("my_process"), None);
    /// ```
    //
    /// ## Unregister Non-Existent Name
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// let removed = reg.unregister_name("nonexistent");
    /// assert!(!removed); // Name was not registered
    /// ```
    //
    /// ## Re-registration After Unregistration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process", 100);
    /// reg.unregister_name("process");
    //
    /// // Name can now be registered with a different ID
    /// reg.register_name("process", 200);
    /// assert_eq!(reg.whereis_name("process"), Some(200));
    /// ```
    //
    pub fn unregister_name(&mut self, name: &str) -> bool {
        self.table.remove(name).is_some()
    }

    /// Unregister a process/port ID from the registration table.
    //
    /// This function removes the association between a process/port ID and its
    /// registered name from the registration table. It performs a reverse lookup
    /// to find the name associated with the ID, then removes that registration.
    //
    /// # Purpose
    //
    /// Unregistering by ID is useful when you have the process/port ID and need
    /// to remove its registration, but don't know or don't have the registered name.
    /// This is common in process cleanup scenarios where you have the PID but need
    /// to clean up the registration.
    //
    /// # Arguments
    //
    /// * `id` - The process/port ID to unregister
    //
    /// # Returns
    //
    /// * `Some(name)` if the ID was registered and has been removed (returns the name)
    /// * `None` if the ID was not registered
    //
    /// # Examples
    //
    /// ## Successful Unregistration by ID
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("my_process", 123);
    //
    /// let name = reg.unregister_id(123);
    /// assert_eq!(name, Some("my_process".to_string()));
    /// assert!(!reg.is_registered("my_process"));
    /// ```
    //
    /// ## ID Not Found
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// let name = reg.unregister_id(999);
    /// assert_eq!(name, None); // ID was not registered
    /// ```
    //
    /// ## Process Cleanup Pattern
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("worker", 100);
    //
    /// // When process terminates, unregister by ID
    /// if let Some(name) = reg.unregister_id(100) {
    ///     println!("Unregistered process: {}", name);
    /// }
    /// ```
    pub fn unregister_id(&mut self, id: u64) -> Option<String> {
        let name_to_remove = self.get_name_for_id(id);
        if let Some(ref name) = name_to_remove {
            self.table.remove(name);
        }
        name_to_remove
    }

    /// Get the number of registered names in the table.
    //
    /// This function returns the current count of registered name-to-ID mappings
    /// in the registration table. This is useful for monitoring, statistics, and
    /// understanding the state of the registration system.
    //
    /// # Purpose
    //
    /// Knowing the size of the registration table is useful for monitoring system
    /// health, debugging registration issues, and implementing administrative
    /// features that need to report on the number of registered processes.
    //
    /// # Returns
    //
    /// The number of registered names in the table.
    //
    /// # Examples
    //
    /// ## Empty Table
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// assert_eq!(reg.size(), 0);
    /// ```
    //
    /// ## After Registrations
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// reg.register_name("process3", 300);
    //
    /// assert_eq!(reg.size(), 3);
    /// ```
    //
    /// ## After Unregistration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// assert_eq!(reg.size(), 2);
    //
    /// reg.unregister_name("process1");
    /// assert_eq!(reg.size(), 1);
    /// ```
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Check if the register table is empty.
    //
    /// This function provides a quick way to determine if there are any registered
    /// names in the table. It's more efficient than checking `size() == 0` and is
    /// useful for conditional logic.
    //
    /// # Purpose
    //
    /// This function is useful for initialization checks, cleanup verification,
    /// and conditional logic that depends on whether any processes are registered.
    //
    /// # Returns
    //
    /// * `true` if there are no registered names
    /// * `false` if there is at least one registered name
    //
    /// # Examples
    //
    /// ## Empty Table
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// assert!(reg.is_empty());
    /// ```
    //
    /// ## After Registration
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// assert!(reg.is_empty());
    //
    /// reg.register_name("my_process", 123);
    /// assert!(!reg.is_empty());
    /// ```
    //
    /// ## After Clear
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// assert!(!reg.is_empty());
    //
    /// reg.clear();
    /// assert!(reg.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Clear all registrations from the table.
    //
    /// This function removes all name-to-ID mappings from the registration table,
    /// effectively resetting it to an empty state. After clearing, all names
    /// become available for registration again.
    //
    /// # Purpose
    //
    /// Clearing the registration table is useful for system shutdown, testing,
    /// and reset scenarios where you need to start with a clean slate. It's
    /// particularly useful in test environments where you want to ensure a
    /// known initial state.
    //
    /// # Examples
    //
    /// ## Clear All Registrations
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// assert_eq!(reg.size(), 2);
    //
    /// reg.clear();
    /// assert!(reg.is_empty());
    /// assert_eq!(reg.size(), 0);
    /// ```
    //
    /// ## Reset for Testing
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// // ... perform tests with registrations ...
    /// reg.clear(); // Reset for next test
    /// assert!(reg.is_empty());
    /// ```
    //
    /// ## System Shutdown
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("server", 100);
    /// reg.register_name("worker", 200);
    //
    /// // On system shutdown, clear all registrations
    /// reg.clear();
    /// assert!(reg.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// Get a list of all registered names.
    //
    /// This function returns a vector containing all currently registered names
    /// in the registration table. The order of names in the vector is not
    /// guaranteed to be consistent across calls.
    //
    /// # Purpose
    //
    /// Getting all registered names is useful for administrative operations,
    /// debugging, monitoring, and implementing features that need to enumerate
    /// all registered processes (such as process listing or health checks).
    //
    /// # Returns
    //
    /// A `Vec<String>` containing all registered names. The vector will be empty
    /// if no names are registered.
    //
    /// # Examples
    //
    /// ## Get All Names
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// reg.register_name("process3", 300);
    //
    /// let names = reg.get_all_names();
    /// assert_eq!(names.len(), 3);
    /// assert!(names.contains(&"process1".to_string()));
    /// assert!(names.contains(&"process2".to_string()));
    /// assert!(names.contains(&"process3".to_string()));
    /// ```
    //
    /// ## Empty Table
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// let names = reg.get_all_names();
    /// assert!(names.is_empty());
    /// ```
    //
    /// ## Iterate Over All Names
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("server", 100);
    /// reg.register_name("worker", 200);
    //
    /// for name in reg.get_all_names() {
    ///     let id = reg.whereis_name(&name).unwrap();
    ///     println!("{} -> {}", name, id);
    /// }
    /// ```
    pub fn get_all_names(&self) -> Vec<String> {
        self.table.keys().cloned().collect()
    }

    /// Get a list of all registered process/port IDs.
    //
    /// This function returns a vector containing all currently registered
    /// process/port IDs in the registration table. The order of IDs in the
    /// vector is not guaranteed to be consistent across calls.
    //
    /// # Purpose
    //
    /// Getting all registered IDs is useful for administrative operations,
    /// debugging, monitoring, and implementing features that need to enumerate
    /// all registered processes (such as process listing, health checks, or
    /// cleanup operations).
    //
    /// # Returns
    //
    /// A `Vec<u64>` containing all registered IDs. The vector will be empty
    /// if no IDs are registered.
    //
    /// # Examples
    //
    /// ## Get All IDs
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("process1", 100);
    /// reg.register_name("process2", 200);
    /// reg.register_name("process3", 300);
    //
    /// let ids = reg.get_all_ids();
    /// assert_eq!(ids.len(), 3);
    /// assert!(ids.contains(&100));
    /// assert!(ids.contains(&200));
    /// assert!(ids.contains(&300));
    /// ```
    //
    /// ## Empty Table
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let reg = Register::new();
    /// let ids = reg.get_all_ids();
    /// assert!(ids.is_empty());
    /// ```
    //
    /// ## Iterate Over All IDs
    //
    /// ```rust
    /// use entities_utilities::Register;
    //
    /// let mut reg = Register::new();
    /// reg.register_name("server", 100);
    /// reg.register_name("worker", 200);
    //
    /// for id in reg.get_all_ids() {
    ///     let name = reg.get_name_for_id(id).unwrap();
    ///     println!("{} -> {}", name, id);
    /// }
    /// ```
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
