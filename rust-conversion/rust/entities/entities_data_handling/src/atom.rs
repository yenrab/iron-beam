//! Atom Table Management Module
//!
//! Provides comprehensive atom table operations for the Erlang/OTP runtime system.
//! Atoms are interned strings that are stored in a global table, allowing efficient
//! comparison and storage. This module provides thread-safe atom table management
//! with support for multiple encoding formats.
//!
//! ## Overview
//!
//! The atom table is a fundamental data structure in Erlang that stores all atom
//! names in the system. Atoms are unique identifiers that can be efficiently
//! compared by index rather than by string comparison. This module provides:
//!
//! - **Atom Creation and Lookup**: Create new atoms or look up existing ones by name
//! - **Thread-Safe Operations**: All operations use `RwLock` for concurrent access
//! - **Multiple Encoding Support**: Handles 7-bit ASCII, Latin1, and UTF-8 encodings
//! - **Validation**: Validates atom names according to encoding rules and length limits
//! - **Encoding Conversion**: Automatically converts Latin1 to UTF-8 for internal storage
//!
//! ## Encoding Support
//!
//! The module supports three encoding formats:
//!
//! - **7-bit ASCII**: Only characters in the range 0x00-0x7F are allowed
//! - **Latin1**: ISO-8859-1 encoding, automatically converted to UTF-8 for storage
//! - **UTF-8**: Full Unicode support with validation of UTF-8 sequences
//!
//! ## Limits
//!
//! - Maximum characters per atom: 255 (`MAX_ATOM_CHARACTERS`)
//! - Maximum bytes per atom: 1024 (`MAX_ATOM_SZ_LIMIT`)
//! - Maximum atoms in table: Configurable via `AtomTable::new(limit)`
//!
//! ## Examples
//!
//! ```rust
//! use entities_data_handling::{AtomTable, AtomEncoding};
//!
//! // Create an atom table
//! let table = AtomTable::new(1000);
//!
//! // Create an atom with 7-bit ASCII encoding
//! let index = table.put_index(b"my_atom", AtomEncoding::SevenBitAscii, false).unwrap();
//!
//! // Look up the atom
//! let found_index = table.get(b"my_atom", AtomEncoding::SevenBitAscii);
//! assert_eq!(found_index, Some(index));
//!
//! // Get the atom name back
//! let name = table.get_name(index);
//! assert_eq!(name, Some(b"my_atom".to_vec()));
//! ```
//!
//! ## See Also
//!
//! - [`term_hashing`](super::term_hashing/index.html): Hash functions that work with atom indices
//! - [`map`](super::map/index.html): Map operations that use atoms as keys

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
 */

use std::sync::RwLock;
use std::collections::HashMap;

