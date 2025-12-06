//! NIF Environment
//!
//! Provides a Rust-native NIF environment that wraps access to the Process heap.
//! Since all NIFs are in Rust, this provides safe access to the Process structure
//! and its heap for term allocation.

use std::sync::Arc;
use entities_process::Process;
use entities_process::ProcessId;
use infrastructure_utilities::process_table::get_global_process_table;

/// NIF Environment
///
/// Provides access to the process heap and runtime resources for NIF operations.
/// This is a Rust-native implementation that wraps the Process structure.
///
/// The environment can be created from a ProcessId (which looks up the process
/// in the global process table) or from an Arc<Process> for direct access.
pub struct NifEnv {
    /// Process reference - provides access to the heap
    process: Arc<Process>,
}

impl NifEnv {
    /// Create a new NIF environment from a Process ID
    ///
    /// Looks up the process in the global process table.
    ///
    /// # Arguments
    /// * `process_id` - Process ID to look up
    ///
    /// # Returns
    /// * `Some(NifEnv)` - If process is found
    /// * `None` - If process is not found
    pub fn from_process_id(process_id: ProcessId) -> Option<Self> {
        let table = get_global_process_table();
        table.lookup(process_id).map(|process| Self {
            process,
        })
    }

    /// Create a new NIF environment from a Process reference
    ///
    /// # Arguments
    /// * `process` - Process reference
    ///
    /// # Returns
    /// * `NifEnv` - New NIF environment
    pub fn from_process(process: Arc<Process>) -> Self {
        Self { process }
    }

    /// Get the process ID
    pub fn process_id(&self) -> ProcessId {
        self.process.id()
    }

    /// Get a reference to the process
    pub fn process(&self) -> &Arc<Process> {
        &self.process
    }

    /// Allocate space on the process heap
    ///
    /// Allocates `words` number of words on the process heap and returns
    /// the index where the allocation starts.
    ///
    /// # Arguments
    /// * `words` - Number of words to allocate
    ///
    /// # Returns
    /// * `Some(usize)` - Heap index where allocation starts
    /// * `None` - If allocation fails (heap full, needs GC, etc.)
    ///
    /// # Note
    ///
    /// This method uses the Process's interior mutability (Mutex) to safely
    /// allocate heap space in a thread-safe manner.
    pub fn allocate_heap(&self, words: usize) -> Option<usize> {
        self.process.allocate_heap_words(words)
    }

    /// Get available heap space in words
    ///
    /// Returns the number of words available on the heap for allocation.
    ///
    /// # Returns
    /// * `usize` - Available heap space in words
    pub fn available_heap_space(&self) -> usize {
        let heap_size = self.process.heap_sz();
        let heap_top = self.process.heap_top_index();
        
        // Available space is from heap_top to heap_size
        heap_size.saturating_sub(heap_top)
    }

    /// Get heap top index
    ///
    /// Returns the current heap top index where new allocations would start.
    pub fn heap_top_index(&self) -> usize {
        self.process.heap_top_index()
    }

    /// Get heap size in words
    pub fn heap_size(&self) -> usize {
        self.process.heap_sz()
    }
}

/// Helper function to get Process from NifEnv
///
/// This is a convenience function for accessing the Process from a NifEnv.
pub fn get_process(env: &NifEnv) -> &Arc<Process> {
    env.process()
}

