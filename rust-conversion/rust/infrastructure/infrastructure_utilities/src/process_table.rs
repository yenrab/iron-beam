//! Process Table Module
//!
//! Provides process table/registry functionality for managing processes.
//! Based on erl_ptab.c - Process/Port table implementation.
//!
//! The process table is a mapping from process identifiers to process
//! structure pointers. When the runtime system needs to operate on a
//! process, it looks up the process structure in the process table using
//! the process identifier.
//!
//! ## Performance Optimization
//!
//! The current implementation uses a HashMap-based design which is sufficient
//! for most use cases. For extreme performance scenarios (similar to the
//! Erlang VM), see `PERFORMANCE_OPTIMIZATION_GUIDE.md` in this directory for
//! a detailed roadmap on optimizing to a lock-free array-based design.

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

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use entities_process::{Process, ProcessId};

/// Process table/registry
///
/// Maps process identifiers to process structures. This is a thread-safe
/// implementation that allows concurrent lookups while maintaining data integrity.
///
/// Based on ErtsPTab in erl_ptab.c
pub struct ProcessTable {
    /// Internal hash map storing processes by ID
    /// Uses Arc for shared ownership and RwLock for thread safety
    table: Arc<RwLock<HashMap<ProcessId, Arc<Process>>>>,
    /// Next ID to use for new processes (atomic counter)
    next_id: AtomicU64,
    /// Queue of free IDs available for reuse
    free_ids: Arc<RwLock<VecDeque<ProcessId>>>,
    /// Maximum number of processes in the table (0 = unlimited)
    max_size: usize,
}

impl ProcessTable {
    /// Create a new empty process table with unlimited capacity
    ///
    /// # Returns
    /// A new ProcessTable instance
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    ///
    /// let table = ProcessTable::new();
    /// assert_eq!(table.size(), 0);
    /// ```
    pub fn new() -> Self {
        Self::with_max_size(0)
    }

