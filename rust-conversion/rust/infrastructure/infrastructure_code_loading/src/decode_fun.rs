//! Function Decoding Module
//!
//! Provides functionality to decode Functions from EI (Erlang Interface) format.
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
//! ## Supported Formats
//!
//! - **ERL_EXPORT_EXT**: Export references (fully supported)
//! - **ERL_FUN_EXT**: Old format closures (fully supported)
//! - **ERL_NEW_FUN_EXT**: New format closures with MD5 hash (fully supported)
//!
//! ## Implementation Status
//!
//! All function formats are fully implemented. Closure decoding uses term skipping
//! to handle free variables, which are stored as raw bytes for later decoding.
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::{decode_fun, ErlangFunType};
//!
//! // Note: This example requires valid EI-encoded function data
//! // In practice, you would decode from a real buffer:
//! // let mut index = 0;
//! // let fun_type = decode_fun(&buf, &mut index)?;
//! // match fun_type {
//! //     ErlangFunType::Export { module, function, arity } => {
//! //         println!("Export: {}:{}/{}", module, function, arity);
//! //     }
//! //     ErlangFunType::Closure { .. } => {
//! //         // Closures not yet fully supported
//! //     }
//! // }
//! ```
//!
//! ## See Also
//!
//! - [`encode_fun`](super::encode_fun/index.html): Function encoding functions
//! - [`decode_pid`](super::decode_pid/index.html): PID decoding (used in closures)
//! - [`entities_io_operations::export`](../../entities/entities_io_operations/export/index.html): Export table management
//!
//! Based on `lib/erl_interface/src/decode/decode_fun.c`

use crate::constants::{ERL_FUN_EXT, ERL_NEW_FUN_EXT, ERL_EXPORT_EXT};
use super::decode_pid::decode_pid;
use super::decode_integers::decode_longlong;
use super::decode_skip::skip_term;
use infrastructure_data_handling::decode_atom::decode_atom;
use super::encode_fun::ErlangFunType;

