//! Atom Table Management Module
//!
//! Provides atom table operations:
//! - Atom creation and lookup
//! - Atom table management
//! - Encoding support (7-bit ASCII, Latin1, UTF-8)
//!
//! Based on atom.c

use std::sync::RwLock;
use std::collections::HashMap;

/// Atom encoding types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomEncoding {
    /// 7-bit ASCII encoding
    SevenBitAscii,
    /// Latin1 encoding
    Latin1,
    /// UTF-8 encoding
    Utf8,
}

/// Maximum number of characters in an atom
pub const MAX_ATOM_CHARACTERS: usize = 255;

/// Maximum atom size limit in bytes
pub const MAX_ATOM_SZ_LIMIT: usize = 1024;

/// Atom representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Atom {
    /// UTF-8 encoded name
    name: Vec<u8>,
    /// Number of Latin1 characters (if applicable)
    latin1_chars: i16,
    /// Index in atom table
    index: usize,
}

/// Atom table for managing all atoms in the system
pub struct AtomTable {
    /// Map from atom name to index
    atoms: RwLock<HashMap<Vec<u8>, usize>>,
    /// Reverse map from index to atom name
    index_to_name: RwLock<Vec<Option<Vec<u8>>>>,
    /// Current number of atoms
    entries: RwLock<usize>,
    /// Maximum number of atoms
    limit: usize,
}

impl AtomTable {
    /// Create a new atom table
    pub fn new(limit: usize) -> Self {
        Self {
            atoms: RwLock::new(HashMap::new()),
            index_to_name: RwLock::new(Vec::new()),
            entries: RwLock::new(0),
            limit,
        }
    }

    /// Get or create an atom by name
    ///
    /// # Arguments
    /// * `name` - Atom name bytes
    /// * `encoding` - Encoding type
    /// * `truncate` - Whether to truncate if too long
    ///
    /// # Returns
    /// Index of the atom, or error code if failed
    pub fn put_index(
        &self,
        name: &[u8],
        encoding: AtomEncoding,
        truncate: bool,
    ) -> Result<usize, AtomError> {
        // Validate length based on encoding
        let (validated_name, validated_len) = self.validate_atom_name(name, encoding, truncate)?;

        // Check if atom already exists
        {
            let atoms = self.atoms.read().unwrap();
            if let Some(&index) = atoms.get(&validated_name) {
                return Ok(index);
            }
        }

        // Create new atom
        let mut atoms = self.atoms.write().unwrap();
        let mut index_to_name = self.index_to_name.write().unwrap();
        let mut entries = self.entries.write().unwrap();

        if *entries >= self.limit {
            return Err(AtomError::TableFull);
        }

        let index = *entries;
        atoms.insert(validated_name.clone(), index);
        if index >= index_to_name.len() {
            index_to_name.resize(index + 1, None);
        }
        index_to_name[index] = Some(validated_name);
        *entries += 1;

        Ok(index)
    }

    /// Get atom by name
    ///
    /// # Arguments
    /// * `name` - Atom name bytes
    /// * `encoding` - Encoding type
    ///
    /// # Returns
    /// Some(index) if found, None otherwise
    pub fn get(&self, name: &[u8], encoding: AtomEncoding) -> Option<usize> {
        let validated_name = self.validate_atom_name(name, encoding, false).ok()?.0;
        let atoms = self.atoms.read().unwrap();
        atoms.get(&validated_name).copied()
    }

    /// Get atom name by index
    ///
    /// # Arguments
    /// * `index` - Atom index
    ///
    /// # Returns
    /// Some(name bytes) if found, None otherwise
    pub fn get_name(&self, index: usize) -> Option<Vec<u8>> {
        let index_to_name = self.index_to_name.read().unwrap();
        index_to_name.get(index)?.clone()
    }

    /// Get number of atoms in table
    pub fn size(&self) -> usize {
        *self.entries.read().unwrap()
    }

    fn validate_atom_name(
        &self,
        name: &[u8],
        encoding: AtomEncoding,
        truncate: bool,
    ) -> Result<(Vec<u8>, usize), AtomError> {
        match encoding {
            AtomEncoding::SevenBitAscii => {
                // Verify 7-bit ASCII
                for &byte in name {
                    if byte & 0x80 != 0 {
                        return Err(AtomError::InvalidEncoding);
                    }
                }
                let len = name.len().min(MAX_ATOM_CHARACTERS);
                if len < name.len() && !truncate {
                    return Err(AtomError::TooLong);
                }
                Ok((name[..len].to_vec(), len))
            }
            AtomEncoding::Latin1 => {
                let len = name.len().min(MAX_ATOM_CHARACTERS);
                if len < name.len() && !truncate {
                    return Err(AtomError::TooLong);
                }
                // Convert Latin1 to UTF-8
                let utf8 = latin1_to_utf8(&name[..len]);
                Ok((utf8, len))
            }
            AtomEncoding::Utf8 => {
                // Validate UTF-8 encoding
                let validated = validate_utf8(name, truncate)?;
                Ok(validated)
            }
        }
    }
}

/// Atom operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomError {
    /// Atom name too long
    TooLong,
    /// Invalid encoding
    InvalidEncoding,
    /// Atom table full
    TableFull,
}

fn latin1_to_utf8(latin1: &[u8]) -> Vec<u8> {
    // Simple Latin1 to UTF-8 conversion
    // Characters 0-127 map directly, 128-255 become 2-byte UTF-8
    let mut utf8 = Vec::with_capacity(latin1.len() * 2);
    for &byte in latin1 {
        if byte < 0x80 {
            utf8.push(byte);
        } else {
            utf8.push(0xC0 | (byte >> 6));
            utf8.push(0x80 | (byte & 0x3F));
        }
    }
    utf8
}

fn validate_utf8(bytes: &[u8], truncate: bool) -> Result<(Vec<u8>, usize), AtomError> {
    // TODO: Implement full UTF-8 validation
    // For now, basic length check
    if bytes.len() > MAX_ATOM_SZ_LIMIT && !truncate {
        return Err(AtomError::TooLong);
    }
    let len = bytes.len().min(MAX_ATOM_SZ_LIMIT);
    Ok((bytes[..len].to_vec(), len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_table_creation() {
        let table = AtomTable::new(1000);
        assert_eq!(table.size(), 0);
    }

    #[test]
    fn test_atom_put_get() {
        let table = AtomTable::new(1000);
        let name = b"test_atom";
        let index = table
            .put_index(name, AtomEncoding::SevenBitAscii, false)
            .unwrap();
        assert_eq!(table.get(name, AtomEncoding::SevenBitAscii), Some(index));
    }

    #[test]
    fn test_atom_duplicate() {
        let table = AtomTable::new(1000);
        let name = b"duplicate";
        let index1 = table
            .put_index(name, AtomEncoding::SevenBitAscii, false)
            .unwrap();
        let index2 = table
            .put_index(name, AtomEncoding::SevenBitAscii, false)
            .unwrap();
        assert_eq!(index1, index2);
    }

    #[test]
    fn test_atom_get_name() {
        let table = AtomTable::new(1000);
        let name = b"named_atom";
        let index = table
            .put_index(name, AtomEncoding::SevenBitAscii, false)
            .unwrap();
        assert_eq!(table.get_name(index), Some(name.to_vec()));
    }
}

