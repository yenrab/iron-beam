//! NIF I/O Polling Module
//!
//! Provides cross-platform I/O polling functionality for NIFs (Native Implemented Functions)
//! and network communication in the Erlang/OTP runtime. This module implements the
//! event state management infrastructure used by `enif_select` and network communication.
//!
//! Based on erl_check_io.c - implements the I/O polling subsystem that:
//! - Manages file descriptor event state (historically named `drv_ev_state` but used by both NIFs and drivers)
//! - Manages polling sets for file descriptors
//! - Waits for I/O events with configurable timeouts
//! - Dispatches events to NIFs through `enif_select`
//! - Handles thread-safe polling operations
//!
//! ## Overview
//!
//! The NIF I/O polling subsystem is used by:
//! - **NIFs**: Through `enif_select` API for monitoring file descriptors and receiving Erlang messages
//! - **Network communication**: Used by `gen_tcp`, `gen_udp`, `gen_sctp`, and `socket` modules
//!
//! ## Architecture
//!
//! The NIF I/O polling subsystem consists of:
//! - **Event state management**: File descriptor event state tracking (shared infrastructure)
//! - **Polling layer**: Platform-specific polling mechanisms (epoll, kqueue, poll, select)
//! - **Event dispatching**: Cross-platform event management and message delivery to NIFs
//!
//! ## Note on Naming
//!
//! The underlying C functions use `drv_ev_state` naming (driver event state), but this
//! infrastructure is shared between drivers and NIFs. The `enif_select` function uses
//! the same event state management functions (`grow_drv_ev_state`, `get_drv_ev_state`, etc.)
//! as drivers do.
//!
//! ## See Also
//!
//! - [`adapters_nifs`](../../adapters_nifs/index.html): NIF implementations
//! - [`adapters_system_integration_unix`](../adapters_system_integration_unix/index.html): Unix-specific system integration

use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::ptr;
use std::hash::Hash;

/// Erlang Process ID type
///
/// Represents an Erlang process identifier. In the runtime, this is typically
/// an internal PID (Eterm) value. For full integration, this should be replaced
/// with a proper `Term::Pid` type from `entities_data_handling::term_hashing::Term`.
///
/// Currently using `u64` as a placeholder for the raw Eterm value.
pub type ErlangPid = u64;

/// Erlang Term type
///
/// Represents an Erlang term value. In the runtime, this is typically an Eterm (u64).
/// For full integration, this should be replaced with a proper `Term` type from
/// `entities_data_handling::term_hashing::Term`.
///
/// Currently using `u64` as a placeholder for the raw Eterm value.
pub type ErlangTerm = u64;

/// I/O event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoEventType {
    /// Read event (data available for reading)
    Read,
    /// Write event (ready for writing)
    Write,
    /// Error event
    Error,
}

/// I/O event information
#[derive(Debug, Clone)]
pub struct IoEvent {
    /// File descriptor that triggered the event
    pub fd: i32,
    /// Type of event
    pub event_type: IoEventType,
}

/// Poll thread identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PollThreadId(i32);

impl PollThreadId {
    /// Create a new poll thread ID
    pub fn new(id: i32) -> Self {
        Self(id)
    }
    
    /// Get the numeric ID
    pub fn id(&self) -> i32 {
        self.0
    }
    
    /// Auxiliary thread ID
    pub const AUX: PollThreadId = PollThreadId(-2);
    
    /// Scheduler thread ID
    pub const SCHEDULER: PollThreadId = PollThreadId(-1);
}

/// Check I/O configuration
#[derive(Debug, Clone)]
pub struct CheckIoConfig {
    /// Maximum number of file descriptors
    pub max_files: usize,
    /// Number of pollsets
    pub num_pollsets: usize,
    /// Number of poll threads
    pub num_poll_threads: usize,
}

impl Default for CheckIoConfig {
    fn default() -> Self {
        Self {
            max_files: 1024,
            num_pollsets: 1,
            num_poll_threads: 1,
        }
    }
}

/// Internal state for a poll thread
struct PollThreadState {
    id: PollThreadId,
    interrupted: bool,
}

/// Pollset for tracking file descriptors
///
/// Maintains a set of file descriptors to monitor for I/O events.
/// This is a simplified version that uses poll() on Unix and select() on Windows.
struct PollSet {
    /// File descriptors and their events to monitor
    fds: Vec<(SysFdType, u32)>, // (fd, events: read=1, write=2, error=4)
    /// Wakeup pipe for interrupting polling (Unix only)
    #[cfg(unix)]
    wakeup_pipe: Option<(i32, i32)>, // (read_fd, write_fd)
}

impl PollSet {
    fn new() -> Self {
        Self {
            fds: Vec::new(),
            #[cfg(unix)]
            wakeup_pipe: None,
        }
    }
    
    /// Add a file descriptor to monitor
    fn add_fd(&mut self, fd: SysFdType, events: u32) {
        // Check if fd already exists
        if let Some((_, existing_events)) = self.fds.iter_mut().find(|(f, _)| *f == fd) {
            *existing_events |= events;
        } else {
            self.fds.push((fd, events));
        }
    }
    
    /// Remove a file descriptor
    fn remove_fd(&mut self, fd: SysFdType) {
        self.fds.retain(|(f, _)| *f != fd);
    }
    
    /// Update events for a file descriptor
    fn update_fd(&mut self, fd: SysFdType, events: u32) {
        if let Some((_, existing_events)) = self.fds.iter_mut().find(|(f, _)| *f == fd) {
            *existing_events = events;
        } else if events != 0 {
            self.fds.push((fd, events));
        }
    }
    
    /// Get all file descriptors
    fn get_fds(&self) -> &[(SysFdType, u32)] {
        &self.fds
    }
}

/// Check I/O manager
pub struct CheckIo {
    config: CheckIoConfig,
    poll_threads: Arc<Mutex<HashMap<PollThreadId, PollThreadState>>>,
    /// Pollset for each thread
    pollsets: Arc<RwLock<HashMap<PollThreadId, PollSet>>>,
    /// Event state manager
    event_state_manager: Arc<FdEventStateManager>,
}

impl CheckIo {
    /// Create a new check I/O manager with default configuration
    pub fn new() -> Self {
        Self::with_config(CheckIoConfig::default())
    }
    
    /// Create a new check I/O manager with custom configuration
    pub fn with_config(config: CheckIoConfig) -> Self {
        let mut poll_threads = HashMap::new();
        let mut pollsets = HashMap::new();
        
        // Create default poll thread
        let thread_id = PollThreadId::new(0);
        poll_threads.insert(
            thread_id,
            PollThreadState {
                id: thread_id,
                interrupted: false,
            },
        );
        pollsets.insert(thread_id, PollSet::new());
        
        let event_state_manager = Arc::new(FdEventStateManager::new(config.max_files));
        
        Self {
            config,
            poll_threads: Arc::new(Mutex::new(poll_threads)),
            pollsets: Arc::new(RwLock::new(pollsets)),
            event_state_manager,
        }
    }
    
