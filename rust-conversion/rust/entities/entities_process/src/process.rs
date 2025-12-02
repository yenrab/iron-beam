//! Process Entity
//!
//! Provides the Process struct and related types for the Erlang runtime system.
//! Based on erts/emulator/beam/erl_process.h
//!
//! The heap is implemented using safe Rust (`Vec<Eterm>`) with index-based
//! access instead of raw pointers for maximum safety.

use std::fmt;

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
    /// Heap data storage (safe Rust Vec)
    heap_data: Vec<Eterm>,
    /// Heap start index (usually 0, but can be offset if needed)
    heap_start_index: usize,
    /// Heap top index (current position where new data is allocated)
    heap_top_index: usize,
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
            heap_data,
            heap_start_index: 0,
            heap_top_index: 0,
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
        self.heap_top_index
    }

    /// Get heap start index
    pub fn heap_start_index(&self) -> usize {
        self.heap_start_index
    }

    /// Get heap data as a slice (safe access to heap contents)
    pub fn heap_slice(&self) -> &[Eterm] {
        &self.heap_data
    }

    /// Get heap data as a mutable slice (for heap modifications)
    pub fn heap_slice_mut(&mut self) -> &mut [Eterm] {
        &mut self.heap_data
    }

    /// Calculate stack size in words
    /// Returns None if stack_top_index is not set
    pub fn stack_size_words(&self) -> Option<usize> {
        self.stack_top_index.map(|stop| stop.saturating_sub(self.heap_top_index))
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
        Some(self.heap_top_index)
    }

    #[deprecated(note = "Use stack_top_index() instead")]
    pub fn stop(&self) -> Option<usize> {
        self.stack_top_index
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
            .field("heap_data_len", &self.heap_data.len())
            .field("heap_start_index", &self.heap_start_index)
            .field("heap_top_index", &self.heap_top_index)
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
        process.heap_top_index = 50;
        
        let stack_size = process.stack_size_words();
        assert_eq!(stack_size, Some(50));
    }

    #[test]
    fn test_process_heap_mutable_access() {
        let mut process = Process::new(1);
        let heap_slice = process.heap_slice_mut();
        // Can safely modify heap data
        heap_slice[0] = 42;
        assert_eq!(process.heap_slice()[0], 42);
    }
}

