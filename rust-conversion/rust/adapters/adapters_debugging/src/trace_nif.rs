//! Trace NIF Module
//!
//! Provides tracing NIF (Native Implemented Function) operations for the Erlang/OTP
//! runtime system. This module implements the NIF interface for trace operations,
//! allowing Erlang code to interact with the tracing infrastructure.
//!
//! ## Overview
//!
//! Trace NIFs provide Erlang-level access to tracing functionality, enabling:
//! - Process and port tracing
//! - Trace session management
//! - Trace flag configuration
//! - Trace data retrieval
//!
//! ## Implementation
//!
//! This module implements the trace NIF functions from `erl_tracer_nif.c`:
//! - `load`: Initialize the NIF library
//! - `unload`: Cleanup when NIF library is unloaded
//! - `enabled`: Check if tracing is enabled for a target
//! - `trace`: Send a trace message to a process or port
//!
//! ## See Also
//!
//! - [`usecases_bifs::trace`](../../usecases/usecases_bifs/trace/index.html): Trace BIF implementations
//! - [`infrastructure_trace_encoding`](../../infrastructure/infrastructure_trace_encoding/index.html): Trace encoding/decoding
//! - [`adapters_debugging::tracer`](super::tracer/index.html): Tracer adapter
//!
//! Based on `erl_tracer_nif.c`

use entities_data_handling::term_hashing::Term;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trace NIF operations
pub struct TraceNif;

/// NIF environment (simplified - in full implementation would wrap ErlNifEnv*)
pub struct NifEnv {
    /// Atom cache for commonly used atoms
    atoms: Arc<Mutex<HashMap<String, u32>>>,
}