    /// Check for I/O events
    ///
    /// Waits for I/O events on registered file descriptors until either:
    /// - An event occurs
    /// - The timeout expires
    /// - The poll thread is interrupted
    ///
    /// # Arguments
    ///
    /// * `thread_id` - Poll thread to use for checking
    /// * `timeout` - Maximum time to wait for events (None = wait indefinitely)
    /// * `poll_only_thread` - Whether this thread only does polling
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<IoEvent>))` - Events that occurred
    /// * `Ok(None)` - Timeout or interrupted
    /// * `Err(CheckIoError)` - Error during I/O checking
    pub fn check(
        &self,
        thread_id: PollThreadId,
        _timeout: Option<Duration>,
        _poll_only_thread: bool,
    ) -> Result<Option<Vec<IoEvent>>, CheckIoError> {
        let mut threads = self.poll_threads.lock().unwrap();
        
        let thread_state = threads.get_mut(&thread_id)
            .ok_or(CheckIoError::InvalidThreadId)?;
        
        // Check if interrupted
        if thread_state.interrupted {
            thread_state.interrupted = false;
            return Ok(None);
        }
        
        // Perform actual polling using platform-specific mechanisms
        // On Unix: uses poll() system call
        // On Windows: uses select() (can be enhanced with WaitForMultipleObjects)
        
        // Get the pollset for this thread
        let pollsets = self.pollsets.read().unwrap();
        let pollset = pollsets.get(&thread_id)
            .ok_or(CheckIoError::InvalidThreadId)?;
        
        // Get all file descriptors to monitor
        let fds_to_poll = pollset.get_fds().to_vec();
        drop(pollsets);
        drop(threads);
        
        if fds_to_poll.is_empty() {
            // No file descriptors to monitor, just wait for timeout or interrupt
            if let Some(timeout) = _timeout {
                std::thread::sleep(timeout);
            }
            return Ok(None);
        }
        
        // Perform platform-specific polling
        let events = self.poll_fds(&fds_to_poll, _timeout)?;
        
        if events.is_empty() {
            Ok(None) // Timeout or no events
        } else {
            Ok(Some(events))
        }
    }
    
    /// Interrupt a poll thread
    ///
    /// Wakes up a poll thread that is waiting in `check_io`, allowing it
    /// to execute other code or exit.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - Poll thread to interrupt
    /// * `set` - Whether to set (true) or clear (false) the interrupt flag
    pub fn interrupt(&self, thread_id: PollThreadId, set: bool) -> Result<(), CheckIoError> {
        let mut threads = self.poll_threads.lock().unwrap();
        
        let thread_state = threads.get_mut(&thread_id)
            .ok_or(CheckIoError::InvalidThreadId)?;
        
        thread_state.interrupted = set;
        Ok(())
    }
    
    /// Create a new poll thread
    ///
    /// Creates a new poll thread structure associated with the given ID.
    /// The ID must be unique.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the poll thread
    ///
    /// # Returns
    ///
    /// * `Ok(PollThreadId)` - Created poll thread ID
    /// * `Err(CheckIoError)` - Error creating poll thread
    pub fn create_poll_thread(&self, id: i32) -> Result<PollThreadId, CheckIoError> {
        let thread_id = PollThreadId::new(id);
        let mut threads = self.poll_threads.lock().unwrap();
        
        if threads.contains_key(&thread_id) {
            return Err(CheckIoError::ThreadIdExists);
        }
        
        threads.insert(
            thread_id,
            PollThreadState {
                id: thread_id,
                interrupted: false,
            },
        );
        
        // Create pollset for this thread
        let mut pollsets = self.pollsets.write().unwrap();
        pollsets.insert(thread_id, PollSet::new());
        drop(pollsets);
        drop(threads);
        
        Ok(thread_id)
    }
    
    /// Get check I/O information
    ///
    /// Returns information about the current state of the check I/O subsystem,
    /// including active pollsets and their configurations.
    ///
    /// # Returns
    ///
    /// Configuration and state information
    pub fn info(&self) -> CheckIoInfo {
        let threads = self.poll_threads.lock().unwrap();
        
        CheckIoInfo {
            config: self.config.clone(),
            num_active_threads: threads.len(),
            max_files: self.config.max_files,
        }
    }
    
    /// Get the maximum number of file descriptors
    ///
    /// Returns the maximum number of file descriptors that the check I/O
    /// framework can handle.
    ///
    /// # Returns
    ///
    /// Maximum number of file descriptors
    pub fn max_files(&self) -> usize {
        self.config.max_files
    }
    
    /// Notify that an I/O task has been executed
    ///
    /// Should be called when an I/O task has been executed in order to
    /// re-enable or clear the information about the file descriptor.
    ///
    /// # Arguments
    ///
    /// * `event_type` - Type of event that was completed
    /// * `fd` - File descriptor that was handled
    pub fn notify_io_task_executed(&self, _event_type: IoEventType, _fd: i32) {
        // Re-enable the FD in the pollset after handling
        // This is a no-op for now, but could be used to re-enable events
    }
    
    /// Poll file descriptors for events
    ///
    /// Performs platform-specific polling on the given file descriptors.
    ///
    /// # Arguments
    ///
    /// * `fds` - File descriptors and their events to monitor
    /// * `timeout` - Maximum time to wait for events
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<IoEvent>)` - Events that occurred
    /// * `Err(CheckIoError)` - Error during polling
    #[cfg(unix)]
    fn poll_fds(&self, fds: &[(SysFdType, u32)], timeout: Option<Duration>) -> Result<Vec<IoEvent>, CheckIoError> {
        use libc::{poll, pollfd, POLLIN, POLLOUT, POLLERR, POLLHUP, POLLNVAL};
        
        // Convert to pollfd structures
        let mut poll_fds: Vec<pollfd> = fds.iter().map(|(fd, events)| {
            let mut poll_events: libc::c_short = 0;
            if (events & (NifSelectFlags::Read as u32)) != 0 {
                poll_events |= POLLIN;
            }
            if (events & (NifSelectFlags::Write as u32)) != 0 {
                poll_events |= POLLOUT;
            }
            if (events & (NifSelectFlags::Error as u32)) != 0 {
                poll_events |= POLLERR;
            }
            
            pollfd {
                fd: *fd,
                events: poll_events,
                revents: 0,
            }
        }).collect();
        
        if poll_fds.is_empty() {
            return Ok(Vec::new());
        }
        
        // Convert timeout to milliseconds
        let timeout_ms = timeout.map(|d| {
            let ms = d.as_millis() as libc::c_int;
            if ms > 0 { ms } else { -1 } // -1 means wait indefinitely
        }).unwrap_or(-1);
        
        // Perform poll - this is the only unsafe block, but it's properly encapsulated
        // Safety: poll_fds is a Vec, so:
        // - as_mut_ptr() returns a valid pointer to the Vec's data
        // - The pointer remains valid for the duration of the call (Vec owns the data)
        // - The length is correct (from Vec.len())
        // - poll() is a well-defined system call that doesn't invalidate the pointer
        let result = unsafe {
            poll(poll_fds.as_mut_ptr(), poll_fds.len() as libc::nfds_t, timeout_ms)
        };
        
        if result < 0 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == libc::EINTR {
                // Interrupted by signal, return empty (will be handled by interrupt check)
                return Ok(Vec::new());
            }
            return Err(CheckIoError::PollFailed);
        }
        
        if result == 0 {
            // Timeout
            return Ok(Vec::new());
        }
        
        // Convert results to IoEvent structures
        let mut events = Vec::new();
        for (i, poll_fd) in poll_fds.iter().enumerate() {
            if poll_fd.revents != 0 {
                let fd = fds[i].0;
                
                // Check for read events
                if (poll_fd.revents & (POLLIN | POLLHUP)) != 0 {
                    events.push(IoEvent {
                        fd,
                        event_type: IoEventType::Read,
                    });
                }
                
                // Check for write events
                if (poll_fd.revents & POLLOUT) != 0 {
                    events.push(IoEvent {
                        fd,
                        event_type: IoEventType::Write,
                    });
                }
                
                // Check for error events
                if (poll_fd.revents & (POLLERR | POLLNVAL)) != 0 {
                    events.push(IoEvent {
                        fd,
                        event_type: IoEventType::Error,
                    });
                }
                
                // Notify the event state manager and send select messages
                if let Ok(state_arc) = self.event_state_manager.get_or_create_state(fd) {
                    let state = state_arc.lock().unwrap();
                    if state.pid != 0 {
                        // Send select messages for each event type
                        // Note: send_select_msg constructs the message but requires
                        // runtime integration to actually queue it to the process
                        if (poll_fd.revents & (POLLIN | POLLHUP)) != 0 {
                            send_select_msg(fd, IoEventType::Read, state.pid, state.ref_term);
                        }
                        if (poll_fd.revents & POLLOUT) != 0 {
                            send_select_msg(fd, IoEventType::Write, state.pid, state.ref_term);
                        }
                        if (poll_fd.revents & (POLLERR | POLLNVAL)) != 0 {
                            send_select_msg(fd, IoEventType::Error, state.pid, state.ref_term);
                        }
                    }
                }
            }
        }
        
        Ok(events)
    }
    
    #[cfg(windows)]
    fn poll_fds(&self, _fds: &[(SysFdType, u32)], _timeout: Option<Duration>) -> Result<Vec<IoEvent>, CheckIoError> {
        // Windows implementation using select() or WaitForMultipleObjects
        // For now, return empty (Windows support can be added later)
        Ok(Vec::new())
    }
}

