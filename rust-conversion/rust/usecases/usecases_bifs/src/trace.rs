//! Trace BIF Module
//!
//! Provides tracing built-in functions.
//! Based on erl_bif_trace.c
//!
//! This module provides a safe Rust API for tracing operations:
//! - Trace session management
//! - Process/port tracing
//! - Sequential tracing
//! - System monitoring
//! - Trace info queries

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::LazyLock;

/// Trace BIF operations
pub struct TraceBif;

/// Trace session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceSessionId(u64);

/// Trace target (process or port)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceTarget {
    /// Process ID
    Process(u64),
    /// Port ID
    Port(u64),
    /// All processes
    AllProcesses,
    /// All ports
    AllPorts,
}

/// Trace flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceFlags {
    /// Enable call tracing
    pub call: bool,
    /// Enable return tracing
    pub return_trace: bool,
    /// Enable send tracing
    pub send: bool,
    /// Enable receive tracing
    pub receive: bool,
    /// Enable garbage collection tracing
    pub garbage_collection: bool,
    /// Enable timestamp
    pub timestamp: bool,
    /// Enable CPU timestamp
    pub cpu_timestamp: bool,
}

impl Default for TraceFlags {
    fn default() -> Self {
        Self {
            call: false,
            return_trace: false,
            send: false,
            receive: false,
            garbage_collection: false,
            timestamp: false,
            cpu_timestamp: false,
        }
    }
}

/// Sequential trace flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqTraceFlags {
    /// Enable send tracing
    pub send: bool,
    /// Enable receive tracing
    pub receive: bool,
    /// Enable print tracing
    pub print: bool,
    /// Enable timestamp
    pub timestamp: bool,
    /// Enable strict monotonic timestamp
    pub strict_monotonic_timestamp: bool,
    /// Enable monotonic timestamp
    pub monotonic_timestamp: bool,
}

impl Default for SeqTraceFlags {
    fn default() -> Self {
        Self {
            send: false,
            receive: false,
            print: false,
            timestamp: false,
            strict_monotonic_timestamp: false,
            monotonic_timestamp: false,
        }
    }
}

/// System monitor configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMonitorConfig {
    /// Long message queue threshold
    pub long_msgq_off: Option<u64>,
    /// Long message queue on threshold
    pub long_msgq_on: Option<u64>,
    /// Large heap threshold
    pub large_heap_off: Option<u64>,
    /// Large heap on threshold
    pub large_heap_on: Option<u64>,
    /// Busy port threshold
    pub busy_port_off: Option<u64>,
    /// Busy port on threshold
    pub busy_port_on: Option<u64>,
    /// Busy dist port threshold
    pub busy_dist_port_off: Option<u64>,
    /// Busy dist port on threshold
    pub busy_dist_port_on: Option<u64>,
}

impl Default for SystemMonitorConfig {
    fn default() -> Self {
        Self {
            long_msgq_off: None,
            long_msgq_on: None,
            large_heap_off: None,
            large_heap_on: None,
            busy_port_off: None,
            busy_port_on: None,
            busy_dist_port_off: None,
            busy_dist_port_on: None,
        }
    }
}

/// Trace info result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceInfo {
    /// Process trace info
    Process {
        /// Process ID
        pid: u64,
        /// Trace flags
        flags: TraceFlags,
    },
    /// Port trace info
    Port {
        /// Port ID
        port: u64,
        /// Trace flags
        flags: TraceFlags,
    },
    /// Function trace info
    Function {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// Arity
        arity: u32,
        /// Trace flags
        flags: TraceFlags,
    },
    /// Sequential trace info
    Sequential {
        /// Sequential trace flags
        flags: SeqTraceFlags,
        /// Label
        label: u64,
    },
    /// System monitor info
    SystemMonitor {
        /// Configuration
        config: SystemMonitorConfig,
    },
    /// Not traced
    NotTraced,
    /// Does not exist
    DoesNotExist,
}

/// Trace operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceError {
    /// Invalid session
    InvalidSession,
    /// Invalid target
    InvalidTarget,
    /// Invalid flags
    InvalidFlags,
    /// Operation not supported
    NotSupported,
    /// Internal error
    InternalError,
}

