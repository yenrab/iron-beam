//! System and Module Information Built-in Functions
//!
//! Provides system information, process information, and module information BIFs:
//! - System information queries (system_info/1)
//! - Process information (process_info/1, process_info/2)
//! - Module information (get_module_info/1, get_module_info/2)
//! - Function information (fun_info/2)
//!
//! This module implements safe Rust equivalents of Erlang information BIFs.

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
 *
 * Creation productivity increased for code in this file by using AALang and GAB.
 * See https://github.com/yenrab/AALang-Gab
 */

use crate::op::ErlangTerm;
use entities_process::{ProcessId, ProcessState};
use infrastructure_utilities::process_table::get_global_process_table;

/// Error type for information operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoError {
    /// Bad argument (e.g., invalid process ID, invalid info type)
    BadArgument(String),
    /// Process not found
    ProcessNotFound(String),
    /// Module not found
    ModuleNotFound(String),
    /// Operation not supported
    NotSupported(String),
}

/// System information BIF operations
pub struct InfoBif;

impl InfoBif {
    /// Get system information (system_info/1)
    ///
    /// Returns information about the current system based on the requested item.
    ///
    /// # Arguments
    /// * `item` - Information item to retrieve (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - System information value
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get scheduler ID
    /// let result = InfoBif::system_info_1(&ErlangTerm::Atom("scheduler_id".to_string()));
    /// assert!(result.is_ok());
    ///
    /// // Get process limit
    /// let result = InfoBif::system_info_1(&ErlangTerm::Atom("process_limit".to_string()));
    /// assert!(result.is_ok());
    ///
    /// // Get system version
    /// let result = InfoBif::system_info_1(&ErlangTerm::Atom("system_version".to_string()));
    /// assert!(result.is_ok());
    /// ```
    pub fn system_info_1(item: &ErlangTerm) -> Result<ErlangTerm, InfoError> {
        let item_str = match item {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "System info item must be an atom".to_string(),
                ));
            }
        };

        match item_str.as_str() {
            "scheduler_id" => {
                // Simplified: return scheduler ID 0
                Ok(ErlangTerm::Integer(0))
            }
            "compat_rel" => {
                // Compatibility release version
                Ok(ErlangTerm::Integer(26)) // Example: OTP 26
            }
            "multi_scheduling" => {
                // Multi-scheduling status
                Ok(ErlangTerm::Atom("enabled".to_string()))
            }
            "build_type" | "emu_type" => {
                // Build type (optimized, debug, etc.)
                #[cfg(debug_assertions)]
                {
                    Ok(ErlangTerm::Atom("debug".to_string()))
                }
                #[cfg(not(debug_assertions))]
                {
                    Ok(ErlangTerm::Atom("opt".to_string()))
                }
            }
            "emu_flavor" => {
                // Emulator flavor (jit, emu)
                Ok(ErlangTerm::Atom("emu".to_string()))
            }
            "time_offset" => {
                // Time offset state
                Ok(ErlangTerm::Atom("final".to_string()))
            }
            "time_correction" => {
                // Whether time correction is enabled
                Ok(ErlangTerm::Atom("true".to_string()))
            }
            "process_limit" => {
                // Maximum number of processes
                Ok(ErlangTerm::Integer(134217727)) // Default Erlang limit
            }
            "system_version" => {
                // System version string
                Ok(ErlangTerm::List(vec![
                    ErlangTerm::Integer(26), // Major version
                    ErlangTerm::Integer(0),  // Minor version
                ]))
            }
            "system_architecture" => {
                // System architecture
                #[cfg(target_arch = "x86_64")]
                {
                    Ok(ErlangTerm::Atom("x86_64-unknown-linux-gnu".to_string()))
                }
                #[cfg(target_arch = "aarch64")]
                {
                    Ok(ErlangTerm::Atom("aarch64-apple-darwin".to_string()))
                }
                #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    Ok(ErlangTerm::Atom("unknown".to_string()))
                }
            }
            "smp_support" => {
                // SMP (Symmetric Multi-Processing) support
                Ok(ErlangTerm::Atom("true".to_string()))
            }
            "threads" => {
                // Thread support
                Ok(ErlangTerm::Atom("true".to_string()))
            }
            "thread_pool_size" => {
                // Thread pool size
                Ok(ErlangTerm::Integer(10)) // Default thread pool size
            }
            "wordsize" => {
                // Word size in bytes
                Ok(ErlangTerm::Integer(8)) // 64-bit system
            }
            "otp_release" => {
                // OTP release version
                Ok(ErlangTerm::Atom("26".to_string()))
            }
            _ => {
                // Unknown system info item
                Err(InfoError::BadArgument(format!(
                    "Unknown system info item: {}",
                    item_str
                )))
            }
        }
    }

    /// Get process information (process_info/1)
    ///
    /// Returns information about a process. Returns a list of all process information.
    ///
    /// # Arguments
    /// * `pid` - Process ID
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - List of process information tuples
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get all process information
    /// let result = InfoBif::process_info_1(&ErlangTerm::Pid(123));
    /// assert!(result.is_ok());
    ///
    /// // Get process info for different PID
    /// let result = InfoBif::process_info_1(&ErlangTerm::Pid(456));
    /// assert!(result.is_ok());
    ///
    /// // Invalid: non-PID argument
    /// let result = InfoBif::process_info_1(&ErlangTerm::Atom("not_a_pid".to_string()));
    /// assert!(result.is_err());
    /// ```
    pub fn process_info_1(pid: &ErlangTerm) -> Result<ErlangTerm, InfoError> {
        let pid_value = match pid {
            ErlangTerm::Pid(val) => *val,
            _ => {
                return Err(InfoError::BadArgument(
                    "Process ID must be a PID".to_string(),
                ));
            }
        };

        // Look up the process in the process table
        let table = get_global_process_table();
        let process = table.lookup(pid_value as ProcessId)
            .ok_or_else(|| InfoError::ProcessNotFound(
                format!("Process with PID {} not found", pid_value)
            ))?;

        // Build process information list
        let mut info = Vec::new();

        // Status
        let status_str = match process.get_state() {
            ProcessState::Free => "free",
            ProcessState::Exiting => "exiting",
            ProcessState::Active => "active",
            ProcessState::Running => "running",
            ProcessState::Suspended => "suspended",
            ProcessState::Gc => "garbage_collecting",
            ProcessState::SysTasks => "sys_tasks",
            ProcessState::RunningSys => "running_sys",
            ProcessState::Proxy => "proxy",
            ProcessState::DelayedSys => "delayed_sys",
            ProcessState::DirtyRunning => "dirty_running",
            ProcessState::DirtyRunningSys => "dirty_running_sys",
            ProcessState::Unknown(_) => "unknown",
        };
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("status".to_string()),
            ErlangTerm::Atom(status_str.to_string()),
        ]));

        // Heap size
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("heap_size".to_string()),
            ErlangTerm::Integer(process.heap_sz() as i64),
        ]));

        // Min heap size
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("min_heap_size".to_string()),
            ErlangTerm::Integer(process.min_heap_size() as i64),
        ]));

        // Max heap size (0 means unlimited)
        if process.max_heap_size() > 0 {
            info.push(ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("max_heap_size".to_string()),
                ErlangTerm::Integer(process.max_heap_size() as i64),
            ]));
        }

        // Stack size
        if let Some(stack_size) = process.stack_size_words() {
            info.push(ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("stack_size".to_string()),
                ErlangTerm::Integer(stack_size as i64),
            ]));
        }

        // Reductions
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("reductions".to_string()),
            ErlangTerm::Integer(process.reds() as i64),
        ]));

        // Message queue length (not available yet, default to 0)
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("message_queue_len".to_string()),
            ErlangTerm::Integer(0),
        ]));

        // Priority (not available yet, default to normal)
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("priority".to_string()),
            ErlangTerm::Atom("normal".to_string()),
        ]));

        // Catches
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("catches".to_string()),
            ErlangTerm::Integer(process.catches() as i64),
        ]));

        // Return trace frames
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("return_trace_frames".to_string()),
            ErlangTerm::Integer(process.return_trace_frames() as i64),
        ]));

        // Arity
        info.push(ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("arity".to_string()),
            ErlangTerm::Integer(process.arity() as i64),
        ]));

        Ok(ErlangTerm::List(info))
    }

    /// Get specific process information (process_info/2)
    ///
    /// Returns specific information about a process.
    ///
    /// # Arguments
    /// * `pid` - Process ID
    /// * `item` - Information item to retrieve (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - Process information value
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get process status
    /// let result = InfoBif::process_info_2(
    ///     &ErlangTerm::Pid(123),
    ///     &ErlangTerm::Atom("status".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Get message queue length
    /// let result = InfoBif::process_info_2(
    ///     &ErlangTerm::Pid(456),
    ///     &ErlangTerm::Atom("message_queue_len".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Get process priority
    /// let result = InfoBif::process_info_2(
    ///     &ErlangTerm::Pid(789),
    ///     &ErlangTerm::Atom("priority".to_string()),
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn process_info_2(pid: &ErlangTerm, item: &ErlangTerm) -> Result<ErlangTerm, InfoError> {
        let pid_value = match pid {
            ErlangTerm::Pid(val) => *val,
            _ => {
                return Err(InfoError::BadArgument(
                    "Process ID must be a PID".to_string(),
                ));
            }
        };

        let item_str = match item {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "Process info item must be an atom".to_string(),
                ));
            }
        };

        // Look up the process in the process table
        let table = get_global_process_table();
        let process = table.lookup(pid_value as ProcessId)
            .ok_or_else(|| InfoError::ProcessNotFound(
                format!("Process with PID {} not found", pid_value)
            ))?;

        // Return the specific requested information item
        // Return the specific requested information item from the actual process
        match item_str.as_str() {
            "status" => {
                let status_str = match process.get_state() {
                    ProcessState::Free => "free",
                    ProcessState::Exiting => "exiting",
                    ProcessState::Active => "active",
                    ProcessState::Running => "running",
                    ProcessState::Suspended => "suspended",
                    ProcessState::Gc => "garbage_collecting",
                    ProcessState::SysTasks => "sys_tasks",
                    ProcessState::RunningSys => "running_sys",
                    ProcessState::Proxy => "proxy",
                    ProcessState::DelayedSys => "delayed_sys",
                    ProcessState::DirtyRunning => "dirty_running",
                    ProcessState::DirtyRunningSys => "dirty_running_sys",
                    ProcessState::Unknown(_) => "unknown",
                };
                Ok(ErlangTerm::Atom(status_str.to_string()))
            },
            "heap_size" => Ok(ErlangTerm::Integer(process.heap_sz() as i64)),
            "min_heap_size" => Ok(ErlangTerm::Integer(process.min_heap_size() as i64)),
            "max_heap_size" => {
                if process.max_heap_size() > 0 {
                    Ok(ErlangTerm::Integer(process.max_heap_size() as i64))
                } else {
                    // 0 means unlimited, return 0
                    Ok(ErlangTerm::Integer(0))
                }
            },
            "stack_size" => {
                if let Some(stack_size) = process.stack_size_words() {
                    Ok(ErlangTerm::Integer(stack_size as i64))
                } else {
                    Ok(ErlangTerm::Integer(0))
                }
            },
            "reductions" => Ok(ErlangTerm::Integer(process.reds() as i64)),
            "catches" => Ok(ErlangTerm::Integer(process.catches() as i64)),
            "return_trace_frames" => Ok(ErlangTerm::Integer(process.return_trace_frames() as i64)),
            "arity" => Ok(ErlangTerm::Integer(process.arity() as i64)),
            "priority" => {
                // Priority not yet available in Process struct, default to normal
                Ok(ErlangTerm::Atom("normal".to_string()))
            },
            "message_queue_len" => {
                // Message queue length not yet available in Process struct, default to 0
                Ok(ErlangTerm::Integer(0))
            },
            "current_function" => {
                // Current function not yet available in Process struct
                Ok(ErlangTerm::Tuple(vec![
                    ErlangTerm::Atom("erlang".to_string()),
                    ErlangTerm::Atom("apply".to_string()),
                    ErlangTerm::Integer(2),
                ]))
            },
            "initial_call" => {
                // Initial call not yet available in Process struct
                Ok(ErlangTerm::Tuple(vec![
                    ErlangTerm::Atom("erlang".to_string()),
                    ErlangTerm::Atom("apply".to_string()),
                    ErlangTerm::Integer(2),
                ]))
            },
            "dictionary" => {
                // Process dictionary not yet integrated, return empty list
                Ok(ErlangTerm::List(vec![]))
            },
            "error_handler" => {
                // Error handler not yet available in Process struct
                Ok(ErlangTerm::Atom("error_handler".to_string()))
            },
            _ => Err(InfoError::BadArgument(format!(
                "Unknown process info item: {}",
                item_str
            ))),
        }
    }

    /// Get module information (get_module_info/1)
    ///
    /// Returns all information about a module.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - List of module information tuples
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Setup: register a module first
    /// LoadBif::clear_all();
    /// LoadBif::register_module("test_module", crate::load::ModuleStatus::Loaded);
    ///
    /// // Get all module information
    /// let result = InfoBif::get_module_info_1(&ErlangTerm::Atom("test_module".to_string()));
    /// assert!(result.is_ok());
    ///
    /// // Get info for non-existent module
    /// let result = InfoBif::get_module_info_1(&ErlangTerm::Atom("nonexistent".to_string()));
    /// assert!(result.is_err());
    ///
    /// // Invalid: non-atom argument
    /// let result = InfoBif::get_module_info_1(&ErlangTerm::Integer(123));
    /// assert!(result.is_err());
    /// ```
    pub fn get_module_info_1(module: &ErlangTerm) -> Result<ErlangTerm, InfoError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        // Check if module is loaded and get its metadata
        use crate::load::LoadBif;
        let metadata = LoadBif::get_module_metadata(&module_name)
            .ok_or_else(|| InfoError::ModuleNotFound(format!(
                "Module {} not found",
                module_name
            )))?;

        // Get module metadata from the registry
        let md5_binary = metadata.md5
            .map(|md5| ErlangTerm::Binary(md5))
            .unwrap_or_else(|| ErlangTerm::Binary(vec![0; 16]));

        let info = vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("module".to_string()),
                ErlangTerm::Atom(module_name.clone()),
            ]),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("exports".to_string()),
                ErlangTerm::List(metadata.exports),
            ]),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("attributes".to_string()),
                ErlangTerm::List(metadata.attributes),
            ]),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("compile".to_string()),
                ErlangTerm::List(metadata.compile),
            ]),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("md5".to_string()),
                md5_binary,
            ]),
        ];

        Ok(ErlangTerm::List(info))
    }

    /// Get specific module information (get_module_info/2)
    ///
    /// Returns specific information about a module.
    ///
    /// # Arguments
    /// * `module` - Module name (atom)
    /// * `item` - Information item to retrieve (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - Module information value
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    /// use usecases_bifs::load::LoadBif;
    ///
    /// // Setup: register a module first
    /// LoadBif::clear_all();
    /// LoadBif::register_module("test_module", crate::load::ModuleStatus::Loaded);
    ///
    /// // Get module exports
    /// let result = InfoBif::get_module_info_2(
    ///     &ErlangTerm::Atom("test_module".to_string()),
    ///     &ErlangTerm::Atom("exports".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Get module MD5
    /// let result = InfoBif::get_module_info_2(
    ///     &ErlangTerm::Atom("test_module".to_string()),
    ///     &ErlangTerm::Atom("md5".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Get module attributes
    /// let result = InfoBif::get_module_info_2(
    ///     &ErlangTerm::Atom("test_module".to_string()),
    ///     &ErlangTerm::Atom("attributes".to_string()),
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn get_module_info_2(
        module: &ErlangTerm,
        item: &ErlangTerm,
    ) -> Result<ErlangTerm, InfoError> {
        let module_name = match module {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "Module name must be an atom".to_string(),
                ));
            }
        };

        let item_str = match item {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "Module info item must be an atom".to_string(),
                ));
            }
        };

        // Check if module is loaded and get its metadata
        use crate::load::LoadBif;
        let metadata = LoadBif::get_module_metadata(&module_name)
            .ok_or_else(|| InfoError::ModuleNotFound(format!(
                "Module {} not found",
                module_name
            )))?;

        // Return specific module information from actual metadata
        match item_str.as_str() {
            "module" => Ok(ErlangTerm::Atom(module_name)),
            "exports" => Ok(ErlangTerm::List(metadata.exports)),
            "attributes" => Ok(ErlangTerm::List(metadata.attributes)),
            "compile" => Ok(ErlangTerm::List(metadata.compile)),
            "md5" => {
                let md5_binary = metadata.md5
                    .map(|md5| ErlangTerm::Binary(md5))
                    .unwrap_or_else(|| ErlangTerm::Binary(vec![0; 16]));
                Ok(md5_binary)
            }
            _ => Err(InfoError::BadArgument(format!(
                "Unknown module info item: {}",
                item_str
            ))),
        }
    }

    /// Get function information (fun_info/2)
    ///
    /// Returns information about a function.
    ///
    /// # Arguments
    /// * `fun_term` - Function term
    /// * `item` - Information item to retrieve (atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm)` - Function information value
    /// * `Err(InfoError)` - If operation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::info::InfoBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// // Get function arity
    /// let fun_term = ErlangTerm::Function { arity: 1 };
    /// let result = InfoBif::fun_info_2(
    ///     &fun_term,
    ///     &ErlangTerm::Atom("arity".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Get function module
    /// let fun_term = ErlangTerm::Function { arity: 2 };
    /// let result = InfoBif::fun_info_2(
    ///     &fun_term,
    ///     &ErlangTerm::Atom("module".to_string()),
    /// );
    /// assert!(result.is_ok());
    ///
    /// // Invalid: non-function term
    /// let result = InfoBif::fun_info_2(
    ///     &ErlangTerm::Atom("not_a_function".to_string()),
    ///     &ErlangTerm::Atom("arity".to_string()),
    /// );
    /// assert!(result.is_err());
    /// ```
    pub fn fun_info_2(fun_term: &ErlangTerm, item: &ErlangTerm) -> Result<ErlangTerm, InfoError> {
        let item_str = match item {
            ErlangTerm::Atom(name) => name.clone(),
            _ => {
                return Err(InfoError::BadArgument(
                    "Function info item must be an atom".to_string(),
                ));
            }
        };

        match fun_term {
            ErlangTerm::Function { arity } => {
                match item_str.as_str() {
                    "arity" => Ok(ErlangTerm::Integer(*arity as i64)),
                    "type" => Ok(ErlangTerm::Atom("external".to_string())),
                    "module" => Ok(ErlangTerm::Atom("unknown".to_string())),
                    "name" => Ok(ErlangTerm::Atom("unknown".to_string())),
                    _ => Err(InfoError::BadArgument(format!(
                        "Unknown function info item: {}",
                        item_str
                    ))),
                }
            }
            _ => Err(InfoError::BadArgument(
                "First argument must be a function".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_1_scheduler_id() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("scheduler_id".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_system_info_1_compat_rel() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("compat_rel".to_string())).unwrap();
        assert!(matches!(result, ErlangTerm::Integer(_)));
    }

    #[test]
    fn test_system_info_1_multi_scheduling() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("multi_scheduling".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("enabled".to_string()));
    }

    #[test]
    fn test_system_info_1_build_type() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("build_type".to_string())).unwrap();
        assert!(matches!(result, ErlangTerm::Atom(_)));
    }

    #[test]
    fn test_system_info_1_process_limit() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("process_limit".to_string())).unwrap();
        assert!(matches!(result, ErlangTerm::Integer(_)));
    }

    #[test]
    fn test_system_info_1_invalid_item() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("invalid_item".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_system_info_1_port_limit_not_supported() {
        // Ports are not supported in this version
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("port_limit".to_string()));
        assert!(result.is_err());
        if let Err(InfoError::BadArgument(msg)) = result {
            assert!(msg.contains("port_limit") || msg.contains("Unknown system info item"));
        } else {
            panic!("Expected BadArgument error for port_limit");
        }
    }

    #[test]
    fn test_system_info_1_invalid_argument() {
        let result = InfoBif::system_info_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_info_1() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_1(&ErlangTerm::Pid(123)).unwrap();
        if let ErlangTerm::List(list) = result {
            assert!(!list.is_empty());
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_process_info_1_invalid_pid() {
        let result = InfoBif::process_info_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_info_2_status() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("status".to_string()),
        ).unwrap();
        // Default state is Unknown(0), which maps to "unknown"
        assert_eq!(result, ErlangTerm::Atom("unknown".to_string()));
    }

    #[test]
    fn test_process_info_2_priority() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("priority".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("normal".to_string()));
    }

    #[test]
    fn test_process_info_2_invalid_item() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("invalid_item".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_module_info_1() {
        // First register a module
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        let result = InfoBif::get_module_info_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        if let ErlangTerm::List(list) = result {
            assert!(!list.is_empty());
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_get_module_info_1_not_found() {
        use crate::load::LoadBif;
        LoadBif::clear_all();

        let result = InfoBif::get_module_info_1(&ErlangTerm::Atom("nonexistent".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_module_info_2_exports() {
        // First register a module
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        // Verify module is loaded before querying info
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("exports".to_string()),
        ).unwrap();
        assert!(matches!(result, ErlangTerm::List(_)));
    }

    #[test]
    fn test_get_module_info_2_module() {
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        // Verify module is loaded before querying info
        let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
        assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("module".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("test_module".to_string()));
    }

    #[test]
    fn test_fun_info_2_arity() {
        let fun_term = ErlangTerm::Function { arity: 3 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("arity".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(3));
    }

    #[test]
    fn test_fun_info_2_type() {
        let fun_term = ErlangTerm::Function { arity: 1 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("type".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("external".to_string()));
    }

    #[test]
    fn test_fun_info_2_invalid_fun() {
        let result = InfoBif::fun_info_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Atom("arity".to_string()),
        );
        assert!(result.is_err());
    }

    // Additional system_info_1 tests
    #[test]
    fn test_system_info_1_emu_type() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("emu_type".to_string())).unwrap();
        assert!(matches!(result, ErlangTerm::Atom(_)));
    }

    #[test]
    fn test_system_info_1_emu_flavor() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("emu_flavor".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("emu".to_string()));
    }

    #[test]
    fn test_system_info_1_time_offset() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("time_offset".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("final".to_string()));
    }

    #[test]
    fn test_system_info_1_time_correction() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("time_correction".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_system_info_1_system_version() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("system_version".to_string())).unwrap();
        if let ErlangTerm::List(list) = result {
            assert_eq!(list.len(), 2);
            assert!(matches!(list[0], ErlangTerm::Integer(_)));
            assert!(matches!(list[1], ErlangTerm::Integer(_)));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_system_info_1_system_architecture() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("system_architecture".to_string())).unwrap();
        assert!(matches!(result, ErlangTerm::Atom(_)));
    }

    #[test]
    fn test_system_info_1_smp_support() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("smp_support".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_system_info_1_threads() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("threads".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_system_info_1_thread_pool_size() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("thread_pool_size".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Integer(10));
    }

    #[test]
    fn test_system_info_1_wordsize() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("wordsize".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Integer(8));
    }

    #[test]
    fn test_system_info_1_otp_release() {
        let result = InfoBif::system_info_1(&ErlangTerm::Atom("otp_release".to_string())).unwrap();
        assert_eq!(result, ErlangTerm::Atom("26".to_string()));
    }

    // Additional process_info_2 tests
    #[test]
    fn test_process_info_2_message_queue_len() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("message_queue_len".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_process_info_2_heap_size() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("heap_size".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(233));
    }

    #[test]
    fn test_process_info_2_stack_size() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("stack_size".to_string()),
        ).unwrap();
        // Default process has no stack_top_index set, so returns 0
        assert_eq!(result, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_process_info_2_reductions() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("reductions".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_process_info_2_current_function() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("current_function".to_string()),
        ).unwrap();
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple.len(), 3);
            assert_eq!(tuple[0], ErlangTerm::Atom("erlang".to_string()));
            assert_eq!(tuple[1], ErlangTerm::Atom("apply".to_string()));
            assert_eq!(tuple[2], ErlangTerm::Integer(2));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_process_info_2_initial_call() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("initial_call".to_string()),
        ).unwrap();
        if let ErlangTerm::Tuple(tuple) = result {
            assert_eq!(tuple.len(), 3);
            assert_eq!(tuple[0], ErlangTerm::Atom("erlang".to_string()));
            assert_eq!(tuple[1], ErlangTerm::Atom("apply".to_string()));
            assert_eq!(tuple[2], ErlangTerm::Integer(2));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_process_info_2_dictionary() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("dictionary".to_string()),
        ).unwrap();
        assert!(matches!(result, ErlangTerm::List(_)));
    }

    #[test]
    fn test_process_info_2_error_handler() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Atom("error_handler".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("error_handler".to_string()));
    }

    #[test]
    fn test_process_info_2_invalid_item_type() {
        // Set up: Create a process in the process table
        use infrastructure_utilities::process_table::get_global_process_table;
        use entities_process::Process;
        use std::sync::Arc;
        
        let table = get_global_process_table();
        let process = Arc::new(Process::new(123));
        table.insert(123, Arc::clone(&process));
        
        let result = InfoBif::process_info_2(
            &ErlangTerm::Pid(123),
            &ErlangTerm::Integer(123),
        );
        assert!(result.is_err());
    }

    // Additional get_module_info_2 tests
    #[test]
    fn test_get_module_info_2_attributes() {
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("attributes".to_string()),
        ).unwrap();
        assert!(matches!(result, ErlangTerm::List(_)));
    }

    #[test]
    fn test_get_module_info_2_compile() {
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("compile".to_string()),
        ).unwrap();
        assert!(matches!(result, ErlangTerm::List(_)));
    }

    #[test]
    fn test_get_module_info_2_md5() {
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("md5".to_string()),
        ).unwrap();
        if let ErlangTerm::Binary(binary) = result {
            assert_eq!(binary.len(), 16);
        } else {
            panic!("Expected Binary");
        }
    }

    #[test]
    fn test_get_module_info_2_invalid_item_type() {
        use crate::load::LoadBif;
        use crate::load::ModuleStatus;
        LoadBif::clear_all();
        LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);

        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Integer(123),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_module_info_2_invalid_module_type() {
        let result = InfoBif::get_module_info_2(
            &ErlangTerm::Integer(123),
            &ErlangTerm::Atom("exports".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_module_info_1_invalid_module_type() {
        let result = InfoBif::get_module_info_1(&ErlangTerm::Integer(123));
        assert!(result.is_err());
    }

    // Additional fun_info_2 tests
    #[test]
    fn test_fun_info_2_module() {
        let fun_term = ErlangTerm::Function { arity: 1 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("module".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("unknown".to_string()));
    }

    #[test]
    fn test_fun_info_2_name() {
        let fun_term = ErlangTerm::Function { arity: 1 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("name".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Atom("unknown".to_string()));
    }

    #[test]
    fn test_fun_info_2_invalid_item() {
        let fun_term = ErlangTerm::Function { arity: 1 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("invalid_item".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_fun_info_2_invalid_item_type() {
        let fun_term = ErlangTerm::Function { arity: 1 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Integer(123),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_fun_info_2_arity_zero() {
        let fun_term = ErlangTerm::Function { arity: 0 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("arity".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_fun_info_2_arity_large() {
        let fun_term = ErlangTerm::Function { arity: 255 };
        let result = InfoBif::fun_info_2(
            &fun_term,
            &ErlangTerm::Atom("arity".to_string()),
        ).unwrap();
        assert_eq!(result, ErlangTerm::Integer(255));
    }

    #[test]
    fn test_get_module_info_2_md5_from_prepared_code() {
        // Test that MD5 is stored when module is loaded via finish_loading
        use crate::load::LoadBif;
        LoadBif::clear_all();

        // Prepare code with MD5
        let code = vec![0xBE, 0x00, 0x01, 0x02, 0x03];
        let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Binary(code),
        ).unwrap();

        // Finish loading to store MD5
        let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
        assert_eq!(result, ErlangTerm::Atom("ok".to_string()));

        // Get module info and verify MD5 is not all zeros
        let md5_result = InfoBif::get_module_info_2(
            &ErlangTerm::Atom("test_module".to_string()),
            &ErlangTerm::Atom("md5".to_string()),
        ).unwrap();

        if let ErlangTerm::Binary(md5) = md5_result {
            assert_eq!(md5.len(), 16);
            // MD5 should not be all zeros (it should be computed from the code)
            let all_zeros = md5.iter().all(|&b| b == 0);
            assert!(!all_zeros, "MD5 should be computed from code, not all zeros");
        } else {
            panic!("Expected Binary for MD5");
        }
    }
}

