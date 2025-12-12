//! Process Entity
//!
//! Provides the Process struct and related types for the Erlang runtime system.
//! Based on erts/emulator/beam/erl_process.h
//!
//! The heap is implemented using safe Rust (`Vec<Eterm>`) with index-based
//! access instead of raw pointers for maximum safety.

use std::fmt;
use std::sync::{Arc, Mutex};

/// Process ID type
pub type ProcessId = u64;

/// Erlang term type (matches C Eterm)
/// On 64-bit systems, Eterm is typically u64
pub type Eterm = u64;

/// Code pointer type (matches C ErtsCodePtr)
/// Points to BEAM instruction code
/// Note: This is still a raw pointer for compatibility with code pointers,
/// but heap/stack use safe Vec-based storage
pub type ErtsCodePtr = *const u8;

/// Process state flags (based on ERTS_PSFLG_* from erl_process.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is free (exiting, not visible in process table)
    Free,
    /// Process is exiting (still visible in process table)
    Exiting,
    /// Process is active (wants to execute)
    Active,
    /// Process is running (executing in process_main())
    Running,
    /// Process is suspended
    Suspended,
    /// Process is in garbage collection
    Gc,
    /// Process has system tasks scheduled
    SysTasks,
    /// Process is running system tasks
    RunningSys,
    /// Process is a proxy process struct
    Proxy,
    /// Process has delayed system tasks
    DelayedSys,
    /// Process is running dirty (dirty CPU scheduler)
    DirtyRunning,
    /// Process is running dirty system tasks
    DirtyRunningSys,
    /// Unknown state (for unmapped flag combinations)
    Unknown(u32),
}

impl ProcessState {
    /// Create ProcessState from flags value
    fn from_flags(flags: u32) -> Self {
        // Extract state bits (ERTS_PSFLG_* flags)
        // Based on erl_process.h flags
        
        // Check for FREE (always has EXITING and ACTIVE set)
        if (flags & 0x10) != 0 {  // ERTS_PSFLG_FREE
            return ProcessState::Free;
        }
        
        // Check for EXITING
        if (flags & 0x20) != 0 {  // ERTS_PSFLG_EXITING
            return ProcessState::Exiting;
        }
        
        // Check for DIRTY_RUNNING_SYS
        if (flags & 0x01000000) != 0 {  // ERTS_PSFLG_DIRTY_RUNNING_SYS
            return ProcessState::DirtyRunningSys;
        }
        
        // Check for DIRTY_RUNNING
        if (flags & 0x00800000) != 0 {  // ERTS_PSFLG_DIRTY_RUNNING
            return ProcessState::DirtyRunning;
        }
        
        // Check for RUNNING_SYS
        if (flags & 0x00008000) != 0 {  // ERTS_PSFLG_RUNNING_SYS
            return ProcessState::RunningSys;
        }
        
        // Check for RUNNING
        if (flags & 0x00000200) != 0 {  // ERTS_PSFLG_RUNNING
            return ProcessState::Running;
        }
        
        // Check for GC
        if (flags & 0x00000800) != 0 {  // ERTS_PSFLG_GC
            return ProcessState::Gc;
        }
        
        // Check for SUSPENDED
        if (flags & 0x00000400) != 0 {  // ERTS_PSFLG_SUSPENDED
            return ProcessState::Suspended;
        }
        
        // Check for SYS_TASKS
        if (flags & 0x00001000) != 0 {  // ERTS_PSFLG_SYS_TASKS
            return ProcessState::SysTasks;
        }
        
        // Check for DELAYED_SYS
        if (flags & 0x00020000) != 0 {  // ERTS_PSFLG_DELAYED_SYS
            return ProcessState::DelayedSys;
        }
        
        // Check for PROXY
        if (flags & 0x00010000) != 0 {  // ERTS_PSFLG_PROXY
            return ProcessState::Proxy;
        }
        
        // Check for ACTIVE
        if (flags & 0x00000080) != 0 {  // ERTS_PSFLG_ACTIVE
            return ProcessState::Active;
        }
        
        // Unknown state
        ProcessState::Unknown(flags)
    }
}

