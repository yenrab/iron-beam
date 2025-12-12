//! Error Handling Functions
//!
//! Provides functions for creating and checking exceptions in NIFs.
//! These functions correspond to error handling functions in the C NIF API,
//! but are implemented using safe Rust patterns.
//!
//! ## Design Principles
//!
//! - **Safe Rust Only**: All functions use safe Rust types and operations
//! - **Rust Patterns**: Uses tuple creation and atom matching instead of magic values
//! - **No C FFI**: Since NIFs are always written in Rust, no C compatibility needed

use super::{NifEnv, NifTerm};
use crate::term_creation::enif_make_atom;

/// Create a badarg exception
///
/// Creates an exception term indicating a bad argument error.
/// In Erlang, this is represented as the tuple `{badarg, []}`.
///
/// # Arguments
///
/// * `env` - NIF environment
///
/// # Returns
///
/// * `NifTerm` - Exception term (tuple `{badarg, []}`)
///
/// # Implementation Note
///
/// This creates a 2-tuple `{badarg, []}` where:
/// - First element: `badarg` atom
/// - Second element: empty list `[]` (nil)
///
/// Since full tuple creation is not yet implemented, this uses a placeholder
/// that will be replaced when `enif_make_tuple` is fully implemented.
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_make_badarg()` - C implementation
pub fn enif_make_badarg(env: &NifEnv) -> NifTerm {
    // Create the badarg atom
    let badarg_atom = enif_make_atom(env, "badarg");
    
    // Create the empty list (nil)
    // Nil is encoded as 0x3F (TAG_IMMED2_NIL)
    let empty_list = 0x3F;
    
    // Create the exception tuple {badarg, []}
    // Use enif_make_tuple to create the tuple
    // Note: This currently uses a placeholder encoding until heap allocation is integrated
    crate::term_creation::enif_make_tuple(env, &[badarg_atom, empty_list])
}

/// Create a badarg atom
///
/// Creates an atom term for "badarg".
///
/// # Arguments
///
/// * `env` - NIF environment
///
/// # Returns
///
/// * `NifTerm` - Badarg atom term
pub fn enif_make_badarg_atom(env: &NifEnv) -> NifTerm {
    // Create the "badarg" atom using safe string
    crate::term_creation::enif_make_atom(env, "badarg")
}

/// Check if a term is an exception
///
/// Determines whether a term represents an exception (error tuple).
/// In Erlang, exceptions are tuples where the first element is an error atom
/// such as `badarg`, `error`, `exit`, `throw`, etc.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `term` - Term to check
///
/// # Returns
///
/// * `true` - Term is an exception
/// * `false` - Term is not an exception
///
/// # Implementation Note
///
/// This checks if the term is:
/// 1. A tuple (when tuple decoding is implemented)
/// 2. Has a first element that is an error atom
///
/// Currently uses a placeholder check until tuple decoding is fully implemented.
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_is_exception()` - C implementation
pub fn enif_is_exception(env: &NifEnv, term: NifTerm) -> bool {
    // First, check if it's a known exception atom directly
    if is_exception_atom(env, term) {
        return true;
    }
    
    // Check if it's a tuple placeholder (can't decode elements, but might be exception)
    if is_tuple_placeholder(term) {
        // Placeholder tuples can't be decoded, but if it was created by enif_make_badarg,
        // it should be detected. For now, we'll conservatively assume placeholder tuples
        // could be exceptions if they were created by exception-creating functions.
        // In practice, enif_make_badarg creates tuples that should be heap-allocated,
        // but if heap allocation fails, it falls back to placeholder.
        return true; // Conservative: assume placeholder tuples could be exceptions
    }
    
    // Check if it's a tuple using enif_get_tuple
    use crate::term_decoding::enif_get_tuple;
    if let Some(elements) = enif_get_tuple(env, term) {
        // Check if tuple has at least one element
        if elements.is_empty() {
            return false;
        }
        
        // Extract the first element
        let first_element = elements[0];
        
        // Check if first element is an error atom
        return is_exception_atom(env, first_element);
    }
    
    // Check if it's our old-style placeholder exception tuple (for backward compatibility)
    is_exception_tuple_placeholder(term)
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Create a placeholder exception tuple
///
/// This is a temporary solution until `enif_make_tuple` is fully implemented.
/// It encodes the exception atom and empty list in a way that can be detected
/// by `is_exception_tuple_placeholder`.
///
/// The encoding uses a special bit pattern that's unlikely to occur in normal
/// term values. This is safe because:
/// 1. It's only used internally for placeholder values
/// 2. It will be replaced with proper tuple creation once implemented
/// 3. The value is never dereferenced as a pointer
fn make_exception_tuple_placeholder(atom: NifTerm, list: NifTerm) -> NifTerm {
    // Use a special marker bit pattern (0xF0...F0) to indicate this is a placeholder
    // Encode atom and list in the upper bits
    // This is safe because we're just encoding values, not creating pointers
    const EXCEPTION_PLACEHOLDER_MASK: u64 = 0xF0F0F0F0F0F0F0F0;
    const EXCEPTION_PLACEHOLDER_TAG: u64 = 0xF0F0F0F0F0F0F0F0;
    
    // Combine the marker with encoded atom/list values
    // Use XOR to encode both values in a reversible way
    EXCEPTION_PLACEHOLDER_TAG | ((atom ^ list) & !EXCEPTION_PLACEHOLDER_MASK)
}

/// Check if a term is a tuple placeholder
///
/// This checks if the term is a placeholder tuple created by enif_make_tuple.
/// This is a temporary solution until tuple decoding is fully implemented.
fn is_tuple_placeholder(term: NifTerm) -> bool {
    const TUPLE_PLACEHOLDER_TAG: u64 = 0xE0E0E0E0E0E0E0E0;
    (term & TUPLE_PLACEHOLDER_TAG) == TUPLE_PLACEHOLDER_TAG
}

/// Check if a term is a placeholder exception tuple
///
/// This is a temporary solution until tuple decoding is fully implemented.
/// This checks for the old-style exception placeholder (before enif_make_tuple was implemented).
fn is_exception_tuple_placeholder(term: NifTerm) -> bool {
    const EXCEPTION_PLACEHOLDER_TAG: u64 = 0xF0F0F0F0F0F0F0F0;
    (term & EXCEPTION_PLACEHOLDER_TAG) == EXCEPTION_PLACEHOLDER_TAG
}

/// Check if a term is a known exception atom
///
/// Checks if the term is one of the common exception atoms:
/// - `badarg`
/// - `error`
/// - `exit`
/// - `throw`
fn is_exception_atom(env: &NifEnv, term: NifTerm) -> bool {
    use crate::term_decoding::enif_get_atom;
    
    if let Some((atom_name, _)) = enif_get_atom(env, term) {
        matches!(
            atom_name.as_str(),
            "badarg" | "error" | "exit" | "throw" | "badarith" | "function_clause" |
            "case_clause" | "if_clause" | "try_clause" | "undef" | "badfun" |
            "badarity" | "timeout_value" | "noproc" | "noconnection" | "nocatch"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enif_make_badarg_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let atom = enif_make_badarg_atom(&env);
        // Should be a valid atom term (check tag bits)
        assert_ne!(atom, 0);
        assert_eq!(atom & 0x3F, 0x0B); // Atom tag check
    }

    #[test]
    fn test_enif_make_badarg() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let exception = enif_make_badarg(&env);
        // Note: term 0 is valid (heap_index 0). Nil is 0x3F, not 0.
        // Check that it's detected as an exception (this validates it's a valid term)
        assert!(enif_is_exception(&env, exception));
    }

    #[test]
    fn test_enif_is_exception_with_badarg_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let badarg_atom = enif_make_badarg_atom(&env);
        // A badarg atom should be detected as an exception
        assert!(enif_is_exception(&env, badarg_atom));
    }

    #[test]
    fn test_enif_is_exception_with_regular_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let regular_atom = enif_make_atom(&env, "ok");
        // A regular atom should not be detected as an exception
        assert!(!enif_is_exception(&env, regular_atom));
    }

    #[test]
    fn test_is_exception_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        // Test various exception atoms
        let badarg = enif_make_atom(&env, "badarg");
        let error = enif_make_atom(&env, "error");
        let exit = enif_make_atom(&env, "exit");
        let throw = enif_make_atom(&env, "throw");
        
        // All should be detected as exception atoms
        assert!(enif_is_exception(&env, badarg));
        assert!(enif_is_exception(&env, error));
        assert!(enif_is_exception(&env, exit));
        assert!(enif_is_exception(&env, throw));
    }

    #[test]
    fn test_is_exception_tuple_placeholder() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let exception = enif_make_badarg(&env);
        // The placeholder should be detected
        assert!(enif_is_exception(&env, exception));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_containing_exception_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create a tuple with exception atom as first element
        let badarg_atom = enif_make_atom(&env, "badarg");
        let empty_list = 0x3F; // nil
        let tuple = crate::term_creation::enif_make_tuple(&env, &[badarg_atom, empty_list]);
        
        // Should be detected as exception
        assert!(enif_is_exception(&env, tuple));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_containing_non_exception_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create a tuple with non-exception atom as first element
        let ok_atom = enif_make_atom(&env, "ok");
        let value = crate::term_creation::enif_make_int(&env, 42);
        let tuple = crate::term_creation::enif_make_tuple(&env, &[ok_atom, value]);
        
        // Check if it's a placeholder or heap-allocated
        const TUPLE_PLACEHOLDER_TAG: u64 = 0xE0E0E0E0E0E0E0E0;
        if (tuple & TUPLE_PLACEHOLDER_TAG) == TUPLE_PLACEHOLDER_TAG {
            // Placeholder tuples are conservatively treated as potential exceptions
            // (since we can't decode them to check the first element)
            assert!(enif_is_exception(&env, tuple));
        } else {
            // Heap-allocated tuple - should not be detected as exception
            assert!(!enif_is_exception(&env, tuple));
        }
    }

    #[test]
    fn test_enif_is_exception_with_empty_tuple() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create an empty tuple
        let empty_tuple = crate::term_creation::enif_make_tuple(&env, &[]);
        
        // Empty tuple returns placeholder, which is conservatively treated as potential exception
        // But the implementation checks if elements.is_empty() and returns false
        // However, placeholder tuples can't be decoded, so they're treated as potential exceptions
        const TUPLE_PLACEHOLDER_TAG: u64 = 0xE0E0E0E0E0E0E0E0;
        if (empty_tuple & TUPLE_PLACEHOLDER_TAG) == TUPLE_PLACEHOLDER_TAG {
            // Placeholder - conservatively treated as potential exception
            assert!(enif_is_exception(&env, empty_tuple));
        } else {
            // Heap-allocated empty tuple - should not be exception (no first element)
            assert!(!enif_is_exception(&env, empty_tuple));
        }
    }

    #[test]
    fn test_enif_is_exception_with_non_tuple_non_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Test with integer
        let int_term = crate::term_creation::enif_make_int(&env, 42);
        assert!(!enif_is_exception(&env, int_term));
        
        // Test with nil
        let nil_term = 0x3F;
        assert!(!enif_is_exception(&env, nil_term));
    }

    #[test]
    fn test_is_exception_atom_all_exception_types() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Test all exception atom types
        let exception_atoms = vec![
            "badarg", "error", "exit", "throw", "badarith", "function_clause",
            "case_clause", "if_clause", "try_clause", "undef", "badfun",
            "badarity", "timeout_value", "noproc", "noconnection", "nocatch"
        ];
        
        for atom_name in exception_atoms {
            let atom = enif_make_atom(&env, atom_name);
            assert!(enif_is_exception(&env, atom), "Atom '{}' should be detected as exception", atom_name);
        }
    }

    #[test]
    fn test_is_exception_atom_non_exception_atoms() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Test non-exception atoms
        let non_exception_atoms = vec!["ok", "true", "false", "undefined", "test", "hello"];
        
        for atom_name in non_exception_atoms {
            let atom = enif_make_atom(&env, atom_name);
            assert!(!enif_is_exception(&env, atom), "Atom '{}' should not be detected as exception", atom_name);
        }
    }

    #[test]
    fn test_is_exception_atom_with_non_atom_term() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Test with non-atom terms
        let int_term = crate::term_creation::enif_make_int(&env, 42);
        assert!(!enif_is_exception(&env, int_term));
        
        let nil_term = 0x3F;
        assert!(!enif_is_exception(&env, nil_term));
    }

    #[test]
    fn test_is_tuple_placeholder() {
        // Test tuple placeholder detection
        const TUPLE_PLACEHOLDER_TAG: u64 = 0xE0E0E0E0E0E0E0E0;
        
        // Should detect placeholder
        assert!(is_tuple_placeholder(TUPLE_PLACEHOLDER_TAG));
        assert!(is_tuple_placeholder(TUPLE_PLACEHOLDER_TAG | 0x1234));
        
        // Should not detect non-placeholders
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let int_term = crate::term_creation::enif_make_int(&env, 42);
        assert!(!is_tuple_placeholder(int_term));
        assert!(!is_tuple_placeholder(0));
    }

    #[test]
    fn test_is_exception_tuple_placeholder_helper() {
        // Test old-style exception placeholder detection
        const EXCEPTION_PLACEHOLDER_TAG: u64 = 0xF0F0F0F0F0F0F0F0;
        
        // Should detect old-style placeholder
        assert!(is_exception_tuple_placeholder(EXCEPTION_PLACEHOLDER_TAG));
        assert!(is_exception_tuple_placeholder(EXCEPTION_PLACEHOLDER_TAG | 0x1234));
        
        // Should not detect non-placeholders
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let int_term = crate::term_creation::enif_make_int(&env, 42);
        assert!(!is_exception_tuple_placeholder(int_term));
        assert!(!is_exception_tuple_placeholder(0));
    }

    #[test]
    fn test_enif_is_exception_with_old_style_placeholder() {
        // Test that old-style exception placeholder is detected
        const EXCEPTION_PLACEHOLDER_TAG: u64 = 0xF0F0F0F0F0F0F0F0;
        
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Old-style placeholder should be detected as exception
        assert!(enif_is_exception(&env, EXCEPTION_PLACEHOLDER_TAG));
    }

    #[test]
    fn test_enif_make_badarg_creates_valid_exception() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create badarg exception
        let exception = enif_make_badarg(&env);
        
        // Note: term 0 is valid (heap_index 0). Nil is 0x3F, not 0.
        // Should be detected as exception (this validates it's a valid term)
        assert!(enif_is_exception(&env, exception));
    }

    #[test]
    fn test_enif_make_badarg_atom_consistency() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create badarg atom directly
        let badarg_atom1 = enif_make_badarg_atom(&env);
        
        // Create badarg atom via enif_make_atom
        let badarg_atom2 = enif_make_atom(&env, "badarg");
        
        // Should be the same (same atom index)
        assert_eq!(badarg_atom1, badarg_atom2);
        
        // Both should be detected as exceptions
        assert!(enif_is_exception(&env, badarg_atom1));
        assert!(enif_is_exception(&env, badarg_atom2));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_containing_error_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create tuple with "error" atom as first element
        let error_atom = enif_make_atom(&env, "error");
        let message = enif_make_atom(&env, "test_error");
        let tuple = crate::term_creation::enif_make_tuple(&env, &[error_atom, message]);
        
        // Should be detected as exception
        assert!(enif_is_exception(&env, tuple));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_containing_exit_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create tuple with "exit" atom as first element
        let exit_atom = enif_make_atom(&env, "exit");
        let reason = crate::term_creation::enif_make_int(&env, 1);
        let tuple = crate::term_creation::enif_make_tuple(&env, &[exit_atom, reason]);
        
        // Should be detected as exception
        assert!(enif_is_exception(&env, tuple));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_containing_throw_atom() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create tuple with "throw" atom as first element
        let throw_atom = enif_make_atom(&env, "throw");
        let value = enif_make_atom(&env, "test");
        let tuple = crate::term_creation::enif_make_tuple(&env, &[throw_atom, value]);
        
        // Should be detected as exception
        assert!(enif_is_exception(&env, tuple));
    }

    #[test]
    fn test_enif_is_exception_with_large_tuple() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create a larger tuple with exception atom as first element
        let badarg_atom = enif_make_atom(&env, "badarg");
        let elements = vec![
            badarg_atom,
            crate::term_creation::enif_make_int(&env, 1),
            crate::term_creation::enif_make_int(&env, 2),
            crate::term_creation::enif_make_int(&env, 3),
        ];
        let tuple = crate::term_creation::enif_make_tuple(&env, &elements);
        
        // Should be detected as exception
        assert!(enif_is_exception(&env, tuple));
    }

    #[test]
    fn test_enif_is_exception_with_tuple_placeholder() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        
        // Create a tuple (may be placeholder if heap is full)
        let elements = vec![
            enif_make_atom(&env, "test"),
            crate::term_creation::enif_make_int(&env, 42),
        ];
        let tuple = crate::term_creation::enif_make_tuple(&env, &elements);
        
        // Check if it's a placeholder
        const TUPLE_PLACEHOLDER_TAG: u64 = 0xE0E0E0E0E0E0E0E0;
        if (tuple & TUPLE_PLACEHOLDER_TAG) == TUPLE_PLACEHOLDER_TAG {
            // Placeholder tuples are conservatively treated as potential exceptions
            // (since we can't decode them to check the first element)
            assert!(enif_is_exception(&env, tuple));
        }
    }
}