    /// Create a new process table with a maximum size limit
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of processes (0 = unlimited)
    ///
    /// # Returns
    /// A new ProcessTable instance with size limit
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    ///
    /// let table = ProcessTable::with_max_size(1000);
    /// assert_eq!(table.max_size(), Some(1000));
    /// ```
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            table: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1), // Start from 1, 0 is reserved
            free_ids: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
        }
    }

    /// Get the maximum size of the table
    ///
    /// # Returns
    /// * `Some(max_size)` - If a limit is set
    /// * `None` - If unlimited
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    ///
    /// let table = ProcessTable::new();
    /// assert_eq!(table.max_size(), None);
    ///
    /// let limited_table = ProcessTable::with_max_size(100);
    /// assert_eq!(limited_table.max_size(), Some(100));
    /// ```
    pub fn max_size(&self) -> Option<usize> {
        if self.max_size == 0 {
            None
        } else {
            Some(self.max_size)
        }
    }

    /// Look up a process by ID
    ///
    /// # Arguments
    /// * `id` - Process ID to look up
    ///
    /// # Returns
    /// * `Some(Arc<Process>)` - If process is found
    /// * `None` - If process is not found
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    ///
    /// let table = ProcessTable::new();
    /// let process = Arc::new(Process::new(123));
    /// table.insert(123, Arc::clone(&process));
    ///
    /// let found = table.lookup(123);
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().get_id(), 123);
    /// ```
    pub fn lookup(&self, id: ProcessId) -> Option<Arc<Process>> {
        let table = self.table.read().unwrap();
        table.get(&id).map(|p| Arc::clone(p))
    }

    /// Insert a process into the table
    ///
    /// # Arguments
    /// * `id` - Process ID
    /// * `process` - Process to insert (wrapped in Arc for shared ownership)
    ///
    /// # Returns
    /// * `Some(Arc<Process>)` - Previous process if one existed with this ID
    /// * `None` - If no previous process existed
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// let process = Arc::new(Process::new(123));
    /// let previous = table.insert(123, Arc::clone(&process));
    /// assert!(previous.is_none());
    ///
    /// let process2 = Arc::new(Process::new(456));
    /// let previous = table.insert(123, process2);
    /// assert!(previous.is_some());
    /// ```
    pub fn insert(&self, id: ProcessId, process: Arc<Process>) -> Option<Arc<Process>> {
        let mut table = self.table.write().unwrap();
        table.insert(id, process).map(|p| Arc::clone(&p))
    }

    /// Remove a process from the table
    ///
    /// When a process is removed, its ID is added to the free ID pool
    /// for reuse by `new_element()`.
    ///
    /// # Arguments
    /// * `id` - Process ID to remove
    ///
    /// # Returns
    /// * `Some(Arc<Process>)` - Removed process if found
    /// * `None` - If process was not found
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// let process = Arc::new(Process::new(123));
    /// table.insert(123, Arc::clone(&process));
    ///
    /// let removed = table.remove(123);
    /// assert!(removed.is_some());
    /// assert_eq!(table.lookup(123), None);
    /// ```
    pub fn remove(&self, id: ProcessId) -> Option<Arc<Process>> {
        let mut table = self.table.write().unwrap();
        let removed = table.remove(&id);
        
        // Add ID to free pool for reuse (if it was actually removed)
        if removed.is_some() && id > 0 {
            let mut free_ids = self.free_ids.write().unwrap();
            free_ids.push_back(id);
        }
        
        removed
    }

    /// Get the number of processes in the table
    ///
    /// # Returns
    /// Number of processes currently in the table
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// assert_eq!(table.size(), 0);
    ///
    /// table.insert(123, Arc::new(Process::new(123)));
    /// table.insert(456, Arc::new(Process::new(456)));
    /// assert_eq!(table.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        let table = self.table.read().unwrap();
        table.len()
    }

    /// Check if the table is empty
    ///
    /// # Returns
    /// `true` if the table is empty, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    ///
    /// let table = ProcessTable::new();
    /// assert!(table.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        let table = self.table.read().unwrap();
        table.is_empty()
    }

    /// Get all process IDs in the table
    ///
    /// # Returns
    /// Vector of all process IDs currently in the table
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// table.insert(123, Arc::new(Process::new(123)));
    /// table.insert(456, Arc::new(Process::new(456)));
    ///
    /// let ids = table.get_all_ids();
    /// assert_eq!(ids.len(), 2);
    /// assert!(ids.contains(&123));
    /// assert!(ids.contains(&456));
    /// ```
    pub fn get_all_ids(&self) -> Vec<ProcessId> {
        let table = self.table.read().unwrap();
        table.keys().copied().collect()
    }

    /// Clear all processes from the table
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// table.insert(123, Arc::new(Process::new(123)));
    /// table.insert(456, Arc::new(Process::new(456)));
    /// assert_eq!(table.size(), 2);
    ///
    /// table.clear();
    /// assert_eq!(table.size(), 0);
    /// ```
    pub fn clear(&self) {
        let mut table = self.table.write().unwrap();
        table.clear();
        let mut free_ids = self.free_ids.write().unwrap();
        free_ids.clear();
    }

    /// Create a new process element with automatically generated ID
    ///
    /// This is equivalent to `erts_ptab_new_element()` in the C code.
    /// It automatically generates a unique process ID and inserts the process
    /// into the table. IDs are reused from a free pool when available.
    ///
    /// # Arguments
    /// * `init_fn` - Function to initialize the process with the generated ID
    ///
    /// # Returns
    /// * `Ok((ProcessId, Arc<Process>))` - Successfully created process with its ID
    /// * `Err(ProcessTableError::TableFull)` - Table is at maximum capacity
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::process_table::ProcessTable;
    /// use entities_process::Process;
    /// use std::sync::Arc;
    ///
    /// let table = ProcessTable::new();
    /// let (id, process) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
    /// assert!(id > 0);
    /// assert_eq!(process.get_id(), id);
    /// assert!(table.lookup(id).is_some());
    /// ```
    pub fn new_element<F>(&self, init_fn: F) -> Result<(ProcessId, Arc<Process>), ProcessTableError>
    where
        F: Fn(ProcessId) -> Arc<Process>,
    {
        // Check capacity
        if self.max_size > 0 {
            let table = self.table.read().unwrap();
            if table.len() >= self.max_size {
                return Err(ProcessTableError::TableFull);
            }
        }

        // Try to get a free ID first (for reuse)
        loop {
            let id = {
                let mut free_ids = self.free_ids.write().unwrap();
                if let Some(free_id) = free_ids.pop_front() {
                    free_id
                } else {
                    // Generate new ID
                    let new_id = self.next_id.fetch_add(1, Ordering::Relaxed);
                    // Skip 0 as it's reserved
                    if new_id == 0 {
                        self.next_id.fetch_add(1, Ordering::Relaxed);
                        1
                    } else {
                        new_id
                    }
                }
            };

            // Create process with the generated ID
            let process = init_fn(id);

            // Insert into table
            let mut table = self.table.write().unwrap();
            
            // Check capacity again (another thread might have filled it)
            if self.max_size > 0 && table.len() >= self.max_size {
                // Put ID back in free pool
                drop(table);
                let mut free_ids = self.free_ids.write().unwrap();
                free_ids.push_front(id);
                return Err(ProcessTableError::TableFull);
            }

            // Verify ID doesn't already exist (shouldn't happen, but be safe)
            if table.contains_key(&id) {
                // Put ID back in free pool and try next ID
                drop(table);
                let mut free_ids = self.free_ids.write().unwrap();
                free_ids.push_front(id);
                continue; // Try again with next ID
            } else {
                table.insert(id, Arc::clone(&process));
                return Ok((id, process));
            }
        }
    }
}

/// Errors that can occur when operating on the process table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessTableError {
    /// Table is at maximum capacity
    TableFull,
    /// Invalid process ID
    InvalidId,
}

impl std::fmt::Display for ProcessTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessTableError::TableFull => write!(f, "Process table is full"),
            ProcessTableError::InvalidId => write!(f, "Invalid process ID"),
        }
    }
}