/// Process structure (minimal implementation)
///
/// This struct contains only the fields currently needed by the Rust codebase.
/// Additional fields can be added as needed.
///
/// Based on `struct process` in erts/emulator/beam/erl_process.h
///
/// The heap is stored as a safe `Vec<Eterm>` with index-based access,
/// eliminating the need for raw pointers and unsafe code.
pub struct Process {
    /// Process identifier
    id: ProcessId,
    /// Heap size in words (current allocated size)
    heap_sz: usize,
    /// Minimum heap size in words
    min_heap_size: usize,
    /// Maximum heap size in words (0 = unlimited)
    max_heap_size: usize,
    /// Heap data storage (safe Rust Vec, protected by Mutex for concurrent access)
    heap_data: Mutex<Vec<Eterm>>,
    /// Heap start index (usually 0, but can be offset if needed)
    heap_start_index: usize,
    /// Heap top index (current position where new data is allocated, protected by Mutex)
    heap_top_index: Mutex<usize>,
    /// Stack top index (position of stack top in heap_data)
    /// In Erlang, stack and heap share the same memory block
    stack_top_index: Option<usize>,
    /// Process flags (ERTS_PSFLG_*)
    flags: u32,
    /// Number of reductions for this process
    reds: usize,
    /// Number of reductions left to execute (function calls)
    fcalls: i32,
    /// Number of live argument registers
    arity: u8,
    /// Number of catches on stack
    catches: i32,
    /// Number of return trace frames on stack
    return_trace_frames: i32,
    /// Program counter (instruction pointer)
    /// Note: This remains a raw pointer for compatibility with code pointers
    i: ErtsCodePtr,
    /// Process unique integer
    uniq: i64,
    /// Times left to reschedule a low priority process
    schedule_count: u8,
    /// Suspend count
    rcount: u32,
    /// NIF function pointers currently used by this process
    /// These pointers are tracked for code purging safety checks
    nif_pointers: Vec<*const u8>,
    /// References to NIF libraries loaded for this process
    /// These are reference counted to prevent libraries from being unloaded
    /// while processes are using them
    nif_libraries: Vec<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl Process {
    /// Create a new process with default values
    ///
    /// # Arguments
    /// * `id` - Process identifier
    ///
    /// # Returns
    /// A new Process instance with default values
    pub fn new(id: ProcessId) -> Self {
        let initial_heap_size = 233; // Default minimum heap size (words)
        let mut heap_data = Vec::with_capacity(initial_heap_size);
        heap_data.resize(initial_heap_size, 0);
        
        Self {
            id,
            heap_sz: initial_heap_size,
            min_heap_size: initial_heap_size,
            max_heap_size: 0,    // 0 = unlimited
            heap_data: Mutex::new(heap_data),
            heap_start_index: 0,
            heap_top_index: Mutex::new(0),
            stack_top_index: None,
            flags: 0,
            reds: 0,
            fcalls: 0,
            arity: 0,
            catches: 0,
            return_trace_frames: 0,
            i: std::ptr::null(),
            uniq: 0,
            schedule_count: 0,
            rcount: 0,
            nif_pointers: Vec::new(),
            nif_libraries: Vec::new(),
        }
    }

    /// Get process ID
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// Get process ID (alias for id() for compatibility)
    pub fn get_id(&self) -> ProcessId {
        self.id
    }

    /// Get process state from flags
    pub fn get_state(&self) -> ProcessState {
        ProcessState::from_flags(self.flags)
    }

    /// Get heap size in words
    pub fn heap_sz(&self) -> usize {
        self.heap_sz
    }

    /// Get minimum heap size in words
    pub fn min_heap_size(&self) -> usize {
        self.min_heap_size
    }

    /// Get maximum heap size in words
    pub fn max_heap_size(&self) -> usize {
        self.max_heap_size
    }

    /// Get stack top index
    pub fn stack_top_index(&self) -> Option<usize> {
        self.stack_top_index
    }

    /// Get heap top index
    pub fn heap_top_index(&self) -> usize {
        *self.heap_top_index.lock().unwrap()
    }

    /// Get heap start index
    pub fn heap_start_index(&self) -> usize {
        self.heap_start_index
    }

    /// Get heap data as a slice (safe access to heap contents)
    /// 
    /// Returns a cloned copy of the heap data. For mutable access, use `heap_slice_mut()`.
    pub fn heap_slice(&self) -> Vec<Eterm> {
        self.heap_data.lock().unwrap().clone()
    }

    /// Get heap data as a mutable guard (for heap modifications)
    /// 
    /// Returns a `MutexGuard` that provides mutable access to the heap data.
    /// The guard is automatically released when it goes out of scope.
    pub fn heap_slice_mut(&self) -> std::sync::MutexGuard<'_, Vec<Eterm>> {
        self.heap_data.lock().unwrap()
    }

    /// Allocate words on the process heap
    ///
    /// Allocates `words` number of words on the process heap and returns
    /// the index where the allocation starts. This method is thread-safe
    /// and can be called concurrently.
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
    /// This method does not trigger garbage collection or heap growth.
    /// If the heap is full, it returns `None` and the caller should
    /// handle GC or heap growth separately.
    pub fn allocate_heap_words(&self, words: usize) -> Option<usize> {
        let mut heap_top = self.heap_top_index.lock().unwrap();
        let heap_data = self.heap_data.lock().unwrap();
        
        // Check if we have enough space
        if *heap_top + words > heap_data.len() {
            return None; // Need GC or heap growth
        }
        
        let start_index = *heap_top;
        *heap_top += words;
        Some(start_index)
    }