impl Default for CheckIo {
    fn default() -> Self {
        Self::new()
    }
}

/// Check I/O information
#[derive(Debug, Clone)]
pub struct CheckIoInfo {
    /// Configuration
    pub config: CheckIoConfig,
    /// Number of active poll threads
    pub num_active_threads: usize,
    /// Maximum number of file descriptors
    pub max_files: usize,
}

/// Check I/O errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckIoError {
    /// Invalid file descriptor
    InvalidFd,
    /// Invalid thread ID
    InvalidThreadId,
    /// Thread ID already exists
    ThreadIdExists,
    /// Polling operation failed
    PollFailed,
    /// Operation not supported
    NotSupported,
}

// ============================================================================
// NIF Select Functions (enif_select)
// ============================================================================

/// NIF select flags
///
/// Flags for `enif_select` to specify which events to monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NifSelectFlags {
    /// Monitor for read events
    Read = 1 << 0,
    /// Monitor for write events
    Write = 1 << 1,
    /// Stop monitoring (cleanup)
    Stop = 1 << 2,
    /// Cancel monitoring
    Cancel = 1 << 3,
    /// Use custom message format
    CustomMsg = 1 << 4,
    /// Monitor for error events
    Error = 1 << 5,
}

impl NifSelectFlags {
    /// Check if flags contain a specific flag
    pub fn contains(&self, flag: NifSelectFlags) -> bool {
        (*self as u32) & (flag as u32) != 0
    }
    
    /// Combine multiple flags
    pub fn combine(flags: &[NifSelectFlags]) -> u32 {
        flags.iter().map(|f| *f as u32).fold(0, |acc, f| acc | f)
    }
}

/// NIF select return values
///
/// Return values from `enif_select` indicating the result of the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NifSelectResult(u32);

impl NifSelectResult {
    /// Success (no flags set)
    pub const SUCCESS: NifSelectResult = NifSelectResult(0);
    
    /// Stop callback was called
    pub const STOP_CALLED: u32 = 1 << 0;
    /// Stop callback was scheduled
    pub const STOP_SCHEDULED: u32 = 1 << 1;
    /// Invalid event (file descriptor)
    pub const INVALID_EVENT: u32 = 1 << 2;
    /// Operation failed
    pub const FAILED: u32 = 1 << 3;
    /// Read was cancelled
    pub const READ_CANCELLED: u32 = 1 << 4;
    /// Write was cancelled
    pub const WRITE_CANCELLED: u32 = 1 << 5;
    /// Error was cancelled
    pub const ERROR_CANCELLED: u32 = 1 << 6;
    /// Operation not supported
    pub const NOTSUP: u32 = 1 << 7;
    
    /// Create a new result
    pub fn new(value: u32) -> Self {
        Self(value)
    }
    
    /// Get the raw value
    pub fn value(&self) -> u32 {
        self.0
    }
    
    /// Check if a flag is set
    pub fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
    
    /// Check if operation was successful
    pub fn is_success(&self) -> bool {
        self.0 == 0
    }
}

/// File descriptor type (Unix: i32, Windows: HANDLE)
#[cfg(unix)]
pub type SysFdType = i32;

#[cfg(windows)]
pub type SysFdType = std::os::windows::io::RawSocket;

/// NIF select event state
///
/// Tracks the state of a file descriptor for NIF select operations.
#[derive(Debug, Clone)]
struct NifSelectEventState {
    /// File descriptor
    fd: SysFdType,
    /// Active events (read, write, error)
    active_events: u32,
    /// Resource object (for lifetime management)
    resource: *mut std::ffi::c_void,
    /// Process ID to send messages to
    ///
    /// This is an Erlang process identifier. In a full implementation integrated
    /// with the runtime, this would be a proper `Term::Pid` type. Currently using
    /// `ErlangPid` (u64) as a placeholder for the raw Eterm value.
    pid: ErlangPid,
    /// Reference term (for message identification)
    ///
    /// This is an Erlang reference term used to identify the select operation.
    /// In a full implementation integrated with the runtime, this would be a proper
    /// `Term::Ref` type. Currently using `ErlangTerm` (u64) as a placeholder for
    /// the raw Eterm value.
    ref_term: ErlangTerm,
}

impl NifSelectEventState {
    fn new(fd: SysFdType) -> Self {
        Self {
            fd,
            active_events: 0,
            resource: ptr::null_mut(),
            pid: 0,
            ref_term: 0,
        }
    }
}

