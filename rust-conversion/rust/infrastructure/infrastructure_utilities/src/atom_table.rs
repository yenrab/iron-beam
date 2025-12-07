//! Atom Table Module
//!
//! Provides global atom table functionality for managing atoms system-wide.
//! Based on the pattern used in process_table.rs - provides a singleton
//! atom table that can be accessed from anywhere in the system.
//!
//! The atom table is a fundamental data structure in Erlang that stores all atom
//! names in the system. Atoms are unique identifiers that can be efficiently
//! compared by index rather than by string comparison.

use entities_data_handling::AtomTable;

/// Global atom table instance
///
/// This provides a singleton atom table that can be accessed from
/// anywhere in the system. In a full implementation, this would be
/// initialized during system startup with predefined atoms.
static GLOBAL_ATOM_TABLE: std::sync::OnceLock<AtomTable> = std::sync::OnceLock::new();

/// Get the global atom table instance
///
/// # Returns
/// Reference to the global atom table
///
/// # Examples
/// ```
/// use infrastructure_utilities::atom_table::get_global_atom_table;
/// use entities_data_handling::AtomEncoding;
///
/// let table = get_global_atom_table();
/// let index = table.put_index(b"my_atom", AtomEncoding::SevenBitAscii, false).unwrap();
/// ```
pub fn get_global_atom_table() -> &'static AtomTable {
    GLOBAL_ATOM_TABLE.get_or_init(|| {
        // Default limit of 1,048,576 atoms (2^20) - matches Erlang's default
        AtomTable::new(1_048_576)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities_data_handling::AtomEncoding;

    #[test]
    fn test_global_atom_table() {
        let table = get_global_atom_table();
        let index1 = table.put_index(b"test_atom", AtomEncoding::SevenBitAscii, false).unwrap();
        let index2 = table.put_index(b"test_atom", AtomEncoding::SevenBitAscii, false).unwrap();
        assert_eq!(index1, index2);
    }

    #[test]
    fn test_global_atom_table_singleton() {
        let table1 = get_global_atom_table();
        let table2 = get_global_atom_table();
        // Should return the same reference
        assert!(std::ptr::eq(table1, table2));
    }
}

