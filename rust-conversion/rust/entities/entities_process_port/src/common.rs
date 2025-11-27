//! Common types and structures for Process and Port
//!
//! Provides shared types used by both Process and Port structures.

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

use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Process ID (PID)
///
/// Represents an Erlang process identifier.
/// In the C code, this is typically represented as an Eterm.
pub type ProcessId = u64;

/// Port ID
///
/// Represents an Erlang port identifier.
/// In the C code, this is typically represented as an Eterm.
pub type PortId = u64;

/// Erlang term
///
/// Represents an Erlang term value.
/// This is a simplified representation for the entities layer.
/// Full term representation is in entities_data_handling.
pub type Eterm = u64;

/// Common element structure for process/port table
///
/// This structure is shared between Process and Port and must be
/// the first field in both structures (as required by the C code).
///
/// Based on ErtsPTabElementCommon in the C code.
#[derive(Debug)]
pub struct ErtsPTabElementCommon {
    /// Element ID (process ID or port ID)
    pub id: AtomicU64,
    /// Reference count (atomic)
    pub refc: AtomicU32,
    /// Trace information (simplified)
    pub tracee: TraceeInfo,
    /// Registration information (simplified)
    pub reg: Option<String>,
}

/// Trace information
#[derive(Debug, Clone)]
pub struct TraceeInfo {
    /// First reference
    pub first_ref: Option<u64>,
    /// All trace flags
    pub all_trace_flags: u32,
}

impl Default for TraceeInfo {
    fn default() -> Self {
        Self {
            first_ref: None,
            all_trace_flags: 0,
        }
    }
}

impl ErtsPTabElementCommon {
    /// Create a new common element
    pub fn new(id: u64) -> Self {
        Self {
            id: AtomicU64::new(id),
            refc: AtomicU32::new(0),
            tracee: TraceeInfo::default(),
            reg: None,
        }
    }

    /// Get the element ID
    pub fn get_id(&self) -> u64 {
        self.id.load(Ordering::Acquire)
    }

    /// Set the element ID
    pub fn set_id(&self, id: u64) {
        self.id.store(id, Ordering::Release);
    }

    /// Increment reference count
    pub fn inc_refc(&self) {
        self.refc.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrement reference count
    ///
    /// Returns true if the element is still referenced (refc > 0)
    pub fn dec_refc(&self) -> bool {
        let prev = self.refc.fetch_sub(1, Ordering::AcqRel);
        prev > 1
    }

    /// Read reference count
    pub fn read_refc(&self) -> u32 {
        self.refc.load(Ordering::Acquire)
    }
}

/// Invalid process ID constant
pub const ERTS_INVALID_PID: ProcessId = u64::MAX;

/// Invalid port ID constant
pub const ERTS_INVALID_PORT_ID: PortId = u64::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_element_creation() {
        let common = ErtsPTabElementCommon::new(123);
        assert_eq!(common.get_id(), 123);
        assert_eq!(common.read_refc(), 0);
    }

    #[test]
    fn test_common_element_refc() {
        let common = ErtsPTabElementCommon::new(456);
        
        // Increment a few times
        common.inc_refc();
        common.inc_refc();
        assert_eq!(common.read_refc(), 2);
        
        // Decrement
        assert!(common.dec_refc());
        assert_eq!(common.read_refc(), 1);
        
        // Decrement to zero
        assert!(!common.dec_refc());
        assert_eq!(common.read_refc(), 0);
    }

    #[test]
    fn test_common_element_id() {
        let common = ErtsPTabElementCommon::new(789);
        assert_eq!(common.get_id(), 789);
        
        common.set_id(999);
        assert_eq!(common.get_id(), 999);
    }

    #[test]
    fn test_tracee_info_default() {
        let tracee = TraceeInfo::default();
        assert!(tracee.first_ref.is_none());
        assert_eq!(tracee.all_trace_flags, 0);
    }

    #[test]
    fn test_tracee_info_with_values() {
        let mut tracee = TraceeInfo::default();
        tracee.first_ref = Some(12345);
        tracee.all_trace_flags = 0xABCD;
        
        assert_eq!(tracee.first_ref, Some(12345));
        assert_eq!(tracee.all_trace_flags, 0xABCD);
    }

    #[test]
    fn test_common_element_registration() {
        let mut common = ErtsPTabElementCommon::new(1000);
        assert!(common.reg.is_none());
        
        common.reg = Some("test_name".to_string());
        assert_eq!(common.reg.as_ref().unwrap(), "test_name");
        
        common.reg = None;
        assert!(common.reg.is_none());
    }

    #[test]
    fn test_common_element_tracee() {
        let mut common = ErtsPTabElementCommon::new(2000);
        assert_eq!(common.tracee.all_trace_flags, 0);
        assert!(common.tracee.first_ref.is_none());
        
        common.tracee.first_ref = Some(999);
        common.tracee.all_trace_flags = 0x1234;
        assert_eq!(common.tracee.first_ref, Some(999));
        assert_eq!(common.tracee.all_trace_flags, 0x1234);
    }

    #[test]
    fn test_common_element_refc_edge_cases() {
        let common = ErtsPTabElementCommon::new(3000);
        
        // Test multiple increments
        for _ in 0..10 {
            common.inc_refc();
        }
        assert_eq!(common.read_refc(), 10);
        
        // Test decrementing to 1
        for _ in 0..9 {
            assert!(common.dec_refc());
        }
        assert_eq!(common.read_refc(), 1);
        
        // Test final decrement
        assert!(!common.dec_refc());
        assert_eq!(common.read_refc(), 0);
    }

    #[test]
    fn test_common_element_id_large_values() {
        let common = ErtsPTabElementCommon::new(u64::MAX);
        assert_eq!(common.get_id(), u64::MAX);
        
        common.set_id(0);
        assert_eq!(common.get_id(), 0);
        
        common.set_id(0x1234567890ABCDEF);
        assert_eq!(common.get_id(), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_invalid_constants() {
        assert_eq!(ERTS_INVALID_PID, u64::MAX);
        assert_eq!(ERTS_INVALID_PORT_ID, u64::MAX);
    }

    #[test]
    fn test_common_element_concurrent_refc() {
        use std::sync::Arc;
        use std::thread;
        
        let common = Arc::new(ErtsPTabElementCommon::new(4000));
        let mut handles = vec![];
        
        // Spawn multiple threads to increment refc
        for _ in 0..5 {
            let common_clone = Arc::clone(&common);
            handles.push(thread::spawn(move || {
                for _ in 0..10 {
                    common_clone.inc_refc();
                }
            }));
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Should have 50 increments
        assert_eq!(common.read_refc(), 50);
    }
}

