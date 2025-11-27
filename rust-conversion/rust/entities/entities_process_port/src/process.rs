//! Process core data structure
//!
//! Provides the Process type definition based on struct process from erl_process.h

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

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::common::{ErtsPTabElementCommon, Eterm, ProcessId};

/// Process structure
///
/// Represents an Erlang process. This is a simplified version of the
/// full C struct process. The common field must be first (as required
/// by the C code structure).
///
/// Based on struct process in erl_process.h
#[derive(Debug)]
pub struct Process {
    /// Common element (MUST be first field)
    pub common: ErtsPTabElementCommon,
    
    /// Heap top pointer
    pub htop: Option<*mut Eterm>,
    /// Stack top pointer
    pub stop: Option<*mut Eterm>,
    
    /// Failure reason
    pub freason: u32,
    /// Failure value (exit/throw value)
    pub fvalue: Eterm,
    
    /// Number of reductions left to execute
    pub fcalls: i32,
    
    /// Process flags (trap exit, etc.)
    pub flags: u32,
    
    /// Suspend count
    pub rcount: u32,
    /// Schedule count (times left to reschedule a low prio process)
    pub schedule_count: u8,
    
    /// Number of live argument registers
    pub arity: u8,
    /// Maximum number of argument registers available
    pub max_arg_reg: u8,
    /// Pointer to argument registers
    pub arg_reg: Option<*mut Eterm>,
    /// Default array for argument registers
    pub def_arg_reg: [Eterm; 6],
    
    /// Heap start
    pub heap: Option<*mut Eterm>,
    /// Heap end
    pub hend: Option<*mut Eterm>,
    /// Abandoned heap (for delayed GC)
    pub abandoned_heap: Option<*mut Eterm>,
    
    /// Size of heap in words
    pub heap_sz: u64,
    /// Minimum size of heap (in words)
    pub min_heap_size: u64,
    /// Minimum size of virtual heap (in words)
    pub min_vheap_size: u64,
    /// Maximum size of heap (in words)
    pub max_heap_size: u64,
    
    /// Program counter (code pointer)
    pub i: Option<*const u8>,
    /// Number of catches on stack
    pub catches: i32,
    /// Number of return trace frames on stack
    pub return_trace_frames: i32,
    /// Number of reductions for this process
    pub reds: u64,
    /// Pid in charge (group leader, can be boxed)
    pub group_leader: Eterm,
    /// Latest exception stack trace dump
    pub ftrace: Eterm,
    
    /// Pointer to next process in run queue
    pub next: Option<*mut Process>,
    
    /// Process unique integer
    pub uniq: i64,
    
    /// Process dictionary (may be NULL)
    pub dictionary: Option<*mut ()>, // ProcDict type placeholder
    
    /// Sequential trace clock
    pub seq_trace_clock: u64,
    /// Sequential trace last count
    pub seq_trace_lastcnt: u64,
    /// Sequential trace token
    pub seq_trace_token: Eterm,
    
    /// Current Erlang function (module, function, arity)
    pub current: Option<*const ()>, // ErtsCodeMFA type placeholder
    
    /// Pid of process that created this process
    pub parent: Eterm,
    
    /// Flags that do not change
    pub static_flags: u32,
    
    /// Number of (minor) generational GCs
    pub gen_gcs: u16,
    /// Max minor gen GCs before fullsweep
    pub max_gen_gcs: u16,
    /// High water mark
    pub high_water: Option<*mut Eterm>,
    /// Old heap end (for generational GC)
    pub old_hend: Option<*mut Eterm>,
    /// Old heap top
    pub old_htop: Option<*mut Eterm>,
    /// Old heap
    pub old_heap: Option<*mut Eterm>,
    
    /// Off-heap data
    pub off_heap: OffHeap,
    
    /// Virtual heap block size for binaries
    pub bin_vheap_sz: u64,
    /// Virtual old heap block size for binaries
    pub bin_old_vheap_sz: u64,
    /// Virtual old heap size for binaries
    pub bin_old_vheap: u64,
    
    /// Process state flags (atomic)
    pub state: AtomicU32,
    /// Process extra state flags (atomic)
    pub xstate: AtomicU32,
    
    /// Run queue (atomic pointer)
    pub run_queue: AtomicU64,
}

/// Off-heap data structure
///
/// Simplified representation of ErlOffHeap
#[derive(Debug, Clone)]
pub struct OffHeap {
    /// First off-heap element
    pub first: Option<*mut ()>,
    /// Overhead
    pub overhead: u64,
}

