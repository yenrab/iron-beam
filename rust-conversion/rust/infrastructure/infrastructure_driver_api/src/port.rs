//! Port Management
//!
//! Provides port management functions for Erlang drivers.

use super::types::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Port registry for managing driver ports
struct PortRegistry {
    ports: HashMap<u64, PortInfo>,
    next_id: u64,
}

struct PortInfo {
    port: DriverPort,
    os_pid: Option<i32>,
    data: Option<DriverData>,
}

impl PortRegistry {
    fn new() -> Self {
        Self {
            ports: HashMap::new(),
            next_id: 1,
        }
    }

    fn create_port(&mut self) -> DriverPort {
        let id = self.next_id;
        self.next_id += 1;
        let port = DriverPort::new(id);
        self.ports.insert(id, PortInfo {
            port,
            os_pid: None,
            data: None,
        });
        port
    }

    fn set_os_pid(&mut self, port: DriverPort, pid: i32) {
        if let Some(info) = self.ports.get_mut(&port.id()) {
            info.os_pid = Some(pid);
        }
    }

    fn get_os_pid(&self, port: DriverPort) -> Option<i32> {
        self.ports.get(&port.id()).and_then(|info| info.os_pid)
    }

    fn set_data(&mut self, port: DriverPort, data: DriverData) {
        if let Some(info) = self.ports.get_mut(&port.id()) {
            info.data = Some(data);
        }
    }

    fn get_data(&self, port: DriverPort) -> Option<DriverData> {
        self.ports.get(&port.id()).and_then(|info| info.data)
    }
}

lazy_static::lazy_static! {
    static ref PORT_REGISTRY: Arc<Mutex<PortRegistry>> = Arc::new(Mutex::new(PortRegistry::new()));
}

/// Create a new driver port
pub fn create_driver_port() -> DriverPort {
    let mut registry = PORT_REGISTRY.lock().unwrap();
    registry.create_port()
}

/// Set the OS process ID for a port
///
/// Equivalent to C's `erl_drv_set_os_pid`.
pub fn set_os_pid(port: DriverPort, pid: i32) {
    let mut registry = PORT_REGISTRY.lock().unwrap();
    registry.set_os_pid(port, pid);
}

/// Get the OS process ID for a port
pub fn get_os_pid(port: DriverPort) -> Option<i32> {
    let registry = PORT_REGISTRY.lock().unwrap();
    registry.get_os_pid(port)
}

/// Set driver data for a port
pub fn set_driver_data(port: DriverPort, data: DriverData) {
    let mut registry = PORT_REGISTRY.lock().unwrap();
    registry.set_data(port, data);
}

/// Get driver data for a port
pub fn get_driver_data(port: DriverPort) -> Option<DriverData> {
    let registry = PORT_REGISTRY.lock().unwrap();
    registry.get_data(port)
}

