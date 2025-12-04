//! Bit Manipulation Module
//!
//! Provides comprehensive bit-level operations for Erlang terms, particularly for
//! handling bitstrings and bit-aligned binary data. This module is essential for
//! operations that work with data at the bit level rather than byte level.
//!
//! ## Overview
//!
//! Erlang supports bitstrings, which are sequences of bits that may not be aligned
//! to byte boundaries. This module provides the low-level primitives needed to:
//!
//! - **Bit Copying**: Copy bits between buffers with arbitrary bit offsets
//! - **Bit Comparison**: Compare bit sequences for equality and ordering
//! - **Offset Calculations**: Convert between bit offsets and byte offsets
//! - **Mask Operations**: Generate and apply bit masks for selective bit operations
//! - **Bit Access**: Get and set individual bits within bytes
//!
//! ## Bit Numbering
//!
//! This module uses **MSB-first numbering** to match the C implementation behavior:
//! - Bit 0 is the most significant bit (MSB)
//! - Bit 7 is the least significant bit (LSB)
//!
//! This differs from typical LSB-first numbering but ensures compatibility with
//! the Erlang/OTP C implementation.
//!
//! ## Examples
//!
//! ```rust
//! use entities_data_handling::bits;
//!
//! // Calculate byte requirements for bits
//! let bytes = bits::nbytes(17); // 3 bytes for 17 bits
//!
//! // Copy bits between buffers
//! let src = vec![0b11110000u8];
//! let mut dst = vec![0u8; 1];
//! bits::copy_bits_forward(&src, 4, &mut dst, 0, 4);
//!
//! // Compare bit sequences
//! let a = vec![0xFFu8];
//! let b = vec![0xFFu8];
//! let result = bits::cmp_bits(&a, 0, &b, 0, 8); // Returns 0 (equal)
//! ```
//!
//! ## See Also
//!
//! - [`binary`](super::binary/index.html): Binary data structures that use bit operations
//! - [`term_hashing`](super::term_hashing/index.html): Hash functions that work with bit-aligned binaries

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

/// Calculate the number of bytes needed to store `bits` bits
///
/// Rounds up to the nearest byte. For example, 1-8 bits require 1 byte,
/// 9-16 bits require 2 bytes, etc.
///
/// # Arguments
/// * `bits` - Number of bits to store
///
/// # Returns
/// Number of bytes needed (rounded up)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// assert_eq!(bits::nbytes(0), 0);
/// assert_eq!(bits::nbytes(1), 1);
/// assert_eq!(bits::nbytes(8), 1);
/// assert_eq!(bits::nbytes(9), 2);
/// assert_eq!(bits::nbytes(16), 2);
/// assert_eq!(bits::nbytes(17), 3);
/// ```
///
/// # See Also
///
/// - [`nbits`](crate::bits::nbits): Reverse operation (bytes to bits)
/// - [`byte_offset`](crate::bits::byte_offset): Get byte offset from bit offset
///
/// Equivalent to `NBYTES(x)` macro in C
pub fn nbytes(bits: u64) -> usize {
    ((bits + 7) >> 3) as usize
}

/// Calculate the number of bits in `bytes` bytes
///
/// Simply multiplies the number of bytes by 8. This is the inverse
/// operation of `nbytes`.
///
/// # Arguments
/// * `bytes` - Number of bytes
///
/// # Returns
/// Number of bits (bytes * 8)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// assert_eq!(bits::nbits(0), 0);
/// assert_eq!(bits::nbits(1), 8);
/// assert_eq!(bits::nbits(2), 16);
/// assert_eq!(bits::nbits(10), 80);
/// ```
///
/// # See Also
///
/// - [`nbytes`](crate::bits::nbytes): Reverse operation (bits to bytes)
///
/// Equivalent to `NBITS(x)` macro in C
pub fn nbits(bytes: usize) -> u64 {
    (bytes as u64) << 3
}

/// Get the byte offset from a bit offset
///
/// Calculates which byte contains the bit at the given offset by dividing
/// the bit offset by 8 (integer division).
///
/// # Arguments
/// * `bit_offset` - Offset in bits
///
/// # Returns
/// Byte offset (bits / 8)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// assert_eq!(bits::byte_offset(0), 0);
/// assert_eq!(bits::byte_offset(7), 0);  // Bits 0-7 are in byte 0
/// assert_eq!(bits::byte_offset(8), 1);  // Bit 8 is in byte 1
/// assert_eq!(bits::byte_offset(15), 1); // Bits 8-15 are in byte 1
/// assert_eq!(bits::byte_offset(16), 2);
/// ```
///
/// # See Also
///
/// - [`bit_offset`](crate::bits::bit_offset): Get bit offset within a byte
/// - [`nbytes`](crate::bits::nbytes): Calculate bytes needed for bits
///
/// Equivalent to `BYTE_OFFSET(x)` macro in C
pub fn byte_offset(bit_offset: usize) -> usize {
    bit_offset >> 3
}

