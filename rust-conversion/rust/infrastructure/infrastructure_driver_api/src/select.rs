//! Event Selection
//!
//! Provides event selection functions for driver I/O operations.

use super::types::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::os::unix::io::RawFd;

/// Event selection state
struct SelectState {
    event: DriverEvent,
    flags: DriverSelectFlags,
    enabled: bool,
}

/// Event selection registry
struct SelectRegistry {
    selections: HashMap<(u64, RawFd), SelectState>,
}

impl SelectRegistry {
    fn new() -> Self {
        Self {
            selections: HashMap::new(),
        }
    }

    fn select(&mut self, port: DriverPort, event: DriverEvent, flags: DriverSelectFlags, on: bool) {
        let key = (port.id(), event.fd());
        if on {
            self.selections.insert(key, SelectState {
                event,
                flags,
                enabled: true,
            });
        } else {
            self.selections.remove(&key);
        }
    }

    fn is_selected(&self, port: DriverPort, event: DriverEvent, flag: DriverSelectFlags) -> bool {
        let key = (port.id(), event.fd());
        self.selections.get(&key)
            .map(|state| state.enabled && state.flags.contains(flag))
            .unwrap_or(false)
    }
}

lazy_static::lazy_static! {
    static ref SELECT_REGISTRY: Arc<Mutex<SelectRegistry>> = 
        Arc::new(Mutex::new(SelectRegistry::new()));
}

/// Select on an event (file descriptor)
///
/// Equivalent to C's `driver_select`.
///
/// # Arguments
///
/// * `port` - Driver port
/// * `event` - Event (file descriptor) to select on
/// * `flags` - Selection flags (READ, WRITE, USE)
/// * `on` - Whether to enable (true) or disable (false) selection
///
/// # Returns
///
/// Returns 0 on success, -1 on error.
pub fn driver_select(
    port: DriverPort,
    event: DriverEvent,
    flags: DriverSelectFlags,
    on: bool,
) -> i32 {
    let mut registry = SELECT_REGISTRY.lock().unwrap();
    registry.select(port, event, flags, on);
    0
}

/// Check if an event is selected
pub fn is_event_selected(port: DriverPort, event: DriverEvent, flag: DriverSelectFlags) -> bool {
    let registry = SELECT_REGISTRY.lock().unwrap();
    registry.is_selected(port, event, flag)
}

