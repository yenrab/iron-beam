//! Process Dump Module
//!
//! Provides process dump functionality.
//! Based on erl_process_dump.c
//!
//! Process dumps provide detailed information about a process's state,
//! including heap, stack, registers, and other internal state.

use entities_process::{Process, ProcessId};
use infrastructure_utilities::process_table::get_global_process_table;

/// Process dump operations
pub struct ProcessDump;

impl ProcessDump {
    /// Dump process information to a formatted string
    ///
    /// # Arguments
    /// * `process` - Reference to the process to dump
    ///
    /// # Returns
    /// Formatted string containing process information
    ///
    /// # Examples
    /// ```
    /// use usecases_process_management::process_dump::ProcessDump;
    /// use entities_process::Process;
    ///
    /// let process = Process::new(123);
    /// let dump = ProcessDump::dump(&process);
    /// assert!(!dump.is_empty());
    /// assert!(dump.contains("Process"));
    /// ```
    pub fn dump(process: &Process) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("=== Process Dump ===\n"));
        output.push_str(&format!("Process ID: {}\n", process.get_id()));
        output.push_str(&format!("State: {:?}\n", process.get_state()));
        
        // Heap information
        output.push_str(&format!("Heap Size: {} words\n", process.heap_sz()));
        output.push_str(&format!("Min Heap Size: {} words\n", process.min_heap_size()));
        output.push_str(&format!("Max Heap Size: {} words\n", process.max_heap_size()));
        
        // Stack information
        if let Some(stack_size) = process.stack_size_words() {
            output.push_str(&format!("Stack Size: {} words\n", stack_size));
        }
        
        // Process flags and state
        output.push_str(&format!("Flags: 0x{:x}\n", process.flags()));
        output.push_str(&format!("Reductions: {}\n", process.reds()));
        output.push_str(&format!("FCalls: {}\n", process.fcalls()));
        output.push_str(&format!("Arity: {}\n", process.arity()));
        output.push_str(&format!("Catches: {}\n", process.catches()));
        output.push_str(&format!("Return Trace Frames: {}\n", process.return_trace_frames()));
        
        // Heap indices (safe index-based access)
        output.push_str(&format!("Heap Start Index: {}\n", process.heap_start_index()));
        output.push_str(&format!("Heap Top Index: {}\n", process.heap_top_index()));
        output.push_str(&format!("Stack Top Index: {:?}\n", process.stack_top_index()));
        output.push_str(&format!("Heap Data Length: {} words\n", process.heap_slice().len()));
        output.push_str(&format!("Program Counter: {:?}\n", process.i() as usize));
        
        // Process metadata
        output.push_str(&format!("Unique: {}\n", process.uniq()));
        output.push_str(&format!("Schedule Count: {}\n", process.schedule_count()));
        output.push_str(&format!("Suspend Count: {}\n", process.rcount()));
        
        output.push_str(&format!("===================\n"));
        
        output
    }

    /// Dump process information by process ID
    ///
    /// # Arguments
    /// * `process_id` - Process ID to dump
    ///
    /// # Returns
    /// Formatted string containing process information, or error message if process not found
    ///
    /// # Examples
    /// ```
    /// use usecases_process_management::process_dump::ProcessDump;
    /// use entities_process::Process;
    /// use infrastructure_utilities::process_table::get_global_process_table;
    /// use std::sync::Arc;
    ///
    /// // Register a process in the table
    /// let table = get_global_process_table();
    /// let process = Arc::new(Process::new(123));
    /// table.insert(123, Arc::clone(&process));
    ///
    /// // Dump the process
    /// let dump = ProcessDump::dump_by_id(123);
    /// assert!(!dump.is_empty());
    /// assert!(dump.contains("Process ID: 123"));
    /// ```
    pub fn dump_by_id(process_id: ProcessId) -> String {
        let table = get_global_process_table();
        if let Some(process) = table.lookup(process_id) {
            Self::dump(&process)
        } else {
            format!("=== Process Not Found ===\nProcess ID: {} not found in process table\n", process_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_dump() {
        let process = Process::new(1);
        let dump = ProcessDump::dump(&process);
        assert!(!dump.is_empty());
        assert!(dump.contains("Process Dump"));
        assert!(dump.contains("Process ID: 1"));
    }

    #[test]
    fn test_process_dump_by_id() {
        use infrastructure_utilities::process_table::get_global_process_table;
        use std::sync::Arc;
        
        // Register a process in the table
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let dump = ProcessDump::dump_by_id(123);
        assert!(!dump.is_empty());
        assert!(dump.contains("Process ID: 123"));
        
        // Test with non-existent process
        let dump_not_found = ProcessDump::dump_by_id(999);
        assert!(dump_not_found.contains("not found"));
    }

    #[test]
    fn test_process_dump_contains_info() {
        let process = Process::new(456);
        let dump = ProcessDump::dump(&process);
        assert!(dump.contains("Heap Size"));
        assert!(dump.contains("State"));
        assert!(dump.contains("Reductions"));
    }
}

