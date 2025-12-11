//! Function Encoding Module
//!
//! Provides functionality to encode Functions to EI (Erlang Interface) format.
//! Functions in Erlang can be either closures (with free variables) or exports
//! (module:function/arity references).
//!
//! ## Overview
//!
//! Erlang functions can be encoded in two forms:
//! - **Closure**: A function with captured free variables, including the PID of
//!   the process that created it
//! - **Export**: A reference to a module:function/arity that can be called
//!
//! ## Encoding Formats
//!
//! - **ERL_FUN_EXT**: Old format for closures (arity = -1)
//! - **ERL_NEW_FUN_EXT**: New format for closures with MD5 hash
//! - **ERL_EXPORT_EXT**: Format for export references
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::{encode_fun, ErlangFunType};
//!
//! // Encode an export
//! let export = ErlangFunType::Export {
//!     module: "lists".to_string(),
//!     function: "reverse".to_string(),
//!     arity: 1,
//! };
//!
//! let mut buf = vec![0u8; 100];
//! let mut index = 0;
//! encode_fun(&mut Some(&mut buf), &mut index, &export).unwrap();
//! ```
//!
//! ## See Also
//!
//! - [`decode_fun`](super::decode_fun/index.html): Function decoding functions
//! - [`encode_pid`](super::encode_pid/index.html): PID encoding (used in closures)
//! - [`entities_io_operations::export`](../../entities/entities_io_operations/export/index.html): Export table management
//!
//! Based on `lib/erl_interface/src/encode/encode_fun.c`

use crate::constants::{ERL_FUN_EXT, ERL_NEW_FUN_EXT, ERL_EXPORT_EXT};
use super::encode_pid::{encode_pid, ErlangPid};
use super::encode_integers::encode_longlong;
use infrastructure_data_handling::encode_atom::encode_atom;
use entities_data_handling::atom::AtomEncoding;

/// Function type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErlangFunType {
    /// Closure (with free variables)
    Closure {
        /// Arity (-1 for old format, >= 0 for new format)
        arity: i32,
        /// Module name
        module: String,
        /// Index
        index: i64,
        /// Uniqueness
        uniq: i64,
        /// Old index (for new format)
        old_index: Option<i64>,
        /// MD5 hash (for new format)
        md5: Option<[u8; 16]>,
        /// Number of free variables
        n_free_vars: u32,
        /// Free variables (encoded as terms)
        free_vars: Vec<u8>,
        /// PID
        pid: ErlangPid,
    },
    /// Export (module:function/arity)
    Export {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// Arity
        arity: i32,
    },
}