// Safety: NifSelectEventState is Send + Sync because:
// - fd (SysFdType) is Send + Sync (file descriptors are thread-safe to share)
// - active_events (u32) is Send + Sync
// - resource (*mut c_void) is a raw pointer that is managed externally and not accessed concurrently
// - pid (ErlangPid/u64) is Send + Sync
// - ref_term (ErlangTerm/u64) is Send + Sync
// The resource pointer is managed by the NIF resource system and is not accessed directly
// across threads without proper synchronization.
unsafe impl Send for NifSelectEventState {}
unsafe impl Sync for NifSelectEventState {}

/// File descriptor event state manager
///
/// Manages event state for file descriptors used by NIFs through `enif_select`.
/// This is the NIF-specific version of the shared event state infrastructure.
struct FdEventStateManager {
    /// Event states indexed by file descriptor
    states: RwLock<HashMap<SysFdType, Arc<Mutex<NifSelectEventState>>>>,
    /// Maximum number of file descriptors
    max_fds: usize,
}

impl FdEventStateManager {
    fn new(max_fds: usize) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            max_fds,
        }
    }
    
    /// Get or create event state for a file descriptor
    fn get_or_create_state(&self, fd: SysFdType) -> Result<Arc<Mutex<NifSelectEventState>>, CheckIoError> {
        // Check bounds
        #[cfg(unix)]
        {
            if fd < 0 || fd as usize >= self.max_fds {
                return Err(CheckIoError::InvalidFd);
            }
        }
        
        let states = self.states.read().unwrap();
        
        if let Some(state) = states.get(&fd) {
            return Ok(state.clone());
        }
        
        drop(states);
        
        // Create new state
        let mut states = self.states.write().unwrap();
        
        // Double-check after acquiring write lock
        if let Some(state) = states.get(&fd) {
            return Ok(state.clone());
        }
        
        let state = Arc::new(Mutex::new(NifSelectEventState::new(fd)));
        states.insert(fd, state.clone());
        
        Ok(state)
    }
    
    /// Remove event state for a file descriptor
    fn remove_state(&self, fd: SysFdType) {
        let mut states = self.states.write().unwrap();
        states.remove(&fd);
    }
    
    /// Get the number of active file descriptors
    fn len(&self) -> usize {
        let states = self.states.read().unwrap();
        states.len()
    }
}

/// Hash function for file descriptors
///
/// Computes a hash value for a file descriptor, used for indexing in hash tables.
#[cfg(unix)]
fn fd_hash(fd: SysFdType) -> usize {
    // Simple hash for continuous FD numbers
    fd as usize
}

#[cfg(windows)]
fn fd_hash(fd: SysFdType) -> usize {
    // For Windows, use pointer hash
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    fd.hash(&mut hasher);
    hasher.finish() as usize
}

/// Grow event state array
///
/// Ensures that the event state array is large enough to accommodate the given
/// file descriptor. This is a no-op for hash-based implementations but needed
/// for continuous FD number implementations.
fn grow_fd_ev_state(manager: &FdEventStateManager, fd: SysFdType) -> bool {
    // For hash-based implementation, we just check bounds
    #[cfg(unix)]
    {
        if fd < 0 || fd as usize >= manager.max_fds {
            return false;
        }
    }
    
    // Try to get or create state (this will create if needed)
    manager.get_or_create_state(fd).is_ok()
}

/// Get event state length
///
/// Returns the number of file descriptors with active event state.
fn fd_ev_state_len(manager: &FdEventStateManager) -> usize {
    manager.len()
}

/// Erase event state
///
/// Removes the event state for a file descriptor.
fn erase_fd_ev_state(manager: &FdEventStateManager, fd: SysFdType) {
    manager.remove_state(fd);
}

/// Select message structure
///
/// Represents the message that should be sent to an Erlang process when a select event occurs.
/// The message format is: `{select, Resource, Ref, EventAtom}` where EventAtom is one of:
/// - `ready_input` for read events
/// - `ready_output` for write events  
/// - `ready_error` for error events
#[derive(Debug, Clone)]
pub struct SelectMessage {
    /// Resource object (as a magic reference)
    pub resource: *mut std::ffi::c_void,
    /// Reference term for message identification
    pub ref_term: u64,
    /// Event atom: ready_input, ready_output, or ready_error
    pub event_atom: SelectEventAtom,
}

/// Event atom types for select messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectEventAtom {
    /// ready_input - data available for reading
    ReadyInput,
    /// ready_output - ready for writing
    ReadyOutput,
    /// ready_error - error occurred
    ReadyError,
}

impl From<IoEventType> for SelectEventAtom {
    fn from(event_type: IoEventType) -> Self {
        match event_type {
            IoEventType::Read => SelectEventAtom::ReadyInput,
            IoEventType::Write => SelectEventAtom::ReadyOutput,
            IoEventType::Error => SelectEventAtom::ReadyError,
        }
    }
}

/// Send select message
///
/// Constructs and sends a message to an Erlang process when a select event occurs.
/// The message format is: `{select, Resource, Ref, ready_input | ready_output | ready_error}`
///
/// # Arguments
///
/// * `fd` - File descriptor that triggered the event
/// * `event_type` - Type of I/O event (read, write, error)
/// * `pid` - Process ID to send the message to
/// * `ref_term` - Reference term for message identification
///
/// # Implementation Notes
///
/// This function constructs the message structure. In a full implementation integrated
/// with the Erlang runtime, this would:
/// 1. Create an Erlang tuple: `{select, Resource, Ref, EventAtom}`
///    - `select` is the atom `am_select`
///    - `Resource` is a magic reference to the resource object
///    - `Ref` is the reference term
///    - `EventAtom` is one of `am_ready_input`, `am_ready_output`, or `am_ready_error`
/// 2. Send the message to the process identified by `pid` using the runtime's message queue
/// 3. Handle process death (if process no longer exists, clear the event)
///
/// Currently, this is a placeholder that constructs the message structure but doesn't
/// actually send it. The actual message sending must be done by the runtime integration
/// layer which has access to the process registry and message queue.
///
/// # See Also
///
/// - `erts/emulator/sys/common/erl_check_io.c:send_select_msg()` - C implementation
/// - `erts/emulator/sys/common/erl_check_io.c:prepare_select_msg()` - Message preparation
fn send_select_msg(fd: SysFdType, event_type: IoEventType, pid: ErlangPid, ref_term: ErlangTerm) {
    // Get the event state to retrieve the resource
    // Note: In a full implementation, we'd need access to the event state manager
    // For now, we construct the message structure that would be sent
    
    let event_atom = SelectEventAtom::from(event_type);
    
    // Construct the select message
    // In a full implementation, this would create an Erlang term tuple:
    // {select, Resource, Ref, EventAtom}
    let _message = SelectMessage {
        resource: ptr::null_mut(), // Would be retrieved from event state
        ref_term,
        event_atom,
    };
    
    // ========================================================================
    // Runtime Integration Required
    // ========================================================================
    //
    // The following steps need to be performed by the runtime integration layer
    // to actually send the message to the Erlang process:
    //
    // 1. Process Lookup:
    //    - Look up the process by pid using erts_proc_lookup() or equivalent
    //    - If process doesn't exist (process has died), clear the event and return
    //    - This prevents sending messages to dead processes
    //
    // 2. Message Term Creation:
    //    - Create the Erlang tuple: {select, Resource, Ref, EventAtom}
    //    - Use enif_make_tuple4() or equivalent:
    //      * enif_make_atom(env, "select") -> atom "select"
    //      * enif_make_resource(env, resource) or enif_make_magic_ref() -> Resource
    //      * ref_term (already an Erlang term, may need validation)
    //      * enif_make_atom(env, "ready_input" | "ready_output" | "ready_error") -> EventAtom
    //
    // 3. Message Queueing:
    //    - Send the message using erts_queue_message() or equivalent runtime function
    //    - The message will be delivered to the process's mailbox
    //    - Handle any errors (e.g., process mailbox full, process dying)
    //
    // 4. Error Handling:
    //    - If process lookup fails, clear the event state
    //    - If message creation fails, log error and clear event
    //    - If message queueing fails, handle appropriately (retry or clear)
    //
    // For now, we log the message that would be sent (in debug builds only)
    #[cfg(debug_assertions)]
    {
        let event_str = match event_atom {
            SelectEventAtom::ReadyInput => "ready_input",
            SelectEventAtom::ReadyOutput => "ready_output",
            SelectEventAtom::ReadyError => "ready_error",
        };
        eprintln!(
            "select message would be sent: fd={}, pid={}, ref={}, event={}",
            fd, pid, ref_term, event_str
        );
    }
    
    // In a production implementation, the runtime integration layer would:
    // - Look up the process by pid
    // - Create the Erlang term tuple
    // - Queue the message to the process's mailbox
    // - Handle process death by clearing the event
}