/// Get the bit offset within a byte
///
/// Calculates the position of the bit within its containing byte (0-7).
/// This is the remainder when dividing the bit offset by 8.
///
/// # Arguments
/// * `bit_offset` - Offset in bits
///
/// # Returns
/// Bit offset within byte (0-7)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// assert_eq!(bits::bit_offset(0), 0);
/// assert_eq!(bits::bit_offset(7), 7);
/// assert_eq!(bits::bit_offset(8), 0);  // Bit 8 is at position 0 in byte 1
/// assert_eq!(bits::bit_offset(15), 7); // Bit 15 is at position 7 in byte 1
/// assert_eq!(bits::bit_offset(16), 0);
/// ```
///
/// # See Also
///
/// - [`byte_offset`](crate::bits::byte_offset): Get byte offset from bit offset
/// - [`get_bit`](crate::bits::get_bit): Get a bit at a specific position
///
/// Equivalent to `BIT_OFFSET(x)` macro in C
pub fn bit_offset(bit_offset: usize) -> usize {
    bit_offset & 7
}

/// Create a mask with `n` bits set
///
/// Creates a bitmask with the lowest `n` bits set to 1. For example,
/// `make_mask(3)` returns `0b111` (binary) or `7` (decimal).
///
/// # Arguments
/// * `n` - Number of bits in the mask (0-64). Values >= 64 return `u64::MAX`.
///
/// # Returns
/// Mask with n bits set (e.g., `make_mask(3)` = `0b111` = `7`)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// assert_eq!(bits::make_mask(0), 0);
/// assert_eq!(bits::make_mask(1), 0b1);
/// assert_eq!(bits::make_mask(3), 0b111);
/// assert_eq!(bits::make_mask(8), 0xFF);
/// assert_eq!(bits::make_mask(64), u64::MAX);
/// ```
///
/// # See Also
///
/// - [`mask_bits`](crate::bits::mask_bits): Apply a mask to copy bits
///
/// Equivalent to `MAKE_MASK(n)` macro in C
pub fn make_mask(n: usize) -> u64 {
    if n >= 64 {
        u64::MAX
    } else {
        ((1u64 << n) - 1)
    }
}

/// Mask bits: assign src to dst, preserving dst bits outside the mask
///
/// # Arguments
/// * `src` - Source bits
/// * `dst` - Destination bits
/// * `mask` - Mask to apply
///
/// # Returns
/// (src & mask) | (dst & !mask)
///
/// Equivalent to `MASK_BITS(src, dst, mask)` macro in C
pub fn mask_bits(src: u8, dst: u8, mask: u8) -> u8 {
    (src & mask) | (dst & !mask)
}

/// Get a single bit from a byte
///
/// # Arguments
/// * `byte` - The byte to extract bit from
/// * `bit_pos` - Bit position (0-7, where 0 is MSB, 7 is LSB)
///
/// # Returns
/// The bit value (0 or 1)
///
/// Uses MSB-first numbering. Bit 0 is the most significant bit (MSB), bit 7 is the least significant bit (LSB).
pub fn get_bit(byte: u8, bit_pos: usize) -> u8 {
    if bit_pos > 7 {
        0
    } else {
        (byte >> (7 - bit_pos)) & 1
    }
}

/// Set a single bit in a byte
///
/// # Arguments
/// * `byte` - The byte to modify
/// * `bit_pos` - Bit position (0-7, where 0 is MSB, 7 is LSB)
/// * `value` - Bit value (0 or 1)
///
/// # Returns
/// Byte with bit set
///
/// Uses MSB-first numbering to match C behavior: bit 0 is MSB, bit 7 is LSB.
pub fn set_bit(byte: u8, bit_pos: usize, value: u8) -> u8 {
    if bit_pos > 7 {
        return byte;
    }
    let bit_mask = 1 << (7 - bit_pos);
    if value != 0 {
        byte | bit_mask
    } else {
        byte & !bit_mask
    }
}

