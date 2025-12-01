//! Hash Utilities
//!
//! Provides hashing utility functions based on hash.c and safe_hash.c.
//! These utilities handle various hashing operations.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash utilities for various hashing operations
pub struct HashUtils;

impl HashUtils {
    /// Calculate a hash value for a value that implements Hash
    ///
    /// # Arguments
    /// * `value` - Value to hash
    ///
    /// # Returns
    /// Hash value
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::HashUtils;
    ///
    /// let hash1 = HashUtils::hash(&42);
    /// let hash2 = HashUtils::hash(&42);
    /// assert_eq!(hash1, hash2);
    /// ```
    pub fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate a hash value for a string
    ///
    /// # Arguments
    /// * `s` - String to hash
    ///
    /// # Returns
    /// Hash value
    pub fn hash_string(s: &str) -> u64 {
        Self::hash(&s)
    }

    /// Calculate a hash value for bytes
    ///
    /// # Arguments
    /// * `bytes` - Bytes to hash
    ///
    /// # Returns
    /// Hash value
    pub fn hash_bytes(bytes: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate a hash value for an integer
    ///
    /// # Arguments
    /// * `value` - Integer to hash
    ///
    /// # Returns
    /// Hash value
    pub fn hash_int(value: i64) -> u64 {
        Self::hash(&value)
    }

    /// Combine two hash values
    ///
    /// # Arguments
    /// * `hash1` - First hash
    /// * `hash2` - Second hash
    ///
    /// # Returns
    /// Combined hash
    pub fn combine_hashes(hash1: u64, hash2: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash1.hash(&mut hasher);
        hash2.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate a simple hash for a string (djb2 algorithm variant)
    ///
    /// # Arguments
    /// * `s` - String to hash
    ///
    /// # Returns
    /// Hash value
    pub fn simple_string_hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let hash1 = HashUtils::hash(&42);
        let hash2 = HashUtils::hash(&42);
        assert_eq!(hash1, hash2);
        
        let hash3 = HashUtils::hash(&43);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_string() {
        let hash1 = HashUtils::hash_string("hello");
        let hash2 = HashUtils::hash_string("hello");
        assert_eq!(hash1, hash2);
        
        let hash3 = HashUtils::hash_string("world");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_bytes() {
        let bytes = b"hello";
        let hash1 = HashUtils::hash_bytes(bytes);
        let hash2 = HashUtils::hash_bytes(bytes);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_int() {
        let hash1 = HashUtils::hash_int(42);
        let hash2 = HashUtils::hash_int(42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_combine_hashes() {
        let hash1 = HashUtils::hash_string("hello");
        let hash2 = HashUtils::hash_string("world");
        let combined = HashUtils::combine_hashes(hash1, hash2);
        assert_ne!(combined, hash1);
        assert_ne!(combined, hash2);
    }

    #[test]
    fn test_simple_string_hash() {
        let hash1 = HashUtils::simple_string_hash("hello");
        let hash2 = HashUtils::simple_string_hash("hello");
        assert_eq!(hash1, hash2);
    }
}