/// Internal trace state
#[derive(Debug)]
struct TraceState {
    /// Next session ID
    next_session_id: u64,
    /// Active trace sessions
    sessions: HashMap<TraceSessionId, TraceSession>,
    /// Process traces
    process_traces: HashMap<u64, TraceFlags>,
    /// Port traces
    port_traces: HashMap<u64, TraceFlags>,
    /// Sequential trace flags
    seq_trace_flags: SeqTraceFlags,
    /// System monitor config
    system_monitor: SystemMonitorConfig,
}

/// Trace session
#[derive(Debug, Clone)]
struct TraceSession {
    /// Session ID
    id: TraceSessionId,
    /// Session name
    name: String,
    /// Trace flags
    flags: TraceFlags,
}

impl TraceState {
    fn new() -> Self {
        Self {
            next_session_id: 1,
            sessions: HashMap::new(),
            process_traces: HashMap::new(),
            port_traces: HashMap::new(),
            seq_trace_flags: SeqTraceFlags::default(),
            system_monitor: SystemMonitorConfig::default(),
        }
    }
}

/// Global trace state (thread-safe)
static TRACE_STATE: LazyLock<Mutex<TraceState>> = LazyLock::new(|| {
    Mutex::new(TraceState::new())
});

impl TraceBif {
    /// Create a new trace session
    ///
    /// # Arguments
    /// * `name` - Session name
    ///
    /// # Returns
    /// Session ID on success
    pub fn create_session(name: String) -> Result<TraceSessionId, TraceError> {
        let mut state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        let id = TraceSessionId(state.next_session_id);
        state.next_session_id += 1;
        
        let session = TraceSession {
            id,
            name,
            flags: TraceFlags::default(),
        };
        
        state.sessions.insert(id, session);
        Ok(id)
    }

    /// Enable or disable tracing for a target
    ///
    /// # Arguments
    /// * `session_id` - Trace session ID (None for default session)
    /// * `target` - Target to trace
    /// * `enable` - Enable or disable tracing
    /// * `flags` - Trace flags
    ///
    /// # Returns
    /// Number of processes/ports traced
    pub fn trace(
        session_id: Option<TraceSessionId>,
        target: TraceTarget,
        enable: bool,
        flags: TraceFlags,
    ) -> Result<u32, TraceError> {
        let mut state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        // Validate session if provided
        if let Some(id) = session_id {
            if !state.sessions.contains_key(&id) {
                return Err(TraceError::InvalidSession);
            }
        }
        
        let count = match target {
            TraceTarget::Process(pid) => {
                if enable {
                    state.process_traces.insert(pid, flags);
                    1
                } else {
                    state.process_traces.remove(&pid);
                    1
                }
            }
            TraceTarget::Port(port) => {
                if enable {
                    state.port_traces.insert(port, flags);
                    1
                } else {
                    state.port_traces.remove(&port);
                    1
                }
            }
            TraceTarget::AllProcesses => {
                // In a real implementation, this would trace all processes
                // For now, we just return a placeholder count
                if enable {
                    state.process_traces.clear();
                    // Mark all processes as traced (simplified)
                    0
                } else {
                    let count = state.process_traces.len() as u32;
                    state.process_traces.clear();
                    count
                }
            }
            TraceTarget::AllPorts => {
                // In a real implementation, this would trace all ports
                if enable {
                    state.port_traces.clear();
                    0
                } else {
                    let count = state.port_traces.len() as u32;
                    state.port_traces.clear();
                    count
                }
            }
        };
        
        Ok(count)
    }