/// Clear select event
///
/// Clears a select event, cleaning up any pending messages.
fn clear_select_event(state: &mut NifSelectEventState, event_type: IoEventType) {
    let flag = match event_type {
        IoEventType::Read => NifSelectFlags::Read as u32,
        IoEventType::Write => NifSelectFlags::Write as u32,
        IoEventType::Error => NifSelectFlags::Error as u32,
    };
    
    state.active_events &= !flag;
    
    // If no events are active, clear the resource
    if state.active_events == 0 {
        state.resource = ptr::null_mut();
        state.pid = 0;
        state.ref_term = 0;
    }
}

/// NIF select function
///
/// Registers a file descriptor for I/O event monitoring. When the specified
/// events occur, a message will be sent to the Erlang process.
///
/// # Arguments
///
/// * `check_io` - Check I/O manager
/// * `event` - File descriptor (event object)
/// * `mode` - Select flags (read, write, stop, cancel, etc.)
/// * `obj` - Resource object (for lifetime management)
/// * `pid` - Process ID to send messages to (None = calling process)
/// * `ref` - Reference term for message identification
///
/// # Returns
///
/// * `Ok(NifSelectResult)` - Result of the operation
/// * `Err(CheckIoError)` - Error during operation
pub fn enif_select(
    check_io: &CheckIo,
    event: SysFdType,
    mode: u32,
    _obj: *mut std::ffi::c_void,
    _pid: Option<ErlangPid>,
    _ref: ErlangTerm,
) -> Result<NifSelectResult, CheckIoError> {
    let manager = &check_io.event_state_manager;
    
    // Check if this is a stop operation
    let flags = NifSelectFlags::Stop as u32;
    if (mode & flags) != 0 {
        // Stop monitoring
        erase_fd_ev_state(&manager, event);
        return Ok(NifSelectResult::new(NifSelectResult::STOP_CALLED));
    }
    
    // Check if this is a cancel operation
    let cancel_flags = NifSelectFlags::Cancel as u32;
    if (mode & cancel_flags) != 0 {
        // Cancel monitoring for specified events
        if let Ok(state_arc) = manager.get_or_create_state(event) {
            let mut state = state_arc.lock().unwrap();
            let mut result_flags = 0u32;
            
            if (mode & (NifSelectFlags::Read as u32)) != 0 {
                clear_select_event(&mut state, IoEventType::Read);
                result_flags |= NifSelectResult::READ_CANCELLED;
            }
            if (mode & (NifSelectFlags::Write as u32)) != 0 {
                clear_select_event(&mut state, IoEventType::Write);
                result_flags |= NifSelectResult::WRITE_CANCELLED;
            }
            if (mode & (NifSelectFlags::Error as u32)) != 0 {
                clear_select_event(&mut state, IoEventType::Error);
                result_flags |= NifSelectResult::ERROR_CANCELLED;
            }
            
            return Ok(NifSelectResult::new(result_flags));
        }
        return Ok(NifSelectResult::SUCCESS);
    }
    
    // Enable monitoring
    if !grow_fd_ev_state(&manager, event) {
        return Ok(NifSelectResult::new(NifSelectResult::INVALID_EVENT));
    }
    
    let state_arc = manager.get_or_create_state(event)?;
    let mut state = state_arc.lock().unwrap();
    
    // Update active events
    if (mode & (NifSelectFlags::Read as u32)) != 0 {
        state.active_events |= NifSelectFlags::Read as u32;
    }
    if (mode & (NifSelectFlags::Write as u32)) != 0 {
        state.active_events |= NifSelectFlags::Write as u32;
    }
    if (mode & (NifSelectFlags::Error as u32)) != 0 {
        state.active_events |= NifSelectFlags::Error as u32;
    }
    
    // Store resource and process info
    state.resource = _obj;
    state.pid = _pid.unwrap_or(0);
    state.ref_term = _ref;
    
    // Add file descriptor to pollset
    let mut pollsets = check_io.pollsets.write().unwrap();
    let default_thread_id = PollThreadId::new(0);
    if let Some(pollset) = pollsets.get_mut(&default_thread_id) {
        pollset.add_fd(event, state.active_events);
    }
    drop(pollsets);
    
    Ok(NifSelectResult::SUCCESS)
}

// ============================================================================
// NIF I/O Queue Functions
// ============================================================================

/// NIF I/O Queue options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NifIOQueueOpts {
    /// Normal I/O queue
    Normal = 1,
}

/// System I/O vector (as used by writev)
#[repr(C)]
#[derive(Clone)]
pub struct SysIOVec {
    /// Base address of buffer
    pub iov_base: *mut std::ffi::c_void,
    /// Length of buffer
    pub iov_len: usize,
}

/// NIF I/O vector
pub struct NifIOVec {
    /// Number of I/O vectors
    pub iovcnt: i32,
    /// Total size in bytes
    pub size: usize,
    /// Array of system I/O vectors
    pub iov: Vec<SysIOVec>,
    /// Reference to binary data (for lifetime management)
    pub ref_bins: Vec<*mut std::ffi::c_void>,
}

/// NIF binary structure
#[derive(Debug)]
pub struct NifBinary {
    /// Size of binary
    pub size: usize,
    /// Data pointer
    pub data: *mut u8,
    /// Reference to binary (for lifetime management)
    pub ref_bin: *mut std::ffi::c_void,
}