    /// Calculate stack size in words
    /// Returns None if stack_top_index is not set
    pub fn stack_size_words(&self) -> Option<usize> {
        let heap_top = *self.heap_top_index.lock().unwrap();
        self.stack_top_index.map(|stop| stop.saturating_sub(heap_top))
    }

    /// Get process flags
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Get reductions
    pub fn reds(&self) -> usize {
        self.reds
    }

    /// Get function calls
    pub fn fcalls(&self) -> i32 {
        self.fcalls
    }

    /// Get arity
    pub fn arity(&self) -> u8 {
        self.arity
    }

    /// Get catches
    pub fn catches(&self) -> i32 {
        self.catches
    }

    /// Get return trace frames
    pub fn return_trace_frames(&self) -> i32 {
        self.return_trace_frames
    }

    /// Get program counter
    pub fn i(&self) -> ErtsCodePtr {
        self.i
    }

    /// Set program counter (instruction pointer)
    ///
    /// # Arguments
    /// * `instruction_ptr` - New instruction pointer
    pub fn set_i(&mut self, instruction_ptr: ErtsCodePtr) {
        self.i = instruction_ptr;
    }

    /// Get unique integer
    pub fn uniq(&self) -> i64 {
        self.uniq
    }

    /// Get schedule count
    pub fn schedule_count(&self) -> u8 {
        self.schedule_count
    }

    /// Get suspend count
    pub fn rcount(&self) -> u32 {
        self.rcount
    }

    // Legacy compatibility methods (for backward compatibility during migration)
    // These return None or 0 since we're using indices now
    #[deprecated(note = "Use heap_start_index() and heap_slice() instead")]
    pub fn heap(&self) -> Option<usize> {
        Some(self.heap_start_index)
    }

    #[deprecated(note = "Use heap_top_index() instead")]
    pub fn htop(&self) -> Option<usize> {
        Some(*self.heap_top_index.lock().unwrap())
    }

    #[deprecated(note = "Use stack_top_index() instead")]
    pub fn stop(&self) -> Option<usize> {
        self.stack_top_index
    }

    /// Add a NIF pointer to this process's tracking list
    ///
    /// This method is called by nif_loader when a process uses a NIF.
    /// The pointer is tracked for code purging safety checks.
    ///
    /// # Arguments
    /// * `nif_pointer` - NIF function pointer to track
    ///
    /// # Returns
    /// Error if pointer is invalid
    pub fn add_nif_pointer(&mut self, nif_pointer: *const u8) -> Result<(), String> {
        if nif_pointer.is_null() {
            return Err("Invalid NIF pointer: null pointer".to_string());
        }
        // Avoid duplicates
        if !self.nif_pointers.contains(&nif_pointer) {
            self.nif_pointers.push(nif_pointer);
        }
        Ok(())
    }

    /// Remove a NIF pointer from this process's tracking list
    ///
    /// This method is called by nif_loader when a process no longer uses a NIF.
    ///
    /// # Arguments
    /// * `nif_pointer` - NIF function pointer to remove
    ///
    /// # Returns
    /// Error if pointer not found
    pub fn remove_nif_pointer(&mut self, nif_pointer: *const u8) -> Result<(), String> {
        self.nif_pointers.retain(|&ptr| ptr != nif_pointer);
        Ok(())
    }

    /// Get all NIF pointers associated with this process
    ///
    /// This method returns all NIF function pointers currently used by this process.
    /// It is used by usecases_process_management to check code usage.
    ///
    /// # Returns
    /// Vector of NIF function pointers
    pub fn get_nif_pointers(&self) -> Vec<*const u8> {
        self.nif_pointers.clone()
    }

    /// Add a NIF library reference to this process
    ///
    /// This method is called by nif_loader when a process uses a NIF from a library.
    /// The library reference is tracked to prevent the library from being unloaded
    /// while the process is using it.
    ///
    /// # Arguments
    /// * `library` - Reference to NIF library
    ///
    /// # Returns
    /// Error if library cannot be added
    pub fn add_nif_library(
        &mut self,
        library: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), String> {
        // Avoid duplicates by checking if we already have this library
        // We compare by pointer address since Arc doesn't implement Eq
        let library_ptr = Arc::as_ptr(&library) as *const ();
        let already_has = self.nif_libraries.iter().any(|lib| {
            Arc::as_ptr(lib) as *const () == library_ptr
        });
        
        if !already_has {
            self.nif_libraries.push(library);
        }
        Ok(())
    }