    /// Get trace info for a target
    ///
    /// # Arguments
    /// * `session_id` - Trace session ID (None for all sessions)
    /// * `target` - Target to query
    ///
    /// # Returns
    /// Trace info
    pub fn trace_info(
        session_id: Option<TraceSessionId>,
        target: TraceTarget,
    ) -> Result<TraceInfo, TraceError> {
        let state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        // Validate session if provided
        if let Some(id) = session_id {
            if !state.sessions.contains_key(&id) {
                return Err(TraceError::InvalidSession);
            }
        }
        
        match target {
            TraceTarget::Process(pid) => {
                if let Some(flags) = state.process_traces.get(&pid) {
                    Ok(TraceInfo::Process {
                        pid,
                        flags: *flags,
                    })
                } else {
                    Ok(TraceInfo::NotTraced)
                }
            }
            TraceTarget::Port(port) => {
                if let Some(flags) = state.port_traces.get(&port) {
                    Ok(TraceInfo::Port {
                        port,
                        flags: *flags,
                    })
                } else {
                    Ok(TraceInfo::NotTraced)
                }
            }
            TraceTarget::AllProcesses | TraceTarget::AllPorts => {
                // Not supported for info queries
                Err(TraceError::InvalidTarget)
            }
        }
    }

    /// Set sequential trace flags
    ///
    /// # Arguments
    /// * `flag` - Flag name ("send", "receive", "print", "timestamp", etc.)
    /// * `enable` - Enable or disable
    ///
    /// # Returns
    /// Previous value of the flag
    pub fn seq_trace(flag: &str, enable: bool) -> Result<bool, TraceError> {
        let mut state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        let previous = match flag {
            "send" => {
                let prev = state.seq_trace_flags.send;
                state.seq_trace_flags.send = enable;
                prev
            }
            "receive" => {
                let prev = state.seq_trace_flags.receive;
                state.seq_trace_flags.receive = enable;
                prev
            }
            "print" => {
                let prev = state.seq_trace_flags.print;
                state.seq_trace_flags.print = enable;
                prev
            }
            "timestamp" => {
                let prev = state.seq_trace_flags.timestamp;
                state.seq_trace_flags.timestamp = enable;
                prev
            }
            "strict_monotonic_timestamp" => {
                let prev = state.seq_trace_flags.strict_monotonic_timestamp;
                state.seq_trace_flags.strict_monotonic_timestamp = enable;
                prev
            }
            "monotonic_timestamp" => {
                let prev = state.seq_trace_flags.monotonic_timestamp;
                state.seq_trace_flags.monotonic_timestamp = enable;
                prev
            }
            _ => return Err(TraceError::InvalidFlags),
        };
        
        Ok(previous)
    }

    /// Get sequential trace info
    ///
    /// # Returns
    /// Sequential trace flags
    pub fn seq_trace_info() -> Result<SeqTraceFlags, TraceError> {
        let state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        Ok(state.seq_trace_flags)
    }

    /// Set system monitor configuration
    ///
    /// # Arguments
    /// * `session_id` - Trace session ID (None for default session)
    /// * `config` - System monitor configuration
    ///
    /// # Returns
    /// Previous configuration
    pub fn system_monitor(
        session_id: Option<TraceSessionId>,
        config: SystemMonitorConfig,
    ) -> Result<SystemMonitorConfig, TraceError> {
        let mut state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        // Validate session if provided
        if let Some(id) = session_id {
            if !state.sessions.contains_key(&id) {
                return Err(TraceError::InvalidSession);
            }
        }
        
        let previous = state.system_monitor.clone();
        state.system_monitor = config;
        Ok(previous)
    }

    /// Get system monitor configuration
    ///
    /// # Arguments
    /// * `session_id` - Trace session ID (None for default session)
    ///
    /// # Returns
    /// System monitor configuration
    pub fn system_monitor_get(
        session_id: Option<TraceSessionId>,
    ) -> Result<SystemMonitorConfig, TraceError> {
        let state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        // Validate session if provided
        if let Some(id) = session_id {
            if !state.sessions.contains_key(&id) {
                return Err(TraceError::InvalidSession);
            }
        }
        
        Ok(state.system_monitor.clone())
    }

