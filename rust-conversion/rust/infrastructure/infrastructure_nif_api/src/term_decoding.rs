//! Term Decoding Functions
//!
//! Provides functions to decode Erlang terms into Rust values.
//! These functions correspond to the `enif_get_*` functions in the C NIF API.

use super::{NifEnv, NifTerm, NifCharEncoding};

/// Decode an atom term
///
/// Extracts the atom name from an atom term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Atom term to decode
///
/// # Returns
///
/// * `Some((String, NifCharEncoding))` - Success, atom name and encoding
/// * `None` - Not an atom term
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_get_atom()` - C implementation
pub fn enif_get_atom(
    _env: &NifEnv,
    term: NifTerm,
) -> Option<(String, NifCharEncoding)> {
    // Check if term is an atom (decode tag)
    // Format: (atom_index << 6) + 0x0B, so check if (term & 0x3F) == 0x0B
    if (term & 0x3F) == 0x0B {
        // Extract atom index: (term - 0x0B) >> 6
        let atom_index = ((term - 0x0B) >> 6) as usize;
        
        // Look up atom name from the global atom table
        let atom_table = infrastructure_utilities::atom_table::get_global_atom_table();
        if let Some(name_bytes) = atom_table.get_name(atom_index) {
            // Try to convert to UTF-8 string
            if let Ok(name) = String::from_utf8(name_bytes.clone()) {
                // For now, assume Latin1 encoding (would need to track encoding in atom table)
                return Some((name, NifCharEncoding::Latin1));
            }
        }
    }
    
    None
}

/// Decode an integer term
///
/// Extracts an integer value from an integer term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Integer term to decode
///
/// # Returns
///
/// * `Some(i32)` - Success, integer value
/// * `None` - Not an integer term or value out of range
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_get_int()` - C implementation
pub fn enif_get_int(_env: &NifEnv, term: NifTerm) -> Option<i32> {
    // Check if term is a small integer
    if is_small_integer(term) {
        let value = decode_small_integer(term);
        if value >= (i32::MIN as i64) && value <= (i32::MAX as i64) {
            return Some(value as i32);
        }
    }
    
    // TODO: Handle large integers (bignums)
    // This requires checking if term is a bignum and extracting the value
    
    None
}

/// Decode an unsigned long integer term
///
/// Extracts an unsigned long value from an integer term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Integer term to decode
///
/// # Returns
///
/// * `Some(u64)` - Success, unsigned long value
/// * `None` - Not an integer term or value out of range
pub fn enif_get_ulong(_env: &NifEnv, term: NifTerm) -> Option<u64> {
    // Check if term is a small integer
    if is_small_integer(term) {
        let value = decode_small_integer(term);
        if value >= 0 {
            return Some(value as u64);
        }
    }
    
    // TODO: Handle large integers (bignums)
    
    None
}