/// Helper function to get Process ID from NifEnv
pub fn get_process_id(env: &NifEnv) -> ProcessId {
    env.process_id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_nif_env_from_process() {
        let process = Arc::new(Process::new(123));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        assert_eq!(env.process_id(), 123);
        assert_eq!(env.heap_size(), 233); // Default heap size
        assert_eq!(env.heap_top_index(), 0);
    }

    #[test]
    fn test_nif_env_from_process_id() {
        // Create a process and add it to the table
        let table = get_global_process_table();
        let process = Arc::new(Process::new(456));
        table.insert(456, Arc::clone(&process));
        
        // Create NIF env from process ID
        let env = NifEnv::from_process_id(456);
        assert!(env.is_some());
        let env = env.unwrap();
        assert_eq!(env.process_id(), 456);
    }

    #[test]
    fn test_nif_env_from_invalid_process_id() {
        let env = NifEnv::from_process_id(99999);
        assert!(env.is_none());
    }

    #[test]
    fn test_available_heap_space() {
        let process = Arc::new(Process::new(789));
        let env = NifEnv::from_process(process);
        
        // Initially, all heap space is available
        assert_eq!(env.available_heap_space(), 233);
    }

    #[test]
    fn test_available_heap_space_after_allocation() {
        let process = Arc::new(Process::new(1000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate some heap space
        let allocated = env.allocate_heap(50);
        assert!(allocated.is_some());
        
        // Available space should decrease
        assert_eq!(env.available_heap_space(), 233 - 50);
    }

    #[test]
    fn test_available_heap_space_when_full() {
        let process = Arc::new(Process::new(2000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate all available space
        let allocated = env.allocate_heap(233);
        assert!(allocated.is_some());
        
        // Available space should be 0
        assert_eq!(env.available_heap_space(), 0);
    }

    #[test]
    fn test_allocate_heap_success() {
        let process = Arc::new(Process::new(3000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate heap space
        let result = env.allocate_heap(10);
        assert!(result.is_some());
        let heap_index = result.unwrap();
        
        // Should return valid heap index
        assert!(heap_index < 233);
        
        // Heap top should have advanced
        assert_eq!(env.heap_top_index(), 10);
    }

    #[test]
    fn test_allocate_heap_multiple_allocations() {
        let process = Arc::new(Process::new(4000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // First allocation
        let result1 = env.allocate_heap(20);
        assert!(result1.is_some());
        let index1 = result1.unwrap();
        
        // Second allocation
        let result2 = env.allocate_heap(30);
        assert!(result2.is_some());
        let index2 = result2.unwrap();
        
        // Second allocation should be after first
        assert_eq!(index2, index1 + 20);
        assert_eq!(env.heap_top_index(), 50);
    }

    #[test]
    fn test_allocate_heap_failure_when_full() {
        let process = Arc::new(Process::new(5000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate all available space
        let result = env.allocate_heap(233);
        assert!(result.is_some());
        
        // Try to allocate more - should fail
        let result2 = env.allocate_heap(1);
        assert!(result2.is_none());
    }

    #[test]
    fn test_allocate_heap_zero_words() {
        let process = Arc::new(Process::new(6000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate zero words
        let result = env.allocate_heap(0);
        assert!(result.is_some());
        let index = result.unwrap();
        
        // Should return current heap top
        assert_eq!(index, env.heap_top_index());
    }

    #[test]
    fn test_heap_top_index_after_allocation() {
        let process = Arc::new(Process::new(7000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Initially heap top is 0
        assert_eq!(env.heap_top_index(), 0);
        
        // Allocate some space
        env.allocate_heap(25);
        assert_eq!(env.heap_top_index(), 25);
        
        // Allocate more
        env.allocate_heap(15);
        assert_eq!(env.heap_top_index(), 40);
    }

    #[test]
    fn test_heap_size() {
        let process = Arc::new(Process::new(8000));
        let env = NifEnv::from_process(process);
        
        // Default heap size
        assert_eq!(env.heap_size(), 233);
    }

    #[test]
    fn test_get_process() {
        let process = Arc::new(Process::new(9000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Get process via helper function
        let retrieved_process = get_process(&env);
        
        // Should be the same process
        assert_eq!(retrieved_process.id(), 9000);
        assert_eq!(Arc::as_ptr(retrieved_process), Arc::as_ptr(&process));
    }

    #[test]
    fn test_get_process_id() {
        let process = Arc::new(Process::new(10000));
        let env = NifEnv::from_process(process);
        
        // Get process ID via helper function
        let process_id = get_process_id(&env);
        
        // Should match
        assert_eq!(process_id, 10000);
    }

    #[test]
    fn test_process_method() {
        let process = Arc::new(Process::new(11000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Get process reference
        let retrieved_process = env.process();
        
        // Should be the same process
        assert_eq!(retrieved_process.id(), 11000);
        assert_eq!(Arc::as_ptr(retrieved_process), Arc::as_ptr(&process));
    }

    #[test]
    fn test_from_process_id_with_existing_process() {
        let table = get_global_process_table();
        let process = Arc::new(Process::new(12000));
        table.insert(12000, Arc::clone(&process));
        
        // Create NIF env from process ID
        let env = NifEnv::from_process_id(12000);
        assert!(env.is_some());
        let env = env.unwrap();
        
        // Verify it's the same process
        assert_eq!(env.process_id(), 12000);
        assert_eq!(env.heap_size(), 233);
    }

    #[test]
    fn test_available_heap_space_saturating_sub() {
        let process = Arc::new(Process::new(13000));
        let env = NifEnv::from_process(Arc::clone(&process));
        
        // Allocate more than available (should fail, but test the calculation)
        // This tests the saturating_sub behavior
        let initial_space = env.available_heap_space();
        assert_eq!(initial_space, 233);
        
        // After allocating all space
        env.allocate_heap(233);
        assert_eq!(env.available_heap_space(), 0);
    }
}