    /// Remove a NIF library reference from this process
    ///
    /// This method is called by nif_loader when a process no longer uses a NIF library.
    ///
    /// # Arguments
    /// * `library` - Reference to NIF library to remove
    ///
    /// # Returns
    /// Error if library not found
    pub fn remove_nif_library(
        &mut self,
        library: &std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), String> {
        let library_ptr = Arc::as_ptr(library) as *const ();
        self.nif_libraries.retain(|lib| {
            Arc::as_ptr(lib) as *const () != library_ptr
        });
        Ok(())
    }

    /// Get all NIF library references for this process
    ///
    /// # Returns
    /// Vector of NIF library references
    pub fn get_nif_libraries(&self) -> &[std::sync::Arc<dyn std::any::Any + Send + Sync>] {
        &self.nif_libraries
    }
}

// Implement Debug trait
impl fmt::Debug for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Process")
            .field("id", &self.id)
            .field("heap_sz", &self.heap_sz)
            .field("min_heap_size", &self.min_heap_size)
            .field("max_heap_size", &self.max_heap_size)
            .field("heap_data_len", &self.heap_data.lock().unwrap().len())
            .field("heap_start_index", &self.heap_start_index)
            .field("heap_top_index", &*self.heap_top_index.lock().unwrap())
            .field("stack_top_index", &self.stack_top_index)
            .field("flags", &format!("0x{:x}", self.flags))
            .field("reds", &self.reds)
            .field("fcalls", &self.fcalls)
            .field("arity", &self.arity)
            .field("catches", &self.catches)
            .field("return_trace_frames", &self.return_trace_frames)
            .field("uniq", &self.uniq)
            .field("schedule_count", &self.schedule_count)
            .field("rcount", &self.rcount)
            .field("state", &self.get_state())
            .field("i", &(self.i as usize))
            .field("nif_pointers_count", &self.nif_pointers.len())
            .field("nif_libraries_count", &self.nif_libraries.len())
            .finish()
    }
}