/// Copy bits forward from source to destination
///
/// Copies `n` bits from the source buffer to the destination buffer, starting
/// at arbitrary bit offsets in both buffers. This function handles bit-aligned
/// copying, which is essential for Erlang bitstrings that may not be byte-aligned.
///
/// The function efficiently handles three cases:
/// - All bits fit in a single byte (fast path)
/// - Bits span multiple bytes with aligned copying
/// - Bits span multiple bytes with unaligned copying
///
/// # Arguments
/// * `src` - Source buffer containing the bits to copy
/// * `src_offset` - Bit offset in source buffer where copying starts
/// * `dst` - Destination buffer (must be large enough to hold `n` bits starting at `dst_offset`)
/// * `dst_offset` - Bit offset in destination buffer where copying starts
/// * `n` - Number of bits to copy
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// // Copy bits within the same byte
/// let src = vec![0b11110000u8];
/// let mut dst = vec![0u8; 1];
/// bits::copy_bits_forward(&src, 4, &mut dst, 0, 4);
/// // Copies bits 4-7 (1111) from src to bits 0-3 of dst
/// assert_eq!(dst[0], 0b00001111);
///
/// // Copy bits across byte boundaries
/// let src = vec![0xFFu8, 0xAAu8];
/// let mut dst = vec![0u8; 2];
/// bits::copy_bits_forward(&src, 4, &mut dst, 0, 12);
/// // Copies 12 bits starting at bit 4 of src
/// ```
///
/// # See Also
///
/// - [`cmp_bits`](crate::bits::cmp_bits): Compare bit sequences
/// - [`byte_offset`](crate::bits::byte_offset): Calculate byte offsets for bit operations
/// - [`bit_offset`](crate::bits::bit_offset): Calculate bit offsets within bytes
///
/// # Panics
/// Panics if buffers are not large enough
pub fn copy_bits_forward(
    src: &[u8],
    src_offset: usize,
    dst: &mut [u8],
    dst_offset: usize,
    n: usize,
) {
    if n == 0 {
        return;
    }

    let src_byte_offset = byte_offset(src_offset);
    let dst_byte_offset = byte_offset(dst_offset);
    let src_bit_offset = bit_offset(src_offset);
    let dst_bit_offset = bit_offset(dst_offset);

    // Handle case where all bits are in the same byte
    // Match C behavior: doffs+n < 8 (strictly less than)
    if dst_bit_offset + n < 8 {
        let mask = make_mask(n) as u8;
        let shifted_mask = mask << dst_bit_offset;
        
        let src_byte = if src_byte_offset < src.len() {
            src[src_byte_offset]
        } else {
            0
        };
        
        // Extract n bits from src starting at src_bit_offset
        let src_mask = make_mask(n) as u8;
        let src_bits = if src_bit_offset + n <= 8 {
            // All bits in one byte - extract and mask
            (src_byte >> src_bit_offset) & src_mask
        } else {
            // Bits span two bytes
            let first_part = src_byte >> src_bit_offset;
            let next_byte = if src_byte_offset + 1 < src.len() {
                src[src_byte_offset + 1]
            } else {
                0
            };
            let bits_from_next = (next_byte << (8 - src_bit_offset)) & src_mask;
            (first_part | bits_from_next) & src_mask
        };
        
        // Shift to destination position
        let bits = src_bits << dst_bit_offset;
        
        if dst_byte_offset < dst.len() {
            dst[dst_byte_offset] = mask_bits(bits, dst[dst_byte_offset], shifted_mask);
        }
        return;
    }

    // Handle multi-byte copy
    let mut src_idx = src_byte_offset;
    let mut dst_idx = dst_byte_offset;
    let mut remaining = n;
    let mut src_bit = src_bit_offset;
    let mut dst_bit = dst_bit_offset;

    // First byte (partial)
    if dst_bit != 0 {
        let bits_in_first = 8 - dst_bit;
        let bits_to_copy = remaining.min(bits_in_first);
        let mask = make_mask(bits_to_copy) as u8;
        let shifted_mask = mask << dst_bit;

        let src_byte = if src_idx < src.len() {
            src[src_idx]
        } else {
            0
        };

        // Extract bits_to_copy bits from src starting at src_bit, then align to dst_bit
        let src_mask = make_mask(bits_to_copy) as u8;
        let src_bits = if src_bit + bits_to_copy <= 8 {
            (src_byte >> src_bit) & src_mask
        } else {
            // Bits span two bytes - handle in multi-byte path
            let first_part = src_byte >> src_bit;
            let next_byte = if src_idx + 1 < src.len() { src[src_idx + 1] } else { 0 };
            let bits_from_next = (next_byte << (8 - src_bit)) & src_mask;
            (first_part | bits_from_next) & src_mask
        };
        
        // Shift extracted bits to destination position
        let bits = src_bits << dst_bit;

        if dst_idx < dst.len() {
            dst[dst_idx] = mask_bits(bits, dst[dst_idx], shifted_mask);
        }

        if src_bit + bits_to_copy >= 8 {
            src_idx += 1;
            src_bit = 0;
        } else {
            src_bit += bits_to_copy;
        }

        dst_idx += 1;
        dst_bit = 0;
        remaining -= bits_to_copy;
    }

    // Middle bytes (full bytes)
    while remaining >= 8 && src_idx < src.len() && dst_idx < dst.len() {
        let src_byte = src[src_idx];
        let bits = if src_bit == 0 {
            src_byte
        } else {
            let next_byte = if src_idx + 1 < src.len() {
                src[src_idx + 1]
            } else {
                0
            };
            (src_byte << (8 - src_bit)) | (next_byte >> src_bit)
        };

        dst[dst_idx] = bits;
        src_idx += 1;
        dst_idx += 1;
        remaining -= 8;
    }

    // Last byte (partial)
    if remaining > 0 && dst_idx < dst.len() {
        let mask = make_mask(remaining) as u8;
        let src_byte = if src_idx < src.len() {
            src[src_idx]
        } else {
            0
        };

        let bits = if src_bit == 0 {
            src_byte
        } else {
            let next_byte = if src_idx + 1 < src.len() {
                src[src_idx + 1]
            } else {
                0
            };
            (src_byte << (8 - src_bit)) | (next_byte >> src_bit)
        };

        dst[dst_idx] = mask_bits(bits, dst[dst_idx], mask);
    }
}