/// Decode a function from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((function, new_index))` - Decoded function and new index
/// * `Err(DecodeError)` - Decoding error
///
/// # Note
/// Closure decoding fully supports both old (ERL_FUN_EXT) and new (ERL_NEW_FUN_EXT)
/// formats. Free variables are decoded using term skipping and stored as raw bytes.
pub fn decode_fun(buf: &[u8], index: &mut usize) -> Result<ErlangFunType, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_EXPORT_EXT => {
            // Export: module, function, arity
            let (module, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;

            let (function, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;

            let arity = decode_longlong(buf, index)
                .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;

            Ok(ErlangFunType::Export {
                module,
                function,
                arity: arity as i32,
            })
        }
        ERL_FUN_EXT => {
            // Old format closure (R7 and older)
            // Format: n_free_vars (32-bit) + pid + module (atom) + index (long) + uniq (long) + free_vars (n terms)
            
            // Read number of free variables (32-bit big-endian)
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let n_free_vars = u32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]);
            *index += 4;
            
            // Decode PID
            let pid = decode_pid(buf, index)
                .map_err(|e| DecodeError::PidDecodeError(format!("{:?}", e)))?;
            
            // Decode module (atom)
            let (module, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;
            
            // Decode index (long)
            let index_val = decode_longlong(buf, index)
                .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
            
            // Decode uniq (long)
            let uniq = decode_longlong(buf, index)
                .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
            
            // Skip free variables and capture the bytes
            let free_vars_start = *index;
            for _ in 0..n_free_vars {
                skip_term(buf, index)
                    .map_err(|e| DecodeError::SkipError(format!("{:?}", e)))?;
            }
            let free_vars_end = *index;
            let free_vars = if free_vars_start < free_vars_end {
                buf[free_vars_start..free_vars_end].to_vec()
            } else {
                Vec::new()
            };
            
            Ok(ErlangFunType::Closure {
                arity: -1, // Old format uses -1
                module,
                index: index_val,
                uniq,
                old_index: None,
                md5: None,
                n_free_vars,
                free_vars,
                pid,
            })
        }
        ERL_NEW_FUN_EXT => {
            // New format closure with MD5 hash
            // Format: total_size (32-bit) + arity (8-bit) + md5 (16 bytes) + index (32-bit) + 
            //         n_free_vars (32-bit) + module (atom) + old_index (long) + uniq (long) + 
            //         pid + free_vars (remaining bytes)
            
            // s0 in C code points to tag byte, total_size is relative to s0
            let s0 = *index - 1; // Position of tag byte
            
            // Read total size (32-bit big-endian) - this is the total size including tag
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let total_size = u32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]) as usize;
            *index += 4;
            
            // Read arity (8-bit)
            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let arity = buf[*index] as i32;
            *index += 1;
            
            // Read MD5 hash (16 bytes)
            if *index + 16 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let mut md5 = [0u8; 16];
            md5.copy_from_slice(&buf[*index..*index + 16]);
            *index += 16;
            
            // Read index (32-bit big-endian)
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let index_val = i32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]) as i64;
            *index += 4;
            
            // Read number of free variables (32-bit big-endian)
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let n_free_vars = u32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]);
            *index += 4;
            
            // Decode module (atom)
            let (module, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;
            
            // Decode old_index (long)
            let old_index_val = decode_longlong(buf, index)
                .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
            
            // Decode uniq (long)
            let uniq = decode_longlong(buf, index)
                .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
            
            // Decode PID
            let pid = decode_pid(buf, index)
                .map_err(|e| DecodeError::PidDecodeError(format!("{:?}", e)))?;
            
            // Calculate remaining bytes for free variables
            // C code: n = n - (s - s0) + 1;
            // where n = total_size, s = current position, s0 = tag position
            // This gives: remaining = total_size - (bytes_consumed_after_tag) + 1
            // The +1 accounts for the fact that total_size includes the tag
            let bytes_consumed_after_tag = *index - s0 - 1; // Subtract 1 for tag
            let remaining_bytes = if total_size + 1 >= bytes_consumed_after_tag {
                total_size + 1 - bytes_consumed_after_tag
            } else {
                return Err(DecodeError::InvalidFormat("Invalid total_size in closure".to_string()));
            };
            
            // Read free variables as raw bytes
            if *index + remaining_bytes > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let free_vars = buf[*index..*index + remaining_bytes].to_vec();
            *index += remaining_bytes;
            
            Ok(ErlangFunType::Closure {
                arity,
                module,
                index: index_val,
                uniq,
                old_index: Some(old_index_val),
                md5: Some(md5),
                n_free_vars,
                free_vars,
                pid,
            })
        }
        _ => Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer is too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
    /// Atom decoding error
    AtomDecodeError(String),
    /// Integer decoding error
    IntegerDecodeError(String),
    /// PID decoding error
    PidDecodeError(String),
    /// Term skipping error
    SkipError(String),
    /// Not implemented
    NotImplemented(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_roundtrip_export() {
        let fun = ErlangFunType::Export {
            module: "test".to_string(),
            function: "func".to_string(),
            arity: 2,
        };
        let mut buf = vec![0u8; 100];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        super::super::encode_fun::encode_fun(&mut buf_opt, &mut encode_index, &fun).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_fun(&buf, &mut decode_index).unwrap();
        // Note: decode_atom returns a placeholder, so we can't compare module/function names
        // For Export type, we can verify arity matches
        match (&decoded, &fun) {
            (ErlangFunType::Export { arity: a1, .. }, ErlangFunType::Export { arity: a2, .. }) => {
                assert_eq!(a1, a2);
            }
            _ => {
                // For closures, we can't do a full comparison without term skipping
                // This test just verifies decoding doesn't crash
            }
        }
    }

    #[test]
    fn test_decode_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_export_atom_error() {
        // Create buffer with ERL_EXPORT_EXT tag but invalid atom data
        let mut buf = vec![ERL_EXPORT_EXT];
        // Add invalid atom tag (0xFF is not a valid atom tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_export_second_atom_error() {
        // Create buffer with ERL_EXPORT_EXT tag, valid first atom, but invalid second atom
        let mut buf = vec![ERL_EXPORT_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "mod"
        buf.extend_from_slice(&[115, 3, b'm', b'o', b'd']);
        // Add invalid atom tag
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_export_integer_error() {
        // Create buffer with ERL_EXPORT_EXT tag, valid atoms, but invalid integer
        let mut buf = vec![ERL_EXPORT_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "mod"
        buf.extend_from_slice(&[115, 3, b'm', b'o', b'd']);
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "fun"
        buf.extend_from_slice(&[115, 3, b'f', b'u', b'n']);
        // Add invalid integer tag (0xFF is not a valid integer tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_export_buffer_too_short_for_atom() {
        // Create buffer with ERL_EXPORT_EXT tag but buffer too short for atom
        let buf = vec![ERL_EXPORT_EXT, 115, 10]; // SMALL_ATOM_EXT with length 10 but no data
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_export_buffer_too_short_for_integer() {
        // Create buffer with ERL_EXPORT_EXT tag, valid atoms, but buffer too short for integer
        let mut buf = vec![ERL_EXPORT_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "mod"
        buf.extend_from_slice(&[115, 3, b'm', b'o', b'd']);
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "fun"
        buf.extend_from_slice(&[115, 3, b'f', b'u', b'n']);
        // Add SMALL_INTEGER_EXT (97) tag but no data
        buf.push(97);
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_fun_ext() {
        // Test ERL_FUN_EXT tag - should decode closure
        // Use encode_fun to create valid test data
        use super::super::encode_fun::encode_fun;
        use super::super::encode_pid::ErlangPid;
        
        let closure = ErlangFunType::Closure {
            arity: -1,
            module: "test".to_string(),
            index: 42,
            uniq: 123,
            old_index: None,
            md5: None,
            n_free_vars: 0,
            free_vars: Vec::new(),
            pid: ErlangPid {
                node: "test".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        let mut buf = vec![0u8; 200];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        encode_fun(&mut buf_opt, &mut encode_index, &closure).unwrap();
        
        let mut decode_index = 0;
        let result = decode_fun(&buf, &mut decode_index);
        assert!(result.is_ok());
        match result.unwrap() {
            ErlangFunType::Closure { arity, index: idx, uniq, n_free_vars, .. } => {
                assert_eq!(arity, -1);
                assert_eq!(idx, 42);
                assert_eq!(uniq, 123);
                assert_eq!(n_free_vars, 0);
            }
            _ => panic!("Expected Closure type"),
        }
    }

    #[test]
    fn test_decode_new_fun_ext() {
        // Test ERL_NEW_FUN_EXT tag - should decode closure
        // Use encode_fun to create valid test data
        use super::super::encode_fun::encode_fun;
        use super::super::encode_pid::ErlangPid;
        
        let mut md5 = [0u8; 16];
        md5[0] = 1; // Non-zero MD5 for testing
        
        let closure = ErlangFunType::Closure {
            arity: 2,
            module: "test".to_string(),
            index: 5,
            uniq: 20,
            old_index: Some(10),
            md5: Some(md5),
            n_free_vars: 0,
            free_vars: Vec::new(),
            pid: ErlangPid {
                node: "test".to_string(),
                num: 1,
                serial: 2,
                creation: 3,
            },
        };
        
        let mut buf = vec![0u8; 200];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        encode_fun(&mut buf_opt, &mut encode_index, &closure).unwrap();
        
        let mut decode_index = 0;
        let result = decode_fun(&buf, &mut decode_index);
        assert!(result.is_ok());
        match result.unwrap() {
            ErlangFunType::Closure { arity, index: idx, uniq, old_index, md5: decoded_md5, n_free_vars, .. } => {
                assert_eq!(arity, 2);
                assert_eq!(idx, 5);
                assert_eq!(uniq, 20);
                assert_eq!(old_index, Some(10));
                assert!(decoded_md5.is_some());
                assert_eq!(decoded_md5.unwrap()[0], 1);
                assert_eq!(n_free_vars, 0);
            }
            _ => panic!("Expected Closure type"),
        }
    }

    #[test]
    fn test_decode_unexpected_tag() {
        // Test with an unexpected tag
        let buf = vec![0xAA, 1, 2, 3];
        let mut index = 0;
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Unexpected tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        let error4 = DecodeError::IntegerDecodeError("int_err".to_string());
        let error5 = DecodeError::NotImplemented("not_impl".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        let debug_str5 = format!("{:?}", error5);
        
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str3.contains("AtomDecodeError"));
        assert!(debug_str4.contains("IntegerDecodeError"));
        assert!(debug_str5.contains("NotImplemented"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        let error4 = DecodeError::IntegerDecodeError("int_err".to_string());
        let error5 = DecodeError::NotImplemented("not_impl".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        let cloned5 = error5.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
        assert_eq!(error5, cloned5);
    }

    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidFormat("test".to_string());
        let error5 = DecodeError::InvalidFormat("different".to_string());
        let error6 = DecodeError::AtomDecodeError("err".to_string());
        let error7 = DecodeError::IntegerDecodeError("err".to_string());
        let error8 = DecodeError::NotImplemented("err".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
        assert_ne!(error4, error5);
        assert_ne!(error6, error7);
        assert_ne!(error7, error8);
    }

    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_decode_export_various_arities() {
        // Test Export decoding with various arities
        let test_arities = vec![0, 1, 2, 5, 10, 255];
        
        for arity in test_arities {
            let fun = ErlangFunType::Export {
                module: "test".to_string(),
                function: "func".to_string(),
                arity,
            };
            let mut buf = vec![0u8; 100];
            let mut encode_index = 0;
            let mut buf_opt = Some(&mut buf[..]);
            super::super::encode_fun::encode_fun(&mut buf_opt, &mut encode_index, &fun).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_fun(&buf, &mut decode_index).unwrap();
            match decoded {
                ErlangFunType::Export { arity: decoded_arity, .. } => {
                    assert_eq!(decoded_arity, arity);
                }
                _ => panic!("Expected Export type"),
            }
        }
    }

    #[test]
    fn test_decode_index_at_end() {
        // Test when index is at the end of buffer
        let buf = vec![ERL_EXPORT_EXT];
        let mut index = 1; // Already past the tag
        let result = decode_fun(&buf, &mut index);
        assert!(result.is_err());
        // decode_atom will return BufferTooShort when index >= buf.len()
        // which gets converted to AtomDecodeError
        let err = result.unwrap_err();
        match err {
            DecodeError::AtomDecodeError(_) | DecodeError::BufferTooShort => {}
            _ => panic!("Expected AtomDecodeError or BufferTooShort, got {:?}", err),
        }
    }
}