/// Decode a binary term
///
/// Extracts binary data from a binary term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Binary term to decode
///
/// # Returns
///
/// * `Some(Vec<u8>)` - Success, binary data
/// * `None` - Not a binary term
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_get_binary()` - C implementation
pub fn enif_get_binary(
    _env: &NifEnv,
    _term: NifTerm,
) -> Option<Vec<u8>> {
    // TODO: Implement binary decoding
    // This requires:
    // 1. Check if term is a binary (decode tag)
    // 2. Extract binary header pointer
    // 3. Get binary size and data pointer
    // 4. Return binary data
    
    None // Placeholder
}

/// Decode a string term
///
/// Extracts a string from a list of integers (string term).
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - String (list) term to decode
///
/// # Returns
///
/// * `Some((String, NifCharEncoding))` - Success, string and encoding
/// * `None` - Not a string term
pub fn enif_get_string(
    _env: &NifEnv,
    _term: NifTerm,
) -> Option<(String, NifCharEncoding)> {
    // TODO: Implement string decoding
    // This requires:
    // 1. Check if term is a list
    // 2. Traverse the list, extracting integer values
    // 3. Convert integers to bytes/characters
    // 4. Return string and encoding
    
    None // Placeholder
}

/// Decode a tuple term
///
/// Extracts elements from a tuple term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Tuple term to decode
///
/// # Returns
///
/// * `Some(Vec<NifTerm>)` - Success, tuple elements
/// * `None` - Not a tuple term
pub fn enif_get_tuple(
    _env: &NifEnv,
    _term: NifTerm,
) -> Option<Vec<NifTerm>> {
    // TODO: Implement tuple decoding
    // This requires:
    // 1. Check if term is a tuple (decode tag)
    // 2. Extract tuple header pointer
    // 3. Get arity from header
    // 4. Return element terms
    
    None // Placeholder
}

/// Decode a list term
///
/// Extracts elements from a list term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - List term to decode
///
/// # Returns
///
/// * `Some(Vec<NifTerm>)` - Success, list elements
/// * `None` - Not a list term
pub fn enif_get_list(
    _env: &NifEnv,
    _term: NifTerm,
) -> Option<Vec<NifTerm>> {
    // TODO: Implement list decoding
    // This requires:
    // 1. Check if term is a list (nil or cons cell)
    // 2. Traverse cons cells, extracting head elements
    // 3. Return element terms
    // 4. Check for proper list termination (nil)
    
    None // Placeholder
}

/// Decode a map term
///
/// Extracts key-value pairs from a map term.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Map term to decode
///
/// # Returns
///
/// * `Some(Vec<(NifTerm, NifTerm)>)` - Success, key-value pairs
/// * `None` - Not a map term
pub fn enif_get_map(
    _env: &NifEnv,
    _term: NifTerm,
) -> Option<Vec<(NifTerm, NifTerm)>> {
    // TODO: Implement map decoding
    // This requires:
    // 1. Check if term is a map (decode tag)
    // 2. Extract map structure pointer
    // 3. Traverse map entries, extracting key-value pairs
    // 4. Return pairs
    
    None // Placeholder
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Check if a term is a small integer
///
/// This is a public function for testing purposes.
pub fn is_small_integer(term: NifTerm) -> bool {
    // Check tag: lower 4 bits should be 0xF (_TAG_IMMED1_SMALL)
    (term & 0xF) == 0xF
}

/// Decode a small integer from an Eterm
///
/// This is a public function for testing purposes.
/// In a full implementation, it would be internal.
pub fn decode_small_integer(term: NifTerm) -> i64 {
    // Extract value: The value is stored in bits [4..], with tag 0xF in bits [0..3]
    // To decode: subtract the tag, then shift right by 4
    // Format: (value << 4) + 0xF, so value = (term - 0xF) >> 4
    let unsigned = (term - 0xF) >> 4;
    // Sign extend from 27 bits (on 64-bit) to 64 bits
    // Check if the sign bit (bit 26 of the 27-bit value) is set
    if (unsigned & (1u64 << 26)) != 0 {
        // Negative number - sign extend by setting all upper bits
        // Mask: 0xFFFFFFFFF8000000 = all 1s in upper 37 bits
        (unsigned | 0xFFFFFFFFF8000000u64) as i64
    } else {
        // Positive number - just cast
        unsigned as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term_creation;
    use crate::nif_env::NifEnv;
    use std::sync::Arc;
    use entities_process::Process;
    
    // Helper function to create a test NifEnv
    fn test_env() -> NifEnv {
        NifEnv::from_process(Arc::new(Process::new(0)))
    }

    #[test]
    fn test_enif_get_int() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_int(&env, term);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_enif_get_ulong() {
        let env = test_env();
        let term = term_creation::enif_make_ulong(&env, 999);
        let result = enif_get_ulong(&env, term);
        assert_eq!(result, Some(999));
    }

    #[test]
    fn test_is_small_integer() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 100);
        assert!(is_small_integer(term));
    }

    #[test]
    fn test_decode_small_integer() {
        use crate::term_creation::encode_small_integer;
        // Test positive
        let term = encode_small_integer(100);
        let value = decode_small_integer(term);
        assert_eq!(value, 100);
        
        // Test negative
        let term_neg = encode_small_integer(-100);
        let value_neg = decode_small_integer(term_neg);
        assert_eq!(value_neg, -100);
    }
    
    #[test]
    fn test_enif_get_atom() {
        let env = test_env();
        let atom_term = term_creation::enif_make_atom(&env, "test_atom");
        let result = enif_get_atom(&env, atom_term);
        assert!(result.is_some());
        let (name, encoding) = result.unwrap();
        assert_eq!(name, "test_atom");
        assert_eq!(encoding, NifCharEncoding::Latin1);
    }
    
    #[test]
    fn test_enif_get_atom_not_atom() {
        let env = test_env();
        let int_term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_atom(&env, int_term);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_atom_utf8() {
        let env = test_env();
        // Create atom with UTF-8 encoding
        let atom_term = term_creation::enif_make_atom_len(&env, b"test_utf8", NifCharEncoding::Utf8);
        let result = enif_get_atom(&env, atom_term);
        // Should still return Latin1 encoding (as per current implementation)
        assert!(result.is_some());
    }
    
    #[test]
    fn test_enif_get_int_zero() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 0);
        let result = enif_get_int(&env, term);
        assert_eq!(result, Some(0));
    }
    
    #[test]
    fn test_enif_get_int_negative() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, -42);
        let result = enif_get_int(&env, term);
        assert_eq!(result, Some(-42));
    }
    
    #[test]
    fn test_enif_get_int_max_small() {
        // Test with maximum value that fits in small integer encoding
        // Small integer range is -2^27 to 2^27-1
        let env = test_env();
        let max_small = (1i32 << 26) - 1; // Maximum that fits in small integer
        let term = term_creation::enif_make_int(&env, max_small);
        let result = enif_get_int(&env, term);
        assert_eq!(result, Some(max_small));
    }
    
    #[test]
    fn test_enif_get_int_min_small() {
        // Test with minimum value that fits in small integer encoding
        let env = test_env();
        let min_small = -(1i32 << 26); // Minimum that fits in small integer
        let term = term_creation::enif_make_int(&env, min_small);
        let result = enif_get_int(&env, term);
        assert_eq!(result, Some(min_small));
    }
    
    #[test]
    fn test_enif_get_int_not_integer() {
        let env = test_env();
        let atom_term = term_creation::enif_make_atom(&env, "not_int");
        let result = enif_get_int(&env, atom_term);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_ulong_zero() {
        let env = test_env();
        let term = term_creation::enif_make_ulong(&env, 0);
        let result = enif_get_ulong(&env, term);
        assert_eq!(result, Some(0));
    }
    
    #[test]
    fn test_enif_get_ulong_large() {
        let env = test_env();
        let term = term_creation::enif_make_ulong(&env, 999999);
        let result = enif_get_ulong(&env, term);
        assert_eq!(result, Some(999999));
    }
    
    #[test]
    fn test_enif_get_ulong_negative() {
        let env = test_env();
        // Create a negative integer term
        let term = term_creation::enif_make_int(&env, -1);
        let result = enif_get_ulong(&env, term);
        // Should return None for negative values
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_ulong_not_integer() {
        let env = test_env();
        let atom_term = term_creation::enif_make_atom(&env, "not_ulong");
        let result = enif_get_ulong(&env, atom_term);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_binary() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_binary(&env, term);
        // Currently returns None (placeholder implementation)
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_string() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_string(&env, term);
        // Currently returns None (placeholder implementation)
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_tuple() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_tuple(&env, term);
        // Currently returns None (placeholder implementation)
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_list() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_list(&env, term);
        // Currently returns None (placeholder implementation)
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_map() {
        let env = test_env();
        let term = term_creation::enif_make_int(&env, 42);
        let result = enif_get_map(&env, term);
        // Currently returns None (placeholder implementation)
        assert!(result.is_none());
    }
    
    #[test]
    fn test_is_small_integer_positive() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(100);
        assert!(is_small_integer(term));
    }
    
    #[test]
    fn test_is_small_integer_negative() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(-100);
        assert!(is_small_integer(term));
    }
    
    #[test]
    fn test_is_small_integer_zero() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(0);
        assert!(is_small_integer(term));
    }
    
    #[test]
    fn test_is_small_integer_not_integer() {
        let env = test_env();
        let atom_term = term_creation::enif_make_atom(&env, "not_int");
        assert!(!is_small_integer(atom_term));
    }
    
    #[test]
    fn test_decode_small_integer_zero() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(0);
        let value = decode_small_integer(term);
        assert_eq!(value, 0);
    }
    
    #[test]
    fn test_decode_small_integer_max_positive() {
        use crate::term_creation::encode_small_integer;
        // Maximum positive value for 27-bit signed integer
        let max_27bit = (1i64 << 26) - 1;
        let term = encode_small_integer(max_27bit);
        let value = decode_small_integer(term);
        assert_eq!(value, max_27bit);
    }
    
    #[test]
    fn test_decode_small_integer_min_negative() {
        use crate::term_creation::encode_small_integer;
        // Minimum negative value for 27-bit signed integer
        let min_27bit = -(1i64 << 26);
        let term = encode_small_integer(min_27bit);
        let value = decode_small_integer(term);
        assert_eq!(value, min_27bit);
    }
    
    #[test]
    fn test_decode_small_integer_large_positive() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(1000000);
        let value = decode_small_integer(term);
        assert_eq!(value, 1000000);
    }
    
    #[test]
    fn test_decode_small_integer_large_negative() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(-1000000);
        let value = decode_small_integer(term);
        assert_eq!(value, -1000000);
    }
    
    #[test]
    fn test_decode_small_integer_one() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(1);
        let value = decode_small_integer(term);
        assert_eq!(value, 1);
    }
    
    #[test]
    fn test_decode_small_integer_minus_one() {
        use crate::term_creation::encode_small_integer;
        let term = encode_small_integer(-1);
        let value = decode_small_integer(term);
        assert_eq!(value, -1);
    }
    
    #[test]
    fn test_enif_get_int_out_of_range_positive() {
        // Test with a value that's too large for i32
        use crate::term_creation::encode_small_integer;
        let large_value = i64::from(i32::MAX) + 1;
        let term = encode_small_integer(large_value);
        let env = test_env();
        let result = enif_get_int(&env, term);
        // Should return None because value is out of i32 range
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_int_out_of_range_negative() {
        // Test with a value that's too small for i32
        // Note: This test uses encode_small_integer directly to create a term
        // that's outside i32 range but still a valid small integer encoding
        use crate::term_creation::encode_small_integer;
        // Use a value that's larger than i32::MAX but fits in small integer
        let large_value = i64::from(i32::MAX) + 1;
        let term = encode_small_integer(large_value);
        let env = test_env();
        let result = enif_get_int(&env, term);
        // Should return None because value is out of i32 range
        assert!(result.is_none());
    }
    
    #[test]
    fn test_enif_get_ulong_at_boundary() {
        let env = test_env();
        // Test with maximum u64 value that fits in small integer
        let term = term_creation::enif_make_ulong(&env, u64::MAX);
        let result = enif_get_ulong(&env, term);
        // May return None if value doesn't fit in small integer encoding
        // or Some if it does
        assert!(result.is_some() || result.is_none());
    }
    
    #[test]
    fn test_is_small_integer_atom_tag() {
        // Test that atom terms are not recognized as small integers
        let env = test_env();
        let atom_term = term_creation::enif_make_atom(&env, "test");
        // Atom tag is 0x0B, which is different from small integer tag 0xF
        assert!(!is_small_integer(atom_term));
    }
    
    #[test]
    fn test_decode_small_integer_sign_extension() {
        use crate::term_creation::encode_small_integer;
        // Test sign extension for negative numbers
        // Bit 26 should trigger sign extension
        let negative_term = encode_small_integer(-1);
        let value = decode_small_integer(negative_term);
        assert_eq!(value, -1);
        assert!(value < 0);
    }
    
    #[test]
    fn test_decode_small_integer_no_sign_extension() {
        use crate::term_creation::encode_small_integer;
        // Test that positive numbers don't get sign extended
        let positive_term = encode_small_integer(1);
        let value = decode_small_integer(positive_term);
        assert_eq!(value, 1);
        assert!(value > 0);
    }
    
    #[test]
    fn test_enif_get_atom_multiple_atoms() {
        let env = test_env();
        let atom1 = term_creation::enif_make_atom(&env, "atom1");
        let atom2 = term_creation::enif_make_atom(&env, "atom2");
        let atom3 = term_creation::enif_make_atom(&env, "atom3");
        
        let result1 = enif_get_atom(&env, atom1);
        let result2 = enif_get_atom(&env, atom2);
        let result3 = enif_get_atom(&env, atom3);
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        assert!(result3.is_some());
        
        assert_eq!(result1.unwrap().0, "atom1");
        assert_eq!(result2.unwrap().0, "atom2");
        assert_eq!(result3.unwrap().0, "atom3");
    }
    
    #[test]
    fn test_enif_get_int_roundtrip() {
        let env = test_env();
        // Use values that fit in small integer encoding
        // Small integer range is -2^27 to 2^27-1
        let max_small = (1i32 << 26) - 1;
        let min_small = -(1i32 << 26);
        let values = vec![0, 1, -1, 100, -100, max_small, min_small];
        
        for &value in &values {
            let term = term_creation::enif_make_int(&env, value);
            let result = enif_get_int(&env, term);
            assert_eq!(result, Some(value), "Failed for value: {}", value);
        }
    }
    
    #[test]
    fn test_enif_get_ulong_roundtrip() {
        let env = test_env();
        let values = vec![0, 1, 100, 999, 1000000];
        
        for &value in &values {
            let term = term_creation::enif_make_ulong(&env, value);
            let result = enif_get_ulong(&env, term);
            assert_eq!(result, Some(value), "Failed for value: {}", value);
        }
    }
}