impl Default for OffHeap {
    fn default() -> Self {
        Self {
            first: None,
            overhead: 0,
        }
    }
}

impl Process {
    /// Create a new empty process
    pub fn new(id: ProcessId) -> Self {
        Self {
            common: ErtsPTabElementCommon::new(id),
            htop: None,
            stop: None,
            freason: 0,
            fvalue: 0,
            fcalls: 0,
            flags: 0,
            rcount: 0,
            schedule_count: 0,
            arity: 0,
            max_arg_reg: 6,
            arg_reg: None,
            def_arg_reg: [0; 6],
            heap: None,
            hend: None,
            abandoned_heap: None,
            heap_sz: 0,
            min_heap_size: 0,
            min_vheap_size: 0,
            max_heap_size: 0,
            i: None,
            catches: 0,
            return_trace_frames: 0,
            reds: 0,
            group_leader: 0,
            ftrace: 0,
            next: None,
            uniq: 0,
            dictionary: None,
            seq_trace_clock: 0,
            seq_trace_lastcnt: 0,
            seq_trace_token: 0,
            current: None,
            parent: 0,
            static_flags: 0,
            gen_gcs: 0,
            max_gen_gcs: 0,
            high_water: None,
            old_hend: None,
            old_htop: None,
            old_heap: None,
            off_heap: OffHeap::default(),
            bin_vheap_sz: 0,
            bin_old_vheap_sz: 0,
            bin_old_vheap: 0,
            state: AtomicU32::new(0),
            xstate: AtomicU32::new(0),
            run_queue: AtomicU64::new(0),
        }
    }

    /// Get process ID
    pub fn get_id(&self) -> ProcessId {
        self.common.get_id()
    }

    /// Get process state
    pub fn get_state(&self) -> ProcessState {
        ProcessState::from_bits(self.state.load(Ordering::Acquire))
    }

    /// Set process state
    pub fn set_state(&self, state: ProcessState) {
        self.state.store(state.bits(), Ordering::Release);
    }
}

/// Process flags
///
/// Based on process flags in erl_process.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessFlags {
    /// Trap exit flag
    pub trap_exit: bool,
    /// Other flags (expanded as needed)
    pub other: u32,
}

impl ProcessFlags {
    /// Create new flags
    pub fn new() -> Self {
        Self {
            trap_exit: false,
            other: 0,
        }
    }

    /// Get flags as u32
    pub fn bits(&self) -> u32 {
        let mut bits = self.other;
        if self.trap_exit {
            bits |= 0x1; // Example flag bit
        }
        bits
    }
}

impl Default for ProcessFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Process state flags
///
/// Based on ERTS_PSFLG_* flags in erl_process.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessState {
    bits: u32,
}

impl ProcessState {
    /// Create from bits
    pub fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Get bits
    pub fn bits(&self) -> u32 {
        self.bits
    }

    /// Check if process is exiting
    pub fn is_exiting(&self) -> bool {
        (self.bits & 0x1) != 0 // ERTS_PSFLG_EXITING
    }

    /// Check if process is suspended
    pub fn is_suspended(&self) -> bool {
        (self.bits & 0x2) != 0 // ERTS_PSFLG_SUSPENDED
    }
}

impl Default for ProcessState {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

/// Process priority levels
///
/// Based on PRIORITY_* definitions in erl_process.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    /// Maximum priority
    Max = 0,
    /// High priority
    High = 1,
    /// Normal priority
    Normal = 2,
    /// Low priority
    Low = 3,
}

