//! Process Code Tracking Module
//!
//! Provides functionality to track which processes are using code from which modules.
//! This is used for code purging - determining if a module can be safely purged
//! by checking if any processes have code pointers pointing into the module.
//!
//! Based on check_process_code() in beam_bif_load.c

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

use entities_process::{Process, ErtsCodePtr, Eterm};
use infrastructure_utilities::process_table::get_global_process_table;

/// Module code area information
///
/// This struct contains the minimal information needed to check if a process
/// is using code from a module. It avoids dependency cycles by not depending
/// on the full ModuleInstance from code_management_code_loading.
#[derive(Debug, Clone, Copy)]
pub struct ModuleCodeArea {
    /// Start address of module code area
    pub code_start: *const u8,
    /// Size of module code area in bytes
    pub code_size: u32,
}

impl ModuleCodeArea {
    /// Create a new ModuleCodeArea from code start and size
    pub fn new(code_start: *const u8, code_size: u32) -> Self {
        Self {
            code_start,
            code_size,
        }
    }
    
    /// Create an empty/invalid ModuleCodeArea
    pub fn empty() -> Self {
        Self {
            code_start: std::ptr::null(),
            code_size: 0,
        }
    }
    
    /// Check if this module code area is valid (has code)
    pub fn is_valid(&self) -> bool {
        !self.code_start.is_null() && self.code_size > 0
    }
}

/// Check if a pointer is within a module's code area
///
/// Equivalent to ErtsInArea() macro in C code.
/// Checks if a pointer falls within the range [mod_start, mod_start + mod_size).
///
/// # Arguments
/// * `ptr` - Pointer to check
/// * `mod_start` - Start address of module code area
/// * `mod_size` - Size of module code area in bytes
///
/// # Returns
/// `true` if pointer is within the module area, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::pointer_in_module_area;
///
/// let mod_start = 0x1000 as *const u8;
/// let mod_size = 4096;
/// let ptr_in = 0x2000 as *const u8;
/// let ptr_out = 0x6000 as *const u8;
///
/// assert!(pointer_in_module_area(ptr_in, mod_start, mod_size));
/// assert!(!pointer_in_module_area(ptr_out, mod_start, mod_size));
/// ```
pub fn pointer_in_module_area(ptr: ErtsCodePtr, mod_start: *const u8, mod_size: u32) -> bool {
    if ptr.is_null() || mod_start.is_null() {
        return false;
    }
    
    let ptr_addr = ptr as usize;
    let mod_start_addr = mod_start as usize;
    let mod_end_addr = mod_start_addr + mod_size as usize;
    
    ptr_addr >= mod_start_addr && ptr_addr < mod_end_addr
}

/// Check if a single process has code pointers to a module
///
/// This checks if a process is using code from a module by examining:
/// 1. The instruction pointer (i) - current execution point
/// 2. NIF function pointers - native functions loaded in the module
/// 3. Continuation pointers on the stack - return addresses
///
/// Based on check_process_code() in beam_bif_load.c
///
/// # Arguments
/// * `process` - Process to check
/// * `module_code` - Module code area to check against (typically old code)
///
/// # Returns
/// `true` if process is using code from the module, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::{check_process_uses_module, ModuleCodeArea};
/// use entities_process::Process;
///
/// let process = Process::new(123);
/// let module_code = ModuleCodeArea::empty();
///
/// // Check if process uses the module
/// let uses = check_process_uses_module(&process, &module_code);
/// ```
pub fn check_process_uses_module(process: &Process, module_code: &ModuleCodeArea) -> bool {
    if !module_code.is_valid() {
        return false;
    }
    
    // Check 1: Instruction pointer (i) - current execution point
    let i = process.i();
    if pointer_in_module_area(i, module_code.code_start, module_code.code_size) {
        return true;
    }
    
    // Check 2: NIF functions in module area
    if check_nif_in_module_area(process, module_code.code_start, module_code.code_size) {
        return true;
    }
    
    // Check 3: Continuation pointers on stack
    if check_continuation_pointers_in_module(process, module_code.code_start, module_code.code_size) {
        return true;
    }
    
    false
}