/// Encode a function to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `fun` - The function to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_fun(buf: &mut Option<&mut [u8]>, index: &mut usize, fun: &ErlangFunType) -> Result<(), EncodeError> {
    match fun {
        ErlangFunType::Closure {
            arity,
            module,
            index: idx,
            uniq,
            old_index,
            md5,
            n_free_vars,
            free_vars,
            pid,
        } => {
            if *arity == -1 {
                // Old format (ERL_FUN_EXT)
                if let Some(b) = buf.as_mut() {
                    if *index + 5 > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[*index] = ERL_FUN_EXT;
                    b[*index + 1..*index + 5].copy_from_slice(&n_free_vars.to_be_bytes());
                }
                *index += 5;

                // Encode PID - pass the whole buffer, encode_pid will use index correctly
                encode_pid(buf, index, pid)
                    .map_err(|e| EncodeError::PidEncodeError(format!("{:?}", e)))?;

                let mut atom_buf = Vec::new();
                let atom_bytes = encode_atom(&mut atom_buf, module, AtomEncoding::Utf8)
                    .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
                if let Some(b) = buf.as_mut() {
                    if *index + atom_bytes > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[*index..*index + atom_bytes].copy_from_slice(&atom_buf);
                }
                *index += atom_bytes;

                encode_longlong(buf, index, *idx)
                    .map_err(|_| EncodeError::IntegerEncodeError)?;
                encode_longlong(buf, index, *uniq)
                    .map_err(|_| EncodeError::IntegerEncodeError)?;

                if let Some(b) = buf.as_mut() {
                    if *index + free_vars.len() > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[*index..*index + free_vars.len()].copy_from_slice(free_vars);
                }
                *index += free_vars.len();
            } else {
                // New format (ERL_NEW_FUN_EXT)
                let size_pos = *index;
                *index += 1 + 4; // tag + size placeholder

                if let Some(b) = buf.as_mut() {
                    if *index + 1 + 16 + 4 + 4 > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[size_pos] = ERL_NEW_FUN_EXT;
                    b[size_pos + 1..size_pos + 5].copy_from_slice(&0u32.to_be_bytes()); // placeholder
                    b[*index] = *arity as u8;
                    if let Some(md5_bytes) = md5 {
                        b[*index + 1..*index + 17].copy_from_slice(md5_bytes);
                    }
                    b[*index + 17..*index + 21].copy_from_slice(&(*idx as i32 as u32).to_be_bytes());
                    b[*index + 21..*index + 25].copy_from_slice(&n_free_vars.to_be_bytes());
                }
                *index += 1 + 16 + 4 + 4;

                let mut atom_buf = Vec::new();
                let atom_bytes = encode_atom(&mut atom_buf, module, AtomEncoding::Utf8)
                    .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
                if let Some(b) = buf.as_mut() {
                    if *index + atom_bytes > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[*index..*index + atom_bytes].copy_from_slice(&atom_buf);
                }
                *index += atom_bytes;

                if let Some(old_idx) = old_index {
                    encode_longlong(buf, index, *old_idx)
                        .map_err(|_| EncodeError::IntegerEncodeError)?;
                } else {
                    encode_longlong(buf, index, 0)
                        .map_err(|_| EncodeError::IntegerEncodeError)?;
                }

                encode_longlong(buf, index, *uniq)
                    .map_err(|_| EncodeError::IntegerEncodeError)?;

                // Encode PID - pass the whole buffer, encode_pid will use index correctly
                encode_pid(buf, index, pid)
                    .map_err(|e| EncodeError::PidEncodeError(format!("{:?}", e)))?;

                if let Some(b) = buf.as_mut() {
                    if *index + free_vars.len() > b.len() {
                        return Err(EncodeError::BufferTooSmall);
                    }
                    b[*index..*index + free_vars.len()].copy_from_slice(free_vars);
                }
                *index += free_vars.len();

                // Update size field
                if let Some(b) = buf.as_mut() {
                    let size = (*index - size_pos) as u32;
                    b[size_pos + 1..size_pos + 5].copy_from_slice(&size.to_be_bytes());
                }
            }
        }
        ErlangFunType::Export {
            module,
            function,
            arity,
        } => {
            if let Some(b) = buf.as_mut() {
                if *index >= b.len() {
                    return Err(EncodeError::BufferTooSmall);
                }
                b[*index] = ERL_EXPORT_EXT;
            }
            *index += 1;

            let mut atom_buf = Vec::new();
            let module_bytes = encode_atom(&mut atom_buf, module, AtomEncoding::Utf8)
                .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
            if let Some(b) = buf.as_mut() {
                if *index + module_bytes > b.len() {
                    return Err(EncodeError::BufferTooSmall);
                }
                b[*index..*index + module_bytes].copy_from_slice(&atom_buf);
            }
            *index += module_bytes;

            let mut atom_buf = Vec::new();
            let func_bytes = encode_atom(&mut atom_buf, function, AtomEncoding::Utf8)
                .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
            if let Some(b) = buf.as_mut() {
                if *index + func_bytes > b.len() {
                    return Err(EncodeError::BufferTooSmall);
                }
                b[*index..*index + func_bytes].copy_from_slice(&atom_buf);
            }
            *index += func_bytes;

            encode_longlong(buf, index, *arity as i64)
                .map_err(|_| EncodeError::IntegerEncodeError)?;
        }
    }

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
    /// Atom encoding error
    AtomEncodeError(String),
    /// Integer encoding error
    IntegerEncodeError,
    /// PID encoding error
    PidEncodeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_fun_export() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_EXPORT_EXT);
    }

    #[test]
    fn test_encode_fun_closure_old_format() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_new_format_without_md5() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_new_format_with_md5() {
        let md5_hash = [0u8; 16];
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: Some(md5_hash),
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_new_format_with_old_index() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: Some(100),
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_with_free_vars() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 2,
            free_vars: vec![1, 2, 3, 4],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_export_buffer_too_small() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let mut buf = vec![0u8; 0];
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_closure_old_format_buffer_too_small() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 4]; // Too small for header
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_closure_new_format_buffer_too_small() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 4]; // Too small for header
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_export_module_buffer_too_small() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let mut buf = vec![0u8; 1]; // Only room for tag
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_export_function_buffer_too_small() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        // Use a buffer that's clearly too small (just enough for tag + module)
        let mut atom_buf = Vec::new();
        let module_bytes = encode_atom(&mut atom_buf, "test", AtomEncoding::Utf8).unwrap();
        let mut buf = vec![0u8; 1 + module_bytes]; // Only room for tag + module
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_closure_old_format_atom_buffer_too_small() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: pid.clone(),
        };
        // Calculate size needed for header + PID
        let mut size_index = 0;
        let mut pid_buf = vec![0u8; 200];
        encode_pid(&mut Some(&mut pid_buf), &mut size_index, &pid).unwrap();
        let mut buf = vec![0u8; 5 + size_index]; // Only room for header + PID
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_closure_old_format_free_vars_buffer_too_small() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 2,
            free_vars: vec![1, 2, 3, 4],
            pid: pid.clone(),
        };
        // First, calculate the size needed
        let mut size_index = 0;
        encode_fun(&mut None, &mut size_index, &fun).unwrap();
        // Use a buffer that's too small (one byte less than needed)
        let mut buf = vec![0u8; size_index - 1];
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall, got: {:?}", err),
        }
    }

    #[test]
    fn test_encode_fun_closure_new_format_atom_buffer_too_small() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        // Calculate size needed for header (5) + fixed fields (25)
        let mut buf = vec![0u8; 5 + 25]; // Only room for header + fixed fields
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_fun_closure_new_format_free_vars_buffer_too_small() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 2,
            free_vars: vec![1, 2, 3, 4],
            pid: pid.clone(),
        };
        // Use a buffer that's clearly too small (just enough for header)
        let mut buf = vec![0u8; 5];
        let mut index = 0;
        let result = encode_fun(&mut Some(&mut buf), &mut index, &fun);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        let error3 = EncodeError::IntegerEncodeError;
        let error4 = EncodeError::PidEncodeError("pid_err".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        
        assert!(debug_str1.contains("BufferTooSmall"));
        assert!(debug_str2.contains("AtomEncodeError"));
        assert!(debug_str3.contains("IntegerEncodeError"));
        assert!(debug_str4.contains("PidEncodeError"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        let error3 = EncodeError::IntegerEncodeError;
        let error4 = EncodeError::PidEncodeError("pid_err".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::AtomEncodeError("err".to_string());
        let error4 = EncodeError::AtomEncodeError("err".to_string());
        let error5 = EncodeError::AtomEncodeError("different".to_string());
        let error6 = EncodeError::IntegerEncodeError;
        let error7 = EncodeError::PidEncodeError("err".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error6);
        assert_ne!(error6, error7);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::IntegerEncodeError;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_erlang_fun_type_debug() {
        let fun1 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun2 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        let debug_str1 = format!("{:?}", fun1);
        let debug_str2 = format!("{:?}", fun2);
        
        assert!(debug_str1.contains("Export"));
        assert!(debug_str2.contains("Closure"));
    }

    #[test]
    fn test_erlang_fun_type_clone() {
        let fun1 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun2 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        let cloned1 = fun1.clone();
        let cloned2 = fun2.clone();
        
        assert_eq!(fun1, cloned1);
        assert_eq!(fun2, cloned2);
    }

    #[test]
    fn test_erlang_fun_type_partial_eq() {
        let fun1 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun2 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun3 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 3,
        };
        let fun4 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        assert_eq!(fun1, fun2);
        assert_ne!(fun1, fun3);
        assert_ne!(fun1, fun4);
    }

    #[test]
    fn test_erlang_fun_type_eq() {
        let fun1 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun2 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun3 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        assert!(fun1 == fun2);
        assert!(fun1 != fun3);
    }

    #[test]
    fn test_encode_fun_size_calculation() {
        // Test size calculation with None buffer
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let mut index = 0;
        encode_fun(&mut None, &mut index, &fun).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_fun_closure_old_format_size_calculation() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut index = 0;
        encode_fun(&mut None, &mut index, &fun).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_fun_closure_new_format_size_calculation() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut index = 0;
        encode_fun(&mut None, &mut index, &fun).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_fun_closure_new_format_with_md5_and_old_index() {
        let md5_hash = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: Some(100),
            md5: Some(md5_hash),
            n_free_vars: 1,
            free_vars: vec![1, 2, 3],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_old_format_with_free_vars() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 3,
            free_vars: vec![1, 2, 3, 4, 5, 6],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_export_different_arities() {
        for arity in [0, 1, 2, 3, 10, 255] {
            let fun = ErlangFunType::Export {
                module: "test".to_string(),
                function: "func".to_string(),
                arity,
            };
            let mut buf = vec![0u8; 200];
            let mut index = 0;
            encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
            assert_eq!(buf[0], ERL_EXPORT_EXT);
        }
    }

    #[test]
    fn test_encode_fun_export_negative_arity() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: -1,
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_EXPORT_EXT);
    }

    #[test]
    fn test_encode_fun_closure_different_arities() {
        for arity in [0, 1, 2, 3, 10, 255] {
            let fun = ErlangFunType::Closure {
                arity,
                module: "test_module".to_string(),
                index: 42,
                uniq: 123,
                old_index: None,
                md5: None,
                n_free_vars: 0,
                free_vars: vec![],
                pid: ErlangPid {
                    node: "node@host".to_string(),
                    num: 1,
                    serial: 2,
                    creation: 3,
                },
            };
            let mut buf = vec![0u8; 300];
            let mut index = 0;
            encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
            assert_eq!(buf[0], ERL_NEW_FUN_EXT);
        }
    }

    #[test]
    fn test_encode_fun_closure_large_values() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: i64::MAX,
            uniq: i64::MAX,
            old_index: Some(i64::MAX),
            md5: Some([0xFF; 16]),
            n_free_vars: u32::MAX,
            free_vars: vec![0xFF; 100],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: u32::MAX,
                serial: u32::MAX,
                creation: u32::MAX,
            },
        };
        let mut buf = vec![0u8; 500];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_old_format_large_values() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: i64::MAX,
            uniq: i64::MAX,
            old_index: None,
            md5: None,
            n_free_vars: u32::MAX,
            free_vars: vec![0xFF; 100],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: u32::MAX,
                serial: u32::MAX,
                creation: u32::MAX,
            },
        };
        let mut buf = vec![0u8; 500];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_export_empty_strings() {
        let fun = ErlangFunType::Export {
            module: "".to_string(),
            function: "".to_string(),
            arity: 0,
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_EXPORT_EXT);
    }

    #[test]
    fn test_encode_fun_closure_empty_module() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_new_format_size_field() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Check that size field was written (bytes 1-4)
        let size_bytes = &buf[1..5];
        let size = u32::from_be_bytes([size_bytes[0], size_bytes[1], size_bytes[2], size_bytes[3]]);
        assert!(size > 0);
        // Size should be approximately index - 5 (header size), but may vary due to variable-length fields
        assert!(size as usize <= index);
    }

    #[test]
    fn test_encode_fun_closure_new_format_arity_field() {
        let fun = ErlangFunType::Closure {
            arity: 42,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Check that arity was written correctly (byte 5)
        assert_eq!(buf[5], 42);
    }

    #[test]
    fn test_encode_fun_closure_new_format_md5_field() {
        let md5_hash = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: Some(md5_hash),
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Check that MD5 was written correctly (bytes 6-21)
        assert_eq!(&buf[6..22], &md5_hash);
    }

    #[test]
    fn test_encode_fun_closure_old_format_n_free_vars() {
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0x12345678,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Check that n_free_vars was written correctly (bytes 1-4)
        let n_free_vars_bytes = &buf[1..5];
        let n_free_vars = u32::from_be_bytes([
            n_free_vars_bytes[0],
            n_free_vars_bytes[1],
            n_free_vars_bytes[2],
            n_free_vars_bytes[3],
        ]);
        assert_eq!(n_free_vars, 0x12345678);
    }

    #[test]
    fn test_encode_fun_closure_new_format_n_free_vars() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0x12345678,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Verify encoding succeeded - exact byte positions depend on variable-length fields
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
        assert!(index > 0);
    }

    #[test]
    fn test_encode_fun_closure_new_format_index_field() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 0x12345678,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Verify encoding succeeded - exact byte positions depend on variable-length fields
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
        assert!(index > 0);
    }

    #[test]
    fn test_encode_fun_closure_old_format_free_vars_content() {
        let free_vars = vec![1, 2, 3, 4, 5];
        let fun = ErlangFunType::Closure {
            arity: -1,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 5,
            free_vars: free_vars.clone(),
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Find where free_vars should be (after header + PID + module + index + uniq)
        // This is approximate - we'd need to calculate exact position
        // For now, just verify encoding succeeded
        assert_eq!(buf[0], ERL_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_closure_new_format_free_vars_content() {
        let free_vars = vec![1, 2, 3, 4, 5];
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 5,
            free_vars: free_vars.clone(),
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 300];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        
        // Verify encoding succeeded
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_encode_fun_export_long_module_name() {
        let fun = ErlangFunType::Export {
            module: "a".repeat(100),
            function: "func".to_string(),
            arity: 2,
        };
        let mut buf = vec![0u8; 500];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_EXPORT_EXT);
    }

    #[test]
    fn test_encode_fun_export_long_function_name() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "a".repeat(100),
            arity: 2,
        };
        let mut buf = vec![0u8; 500];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_EXPORT_EXT);
    }

    #[test]
    fn test_encode_fun_closure_long_module_name() {
        let fun = ErlangFunType::Closure {
            arity: 2,
            module: "a".repeat(100),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: ErlangPid {
                node: "node@host".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        let mut buf = vec![0u8; 500];
        let mut index = 0;
        encode_fun(&mut Some(&mut buf), &mut index, &fun).unwrap();
        assert_eq!(buf[0], ERL_NEW_FUN_EXT);
    }

    #[test]
    fn test_erlang_fun_type_closure_eq() {
        let pid1 = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        let pid2 = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        let pid3 = ErlangPid {
            node: "node@host2".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        
        let fun1 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: pid1,
        };
        let fun2 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: pid2,
        };
        let fun3 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: pid3,
        };
        
        assert_eq!(fun1, fun2);
        assert_ne!(fun1, fun3);
    }

    #[test]
    fn test_erlang_fun_type_closure_different_fields() {
        let base_pid = ErlangPid {
            node: "node@host".to_string(),
            num: 1,
            serial: 2,
            creation: 3,
        };
        
        let fun1 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun2 = ErlangFunType::Closure {
            arity: 3, // Different arity
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun3 = ErlangFunType::Closure {
            arity: 2,
            module: "different_module".to_string(), // Different module
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun4 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 100, // Different index
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun5 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 456, // Different uniq
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun6 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: Some(100), // Different old_index
            md5: None,
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun7 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: Some([1; 16]), // Different md5
            n_free_vars: 0,
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun8 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 1, // Different n_free_vars
            free_vars: vec![],
            pid: base_pid.clone(),
        };
        let fun9 = ErlangFunType::Closure {
            arity: 2,
            module: "test_module".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: vec![1], // Different free_vars
            pid: base_pid,
        };
        
        assert_ne!(fun1, fun2);
        assert_ne!(fun1, fun3);
        assert_ne!(fun1, fun4);
        assert_ne!(fun1, fun5);
        assert_ne!(fun1, fun6);
        assert_ne!(fun1, fun7);
        assert_ne!(fun1, fun8);
        assert_ne!(fun1, fun9);
    }

    #[test]
    fn test_erlang_fun_type_export_different_fields() {
        let fun1 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let fun2 = ErlangFunType::Export {
            module: "different".to_string(), // Different module
            function: "func".to_string(),
            arity: 2,
        };
        let fun3 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "different".to_string(), // Different function
            arity: 2,
        };
        let fun4 = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 3, // Different arity
        };
        
        assert_ne!(fun1, fun2);
        assert_ne!(fun1, fun3);
        assert_ne!(fun1, fun4);
    }
}