impl ProcessPriority {
    /// Number of priority levels
    pub const NO_LEVELS: u32 = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let process = Process::new(123);
        assert_eq!(process.get_id(), 123);
        assert_eq!(process.heap_sz, 0);
        assert_eq!(process.reds, 0);
    }

    #[test]
    fn test_process_state() {
        let process = Process::new(456);
        let state = process.get_state();
        assert!(!state.is_exiting());
        assert!(!state.is_suspended());
    }

    #[test]
    fn test_process_flags() {
        let mut flags = ProcessFlags::new();
        assert!(!flags.trap_exit);
        
        flags.trap_exit = true;
        assert!(flags.trap_exit);
        assert_ne!(flags.bits(), 0);
    }

    #[test]
    fn test_process_priority() {
        assert_eq!(ProcessPriority::Max as u32, 0);
        assert_eq!(ProcessPriority::High as u32, 1);
        assert_eq!(ProcessPriority::Normal as u32, 2);
        assert_eq!(ProcessPriority::Low as u32, 3);
        assert_eq!(ProcessPriority::NO_LEVELS, 4);
    }

    #[test]
    fn test_process_state_with_flags() {
        let process = Process::new(200);
        
        // Test with EXITING flag
        let state = ProcessState::from_bits(0x1);
        process.set_state(state);
        assert!(process.get_state().is_exiting());
        assert!(!process.get_state().is_suspended());
        
        // Test with SUSPENDED flag
        let state = ProcessState::from_bits(0x2);
        process.set_state(state);
        assert!(process.get_state().is_suspended());
        assert!(!process.get_state().is_exiting());
        
        // Test with both flags
        let state = ProcessState::from_bits(0x3);
        process.set_state(state);
        assert!(process.get_state().is_exiting());
        assert!(process.get_state().is_suspended());
    }

    #[test]
    fn test_process_state_bits() {
        let state = ProcessState::from_bits(0x1234);
        assert_eq!(state.bits(), 0x1234);
        
        let state2 = ProcessState::from_bits(0x5678);
        assert_eq!(state2.bits(), 0x5678);
        assert_ne!(state.bits(), state2.bits());
    }

    #[test]
    fn test_process_state_default() {
        let state = ProcessState::default();
        assert_eq!(state.bits(), 0);
        assert!(!state.is_exiting());
        assert!(!state.is_suspended());
    }

    #[test]
    fn test_process_flags_bits() {
        let mut flags = ProcessFlags::new();
        assert_eq!(flags.bits(), 0);
        
        flags.trap_exit = true;
        assert_ne!(flags.bits(), 0);
        assert!(flags.bits() & 0x1 != 0);
        
        flags.other = 0x100;
        assert!(flags.bits() & 0x100 != 0);
    }

    #[test]
    fn test_process_flags_default() {
        let flags = ProcessFlags::default();
        assert!(!flags.trap_exit);
        assert_eq!(flags.other, 0);
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_process_fields() {
        let mut process = Process::new(300);
        
        // Test heap fields
        process.heap_sz = 1024;
        process.min_heap_size = 512;
        process.min_vheap_size = 256;
        process.max_heap_size = 2048;
        assert_eq!(process.heap_sz, 1024);
        assert_eq!(process.min_heap_size, 512);
        assert_eq!(process.min_vheap_size, 256);
        assert_eq!(process.max_heap_size, 2048);
        
        // Test reduction fields
        process.fcalls = 100;
        process.reds = 500;
        assert_eq!(process.fcalls, 100);
        assert_eq!(process.reds, 500);
        
        // Test flags
        process.freason = 1;
        process.fvalue = 0x1234;
        process.flags = 0x5678;
        assert_eq!(process.freason, 1);
        assert_eq!(process.fvalue, 0x1234);
        assert_eq!(process.flags, 0x5678);
    }

    #[test]
    fn test_process_suspend_and_schedule() {
        let mut process = Process::new(400);
        assert_eq!(process.rcount, 0);
        assert_eq!(process.schedule_count, 0);
        
        process.rcount = 5;
        process.schedule_count = 3;
        assert_eq!(process.rcount, 5);
        assert_eq!(process.schedule_count, 3);
    }

    #[test]
    fn test_process_registers() {
        let mut process = Process::new(500);
        assert_eq!(process.arity, 0);
        assert_eq!(process.max_arg_reg, 6);
        assert_eq!(process.def_arg_reg, [0; 6]);
        
        process.arity = 3;
        process.max_arg_reg = 8;
        process.def_arg_reg[0] = 1;
        process.def_arg_reg[1] = 2;
        process.def_arg_reg[2] = 3;
        assert_eq!(process.arity, 3);
        assert_eq!(process.max_arg_reg, 8);
        assert_eq!(process.def_arg_reg[0], 1);
        assert_eq!(process.def_arg_reg[1], 2);
        assert_eq!(process.def_arg_reg[2], 3);
    }

    #[test]
    fn test_process_trace_fields() {
        let mut process = Process::new(600);
        assert_eq!(process.catches, 0);
        assert_eq!(process.return_trace_frames, 0);
        assert_eq!(process.ftrace, 0);
        
        process.catches = 5;
        process.return_trace_frames = 2;
        process.ftrace = 0xABCD;
        assert_eq!(process.catches, 5);
        assert_eq!(process.return_trace_frames, 2);
        assert_eq!(process.ftrace, 0xABCD);
    }

    #[test]
    fn test_process_sequential_trace() {
        let mut process = Process::new(700);
        assert_eq!(process.seq_trace_clock, 0);
        assert_eq!(process.seq_trace_lastcnt, 0);
        assert_eq!(process.seq_trace_token, 0);
        
        process.seq_trace_clock = 100;
        process.seq_trace_lastcnt = 50;
        process.seq_trace_token = 0x1234;
        assert_eq!(process.seq_trace_clock, 100);
        assert_eq!(process.seq_trace_lastcnt, 50);
        assert_eq!(process.seq_trace_token, 0x1234);
    }

    #[test]
    fn test_process_group_leader_and_parent() {
        let mut process = Process::new(800);
        assert_eq!(process.group_leader, 0);
        assert_eq!(process.parent, 0);
        
        process.group_leader = 0x1000;
        process.parent = 0x2000;
        assert_eq!(process.group_leader, 0x1000);
        assert_eq!(process.parent, 0x2000);
    }

    #[test]
    fn test_process_uniq() {
        let mut process = Process::new(900);
        assert_eq!(process.uniq, 0);
        
        process.uniq = 0x1234567890ABCDEF;
        assert_eq!(process.uniq, 0x1234567890ABCDEF);
    }

    #[test]
    fn test_process_static_flags() {
        let mut process = Process::new(1000);
        assert_eq!(process.static_flags, 0);
        
        process.static_flags = 0xFFFF;
        assert_eq!(process.static_flags, 0xFFFF);
    }

    #[test]
    fn test_process_gc_fields() {
        let mut process = Process::new(1100);
        assert_eq!(process.gen_gcs, 0);
        assert_eq!(process.max_gen_gcs, 0);
        
        process.gen_gcs = 10;
        process.max_gen_gcs = 20;
        assert_eq!(process.gen_gcs, 10);
        assert_eq!(process.max_gen_gcs, 20);
    }

    #[test]
    fn test_process_binary_vheap() {
        let mut process = Process::new(1200);
        assert_eq!(process.bin_vheap_sz, 0);
        assert_eq!(process.bin_old_vheap_sz, 0);
        assert_eq!(process.bin_old_vheap, 0);
        
        process.bin_vheap_sz = 1000;
        process.bin_old_vheap_sz = 500;
        process.bin_old_vheap = 250;
        assert_eq!(process.bin_vheap_sz, 1000);
        assert_eq!(process.bin_old_vheap_sz, 500);
        assert_eq!(process.bin_old_vheap, 250);
    }

    #[test]
    fn test_process_xstate() {
        let process = Process::new(1300);
        assert_eq!(process.xstate.load(Ordering::Acquire), 0);
        
        process.xstate.store(0xABCD, Ordering::Release);
        assert_eq!(process.xstate.load(Ordering::Acquire), 0xABCD);
    }

    #[test]
    fn test_process_run_queue() {
        let process = Process::new(1400);
        assert_eq!(process.run_queue.load(Ordering::Acquire), 0);
        
        process.run_queue.store(0x1234567890ABCDEF, Ordering::Release);
        assert_eq!(process.run_queue.load(Ordering::Acquire), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_off_heap() {
        let off_heap = OffHeap::default();
        assert!(off_heap.first.is_none());
        assert_eq!(off_heap.overhead, 0);
        
        let mut off_heap2 = OffHeap {
            first: None,
            overhead: 100,
        };
        assert_eq!(off_heap2.overhead, 100);
        off_heap2.overhead = 200;
        assert_eq!(off_heap2.overhead, 200);
    }

    #[test]
    fn test_process_off_heap() {
        let mut process = Process::new(1500);
        assert_eq!(process.off_heap.overhead, 0);
        
        process.off_heap.overhead = 500;
        assert_eq!(process.off_heap.overhead, 500);
    }

    #[test]
    fn test_process_priority_ordering() {
        assert!(ProcessPriority::Max < ProcessPriority::High);
        assert!(ProcessPriority::High < ProcessPriority::Normal);
        assert!(ProcessPriority::Normal < ProcessPriority::Low);
        
        assert!(ProcessPriority::Max <= ProcessPriority::High);
        assert!(ProcessPriority::High <= ProcessPriority::Normal);
        assert!(ProcessPriority::Normal <= ProcessPriority::Low);
        
        assert!(ProcessPriority::Low > ProcessPriority::Normal);
        assert!(ProcessPriority::Normal > ProcessPriority::High);
        assert!(ProcessPriority::High > ProcessPriority::Max);
    }
}