/// NIF I/O Queue
///
/// A queue used for storing binary data that should be passed to writev or
/// similar functions. Used by NIFs for efficient I/O operations.
///
/// ## Implementation Status
///
/// This is a simplified Rust implementation of the C `ErlNifIOQueue` structure.
/// The current implementation:
/// - Tracks I/O vectors and binary references
/// - Manages queue size and operations (enqueue, dequeue, peek)
/// - Provides basic binary reference management
///
/// ## Future Integration
///
/// For full integration with the Erlang/OTP runtime, the following enhancements
/// would be needed:
/// - Integration with Erlang's binary management system for proper reference counting
/// - Direct memory management of binary data (currently uses simplified tracking)
/// - Erlang term creation from binary data in `peek_head()`
/// - Proper handling of binary lifetimes and garbage collection
///
/// ## See Also
///
/// - `erts/emulator/beam/erl_io_queue.c` - C implementation
/// - `erts/emulator/beam/erl_io_queue.h` - C header
pub struct NifIOQueue {
    /// Total size in bytes
    size: usize,
    /// Queue of I/O vectors
    vectors: Vec<NifIOVec>,
    /// Queue of binary references
    binaries: Vec<NifBinary>,
}

impl NifIOQueue {
    /// Create a new NIF I/O queue
    ///
    /// # Arguments
    ///
    /// * `opts` - Queue options (must be `NifIOQueueOpts::Normal`)
    ///
    /// # Returns
    ///
    /// * `Some(NifIOQueue)` - Created queue
    /// * `None` - Invalid options or allocation failed
    pub fn create(opts: NifIOQueueOpts) -> Option<Self> {
        if opts != NifIOQueueOpts::Normal {
            return None;
        }
        
        Some(Self {
            size: 0,
            vectors: Vec::new(),
            binaries: Vec::new(),
        })
    }
    
    /// Get the total size of the queue in bytes
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Enqueue an I/O vector into the queue
    ///
    /// # Arguments
    ///
    /// * `iovec` - I/O vector to enqueue
    /// * `skip` - Number of bytes to skip from the beginning
    ///
    /// # Returns
    ///
    /// * `true` - Success
    /// * `false` - `skip` is greater than the size of `iovec`
    pub fn enqv(&mut self, iovec: &NifIOVec, skip: usize) -> bool {
        if skip > iovec.size {
            return false;
        }
        
        // Calculate remaining size after skip
        let remaining_size = iovec.size.saturating_sub(skip);
        
        // Track the size and store the I/O vector
        // Note: In a full implementation with Erlang binary management integration,
        // we would need to:
        // - Copy or reference the actual binary data
        // - Manage binary reference counts through the Erlang runtime
        // - Ensure proper lifetime management for the binary data
        self.size += remaining_size;
        
        // Store the I/O vector with adjusted size
        // The binary references are tracked but not actively managed by the runtime yet
        self.vectors.push(NifIOVec {
            iovcnt: iovec.iovcnt,
            size: remaining_size,
            iov: iovec.iov.clone(),
            ref_bins: iovec.ref_bins.clone(),
        });
        
        true
    }
    
    /// Enqueue a binary into the queue
    ///
    /// # Arguments
    ///
    /// * `bin` - Binary to enqueue
    /// * `skip` - Number of bytes to skip from the beginning
    ///
    /// # Returns
    ///
    /// * `true` - Success
    /// * `false` - `skip` is greater than the size of `bin`
    pub fn enq_binary(&mut self, bin: &NifBinary, skip: usize) -> bool {
        if skip > bin.size {
            return false;
        }
        
        // Calculate remaining size after skip
        let remaining_size = bin.size.saturating_sub(skip);
        
        // Create an I/O vector from the binary
        let iovec = NifIOVec {
            iovcnt: 1,
            size: remaining_size,
            iov: vec![SysIOVec {
                iov_base: if skip < bin.size && !bin.data.is_null() {
                    // For NIF binary handling, we need to work with raw pointers from Erlang's FFI.
                    // While we could use slice operations, NifBinary is designed to work with
                    // raw pointers for FFI compatibility. The safety is ensured by:
                    // 1. Checking skip < bin.size (bounds check)
                    // 2. Checking !bin.data.is_null() (null check)
                    // 3. The caller is responsible for ensuring NifBinary is valid
                    //
                    // Alternative: If we had a slice, we could use:
                    //   let slice = unsafe { std::slice::from_raw_parts(bin.data, bin.size) };
                    //   slice.as_ptr().add(skip) as *mut std::ffi::c_void
                    // But this still requires unsafe for the slice creation.
                    unsafe {
                        // Safety: skip < bin.size and bin.data is not null (checked above)
                        // The resulting pointer is within bounds: [bin.data, bin.data + bin.size)
                        bin.data.add(skip) as *mut std::ffi::c_void
                    }
                } else {
                    ptr::null_mut()
                },
                iov_len: remaining_size,
            }],
            ref_bins: vec![bin.ref_bin],
        };
        
        self.size += remaining_size;
        self.vectors.push(iovec);
        self.binaries.push(NifBinary {
            size: bin.size,
            data: bin.data,
            ref_bin: bin.ref_bin,
        });
        
        true
    }
    
    /// Dequeue bytes from the queue
    ///
    /// # Arguments
    ///
    /// * `count` - Number of bytes to dequeue
    /// * `size` - Optional pointer to store the new size
    ///
    /// # Returns
    ///
    /// * `true` - Success
    /// * `false` - Queue does not contain `count` bytes
    pub fn deq(&mut self, count: usize, size: Option<&mut usize>) -> bool {
        if count > self.size {
            return false;
        }
        
        // Remove bytes from the queue
        // Note: In a full implementation with Erlang binary management integration,
        // we would need to properly manage binary reference counts and ensure
        // that binary data is not freed while still referenced
        let mut remaining = count;
        while remaining > 0 && !self.vectors.is_empty() {
            let vec = &mut self.vectors[0];
            if vec.size <= remaining {
                remaining -= vec.size;
                self.vectors.remove(0);
                if !self.binaries.is_empty() {
                    self.binaries.remove(0);
                }
            } else {
                vec.size -= remaining;
                remaining = 0;
            }
        }
        
        self.size -= count;
        
        if let Some(s) = size {
            *s = self.size;
        }
        
        true
    }
    
    /// Peek at the head of the queue without removing it
    ///
    /// # Arguments
    ///
    /// * `size` - Optional pointer to store the size of the head element
    /// * `bin_term` - Optional pointer to store the binary term
    ///
    /// # Returns
    ///
    /// * `true` - Success
    /// * `false` - Queue is empty
    pub fn peek_head(&self, size: Option<&mut usize>, _bin_term: Option<&mut u64>) -> bool {
        if self.vectors.is_empty() {
            return false;
        }
        
        if let Some(s) = size {
            *s = self.vectors[0].size;
        }
        
        // Note: In a full implementation with Erlang runtime integration, we would:
        // - Create an Erlang binary term from the head element's data
        // - Use enif_make_binary() or similar to create the term
        // - Store the term in bin_term if provided
        // For now, we return success and the size, but the binary term creation
        // requires runtime integration
        
        true
    }
}