/// Atom encoding types
///
/// Specifies the character encoding format for atom names. The encoding affects
/// how atom names are validated and stored internally. All atom names are
/// ultimately stored as UTF-8 in the atom table, but the encoding parameter
/// determines the validation rules and conversion process.
///
/// # Variants
///
/// - **SevenBitAscii**: Only characters in the range 0x00-0x7F are allowed.
///   This is the most restrictive encoding and ensures compatibility with
///   older systems that only support ASCII.
///
/// - **Latin1**: ISO-8859-1 encoding. Characters in the range 0x00-0xFF are
///   allowed. Characters 0x80-0xFF are automatically converted to UTF-8
///   (2-byte sequences) for internal storage.
///
/// - **Utf8**: Full Unicode support. Valid UTF-8 sequences are accepted,
///   including multi-byte characters. The encoding is validated to ensure
///   it's well-formed UTF-8, rejecting overlong encodings, surrogate pairs,
///   and invalid sequences.
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::{AtomTable, AtomEncoding};
///
/// let table = AtomTable::new(1000);
///
/// // 7-bit ASCII: only basic characters
/// let _ = table.put_index(b"hello", AtomEncoding::SevenBitAscii, false);
///
/// // Latin1: extended characters allowed
/// let _ = table.put_index(&[0xC4, 0xE5], AtomEncoding::Latin1, false);
///
/// // UTF-8: full Unicode support
/// let _ = table.put_index("世界".as_bytes(), AtomEncoding::Utf8, false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomEncoding {
    /// 7-bit ASCII encoding (characters 0x00-0x7F only)
    SevenBitAscii,
    /// Latin1 encoding (ISO-8859-1, automatically converted to UTF-8)
    Latin1,
    /// UTF-8 encoding (full Unicode support with validation)
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
    /// This function provides the primary interface for atom management. If an atom
    /// with the given name already exists, it returns the existing index. Otherwise,
    /// it creates a new atom entry and returns its index.
    ///
    /// The function validates the atom name according to the specified encoding and
    /// enforces length limits. If truncation is enabled and the name exceeds limits,
    /// it will be truncated to fit. Otherwise, an error is returned.
    ///
    /// # Arguments
    /// * `name` - Atom name bytes (raw byte slice)
    /// * `encoding` - Encoding type (SevenBitAscii, Latin1, or Utf8)
    /// * `truncate` - Whether to truncate the name if it exceeds length limits
    ///
    /// # Returns
    /// * `Ok(usize)` - The atom index if successful
    /// * `Err(AtomError::TooLong)` - If the name is too long and truncation is disabled
    /// * `Err(AtomError::InvalidEncoding)` - If the encoding is invalid for the given bytes
    /// * `Err(AtomError::TableFull)` - If the atom table has reached its capacity limit
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::{AtomTable, AtomEncoding};
    ///
    /// let table = AtomTable::new(1000);
    ///
    /// // Create a new atom
    /// let index1 = table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
    ///
    /// // Get the same atom (returns existing index)
    /// let index2 = table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
    /// assert_eq!(index1, index2);
    ///
    /// // Create an atom with UTF-8 encoding
    /// let utf8_atom = table.put_index("世界".as_bytes(), AtomEncoding::Utf8, false).unwrap();
    /// ```
    ///
    /// # See Also
    ///
    /// - [`get`](Self::get): Look up an existing atom without creating it
    /// - [`get_name`](Self::get_name): Get the name of an atom by its index
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

    /// Get atom by name without creating it
    ///
    /// Looks up an existing atom in the table by its name. Unlike `put_index`,
    /// this function does not create a new atom if one doesn't exist.
    ///
    /// # Arguments
    /// * `name` - Atom name bytes to look up
    /// * `encoding` - Encoding type used to validate and normalize the name
    ///
    /// # Returns
    /// * `Some(usize)` - The atom index if found
    /// * `None` - If the atom doesn't exist or the encoding is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::{AtomTable, AtomEncoding};
    ///
    /// let table = AtomTable::new(1000);
    ///
    /// // Atom doesn't exist yet
    /// assert_eq!(table.get(b"nonexistent", AtomEncoding::SevenBitAscii), None);
    ///
    /// // Create the atom
    /// let index = table.put_index(b"my_atom", AtomEncoding::SevenBitAscii, false).unwrap();
    ///
    /// // Now we can look it up
    /// assert_eq!(table.get(b"my_atom", AtomEncoding::SevenBitAscii), Some(index));
    /// ```
    ///
    /// # See Also
    ///
    /// - [`put_index`](Self::put_index): Get or create an atom
    /// - [`get_name`](Self::get_name): Reverse lookup by index
    pub fn get(&self, name: &[u8], encoding: AtomEncoding) -> Option<usize> {
        let validated_name = self.validate_atom_name(name, encoding, false).ok()?.0;
        let atoms = self.atoms.read().unwrap();
        atoms.get(&validated_name).copied()
    }

    /// Get atom name by index
    ///
    /// Performs a reverse lookup to retrieve the atom name given its index.
    /// This is useful when you have an atom index (e.g., from a `Term::Atom`)
    /// and need to get the actual string representation.
    ///
    /// # Arguments
    /// * `index` - The atom index to look up
    ///
    /// # Returns
    /// * `Some(Vec<u8>)` - The atom name bytes if the index is valid
    /// * `None` - If the index is out of bounds or the atom doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::{AtomTable, AtomEncoding};
    ///
    /// let table = AtomTable::new(1000);
    ///
    /// // Create an atom and get its index
    /// let index = table.put_index(b"my_atom", AtomEncoding::SevenBitAscii, false).unwrap();
    ///
    /// // Retrieve the name by index
    /// let name = table.get_name(index);
    /// assert_eq!(name, Some(b"my_atom".to_vec()));
    ///
    /// // Invalid index returns None
    /// assert_eq!(table.get_name(9999), None);
    /// ```
    ///
    /// # See Also
    ///
    /// - [`get`](Self::get): Look up an atom by name
    /// - [`put_index`](Self::put_index): Create an atom and get its index
    pub fn get_name(&self, index: usize) -> Option<Vec<u8>> {
        let index_to_name = self.index_to_name.read().unwrap();
        index_to_name.get(index)?.clone()
    }

    /// Get the number of atoms in the table
    ///
    /// Returns the current count of atoms stored in the table. This count
    /// increases as new atoms are added via `put_index` and never decreases
    /// (atoms are never removed from the table).
    ///
    /// # Returns
    /// The number of atoms currently in the table
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::{AtomTable, AtomEncoding};
    ///
    /// let table = AtomTable::new(1000);
    /// assert_eq!(table.size(), 0);
    ///
    /// let _ = table.put_index(b"atom1", AtomEncoding::SevenBitAscii, false);
    /// assert_eq!(table.size(), 1);
    ///
    /// let _ = table.put_index(b"atom2", AtomEncoding::SevenBitAscii, false);
    /// assert_eq!(table.size(), 2);
    ///
    /// // Duplicate atom doesn't increase count
    /// let _ = table.put_index(b"atom1", AtomEncoding::SevenBitAscii, false);
    /// assert_eq!(table.size(), 2);
    /// ```
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
///
/// Represents errors that can occur during atom table operations.
///
/// # Variants
///
/// - **TooLong**: The atom name exceeds the maximum length limits. For character
///   count, the limit is `MAX_ATOM_CHARACTERS` (255). For byte count, the limit
///   is `MAX_ATOM_SZ_LIMIT` (1024 bytes). This error occurs when truncation is
///   disabled and the name is too long.
///
/// - **InvalidEncoding**: The provided bytes do not match the specified encoding
///   format. For example, a byte with the high bit set (0x80-0xFF) would be invalid
///   for `AtomEncoding::SevenBitAscii`, or an invalid UTF-8 sequence would be
///   rejected for `AtomEncoding::Utf8`.
///
/// - **TableFull**: The atom table has reached its maximum capacity. The capacity
///   is set when creating the table with `AtomTable::new(limit)`. Once the limit
///   is reached, no new atoms can be created.
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::{AtomTable, AtomEncoding};
/// use entities_data_handling::atom::AtomError;
///
/// let table = AtomTable::new(2); // Small limit for testing
///
/// // Fill the table
/// let _ = table.put_index(b"atom1", AtomEncoding::SevenBitAscii, false);
/// let _ = table.put_index(b"atom2", AtomEncoding::SevenBitAscii, false);
///
/// // Try to add one more - should fail
/// let result = table.put_index(b"atom3", AtomEncoding::SevenBitAscii, false);
/// assert_eq!(result, Err(AtomError::TableFull));
///
/// // Invalid encoding
/// let result = table.put_index(&[0x80], AtomEncoding::SevenBitAscii, false);
/// assert_eq!(result, Err(AtomError::InvalidEncoding));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomError {
    /// Atom name exceeds maximum length limits
    TooLong,
    /// Invalid encoding for the specified format
    InvalidEncoding,
    /// Atom table has reached its capacity limit
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
    // Validate UTF-8 encoding and count characters
    
    let mut pos = 0;
    let mut num_chars = 0;
    let mut validated_bytes = Vec::new();
    
    while pos < bytes.len() {
        // Check if we've exceeded byte limit
        if validated_bytes.len() >= MAX_ATOM_SZ_LIMIT {
            if !truncate {
                return Err(AtomError::TooLong);
            }
            break;
        }
        
        // Check if we've exceeded character limit
        if num_chars >= MAX_ATOM_CHARACTERS {
            if !truncate {
                return Err(AtomError::TooLong);
            }
            break;
        }
        
        let byte = bytes[pos];
        
        // Single byte character (ASCII): 0x00-0x7F
        if (byte & 0x80) == 0 {
            validated_bytes.push(byte);
            pos += 1;
            num_chars += 1;
        }
        // Two byte character: 0xC0-0xDF
        else if (byte & 0xE0) == 0xC0 {
            if pos + 1 >= bytes.len() {
                return Err(AtomError::InvalidEncoding);
            }
            
            let byte2 = bytes[pos + 1];
            
            // Check continuation byte: must be 0x80-0xBF
            if (byte2 & 0xC0) != 0x80 {
                return Err(AtomError::InvalidEncoding);
            }
            
            // Check for overlong encoding: reject 0xC0 and 0xC1
            // Valid range is 0xC2-0xDF
            if byte < 0xC2 {
                return Err(AtomError::InvalidEncoding);
            }
            
            validated_bytes.push(byte);
            validated_bytes.push(byte2);
            pos += 2;
            num_chars += 1;
        }
        // Three byte character: 0xE0-0xEF
        else if (byte & 0xF0) == 0xE0 {
            if pos + 2 >= bytes.len() {
                return Err(AtomError::InvalidEncoding);
            }
            
            let byte2 = bytes[pos + 1];
            let byte3 = bytes[pos + 2];
            
            // Check continuation bytes: must be 0x80-0xBF
            if (byte2 & 0xC0) != 0x80 || (byte3 & 0xC0) != 0x80 {
                return Err(AtomError::InvalidEncoding);
            }
            
            // Check for overlong encoding: 0xE0 must be followed by >= 0xA0
            if byte == 0xE0 && byte2 < 0xA0 {
                return Err(AtomError::InvalidEncoding);
            }
            
            // Check for surrogate pairs: 0xED must not be followed by >= 0xA0
            // Surrogates are in range 0xD800-0xDFFF, encoded as ED A0-BF 80-BF
            if (byte & 0x0F) == 0x0D && (byte2 & 0x20) != 0 {
                return Err(AtomError::InvalidEncoding);
            }
            
            validated_bytes.push(byte);
            validated_bytes.push(byte2);
            validated_bytes.push(byte3);
            pos += 3;
            num_chars += 1;
        }
        // Four byte character: 0xF0-0xF7
        else if (byte & 0xF8) == 0xF0 {
            if pos + 3 >= bytes.len() {
                return Err(AtomError::InvalidEncoding);
            }
            
            let byte2 = bytes[pos + 1];
            let byte3 = bytes[pos + 2];
            let byte4 = bytes[pos + 3];
            
            // Check continuation bytes: must be 0x80-0xBF
            if (byte2 & 0xC0) != 0x80 || (byte3 & 0xC0) != 0x80 || (byte4 & 0xC0) != 0x80 {
                return Err(AtomError::InvalidEncoding);
            }
            
            // Check for overlong encoding: 0xF0 must be followed by >= 0x90
            if byte == 0xF0 && byte2 < 0x90 {
                return Err(AtomError::InvalidEncoding);
            }
            
            // Check for code points > 0x10FFFF (invalid Unicode)
            // Code point = ((byte & 0x07) << 18) | ((byte2 & 0x3F) << 12) | ...
            // Maximum valid is 0x10FFFF = 0x1F 0x0F 0xFF
            // So: (byte & 0x07) > 0x04, or (byte & 0x07) == 0x04 && (byte2 & 0x3F) > 0x0F
            if (byte & 0x07) > 0x04 || ((byte & 0x07) == 0x04 && (byte2 & 0x3F) > 0x0F) {
                return Err(AtomError::InvalidEncoding);
            }
            
            validated_bytes.push(byte);
            validated_bytes.push(byte2);
            validated_bytes.push(byte3);
            validated_bytes.push(byte4);
            pos += 4;
            num_chars += 1;
        }
        // Invalid UTF-8 sequence
        else {
            return Err(AtomError::InvalidEncoding);
        }
    }
    
    // Return validated bytes and byte length
    // Note: Character count is validated internally (must be <= MAX_ATOM_CHARACTERS)
    let byte_len = validated_bytes.len();
    Ok((validated_bytes, byte_len))
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

    #[test]
    fn test_utf8_validation_valid() {
        let table = AtomTable::new(1000);
        
        // Valid ASCII
        assert!(table.put_index(b"hello", AtomEncoding::Utf8, false).is_ok());
        
        // Valid 2-byte UTF-8 (Latin-1 supplement)
        let latin1 = [0xC3, 0xA5]; // 'å' in UTF-8
        assert!(table.put_index(&latin1, AtomEncoding::Utf8, false).is_ok());
        
        // Valid 3-byte UTF-8
        let chinese = [0xE4, 0xB8, 0xAD]; // '中' in UTF-8
        assert!(table.put_index(&chinese, AtomEncoding::Utf8, false).is_ok());
        
        // Valid 4-byte UTF-8 (emoji)
        let emoji = [0xF0, 0x9F, 0x98, 0x80]; // '😀' in UTF-8
        assert!(table.put_index(&emoji, AtomEncoding::Utf8, false).is_ok());
    }

    #[test]
    fn test_utf8_validation_invalid_overlong() {
        let table = AtomTable::new(1000);
        
        // Overlong encoding: 'A' (0x41) encoded as 0xC1 0x81
        let overlong = [0xC1, 0x81];
        assert_eq!(
            table.put_index(&overlong, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Overlong encoding: 0xE0 0x80 0x80 (should be 0x00)
        let overlong2 = [0xE0, 0x80, 0x80];
        assert_eq!(
            table.put_index(&overlong2, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Overlong encoding: 0xF0 0x80 0x80 0x80
        let overlong3 = [0xF0, 0x80, 0x80, 0x80];
        assert_eq!(
            table.put_index(&overlong3, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_utf8_validation_invalid_incomplete() {
        let table = AtomTable::new(1000);
        
        // Incomplete 2-byte sequence
        let incomplete = [0xC3];
        assert_eq!(
            table.put_index(&incomplete, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Incomplete 3-byte sequence
        let incomplete2 = [0xE4, 0xB8];
        assert_eq!(
            table.put_index(&incomplete2, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Incomplete 4-byte sequence
        let incomplete3 = [0xF0, 0x9F, 0x98];
        assert_eq!(
            table.put_index(&incomplete3, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_utf8_validation_invalid_continuation() {
        let table = AtomTable::new(1000);
        
        // Invalid continuation byte
        let invalid = [0xC3, 0x40]; // 0x40 is not a valid continuation byte
        assert_eq!(
            table.put_index(&invalid, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_utf8_validation_invalid_surrogate() {
        let table = AtomTable::new(1000);
        
        // Surrogate pair: 0xED 0xA0 0x80 (U+D800)
        let surrogate = [0xED, 0xA0, 0x80];
        assert_eq!(
            table.put_index(&surrogate, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_utf8_validation_invalid_out_of_range() {
        let table = AtomTable::new(1000);
        
        // Code point > 0x10FFFF: 0xF5 0x80 0x80 0x80
        let out_of_range = [0xF5, 0x80, 0x80, 0x80];
        assert_eq!(
            table.put_index(&out_of_range, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Code point > 0x10FFFF: 0xF4 0x90 0x80 0x80
        let out_of_range2 = [0xF4, 0x90, 0x80, 0x80];
        assert_eq!(
            table.put_index(&out_of_range2, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_utf8_validation_character_limit() {
        let table = AtomTable::new(1000);
        
        // Create a string with exactly 255 characters (all ASCII)
        let long_name = vec![b'a'; MAX_ATOM_CHARACTERS];
        assert!(table.put_index(&long_name, AtomEncoding::Utf8, false).is_ok());
        
        // Create a string with 256 characters (exceeds limit)
        let too_long = vec![b'a'; MAX_ATOM_CHARACTERS + 1];
        assert_eq!(
            table.put_index(&too_long, AtomEncoding::Utf8, false),
            Err(AtomError::TooLong)
        );
    }

    #[test]
    fn test_utf8_validation_byte_limit() {
        let table = AtomTable::new(1000);
        
        // Create a string at byte limit (all 4-byte UTF-8 characters)
        // Each character is 4 bytes, so 255 characters = 1020 bytes
        let mut long_name = Vec::new();
        for _ in 0..MAX_ATOM_CHARACTERS {
            long_name.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]); // 😀
        }
        assert!(long_name.len() <= MAX_ATOM_SZ_LIMIT);
        assert!(table.put_index(&long_name, AtomEncoding::Utf8, false).is_ok());
    }

    #[test]
    fn test_utf8_validation_truncate() {
        let table = AtomTable::new(1000);
        
        // Create a string that exceeds character limit
        let too_long = vec![b'a'; MAX_ATOM_CHARACTERS + 10];
        
        // Without truncate, should fail
        assert_eq!(
            table.put_index(&too_long, AtomEncoding::Utf8, false),
            Err(AtomError::TooLong)
        );
        
        // With truncate, should succeed (but we truncate at character boundary)
        // Since all characters are ASCII (1 byte each), truncation is straightforward
        let result = table.put_index(&too_long, AtomEncoding::Utf8, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_utf8_validation_mixed_sequences() {
        let table = AtomTable::new(1000);
        
        // Mixed ASCII and multi-byte UTF-8
        let mixed = b"Hello\xC3\xA5\xE4\xB8\xAD\xF0\x9F\x98\x80World";
        assert!(table.put_index(mixed, AtomEncoding::Utf8, false).is_ok());
        
        // Verify we can retrieve it
        let index = table.put_index(mixed, AtomEncoding::Utf8, false).unwrap();
        let retrieved = table.get_name(index).unwrap();
        assert_eq!(retrieved, mixed);
    }

    #[test]
    fn test_table_full() {
        let table = AtomTable::new(2); // Small limit
        
        // Fill the table
        assert!(table.put_index(b"atom1", AtomEncoding::SevenBitAscii, false).is_ok());
        assert!(table.put_index(b"atom2", AtomEncoding::SevenBitAscii, false).is_ok());
        
        // Try to add one more - should fail
        assert_eq!(
            table.put_index(b"atom3", AtomEncoding::SevenBitAscii, false),
            Err(AtomError::TableFull)
        );
    }

    #[test]
    fn test_seven_bit_ascii_invalid_encoding() {
        let table = AtomTable::new(1000);
        
        // Byte with high bit set (invalid for 7-bit ASCII)
        let invalid = [0x80];
        assert_eq!(
            table.put_index(&invalid, AtomEncoding::SevenBitAscii, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Mixed valid and invalid
        let mixed = b"hello\x80world";
        assert_eq!(
            table.put_index(mixed, AtomEncoding::SevenBitAscii, false),
            Err(AtomError::InvalidEncoding)
        );
    }

    #[test]
    fn test_seven_bit_ascii_too_long() {
        let table = AtomTable::new(1000);
        
        // Create a string longer than MAX_ATOM_CHARACTERS
        let too_long = vec![b'a'; MAX_ATOM_CHARACTERS + 1];
        
        // Without truncate, should fail
        assert_eq!(
            table.put_index(&too_long, AtomEncoding::SevenBitAscii, false),
            Err(AtomError::TooLong)
        );
        
        // With truncate, should succeed
        assert!(table.put_index(&too_long, AtomEncoding::SevenBitAscii, true).is_ok());
    }

    #[test]
    fn test_latin1_encoding() {
        let table = AtomTable::new(1000);
        
        // Latin1 encoding with ASCII characters
        let ascii = b"hello";
        let index = table.put_index(ascii, AtomEncoding::Latin1, false).unwrap();
        assert_eq!(table.get(ascii, AtomEncoding::Latin1), Some(index));
        
        // Latin1 encoding with extended characters (128-255)
        let extended = [0xC4, 0xE5]; // Latin1 characters
        let index2 = table.put_index(&extended, AtomEncoding::Latin1, false).unwrap();
        assert_eq!(table.get(&extended, AtomEncoding::Latin1), Some(index2));
        
        // Verify conversion to UTF-8
        let name = table.get_name(index2).unwrap();
        // Should be converted to UTF-8 (2 bytes per extended char)
        assert_eq!(name.len(), 4); // 2 chars * 2 bytes each
    }

    #[test]
    fn test_latin1_too_long() {
        let table = AtomTable::new(1000);
        
        // Create a string longer than MAX_ATOM_CHARACTERS
        let too_long = vec![0xA5; MAX_ATOM_CHARACTERS + 1];
        
        // Without truncate, should fail
        assert_eq!(
            table.put_index(&too_long, AtomEncoding::Latin1, false),
            Err(AtomError::TooLong)
        );
        
        // With truncate, should succeed
        assert!(table.put_index(&too_long, AtomEncoding::Latin1, true).is_ok());
    }

    #[test]
    fn test_latin1_truncate() {
        let table = AtomTable::new(1000);
        
        // Create a string at the limit
        let at_limit = vec![0xA5; MAX_ATOM_CHARACTERS];
        assert!(table.put_index(&at_limit, AtomEncoding::Latin1, false).is_ok());
        
        // Create a string just over the limit with truncate
        let over_limit = vec![0xA5; MAX_ATOM_CHARACTERS + 5];
        let result = table.put_index(&over_limit, AtomEncoding::Latin1, true);
        assert!(result.is_ok());
        
        // Verify it was truncated
        let index = result.unwrap();
        let name = table.get_name(index).unwrap();
        // Should be truncated to MAX_ATOM_CHARACTERS characters
        // Each Latin1 char > 127 becomes 2 UTF-8 bytes
        assert_eq!(name.len(), MAX_ATOM_CHARACTERS * 2);
    }

    #[test]
    fn test_get_invalid_encoding() {
        let table = AtomTable::new(1000);
        
        // Try to get with invalid encoding
        let invalid = [0x80]; // High bit set
        assert_eq!(
            table.get(&invalid, AtomEncoding::SevenBitAscii),
            None
        );
        
        // Try to get with invalid UTF-8
        let invalid_utf8 = [0xC0, 0x80]; // Invalid UTF-8
        assert_eq!(
            table.get(&invalid_utf8, AtomEncoding::Utf8),
            None
        );
    }

    #[test]
    fn test_get_name_invalid_index() {
        let table = AtomTable::new(1000);
        
        // Try to get name for non-existent index
        assert_eq!(table.get_name(999), None);
        
        // Try to get name for index 0 when table is empty
        assert_eq!(table.get_name(0), None);
        
        // Add an atom and verify valid index works
        let index = table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
        assert_eq!(table.get_name(index), Some(b"test".to_vec()));
        
        // Try index just after the last one
        assert_eq!(table.get_name(index + 1), None);
    }

    #[test]
    fn test_utf8_validation_byte_limit_truncate() {
        let table = AtomTable::new(1000);
        
        // Create a string that exceeds byte limit
        let mut too_long = Vec::new();
        // Add enough 4-byte characters to exceed MAX_ATOM_SZ_LIMIT
        for _ in 0..(MAX_ATOM_SZ_LIMIT / 4 + 10) {
            too_long.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]);
        }
        
        // Without truncate, should fail
        assert_eq!(
            table.put_index(&too_long, AtomEncoding::Utf8, false),
            Err(AtomError::TooLong)
        );
        
        // With truncate, should succeed
        let result = table.put_index(&too_long, AtomEncoding::Utf8, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_utf8_validation_character_limit_at_boundary() {
        let table = AtomTable::new(1000);
        
        // Test exactly at character limit with multi-byte characters
        let mut at_limit = Vec::new();
        for _ in 0..MAX_ATOM_CHARACTERS {
            at_limit.extend_from_slice(&[0xC3, 0xA5]); // 2-byte UTF-8
        }
        assert!(table.put_index(&at_limit, AtomEncoding::Utf8, false).is_ok());
        
        // Test one character over limit
        let mut over_limit = at_limit.clone();
        over_limit.extend_from_slice(&[0xC3, 0xA5]);
        assert_eq!(
            table.put_index(&over_limit, AtomEncoding::Utf8, false),
            Err(AtomError::TooLong)
        );
    }

    #[test]
    fn test_utf8_validation_truncate_at_character_boundary() {
        let table = AtomTable::new(1000);
        
        // Create a string with multi-byte characters that exceeds limit
        let mut too_long = Vec::new();
        for _ in 0..(MAX_ATOM_CHARACTERS + 5) {
            too_long.extend_from_slice(&[0xE4, 0xB8, 0xAD]); // 3-byte UTF-8
        }
        
        // With truncate, should succeed and truncate at character boundary
        let result = table.put_index(&too_long, AtomEncoding::Utf8, true);
        assert!(result.is_ok());
        
        // Verify the truncated result
        let index = result.unwrap();
        let name = table.get_name(index).unwrap();
        // Should be truncated to MAX_ATOM_CHARACTERS characters
        assert_eq!(name.len(), MAX_ATOM_CHARACTERS * 3);
    }

    #[test]
    fn test_index_to_name_resize() {
        let table = AtomTable::new(1000);
        
        // Add atoms to trigger resize of index_to_name vector
        // First atom at index 0
        let index1 = table.put_index(b"atom1", AtomEncoding::SevenBitAscii, false).unwrap();
        assert_eq!(index1, 0);
        
        // Second atom at index 1
        let index2 = table.put_index(b"atom2", AtomEncoding::SevenBitAscii, false).unwrap();
        assert_eq!(index2, 1);
        
        // Verify both can be retrieved
        assert_eq!(table.get_name(0), Some(b"atom1".to_vec()));
        assert_eq!(table.get_name(1), Some(b"atom2".to_vec()));
    }

    #[test]
    fn test_seven_bit_ascii_exact_limit() {
        let table = AtomTable::new(1000);
        
        // Create a string with exactly MAX_ATOM_CHARACTERS
        let exact = vec![b'a'; MAX_ATOM_CHARACTERS];
        assert!(table.put_index(&exact, AtomEncoding::SevenBitAscii, false).is_ok());
        
        // Verify it can be retrieved
        let index = table.put_index(&exact, AtomEncoding::SevenBitAscii, false).unwrap();
        let retrieved = table.get_name(index).unwrap();
        assert_eq!(retrieved.len(), MAX_ATOM_CHARACTERS);
    }

    #[test]
    fn test_latin1_mixed_ascii_and_extended() {
        let table = AtomTable::new(1000);
        
        // Mix of ASCII and extended Latin1
        let mixed = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xC4, 0xE5, 0x6C, 0x6C, 0x6F];
        let index = table.put_index(&mixed, AtomEncoding::Latin1, false).unwrap();
        
        // Verify retrieval
        assert_eq!(table.get(&mixed, AtomEncoding::Latin1), Some(index));
        
        // Verify UTF-8 conversion
        let name = table.get_name(index).unwrap();
        // ASCII chars: 0x48,0x65,0x6C,0x6C,0x6F (5 bytes) + extended: 0xC4,0xE5 (4 bytes) + ASCII: 0x6C,0x6C,0x6F (3 bytes) = 12 bytes
        assert_eq!(name.len(), 12);
    }

    #[test]
    fn test_utf8_validation_edge_cases() {
        let table = AtomTable::new(1000);
        
        // Test empty string
        assert!(table.put_index(b"", AtomEncoding::Utf8, false).is_ok());
        
        // Test single byte (ASCII)
        assert!(table.put_index(b"A", AtomEncoding::Utf8, false).is_ok());
        
        // Test boundary 2-byte UTF-8 (0xC2 is minimum valid)
        let min_2byte = [0xC2, 0x80];
        assert!(table.put_index(&min_2byte, AtomEncoding::Utf8, false).is_ok());
        
        // Test boundary 3-byte UTF-8 (0xE0 with 0xA0)
        let min_3byte = [0xE0, 0xA0, 0x80];
        assert!(table.put_index(&min_3byte, AtomEncoding::Utf8, false).is_ok());
        
        // Test boundary 4-byte UTF-8 (0xF0 with 0x90)
        let min_4byte = [0xF0, 0x90, 0x80, 0x80];
        assert!(table.put_index(&min_4byte, AtomEncoding::Utf8, false).is_ok());
        
        // Test maximum valid 4-byte UTF-8 (0x10FFFF)
        let max_4byte = [0xF4, 0x8F, 0xBF, 0xBF];
        assert!(table.put_index(&max_4byte, AtomEncoding::Utf8, false).is_ok());
    }

    #[test]
    fn test_utf8_validation_invalid_start_byte() {
        let table = AtomTable::new(1000);
        
        // Invalid start bytes: 0xF8-0xFF (5+ byte sequences are invalid in UTF-8)
        let invalid_start = [0xF8, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(
            table.put_index(&invalid_start, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Invalid start byte: 0xF9
        let invalid_start2 = [0xF9, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(
            table.put_index(&invalid_start2, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Invalid start byte: 0xFF
        let invalid_start3 = [0xFF];
        assert_eq!(
            table.put_index(&invalid_start3, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
        
        // Invalid start byte: 0xFE
        let invalid_start4 = [0xFE, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(
            table.put_index(&invalid_start4, AtomEncoding::Utf8, false),
            Err(AtomError::InvalidEncoding)
        );
    }
}

