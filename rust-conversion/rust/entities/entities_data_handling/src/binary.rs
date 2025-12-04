//! Binary Operations Module
//!
//! Provides binary data handling for Erlang terms. Binaries are sequences of bytes
//! that represent arbitrary data in the Erlang runtime system.
//!
//! ## Overview
//!
//! The `Binary` type represents Erlang binaries, which are immutable sequences of
//! bytes. Binaries are used throughout the Erlang runtime for storing arbitrary data,
//! including strings, serialized terms, network packets, and more.
//!
//! This module provides a basic binary data structure. For bit-aligned operations
//! (bitstrings), see the [`bits`](super::bits/index.html) module.
//!
//! ## Examples
//!
//! ```rust
//! use entities_data_handling::binary::Binary;
//!
//! // Create a binary from byte data
//! let data = vec![1, 2, 3, 4, 5];
//! let binary = Binary::new(data.clone());
//!
//! // Access the binary data
//! let retrieved = binary.data();
//! assert_eq!(retrieved, &data);
//! ```
//!
//! ## See Also
//!
//! - [`bits`](super::bits/index.html): Bit-level operations for bitstrings
//! - [`term_hashing`](super::term_hashing/index.html): Hash functions for binary terms

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

/// Binary data structure for Erlang binaries
///
/// Represents an immutable sequence of bytes. Binaries are a fundamental data type
/// in Erlang used for storing arbitrary byte data. Unlike lists, binaries are stored
/// contiguously in memory, making them efficient for large data.
///
/// ## Examples
///
/// ```rust
/// use entities_data_handling::binary::Binary;
///
/// // Create a binary from byte data
/// let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
/// let binary = Binary::new(data.clone());
///
/// // Access the binary data
/// let retrieved = binary.data();
/// assert_eq!(retrieved, &data);
/// ```
///
/// ## See Also
///
/// - [`bits`](super::bits/index.html): For bit-aligned operations (bitstrings)
/// - [`Term::Binary`](super::term_hashing::Term::Binary): Term representation of binaries
pub struct Binary {
    data: Vec<u8>,
}

impl Binary {
    /// Create a new binary from a byte vector
    ///
    /// Takes ownership of the byte vector and wraps it in a `Binary` structure.
    ///
    /// # Arguments
    /// * `data` - The byte data to store in the binary
    ///
    /// # Returns
    /// A new `Binary` instance containing the provided data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::binary::Binary;
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let binary = Binary::new(data);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a reference to the binary data
    ///
    /// Returns a slice reference to the underlying byte data. This allows
    /// read-only access to the binary contents without copying.
    ///
    /// # Returns
    /// A slice reference to the binary data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_data_handling::binary::Binary;
    ///
    /// let binary = Binary::new(vec![1, 2, 3, 4]);
    /// let data = binary.data();
    /// assert_eq!(data, &[1, 2, 3, 4]);
    /// ```
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_creation() {
        let data = vec![1, 2, 3, 4];
        let binary = Binary::new(data.clone());
        assert_eq!(binary.data(), &data);
    }
}