impl std::error::Error for ProcessTableError {}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Global process table instance
///
/// This provides a singleton process table that can be accessed from
/// anywhere in the system. In a full implementation, this would be
/// initialized during system startup.
static GLOBAL_PROCESS_TABLE: std::sync::OnceLock<ProcessTable> = std::sync::OnceLock::new();

/// Get the global process table instance
///
/// # Returns
/// Reference to the global process table
///
/// # Examples
/// ```
/// use infrastructure_utilities::process_table::get_global_process_table;
/// use entities_process_port::Process;
/// use std::sync::Arc;
///
/// let table = get_global_process_table();
/// let process = Arc::new(Process::new(123));
/// table.insert(123, process);
/// ```
pub fn get_global_process_table() -> &'static ProcessTable {
    GLOBAL_PROCESS_TABLE.get_or_init(ProcessTable::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities_process::Process;
    use std::sync::Arc;

    #[test]
    fn test_process_table_new() {
        let table = ProcessTable::new();
        assert_eq!(table.size(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_process_table_insert() {
        let table = ProcessTable::new();
        let process = Arc::new(Process::new(123));
        
        let previous = table.insert(123, Arc::clone(&process));
        assert!(previous.is_none());
        assert_eq!(table.size(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_process_table_lookup() {
        let table = ProcessTable::new();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));

        let found = table.lookup(123);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_id(), 123);

        let not_found = table.lookup(456);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_process_table_remove() {
        let table = ProcessTable::new();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));

        let removed = table.remove(123);
        assert!(removed.is_some());
        assert_eq!(table.size(), 0);
        assert!(table.is_empty());

        let not_found = table.lookup(123);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_process_table_replace() {
        let table = ProcessTable::new();
        let process1 = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process1));

        let process2 = Arc::new(Process::new(456));
        let previous = table.insert(123, process2);
        assert!(previous.is_some());
        assert_eq!(previous.unwrap().get_id(), 123);
        assert_eq!(table.size(), 1);
    }

    #[test]
    fn test_process_table_get_all_ids() {
        let table = ProcessTable::new();
        table.insert(123, Arc::new(Process::new(123)));
        table.insert(456, Arc::new(Process::new(456)));
        table.insert(789, Arc::new(Process::new(789)));

        let ids = table.get_all_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&123));
        assert!(ids.contains(&456));
        assert!(ids.contains(&789));
    }

    #[test]
    fn test_process_table_clear() {
        let table = ProcessTable::new();
        table.insert(123, Arc::new(Process::new(123)));
        table.insert(456, Arc::new(Process::new(456)));
        assert_eq!(table.size(), 2);

        table.clear();
        assert_eq!(table.size(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_global_process_table() {
        let table1 = get_global_process_table();
        let table2 = get_global_process_table();
        
        // Should be the same instance
        assert_eq!(table1.size(), table2.size());
        
        let process = Arc::new(Process::new(999));
        table1.insert(999, process);
        
        // Should be visible from both references
        assert_eq!(table1.size(), table2.size());
        assert!(table2.lookup(999).is_some());
    }

    #[test]
    fn test_process_table_thread_safety() {
        use std::thread;
        
        let table = Arc::new(ProcessTable::new());
        let mut handles = vec![];

        // Spawn multiple threads inserting processes
        for i in 0..10 {
            let table_clone = Arc::clone(&table);
            let handle = thread::spawn(move || {
                let process = Arc::new(Process::new(i));
                table_clone.insert(i, process);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all processes were inserted
        assert_eq!(table.size(), 10);
        for i in 0..10 {
            assert!(table.lookup(i).is_some());
        }
    }

    #[test]
    fn test_new_element_auto_id_generation() {
        let table = ProcessTable::new();
        
        let (id1, process1) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        assert!(id1 > 0);
        assert_eq!(process1.get_id(), id1);
        assert!(table.lookup(id1).is_some());

        let (id2, process2) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        assert!(id2 > id1); // Should be sequential
        assert_eq!(table.size(), 2);
    }

    #[test]
    fn test_new_element_id_reuse() {
        let table = ProcessTable::new();
        
        // Create and remove a process
        let (id1, _) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        table.remove(id1).unwrap();
        
        // Create another - should reuse the ID
        let (id2, _) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        assert_eq!(id2, id1); // ID should be reused
    }

    #[test]
    fn test_new_element_max_size() {
        let table = ProcessTable::with_max_size(2);
        
        let (id1, _) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        let (id2, _) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        
        // Third should fail
        let result = table.new_element(|id| Arc::new(Process::new(id)));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProcessTableError::TableFull);
        
        // After removing one, should be able to add again
        table.remove(id1).unwrap();
        let (id3, _) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
        assert_eq!(table.size(), 2);
    }

    #[test]
    fn test_max_size() {
        let table1 = ProcessTable::new();
        assert_eq!(table1.max_size(), None);
        
        let table2 = ProcessTable::with_max_size(100);
        assert_eq!(table2.max_size(), Some(100));
    }
}