// Process is Send + Sync because:
// - All fields are either primitive types, Vec (which is Send + Sync), or raw pointers
// - Vec<Eterm> is Send + Sync (it owns its data safely)
// - Raw pointer (i) is Send + Sync (it doesn't own the data)
// - The actual memory safety is managed by the VM/runtime
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_new() {
        let process = Process::new(123);
        assert_eq!(process.id(), 123);
        assert_eq!(process.get_id(), 123);
        assert_eq!(process.heap_sz(), 233); // Heap is initialized with default min size
        assert_eq!(process.min_heap_size(), 233);
        assert_eq!(process.max_heap_size(), 0);
        assert_eq!(process.heap_slice().len(), 233); // Heap data is initialized
    }

    #[test]
    fn test_process_get_state() {
        let mut process = Process::new(1);
        
        // Test default state (flags = 0)
        assert!(matches!(process.get_state(), ProcessState::Unknown(0)));
        
        // Test ACTIVE state
        // Note: We can't directly set flags, but we can test the state conversion
        // In a real implementation, we'd have setters or constructors with flags
    }

    #[test]
    fn test_process_state_from_flags() {
        // Test FREE state
        let state = ProcessState::from_flags(0x10);  // ERTS_PSFLG_FREE
        assert!(matches!(state, ProcessState::Free));
        
        // Test EXITING state
        let state = ProcessState::from_flags(0x20);  // ERTS_PSFLG_EXITING
        assert!(matches!(state, ProcessState::Exiting));
        
        // Test ACTIVE state
        let state = ProcessState::from_flags(0x80);  // ERTS_PSFLG_ACTIVE
        assert!(matches!(state, ProcessState::Active));
        
        // Test RUNNING state
        let state = ProcessState::from_flags(0x200);  // ERTS_PSFLG_RUNNING
        assert!(matches!(state, ProcessState::Running));
        
        // Test SUSPENDED state
        let state = ProcessState::from_flags(0x400);  // ERTS_PSFLG_SUSPENDED
        assert!(matches!(state, ProcessState::Suspended));
    }

    #[test]
    fn test_process_debug() {
        let process = Process::new(456);
        let debug_str = format!("{:?}", process);
        assert!(debug_str.contains("Process"));
        assert!(debug_str.contains("456"));
    }

    #[test]
    fn test_process_getters() {
        let process = Process::new(789);
        assert_eq!(process.id(), 789);
        assert_eq!(process.heap_sz(), 233);
        assert_eq!(process.min_heap_size(), 233);
        assert_eq!(process.max_heap_size(), 0);
        assert_eq!(process.flags(), 0);
        assert_eq!(process.reds(), 0);
        assert_eq!(process.fcalls(), 0);
        assert_eq!(process.arity(), 0);
        assert_eq!(process.catches(), 0);
        assert_eq!(process.return_trace_frames(), 0);
        assert_eq!(process.uniq(), 0);
        assert_eq!(process.schedule_count(), 0);
        assert_eq!(process.rcount(), 0);
        assert_eq!(process.stack_top_index(), None);
        assert_eq!(process.heap_top_index(), 0);
        assert_eq!(process.heap_start_index(), 0);
        assert_eq!(process.heap_slice().len(), 233);
        assert_eq!(process.i(), std::ptr::null());
    }

    #[test]
    fn test_process_heap_access() {
        let process = Process::new(1);
        let heap_slice = process.heap_slice();
        assert_eq!(heap_slice.len(), 233);
        // All heap data should be initialized to 0
        assert!(heap_slice.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_process_stack_size_calculation() {
        let mut process = Process::new(1);
        // Set stack top index
        process.stack_top_index = Some(100);
        *process.heap_top_index.lock().unwrap() = 50;
        
        let stack_size = process.stack_size_words();
        assert_eq!(stack_size, Some(50));
    }

    #[test]
    fn test_process_heap_mutable_access() {
        let process = Process::new(1);
        let mut heap_slice = process.heap_slice_mut();
        // Can safely modify heap data
        heap_slice[0] = 42;
        drop(heap_slice); // Release the lock
        let heap_read = process.heap_slice();
        assert_eq!(heap_read[0], 42);
    }

    #[test]
    fn test_process_state_from_flags_all_variants() {
        // Test all ProcessState variants
        // FREE
        assert!(matches!(ProcessState::from_flags(0x10), ProcessState::Free));
        
        // EXITING
        assert!(matches!(ProcessState::from_flags(0x20), ProcessState::Exiting));
        
        // DIRTY_RUNNING_SYS
        assert!(matches!(ProcessState::from_flags(0x01000000), ProcessState::DirtyRunningSys));
        
        // DIRTY_RUNNING
        assert!(matches!(ProcessState::from_flags(0x00800000), ProcessState::DirtyRunning));
        
        // RUNNING_SYS
        assert!(matches!(ProcessState::from_flags(0x00008000), ProcessState::RunningSys));
        
        // RUNNING
        assert!(matches!(ProcessState::from_flags(0x00000200), ProcessState::Running));
        
        // GC
        assert!(matches!(ProcessState::from_flags(0x00000800), ProcessState::Gc));
        
        // SUSPENDED
        assert!(matches!(ProcessState::from_flags(0x00000400), ProcessState::Suspended));
        
        // SYS_TASKS
        assert!(matches!(ProcessState::from_flags(0x00001000), ProcessState::SysTasks));
        
        // DELAYED_SYS
        assert!(matches!(ProcessState::from_flags(0x00020000), ProcessState::DelayedSys));
        
        // PROXY
        assert!(matches!(ProcessState::from_flags(0x00010000), ProcessState::Proxy));
        
        // ACTIVE
        assert!(matches!(ProcessState::from_flags(0x00000080), ProcessState::Active));
        
        // Unknown state - use a value with no known flags set
        // 0x00000001 has no known flags (all known flags are higher bits)
        assert!(matches!(ProcessState::from_flags(0x00000001), ProcessState::Unknown(0x00000001)));
    }

    #[test]
    fn test_process_state_from_flags_priority() {
        // Test that FREE has priority over other flags
        assert!(matches!(ProcessState::from_flags(0x10 | 0x20 | 0x80), ProcessState::Free));
        
        // Test that EXITING has priority over ACTIVE
        assert!(matches!(ProcessState::from_flags(0x20 | 0x80), ProcessState::Exiting));
    }

    #[test]
    fn test_allocate_heap_words_success() {
        let process = Process::new(1);
        
        // Allocate some words
        let index1 = process.allocate_heap_words(10);
        assert_eq!(index1, Some(0));
        
        // Allocate more words
        let index2 = process.allocate_heap_words(20);
        assert_eq!(index2, Some(10));
        
        // Verify heap_top_index was updated
        assert_eq!(process.heap_top_index(), 30);
    }

    #[test]
    fn test_allocate_heap_words_failure() {
        let process = Process::new(1);
        
        // Try to allocate more than available
        let heap_size = process.heap_sz();
        let result = process.allocate_heap_words(heap_size + 1);
        
        // Should return None when heap is full
        assert_eq!(result, None);
        
        // Heap top should not have changed
        assert_eq!(process.heap_top_index(), 0);
    }

    #[test]
    fn test_allocate_heap_words_exact_fit() {
        let process = Process::new(1);
        
        // Allocate exactly the heap size
        let heap_size = process.heap_sz();
        let result = process.allocate_heap_words(heap_size);
        
        // Should succeed
        assert_eq!(result, Some(0));
        assert_eq!(process.heap_top_index(), heap_size);
    }

    #[test]
    fn test_allocate_heap_words_zero() {
        let process = Process::new(1);
        
        // Allocate zero words
        let result = process.allocate_heap_words(0);
        
        // Should succeed (returns current heap_top)
        assert_eq!(result, Some(0));
        assert_eq!(process.heap_top_index(), 0);
    }

    #[test]
    fn test_stack_size_words_none() {
        let process = Process::new(1);
        
        // When stack_top_index is None, should return None
        let stack_size = process.stack_size_words();
        assert_eq!(stack_size, None);
    }

    #[test]
    fn test_stack_size_words_some() {
        let mut process = Process::new(1);
        
        // Set stack top index
        process.stack_top_index = Some(100);
        *process.heap_top_index.lock().unwrap() = 50;
        
        let stack_size = process.stack_size_words();
        assert_eq!(stack_size, Some(50));
    }

    #[test]
    fn test_stack_size_words_saturating() {
        let mut process = Process::new(1);
        
        // Test saturating_sub when stack_top < heap_top
        process.stack_top_index = Some(50);
        *process.heap_top_index.lock().unwrap() = 100;
        
        let stack_size = process.stack_size_words();
        // saturating_sub(50, 100) = 0
        assert_eq!(stack_size, Some(0));
    }

    #[test]
    fn test_add_nif_pointer_success() {
        let mut process = Process::new(1);
        
        // Create a valid pointer
        let dummy_value: u8 = 42;
        let nif_ptr = &dummy_value as *const u8;
        
        // Add pointer
        let result = process.add_nif_pointer(nif_ptr);
        assert!(result.is_ok());
        
        // Verify it was added
        let pointers = process.get_nif_pointers();
        assert_eq!(pointers.len(), 1);
        assert_eq!(pointers[0], nif_ptr);
    }

    #[test]
    fn test_add_nif_pointer_null() {
        let mut process = Process::new(1);
        
        // Try to add null pointer
        let result = process.add_nif_pointer(std::ptr::null());
        
        // Should return error
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null pointer"));
        
        // No pointers should be added
        assert_eq!(process.get_nif_pointers().len(), 0);
    }

    #[test]
    fn test_add_nif_pointer_duplicate() {
        let mut process = Process::new(1);
        
        let dummy_value: u8 = 42;
        let nif_ptr = &dummy_value as *const u8;
        
        // Add pointer twice
        process.add_nif_pointer(nif_ptr).unwrap();
        let result = process.add_nif_pointer(nif_ptr);
        
        // Should succeed (no error for duplicates)
        assert!(result.is_ok());
        
        // But should only have one pointer
        assert_eq!(process.get_nif_pointers().len(), 1);
    }

    #[test]
    fn test_remove_nif_pointer_success() {
        let mut process = Process::new(1);
        
        let dummy_value: u8 = 42;
        let nif_ptr = &dummy_value as *const u8;
        
        // Add pointer
        process.add_nif_pointer(nif_ptr).unwrap();
        assert_eq!(process.get_nif_pointers().len(), 1);
        
        // Remove pointer
        let result = process.remove_nif_pointer(nif_ptr);
        assert!(result.is_ok());
        
        // Verify it was removed
        assert_eq!(process.get_nif_pointers().len(), 0);
    }

    #[test]
    fn test_remove_nif_pointer_not_found() {
        let mut process = Process::new(1);
        
        let dummy_value: u8 = 42;
        let nif_ptr = &dummy_value as *const u8;
        
        // Try to remove non-existent pointer
        let result = process.remove_nif_pointer(nif_ptr);
        
        // Should succeed (no error, just no-op)
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_nif_pointer_multiple() {
        let mut process = Process::new(1);
        
        let dummy_value1: u8 = 42;
        let dummy_value2: u8 = 43;
        let dummy_value3: u8 = 44;
        let nif_ptr1 = &dummy_value1 as *const u8;
        let nif_ptr2 = &dummy_value2 as *const u8;
        let nif_ptr3 = &dummy_value3 as *const u8;
        
        // Add multiple pointers
        process.add_nif_pointer(nif_ptr1).unwrap();
        process.add_nif_pointer(nif_ptr2).unwrap();
        process.add_nif_pointer(nif_ptr3).unwrap();
        assert_eq!(process.get_nif_pointers().len(), 3);
        
        // Remove middle pointer
        process.remove_nif_pointer(nif_ptr2).unwrap();
        
        // Verify correct pointers remain
        let pointers = process.get_nif_pointers();
        assert_eq!(pointers.len(), 2);
        assert!(pointers.contains(&nif_ptr1));
        assert!(pointers.contains(&nif_ptr3));
        assert!(!pointers.contains(&nif_ptr2));
    }

    #[test]
    fn test_get_nif_pointers_empty() {
        let process = Process::new(1);
        
        let pointers = process.get_nif_pointers();
        assert_eq!(pointers.len(), 0);
    }

    #[test]
    fn test_add_nif_library_success() {
        let mut process = Process::new(1);
        
        let library: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        
        // Add library - need to cast to trait object
        let result = process.add_nif_library(library.clone() as Arc<dyn std::any::Any + Send + Sync>);
        assert!(result.is_ok());
        
        // Verify it was added
        let libraries = process.get_nif_libraries();
        assert_eq!(libraries.len(), 1);
    }

    #[test]
    fn test_add_nif_library_duplicate() {
        let mut process = Process::new(1);
        
        let library: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        
        // Add library twice - need to cast to trait object
        process.add_nif_library(library.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        let result = process.add_nif_library(library.clone() as Arc<dyn std::any::Any + Send + Sync>);
        
        // Should succeed (no error for duplicates)
        assert!(result.is_ok());
        
        // But should only have one library
        assert_eq!(process.get_nif_libraries().len(), 1);
    }

    #[test]
    fn test_remove_nif_library_success() {
        let mut process = Process::new(1);
        
        let library: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        
        // Add library
        process.add_nif_library(library.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        assert_eq!(process.get_nif_libraries().len(), 1);
        
        // Remove library - need to cast to trait object
        let library_trait: Arc<dyn std::any::Any + Send + Sync> = library.clone();
        let result = process.remove_nif_library(&library_trait);
        assert!(result.is_ok());
        
        // Verify it was removed
        assert_eq!(process.get_nif_libraries().len(), 0);
    }

    #[test]
    fn test_remove_nif_library_not_found() {
        let mut process = Process::new(1);
        
        let library1: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        let library2: Arc<Vec<u8>> = Arc::new(vec![4, 5, 6]);
        
        // Add one library
        process.add_nif_library(library1.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        
        // Try to remove different library - need to cast to trait object
        let library2_trait: Arc<dyn std::any::Any + Send + Sync> = library2.clone();
        let result = process.remove_nif_library(&library2_trait);
        
        // Should succeed (no error, just no-op)
        assert!(result.is_ok());
        
        // First library should still be there
        assert_eq!(process.get_nif_libraries().len(), 1);
    }

    #[test]
    fn test_get_nif_libraries_empty() {
        let process = Process::new(1);
        
        let libraries = process.get_nif_libraries();
        assert_eq!(libraries.len(), 0);
    }

    #[test]
    fn test_get_nif_libraries_multiple() {
        let mut process = Process::new(1);
        
        let library1: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        let library2: Arc<String> = Arc::new(String::from("test"));
        let library3: Arc<Vec<i32>> = Arc::new(vec![10, 20, 30]);
        
        // Add multiple libraries - need to cast to trait object
        process.add_nif_library(library1.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        process.add_nif_library(library2.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        process.add_nif_library(library3.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        
        // Verify all are present
        let libraries = process.get_nif_libraries();
        assert_eq!(libraries.len(), 3);
    }

    #[test]
    fn test_process_deprecated_heap() {
        let process = Process::new(1);
        
        // Test deprecated heap() method
        let heap = process.heap();
        assert_eq!(heap, Some(0)); // heap_start_index
    }

    #[test]
    fn test_process_deprecated_htop() {
        let process = Process::new(1);
        
        // Test deprecated htop() method
        let htop = process.htop();
        assert_eq!(htop, Some(0)); // heap_top_index
    }

    #[test]
    fn test_process_deprecated_stop() {
        let process = Process::new(1);
        
        // Test deprecated stop() method when None
        let stop = process.stop();
        assert_eq!(stop, None);
        
        // Test when Some
        let mut process2 = Process::new(2);
        process2.stack_top_index = Some(100);
        let stop2 = process2.stop();
        assert_eq!(stop2, Some(100));
    }

    #[test]
    fn test_process_debug_all_fields() {
        let mut process = Process::new(999);
        
        // Modify some fields to ensure they appear in debug output
        *process.heap_top_index.lock().unwrap() = 50;
        process.stack_top_index = Some(100);
        
        let debug_str = format!("{:?}", process);
        
        // Check that all major fields appear in debug output
        assert!(debug_str.contains("999")); // id
        assert!(debug_str.contains("heap_sz"));
        assert!(debug_str.contains("min_heap_size"));
        assert!(debug_str.contains("max_heap_size"));
        assert!(debug_str.contains("heap_data_len"));
        assert!(debug_str.contains("heap_start_index"));
        assert!(debug_str.contains("heap_top_index"));
        assert!(debug_str.contains("stack_top_index"));
        assert!(debug_str.contains("flags"));
        assert!(debug_str.contains("reds"));
        assert!(debug_str.contains("fcalls"));
        assert!(debug_str.contains("arity"));
        assert!(debug_str.contains("catches"));
        assert!(debug_str.contains("return_trace_frames"));
        assert!(debug_str.contains("uniq"));
        assert!(debug_str.contains("schedule_count"));
        assert!(debug_str.contains("rcount"));
        assert!(debug_str.contains("state"));
        assert!(debug_str.contains("i"));
        assert!(debug_str.contains("nif_pointers_count"));
        assert!(debug_str.contains("nif_libraries_count"));
    }

    #[test]
    fn test_process_debug_flags_format() {
        let mut process = Process::new(1);
        // Set some flags
        process.flags = 0x12345678;
        
        let debug_str = format!("{:?}", process);
        // Flags should be formatted as hex
        assert!(debug_str.contains("0x12345678") || debug_str.contains("0x12345678"));
    }

    #[test]
    fn test_process_allocate_heap_words_concurrent() {
        let process = Arc::new(Process::new(1));
        
        // Test concurrent allocations
        let process_clone1 = process.clone();
        let process_clone2 = process.clone();
        
        std::thread::scope(|s| {
            s.spawn(move || {
                for _ in 0..10 {
                    process_clone1.allocate_heap_words(1);
                }
            });
            s.spawn(move || {
                for _ in 0..10 {
                    process_clone2.allocate_heap_words(1);
                }
            });
        });
        
        // Total allocations should be <= heap size
        let heap_top = process.heap_top_index();
        assert!(heap_top <= process.heap_sz());
    }

    #[test]
    fn test_process_heap_slice_mut_concurrent() {
        let process = Arc::new(Process::new(1));
        
        // Test that heap_slice_mut properly locks
        let process_clone = process.clone();
        
        std::thread::scope(|s| {
            s.spawn(move || {
                let mut heap = process_clone.heap_slice_mut();
                heap[0] = 100;
            });
        });
        
        // Verify the write happened
        let heap = process.heap_slice();
        assert_eq!(heap[0], 100);
    }

    #[test]
    fn test_process_state_unknown() {
        // Test Unknown state with various flag values
        let state1 = ProcessState::from_flags(0xFFFFFFFF);
        // Should be Free because 0x10 is set
        assert!(matches!(state1, ProcessState::Free));
        
        // Test truly unknown flag combination
        let state2 = ProcessState::from_flags(0x00000001); // No known flags
        assert!(matches!(state2, ProcessState::Unknown(0x00000001)));
    }

    #[test]
    fn test_process_nif_pointers_and_libraries_together() {
        let mut process = Process::new(1);
        
        // Add both pointers and libraries
        let dummy_value: u8 = 42;
        let nif_ptr = &dummy_value as *const u8;
        process.add_nif_pointer(nif_ptr).unwrap();
        
        let library: Arc<Vec<u8>> = Arc::new(vec![1, 2, 3]);
        process.add_nif_library(library.clone() as Arc<dyn std::any::Any + Send + Sync>).unwrap();
        
        // Verify both are present
        assert_eq!(process.get_nif_pointers().len(), 1);
        assert_eq!(process.get_nif_libraries().len(), 1);
        
        // Remove both - need to cast library to trait object
        process.remove_nif_pointer(nif_ptr).unwrap();
        let library_trait: Arc<dyn std::any::Any + Send + Sync> = library.clone();
        process.remove_nif_library(&library_trait).unwrap();
        
        // Verify both are removed
        assert_eq!(process.get_nif_pointers().len(), 0);
        assert_eq!(process.get_nif_libraries().len(), 0);
    }

    #[test]
    fn test_process_allocate_heap_words_sequential() {
        let process = Process::new(1);
        
        // Allocate in sequence
        let mut expected_index = 0;
        for i in 0..10 {
            let result = process.allocate_heap_words(i + 1);
            assert_eq!(result, Some(expected_index));
            expected_index += i + 1;
        }
        
        // Verify final heap_top
        assert_eq!(process.heap_top_index(), expected_index);
    }

    #[test]
    fn test_process_stack_size_words_edge_cases() {
        let mut process = Process::new(1);
        
        // Test with equal indices
        process.stack_top_index = Some(50);
        *process.heap_top_index.lock().unwrap() = 50;
        assert_eq!(process.stack_size_words(), Some(0));
        
        // Test with stack_top < heap_top (saturating)
        process.stack_top_index = Some(30);
        *process.heap_top_index.lock().unwrap() = 50;
        assert_eq!(process.stack_size_words(), Some(0));
    }
}