    /// Destroy a trace session
    ///
    /// # Arguments
    /// * `session_id` - Trace session ID
    ///
    /// # Returns
    /// Success or error
    pub fn destroy_session(session_id: TraceSessionId) -> Result<(), TraceError> {
        let mut state = TRACE_STATE.lock().map_err(|_| TraceError::InternalError)?;
        
        if state.sessions.remove(&session_id).is_some() {
            Ok(())
        } else {
            Err(TraceError::InvalidSession)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let session_id = TraceBif::create_session("test_session".to_string()).unwrap();
        assert!(session_id.0 >= 1);
        
        let session_id2 = TraceBif::create_session("test_session2".to_string()).unwrap();
        assert!(session_id2.0 > session_id.0);
    }

    #[test]
    fn test_trace_process() {
        let target = TraceTarget::Process(123);
        let flags = TraceFlags {
            call: true,
            return_trace: true,
            ..Default::default()
        };
        
        let count = TraceBif::trace(None, target, true, flags).unwrap();
        assert_eq!(count, 1);
        
        let info = TraceBif::trace_info(None, TraceTarget::Process(123)).unwrap();
        match info {
            TraceInfo::Process { pid, flags: info_flags } => {
                assert_eq!(pid, 123);
                assert!(info_flags.call);
                assert!(info_flags.return_trace);
            }
            _ => panic!("Expected Process trace info"),
        }
    }

    #[test]
    fn test_trace_port() {
        let target = TraceTarget::Port(456);
        let flags = TraceFlags {
            send: true,
            receive: true,
            ..Default::default()
        };
        
        let count = TraceBif::trace(None, target, true, flags).unwrap();
        assert_eq!(count, 1);
        
        let info = TraceBif::trace_info(None, TraceTarget::Port(456)).unwrap();
        match info {
            TraceInfo::Port { port, flags: info_flags } => {
                assert_eq!(port, 456);
                assert!(info_flags.send);
                assert!(info_flags.receive);
            }
            _ => panic!("Expected Port trace info"),
        }
    }

    #[test]
    fn test_trace_disable() {
        let target = TraceTarget::Process(789);
        let flags = TraceFlags {
            call: true,
            ..Default::default()
        };
        
        // Enable tracing
        TraceBif::trace(None, target, true, flags).unwrap();
        
        // Disable tracing
        let count = TraceBif::trace(None, target, false, TraceFlags::default()).unwrap();
        assert_eq!(count, 1);
        
        // Verify it's not traced
        let info = TraceBif::trace_info(None, TraceTarget::Process(789)).unwrap();
        assert_eq!(info, TraceInfo::NotTraced);
    }

    #[test]
    fn test_trace_invalid_session() {
        let invalid_session = TraceSessionId(999);
        let target = TraceTarget::Process(123);
        let flags = TraceFlags::default();
        
        let result = TraceBif::trace(Some(invalid_session), target, true, flags);
        assert_eq!(result, Err(TraceError::InvalidSession));
    }

    #[test]
    fn test_seq_trace() {
        // Get initial state
        let initial_flags = TraceBif::seq_trace_info().unwrap();
        let initial_send = initial_flags.send;
        
        // Enable send flag
        let prev = TraceBif::seq_trace("send", true).unwrap();
        assert_eq!(prev, initial_send);
        
        // Check it's enabled
        let flags = TraceBif::seq_trace_info().unwrap();
        assert!(flags.send);
        
        // Disable send flag
        let prev = TraceBif::seq_trace("send", false).unwrap();
        assert!(prev);
        
        // Check it's disabled
        let flags = TraceBif::seq_trace_info().unwrap();
        assert!(!flags.send);
    }

    #[test]
    fn test_seq_trace_all_flags() {
        // Test all sequential trace flags
        TraceBif::seq_trace("send", true).unwrap();
        TraceBif::seq_trace("receive", true).unwrap();
        TraceBif::seq_trace("print", true).unwrap();
        TraceBif::seq_trace("timestamp", true).unwrap();
        TraceBif::seq_trace("strict_monotonic_timestamp", true).unwrap();
        TraceBif::seq_trace("monotonic_timestamp", true).unwrap();
        
        let flags = TraceBif::seq_trace_info().unwrap();
        assert!(flags.send);
        assert!(flags.receive);
        assert!(flags.print);
        assert!(flags.timestamp);
        assert!(flags.strict_monotonic_timestamp);
        assert!(flags.monotonic_timestamp);
    }

    #[test]
    fn test_seq_trace_invalid_flag() {
        let result = TraceBif::seq_trace("invalid_flag", true);
        assert_eq!(result, Err(TraceError::InvalidFlags));
    }

    #[test]
    fn test_system_monitor() {
        let config = SystemMonitorConfig {
            long_msgq_off: Some(1000),
            long_msgq_on: Some(500),
            ..Default::default()
        };
        
        // Get previous config (may not be default if other tests ran)
        let _prev = TraceBif::system_monitor(None, config.clone()).unwrap();
        
        let retrieved = TraceBif::system_monitor_get(None).unwrap();
        assert_eq!(retrieved.long_msgq_off, Some(1000));
        assert_eq!(retrieved.long_msgq_on, Some(500));
    }

    #[test]
    fn test_system_monitor_with_session() {
        let session_id = TraceBif::create_session("monitor_session".to_string()).unwrap();
        
        let config = SystemMonitorConfig {
            large_heap_off: Some(2000),
            large_heap_on: Some(1000),
            ..Default::default()
        };
        
        // Get previous config (may not be default if other tests ran)
        let _prev = TraceBif::system_monitor(Some(session_id), config.clone()).unwrap();
        
        let retrieved = TraceBif::system_monitor_get(Some(session_id)).unwrap();
        assert_eq!(retrieved.large_heap_off, Some(2000));
        assert_eq!(retrieved.large_heap_on, Some(1000));
    }

    #[test]
    fn test_destroy_session() {
        let session_id = TraceBif::create_session("temp_session".to_string()).unwrap();
        
        // Session should exist
        let target = TraceTarget::Process(123);
        let flags = TraceFlags::default();
        TraceBif::trace(Some(session_id), target, true, flags).unwrap();
        
        // Destroy session
        TraceBif::destroy_session(session_id).unwrap();
        
        // Using destroyed session should fail
        let result = TraceBif::trace(Some(session_id), target, true, flags);
        assert_eq!(result, Err(TraceError::InvalidSession));
    }

    #[test]
    fn test_trace_info_not_traced() {
        let info = TraceBif::trace_info(None, TraceTarget::Process(999)).unwrap();
        assert_eq!(info, TraceInfo::NotTraced);
    }

    #[test]
    fn test_trace_all_processes() {
        // Enable tracing for all processes
        let count = TraceBif::trace(
            None,
            TraceTarget::AllProcesses,
            true,
            TraceFlags::default(),
        ).unwrap();
        // Note: In simplified implementation, this returns 0
        assert_eq!(count, 0);
        
        // Disable tracing for all processes
        let count = TraceBif::trace(
            None,
            TraceTarget::AllProcesses,
            false,
            TraceFlags::default(),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_trace_flags_default() {
        let flags = TraceFlags::default();
        assert!(!flags.call);
        assert!(!flags.return_trace);
        assert!(!flags.send);
        assert!(!flags.receive);
        assert!(!flags.garbage_collection);
        assert!(!flags.timestamp);
        assert!(!flags.cpu_timestamp);
    }

    #[test]
    fn test_seq_trace_flags_default() {
        let flags = SeqTraceFlags::default();
        assert!(!flags.send);
        assert!(!flags.receive);
        assert!(!flags.print);
        assert!(!flags.timestamp);
        assert!(!flags.strict_monotonic_timestamp);
        assert!(!flags.monotonic_timestamp);
    }

    #[test]
    fn test_trace_all_ports() {
        // Enable tracing for all ports
        let count = TraceBif::trace(
            None,
            TraceTarget::AllPorts,
            true,
            TraceFlags::default(),
        ).unwrap();
        assert_eq!(count, 0);
        
        // Disable tracing for all ports
        let count = TraceBif::trace(
            None,
            TraceTarget::AllPorts,
            false,
            TraceFlags::default(),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_trace_info_all_targets_error() {
        // Trace info for AllProcesses should return error
        let result = TraceBif::trace_info(None, TraceTarget::AllProcesses);
        assert_eq!(result, Err(TraceError::InvalidTarget));
        
        // Trace info for AllPorts should return error
        let result = TraceBif::trace_info(None, TraceTarget::AllPorts);
        assert_eq!(result, Err(TraceError::InvalidTarget));
    }

    #[test]
    fn test_trace_with_all_flags() {
        let target = TraceTarget::Process(999);
        let flags = TraceFlags {
            call: true,
            return_trace: true,
            send: true,
            receive: true,
            garbage_collection: true,
            timestamp: true,
            cpu_timestamp: true,
        };
        
        let count = TraceBif::trace(None, target, true, flags).unwrap();
        assert_eq!(count, 1);
        
        let info = TraceBif::trace_info(None, TraceTarget::Process(999)).unwrap();
        match info {
            TraceInfo::Process { flags: info_flags, .. } => {
                assert!(info_flags.call);
                assert!(info_flags.return_trace);
                assert!(info_flags.send);
                assert!(info_flags.receive);
                assert!(info_flags.garbage_collection);
                assert!(info_flags.timestamp);
                assert!(info_flags.cpu_timestamp);
            }
            _ => panic!("Expected Process trace info"),
        }
    }

    #[test]
    fn test_trace_info_with_session() {
        let session_id = TraceBif::create_session("info_session".to_string()).unwrap();
        
        let target = TraceTarget::Process(555);
        let flags = TraceFlags {
            call: true,
            ..Default::default()
        };
        
        TraceBif::trace(Some(session_id), target, true, flags).unwrap();
        
        let info = TraceBif::trace_info(Some(session_id), TraceTarget::Process(555)).unwrap();
        match info {
            TraceInfo::Process { pid, flags: info_flags } => {
                assert_eq!(pid, 555);
                assert!(info_flags.call);
            }
            _ => panic!("Expected Process trace info"),
        }
    }

    #[test]
    fn test_trace_info_invalid_session() {
        let invalid_session = TraceSessionId(9999);
        let result = TraceBif::trace_info(Some(invalid_session), TraceTarget::Process(123));
        assert_eq!(result, Err(TraceError::InvalidSession));
    }

    #[test]
    fn test_system_monitor_invalid_session() {
        let invalid_session = TraceSessionId(9999);
        let config = SystemMonitorConfig::default();
        
        let result = TraceBif::system_monitor(Some(invalid_session), config);
        assert_eq!(result, Err(TraceError::InvalidSession));
        
        let result = TraceBif::system_monitor_get(Some(invalid_session));
        assert_eq!(result, Err(TraceError::InvalidSession));
    }

    #[test]
    fn test_destroy_invalid_session() {
        let invalid_session = TraceSessionId(9999);
        let result = TraceBif::destroy_session(invalid_session);
        assert_eq!(result, Err(TraceError::InvalidSession));
    }

    #[test]
    fn test_trace_multiple_processes() {
        // Trace multiple processes
        let p1 = TraceTarget::Process(1001);
        let p2 = TraceTarget::Process(1002);
        let p3 = TraceTarget::Process(1003);
        
        let flags = TraceFlags { call: true, ..Default::default() };
        
        TraceBif::trace(None, p1, true, flags).unwrap();
        TraceBif::trace(None, p2, true, flags).unwrap();
        TraceBif::trace(None, p3, true, flags).unwrap();
        
        // Verify all are traced
        assert!(matches!(
            TraceBif::trace_info(None, TraceTarget::Process(1001)).unwrap(),
            TraceInfo::Process { .. }
        ));
        assert!(matches!(
            TraceBif::trace_info(None, TraceTarget::Process(1002)).unwrap(),
            TraceInfo::Process { .. }
        ));
        assert!(matches!(
            TraceBif::trace_info(None, TraceTarget::Process(1003)).unwrap(),
            TraceInfo::Process { .. }
        ));
    }

    #[test]
    fn test_trace_multiple_ports() {
        // Trace multiple ports
        let port1 = TraceTarget::Port(2001);
        let port2 = TraceTarget::Port(2002);
        
        let flags = TraceFlags { send: true, receive: true, ..Default::default() };
        
        TraceBif::trace(None, port1, true, flags).unwrap();
        TraceBif::trace(None, port2, true, flags).unwrap();
        
        // Verify all are traced
        assert!(matches!(
            TraceBif::trace_info(None, TraceTarget::Port(2001)).unwrap(),
            TraceInfo::Port { .. }
        ));
        assert!(matches!(
            TraceBif::trace_info(None, TraceTarget::Port(2002)).unwrap(),
            TraceInfo::Port { .. }
        ));
    }
}