/// Check if any process in the process table uses the module
///
/// Iterates through all processes in the process table and checks if any
/// of them have code pointers pointing into the module.
///
/// # Arguments
/// * `module_code` - Module code area to check against (typically old code)
///
/// # Returns
/// `true` if any process is using code from the module, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::{any_process_uses_module, ModuleCodeArea};
///
/// let module_code = ModuleCodeArea::empty();
///
/// // Check if any process uses the module
/// let any_uses = any_process_uses_module(&module_code);
/// ```
pub fn any_process_uses_module(module_code: &ModuleCodeArea) -> bool {
    let table = get_global_process_table();
    let process_ids = table.get_all_ids();
    
    for process_id in process_ids {
        if let Some(process) = table.lookup(process_id) {
            if check_process_uses_module(&process, module_code) {
                return true;
            }
        }
    }
    
    false
}

/// Check if any dirty process uses the module
///
/// Iterates through all processes and checks if any dirty processes
/// (processes running on dirty schedulers) have code pointers pointing
/// into the module.
///
/// A process is considered "dirty" if it has dirty flags set:
/// - DIRTY_RUNNING
/// - DIRTY_CPU_PROC
/// - DIRTY_IO_PROC
/// - DIRTY_ACTIVE_SYS
///
/// # Arguments
/// * `module_code` - Module code area to check against (typically old code)
///
/// # Returns
/// `true` if any dirty process is using code from the module, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::{any_dirty_process_uses_module, ModuleCodeArea};
///
/// let module_code = ModuleCodeArea::empty();
///
/// // Check if any dirty process uses the module
/// let any_dirty_uses = any_dirty_process_uses_module(&module_code);
/// ```
pub fn any_dirty_process_uses_module(module_code: &ModuleCodeArea) -> bool {
    let table = get_global_process_table();
    let process_ids = table.get_all_ids();
    
    for process_id in process_ids {
        if let Some(process) = table.lookup(process_id) {
            // Check if process is dirty
            if is_dirty_process(&process) {
                if check_process_uses_module(&process, module_code) {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Helper: Check if NIF functions are in the module area
///
/// Checks if the process has any NIF (Native Implemented Function) pointers
/// that point into the module's code area.
///
/// # Arguments
/// * `process` - Process to check
/// * `mod_start` - Start address of module code area
/// * `mod_size` - Size of module code area in bytes
///
/// # Returns
/// `true` if any NIF function pointer is in the module area, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::check_nif_in_module_area;
/// use entities_process::Process;
///
/// let process = Process::new(123);
/// let mod_start = 0x1000 as *const u8;
/// let mod_size = 4096;
///
/// let has_nif = check_nif_in_module_area(&process, mod_start, mod_size);
/// ```
pub fn check_nif_in_module_area(
    _process: &Process,
    _mod_start: *const u8,
    _mod_size: u32,
) -> bool {
    // Note: NIF tracking is not yet fully implemented in the Process struct.
    // In a full implementation, this would:
    // 1. Check process.nif pointer if it exists
    // 2. Check any NIF function table associated with the process
    // 3. Check off-heap structures that might contain NIF pointers
    //
    // For now, we return false as NIF tracking infrastructure is still being developed.
    // TODO: Implement NIF pointer checking when NIF infrastructure is available
    
    // Placeholder: In C code, this calls erts_check_nfunc_in_area() which
    // checks various NIF-related pointers. We'll need to add NIF tracking
    // to the Process struct to fully implement this.
    false
}

/// Helper: Check if continuation pointers on stack point into the module
///
/// Continuation pointers (CP) are return addresses stored on the process stack.
/// This function scans the stack for continuation pointers and checks if any
/// of them point into the module's code area.
///
/// # Arguments
/// * `process` - Process to check
/// * `mod_start` - Start address of module code area
/// * `mod_size` - Size of module code area in bytes
///
/// # Returns
/// `true` if any continuation pointer points into the module, `false` otherwise
///
/// # Examples
/// ```
/// use usecases_process_management::process_code_tracking::check_continuation_pointers_in_module;
/// use entities_process::Process;
///
/// let process = Process::new(123);
/// let mod_start = 0x1000 as *const u8;
/// let mod_size = 4096;
///
/// let has_cp = check_continuation_pointers_in_module(&process, mod_start, mod_size);
/// ```
pub fn check_continuation_pointers_in_module(
    process: &Process,
    mod_start: *const u8,
    mod_size: u32,
) -> bool {
    // Get stack boundaries
    let stack_top = match process.stack_top_index() {
        Some(stop) => stop,
        None => return false, // No stack
    };
    
    let heap_top = process.heap_top_index();
    
    // Stack grows downward, so stack_top is the top (lowest address in heap_data)
    // and heap_top is the bottom (highest address in heap_data)
    // We need to scan from stack_top to heap_top
    if stack_top >= process.heap_slice().len() {
        return false; // Invalid stack top
    }
    
    let heap_data = process.heap_slice();
    
    // Scan stack for continuation pointers
    // In Erlang, continuation pointers are tagged values on the stack
    // We need to check each word on the stack to see if it's a CP and points into module
    for i in stack_top..heap_top.min(heap_data.len()) {
        let val = heap_data[i];
        
        // Check if this is a continuation pointer
        // In Erlang, CPs are tagged with a specific tag
        // For now, we'll check if the value (when untagged) is a pointer in the module area
        // This is a simplified check - full implementation would properly tag/untag
        if is_continuation_pointer(val) {
            let cp_ptr = continuation_pointer_value(val);
            if pointer_in_module_area(cp_ptr, mod_start, mod_size) {
                return true;
            }
        }
    }
    
    false
}

/// Check if a value is a continuation pointer
///
/// Continuation pointers are tagged values. This function checks if a value
/// has the continuation pointer tag.
///
/// # Arguments
/// * `val` - Value to check
///
/// # Returns
/// `true` if value is a continuation pointer, `false` otherwise
fn is_continuation_pointer(val: Eterm) -> bool {
    // In Erlang, continuation pointers are tagged with TAG_CP
    // The exact tag value depends on the architecture and tagging scheme
    // For now, we use a heuristic: check if it looks like a pointer
    // In a full implementation, we'd check the actual CP tag
    
    // Simplified: check if value is in a reasonable pointer range and aligned
    // This is a placeholder - proper implementation needs CP tag checking
    (val & 0x3) == 0x1 && val > 0x1000 // Heuristic: tagged pointer, reasonable address
}

/// Extract the pointer value from a continuation pointer
///
/// Continuation pointers are tagged, so we need to untag them to get
/// the actual pointer value.
///
/// # Arguments
/// * `cp` - Continuation pointer value
///
/// # Returns
/// Untagged pointer value
fn continuation_pointer_value(cp: Eterm) -> ErtsCodePtr {
    // Untag the continuation pointer
    // In Erlang, CPs are tagged with TAG_CP, we need to remove the tag
    // Simplified: assume tag is in lowest 2 bits
    (cp & !0x3) as ErtsCodePtr
}

/// Check if a process is a dirty process
///
/// A process is considered "dirty" if it has dirty flags set:
/// - DIRTY_RUNNING (0x00800000)
/// - DIRTY_CPU_PROC (0x00100000)
/// - DIRTY_IO_PROC (0x00200000)
/// - DIRTY_ACTIVE_SYS (0x00400000)
///
/// Based on ERTS_PROC_IN_DIRTY_STATE macro in erl_process.h
///
/// # Arguments
/// * `process` - Process to check
///
/// # Returns
/// `true` if process is dirty, `false` otherwise
fn is_dirty_process(process: &Process) -> bool {
    let flags = process.flags();
    
    // Check for dirty flags
    // DIRTY_RUNNING = 0x00800000
    // DIRTY_CPU_PROC = 0x00100000
    // DIRTY_IO_PROC = 0x00200000
    // DIRTY_ACTIVE_SYS = 0x00400000
    // DIRTY_RUNNING_SYS = 0x01000000
    
    let dirty_work = 0x00100000 | 0x00200000 | 0x00400000; // DIRTY_CPU_PROC | DIRTY_IO_PROC | DIRTY_ACTIVE_SYS
    let dirty_running = 0x00800000; // DIRTY_RUNNING
    let dirty_running_sys = 0x01000000; // DIRTY_RUNNING_SYS
    let running_sys = 0x00008000; // RUNNING_SYS
    let running = 0x00000200; // RUNNING
    
    // Process is in dirty state if it has dirty work or is dirty running,
    // but NOT if it's running sys or running (those are handled separately)
    let has_dirty_work = (flags & dirty_work) != 0;
    let is_dirty_running = (flags & dirty_running) != 0;
    let is_running_sys = (flags & running_sys) != 0;
    let is_running = (flags & running) != 0;
    let is_dirty_running_sys = (flags & dirty_running_sys) != 0;
    
    (has_dirty_work || is_dirty_running) && !(is_running_sys || is_running || is_dirty_running_sys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities_process::Process;

    #[test]
    fn test_pointer_in_module_area() {
        let mod_start = 0x1000 as *const u8;
        let mod_size = 4096;
        
        // Pointer inside module (0x1500 is 0x1000 + 1280, well inside)
        let ptr_in = 0x1500 as *const u8;
        assert!(pointer_in_module_area(ptr_in, mod_start, mod_size));
        
        // Pointer at start
        assert!(pointer_in_module_area(mod_start, mod_start, mod_size));
        
        // Pointer just before end (one byte before the end)
        let ptr_near_end = unsafe { mod_start.add(mod_size as usize - 1) };
        assert!(pointer_in_module_area(ptr_near_end, mod_start, mod_size));
        
        // Pointer at exact end (should be excluded - it's one byte past)
        let ptr_at_end = unsafe { mod_start.add(mod_size as usize) };
        assert!(!pointer_in_module_area(ptr_at_end, mod_start, mod_size));
        
        // Pointer outside module
        let ptr_out = 0x6000 as *const u8;
        assert!(!pointer_in_module_area(ptr_out, mod_start, mod_size));
        
        // Null pointer
        assert!(!pointer_in_module_area(std::ptr::null(), mod_start, mod_size));
    }

    #[test]
    fn test_module_code_area() {
        let mod_start = 0x1000 as *const u8;
        let mod_size = 4096;
        let module_code = ModuleCodeArea::new(mod_start, mod_size);
        
        assert!(module_code.is_valid());
        assert_eq!(module_code.code_start, mod_start);
        assert_eq!(module_code.code_size, mod_size);
        
        let empty = ModuleCodeArea::empty();
        assert!(!empty.is_valid());
    }

    #[test]
    fn test_check_process_uses_module_no_code() {
        let process = Process::new(123);
        let module_code = ModuleCodeArea::empty();
        
        // Module with no code should return false
        assert!(!check_process_uses_module(&process, &module_code));
    }

    #[test]
    fn test_is_dirty_process() {
        // Create a process and test dirty flag detection
        // Note: We can't directly set flags in the current Process struct,
        // so this test verifies the logic works with default (non-dirty) process
        let process = Process::new(123);
        assert!(!is_dirty_process(&process));
    }

    #[test]
    fn test_any_process_uses_module_empty_table() {
        let module_code = ModuleCodeArea::empty();
        
        // With empty process table, should return false
        let table = get_global_process_table();
        let _initial_size = table.size();
        
        // Clear table for test
        // Note: ProcessTable doesn't have a clear() method, so we test with empty state
        let result = any_process_uses_module(&module_code);
        // Result depends on table state, but should not panic
        let _ = result;
    }
}