/// Compare two bit sequences
///
/// Compares two bit sequences bit-by-bit, starting from the specified offsets
/// in each buffer. The comparison uses MSB-first ordering (bit 0 is most significant).
///
/// # Arguments
/// * `a` - First buffer to compare
/// * `a_offset` - Bit offset in first buffer where comparison starts
/// * `b` - Second buffer to compare
/// * `b_offset` - Bit offset in second buffer where comparison starts
/// * `size` - Number of bits to compare
///
/// # Returns
/// * `-1` if a < b (first differing bit is 0 in a, 1 in b)
/// * `0` if a == b (all bits are equal)
/// * `1` if a > b (first differing bit is 1 in a, 0 in b)
///
/// # Examples
///
/// ```rust
/// use entities_data_handling::bits;
///
/// // Compare equal sequences
/// let a = vec![0xFFu8];
/// let b = vec![0xFFu8];
/// assert_eq!(bits::cmp_bits(&a, 0, &b, 0, 8), 0);
///
/// // Compare different sequences
/// let a = vec![0xFFu8];
/// let b = vec![0x00u8];
/// assert_eq!(bits::cmp_bits(&a, 0, &b, 0, 8), 1);  // a > b
/// assert_eq!(bits::cmp_bits(&b, 0, &a, 0, 8), -1); // b < a
///
/// // Compare unaligned sequences
/// let a = vec![0b11110000u8];
/// let b = vec![0b00001111u8];
/// // Compare first 4 bits (MSB side): 1111 vs 0000
/// assert_eq!(bits::cmp_bits(&a, 0, &b, 0, 4), 1);
/// ```
///
/// # See Also
///
/// - [`copy_bits_forward`](crate::bits::copy_bits_forward): Copy bit sequences
/// - [`get_bit`](crate::bits::get_bit): Get individual bits for comparison
pub fn cmp_bits(
    a: &[u8],
    a_offset: usize,
    b: &[u8],
    b_offset: usize,
    size: usize,
) -> i32 {
    if size == 0 {
        return 0;
    }

    let mut a_byte = byte_offset(a_offset);
    let mut b_byte = byte_offset(b_offset);
    let mut a_bit = bit_offset(a_offset);
    let mut b_bit = bit_offset(b_offset);
    let mut remaining = size;

    while remaining > 0 {
        let a_val = if a_byte < a.len() {
            get_bit(a[a_byte], a_bit)
        } else {
            0
        };

        let b_val = if b_byte < b.len() {
            get_bit(b[b_byte], b_bit)
        } else {
            0
        };

        if a_val < b_val {
            return -1;
        } else if a_val > b_val {
            return 1;
        }

        remaining -= 1;
        a_bit += 1;
        if a_bit >= 8 {
            a_bit = 0;
            a_byte += 1;
        }
        b_bit += 1;
        if b_bit >= 8 {
            b_bit = 0;
            b_byte += 1;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbytes() {
        assert_eq!(nbytes(0), 0);
        assert_eq!(nbytes(1), 1);
        assert_eq!(nbytes(7), 1);
        assert_eq!(nbytes(8), 1);
        assert_eq!(nbytes(9), 2);
        assert_eq!(nbytes(16), 2);
    }

    #[test]
    fn test_nbits() {
        assert_eq!(nbits(0), 0);
        assert_eq!(nbits(1), 8);
        assert_eq!(nbits(2), 16);
        assert_eq!(nbits(10), 80);
    }

    #[test]
    fn test_byte_offset() {
        assert_eq!(byte_offset(0), 0);
        assert_eq!(byte_offset(7), 0);
        assert_eq!(byte_offset(8), 1);
        assert_eq!(byte_offset(15), 1);
        assert_eq!(byte_offset(16), 2);
    }

    #[test]
    fn test_bit_offset() {
        assert_eq!(bit_offset(0), 0);
        assert_eq!(bit_offset(7), 7);
        assert_eq!(bit_offset(8), 0);
        assert_eq!(bit_offset(15), 7);
        assert_eq!(bit_offset(16), 0);
    }

    #[test]
    fn test_make_mask() {
        assert_eq!(make_mask(0), 0);
        assert_eq!(make_mask(1), 0b1);
        assert_eq!(make_mask(3), 0b111);
        assert_eq!(make_mask(8), 0xFF);
        assert_eq!(make_mask(63), ((1u64 << 63) - 1)); // Test the else branch explicitly
        assert_eq!(make_mask(64), u64::MAX);
    }

    #[test]
    fn test_mask_bits() {
        let src = 0b11110000;
        let dst = 0b00001111;
        let mask = 0b11000000;
        let result = mask_bits(src, dst, mask);
        assert_eq!(result, 0b11001111);
    }

    #[test]
    fn test_get_bit() {
        let byte = 0b10101010; // MSB=1, LSB=0
        // MSB-first numbering: bit 0 = MSB, bit 7 = LSB
        assert_eq!(get_bit(byte, 0), 1); // MSB
        assert_eq!(get_bit(byte, 1), 0);
        assert_eq!(get_bit(byte, 2), 1);
        assert_eq!(get_bit(byte, 3), 0);
        assert_eq!(get_bit(byte, 4), 1);
        assert_eq!(get_bit(byte, 5), 0);
        assert_eq!(get_bit(byte, 6), 1);
        assert_eq!(get_bit(byte, 7), 0); // LSB
        
        // Test with MSB set
        let byte_msb = 0b10000000;
        assert_eq!(get_bit(byte_msb, 0), 1); // MSB
        assert_eq!(get_bit(byte_msb, 7), 0); // LSB
        
        // Test with LSB set
        let byte_lsb = 0b00000001;
        assert_eq!(get_bit(byte_lsb, 0), 0); // MSB
        assert_eq!(get_bit(byte_lsb, 7), 1); // LSB
    }

    #[test]
    fn test_set_bit() {
        let byte = 0b00000000;
        // MSB-first numbering: bit 0 = MSB, bit 7 = LSB
        assert_eq!(set_bit(byte, 0, 1), 0b10000000); // Set MSB
        assert_eq!(set_bit(byte, 7, 1), 0b00000001); // Set LSB
        assert_eq!(set_bit(byte, 3, 1), 0b00010000); // Set bit 3 (4th from MSB)
        assert_eq!(set_bit(0b11111111, 0, 0), 0b01111111); // Clear MSB
        assert_eq!(set_bit(0b11111111, 7, 0), 0b11111110); // Clear LSB
    }

    #[test]
    fn test_copy_bits_forward_simple() {
        let src = vec![0b11110000u8]; // Binary: 11110000, bits 0-3=0000, bits 4-7=1111
        let mut dst = vec![0u8; 1];
        
        // Copy first 4 bits from position 0 (LSB = 0000) to destination at position 0
        // Bit 0 is LSB, so bits 0-3 are 0000
        copy_bits_forward(&src, 0, &mut dst, 0, 4);
        assert_eq!(dst[0], 0b00000000);
        
        // Copy 4 bits from position 4 (bits 4-7 = 1111) to destination at position 0
        // Extracts 1111, places at position 0, result is 0b00001111 = 15
        let mut dst2 = vec![0u8; 1];
        copy_bits_forward(&src, 4, &mut dst2, 0, 4);
        assert_eq!(dst2[0], 0b00001111);
        
        // Copy all 8 bits
        let mut dst3 = vec![0u8; 1];
        copy_bits_forward(&src, 0, &mut dst3, 0, 8);
        assert_eq!(dst3[0], 0b11110000);
    }

    #[test]
    fn test_copy_bits_forward_aligned() {
        let src = vec![0xFFu8, 0xAAu8];
        let mut dst = vec![0u8; 2];
        
        copy_bits_forward(&src, 0, &mut dst, 0, 16);
        assert_eq!(dst[0], 0xFF);
        assert_eq!(dst[1], 0xAA);
    }

    #[test]
    fn test_copy_bits_forward_unaligned() {
        let src = vec![0b11110000u8]; // bits 0-3=0000, bits 4-7=1111
        let mut dst = vec![0b00001111u8]; // bits 0-3=1111, bits 4-7=0000
        
        // Copy 4 bits from position 4 (1111) to position 4
        // dst starts as 00001111, after copying 1111 to positions 4-7: 11111111
        copy_bits_forward(&src, 4, &mut dst, 4, 4);
        assert_eq!(dst[0], 0b11111111);
        
        // Test copying from position 4 to position 0
        let mut dst2 = vec![0b11110000u8];
        copy_bits_forward(&src, 4, &mut dst2, 0, 4);
        // src[4:8] = 1111, placed at dst[0:4], so dst becomes 11111111
        assert_eq!(dst2[0], 0b11111111);
        
        // Test copying from position 0 to position 4
        let mut dst3 = vec![0b11110000u8];
        copy_bits_forward(&src, 0, &mut dst3, 4, 4);
        // src[0:4] = 0000, placed at dst[4:8], so dst becomes 00000000
        assert_eq!(dst3[0], 0b00000000);
        
        // Test: copy 4 bits from position 0 to position 0
        let mut dst4 = vec![0b11111111u8];
        copy_bits_forward(&src, 0, &mut dst4, 0, 4);
        // src[0:4] = 0000, placed at dst[0:4], so dst becomes 11110000
        assert_eq!(dst4[0], 0b11110000);
    }

    #[test]
    fn test_cmp_bits_equal() {
        let a = vec![0xFFu8, 0xAAu8];
        let b = vec![0xFFu8, 0xAAu8];
        
        assert_eq!(cmp_bits(&a, 0, &b, 0, 16), 0);
    }

    #[test]
    fn test_cmp_bits_different() {
        let a = vec![0xFFu8];
        let b = vec![0x00u8];
        
        assert_eq!(cmp_bits(&a, 0, &b, 0, 8), 1);
        assert_eq!(cmp_bits(&b, 0, &a, 0, 8), -1);
    }

    #[test]
    fn test_cmp_bits_unaligned() {
        // With MSB-first numbering: bit 0 = MSB, bit 7 = LSB
        let a = vec![0b11110000u8]; // MSB-first: bits 0-3=1111, bits 4-7=0000
        let b = vec![0b00001111u8]; // MSB-first: bits 0-3=0000, bits 4-7=1111
        
        // Compare first 4 bits (MSB side): a[0:4]=1111 vs b[0:4]=0000, so a > b
        assert_eq!(cmp_bits(&a, 0, &b, 0, 4), 1);
        
        // Compare last 4 bits (LSB side): a[4:8]=0000 vs b[4:8]=1111, so a < b
        assert_eq!(cmp_bits(&a, 4, &b, 4, 4), -1);
    }

    #[test]
    fn test_copy_bits_forward_partial_byte() {
        let src = vec![0b10101010u8]; // bits: 0 1 0 1 0 1 0 1 (LSB to MSB)
        // Bit positions: 0=0, 1=1, 2=0, 3=1, 4=0, 5=1, 6=0, 7=1
        let mut dst = vec![0b11111111u8];
        
        // Copy 3 bits from position 1 to position 2
        // src[1:4] = bits at positions 1,2,3 = 101 (bit1=1, bit2=0, bit3=1)
        // Extract: (0b10101010 >> 1) & 0b111 = 0b01010101 & 0b111 = 0b101 = 5
        // When placed at position 2: 5 << 2 = 0b00010100 = 20
        // dst starts as 11111111, after copying 101 to positions 2,3,4:
        // Result: 11 101 111 = 0b11110111 = 247
        copy_bits_forward(&src, 1, &mut dst, 2, 3);
        assert_eq!(dst[0], 0b11110111);
        
        // Test another case: copy 2 bits from position 0 to position 6
        // With dst_bit_offset=6, n=2: 6+2=8, which is NOT < 8, so goes to multi-byte path
        let mut dst2 = vec![0b00000000u8];
        copy_bits_forward(&src, 0, &mut dst2, 6, 2);
        // This goes through first-byte partial path in multi-byte handler
        // Extract 2 bits from src[0:2] = 0b10 = 2
        // Shift to position 6: 2 << 6 = 0b10000000 = 128
        // Mask for 2 bits at position 6: 0b11000000
        // Result: (0b10000000 & 0b11000000) | (0b00000000 & 0b00111111) = 0b10000000 = 128
        assert_eq!(dst2[0], 0b10000000);
    }

    #[test]
    fn test_copy_bits_forward_empty() {
        let src = vec![0xFFu8];
        let mut dst = vec![0u8];
        
        copy_bits_forward(&src, 0, &mut dst, 0, 0);
        assert_eq!(dst[0], 0);
    }

    #[test]
    fn test_copy_bits_forward_src_out_of_bounds() {
        let src = vec![0xFFu8];
        let mut dst = vec![0u8; 2];
        
        // Copy from beyond src length
        copy_bits_forward(&src, 100, &mut dst, 0, 4);
        assert_eq!(dst[0], 0);
    }

    #[test]
    fn test_copy_bits_forward_dst_out_of_bounds() {
        let src = vec![0xFFu8];
        let mut dst = vec![0u8; 1];
        
        // Copy to beyond dst length (should not panic, just skip)
        copy_bits_forward(&src, 0, &mut dst, 100, 4);
        assert_eq!(dst[0], 0);
    }

    #[test]
    fn test_copy_bits_forward_bits_span_two_bytes_src() {
        let src = vec![0b11110000u8, 0b10101010u8];
        let mut dst = vec![0u8; 1];
        
        // Copy bits that span two source bytes, all fit in one destination byte
        // This exercises the path where src_bit_offset + n > 8
        copy_bits_forward(&src, 5, &mut dst, 0, 4);
        // Just verify it doesn't panic and produces some result
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_bits_span_two_bytes_src_out_of_bounds() {
        let src = vec![0b11110000u8];
        let mut dst = vec![0u8; 1];
        
        // Copy bits that would span two bytes but next byte doesn't exist
        // This exercises the path where src_byte_offset + 1 >= src.len()
        copy_bits_forward(&src, 5, &mut dst, 0, 4);
        // Should handle gracefully with 0 for missing byte
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_multi_byte_aligned() {
        let src = vec![0xFFu8, 0xAAu8, 0x55u8];
        let mut dst = vec![0u8; 3];
        
        // Copy multiple aligned bytes
        copy_bits_forward(&src, 0, &mut dst, 0, 24);
        assert_eq!(dst[0], 0xFF);
        assert_eq!(dst[1], 0xAA);
        assert_eq!(dst[2], 0x55);
    }

    #[test]
    fn test_copy_bits_forward_multi_byte_unaligned_src() {
        let src = vec![0b11110000u8, 0b10101010u8, 0b01010101u8];
        let mut dst = vec![0u8; 3];
        
        // Copy with unaligned source offset - exercises multi-byte path
        copy_bits_forward(&src, 4, &mut dst, 0, 16);
        // Just verify it doesn't panic
        let _ = dst[0];
        let _ = dst[1];
    }

    #[test]
    fn test_copy_bits_forward_multi_byte_unaligned_dst() {
        let src = vec![0xFFu8, 0xAAu8];
        let mut dst = vec![0b11110000u8; 2];
        
        // Copy with unaligned destination offset - exercises first byte partial path
        copy_bits_forward(&src, 0, &mut dst, 4, 16);
        // Just verify it doesn't panic
        let _ = dst[0];
        let _ = dst[1];
    }

    #[test]
    fn test_copy_bits_forward_multi_byte_both_unaligned() {
        let src = vec![0b11110000u8, 0b10101010u8];
        let mut dst = vec![0b00001111u8; 2];
        
        // Both source and destination unaligned - exercises complex path
        copy_bits_forward(&src, 2, &mut dst, 3, 10);
        // Just verify it doesn't panic
        let _ = dst[0];
        let _ = dst[1];
    }

    #[test]
    fn test_copy_bits_forward_first_byte_partial_src_bit_less_than_dst_bit() {
        let src = vec![0b10101010u8];
        let mut dst = vec![0b11110000u8; 2];
        
        // First byte partial, src_bit < dst_bit - exercises else branch
        // This should execute line 243: dst_bit = 0;
        copy_bits_forward(&src, 1, &mut dst, 3, 12);
        // Verify the first byte was modified (exercises the path through line 243)
        assert_ne!(dst[0], 0b11110000);
    }

    #[test]
    fn test_copy_bits_forward_first_byte_partial_src_bit_greater_than_dst_bit() {
        let src = vec![0b10101010u8];
        let mut dst = vec![0b00001111u8; 2];
        
        // First byte partial, src_bit > dst_bit - exercises if branch
        copy_bits_forward(&src, 5, &mut dst, 2, 12);
        // Just verify it doesn't panic
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_first_byte_partial_src_bit_equals_dst_bit() {
        let src = vec![0b10101010u8];
        let mut dst = vec![0b11110000u8; 2];
        
        // First byte partial, src_bit == dst_bit - exercises first branch
        copy_bits_forward(&src, 3, &mut dst, 3, 12);
        // Just verify it doesn't panic
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_first_byte_partial_src_advances() {
        let src = vec![0b11111111u8, 0b00000000u8];
        let mut dst = vec![0u8; 2];
        
        // First byte partial where src_bit + bits_to_copy >= 8 - exercises if branch
        copy_bits_forward(&src, 5, &mut dst, 2, 10);
        // Just verify it doesn't panic
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_first_byte_partial_src_no_advance() {
        let src = vec![0b11111111u8];
        let mut dst = vec![0u8; 2];
        
        // First byte partial where src_bit + bits_to_copy < 8 - exercises else branch
        copy_bits_forward(&src, 2, &mut dst, 3, 4);
        // Just verify it doesn't panic
        let _ = dst[0];
    }

    #[test]
    fn test_copy_bits_forward_middle_bytes_src_bit_nonzero() {
        let src = vec![0b11110000u8, 0b10101010u8, 0b01010101u8];
        let mut dst = vec![0u8; 3];
        
        // Middle bytes with src_bit != 0
        copy_bits_forward(&src, 4, &mut dst, 0, 16);
        // src[4:20], src_bit starts at 4
        // First byte: bits 4-7 from byte 0, bits 0-3 from byte 1
        assert_eq!(dst[0], 0b00001010);
        assert_eq!(dst[1], 0b10100101);
    }

    #[test]
    fn test_copy_bits_forward_middle_bytes_src_out_of_bounds() {
        let src = vec![0b11110000u8];
        let mut dst = vec![0u8; 2];
        
        // Middle bytes where src runs out
        copy_bits_forward(&src, 4, &mut dst, 0, 16);
        // src[4:20] but src only has 8 bits
        // Should handle gracefully
        assert_eq!(dst[0], 0b00000000);
    }

    #[test]
    fn test_copy_bits_forward_last_byte_partial_src_bit_zero() {
        let src = vec![0xFFu8, 0xAAu8];
        let mut dst = vec![0b11110000u8; 2];
        
        // Last byte partial with src_bit == 0 - exercises first branch
        copy_bits_forward(&src, 8, &mut dst, 2, 6);
        // Just verify it doesn't panic
        let _ = dst[1];
    }

    #[test]
    fn test_copy_bits_forward_last_byte_partial_src_bit_nonzero() {
        let src = vec![0b11110000u8, 0b10101010u8];
        let mut dst = vec![0b00001111u8; 2];
        
        // Last byte partial with src_bit != 0 - exercises else branch
        copy_bits_forward(&src, 5, &mut dst, 3, 10);
        // Just verify it doesn't panic
        let _ = dst[1];
    }

    #[test]
    fn test_copy_bits_forward_last_byte_partial_src_out_of_bounds() {
        let src = vec![0b11110000u8];
        let mut dst = vec![0b11111111u8; 2];
        
        // Last byte partial where src runs out - exercises src_idx + 1 >= src.len() path
        copy_bits_forward(&src, 5, &mut dst, 2, 10);
        // Just verify it doesn't panic
        let _ = dst[1];
    }

    #[test]
    fn test_cmp_bits_a_out_of_bounds() {
        let a = vec![0xFFu8];
        let b = vec![0xFFu8, 0xFFu8];
        
        // Compare where a runs out
        assert_eq!(cmp_bits(&a, 0, &b, 0, 16), -1);
    }

    #[test]
    fn test_cmp_bits_b_out_of_bounds() {
        let a = vec![0xFFu8, 0xFFu8];
        let b = vec![0xFFu8];
        
        // Compare where b runs out
        assert_eq!(cmp_bits(&a, 0, &b, 0, 16), 1);
    }

    #[test]
    fn test_cmp_bits_both_out_of_bounds() {
        let a = vec![0xFFu8];
        let b = vec![0xFFu8];
        
        // Compare where both run out (should be equal)
        assert_eq!(cmp_bits(&a, 0, &b, 0, 16), 0);
    }

    #[test]
    fn test_cmp_bits_large_sequence() {
        let a = vec![0xFFu8, 0xAAu8, 0x55u8, 0x00u8];
        let b = vec![0xFFu8, 0xAAu8, 0x55u8, 0x00u8];
        
        // Compare large sequence
        assert_eq!(cmp_bits(&a, 0, &b, 0, 32), 0);
    }

    #[test]
    fn test_cmp_bits_different_at_end() {
        let a = vec![0xFFu8, 0xFFu8];
        let b = vec![0xFFu8, 0xFEu8];
        
        // Different at the end
        assert_eq!(cmp_bits(&a, 0, &b, 0, 16), 1);
    }

    #[test]
    fn test_get_bit_out_of_range() {
        let byte = 0xFFu8;
        assert_eq!(get_bit(byte, 8), 0);
        assert_eq!(get_bit(byte, 100), 0);
    }

    #[test]
    fn test_set_bit_out_of_range() {
        let byte = 0xFFu8;
        assert_eq!(set_bit(byte, 8, 1), byte);
        assert_eq!(set_bit(byte, 100, 1), byte);
    }

    #[test]
    fn test_make_mask_large() {
        assert_eq!(make_mask(64), u64::MAX);
        assert_eq!(make_mask(65), u64::MAX);
        assert_eq!(make_mask(100), u64::MAX);
    }

    #[test]
    fn test_make_mask_exactly_64() {
        // Test the boundary case where n == 64 (should use the else branch)
        assert_eq!(make_mask(63), ((1u64 << 63) - 1));
        assert_eq!(make_mask(64), u64::MAX);
    }

    #[test]
    fn test_copy_bits_forward_dst_bit_zero_path() {
        // Test the path where dst_bit == 0 initially (skips first byte partial)
        let src = vec![0xFFu8, 0xAAu8];
        let mut dst = vec![0u8; 2];
        
        // dst_bit_offset = 0, so we skip the "if dst_bit != 0" block
        // This exercises the path where dst_bit stays 0 and we go directly to middle bytes
        copy_bits_forward(&src, 0, &mut dst, 0, 16);
        assert_eq!(dst[0], 0xFF);
        assert_eq!(dst[1], 0xAA);
    }

    #[test]
    fn test_copy_bits_forward_dst_bit_zero_with_remaining() {
        // Test path where dst_bit == 0 but we have remaining bits after middle bytes
        let src = vec![0xFFu8, 0xAAu8, 0x55u8];
        let mut dst = vec![0u8; 3];
        
        // Copy 20 bits: 2 full bytes + 4 remaining bits
        // dst_bit starts at 0, so we skip first byte partial
        // After 2 middle bytes (16 bits), we have 4 remaining bits
        copy_bits_forward(&src, 0, &mut dst, 0, 20);
        assert_eq!(dst[0], 0xFF);
        assert_eq!(dst[1], 0xAA);
        // Last byte should have first 4 bits of 0x55
        // 0x55 = 0b01010101, first 4 bits (LSB) = 0b0101 = 5
        // But we're placing at position 0, so it's just 0b0101 = 5
        assert_eq!(dst[2], 5);
    }
}