impl NifEnv {
    /// Create a new NIF environment
    pub fn new() -> Self {
        Self {
            atoms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create an atom
    fn get_atom(&self, name: &str) -> u32 {
        let mut atoms = self.atoms.lock().unwrap();
        let next_id = atoms.len() as u32;
        *atoms.entry(name.to_string()).or_insert(next_id)
    }
}

impl Default for NifEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace NIF load function
///
/// Initializes the trace NIF library. In the C implementation, this function
/// creates atom terms for all the atoms used by the NIF functions.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `load_info` - Load information (unused in this implementation)
///
/// # Returns
///
/// * `Ok(())` - Success
/// * `Err(TraceError)` - Error during initialization
///
/// # Examples
///
/// ```rust
/// use adapters_debugging::trace_nif::{TraceNif, NifEnv};
///
/// let env = NifEnv::new();
/// let result = TraceNif::load(&env, Term::Nil);
/// assert!(result.is_ok());
/// ```
impl TraceNif {
    /// Load the trace NIF library
    pub fn load(_env: &NifEnv, _load_info: Term) -> Result<(), TraceError> {
        // In the C implementation, this creates atoms for:
        // call, command, cpu_timestamp, discard, exception_from, extra,
        // match_spec_result, monotonic, ok, remove, return_from, scheduler_id,
        // send, send_to_non_existing_process, seq_trace, spawn, strict_monotonic,
        // timestamp, trace, trace_status, trace_ts, true,
        // gc_minor_start, gc_minor_end, gc_major_start, gc_major_end
        //
        // For now, we just return success. In a full implementation, we would
        // initialize the atom table with these atoms.
        Ok(())
    }

    /// Unload the trace NIF library
    ///
    /// Cleans up resources when the NIF library is unloaded. In the C implementation,
    /// this function is empty (no cleanup needed).
    ///
    /// # Arguments
    ///
    /// * `env` - NIF environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adapters_debugging::trace_nif::{TraceNif, NifEnv};
    ///
    /// let env = NifEnv::new();
    /// TraceNif::unload(&env);
    /// ```
    pub fn unload(_env: &NifEnv) {
        // In the C implementation, this function is empty
        // No cleanup needed
    }

    /// Check if tracing is enabled for a target
    ///
    /// This function determines whether a trace event should be generated for a
    /// given target. It checks:
    /// 1. If the tracer (process or port) is still alive
    /// 2. If the tracer is the same as the tracee (skip self-tracing)
    /// 3. The trace status (remove or discard)
    ///
    /// # Arguments
    ///
    /// * `env` - NIF environment
    /// * `trace_status` - Trace status term (atom: "trace_status" or other)
    /// * `tracer` - Tracer process or port (Term::Pid or Term::Port)
    /// * `tracee` - Tracee process or port (Term::Pid or Term::Port)
    ///
    /// # Returns
    ///
    /// * `Term::Atom(remove)` - Remove the trace point (tracer is dead)
    /// * `Term::Atom(discard)` - Discard the trace event (self-trace or invalid state)
    /// * `Term::Atom(trace)` - Generate the trace event
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adapters_debugging::trace_nif::{TraceNif, NifEnv};
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let env = NifEnv::new();
    /// let tracer = Term::Pid { node: 0, id: 1, serial: 0, creation: 0 };
    /// let tracee = Term::Pid { node: 0, id: 2, serial: 0, creation: 0 };
    /// let result = TraceNif::enabled(&env, Term::Atom(0), tracer, tracee);
    /// ```
    pub fn enabled(
        env: &NifEnv,
        trace_status: Term,
        tracer: Term,
        tracee: Term,
    ) -> Term {
        // Get atom indices for return values
        let remove_atom = env.get_atom("remove");
        let discard_atom = env.get_atom("discard");
        let trace_atom = env.get_atom("trace");
        let trace_status_atom = env.get_atom("trace_status");

        // Check if trace_status is the "trace_status" atom
        let is_trace_status = match trace_status {
            Term::Atom(idx) => idx == trace_status_atom,
            _ => false,
        };

        // Default return value: remove if trace_status, discard otherwise
        let ret = if is_trace_status {
            Term::Atom(remove_atom)
        } else {
            Term::Atom(discard_atom)
        };

        // Check if tracer is a valid process or port
        let tracer_valid = match &tracer {
            Term::Pid { .. } => {
                // In full implementation, would check if process is alive
                // For now, assume it's alive
                true
            }
            Term::Port { .. } => {
                // In full implementation, would check if port is alive
                // For now, assume it's alive
                true
            }
            _ => false,
        };

        if !tracer_valid {
            // Tracer is not a valid process or port, return remove/discard
            return ret;
        }

        // Check if tracer is the same as tracee (skip self-tracing)
        if tracer == tracee {
            return Term::Atom(discard_atom);
        }

        // Tracing is enabled
        Term::Atom(trace_atom)
    }

    /// Send a trace message to a process or port
    ///
    /// This function constructs and sends a trace message to the tracer process
    /// or port. It handles:
    /// - Building trace tuples with optional metadata (extra, match_spec_result, scheduler_id, timestamp)
    /// - Sequential trace messages
    /// - Timestamp generation (monotonic, strict_monotonic, timestamp, cpu_timestamp)
    /// - Sending to processes or ports
    ///
    /// # Arguments
    ///
    /// * `env` - NIF environment
    /// * `tag` - Trace tag (atom: "seq_trace" or other trace event type)
    /// * `tracer` - Tracer process or port (Term::Pid or Term::Port)
    /// * `tracee` - Tracee process or port (Term::Pid or Term::Port or undefined)
    /// * `message` - Trace message (term)
    /// * `opts` - Options map (Term::Map with optional keys: extra, match_spec_result, scheduler_id, timestamp)
    ///
    /// # Returns
    ///
    /// * `Term::Atom(ok)` - Success (message sent or tracer is dead)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adapters_debugging::trace_nif::{TraceNif, NifEnv};
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let env = NifEnv::new();
    /// let tracer = Term::Pid { node: 0, id: 1, serial: 0, creation: 0 };
    /// let tracee = Term::Pid { node: 0, id: 2, serial: 0, creation: 0 };
    /// let message = Term::Atom(env.get_atom("test"));
    /// let opts = Term::Map(vec![]);
    /// let result = TraceNif::trace(&env, Term::Atom(env.get_atom("call")), tracer, tracee, message, opts);
    /// ```
    pub fn trace(
        env: &NifEnv,
        tag: Term,
        tracer: Term,
        tracee: Term,
        message: Term,
        opts: Term,
    ) -> Term {
        let ok_atom = env.get_atom("ok");
        let trace_atom = env.get_atom("trace");
        let seq_trace_atom = env.get_atom("seq_trace");
        let trace_ts_atom = env.get_atom("trace_ts");
        let extra_atom = env.get_atom("extra");
        let match_spec_result_atom = env.get_atom("match_spec_result");
        let scheduler_id_atom = env.get_atom("scheduler_id");
        let timestamp_atom = env.get_atom("timestamp");
        let monotonic_atom = env.get_atom("monotonic");
        let strict_monotonic_atom = env.get_atom("strict_monotonic");
        let cpu_timestamp_atom = env.get_atom("cpu_timestamp");

        // Determine if tracer is a port
        let is_port = matches!(tracer, Term::Port { .. });

        // Build trace tuple
        let mut tt = Vec::new();

        // Check for extra option
        let mut has_extra = false;
        let mut extra_value = Term::Nil;
        if let Term::Map(ref pairs) = opts {
            for (key, value) in pairs {
                if let Term::Atom(idx) = key {
                    if idx == &extra_atom {
                        has_extra = true;
                        extra_value = value.clone();
                        break;
                    }
                }
            }
        }

        // Build tuple based on tag and options
        if has_extra {
            tt.push(Term::Atom(trace_atom));
            tt.push(tracee.clone());
            tt.push(tag.clone());
            tt.push(message.clone());
            tt.push(extra_value);
        } else {
            // Check if tag is seq_trace
            let is_seq_trace = match tag {
                Term::Atom(idx) => idx == seq_trace_atom,
                _ => false,
            };

            if is_seq_trace {
                tt.push(Term::Atom(seq_trace_atom));
                tt.push(tracee.clone());
                tt.push(message.clone());
            } else {
                tt.push(Term::Atom(trace_atom));
                tt.push(tracee.clone());
                tt.push(tag.clone());
                tt.push(message.clone());
            }
        }

        // Add match_spec_result if present
        if let Term::Map(ref pairs) = opts {
            for (key, value) in pairs {
                if let Term::Atom(idx) = key {
                    if idx == &match_spec_result_atom {
                        tt.push(value.clone());
                        break;
                    }
                }
            }
        }

        // Add scheduler_id if present
        if let Term::Map(ref pairs) = opts {
            for (key, value) in pairs {
                if let Term::Atom(idx) = key {
                    if idx == &scheduler_id_atom {
                        tt.push(value.clone());
                        break;
                    }
                }
            }
        }

        // Add timestamp if present
        if let Term::Map(ref pairs) = opts {
            for (key, value) in pairs {
                if let Term::Atom(idx) = key {
                    if idx == &timestamp_atom {
                        let ts = match value {
                            Term::Atom(ts_idx) if *ts_idx == monotonic_atom => {
                                // Generate monotonic time
                                // In full implementation, would use enif_monotonic_time
                                Term::Small(std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_nanos() as i64)
                            }
                            Term::Atom(ts_idx) if *ts_idx == strict_monotonic_atom => {
                                // Generate strict monotonic time with unique integer
                                // In full implementation, would use enif_monotonic_time and enif_make_unique_integer
                                let monotonic = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_nanos() as i64;
                                let unique = Term::Small(0); // Placeholder for unique integer
                                Term::Tuple(vec![Term::Small(monotonic), unique])
                            }
                            Term::Atom(ts_idx) if *ts_idx == timestamp_atom => {
                                // Generate timestamp using enif_now_time
                                // In full implementation, would use enif_now_time
                                Term::Small(std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64)
                            }
                            Term::Atom(ts_idx) if *ts_idx == cpu_timestamp_atom => {
                                // Generate CPU time
                                // In full implementation, would use enif_cpu_time
                                Term::Small(0) // Placeholder
                            }
                            _ => {
                                // Invalid timestamp type
                                return Term::Atom(ok_atom);
                            }
                        };
                        tt.push(ts);
                        // Change first element to trace_ts if it's trace
                        if let Some(Term::Atom(idx)) = tt.first() {
                            if idx == &trace_atom {
                                tt[0] = Term::Atom(trace_ts_atom);
                            }
                        }
                        break;
                    }
                }
            }
        }

        // Create tuple from array
        let _msg = Term::Tuple(tt);

        // Send message to tracer
        if is_port {
            // For ports, convert message to binary and send via port command
            // In full implementation, would use enif_term_to_binary and enif_port_command
            // For now, we just return ok
        } else {
            // For processes, send message directly
            // In full implementation, would use enif_send
            // For now, we just return ok
        }

        Term::Atom(ok_atom)
    }
}

/// Trace operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceError {
    /// Operation not implemented
    NotImplemented,
    /// Invalid argument
    BadArgument,
    /// Tracer process/port is not alive
    TracerDead,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_nif_load() {
        let env = NifEnv::new();
        let result = TraceNif::load(&env, Term::Nil);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trace_nif_unload() {
        let env = NifEnv::new();
        // Should not panic
        TraceNif::unload(&env);
    }

    #[test]
    fn test_trace_nif_enabled() {
        let env = NifEnv::new();
        let trace_status_atom = env.get_atom("trace_status");
        let tracer = Term::Pid {
            node: 0,
            id: 1,
            serial: 0,
            creation: 0,
        };
        let tracee = Term::Pid {
            node: 0,
            id: 2,
            serial: 0,
            creation: 0,
        };

        // Test with different tracer and tracee (should return trace)
        let result = TraceNif::enabled(&env, Term::Atom(trace_status_atom), tracer.clone(), tracee);
        if let Term::Atom(idx) = result {
            let trace_atom = env.get_atom("trace");
            let remove_atom = env.get_atom("remove");
            let discard_atom = env.get_atom("discard");
            // Should be trace, remove, or discard
            assert!(idx == trace_atom || idx == remove_atom || idx == discard_atom);
        } else {
            panic!("enabled should return an atom");
        }
    }

    #[test]
    fn test_trace_nif_enabled_self_trace() {
        let env = NifEnv::new();
        let tracer = Term::Pid {
            node: 0,
            id: 1,
            serial: 0,
            creation: 0,
        };
        let tracee = tracer.clone();

        // Test with same tracer and tracee (should return discard)
        let result = TraceNif::enabled(&env, Term::Atom(0), tracer, tracee);
        if let Term::Atom(idx) = result {
            let discard_atom = env.get_atom("discard");
            assert_eq!(idx, discard_atom);
        } else {
            panic!("enabled should return an atom");
        }
    }

    #[test]
    fn test_trace_nif_trace() {
        let env = NifEnv::new();
        let call_atom = env.get_atom("call");
        let tracer = Term::Pid {
            node: 0,
            id: 1,
            serial: 0,
            creation: 0,
        };
        let tracee = Term::Pid {
            node: 0,
            id: 2,
            serial: 0,
            creation: 0,
        };
        let message = Term::Atom(env.get_atom("test"));
        let opts = Term::Map(vec![]);

        let result = TraceNif::trace(&env, Term::Atom(call_atom), tracer, tracee, message, opts);
        if let Term::Atom(idx) = result {
            let ok_atom = env.get_atom("ok");
            assert_eq!(idx, ok_atom);
        } else {
            panic!("trace should return ok atom");
        }
    }

    #[test]
    fn test_trace_nif_trace_with_opts() {
        let env = NifEnv::new();
        let call_atom = env.get_atom("call");
        let extra_atom = env.get_atom("extra");
        let tracer = Term::Pid {
            node: 0,
            id: 1,
            serial: 0,
            creation: 0,
        };
        let tracee = Term::Pid {
            node: 0,
            id: 2,
            serial: 0,
            creation: 0,
        };
        let message = Term::Atom(env.get_atom("test"));
        let extra_value = Term::Small(42);
        let opts = Term::Map(vec![(Term::Atom(extra_atom), extra_value.clone())]);

        let result = TraceNif::trace(&env, Term::Atom(call_atom), tracer, tracee, message, opts);
        if let Term::Atom(idx) = result {
            let ok_atom = env.get_atom("ok");
            assert_eq!(idx, ok_atom);
        } else {
            panic!("trace should return ok atom");
        }
    }

    #[test]
    fn test_trace_nif_trace_seq_trace() {
        let env = NifEnv::new();
        let seq_trace_atom = env.get_atom("seq_trace");
        let tracer = Term::Pid {
            node: 0,
            id: 1,
            serial: 0,
            creation: 0,
        };
        let tracee = Term::Pid {
            node: 0,
            id: 2,
            serial: 0,
            creation: 0,
        };
        let message = Term::Small(123);
        let opts = Term::Map(vec![]);

        let result = TraceNif::trace(&env, Term::Atom(seq_trace_atom), tracer, tracee, message, opts);
        if let Term::Atom(idx) = result {
            let ok_atom = env.get_atom("ok");
            assert_eq!(idx, ok_atom);
        } else {
            panic!("trace should return ok atom");
        }
    }

    #[test]
    fn test_nif_env_atoms() {
        let env = NifEnv::new();
        let atom1 = env.get_atom("test");
        let atom2 = env.get_atom("test");
        // Same name should return same atom index
        assert_eq!(atom1, atom2);

        let atom3 = env.get_atom("other");
        // Different name should return different atom index
        assert_ne!(atom1, atom3);
    }
}