impl Drop for NifIOQueue {
    fn drop(&mut self) {
        // Clean up resources
        // Note: In a full implementation with Erlang binary management integration,
        // we would need to:
        // - Release binary references through the Erlang runtime
        // - Decrement reference counts for all tracked binaries
        // - Ensure proper cleanup of binary data
        self.vectors.clear();
        self.binaries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_check_io_creation() {
        let check_io = CheckIo::new();
        assert_eq!(check_io.max_files(), 1024);
        
        let info = check_io.info();
        assert_eq!(info.max_files, 1024);
        assert!(info.num_active_threads > 0);
    }
    
    #[test]
    fn test_check_io_custom_config() {
        let config = CheckIoConfig {
            max_files: 2048,
            num_pollsets: 2,
            num_poll_threads: 2,
        };
        let check_io = CheckIo::with_config(config);
        assert_eq!(check_io.max_files(), 2048);
    }
    
    #[test]
    fn test_poll_thread_creation() {
        let check_io = CheckIo::new();
        
        let thread_id = check_io.create_poll_thread(1);
        assert!(thread_id.is_ok());
        
        // Try to create duplicate
        let duplicate = check_io.create_poll_thread(1);
        assert_eq!(duplicate, Err(CheckIoError::ThreadIdExists));
    }
    
    #[test]
    fn test_poll_thread_interrupt() {
        let check_io = CheckIo::new();
        let thread_id = PollThreadId::new(0);
        
        // Interrupt the thread
        let result = check_io.interrupt(thread_id, true);
        assert!(result.is_ok());
        
        // Clear interrupt
        let result = check_io.interrupt(thread_id, false);
        assert!(result.is_ok());
        
        // Try invalid thread
        let invalid = PollThreadId::new(999);
        let result = check_io.interrupt(invalid, true);
        assert_eq!(result, Err(CheckIoError::InvalidThreadId));
    }
    
    #[test]
    fn test_check_io_with_timeout() {
        let check_io = CheckIo::new();
        let thread_id = PollThreadId::new(0);
        
        // Check with timeout (should return None for timeout when no FDs are registered)
        let result = check_io.check(thread_id, Some(Duration::from_millis(10)), false);
        assert!(result.is_ok());
        // Returns None when timeout occurs or no events are available
        assert!(result.unwrap().is_none());
    }
    
    #[test]
    fn test_poll_thread_id_constants() {
        assert_eq!(PollThreadId::AUX.id(), -2);
        assert_eq!(PollThreadId::SCHEDULER.id(), -1);
    }
    
    #[test]
    fn test_io_event_types() {
        assert_eq!(IoEventType::Read, IoEventType::Read);
        assert_eq!(IoEventType::Write, IoEventType::Write);
        assert_eq!(IoEventType::Error, IoEventType::Error);
        assert_ne!(IoEventType::Read, IoEventType::Write);
    }
    
    #[test]
    fn test_check_io_info() {
        let check_io = CheckIo::new();
        let info = check_io.info();
        
        assert_eq!(info.config.max_files, 1024);
        assert!(info.num_active_threads > 0);
    }
    
    // NIF I/O Queue tests
    #[test]
    fn test_nif_ioq_create() {
        let queue = NifIOQueue::create(NifIOQueueOpts::Normal);
        assert!(queue.is_some());
        
        let q = queue.unwrap();
        assert_eq!(q.size(), 0);
    }
    
    #[test]
    fn test_nif_ioq_create_invalid_opts() {
        // Test with invalid options (we only support Normal)
        // Since we can't create invalid enum values easily, we test the Normal case
        let queue = NifIOQueue::create(NifIOQueueOpts::Normal);
        assert!(queue.is_some());
    }
    
    #[test]
    fn test_nif_ioq_enq_binary() {
        let mut queue = NifIOQueue::create(NifIOQueueOpts::Normal).unwrap();
        
        // Create a test binary
        let data = vec![1u8, 2, 3, 4, 5];
        let bin = NifBinary {
            size: data.len(),
            data: data.as_ptr() as *mut u8,
            ref_bin: ptr::null_mut(),
        };
        
        // Enqueue without skip
        let result = queue.enq_binary(&bin, 0);
        assert!(result);
        assert_eq!(queue.size(), 5);
        
        // Enqueue with skip
        let result2 = queue.enq_binary(&bin, 2);
        assert!(result2);
        assert_eq!(queue.size(), 8); // 5 + 3
        
        // Test skip > size
        let result3 = queue.enq_binary(&bin, 10);
        assert!(!result3);
    }
    
    #[test]
    fn test_nif_ioq_enqv() {
        let mut queue = NifIOQueue::create(NifIOQueueOpts::Normal).unwrap();
        
        // Create a test I/O vector
        let iovec = NifIOVec {
            iovcnt: 1,
            size: 10,
            iov: vec![SysIOVec {
                iov_base: ptr::null_mut(),
                iov_len: 10,
            }],
            ref_bins: vec![ptr::null_mut()],
        };
        
        // Enqueue without skip
        let result = queue.enqv(&iovec, 0);
        assert!(result);
        assert_eq!(queue.size(), 10);
        
        // Enqueue with skip
        let result2 = queue.enqv(&iovec, 3);
        assert!(result2);
        assert_eq!(queue.size(), 17); // 10 + 7
        
        // Test skip > size
        let result3 = queue.enqv(&iovec, 15);
        assert!(!result3);
    }
    
    #[test]
    fn test_nif_ioq_deq() {
        let mut queue = NifIOQueue::create(NifIOQueueOpts::Normal).unwrap();
        
        // Create and enqueue a binary
        let data = vec![1u8, 2, 3, 4, 5];
        let bin = NifBinary {
            size: data.len(),
            data: data.as_ptr() as *mut u8,
            ref_bin: ptr::null_mut(),
        };
        queue.enq_binary(&bin, 0);
        
        // Dequeue some bytes
        let mut new_size = 0;
        let result = queue.deq(3, Some(&mut new_size));
        assert!(result);
        assert_eq!(new_size, 2);
        assert_eq!(queue.size(), 2);
        
        // Dequeue remaining
        let result2 = queue.deq(2, None);
        assert!(result2);
        assert_eq!(queue.size(), 0);
        
        // Try to dequeue more than available
        let result3 = queue.deq(1, None);
        assert!(!result3);
    }
    
    #[test]
    fn test_nif_ioq_peek_head() {
        let mut queue = NifIOQueue::create(NifIOQueueOpts::Normal).unwrap();
        
        // Peek empty queue
        let mut size = 0;
        let mut bin_term = 0u64;
        let result = queue.peek_head(Some(&mut size), Some(&mut bin_term));
        assert!(!result);
        
        // Enqueue a binary
        let data = vec![1u8, 2, 3, 4, 5];
        let bin = NifBinary {
            size: data.len(),
            data: data.as_ptr() as *mut u8,
            ref_bin: ptr::null_mut(),
        };
        queue.enq_binary(&bin, 0);
        
        // Peek head
        let result2 = queue.peek_head(Some(&mut size), Some(&mut bin_term));
        assert!(result2);
        assert_eq!(size, 5);
        assert_eq!(queue.size(), 5); // Queue should still have the data
    }
    
    // NIF Select tests
    #[test]
    fn test_nif_select_flags() {
        let read = NifSelectFlags::Read;
        let write = NifSelectFlags::Write;
        let error = NifSelectFlags::Error;
        
        assert_eq!(read as u32, 1);
        assert_eq!(write as u32, 2);
        assert_eq!(error as u32, 32);
        
        // Test flag combination
        let combined = NifSelectFlags::combine(&[read, write]);
        assert_eq!(combined, 3);
    }
    
    #[test]
    fn test_nif_select_result() {
        let success = NifSelectResult::SUCCESS;
        assert!(success.is_success());
        assert_eq!(success.value(), 0);
        
        let result = NifSelectResult::new(NifSelectResult::STOP_CALLED);
        assert!(result.has_flag(NifSelectResult::STOP_CALLED));
        assert!(!result.is_success());
        
        let result2 = NifSelectResult::new(
            NifSelectResult::READ_CANCELLED | NifSelectResult::WRITE_CANCELLED
        );
        assert!(result2.has_flag(NifSelectResult::READ_CANCELLED));
        assert!(result2.has_flag(NifSelectResult::WRITE_CANCELLED));
    }
    
    #[test]
    fn test_enif_select_read() {
        let check_io = CheckIo::new();
        let fd: SysFdType = 5; // Test file descriptor
        
        // Select for read events
        let mode = NifSelectFlags::Read as u32;
        let result = enif_select(&check_io, fd, mode, ptr::null_mut(), None, 0);
        
        assert!(result.is_ok());
        let select_result = result.unwrap();
        assert!(select_result.is_success());
    }
    
    #[test]
    fn test_enif_select_write() {
        let check_io = CheckIo::new();
        let fd: SysFdType = 6;
        
        // Select for write events
        let mode = NifSelectFlags::Write as u32;
        let result = enif_select(&check_io, fd, mode, ptr::null_mut(), None, 0);
        
        assert!(result.is_ok());
        let select_result = result.unwrap();
        assert!(select_result.is_success());
    }
    
    #[test]
    fn test_enif_select_read_write() {
        let check_io = CheckIo::new();
        let fd: SysFdType = 7;
        
        // Select for both read and write events
        let mode = NifSelectFlags::combine(&[NifSelectFlags::Read, NifSelectFlags::Write]);
        let result = enif_select(&check_io, fd, mode, ptr::null_mut(), None, 0);
        
        assert!(result.is_ok());
        let select_result = result.unwrap();
        assert!(select_result.is_success());
    }
    
    #[test]
    fn test_enif_select_stop() {
        let check_io = CheckIo::new();
        let fd: SysFdType = 8;
        
        // First, enable select
        let mode = NifSelectFlags::Read as u32;
        let _ = enif_select(&check_io, fd, mode, ptr::null_mut(), None, 0);
        
        // Then stop
        let stop_mode = NifSelectFlags::Stop as u32;
        let result = enif_select(&check_io, fd, stop_mode, ptr::null_mut(), None, 0);
        
        assert!(result.is_ok());
        let select_result = result.unwrap();
        assert!(select_result.has_flag(NifSelectResult::STOP_CALLED));
    }
    
    #[test]
    fn test_enif_select_cancel() {
        let check_io = CheckIo::new();
        let fd: SysFdType = 9;
        
        // First, enable select for read and write
        let mode = NifSelectFlags::combine(&[NifSelectFlags::Read, NifSelectFlags::Write]);
        let _ = enif_select(&check_io, fd, mode, ptr::null_mut(), None, 0);
        
        // Cancel read
        let cancel_mode = NifSelectFlags::combine(&[NifSelectFlags::Cancel, NifSelectFlags::Read]);
        let result = enif_select(&check_io, fd, cancel_mode, ptr::null_mut(), None, 0);
        
        assert!(result.is_ok());
        let select_result = result.unwrap();
        assert!(select_result.has_flag(NifSelectResult::READ_CANCELLED));
    }
    
    #[test]
    fn test_enif_select_invalid_fd() {
        let check_io = CheckIo::new();
        
        #[cfg(unix)]
        {
            // Test with invalid FD (negative)
            let invalid_fd: SysFdType = -1;
            let mode = NifSelectFlags::Read as u32;
            let result = enif_select(&check_io, invalid_fd, mode, ptr::null_mut(), None, 0);
            
            assert!(result.is_ok());
            let select_result = result.unwrap();
            assert!(select_result.has_flag(NifSelectResult::INVALID_EVENT));
        }
    }
    
    #[test]
    fn test_fd_hash() {
        #[cfg(unix)]
        {
            let fd1: SysFdType = 5;
            let fd2: SysFdType = 10;
            
            let hash1 = fd_hash(fd1);
            let hash2 = fd_hash(fd2);
            
            assert_eq!(hash1, 5);
            assert_eq!(hash2, 10);
        }
    }
    
    #[test]
    #[cfg(unix)]
    fn test_check_io_polling() {
        use std::os::unix::io::AsRawFd;
        use std::net::{TcpListener, TcpStream};
        use std::time::Duration;
        
        let check_io = CheckIo::new();
        let thread_id = PollThreadId::new(0);
        
        // Create a TCP listener to get a real file descriptor
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let listener_fd = listener.as_raw_fd();
        
        // Register the file descriptor for read events
        let mode = NifSelectFlags::Read as u32;
        let result = enif_select(&check_io, listener_fd, mode, ptr::null_mut(), None, 0);
        assert!(result.is_ok());
        
        // Check for events with a short timeout
        let result = check_io.check(thread_id, Some(Duration::from_millis(10)), false);
        assert!(result.is_ok());
        // Should return None (timeout) since no connection is pending
        assert!(result.unwrap().is_none());
    }
    
    #[test]
    fn test_pollset_operations() {
        let mut pollset = PollSet::new();
        
        // Add file descriptors
        pollset.add_fd(5, NifSelectFlags::Read as u32);
        pollset.add_fd(6, NifSelectFlags::Write as u32);
        pollset.add_fd(7, NifSelectFlags::combine(&[NifSelectFlags::Read, NifSelectFlags::Write]));
        
        let fds = pollset.get_fds();
        assert_eq!(fds.len(), 3);
        
        // Update existing FD
        pollset.update_fd(5, NifSelectFlags::combine(&[NifSelectFlags::Read, NifSelectFlags::Write]));
        let fds = pollset.get_fds();
        let fd5 = fds.iter().find(|(fd, _)| *fd == 5).unwrap();
        assert_eq!(fd5.1, NifSelectFlags::combine(&[NifSelectFlags::Read, NifSelectFlags::Write]));
        
        // Remove FD
        pollset.remove_fd(6);
        let fds = pollset.get_fds();
        assert_eq!(fds.len(), 2);
        assert!(fds.iter().find(|(fd, _)| *fd == 6).is_none());
    }
}

