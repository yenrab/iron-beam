//! Failure Handling
//!
//! Provides failure reporting functions for drivers.

use super::types::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Failure type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    /// POSIX error (errno)
    Posix(i32),
    /// EOF
    Eof,
    /// Atom (normal termination)
    Atom(&'static str),
}

/// Failure registry
struct FailureRegistry {
    failures: HashMap<u64, FailureType>,
}

impl FailureRegistry {
    fn new() -> Self {
        Self {
            failures: HashMap::new(),
        }
    }

    fn set_failure(&mut self, port: DriverPort, failure: FailureType) {
        self.failures.insert(port.id(), failure);
    }

    fn get_failure(&self, port: DriverPort) -> Option<FailureType> {
        self.failures.get(&port.id()).copied()
    }
}

lazy_static::lazy_static! {
    static ref FAILURE_REGISTRY: Arc<Mutex<FailureRegistry>> = 
        Arc::new(Mutex::new(FailureRegistry::new()));
}

/// Report a POSIX error
///
/// Equivalent to C's `driver_failure_posix`.
pub fn driver_failure_posix(port: DriverPort, errno: i32) {
    let mut registry = FAILURE_REGISTRY.lock().unwrap();
    registry.set_failure(port, FailureType::Posix(errno));
}

/// Report EOF
///
/// Equivalent to C's `driver_failure_eof`.
pub fn driver_failure_eof(port: DriverPort) {
    let mut registry = FAILURE_REGISTRY.lock().unwrap();
    registry.set_failure(port, FailureType::Eof);
}

/// Report an atom failure (normal termination)
///
/// Equivalent to C's `driver_failure_atom`.
pub fn driver_failure_atom(port: DriverPort, atom: &'static str) {
    let mut registry = FAILURE_REGISTRY.lock().unwrap();
    registry.set_failure(port, FailureType::Atom(atom));
}

/// Get the failure for a port
pub fn get_failure(port: DriverPort) -> Option<FailureType> {
    let registry = FAILURE_REGISTRY.lock().unwrap();
    registry.get_failure(port)
}


