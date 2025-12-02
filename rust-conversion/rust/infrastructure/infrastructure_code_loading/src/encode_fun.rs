//! Function Encoding Module
//!
//! Provides functionality to encode functions to EI format.
//! Based on lib/erl_interface/src/encode/encode_fun.c

use crate::constants::{ERL_FUN_EXT, ERL_NEW_FUN_EXT, ERL_EXPORT_EXT};
use super::encode_pid::{encode_pid, ErlangPid, EncodeError as PidEncodeError};
use super::encode_integers::{encode_long, encode_longlong};
use infrastructure_data_handling::encode_atom::{encode_atom, EncodeAtomError};
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
                    .map_err(|e| EncodeError::IntegerEncodeError)?;
                encode_longlong(buf, index, *uniq)
                    .map_err(|e| EncodeError::IntegerEncodeError)?;

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
                        .map_err(|e| EncodeError::IntegerEncodeError)?;
                } else {
                    encode_longlong(buf, index, 0)
                        .map_err(|e| EncodeError::IntegerEncodeError)?;
                }

                encode_longlong(buf, index, *uniq)
                    .map_err(|e| EncodeError::IntegerEncodeError)?;

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
                .map_err(|e| EncodeError::IntegerEncodeError)?;
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
}

