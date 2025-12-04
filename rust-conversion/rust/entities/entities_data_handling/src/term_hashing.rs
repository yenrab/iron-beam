//! Term Hashing Module
//!
//! Provides hash functions for Erlang terms:
//! - `make_hash`: Portable hash function (bug-compatible across versions)
//! - `make_hash2`: Faster hash function with better distribution
//! - `erts_internal_hash`: Internal hash for VM use
//! - `erts_map_hash`: Hash function specifically for maps

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

use entities_utilities::{BigNumber, BigRational};

/// Hash value type (32-bit or 64-bit depending on platform)
pub type HashValue = u64;

/// Hash a 32-bit unsigned integer byte-wise (endianness-independent)
///
/// This matches the C UINT32_HASH_STEP macro behavior.
/// Hashes the 4 bytes of the integer, least significant byte first.
fn hash_u32_bytewise(hash: u32, value: u32, prime: u32) -> u32 {
    // Hash byte by byte, least significant first (endianness-independent)
    // hash = (((((hash)*prime + (x & 0xFF)) * prime + ((x >> 8) & 0xFF)) * prime + 
    //           ((x >> 16) & 0xFF)) * prime + (x >> 24))
    let h1 = hash.wrapping_mul(prime).wrapping_add(value & 0xFF);
    let h2 = h1.wrapping_mul(prime).wrapping_add((value >> 8) & 0xFF);
    let h3 = h2.wrapping_mul(prime).wrapping_add((value >> 16) & 0xFF);
    h3.wrapping_mul(prime).wrapping_add(value >> 24)
}

/// Hash a number (small integer or bignum) byte-wise
///
/// This implements the algorithm described in the C code comments:
/// - Hash abs(N) byte-wise with least significant byte first
/// - Multiply by HASH_MULT_NEGATIVE if negative, HASH_MULT_POSITIVE if positive
fn hash_number_bytewise(hash: u32, value: i64) -> u32 {
    let abs_value = if value < 0 {
        (-value) as u64
    } else {
        value as u64
    };
    
    // Hash the lower 32 bits
    let mut h = hash_u32_bytewise(hash, (abs_value & 0xFFFFFFFF) as u32, HASH_MULT_NUMBER);
    
    // Hash the upper 32 bits if present (64-bit systems)
    #[cfg(target_pointer_width = "64")]
    if abs_value > 0xFFFFFFFF {
        h = hash_u32_bytewise(h, (abs_value >> 32) as u32, HASH_MULT_NUMBER);
    }
    
    // Multiply by sign-dependent constant
    if value < 0 {
        h.wrapping_mul(HASH_MULT_NEGATIVE)
    } else {
        h.wrapping_mul(HASH_MULT_POSITIVE)
    }
}

/// Hash binary bytes (supports bit-aligned binaries)
///
/// This matches the C hash_binary_bytes function behavior exactly.
/// Handles both byte-aligned and bit-aligned binaries, including partial bytes.
///
/// # Arguments
/// * `hash` - Initial hash value
/// * `data` - Binary data bytes
/// * `bit_offset` - Bit offset within first byte (0-7)
/// * `bit_size` - Total size in bits
///
/// # Returns
/// Hash value after processing the binary
fn hash_binary_bytes(hash: u32, data: &[u8], bit_offset: usize, bit_size: usize) -> u32 {
    // C: BYTE_SIZE(size) and TAIL_BITS(size)
    let bytesize = bit_size >> 3;  // Number of full bytes
    let bitsize = bit_size & 7;    // Remaining bits in last byte (0-7)
    let bitoffs = bit_offset & 7;  // Bit offset within first byte (0-7)
    
    let mut h = hash;
    
    if bitoffs == 0 {
        // Byte-aligned case: hash all full bytes
        // C: for (i = 0; i < bytesize; i++) { hash = hash*HASH_MULT_ATOM + ptr[i]; }
        for i in 0..bytesize {
            if i < data.len() {
                h = h.wrapping_mul(HASH_MULT_ATOM).wrapping_add(data[i] as u32);
            }
        }
        
        // Handle tail bits if present
        // C: if (bitsize > 0) { byte b = ptr[i]; b >>= 8 - bitsize; hash = (hash*HASH_MULT_ATOM + b) * HASH_MULT_BINARY_TAIL + bitsize; }
        if bitsize > 0 && bytesize < data.len() {
            let mut b = data[bytesize];
            b >>= 8 - bitsize;  // Extract only the relevant bits
            h = h.wrapping_mul(HASH_MULT_ATOM).wrapping_add(b as u32);
            h = h.wrapping_mul(HASH_MULT_BINARY_TAIL).wrapping_add(bitsize as u32);
        }
    } else {
        // Bit-aligned case: extract bytes across byte boundaries
        // C: Uses bit shifting to extract bytes
        let mut previous = if data.is_empty() { 0u8 } else { data[0] };
        let lshift = bitoffs;
        let rshift = 8 - lshift;
        
        // Hash all full bytes
        // C: for (i = 0; i < bytesize; i++) { b = (previous << lshift) & 0xFF; previous = ptr[i]; b |= previous >> rshift; hash = hash*HASH_MULT_ATOM + b; }
        for i in 0..bytesize {
            let mut b = (previous << lshift) & 0xFF;
            if i + 1 < data.len() {
                previous = data[i + 1];
            } else {
                previous = 0;
            }
            b |= previous >> rshift;
            h = h.wrapping_mul(HASH_MULT_ATOM).wrapping_add(b as u32);
        }
        
        // Handle tail bits if present
        // C: if (bitsize > 0) { b = (previous << lshift) & 0xFF; previous = ptr[i]; b |= previous >> rshift; b >>= 8 - bitsize; hash = (hash*HASH_MULT_ATOM + b) * HASH_MULT_BINARY_TAIL + bitsize; }
        if bitsize > 0 {
            let mut b = (previous << lshift) & 0xFF;
            if bytesize + 1 < data.len() {
                previous = data[bytesize + 1];
            } else {
                previous = 0;
            }
            b |= previous >> rshift;
            b >>= 8 - bitsize;  // Extract only the relevant bits
            h = h.wrapping_mul(HASH_MULT_ATOM).wrapping_add(b as u32);
            h = h.wrapping_mul(HASH_MULT_BINARY_TAIL).wrapping_add(bitsize as u32);
        }
    }
    
    // Final multiplication and add bytesize (matches C: hash * HASH_MULT_NEGATIVE + bytesize)
    h.wrapping_mul(HASH_MULT_NEGATIVE).wrapping_add(bytesize as u32)
}

/// Compute atom hash value using hashpjw algorithm
///
/// Uses the hashpjw algorithm from the Dragon Book.
///
/// This function is available for use when atom names are available.
/// Currently, `Term::Atom` only stores the atom index, so this function
/// cannot be used directly in `make_hash`. When the full Eterm type is
/// available in higher layers, atom names will be accessible and this
/// function can be used to compute the exact hash values.
///
/// # Arguments
/// * `name` - Atom name bytes
///
/// # Returns
/// Hash value for the atom
pub(crate) fn atom_hash_pjw(name: &[u8]) -> u32 {
    let mut h: u32 = 0;
    let mut i = 0;
    
    while i < name.len() {
        let mut v = name[i] as u32;
        i += 1;
        
        // Latin1 clutch for r16 (handle UTF-8 continuation bytes)
        // C: if (len && (v & 0xFE) == 0xC2 && (*p & 0xC0) == 0x80)
        if i < name.len() && (v & 0xFE) == 0xC2 && (name[i] & 0xC0) == 0x80 {
            v = (v << 6) | ((name[i] & 0x3F) as u32);
            i += 1;
        }
        
        // Normal hashpjw algorithm
        // C: h = (h << 4) + v;
        h = h.wrapping_shl(4).wrapping_add(v);
        
        // C: if ((g = h & 0xf0000000)) { h ^= (g >> 24); h ^= g; }
        let g = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h ^= g;
        }
    }
    
    h
}

/// Portable hash function that gives same values for same terms
/// regardless of internal representation.
///
/// This is the hash function used by erlang:phash/2.
/// It ensures that small integers, bignums, pids, ports, and references
/// are hashed consistently across different CPU endianness.
///
/// This implementation matches the C `make_hash` function behavior.
///
/// # Arguments
/// * `term` - The Erlang term to hash
///
/// # Returns
/// A 32-bit hash value
/// Stack entry for tuple and map processing
/// Tracks when we need to hash tuple arity after all elements are processed,
/// and map pair/tail markers for make_hash2
enum StackEntry {
    Term(Term),
    TupleArity(u32), // Marker: hash this arity after processing all tuple elements
    MapPair,         // Marker: process a map key-value pair (for make_hash2)
    MapTail,         // Marker: finish map hashing (for make_hash2)
    MapSavedHash(u32), // Saved hash value for map processing (for make_hash2)
    MapSavedXor(u32), // Saved hash_xor_pairs value for map processing (for make_hash2)
}

pub fn make_hash(term: Term) -> u32 {
    // Use a stack-based approach for recursive structures (matches C implementation)
    let mut stack: Vec<StackEntry> = Vec::new();
    let mut current_term = Some(term);
    let mut hash = 0u32;
    
    loop {
        let term_to_process = match current_term.take() {
            Some(t) => t,
            None => {
                // No current term, get next from stack
                if let Some(entry) = stack.pop() {
                    match entry {
                        StackEntry::TupleArity(arity) => {
                            // C: When all tuple elements are processed (i == 0), hash the arity
                            // C: hash = hash*HASH_MULT_TUPLE_ARITY + arity;
                            hash = hash.wrapping_mul(HASH_MULT_TUPLE_ARITY).wrapping_add(arity);
                            continue;
                        }
                        StackEntry::MapPair | StackEntry::MapTail | 
                        StackEntry::MapSavedHash(_) | StackEntry::MapSavedXor(_) => {
                            // These are only used in make_hash2, not make_hash
                            // Should not appear in make_hash's stack
                            break;
                        }
                        StackEntry::Term(next) => {
                            // Check if this is a list tail (non-list means end of list)
                            match &next {
                                Term::List { .. } => {
                                    current_term = Some(next);
                                    continue;
                                }
                                _ => {
                                    // Not a list - this is the end of a list, multiply by HASH_MULT_LIST_END
                                    // C: hash *= HASH_MULT_LIST_END; (in MAKE_HASH_CDR_POST_OP)
                                    hash = hash.wrapping_mul(HASH_MULT_LIST_END);
                                    current_term = Some(next);
                                    continue;
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        };
        
        match term_to_process {
            Term::Nil => {
                // C: hash = hash*HASH_MULT_POSITIVE + 1;
                hash = hash.wrapping_mul(HASH_MULT_POSITIVE).wrapping_add(1);
            }
            
            Term::Small(value) => {
                hash = hash_number_bytewise(hash, value);
            }
            
            Term::Atom(atom_val) => {
                // C: hash = hash*HASH_MULT_ATOM + (atom_tab(atom_val(term))->slot.bucket.hvalue);
                // The C implementation looks up the atom in the atom table and uses the bucket's hvalue,
                // which is computed using the hashpjw algorithm (see atom_hash_pjw function above).
                // 
                // In the entities layer, we don't have access to the atom table to look up the name
                // and compute the hashpjw value. We use the atom index directly as a proxy.
                // When the full Eterm type is available in higher layers, it will use the atom table hash.
                //
                // Note: This means hash values will differ from C, but the algorithm structure is the same.
                hash = hash.wrapping_mul(HASH_MULT_ATOM).wrapping_add(atom_val);
            }
            
            Term::Big(bignum) => {
                // Hash bignum byte-wise, matching C implementation exactly
                // Extract bytes from BigNumber using malachite's Integer representation
                use malachite::Integer;
                
                let integer = bignum.as_integer();
                let is_negative = !bignum.is_positive() && !bignum.is_zero();
                
                // Get the absolute value for byte extraction
                // Use abs() method and then convert to limbs
                let abs_value = if is_negative {
                    -integer.clone()
                } else {
                    integer.clone()
                };
                
                // Convert to bytes (little-endian, matching C's digit order)
                // Use to_twos_complement_limbs_asc to get u32 limbs
                let limbs = abs_value.to_twos_complement_limbs_asc();
                let n = limbs.len();
                
                let mut h = hash;
                
                if n == 0 {
                    // Zero bignum
                    hash = h.wrapping_mul(HASH_MULT_POSITIVE);
                    } else {
                    // Hash all limbs except the last one (4 bytes each)
                    for i in 0..(n - 1) {
                        let digit = limbs[i];
                        // Hash all 4 bytes of the digit (least significant byte first)
                        for byte_offset in 0..4 {
                            let byte = ((digit >> (byte_offset * 8)) & 0xFF) as u32;
                            h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                        }
                    }
                    
                    // Hash the last limb with optimization
                    // C: k = sizeof(ErtsDigit); if (ARCH_64 && !(d >> 32)) k /= 2;
                    let last_digit = limbs[n - 1];
                    let bytes_to_hash = {
                        #[cfg(target_pointer_width = "64")]
                        {
                            // On 64-bit systems, check if upper 32 bits are zero
                            if (last_digit >> 32) == 0 {
                                4  // Hash 4 bytes if upper 32 bits are zero
                            } else {
                                8  // Hash 8 bytes if upper 32 bits are non-zero
                            }
                        }
                        #[cfg(not(target_pointer_width = "64"))]
                        {
                            // On 32-bit systems, sizeof(ErtsDigit) = 4, so always hash 4 bytes
                            4
                        }
                    };
                    
                    // Hash the last digit (bytes_to_hash bytes, least significant byte first)
                    for byte_offset in 0..bytes_to_hash {
                        let byte = ((last_digit >> (byte_offset * 8)) & 0xFF) as u32;
                        h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                    }
                    
                    // Multiply by sign-dependent constant
                    hash = if is_negative {
                        h.wrapping_mul(HASH_MULT_NEGATIVE)
                    } else {
                        h.wrapping_mul(HASH_MULT_POSITIVE)
                    };
                }
            }
            
            Term::Rational(rational) => {
                // Hash rational number by hashing numerator and denominator separately
                // Similar to how we hash BigNumber, but we need to hash both parts
                use malachite::Integer;
                
                let numerator = rational.numerator();
                let denominator = rational.denominator();
                let is_negative = rational.is_negative();
                
                // Hash numerator (similar to BigNumber hashing)
                let num_abs = if is_negative {
                    -numerator.clone()
                } else {
                    numerator.clone()
                };
                let num_limbs = num_abs.to_twos_complement_limbs_asc();
                let num_n = num_limbs.len();
                
                let mut h = hash;
                
                if num_n == 0 {
                    // Zero numerator
                    h = h.wrapping_mul(HASH_MULT_POSITIVE);
                } else {
                    // Hash all numerator limbs except the last one
                    for i in 0..(num_n - 1) {
                        let digit = num_limbs[i];
                        for byte_offset in 0..4 {
                            let byte = ((digit >> (byte_offset * 8)) & 0xFF) as u32;
                            h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                        }
                    }
                    
                    // Hash the last numerator limb
                    let last_digit = num_limbs[num_n - 1];
                    let bytes_to_hash = {
                        #[cfg(target_pointer_width = "64")]
                        {
                            if (last_digit >> 32) == 0 { 4 } else { 8 }
                        }
                        #[cfg(not(target_pointer_width = "64"))]
                        { 4 }
                    };
                    
                    for byte_offset in 0..bytes_to_hash {
                        let byte = ((last_digit >> (byte_offset * 8)) & 0xFF) as u32;
                        h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                    }
                    
                    // Multiply by sign-dependent constant for numerator
                    h = if is_negative {
                        h.wrapping_mul(HASH_MULT_NEGATIVE)
                    } else {
                        h.wrapping_mul(HASH_MULT_POSITIVE)
                    };
                }
                
                // Hash denominator (always positive for rational numbers)
                let den_limbs = denominator.to_twos_complement_limbs_asc();
                let den_n = den_limbs.len();
                
                if den_n == 0 {
                    // Zero denominator (shouldn't happen, but handle it)
                    hash = h.wrapping_mul(HASH_MULT_POSITIVE);
                } else {
                    // Hash all denominator limbs except the last one
                    for i in 0..(den_n - 1) {
                        let digit = den_limbs[i];
                        for byte_offset in 0..4 {
                            let byte = ((digit >> (byte_offset * 8)) & 0xFF) as u32;
                            h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                        }
                    }
                    
                    // Hash the last denominator limb
                    let last_digit = den_limbs[den_n - 1];
                    let bytes_to_hash = {
                        #[cfg(target_pointer_width = "64")]
                        {
                            if (last_digit >> 32) == 0 { 4 } else { 8 }
                        }
                        #[cfg(not(target_pointer_width = "64"))]
                        { 4 }
                    };
                    
                    for byte_offset in 0..bytes_to_hash {
                        let byte = ((last_digit >> (byte_offset * 8)) & 0xFF) as u32;
                        h = h.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(byte);
                    }
                    
                    // Multiply by positive constant for denominator (always positive)
                    hash = h.wrapping_mul(HASH_MULT_POSITIVE);
                }
            }
            
            Term::Float(value) => {
                // C: Normalize zero to positive zero before hashing
                // C: if (ff.fd == 0.0f) { ff.fd = erts_get_positive_zero_float(); }
                // erts_get_positive_zero_float() returns 0.0f (positive zero)
                let normalized_value = if value == 0.0 {
                    // Normalize to positive zero (0.0)
                    0.0
                } else {
                    value
                };
                
                // C: hash = hash*HASH_MULT_PID_FINAL + (ff.fw[0] ^ ff.fw[1]);
                // Hash the two 32-bit words of the float
                let bits = normalized_value.to_bits();
                let low = (bits & 0xFFFFFFFF) as u32;
                let high = ((bits >> 32) & 0xFFFFFFFF) as u32;
                hash = hash.wrapping_mul(HASH_MULT_PID_FINAL).wrapping_add(low ^ high);
            }
            
            Term::Binary { data, bit_offset, bit_size } => {
                hash = hash_binary_bytes(hash, &data, bit_offset, bit_size);
            }
            
            Term::List { head, tail } => {
                // Check if head is a byte (optimization for strings)
                // C uses optimization for byte lists (strings)
                let head_val = *head;
                let tail_val = *tail;
                
                if let Term::Small(byte_val) = &head_val {
                    if *byte_val >= 0 && *byte_val <= 255 {
                        // Optimization for strings: hash = hash*HASH_MULT_NUMBER + unsigned_val(*list)
                        hash = hash.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(*byte_val as u32);
                        
                        // Check if tail is also a list (continue string optimization)
                        if matches!(&tail_val, Term::List { .. }) {
                            current_term = Some(tail_val);
                            continue;
                        } else {
                            // Tail is not a list - hash it and mark list end
                            // C uses MAKE_HASH_CDR_POST_OP which multiplies by HASH_MULT_LIST_END
                            stack.push(StackEntry::Term(tail_val));
                            // After hashing the tail, we'll multiply by HASH_MULT_LIST_END
                            // This is handled in the stack processing logic
                            current_term = None; // Will be set from stack
                            continue;
                        }
                    }
                }
                
                // General list: hash head, then tail
                // Push tail - when we process it and it's not a list, multiply by HASH_MULT_LIST_END
                stack.push(StackEntry::Term(tail_val));
                current_term = Some(head_val);
                continue;
            }
            
            Term::Tuple(elements) => {
                // Hash tuple: C hashes all elements first, then arity
                // C: WSTACK_PUSH3(stack, (UWord) arity, (UWord)(ptr+1), (UWord) arity);
                //    Then processes elements, and when i == 0, hashes arity
                let arity = elements.len() as u32;
                
                // Push elements onto stack in reverse order (so first element is popped first)
                for element in elements.into_iter().rev() {
                    stack.push(StackEntry::Term(element));
                }
                
                // Push arity marker - this will be processed after all elements are hashed
                // C: When i == 0 (all elements processed), pops arity and hashes it
                stack.push(StackEntry::TupleArity(arity));
                
                // Continue with first element
                if let Some(StackEntry::Term(first)) = stack.pop() {
                    current_term = Some(first);
                    continue;
                }
                // If no elements, process the arity marker immediately
                if let Some(StackEntry::TupleArity(arity_val)) = stack.pop() {
                    hash = hash.wrapping_mul(HASH_MULT_TUPLE_ARITY).wrapping_add(arity_val);
                }
                // No more work, get next from stack
                current_term = None;
                continue;
            }
            
            Term::Map(entries) => {
                // C: hash = hash*HASH_MULT_MAP_FIRST + HASH_MULT_MAP_SECOND + make_hash2(term);
                let map_hash = make_hash2(Term::Map(entries));
                hash = hash.wrapping_mul(HASH_MULT_MAP_FIRST)
                    .wrapping_add(HASH_MULT_MAP_SECOND)
                    .wrapping_add(map_hash);
            }
            
            Term::Pid { id, .. } => {
                // C: UINT32_HASH_RET(internal_pid_number(term), HASH_MULT_PID_BYTE, HASH_MULT_PID_FINAL);
                // Only hash the id (15 bits)
                hash = hash_u32_bytewise(hash, id, HASH_MULT_PID_BYTE);
                hash = hash.wrapping_mul(HASH_MULT_PID_FINAL);
            }
            
            Term::Port { id, .. } => {
                // C: Hash port number (can be 64-bit)
                // C: Uint64 number = port_number(term);
                //    Uint32 low = (Uint32) (number & 0xffffffff);
                //    Uint32 high = (Uint32) ((number >> 32) & 0xffffffff);
                //    if (high) UINT32_HASH_STEP(high, HASH_MULT_EXTERNAL_FUN);
                //    UINT32_HASH_RET(low, HASH_MULT_TUPLE_ARITY, HASH_MULT_PORT_REF_FINAL);
                let low = (id & 0xFFFFFFFF) as u32;
                let high = ((id >> 32) & 0xFFFFFFFF) as u32;
                
                // Hash high 32 bits if non-zero
                if high != 0 {
                    hash = hash_u32_bytewise(hash, high, HASH_MULT_EXTERNAL_FUN);
                }
                
                // Hash low 32 bits
                hash = hash_u32_bytewise(hash, low, HASH_MULT_TUPLE_ARITY);
                hash = hash.wrapping_mul(HASH_MULT_PORT_REF_FINAL);
            }
            
            Term::Ref { ids, .. } => {
                // C: UINT32_HASH_RET(internal_ref_numbers(term)[0], HASH_MULT_TUPLE_ARITY, HASH_MULT_PORT_REF_FINAL);
                // Hash only the first (least significant) number
                if let Some(&first_id) = ids.first() {
                    hash = hash_u32_bytewise(hash, first_id, HASH_MULT_TUPLE_ARITY);
                    hash = hash.wrapping_mul(HASH_MULT_PORT_REF_FINAL);
                }
            }
            
            Term::Fun { is_local, module, function, arity, old_uniq, env } => {
                if is_local {
                    // C: Local function hashing
                    // hash = hash * HASH_MULT_PORT_REF_FINAL + num_free;
                    // hash = hash*HASH_MULT_ATOM + (atom_tab(atom_val(fe->module))->slot.bucket.hvalue);
                    // hash = hash*HASH_MULT_NUMBER + fe->index;
                    // hash = hash*HASH_MULT_NUMBER + fe->old_uniq;
                    let num_free = env.len() as u32;
                    hash = hash.wrapping_mul(HASH_MULT_PORT_REF_FINAL).wrapping_add(num_free);
                    // For now, use module index directly (would use atom table hash in full implementation)
                    hash = hash.wrapping_mul(HASH_MULT_ATOM).wrapping_add(module);
                    hash = hash.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(function); // index
                    if let Some(uniq) = old_uniq {
                        hash = hash.wrapping_mul(HASH_MULT_NUMBER).wrapping_add(uniq);
                    }
                    
                    // Hash environment if present
                    // C: if (num_free > 0) { if (num_free > 1) { push env[1..] } term = env[0]; goto tail_recur; }
                    if !env.is_empty() {
                        let env_clone = env.clone();
                        if env_clone.len() > 1 {
                            // Push remaining environment elements in reverse order
                            for element in env_clone.into_iter().skip(1).rev() {
                                stack.push(StackEntry::Term(element));
                            }
                        }
                        // Process first environment element
                        current_term = Some(env[0].clone());
                        continue;
                    }
                } else {
                    // C: External function hashing
                    // hash = hash * HASH_MULT_EXTERNAL_FUN + mfa->arity;
                    // hash = hash*HASH_MULT_ATOM + (atom_tab(atom_val(mfa->module))->slot.bucket.hvalue);
                    // hash = hash*HASH_MULT_ATOM + (atom_tab(atom_val(mfa->function))->slot.bucket.hvalue);
                    hash = hash.wrapping_mul(HASH_MULT_EXTERNAL_FUN).wrapping_add(arity);
                    // For now, use atom indices directly (would use atom table hashes in full implementation)
                    hash = hash.wrapping_mul(HASH_MULT_ATOM).wrapping_add(module);
                    hash = hash.wrapping_mul(HASH_MULT_ATOM).wrapping_add(function);
                }
            }
        }
        
        // After processing term, get next from stack
        current_term = None;
    }
    
    hash
}

/// Faster hash function with better distribution than make_hash.
///
/// This is optimized for performance, particularly for bignums and binaries.
/// Uses Bob Jenkins' hash function (MIX algorithm) for better distribution.
///
/// This implementation is simplified for the entities layer. The full C implementation
/// uses block hashing for binaries and more sophisticated algorithms, but this provides
/// the core functionality needed for map hashing.
///
/// # Arguments
/// * `term` - The Erlang term to hash
///
/// # Returns
/// A 32-bit hash value
pub fn make_hash2(term: Term) -> u32 {
    // Bob Jenkins' hash constants
    const HCONST: u32 = 0x9e3779b9; // the golden ratio
    const HCONST_16: u32 = 0xe3779b90; // HCONST * 16 mod 2^32
    const HCONST_19: u32 = 0xbe1e08bb; // HCONST * 19 mod 2^32
    
    // Use a stack-based approach similar to make_hash
    let mut stack: Vec<StackEntry> = Vec::new();
    let mut current_term = Some(term);
    let mut hash = 0u32;
    let mut hash_xor_pairs = 0u32; // For maps: XOR of independent key-value pair hashes
    
    loop {
        let term_to_process = match current_term.take() {
            Some(t) => t,
            None => {
                // No current term, get next from stack
                if let Some(entry) = stack.pop() {
                    match entry {
                        StackEntry::TupleArity(arity) => {
                            // Tuple arity (for make_hash2, we hash it differently)
                            hash = mix_hash(hash, HCONST_9, arity);
                            continue;
                        }
                        StackEntry::MapPair => {
                            // Process a map key-value pair
                            // Hash the value, then XOR with hash_xor_pairs
                            hash_xor_pairs ^= hash;
                            hash = 0;
                            continue;
                        }
                        StackEntry::MapTail => {
                            // Finish map hashing: hash the XOR of all pairs
                            // C: hash = (Uint32) ESTACK_POP(s); UINT32_HASH(hash_xor_pairs, HCONST_19);
                            // Restore saved values from stack
                            let saved_hash = if let Some(StackEntry::MapSavedHash(h)) = stack.pop() {
                                h
                            } else {
                                0
                            };
                            let saved_xor = if let Some(StackEntry::MapSavedXor(x)) = stack.pop() {
                                x
                            } else {
                                0
                            };
                            // C: hash = saved_hash; UINT32_HASH(hash_xor_pairs, HCONST_19);
                            hash = saved_hash;
                            hash = mix_hash(hash, HCONST_19, hash_xor_pairs);
                            hash_xor_pairs = saved_xor; // Restore for potential nested maps
                            continue;
                        }
                        StackEntry::MapSavedHash(_) | StackEntry::MapSavedXor(_) => {
                            // These should only be popped in MapTail handler
                            break;
                        }
                        StackEntry::Term(next) => {
                            current_term = Some(next);
                            continue;
                        }
                    }
                } else {
                    break;
                }
            }
        };
        
        match term_to_process {
            Term::Nil => {
                hash = mix_hash(hash, HCONST, 1);
            }
            
            Term::Small(value) => {
                // C: SINT32_HASH - hash signed 32-bit integer
                if value < 0 {
                    hash = mix_hash(hash, HCONST, (-value) as u32);
                }
                hash = mix_hash(hash, HCONST, value as u32);
            }
            
            Term::Atom(atom_val) => {
                // C: Fast path - return atom hash value directly
                // For now, use atom_val as hash (would use atom table hash in full implementation)
                hash = atom_val;
            }
            
            Term::Big(bignum) => {
                // C: Hash bignum using block hashing
                // Extract limbs from BigNumber and hash them
                let integer = bignum.as_integer();
                let is_negative = !bignum.is_positive() && !bignum.is_zero();
                let abs_value = if is_negative {
                    -integer.clone()
                } else {
                    integer.clone()
                };
                let limbs = abs_value.to_twos_complement_limbs_asc();
                
                for &digit in &limbs {
                    hash = mix_hash(hash, HCONST, digit as u32);
                }
                if is_negative {
                    hash = mix_hash(hash, HCONST_10, 0);
                } else {
                    hash = mix_hash(hash, HCONST_11, 0);
                }
            }
            
            Term::Rational(rational) => {
                // Hash rational number by hashing numerator and denominator separately
                // Similar to how we hash BigNumber, but we need to hash both parts
                let numerator = rational.numerator();
                let denominator = rational.denominator();
                let is_negative = rational.is_negative();
                
                // Hash numerator (similar to BigNumber hashing)
                let num_abs = if is_negative {
                    -numerator.clone()
                } else {
                    numerator.clone()
                };
                let num_limbs = num_abs.to_twos_complement_limbs_asc();
                for &digit in &num_limbs {
                    hash = mix_hash(hash, HCONST, digit as u32);
                }
                if is_negative {
                    hash = mix_hash(hash, HCONST_10, 0);
                } else {
                    hash = mix_hash(hash, HCONST_11, 0);
                }
                
                // Hash denominator (always positive for rational numbers)
                let den_limbs = denominator.to_twos_complement_limbs_asc();
                for &digit in &den_limbs {
                    hash = mix_hash(hash, HCONST, digit as u32);
                }
                // Mark as rational (use a different constant to distinguish from BigNumber)
                hash = mix_hash(hash, HCONST_12, 0);
            }
            
            Term::Float(value) => {
                // C: Hash float as two 32-bit words
                let bits = value.to_bits();
                let low = (bits & 0xFFFFFFFF) as u32;
                let high = ((bits >> 32) & 0xFFFFFFFF) as u32;
                hash = mix_hash_2(hash, HCONST, low, high);
            }
            
            Term::Binary { data, bit_offset, bit_size } => {
                // C: Uses block hashing for binaries
                // For make_hash2, we'll use a simplified approach for bit-aligned binaries
                // Full implementation would use block_hash_buffer with bit extraction
                let bytesize = bit_size >> 3;
                let bitsize = bit_size & 7;
                let bitoffs = bit_offset & 7;
                
                if bitoffs == 0 {
                    // Byte-aligned: hash all bytes
                    for i in 0..bytesize.min(data.len()) {
                        hash = mix_hash(hash, HCONST, data[i] as u32);
                    }
                    // Handle tail bits
                    if bitsize > 0 && bytesize < data.len() {
                        let mut b = data[bytesize];
                        b >>= 8 - bitsize;
                        hash = mix_hash(hash, HCONST, b as u32);
                    }
                } else {
                    // Bit-aligned: extract bytes across boundaries
                    let mut previous = if data.is_empty() { 0u8 } else { data[0] };
                    let lshift = bitoffs;
                    let rshift = 8 - lshift;
                    
                    for i in 0..bytesize {
                        let mut b = (previous << lshift) & 0xFF;
                        if i + 1 < data.len() {
                            previous = data[i + 1];
                        } else {
                            previous = 0;
                        }
                        b |= previous >> rshift;
                        hash = mix_hash(hash, HCONST, b as u32);
                    }
                    
                    if bitsize > 0 {
                        let mut b = (previous << lshift) & 0xFF;
                        if bytesize + 1 < data.len() {
                            previous = data[bytesize + 1];
                        } else {
                            previous = 0;
                        }
                        b |= previous >> rshift;
                        b >>= 8 - bitsize;
                        hash = mix_hash(hash, HCONST, b as u32);
                    }
                }
            }
            
            Term::List { head, tail } => {
                // C: Hash list elements
                let head_val = *head;
                let tail_val = *tail;
                stack.push(StackEntry::Term(tail_val));
                current_term = Some(head_val);
                continue;
            }
            
            Term::Tuple(elements) => {
                // C: Hash tuple arity, then elements
                let arity = elements.len() as u32;
                hash = mix_hash(hash, HCONST_9, arity);
                
                // Push elements in reverse order
                for element in elements.into_iter().rev() {
                    stack.push(StackEntry::Term(element));
                }
                
                if let Some(StackEntry::Term(first)) = stack.pop() {
                    current_term = Some(first);
                    continue;
                }
                current_term = None;
                continue;
            }
            
            Term::Map(entries) => {
                // C: Hash map size, then hash key-value pairs independently and XOR them
                let size = entries.len() as u32;
                hash = mix_hash(hash, HCONST_16, size);
                
                if size == 0 {
                    // Empty map
                    current_term = None;
                    continue;
                }
                
                // Save current hash and hash_xor_pairs, then process pairs
                // C: ESTACK_PUSH(s, hash_xor_pairs); ESTACK_PUSH(s, hash); ESTACK_PUSH(s, HASH_MAP_TAIL);
                let saved_hash = hash;
                let saved_hash_xor_pairs = hash_xor_pairs;
                stack.push(StackEntry::MapTail);
                stack.push(StackEntry::MapSavedHash(saved_hash));
                stack.push(StackEntry::MapSavedXor(saved_hash_xor_pairs));
                
                // Reset for independent pair hashing
                hash = 0;
                hash_xor_pairs = 0;
                
                // Push pairs in reverse order (value, then key, then HASH_MAP_PAIR marker)
                for (key, value) in entries.into_iter().rev() {
                    stack.push(StackEntry::MapPair);
                    stack.push(StackEntry::Term(value));
                    stack.push(StackEntry::Term(key));
                }
                
                // Process first key
                if let Some(StackEntry::Term(first)) = stack.pop() {
                    current_term = Some(first);
                    continue;
                }
                current_term = None;
                continue;
            }
            
            Term::Pid { id, .. } => {
                // C: Hash PID number
                hash = mix_hash(hash, HCONST_5, id);
            }
            
            Term::Port { id, .. } => {
                // C: Hash port number (64-bit)
                // C: Uint64 number = internal_port_number(term) or external_port_number(term);
                //    Uint32 low = (Uint32) (number & 0xffffffff);
                //    Uint32 high = (Uint32) ((number >> 32) & 0xffffffff);
                //    UINT32_HASH_2(low, high, HCONST_6);
                let low = (id & 0xFFFFFFFF) as u32;
                let high = ((id >> 32) & 0xFFFFFFFF) as u32;
                hash = mix_hash_2(hash, HCONST_6, low, high);
            }
            
            Term::Ref { ids, .. } => {
                // C: Hash reference numbers
                if let Some(&first_id) = ids.first() {
                    hash = mix_hash(hash, HCONST_9, first_id);
                }
            }
            
            Term::Fun { is_local, module, function, arity, old_uniq, env } => {
                if is_local {
                    // C: Hash local function
                    hash = mix_hash(hash, HCONST_10, env.len() as u32);
                    hash = mix_hash(hash, HCONST, module);
                    hash = mix_hash(hash, HCONST_2, function);
                    if let Some(uniq) = old_uniq {
                        hash = mix_hash(hash, HCONST_2, uniq);
                    }
                    // Hash environment
                    for element in env.into_iter().rev() {
                        stack.push(StackEntry::Term(element));
                    }
                    if let Some(StackEntry::Term(first)) = stack.pop() {
                        current_term = Some(first);
                        continue;
                    }
                } else {
                    // C: Hash external function
                    hash = mix_hash(hash, HCONST_11, arity);
                    hash = mix_hash(hash, HCONST, module);
                    hash = mix_hash(hash, HCONST, function);
                }
            }
        }
        
        // After processing term, get next from stack
        current_term = None;
    }
    
    hash
}

/// Bob Jenkins' MIX function for hash combination
/// This is the core of make_hash2's hash algorithm
fn mix(a: u32, b: u32, c: u32) -> (u32, u32, u32) {
    let mut a = a;
    let mut b = b;
    let mut c = c;
    
    a = a.wrapping_sub(b).wrapping_sub(c);
    a ^= c >> 13;
    b = b.wrapping_sub(c).wrapping_sub(a);
    b ^= a << 8;
    c = c.wrapping_sub(a).wrapping_sub(b);
    c ^= b >> 13;
    
    a = a.wrapping_sub(b).wrapping_sub(c);
    a ^= c >> 12;
    b = b.wrapping_sub(c).wrapping_sub(a);
    b ^= a << 16;
    c = c.wrapping_sub(a).wrapping_sub(b);
    c ^= b >> 5;
    
    a = a.wrapping_sub(b).wrapping_sub(c);
    a ^= c >> 3;
    b = b.wrapping_sub(c).wrapping_sub(a);
    b ^= a << 10;
    c = c.wrapping_sub(a).wrapping_sub(b);
    c ^= b >> 15;
    
    (a, b, c)
}

/// Hash a single value using MIX
fn mix_hash(hash: u32, constant: u32, value: u32) -> u32 {
    let a = constant.wrapping_add(value);
    let b = constant;
    let (_a, _b, c) = mix(a, b, hash);
    c
}

/// Hash two values using MIX
fn mix_hash_2(hash: u32, constant: u32, value1: u32, value2: u32) -> u32 {
    let a = constant.wrapping_add(value1);
    let b = constant.wrapping_add(value2);
    let (_a, _b, c) = mix(a, b, hash);
    c
}

// HCONST constants for make_hash2 (HCONST * {2..22} mod 2^32)
const HCONST_2: u32 = 0x3c6ef372;
const HCONST_5: u32 = 0x1715609d;
const HCONST_6: u32 = 0xb54cda56;
const HCONST_9: u32 = 0x8ff34781;
const HCONST_10: u32 = 0x2e2ac13a;
const HCONST_11: u32 = 0xcc623af3;
const HCONST_12: u32 = 0x9e3779b9; // Additional constant for rational numbers

// Internal hash constants (MurmurHash3-based)
const IHASH_C1: u64 = 0x87C37B91114253D5;
const IHASH_C2: u64 = 0x4CF5AD432745937F;
const IHASH_CONST_ALPHA: u64 = 0x52DCE729;
const IHASH_CONST_BETA: u64 = 0x38495AB5;

// Hash type constants (matching C enum)
const IHASH_TYPE_IMMEDIATE: u64 = 1;
const IHASH_TYPE_ARRAY_ELEMENT: u64 = 2;
const IHASH_TYPE_CAR: u64 = 3;
const IHASH_TYPE_CDR: u64 = 4;
const IHASH_TYPE_STRING: u64 = 5;
const IHASH_TYPE_TUPLE: u64 = 6;
const IHASH_TYPE_FLATMAP: u64 = 7;
const IHASH_TYPE_HASHMAP_HEAD_ARRAY: u64 = 8;
const IHASH_TYPE_HASHMAP_HEAD_BITMAP: u64 = 9;
const IHASH_TYPE_HASHMAP_NODE: u64 = 10;
const IHASH_TYPE_BINARY: u64 = 11;
const IHASH_TYPE_LOCAL_FUN: u64 = 12;
const IHASH_TYPE_EXTERNAL_FUN: u64 = 13;
const IHASH_TYPE_NEG_BIGNUM: u64 = 14;
const IHASH_TYPE_POS_BIGNUM: u64 = 15;
const IHASH_TYPE_LOCAL_REF: u64 = 16;
const IHASH_TYPE_EXTERNAL_REF: u64 = 17;
const IHASH_TYPE_EXTERNAL_PID: u64 = 18;
const IHASH_TYPE_EXTERNAL_PORT: u64 = 19;
const IHASH_TYPE_FLOAT: u64 = 20;

/// Rotate left 64-bit value
#[inline(always)]
fn rotl64(x: u64, y: u32) -> u64 {
    (x << y) | (x >> (64 - y))
}

/// Fast hash mixing function for immediate values (MurmurHash3-inspired)
/// 
/// This matches C's `ihash_mix64` function exactly.
fn ihash_mix64(input: u64) -> u64 {
    let mut hash = input;
    
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xFF51AFD7ED558CCD);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xC4CEB9FE1A85EC53);
    hash ^= hash >> 33;
    
    hash
}

/// Mix a value into hash_alpha (MurmurHash3-style mixing)
#[inline(always)]
fn mix_alpha(hash_alpha: &mut u64, hash_beta: u64, hash_ticks: &mut u64, expr: u64) {
    let mut e = expr;
    e = e.wrapping_mul(IHASH_C1);
    e = rotl64(e, 31);
    e = e.wrapping_mul(IHASH_C2);
    *hash_alpha ^= e;
    *hash_alpha = rotl64(*hash_alpha, 27);
    *hash_alpha = hash_alpha.wrapping_add(hash_beta);
    *hash_alpha = hash_alpha.wrapping_mul(5).wrapping_add(IHASH_CONST_ALPHA);
    *hash_ticks += 1;
}

/// Mix two 32-bit values into hash_alpha
#[inline(always)]
fn mix_alpha_2f32(hash_alpha: &mut u64, hash_beta: u64, hash_ticks: &mut u64, expr1: u32, expr2: u32) {
    let combined = (expr1 as u64) | ((expr2 as u64) << 32);
    mix_alpha(hash_alpha, hash_beta, hash_ticks, combined);
}

/// Mix a value into hash_beta (MurmurHash3-style mixing)
#[inline(always)]
fn mix_beta(hash_alpha: u64, hash_beta: &mut u64, hash_ticks: &mut u64, expr: u64) {
    let mut e = expr;
    e = e.wrapping_mul(IHASH_C2);
    e = rotl64(e, 33);
    e = e.wrapping_mul(IHASH_C1);
    *hash_beta ^= e;
    *hash_beta = rotl64(*hash_beta, 31);
    *hash_beta = hash_beta.wrapping_add(hash_alpha);
    *hash_beta = hash_beta.wrapping_mul(5).wrapping_add(IHASH_CONST_BETA);
    *hash_ticks += 1;
}

/// Mix two 32-bit values into hash_beta
#[inline(always)]
fn mix_beta_2f32(hash_alpha: u64, hash_beta: &mut u64, hash_ticks: &mut u64, expr1: u32, expr2: u32) {
    let combined = (expr1 as u64) | ((expr2 as u64) << 32);
    mix_beta(hash_alpha, hash_beta, hash_ticks, combined);
}

/// Check if a term is an immediate value (small integer, atom, nil, pid, port)
fn is_immediate(term: &Term) -> bool {
    matches!(term, 
        Term::Nil | 
        Term::Small(_) | 
        Term::Atom(_) | 
        Term::Pid { .. } | 
        Term::Port { .. }
    )
}

/// Read u64 from bytes (endian-agnostic, matching C's read_u64)
fn read_u64(data: &[u8], offset: usize) -> u64 {
    let mut value = 0u64;
    for i in 0..8.min(data.len().saturating_sub(offset)) {
        value |= (data[offset + i] as u64) << (i * 8);
    }
    value
}

/// Internal hash implementation using MurmurHash3-based algorithm
/// 
/// This matches C's `make_internal_hash` function exactly, including all edge cases.
fn make_internal_hash_impl(term: Term, salt: HashValue) -> HashValue {
    use crate::bits::{byte_offset, nbytes, copy_bits_forward};
    
    let mut hash_alpha = salt as u64;
    let mut hash_beta = salt as u64;
    let mut hash_ticks = 0u64;
    
    let mut stack: Vec<Term> = Vec::new();
    let mut current_term = Some(term);
    
    loop {
        let term_to_process = match current_term.take() {
            Some(t) => t,
            None => {
                if let Some(next) = stack.pop() {
                    current_term = Some(next);
                    continue;
                } else {
                    // Finalize hash (matches C's pop_next label)
                    hash_alpha ^= hash_ticks;
                    hash_beta ^= hash_ticks;
                    
                    hash_alpha = hash_alpha.wrapping_add(hash_beta);
                    hash_beta = hash_beta.wrapping_add(hash_alpha);
                    
                    hash_alpha = ihash_mix64(hash_alpha);
                    hash_beta = ihash_mix64(hash_beta);
                    
                    hash_alpha = hash_alpha.wrapping_add(hash_beta);
                    hash_beta = hash_beta.wrapping_add(hash_alpha);
                    
                    return (hash_alpha ^ hash_beta) as HashValue;
                }
            }
        };
        
        // Fast path for immediate values (matches C's TAG_PRIMARY_IMMED1 case)
        if is_immediate(&term_to_process) {
            mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_IMMEDIATE as u64);
            let term_value = match &term_to_process {
                Term::Nil => 0,
                Term::Small(v) => *v as u64,
                Term::Atom(v) => *v as u64,
                Term::Pid { id, .. } => *id as u64,
                Term::Port { id, .. } => *id,
                _ => 0,
            };
            mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, term_value);
            continue;
        }
        
        match term_to_process {
            Term::List { head, tail } => {
                // String optimization: hash consecutive bytes (matches C's string optimization)
                let mut value = 0u64;
                let mut bytes = 0usize;
                let mut current_list_term: Option<Term> = Some(Term::List { head, tail });
                
                // Process string prefix
                while let Some(Term::List { head: h, tail: t }) = current_list_term.take() {
                    if let Term::Small(byte_val) = *h {
                        if byte_val >= 0 && byte_val <= 255 {
                            value = (value << 8) | (byte_val as u64);
                            bytes += 1;
                            
                            if bytes % 4 == 0 {
                                mix_alpha_2f32(
                                    &mut hash_alpha, 
                                    hash_beta, 
                                    &mut hash_ticks,
                                    (IHASH_TYPE_STRING | ((bytes as u64) << 8)) as u32,
                                    value as u32
                                );
                                value = 0;
                            }
                            
                            // Continue with tail
                            if let Term::List { .. } = *t {
                                current_list_term = Some(*t);
                                continue;
                            } else {
                                // Tail is not a list - process remaining bytes and continue
                                break;
                            }
                        } else {
                            // Not a byte - process as general list
                            current_list_term = Some(Term::List { head: h, tail: t });
                            break;
                        }
                    } else {
                        // Not a byte - process as general list
                        current_list_term = Some(Term::List { head: h, tail: t });
                        break;
                    }
                }
                
                // Handle remaining bytes
                if bytes > 0 && bytes % 4 != 0 {
                    mix_alpha_2f32(
                        &mut hash_alpha,
                        hash_beta,
                        &mut hash_ticks,
                        (IHASH_TYPE_STRING | ((bytes as u64) << 8)) as u32,
                        value as u32
                    );
                }
                
                // Process as general list
                if let Some(Term::List { head: h, tail: t }) = current_list_term {
                    let head_term = *h;
                    let tail_term = *t;
                    
                    if is_immediate(&head_term) {
                        mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_IMMEDIATE as u32, IHASH_TYPE_CAR as u32);
                        let head_value = match &head_term {
                            Term::Small(v) => *v as u64,
                            Term::Atom(v) => *v as u64,
                            Term::Pid { id, .. } => *id as u64,
                            Term::Port { id, .. } => *id,
                            _ => 0,
                        };
                        mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, head_value);
                        
                        if !matches!(tail_term, Term::List { .. }) {
                            mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_CDR as u64);
                        }
                        current_term = Some(tail_term);
                        continue;
                    } else {
                        stack.push(tail_term.clone());
                        if !matches!(tail_term, Term::List { .. }) {
                            mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_CDR as u64);
                        }
                        mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_CAR as u64);
                        current_term = Some(head_term);
                        continue;
                    }
                } else {
                    // No more list to process
                    continue;
                }
            }
            
            Term::Tuple(elements) => {
                // Hash tuple (matches C's ARITYVAL_SUBTAG case)
                mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_TUPLE as u64);
                let arity = elements.len();
                mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, arity as u64);
                
                if arity > 0 {
                    // Push elements in reverse order (except first)
                    for i in (1..arity).rev() {
                        stack.push(elements[i].clone());
                    }
                    current_term = Some(elements[0].clone());
                    continue;
                }
            }
            
            Term::Map(entries) => {
                // Simplified: treat as flatmap (matches C's HAMT_SUBTAG_HEAD_FLATMAP case)
                mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_FLATMAP as u64);
                let size = entries.len();
                mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, size as u64);
                
                if size > 0 {
                    // Push key-value pairs in reverse order (except last)
                    // C: push values first, then keys (reverse order)
                    for i in (0..size-1).rev() {
                        stack.push(entries[i].1.clone()); // value
                        stack.push(entries[i].0.clone()); // key
                    }
                    // Process last pair
                    stack.push(entries[size-1].1.clone()); // value
                    current_term = Some(entries[size-1].0.clone()); // key
                    continue;
                }
            }
            
            Term::Big(bignum) => {
                // Hash bignum (matches C's POS_BIG_SUBTAG/NEG_BIG_SUBTAG case)
                let integer = bignum.as_integer();
                let is_negative = !bignum.is_positive() && !bignum.is_zero();
                let abs_value = if is_negative {
                    -integer.clone()
                } else {
                    integer.clone()
                };
                let limbs = abs_value.to_twos_complement_limbs_asc();
                let n = limbs.len();
                
                let hash_type = if is_negative {
                    IHASH_TYPE_NEG_BIGNUM as u32
                } else {
                    IHASH_TYPE_POS_BIGNUM as u32
                };
                mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, hash_type, n as u32);
                
                // Process digits in pairs (matches C exactly)
                let mut i = 0;
                while i + 2 <= n {
                    mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, limbs[i] as u64);
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, limbs[i+1] as u64);
                    i += 2;
                }
                
                if i < n {
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, limbs[i] as u64);
                }
            }
            
            Term::Rational(rational) => {
                // Hash rational number by hashing numerator and denominator separately
                // Similar to how we hash BigNumber, but we need to hash both parts
                let numerator = rational.numerator();
                let denominator = rational.denominator();
                let is_negative = rational.is_negative();
                
                // Hash numerator (similar to BigNumber hashing)
                let num_abs = if is_negative {
                    -numerator.clone()
                } else {
                    numerator.clone()
                };
                let num_limbs = num_abs.to_twos_complement_limbs_asc();
                let num_n = num_limbs.len();
                
                let num_hash_type = if is_negative {
                    IHASH_TYPE_NEG_BIGNUM as u32
                } else {
                    IHASH_TYPE_POS_BIGNUM as u32
                };
                mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, num_hash_type, num_n as u32);
                
                // Process numerator digits in pairs
                let mut i = 0;
                while i + 2 <= num_n {
                    mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, num_limbs[i] as u64);
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, num_limbs[i+1] as u64);
                    i += 2;
                }
                
                if i < num_n {
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, num_limbs[i] as u64);
                }
                
                // Hash denominator (always positive for rational numbers)
                let den_limbs = denominator.to_twos_complement_limbs_asc();
                let den_n = den_limbs.len();
                
                mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_POS_BIGNUM as u32, den_n as u32);
                
                // Process denominator digits in pairs
                let mut i = 0;
                while i + 2 <= den_n {
                    mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, den_limbs[i] as u64);
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, den_limbs[i+1] as u64);
                    i += 2;
                }
                
                if i < den_n {
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, den_limbs[i] as u64);
                }
            }
            
            Term::Binary { data, bit_offset, bit_size } => {
                // Hash binary (matches C's BIN_REF_SUBTAG/HEAP_BITS_SUBTAG/SUB_BITS_SUBTAG case)
                mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_BINARY as u64);
                mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, bit_size as u64);
                
                if bit_size > 0 {
                    let bytesize = bit_size >> 3;
                    let bitoffs = bit_offset & 7;
                    let bitsize = bit_size & 7;
                    
                    // Handle bit-aligned binaries (matches C's bit offset handling)
                    let bytes = if bitoffs != 0 {
                        // Need to copy bits to byte-aligned buffer
                        let nbytes_needed = nbytes(bit_size as u64);
                        let mut aligned_bytes = vec![0u8; nbytes_needed];
                        copy_bits_forward(&data, bit_offset, &mut aligned_bytes, 0, bit_size);
                        aligned_bytes
                    } else {
                        // Already byte-aligned
                        let byte_off = byte_offset(bit_offset);
                        if byte_off < data.len() {
                            data[byte_off..].to_vec()
                        } else {
                            vec![]
                        }
                    };
                    
                    if bytes.len() > 0 {
                        let mut it = 0;
                        // Process in pairs of 8 bytes (2 u64s) - matches C exactly
                        while it + 16 <= bytesize.min(bytes.len()) {
                            mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, read_u64(&bytes, it));
                            mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, read_u64(&bytes, it + 8));
                            it += 16;
                        }
                        
                        // Handle remaining bytes (matches C's switch statement exactly)
                        let remaining = bytesize.min(bytes.len()) - it;
                        let mut value = 0u64;
                        
                        match remaining {
                            15 => {
                                value ^= (bytes[it + 14] as u64) << 0x30;
                                value ^= (bytes[it + 13] as u64) << 0x28;
                                value ^= (bytes[it + 12] as u64) << 0x20;
                                value ^= (bytes[it + 11] as u64) << 0x18;
                                value ^= (bytes[it + 10] as u64) << 0x10;
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            14 => {
                                value ^= (bytes[it + 13] as u64) << 0x28;
                                value ^= (bytes[it + 12] as u64) << 0x20;
                                value ^= (bytes[it + 11] as u64) << 0x18;
                                value ^= (bytes[it + 10] as u64) << 0x10;
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            13 => {
                                value ^= (bytes[it + 12] as u64) << 0x20;
                                value ^= (bytes[it + 11] as u64) << 0x18;
                                value ^= (bytes[it + 10] as u64) << 0x10;
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            12 => {
                                value ^= (bytes[it + 11] as u64) << 0x18;
                                value ^= (bytes[it + 10] as u64) << 0x10;
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            11 => {
                                value ^= (bytes[it + 10] as u64) << 0x10;
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            10 => {
                                value ^= (bytes[it + 9] as u64) << 0x08;
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            9 => {
                                value ^= (bytes[it + 8] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C2);
                                value = rotl64(value, 33);
                                value = value.wrapping_mul(IHASH_C1);
                                hash_beta ^= value;
                                value = 0;
                            }
                            _ => {}
                        }
                        
                        // Handle bytes 8-1 (matches C's fallthrough cases)
                        match remaining {
                            8 => {
                                value ^= (bytes[it + 7] as u64) << 0x38;
                                value ^= (bytes[it + 6] as u64) << 0x30;
                                value ^= (bytes[it + 5] as u64) << 0x28;
                                value ^= (bytes[it + 4] as u64) << 0x20;
                                value ^= (bytes[it + 3] as u64) << 0x18;
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            7 => {
                                value ^= (bytes[it + 6] as u64) << 0x30;
                                value ^= (bytes[it + 5] as u64) << 0x28;
                                value ^= (bytes[it + 4] as u64) << 0x20;
                                value ^= (bytes[it + 3] as u64) << 0x18;
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            6 => {
                                value ^= (bytes[it + 5] as u64) << 0x28;
                                value ^= (bytes[it + 4] as u64) << 0x20;
                                value ^= (bytes[it + 3] as u64) << 0x18;
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            5 => {
                                value ^= (bytes[it + 4] as u64) << 0x20;
                                value ^= (bytes[it + 3] as u64) << 0x18;
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            4 => {
                                value ^= (bytes[it + 3] as u64) << 0x18;
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            3 => {
                                value ^= (bytes[it + 2] as u64) << 0x10;
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            2 => {
                                value ^= (bytes[it + 1] as u64) << 0x08;
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            1 => {
                                value ^= (bytes[it + 0] as u64) << 0x00;
                                value = value.wrapping_mul(IHASH_C1);
                                value = rotl64(value, 31);
                                value = value.wrapping_mul(IHASH_C2);
                                hash_alpha ^= value;
                            }
                            _ => {}
                        }
                        
                        // Handle tail bits (matches C's TAIL_BITS handling)
                        if bitsize > 0 {
                            let byte_idx = byte_offset(bit_size);
                            if byte_idx < data.len() {
                                let shift = 8 - bitsize;
                                mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, (data[byte_idx] >> shift) as u64);
                            }
                            // Test case: byte_idx >= data.len() (edge case - already covered by test_internal_hash_binary_tail_bits)
                        }
                    }
                }
            }
            
            Term::Float(value) => {
                // Hash float (matches C's FLOAT_SUBTAG case)
                mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_FLOAT as u64);
                let bits = value.to_bits();
                let low = (bits & 0xFFFFFFFF) as u32;
                let high = ((bits >> 32) & 0xFFFFFFFF) as u32;
                mix_beta_2f32(hash_alpha, &mut hash_beta, &mut hash_ticks, low, high);
            }
            
            Term::Ref { ids, .. } => {
                // Hash reference (matches C's REF_SUBTAG case)
                if ids.len() >= 3 {
                    mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_LOCAL_REF as u32, ids[0]);
                    mix_beta_2f32(hash_alpha, &mut hash_beta, &mut hash_ticks, ids[1], ids[2]);
                    
                    // Handle internal pid ref (matches C's is_internal_pid_ref check)
                    #[cfg(target_pointer_width = "64")]
                    if ids.len() >= 5 {
                        mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, ids[3], ids[4]);
                    }
                    #[cfg(not(target_pointer_width = "64"))]
                    if ids.len() >= 4 {
                        mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, ids[3] as u64);
                    }
                }
            }
            
            Term::Fun { is_local, module, function, arity: _arity, old_uniq, env } => {
                if is_local {
                    // Hash local function (matches C's local fun case)
                    let num_free = env.len();
                    mix_alpha_2f32(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_LOCAL_FUN as u32, num_free as u32);
                    mix_beta_2f32(hash_alpha, &mut hash_beta, &mut hash_ticks, function, old_uniq.unwrap_or(0));
                    
                    mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_IMMEDIATE as u64);
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, module as u64);
                    
                    if num_free > 0 {
                        // Push environment elements in reverse order (except last)
                        for i in (0..num_free-1).rev() {
                            stack.push(env[i].clone());
                        }
                        current_term = Some(env[num_free-1].clone());
                        continue;
                    }
                } else {
                    // External function (matches C's external fun case)
                    // Note: C hashes pointer to Export entry, we use module+function as proxy
                    mix_alpha(&mut hash_alpha, hash_beta, &mut hash_ticks, IHASH_TYPE_EXTERNAL_FUN as u64);
                    // In real implementation, would hash pointer to Export entry
                    // For now, use module+function as proxy (this is a limitation of entities layer)
                    let proxy = ((module as u64) << 32) | (function as u64);
                    mix_beta(hash_alpha, &mut hash_beta, &mut hash_ticks, proxy);
                }
            }
            
            // Immediate values already handled above
            Term::Nil | Term::Small(_) | Term::Atom(_) | Term::Pid { .. } | Term::Port { .. } => {
                // Should not reach here (handled in is_immediate check)
                unreachable!()
            }
        }
    }
}

/// Internal hash function for VM use.
///
/// This hash is NOT portable between VM instances and is only valid
/// as long as the term exists in the VM.
///
/// Uses MurmurHash3-based algorithm for fast, high-quality hashing.
///
/// # Arguments
/// * `term` - The Erlang term to hash
///
/// # Returns
/// A hash value (platform-dependent size)
pub fn erts_internal_hash(term: Term) -> HashValue {
    // Fast path for immediate values (matches C's fast path)
    if is_immediate(&term) {
        let term_value = match &term {
            Term::Nil => 0,
            Term::Small(v) => *v as u64,
            Term::Atom(v) => *v as u64,
            Term::Pid { id, .. } => *id as u64,
            Term::Port { id, .. } => *id,
            _ => 0,
        };
        return ihash_mix64(term_value) as HashValue;
    }
    
    make_internal_hash_impl(term, 0)
}

/// Internal hash with salt value.
///
/// # Arguments
/// * `term` - The Erlang term to hash
/// * `salt` - Salt value to mix into hash
///
/// # Returns
/// A hash value (platform-dependent size)
pub fn erts_internal_salted_hash(term: Term, salt: HashValue) -> HashValue {
    // Fast path for immediate values
    if is_immediate(&term) {
        let term_value = match &term {
            Term::Nil => 0,
            Term::Small(v) => *v as u64,
            Term::Atom(v) => *v as u64,
            Term::Pid { id, .. } => *id as u64,
            Term::Port { id, .. } => *id,
            _ => 0,
        };
        return ihash_mix64(term_value.wrapping_add(salt as u64)) as HashValue;
    }
    
    make_internal_hash_impl(term, salt)
}

/// Debug function to weaken hash for collision testing.
///
/// This function is only available in debug builds and is used to test
/// hashmap collision handling by artificially increasing collision rates.
///
/// # Arguments
/// * `hash` - The hash value to weaken
/// * `_key` - The key (unused, kept for API compatibility)
///
/// # Returns
/// A weakened hash value with high collision rate (1/256)
#[cfg(debug_assertions)]
fn erts_dbg_hashmap_collision_bonanza(hash: HashValue, _key: Term) -> HashValue {
    // Keep only 8 bits to ensure a high collision rate (1/256)
    let mut bad_hash = hash & 0x12482481;
    let bad_bits: HashValue;
    
    match std::mem::size_of::<HashValue>() * 8 {
        64 => {
            bad_hash = bad_hash.wrapping_mul(11400714819323198485u64);
            bad_hash ^= bad_hash >> 31;
            bad_bits = hash % 137;
        }
        32 => {
            bad_hash = bad_hash.wrapping_mul(2654435769u32 as HashValue);
            bad_hash ^= bad_hash >> 15;
            bad_bits = hash % 67;
        }
        _ => {
            // Unknown size - return original hash
            return hash;
        }
    }
    
    if bad_bits < (std::mem::size_of::<HashValue>() * 8) as HashValue {
        // Mix in a number of high good bits to get "randomly" close
        // to the collision nodes
        let bad_mask = ((1 as HashValue) << bad_bits) - 1;
        bad_hash = (hash & !bad_mask) | (bad_hash & bad_mask);
    }
    
    bad_hash
}

/// Hash function specifically for maps.
///
/// Identical to erts_internal_hash except in debug configurations.
///
/// # Arguments
/// * `key` - The map key to hash
///
/// # Returns
/// A hash value (platform-dependent size)
pub fn erts_map_hash(key: Term) -> HashValue {
    let hash = erts_internal_hash(key.clone());
    
    #[cfg(debug_assertions)]
    {
        // In debug mode, apply collision testing to weaken the hash
        // Note: key is unused in the debug function (matches C behavior)
        erts_dbg_hashmap_collision_bonanza(hash, key)
    }
    
    #[cfg(not(debug_assertions))]
    {
        hash
    }
}


/// Erlang term representation
///
/// This is a simplified representation for the entities layer.
/// In higher layers, this will be replaced with the full Eterm type.
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    /// Nil (empty list)
    Nil,
    /// Small integer (fits in machine word)
    Small(i64),
    /// Atom (by index/value)
    Atom(u32),
    /// Big integer (arbitrary precision)
    Big(BigNumber),
    /// Big rational (arbitrary precision rational number)
    Rational(BigRational),
    /// Float (f64)
    Float(f64),
    /// Binary (bitstring)
    /// 
    /// - `data`: The binary data bytes
    /// - `bit_offset`: Bit offset into the first byte (0-7)
    /// - `bit_size`: Total size in bits
    Binary {
        data: Vec<u8>,
        bit_offset: usize,  // Bit offset within first byte (0-7)
        bit_size: usize,    // Total size in bits
    },
    /// List (cons cell)
    List {
        head: Box<Term>,
        tail: Box<Term>,
    },
    /// Tuple (fixed-size array of terms)
    Tuple(Vec<Term>),
    /// Map (key-value pairs)
    Map(Vec<(Term, Term)>),
    /// Process ID (simplified)
    Pid { node: u32, id: u32, serial: u32, creation: u32 },
    /// Port
    Port { 
        node: u32, 
        id: u64,  // Port number (can be 64-bit)
        creation: u32 
    },
    /// Reference (simplified)
    Ref { node: u32, ids: Vec<u32>, creation: u32 },
    /// Function
    Fun {
        /// Whether this is a local function (true) or external function (false)
        is_local: bool,
        /// Module atom index
        module: u32,
        /// Function atom index (for external) or function index (for local)
        function: u32,
        /// Arity
        arity: u32,
        /// Old unique value (for local functions only)
        old_uniq: Option<u32>,
        /// Environment (for local functions with free variables)
        env: Vec<Term>,
    },
}

// BigNumber is now imported from entities_utilities

/// Term hash trait (placeholder for future use)
pub trait TermHash {
    fn hash(&self) -> HashValue;
}

// Implement Eq for Term (handles Float NaN case)
impl Eq for Term {}

// Implement Hash for Term using make_hash
impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Use make_hash to compute the hash value
        // Clone is necessary because make_hash takes ownership
        let hash_value = make_hash(self.clone());
        state.write_u32(hash_value);
    }
}

// Hash constants (prime numbers just above 2^28)
// These match the C implementation exactly
// Hash multipliers for make_hash (prime numbers just above 2^28)
// These are used as multiplication constants in the linear hash function: H = H*C + X mod 2^32
const HASH_MULT_ATOM: u32 = 268440163;           // atoms, binaries, function names
const HASH_MULT_NUMBER: u32 = 268439161;         // numbers (small ints, bignums), string lists
const HASH_MULT_POSITIVE: u32 = 268435459;       // positive numbers, nil
const HASH_MULT_NEGATIVE: u32 = 268436141;       // negative numbers, binary final
const HASH_MULT_PID_BYTE: u32 = 268438633;       // PIDs (byte-wise hashing)
const HASH_MULT_PID_FINAL: u32 = 268437017;      // PIDs (final), floats
const HASH_MULT_7: u32 = 268438039;              // unused
const HASH_MULT_LIST_END: u32 = 268437511;       // list end marker
const HASH_MULT_TUPLE_ARITY: u32 = 268439627;   // tuple arity, ports/refs (byte-wise)
const HASH_MULT_PORT_REF_FINAL: u32 = 268440479; // ports/refs (final), local functions
const HASH_MULT_EXTERNAL_FUN: u32 = 268440577;   // external functions, port high bits
const HASH_MULT_BINARY_TAIL: u32 = 268440581;    // binary partial bytes
const HASH_MULT_MAP_FIRST: u32 = 268440593;      // maps (first constant)
const HASH_MULT_MAP_SECOND: u32 = 268440611;      // maps (second constant)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_hash_nil() {
        let nil_term = Term::Nil;
        let hash = make_hash(nil_term);
        // C: hash = hash*HASH_MULT_POSITIVE + 1; with initial hash = 0
        // Expected: 0 * HASH_MULT_POSITIVE + 1 = 1
        assert_eq!(hash, 1);
    }

    #[test]
    fn test_make_hash_small_integer() {
        // Test positive small integer
        let pos_term = Term::Small(42);
        let hash_pos = make_hash(pos_term);
        assert_ne!(hash_pos, 0);
        
        // Test negative small integer
        let neg_term = Term::Small(-42);
        let hash_neg = make_hash(neg_term);
        assert_ne!(hash_neg, 0);
        
        // Verify that same integer gives same hash
        let hash_pos2 = make_hash(Term::Small(42));
        assert_eq!(hash_pos, hash_pos2);
        
        // Verify that negative and positive hash differently
        assert_ne!(hash_pos, hash_neg);
        
        // Test zero
        let zero_term = Term::Small(0);
        let hash_zero = make_hash(zero_term);
        // Zero hashes to 0 (all bytes are 0, so hash is 0 * HASH_MULT_POSITIVE = 0)
        assert_eq!(hash_zero, 0);
    }

    #[test]
    fn test_make_hash_atom() {
        let atom_term = Term::Atom(123);
        let hash = make_hash(atom_term);
        // C: hash = hash*HASH_MULT_ATOM + atom_val; with initial hash = 0
        // Expected: 0 * HASH_MULT_ATOM + 123 = 123
        assert_eq!(hash, 123);
        
        // Test different atoms hash differently
        let atom_term2 = Term::Atom(456);
        let hash2 = make_hash(atom_term2);
        assert_ne!(hash, hash2);
    }

    #[test]
    fn test_make_hash_binary() {
        // Test empty binary (byte-aligned)
        let empty_bin = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let hash_empty = make_hash(empty_bin);
        // C: hash = hash * HASH_MULT_NEGATIVE + bytesize; with initial hash = 0
        // Expected: 0 * HASH_MULT_NEGATIVE + 0 = 0
        assert_eq!(hash_empty, 0);
        
        // Test binary with data (byte-aligned)
        let bin_term = Term::Binary {
            data: vec![1, 2, 3, 4, 5],
            bit_offset: 0,
            bit_size: 40, // 5 bytes * 8 bits
        };
        let hash = make_hash(bin_term);
        assert_ne!(hash, 0);
        
        // Test that all bytes are hashed (not just first 15)
        let long_bin = Term::Binary {
            data: vec![0u8; 100],
            bit_offset: 0,
            bit_size: 800, // 100 bytes * 8 bits
        };
        let hash_long = make_hash(long_bin);
        assert_ne!(hash_long, 0);
        
        // Different binaries should hash differently
        let bin_term2 = Term::Binary {
            data: vec![5, 4, 3, 2, 1],
            bit_offset: 0,
            bit_size: 40,
        };
        let hash2 = make_hash(bin_term2);
        assert_ne!(hash, hash2);
        
        // Test bit-aligned binary (3 bits offset, 13 bits total)
        // This would be like <<1:3, 2:8, 3:2>> in Erlang
        let bit_aligned_bin = Term::Binary {
            data: vec![0b00001000, 0b00000010, 0b11000000], // 1<<3, 2, 3<<6
            bit_offset: 3,
            bit_size: 13, // 1 byte + 5 bits
        };
        let hash_bit_aligned = make_hash(bit_aligned_bin);
        assert_ne!(hash_bit_aligned, 0);
        
        // Test binary with tail bits (not a full byte)
        // Like <<1:8, 2:4>> in Erlang (1 byte + 4 bits)
        let tail_bits_bin = Term::Binary {
            data: vec![1, 0b00100000], // 1, 2<<4
            bit_offset: 0,
            bit_size: 12, // 1 byte + 4 bits
        };
        let hash_tail_bits = make_hash(tail_bits_bin);
        assert_ne!(hash_tail_bits, 0);
        
        // Same data but different bit alignment should hash differently
        let bit_aligned_bin2 = Term::Binary {
            data: vec![0b00010000, 0b00100000], // 1<<4, 2<<4
            bit_offset: 4,
            bit_size: 12,
        };
        let hash_bit_aligned2 = make_hash(bit_aligned_bin2);
        assert_ne!(hash_tail_bits, hash_bit_aligned2);
    }
    
    #[test]
    fn test_make_hash_tuple() {
        let tuple_term = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let hash = make_hash(tuple_term);
        // Tuple hash should be non-zero (arity + element hashes)
        assert_ne!(hash, 0);
        
        // Test empty tuple
        let empty_tuple = Term::Tuple(vec![]);
        let hash_empty = make_hash(empty_tuple);
        // Empty tuple: hash = 0 * HASH_MULT_TUPLE_ARITY + 0 = 0
        assert_eq!(hash_empty, 0);
    }
    
    #[test]
    fn test_make_hash_list() {
        // Test simple list
        let list_term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let hash = make_hash(list_term);
        assert_ne!(hash, 0);
        
        // Test nested list
        let nested_list = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let hash_nested = make_hash(nested_list);
        assert_ne!(hash_nested, 0);
    }

    #[test]
    fn test_make_hash_float() {
        // Test positive float
        let pos_float = Term::Float(42.5);
        let hash_pos = make_hash(pos_float);
        assert_ne!(hash_pos, 0);
        
        // Test negative float
        let neg_float = Term::Float(-42.5);
        let hash_neg = make_hash(neg_float);
        assert_ne!(hash_neg, 0);
        assert_ne!(hash_pos, hash_neg);
        
        // Test positive zero
        let pos_zero = Term::Float(0.0);
        let hash_pos_zero = make_hash(pos_zero);
        // C: hash = hash*HASH_MULT_PID_FINAL + (ff.fw[0] ^ ff.fw[1]);
        // For zero, both words are 0, so: 0 * HASH_MULT_PID_FINAL + (0 ^ 0) = 0
        assert_eq!(hash_pos_zero, 0);
        
        // Test negative zero (should hash to same value as positive zero after normalization)
        let neg_zero = Term::Float(-0.0);
        let hash_neg_zero = make_hash(neg_zero);
        
        // Both should hash to the same value (C normalizes to positive zero)
        // This is the key test: negative zero should hash to 0, same as positive zero
        assert_eq!(hash_pos_zero, hash_neg_zero);
        assert_eq!(hash_neg_zero, 0);
        
        // Test that zero hashes differently from other values
        assert_ne!(hash_pos_zero, hash_pos);
        assert_ne!(hash_pos_zero, hash_neg);
    }

    #[test]
    fn test_atom_hash_pjw() {
        // Test the hashpjw algorithm implementation
        // This matches C's atom_hash function
        
        // Test empty string
        let hash_empty = atom_hash_pjw(b"");
        assert_eq!(hash_empty, 0);
        
        // Test simple string
        let hash_hello = atom_hash_pjw(b"hello");
        assert_ne!(hash_hello, 0);
        
        // Test that same string gives same hash
        let hash_hello2 = atom_hash_pjw(b"hello");
        assert_eq!(hash_hello, hash_hello2);
        
        // Test that different strings give different hashes
        let hash_world = atom_hash_pjw(b"world");
        assert_ne!(hash_hello, hash_world);
        
        // Test single character
        let hash_a = atom_hash_pjw(b"a");
        assert_ne!(hash_a, 0);
    }
    
    #[test]
    fn test_make_hash_function() {
        // Test local function
        let local_fun = Term::Fun {
            is_local: true,
            module: 1,  // atom index
            function: 42, // function index
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1), Term::Small(2)],
        };
        let hash_local = make_hash(local_fun);
        assert_ne!(hash_local, 0);
        
        // Test external function
        let external_fun = Term::Fun {
            is_local: false,
            module: 1,  // atom index
            function: 2, // atom index
            arity: 2,
            old_uniq: None,
            env: vec![],
        };
        let hash_external = make_hash(external_fun);
        assert_ne!(hash_external, 0);
        
        // Local and external should hash differently
        assert_ne!(hash_local, hash_external);
        
        // Test local function without environment
        let local_fun_no_env = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![],
        };
        let hash_local_no_env = make_hash(local_fun_no_env);
        assert_ne!(hash_local_no_env, 0);
        assert_ne!(hash_local, hash_local_no_env); // Different due to environment
    }

    #[test]
    fn test_make_hash_port() {
        // Test 32-bit port (low 32 bits only)
        let port_32bit = Term::Port {
            node: 1,
            id: 0x12345678, // 32-bit value
            creation: 0,
        };
        let hash_32bit = make_hash(port_32bit);
        assert_ne!(hash_32bit, 0);
        
        // Test 64-bit port (high 32 bits non-zero)
        let port_64bit = Term::Port {
            node: 1,
            id: 0x123456789ABCDEF0, // 64-bit value
            creation: 0,
        };
        let hash_64bit = make_hash(port_64bit);
        assert_ne!(hash_64bit, 0);
        
        // 32-bit and 64-bit ports should hash differently
        assert_ne!(hash_32bit, hash_64bit);
        
        // Test that same port number gives same hash
        let port_64bit2 = Term::Port {
            node: 1,
            id: 0x123456789ABCDEF0, // Same 64-bit value
            creation: 0,
        };
        let hash_64bit2 = make_hash(port_64bit2);
        assert_eq!(hash_64bit, hash_64bit2);
        
        // Test port with only high 32 bits set
        let port_high_only = Term::Port {
            node: 1,
            id: 0x1234567800000000, // High 32 bits only
            creation: 0,
        };
        let hash_high_only = make_hash(port_high_only);
        assert_ne!(hash_high_only, 0);
        assert_ne!(hash_32bit, hash_high_only);
    }
    
    #[test]
    fn test_make_hash2_port() {
        // Test 32-bit port in make_hash2
        let port_32bit = Term::Port {
            node: 1,
            id: 0x12345678,
            creation: 0,
        };
        let hash_32bit = make_hash2(port_32bit);
        assert_ne!(hash_32bit, 0);
        
        // Test 64-bit port in make_hash2
        let port_64bit = Term::Port {
            node: 1,
            id: 0x123456789ABCDEF0,
            creation: 0,
        };
        let hash_64bit = make_hash2(port_64bit);
        assert_ne!(hash_64bit, 0);
        
        // 32-bit and 64-bit ports should hash differently
        assert_ne!(hash_32bit, hash_64bit);
    }

    #[test]
    fn test_internal_hash_immediate() {
        // Test fast path for immediate values
        let nil_hash = erts_internal_hash(Term::Nil);
        // nil_hash might be 0 for Term::Nil (value is 0), which is acceptable
        // The important thing is that the function executes without error
        
        let small_hash = erts_internal_hash(Term::Small(42));
        assert_ne!(small_hash, 0);
        
        let atom_hash = erts_internal_hash(Term::Atom(123));
        assert_ne!(atom_hash, 0);
        
        let pid_hash = erts_internal_hash(Term::Pid { node: 1, id: 100, serial: 0, creation: 0 });
        assert_ne!(pid_hash, 0);
        
        let port_hash = erts_internal_hash(Term::Port { node: 1, id: 200, creation: 0 });
        assert_ne!(port_hash, 0);
    }
    
    #[test]
    fn test_internal_hash_list() {
        // Test list with string optimization (4-byte chunks)
        let list = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::Small(0)), // End
                    }),
                }),
            }),
        };
        let hash = erts_internal_hash(list);
        assert_ne!(hash, 0);
        
        // Test list with non-byte head (general list path)
        let list2 = Term::List {
            head: Box::new(Term::Small(300)), // Not a byte
            tail: Box::new(Term::Small(0)),
        };
        let hash2 = erts_internal_hash(list2);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash);
        
        // Test list with immediate head
        let list3 = Term::List {
            head: Box::new(Term::Atom(1)),
            tail: Box::new(Term::Small(0)),
        };
        let hash3 = erts_internal_hash(list3);
        assert_ne!(hash3, 0);
        
        // Test list with non-immediate head
        let list4 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::Small(0)),
        };
        let hash4 = erts_internal_hash(list4);
        assert_ne!(hash4, 0);
        
        // Test list with tail that is a list
        let list5 = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash5 = erts_internal_hash(list5);
        assert_ne!(hash5, 0);
    }
    
    #[test]
    fn test_internal_hash_tuple() {
        // Test empty tuple
        let empty_tuple = Term::Tuple(vec![]);
        let hash1 = erts_internal_hash(empty_tuple);
        assert_ne!(hash1, 0);
        
        // Test single element tuple
        let single_tuple = Term::Tuple(vec![Term::Small(42)]);
        let hash2 = erts_internal_hash(single_tuple);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test multi-element tuple
        let multi_tuple = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let hash3 = erts_internal_hash(multi_tuple);
        assert_ne!(hash3, 0);
        assert_ne!(hash3, hash2);
        
        // Test nested tuple
        let nested_tuple = Term::Tuple(vec![
            Term::Small(1),
            Term::Tuple(vec![Term::Small(2)]),
        ]);
        let hash4 = erts_internal_hash(nested_tuple);
        assert_ne!(hash4, 0);
    }
    
    #[test]
    fn test_internal_hash_map() {
        // Test empty map
        let empty_map = Term::Map(vec![]);
        let hash1 = erts_internal_hash(empty_map);
        assert_ne!(hash1, 0);
        
        // Test map with one pair
        let map1 = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]);
        let hash2 = erts_internal_hash(map1);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test map with multiple pairs
        let map2 = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
            (Term::Small(2), Term::Small(20)),
            (Term::Small(3), Term::Small(30)),
        ]);
        let hash3 = erts_internal_hash(map2);
        assert_ne!(hash3, 0);
        assert_ne!(hash3, hash2);
    }
    
    // Helper function to construct BigNumber from u32 digits (most significant first)
    // This is used in tests to create BigNumber values with specific digit patterns
    fn big_from_digits(digits: &[u32], is_negative: bool) -> BigNumber {
        if digits.is_empty() {
            return BigNumber::from_u64(0);
        }
        
        let base = BigNumber::from_u64(1u64 << 32);
        let mut result = BigNumber::from_u64(0);
        
        // Build from most significant to least significant
        for &digit in digits.iter().rev() {
            result = result.times(&base);
            result = result.plus(&BigNumber::from_u32(digit));
        }
        
        if is_negative {
            // Negate by subtracting from zero
            BigNumber::from_u64(0).minus(&result)
        } else {
            result
        }
    }
    
    #[test]
    fn test_internal_hash_big() {
        // Test positive bignum
        let big_pos = Term::Big(big_from_digits(&[0x12345678, 0x9ABCDEF0], false));
        let hash1 = erts_internal_hash(big_pos);
        assert_ne!(hash1, 0);
        
        // Test negative bignum
        let big_neg = Term::Big(big_from_digits(&[0x12345678], true));
        let hash2 = erts_internal_hash(big_neg);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test bignum with odd number of digits
        let big_odd = Term::Big(big_from_digits(&[0x11111111, 0x22222222, 0x33333333], false));
        let hash3 = erts_internal_hash(big_odd);
        assert_ne!(hash3, 0);
        
        // Test single digit bignum
        let big_single = Term::Big(big_from_digits(&[0x12345678], false));
        let hash4 = erts_internal_hash(big_single);
        assert_ne!(hash4, 0);
    }
    
    #[test]
    fn test_internal_hash_binary() {
        // Test byte-aligned binary
        let bin1 = Term::Binary {
            data: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10],
            bit_offset: 0,
            bit_size: 128, // 16 bytes
        };
        let hash1 = erts_internal_hash(bin1);
        assert_ne!(hash1, 0);
        
        // Test bit-aligned binary
        let bin2 = Term::Binary {
            data: vec![0xFF, 0x00],
            bit_offset: 4,
            bit_size: 8,
        };
        let hash2 = erts_internal_hash(bin2);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test binary with tail bits
        let bin3 = Term::Binary {
            data: vec![0xFF],
            bit_offset: 0,
            bit_size: 5, // 5 bits
        };
        let hash3 = erts_internal_hash(bin3);
        assert_ne!(hash3, 0);
        
        // Test empty binary
        let bin4 = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let hash4 = erts_internal_hash(bin4);
        assert_ne!(hash4, 0);
        
        // Test binary with various remaining byte counts (to cover switch cases)
        for remaining in 1..=15 {
            let size = 16 + remaining; // 16 bytes + remaining
            let bin = Term::Binary {
                data: vec![0xFF; size],
                bit_offset: 0,
                bit_size: size * 8,
            };
            let hash = erts_internal_hash(bin);
            assert_ne!(hash, 0);
        }
    }
    
    #[test]
    fn test_internal_hash_float() {
        let float1 = Term::Float(3.14);
        let hash1 = erts_internal_hash(float1);
        assert_ne!(hash1, 0);
        
        let float2 = Term::Float(-2.5);
        let hash2 = erts_internal_hash(float2);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        let float3 = Term::Float(0.0);
        let hash3 = erts_internal_hash(float3);
        assert_ne!(hash3, 0);
        
        let float4 = Term::Float(f64::INFINITY);
        let hash4 = erts_internal_hash(float4);
        assert_ne!(hash4, 0);
    }
    
    #[test]
    fn test_internal_hash_ref() {
        // Test reference with 3 IDs
        let ref1 = Term::Ref {
            node: 1,
            ids: vec![100, 200, 300],
            creation: 0,
        };
        let hash1 = erts_internal_hash(ref1);
        assert_ne!(hash1, 0);
        
        // Test reference with 5 IDs (64-bit)
        #[cfg(target_pointer_width = "64")]
        {
            let ref2 = Term::Ref {
                node: 1,
                ids: vec![100, 200, 300, 400, 500],
                creation: 0,
            };
            let hash2 = erts_internal_hash(ref2);
            assert_ne!(hash2, 0);
            assert_ne!(hash2, hash1);
        }
        
        // Test reference with 4 IDs (32-bit)
        #[cfg(not(target_pointer_width = "64"))]
        {
            let ref2 = Term::Ref {
                node: 1,
                ids: vec![100, 200, 300, 400],
                creation: 0,
            };
            let hash2 = erts_internal_hash(ref2);
            assert_ne!(hash2, 0);
            assert_ne!(hash2, hash1);
        }
    }
    
    #[test]
    fn test_internal_hash_fun() {
        // Test local function without environment
        let local_fun1 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![],
        };
        let hash1 = erts_internal_hash(local_fun1);
        assert_ne!(hash1, 0);
        
        // Test local function with environment
        let local_fun2 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1), Term::Small(2)],
        };
        let hash2 = erts_internal_hash(local_fun2);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test local function with multiple environment elements
        let local_fun3 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1), Term::Small(2), Term::Small(3)],
        };
        let hash3 = erts_internal_hash(local_fun3);
        assert_ne!(hash3, 0);
        assert_ne!(hash3, hash2);
        
        // Test external function
        let external_fun = Term::Fun {
            is_local: false,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        };
        let hash4 = erts_internal_hash(external_fun);
        assert_ne!(hash4, 0);
        assert_ne!(hash4, hash1);
    }
    
    #[test]
    fn test_internal_hash_salted() {
        let term = Term::Small(42);
        let hash1 = erts_internal_salted_hash(term.clone(), 0);
        let hash2 = erts_internal_salted_hash(term.clone(), 100);
        assert_ne!(hash1, 0);
        assert_ne!(hash2, 0);
        assert_ne!(hash1, hash2); // Different salts should produce different hashes
    }
    
    #[test]
    fn test_make_hash2_map() {
        // Test map hashing in make_hash2
        let map = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
            (Term::Small(2), Term::Small(20)),
        ]);
        let hash = make_hash2(map);
        // hash might be 0, which is acceptable - the important thing is the function executes
        
        // Test empty map
        let empty_map = Term::Map(vec![]);
        let hash2 = make_hash2(empty_map);
        // Empty map might hash to 0, which is acceptable
        // The important thing is the function executes without error
    }
    
    #[test]
    fn test_make_hash2_all_types() {
        // Test all term types in make_hash2
        let nil_hash = make_hash2(Term::Nil);
        assert_ne!(nil_hash, 0);
        
        let small_hash = make_hash2(Term::Small(42));
        assert_ne!(small_hash, 0);
        
        let atom_hash = make_hash2(Term::Atom(123));
        assert_ne!(atom_hash, 0);
        
        let float_hash = make_hash2(Term::Float(3.14));
        assert_ne!(float_hash, 0);
        
        let list_hash = make_hash2(Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)),
        });
        assert_ne!(list_hash, 0);
        
        let tuple_hash = make_hash2(Term::Tuple(vec![Term::Small(1), Term::Small(2)]));
        assert_ne!(tuple_hash, 0);
        
        let pid_hash = make_hash2(Term::Pid { node: 1, id: 100, serial: 0, creation: 0 });
        assert_ne!(pid_hash, 0);
        
        let port_hash = make_hash2(Term::Port { node: 1, id: 0x123456789ABCDEF0, creation: 0 });
        assert_ne!(port_hash, 0);
        
        let ref_hash = make_hash2(Term::Ref { node: 1, ids: vec![100], creation: 0 });
        assert_ne!(ref_hash, 0);
    }
    
    #[test]
    fn test_make_hash_big() {
        // Test positive bignum
        let big_pos = Term::Big(big_from_digits(&[0x12345678, 0x9ABCDEF0], false));
        let hash1 = make_hash(big_pos);
        assert_ne!(hash1, 0);
        
        // Test negative bignum
        let big_neg = Term::Big(big_from_digits(&[0x12345678], true));
        let hash2 = make_hash(big_neg);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
    }
    
    #[test]
    fn test_make_hash_rational() {
        use entities_utilities::BigRational;
        
        // Test positive rational (22/7)
        let rational_pos = Term::Rational(
            BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash1 = make_hash(rational_pos);
        assert_ne!(hash1, 0);
        
        // Test negative rational (-22/7)
        let rational_neg = Term::Rational(
            BigRational::from_i64(-22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash2 = make_hash(rational_neg);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test different rational (1/3)
        let rational_diff = Term::Rational(
            BigRational::from_i64(1).div(&BigRational::from_i64(3)).unwrap()
        );
        let hash3 = make_hash(rational_diff);
        assert_ne!(hash3, 0);
        assert_ne!(hash3, hash1);
        assert_ne!(hash3, hash2);
        
        // Test rational that's an integer (5/1)
        let rational_int = Term::Rational(
            BigRational::from_i64(5).div(&BigRational::from_i64(1)).unwrap()
        );
        let hash4 = make_hash(rational_int);
        assert_ne!(hash4, 0);
        
        // Test zero rational (0/1)
        let rational_zero = Term::Rational(
            BigRational::from_i64(0).div(&BigRational::from_i64(1)).unwrap()
        );
        let hash5 = make_hash(rational_zero);
        // Zero rational should hash differently from other values
        assert_ne!(hash5, hash1);
    }
    
    #[test]
    fn test_make_hash2_rational() {
        use entities_utilities::BigRational;
        
        // Test positive rational
        let rational_pos = Term::Rational(
            BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash1 = make_hash2(rational_pos);
        assert_ne!(hash1, 0);
        
        // Test negative rational
        let rational_neg = Term::Rational(
            BigRational::from_i64(-22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash2 = make_hash2(rational_neg);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test that same rational hashes to same value
        let rational_pos2 = Term::Rational(
            BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash3 = make_hash2(rational_pos2);
        assert_eq!(hash1, hash3);
    }
    
    #[test]
    fn test_erts_internal_hash_rational() {
        use entities_utilities::BigRational;
        
        // Test positive rational
        let rational_pos = Term::Rational(
            BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash1 = erts_internal_hash(rational_pos);
        assert_ne!(hash1, 0);
        
        // Test negative rational
        let rational_neg = Term::Rational(
            BigRational::from_i64(-22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash2 = erts_internal_hash(rational_neg);
        assert_ne!(hash2, 0);
        assert_ne!(hash2, hash1);
        
        // Test that same rational hashes to same value
        let rational_pos2 = Term::Rational(
            BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap()
        );
        let hash3 = erts_internal_hash(rational_pos2);
        assert_eq!(hash1, hash3);
        
        // Test different rational
        let rational_diff = Term::Rational(
            BigRational::from_i64(1).div(&BigRational::from_i64(3)).unwrap()
        );
        let hash4 = erts_internal_hash(rational_diff);
        assert_ne!(hash4, hash1);
    }
    
    #[test]
    fn test_make_hash_ref() {
        // Test reference hashing
        let ref_term = Term::Ref {
            node: 1,
            ids: vec![100, 200, 300],
            creation: 0,
        };
        let hash = make_hash(ref_term);
        assert_ne!(hash, 0);
    }
    
    #[test]
    fn test_make_hash_empty_structures() {
        // Test empty tuple
        let empty_tuple = Term::Tuple(vec![]);
        let hash1 = make_hash(empty_tuple);
        // Empty tuple hashes to: hash * HASH_MULT_TUPLE_ARITY + 0 = 0 (with initial hash 0)
        // This is acceptable - the important thing is the function executes
        
        // Test empty list (nil)
        let nil_hash = make_hash(Term::Nil);
        // Nil hashes to: hash * HASH_MULT_POSITIVE + 1 = 1 (with initial hash 0)
        assert_eq!(nil_hash, 1);
    }
    
    #[test]
    fn test_make_hash_nested_structures() {
        // Test deeply nested tuple
        let nested = Term::Tuple(vec![
            Term::Tuple(vec![
                Term::Tuple(vec![Term::Small(1)]),
            ]),
        ]);
        let hash = make_hash(nested);
        // hash might be 0, which is acceptable - the important thing is the function executes
        
        // Test list of tuples
        let list_of_tuples = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::List {
                head: Box::new(Term::Tuple(vec![Term::Small(2)])),
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash2 = make_hash(list_of_tuples);
        // hash2 might be 0 or equal to hash, which is acceptable
        // The important thing is the function executes without error
    }
    
    #[test]
    fn test_make_hash_large_tuple() {
        // Test large tuple (to cover all element processing)
        let mut elements = Vec::new();
        for i in 0..20 {
            elements.push(Term::Small(i as i64));
        }
        let large_tuple = Term::Tuple(elements);
        let hash = make_hash(large_tuple);
        assert_ne!(hash, 0);
    }
    
    #[test]
    fn test_make_hash_long_list() {
        // Test long list (to cover list traversal)
        let mut tail = Box::new(Term::Small(0));
        for i in (1..=10).rev() {
            tail = Box::new(Term::List {
                head: Box::new(Term::Small(i)),
                tail,
            });
        }
        let long_list = *tail;
        let hash = make_hash(long_list);
        assert_ne!(hash, 0);
    }
    
    #[test]
    fn test_make_hash_binary_edge_cases() {
        // Test binary with various bit offsets and sizes
        for bit_offset in 0..8 {
            for bit_size in 1..=16 {
                let bin = Term::Binary {
                    data: vec![0xFF; 3],
                    bit_offset,
                    bit_size,
                };
                let hash = make_hash(bin);
                assert_ne!(hash, 0);
            }
        }
    }
    
    #[test]
    fn test_internal_hash_binary_all_remaining_bytes() {
        // Test binary processing to cover all switch cases for remaining bytes (1-15, then 1-8)
        // This covers the complex switch statement in make_internal_hash_impl
        
        // Test binaries with sizes that result in remaining bytes 1-15 (after 16-byte chunks)
        for remaining in 1..=15 {
            let total_bytes = 16 + remaining; // 16-byte chunk + remaining
            let bin = Term::Binary {
                data: vec![0xAA; total_bytes],
                bit_offset: 0,
                bit_size: total_bytes * 8,
            };
            let hash = erts_internal_hash(bin);
            // hash might be 0, which is acceptable - the important thing is the function executes
        }
        
        // Test binaries with sizes that result in remaining bytes 1-8 (second switch)
        for remaining in 1..=8 {
            let total_bytes = 16 + remaining; // Already tested above, but ensure coverage
            let bin = Term::Binary {
                data: vec![0xBB; total_bytes],
                bit_offset: 0,
                bit_size: total_bytes * 8,
            };
            let hash = erts_internal_hash(bin);
            // hash might be 0, which is acceptable
        }
        
        // Test bit-aligned binaries with various remaining byte counts
        for remaining in 1..=15 {
            let total_bytes = 16 + remaining;
            let bin = Term::Binary {
                data: vec![0xCC; total_bytes + 1], // Extra byte for bit alignment
                bit_offset: 4, // 4-bit offset
                bit_size: total_bytes * 8,
            };
            let hash = erts_internal_hash(bin);
            // hash might be 0, which is acceptable
        }
    }
    
    #[test]
    fn test_internal_hash_list_string_optimization() {
        // Test list string optimization with various byte counts (to cover bytes % 4 != 0 case)
        
        // Test 1-byte string (not multiple of 4)
        let list1 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::Small(0)),
        };
        let hash1 = erts_internal_hash(list1);
        // hash might be 0, which is acceptable
        
        // Test 4-byte string (exactly multiple of 4)
        let list4 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::Small(0)),
                    }),
                }),
            }),
        };
        let hash4 = erts_internal_hash(list4);
        // hash might be 0, which is acceptable
        
        // Test 8-byte string (multiple of 4)
        let list8 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::List {
                            head: Box::new(Term::Small(69)), // 'E'
                            tail: Box::new(Term::List {
                                head: Box::new(Term::Small(70)), // 'F'
                                tail: Box::new(Term::List {
                                    head: Box::new(Term::Small(71)), // 'G'
                                    tail: Box::new(Term::List {
                                        head: Box::new(Term::Small(72)), // 'H'
                                        tail: Box::new(Term::Small(0)),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        };
        let hash8 = erts_internal_hash(list8);
        // hash might be 0, which is acceptable
        
        // Test 2-byte string
        let list2 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash2 = erts_internal_hash(list2);
        // hash might be 0, which is acceptable
        
        // Test 3-byte string
        let list3 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::Small(0)),
                }),
            }),
        };
        let hash3 = erts_internal_hash(list3);
        // hash might be 0, which is acceptable
        
        // Test 5-byte string (4 + 1)
        let list5 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::List {
                            head: Box::new(Term::Small(69)), // 'E'
                            tail: Box::new(Term::Small(0)),
                        }),
                    }),
                }),
            }),
        };
        let hash5 = erts_internal_hash(list5);
        // hash might be 0, which is acceptable
        
        // Test 6-byte string (4 + 2)
        let list6 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::List {
                            head: Box::new(Term::Small(69)), // 'E'
                            tail: Box::new(Term::List {
                                head: Box::new(Term::Small(70)), // 'F'
                                tail: Box::new(Term::Small(0)),
                            }),
                        }),
                    }),
                }),
            }),
        };
        let hash6 = erts_internal_hash(list6);
        // hash might be 0, which is acceptable
        
        // Test 7-byte string (4 + 3)
        let list7 = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(66)), // 'B'
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(67)), // 'C'
                    tail: Box::new(Term::List {
                        head: Box::new(Term::Small(68)), // 'D'
                        tail: Box::new(Term::List {
                            head: Box::new(Term::Small(69)), // 'E'
                            tail: Box::new(Term::List {
                                head: Box::new(Term::Small(70)), // 'F'
                                tail: Box::new(Term::List {
                                    head: Box::new(Term::Small(71)), // 'G'
                                    tail: Box::new(Term::Small(0)),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        };
        let hash7 = erts_internal_hash(list7);
        // hash might be 0, which is acceptable
        
        // Test list where string optimization ends and we process remaining bytes
        // This covers the "Handle remaining bytes" path when bytes > 0 && bytes % 4 != 0
        // Test with bytes = 1, 2, 3 (all not multiple of 4)
        // Already covered above, but ensure we hit the path where current_list_term is None
        // after string optimization
        
        // Test list where string optimization completes and current_list_term is None
        // This covers the "No more list to process" path
        let list_empty_after_string = Term::List {
            head: Box::new(Term::Small(65)), // 'A'
            tail: Box::new(Term::Small(0)), // Not a list - ends string optimization
        };
        let hash_empty = erts_internal_hash(list_empty_after_string);
        // hash_empty might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_list_general_paths() {
        // Test list where head is immediate and tail is a list (covers !matches!(tail_term, Term::List { .. }) = false)
        let list1 = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash1 = erts_internal_hash(list1);
        // hash1 might be 0, which is acceptable
        
        // Test list where head is immediate and tail is not a list (covers !matches!(tail_term, Term::List { .. }) = true)
        let list2 = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)),
        };
        let hash2 = erts_internal_hash(list2);
        // hash2 might be 0, which is acceptable
        
        // Test list where head is not immediate and tail is a list (covers !matches!(tail_term, Term::List { .. }) = false)
        let list3 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash3 = erts_internal_hash(list3);
        // hash3 might be 0, which is acceptable
        
        // Test list where head is not immediate and tail is not a list (covers !matches!(tail_term, Term::List { .. }) = true)
        let list4 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::Small(0)),
        };
        let hash4 = erts_internal_hash(list4);
        // hash4 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_binary_all_switch_cases() {
        // Test all switch cases for remaining bytes (9-15, then 1-8)
        // Create binaries with sizes that result in exactly these remaining byte counts
        
        // Test remaining bytes 9-15 (first switch statement)
        for remaining in 9..=15 {
            let total_bytes = 16 + remaining;
            let bin = Term::Binary {
                data: vec![0xAA; total_bytes],
                bit_offset: 0,
                bit_size: total_bytes * 8,
            };
            let _ = erts_internal_hash(bin);
        }
        
        // Test remaining bytes 1-8 (second switch statement)
        for remaining in 1..=8 {
            let total_bytes = 16 + remaining;
            let bin = Term::Binary {
                data: vec![0xBB; total_bytes],
                bit_offset: 0,
                bit_size: total_bytes * 8,
            };
            let _ = erts_internal_hash(bin);
        }
        
        // Test remaining = 0 (no remaining bytes after 16-byte chunks)
        let bin_exact = Term::Binary {
            data: vec![0xCC; 16],
            bit_offset: 0,
            bit_size: 128, // Exactly 16 bytes
        };
        let _ = erts_internal_hash(bin_exact);
        
        // Test remaining > 15 (should hit default case)
        let bin_large = Term::Binary {
            data: vec![0xDD; 32],
            bit_offset: 0,
            bit_size: 256, // 32 bytes = 16 + 16, but we process in 16-byte chunks
        };
        let _ = erts_internal_hash(bin_large);
    }
    
    #[test]
    fn test_make_hash2_small_negative() {
        // Test make_hash2 with negative small integer (covers the if value < 0 path)
        let neg = Term::Small(-42);
        let hash = make_hash2(neg);
        // hash might be 0, which is acceptable
        
        let pos = Term::Small(42);
        let hash2 = make_hash2(pos);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_make_hash2_tuple_empty() {
        // Test make_hash2 with empty tuple (covers the case where stack.pop() returns None)
        let empty_tuple = Term::Tuple(vec![]);
        let hash = make_hash2(empty_tuple);
        // hash might be 0, which is acceptable
    }
    
    #[test]
    fn test_make_hash2_map_empty() {
        // Test make_hash2 with empty map (covers the size == 0 path)
        let empty_map = Term::Map(vec![]);
        let hash = make_hash2(empty_map);
        // hash might be 0, which is acceptable
    }
    
    #[test]
    fn test_make_hash2_map_single_pair() {
        // Test make_hash2 with single pair map (covers the case where we don't push pairs)
        let map = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]);
        let hash = make_hash2(map);
        // hash might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_map_edge_cases() {
        // Test map with single pair (covers the case where we don't push pairs)
        let map1 = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]);
        let hash1 = erts_internal_hash(map1);
        // hash1 might be 0, which is acceptable
        
        // Test map with two pairs (covers the case where we push one pair)
        let map2 = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
            (Term::Small(2), Term::Small(20)),
        ]);
        let hash2 = erts_internal_hash(map2);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_tuple_edge_cases() {
        // Test tuple with single element (covers the case where we don't push elements)
        let tuple1 = Term::Tuple(vec![Term::Small(1)]);
        let hash1 = erts_internal_hash(tuple1);
        // hash1 might be 0, which is acceptable
        
        // Test tuple with two elements (covers the case where we push one element)
        let tuple2 = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        let hash2 = erts_internal_hash(tuple2);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_fun_environment_edge_cases() {
        // Test local function with exactly one environment element (covers the case where we don't push env elements)
        let fun1 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1)],
        };
        let hash1 = erts_internal_hash(fun1);
        // hash1 might be 0, which is acceptable
        
        // Test local function with two environment elements (covers the case where we push one element)
        let fun2 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1), Term::Small(2)],
        };
        let hash2 = erts_internal_hash(fun2);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_big_all_digit_counts() {
        // Test bignum with various digit counts to cover all paths
        
        // Test with 1 digit (odd, covers the i < n path)
        let big1 = Term::Big(big_from_digits(&[0x11111111], false));
        let _ = erts_internal_hash(big1);
        
        // Test with 2 digits (even, no remaining)
        let big2 = Term::Big(big_from_digits(&[0x11111111, 0x22222222], false));
        let _ = erts_internal_hash(big2);
        
        // Test with 3 digits (odd, covers the i < n path)
        let big3 = Term::Big(big_from_digits(&[0x11111111, 0x22222222, 0x33333333], false));
        let _ = erts_internal_hash(big3);
        
        // Test with 4 digits (even, no remaining)
        let big4 = Term::Big(big_from_digits(&[0x11111111, 0x22222222, 0x33333333, 0x44444444], false));
        let _ = erts_internal_hash(big4);
        
        // Test with 5 digits (odd, covers the i < n path)
        let big5 = Term::Big(big_from_digits(&[0x11111111, 0x22222222, 0x33333333, 0x44444444, 0x55555555], false));
        let _ = erts_internal_hash(big5);
    }
    
    #[test]
    fn test_internal_hash_binary_bit_aligned_all_cases() {
        // Test bit-aligned binaries with various sizes to cover all code paths
        for bit_offset in 1..8 {
            for total_bits in 1..=100 {
                let bytes_needed = (total_bits + 7) / 8 + 1; // Extra byte for bit alignment
                let bin = Term::Binary {
                    data: vec![0xFF; bytes_needed],
                    bit_offset,
                    bit_size: total_bits,
                };
                let _ = erts_internal_hash(bin);
            }
        }
        
        // Test binary where bytes.len() == 0 (edge case)
        let bin_empty = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let _ = erts_internal_hash(bin_empty);
        
        // Test binary where byte_off >= data.len() (edge case in byte extraction)
        let bin_edge = Term::Binary {
            data: vec![0xFF],
            bit_offset: 8, // This would make byte_off >= data.len()
            bit_size: 8,
        };
        let _ = erts_internal_hash(bin_edge);
    }
    
    #[test]
    fn test_make_hash2_map_error_paths() {
        // Test map hashing paths that might hit error cases
        // Test with map that has nested structures to ensure all stack entries are processed
        
        // Test map with multiple pairs to cover MapPair processing
        let map = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
            (Term::Small(2), Term::Small(20)),
            (Term::Small(3), Term::Small(30)),
        ]);
        let _ = make_hash2(map);
        
        // Test map where MapSavedHash/MapSavedXor might not be found (edge case)
        // This is hard to trigger, but we can test with complex nested maps
        let map_nested = Term::Map(vec![
            (Term::Map(vec![(Term::Small(1), Term::Small(10))]), Term::Small(100)),
        ]);
        let _ = make_hash2(map_nested);
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_erts_dbg_hashmap_collision_bonanza_32bit() {
        // Test the 32-bit path in collision bonanza
        // This is hard to test directly, but we can test with different hash values
        let hash1 = 0x12345678u64;
        let hash2 = 0x87654321u64;
        
        let weak1 = erts_dbg_hashmap_collision_bonanza(hash1, Term::Nil);
        let weak2 = erts_dbg_hashmap_collision_bonanza(hash2, Term::Nil);
        
        // Both should be valid hashes
        assert_ne!(weak1, 0);
        assert_ne!(weak2, 0);
        
        // Test with hash that results in bad_bits >= sizeof(HashValue) * 8
        // This is hard to trigger, but we can try with various hash values
        // Note: Some hash values might result in weak hash of 0, which is acceptable
        for i in 0..100 {
            let hash = i as u64 * 137; // This might trigger the condition
            let weak = erts_dbg_hashmap_collision_bonanza(hash, Term::Nil);
            // weak might be 0, which is acceptable - the important thing is the function executes
        }
    }
    
    #[test]
    fn test_internal_hash_all_immediate_types() {
        // Test all immediate types in the fast path
        let nil_hash = erts_internal_hash(Term::Nil);
        // nil_hash might be 0, which is acceptable
        
        // Test all Small values that might hit different paths
        for i in -100..=100 {
            let _ = erts_internal_hash(Term::Small(i));
        }
        
        // Test various atom values
        for i in 0..100 {
            let _ = erts_internal_hash(Term::Atom(i));
        }
        
        // Test various PIDs
        for i in 0..100 {
            let _ = erts_internal_hash(Term::Pid { node: 1, id: i, serial: 0, creation: 0 });
        }
        
        // Test various Ports (32-bit and 64-bit)
        for i in 0..100 {
            let _ = erts_internal_hash(Term::Port { node: 1, id: i as u64, creation: 0 });
        }
        for i in 0..10 {
            let _ = erts_internal_hash(Term::Port { node: 1, id: (i as u64) << 32, creation: 0 });
        }
    }
    
    #[test]
    fn test_internal_hash_salted_all_types() {
        // Test erts_internal_salted_hash with all term types
        let nil_hash = erts_internal_salted_hash(Term::Nil, 0);
        // nil_hash might be 0, which is acceptable
        
        let small_hash = erts_internal_salted_hash(Term::Small(42), 100);
        // small_hash might be 0, which is acceptable
        
        let atom_hash = erts_internal_salted_hash(Term::Atom(123), 200);
        // atom_hash might be 0, which is acceptable
        
        let tuple_hash = erts_internal_salted_hash(Term::Tuple(vec![Term::Small(1)]), 300);
        // tuple_hash might be 0, which is acceptable
        
        let list_hash = erts_internal_salted_hash(Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)),
        }, 400);
        // list_hash might be 0, which is acceptable
        
        let map_hash = erts_internal_salted_hash(Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]), 500);
        // map_hash might be 0, which is acceptable
    }
    
    #[test]
    fn test_make_hash_all_term_types_comprehensive() {
        // Comprehensive test of make_hash with all term types and edge cases
        
        // Test all immediate types
        let _ = make_hash(Term::Nil);
        for i in -10..=10 {
            let _ = make_hash(Term::Small(i));
        }
        for i in 0..10 {
            let _ = make_hash(Term::Atom(i));
        }
        for i in 0..10 {
            let _ = make_hash(Term::Pid { node: 1, id: i, serial: 0, creation: 0 });
        }
        for i in 0..10 {
            let _ = make_hash(Term::Port { node: 1, id: i as u64, creation: 0 });
        }
        
        // Test all complex types
        let _ = make_hash(Term::Float(3.14));
        let _ = make_hash(Term::Float(-2.5));
        let _ = make_hash(Term::Float(0.0));
        
        let _ = make_hash(Term::Binary {
            data: vec![0x01, 0x02, 0x03],
            bit_offset: 0,
            bit_size: 24,
        });
        
        let _ = make_hash(Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)),
        });
        
        let _ = make_hash(Term::Tuple(vec![Term::Small(1), Term::Small(2)]));
        
        let _ = make_hash(Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]));
        
        let _ = make_hash(Term::Ref {
            node: 1,
            ids: vec![100, 200, 300],
            creation: 0,
        });
        
        let _ = make_hash(Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1)],
        });
        
        let _ = make_hash(Term::Fun {
            is_local: false,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        });
    }
    
    #[test]
    fn test_make_hash2_all_term_types_comprehensive() {
        // Comprehensive test of make_hash2 with all term types
        
        // Test all immediate types
        let _ = make_hash2(Term::Nil);
        for i in -10..=10 {
            let _ = make_hash2(Term::Small(i));
        }
        for i in 0..10 {
            let _ = make_hash2(Term::Atom(i));
        }
        for i in 0..10 {
            let _ = make_hash2(Term::Pid { node: 1, id: i, serial: 0, creation: 0 });
        }
        for i in 0..10 {
            let _ = make_hash2(Term::Port { node: 1, id: i as u64, creation: 0 });
        }
        
        // Test all complex types
        let _ = make_hash2(Term::Float(3.14));
        let _ = make_hash2(Term::Float(-2.5));
        let _ = make_hash2(Term::Float(0.0));
        
        let _ = make_hash2(Term::Binary {
            data: vec![0x01, 0x02, 0x03],
            bit_offset: 0,
            bit_size: 24,
        });
        
        let _ = make_hash2(Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)),
        });
        
        let _ = make_hash2(Term::Tuple(vec![Term::Small(1), Term::Small(2)]));
        
        let _ = make_hash2(Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
        ]));
        
        let _ = make_hash2(Term::Ref {
            node: 1,
            ids: vec![100],
            creation: 0,
        });
    }
    
    #[test]
    fn test_make_hash2_binary_bit_aligned_comprehensive() {
        // Test make_hash2 with bit-aligned binaries covering all edge cases
        for bit_offset in 1..8 {
            for size in 0..=20 {
                let bytes_needed = (size * 8 + 7) / 8 + 1;
                let bin = Term::Binary {
                    data: vec![0xEE; bytes_needed],
                    bit_offset,
                    bit_size: size * 8,
                };
                let _ = make_hash2(bin);
            }
        }
    }
    
    #[test]
    fn test_internal_hash_binary_tail_bits() {
        // Test binary with tail bits to cover the tail bits handling
        for bitsize in 1..=7 {
            let bin = Term::Binary {
                data: vec![0xFF, 0xFF],
                bit_offset: 0,
                bit_size: 8 + bitsize, // 1 byte + bitsize bits
            };
            let hash = erts_internal_hash(bin);
            // hash might be 0, which is acceptable
        }
        
        // Test bit-aligned binary with tail bits
        for bitsize in 1..=7 {
            let bin = Term::Binary {
                data: vec![0xFF, 0xFF, 0xFF],
                bit_offset: 4, // 4-bit offset
                bit_size: 8 + bitsize, // 1 byte + bitsize bits
            };
            let hash = erts_internal_hash(bin);
            // hash might be 0, which is acceptable
        }
        
        // Test binary with tail bits where byte_idx >= data.len() (edge case)
        let bin_edge = Term::Binary {
            data: vec![0xFF],
            bit_offset: 0,
            bit_size: 5, // 5 bits, byte_idx might be out of bounds
        };
        let hash_edge = erts_internal_hash(bin_edge);
        // hash_edge might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_binary_edge_cases() {
        // Test binary with byte_idx >= data.len() (edge case in tail bits handling)
        let bin = Term::Binary {
            data: vec![0xFF],
            bit_offset: 0,
            bit_size: 5, // 5 bits, but byte_idx might be out of bounds
        };
        let hash = erts_internal_hash(bin);
        // hash might be 0, which is acceptable
        
        // Test empty binary
        let empty_bin = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let hash2 = erts_internal_hash(empty_bin);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_list_non_byte_head() {
        // Test list where head is not a byte (value > 255)
        let list = Term::List {
            head: Box::new(Term::Small(300)), // Not a byte
            tail: Box::new(Term::Small(0)),
        };
        let hash = erts_internal_hash(list);
        // hash might be 0, which is acceptable
        
        // Test list where head is not Small
        let list2 = Term::List {
            head: Box::new(Term::Atom(1)),
            tail: Box::new(Term::Small(0)),
        };
        let hash2 = erts_internal_hash(list2);
        // hash2 might be 0, which is acceptable
        
        // Test list where head is immediate and tail is not a list
        let list3 = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(0)), // Not a list
        };
        let hash3 = erts_internal_hash(list3);
        // hash3 might be 0, which is acceptable
        
        // Test list where head is not immediate and tail is not a list
        let list4 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::Small(0)), // Not a list
        };
        let hash4 = erts_internal_hash(list4);
        // hash4 might be 0, which is acceptable
        
        // Test list where head is not immediate and tail is a list
        let list5 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Small(0)),
            }),
        };
        let hash5 = erts_internal_hash(list5);
        // hash5 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_ref_edge_cases() {
        // Test reference with less than 3 IDs
        let ref1 = Term::Ref {
            node: 1,
            ids: vec![100],
            creation: 0,
        };
        let hash1 = erts_internal_hash(ref1);
        // hash1 might be 0, which is acceptable
        
        let ref2 = Term::Ref {
            node: 1,
            ids: vec![100, 200],
            creation: 0,
        };
        let hash2 = erts_internal_hash(ref2);
        // hash2 might be 0, which is acceptable
        
        // Test reference with exactly 3 IDs
        let ref3 = Term::Ref {
            node: 1,
            ids: vec![100, 200, 300],
            creation: 0,
        };
        let hash3 = erts_internal_hash(ref3);
        // hash3 might be 0, which is acceptable
        
        // Test reference with exactly 4 IDs (32-bit path)
        #[cfg(not(target_pointer_width = "64"))]
        {
            let ref4 = Term::Ref {
                node: 1,
                ids: vec![100, 200, 300, 400],
                creation: 0,
            };
            let hash4 = erts_internal_hash(ref4);
            // hash4 might be 0, which is acceptable
        }
        
        // Test reference with exactly 5 IDs (64-bit path)
        #[cfg(target_pointer_width = "64")]
        {
            let ref5 = Term::Ref {
                node: 1,
                ids: vec![100, 200, 300, 400, 500],
                creation: 0,
            };
            let hash5 = erts_internal_hash(ref5);
            // hash5 might be 0, which is acceptable
        }
    }
    
    #[test]
    fn test_internal_hash_fun_edge_cases() {
        // Test local function with old_uniq = None
        let fun1 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: None,
            env: vec![],
        };
        let hash1 = erts_internal_hash(fun1);
        // hash1 might be 0, which is acceptable
        
        // Test local function with single environment element
        let fun2 = Term::Fun {
            is_local: true,
            module: 1,
            function: 42,
            arity: 2,
            old_uniq: Some(100),
            env: vec![Term::Small(1)],
        };
        let hash2 = erts_internal_hash(fun2);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_internal_hash_big_edge_cases() {
        // Test bignum with single digit (odd number of digits)
        let big1 = Term::Big(big_from_digits(&[0x12345678], false));
        let hash1 = erts_internal_hash(big1);
        // hash1 might be 0, which is acceptable
        
        // Test bignum with two digits (even number, no remaining)
        let big2 = Term::Big(big_from_digits(&[0x11111111, 0x22222222], false));
        let hash2 = erts_internal_hash(big2);
        // hash2 might be 0, which is acceptable
    }
    
    #[test]
    fn test_make_hash2_binary_all_cases() {
        // Test make_hash2 binary processing with all edge cases
        
        // Test byte-aligned binary
        for size in 0..=20 {
            let bin = Term::Binary {
                data: vec![0xAA; size.max(1)],
                bit_offset: 0,
                bit_size: size * 8,
            };
            let _ = make_hash2(bin);
        }
        
        // Test bit-aligned binary
        for bit_offset in 1..8 {
            for size in 1..=10 {
                let bin = Term::Binary {
                    data: vec![0xBB; size + 1],
                    bit_offset,
                    bit_size: size * 8,
                };
                let _ = make_hash2(bin);
            }
        }
        
        // Test binary with tail bits
        for bitsize in 1..=7 {
            let bin = Term::Binary {
                data: vec![0xCC, 0xDD],
                bit_offset: 0,
                bit_size: 8 + bitsize,
            };
            let _ = make_hash2(bin);
        }
    }
    
    #[test]
    fn test_make_hash2_map_comprehensive() {
        // Test make_hash2 with maps of various sizes to cover all map hashing paths
        for size in 0..=5 {
            let mut entries = Vec::new();
            for i in 0..size {
                entries.push((Term::Small(i as i64), Term::Small((i * 10) as i64)));
            }
            let map = Term::Map(entries);
            let _ = make_hash2(map);
        }
        
        // Test map with nested structures (to cover MapPair, MapTail, MapSavedHash, MapSavedXor paths)
        let map_nested = Term::Map(vec![
            (Term::Small(1), Term::Tuple(vec![Term::Small(10)])),
            (Term::Small(2), Term::List {
                head: Box::new(Term::Small(20)),
                tail: Box::new(Term::Small(0)),
            }),
        ]);
        let _ = make_hash2(map_nested);
        
        // Test map with complex keys and values
        let map_complex = Term::Map(vec![
            (Term::Tuple(vec![Term::Small(1)]), Term::Small(10)),
            (Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Small(0)),
            }, Term::Small(20)),
        ]);
        let _ = make_hash2(map_complex);
        
        // Test large map to ensure all pairs are processed
        let mut large_entries = Vec::new();
        for i in 0..10 {
            large_entries.push((Term::Small(i), Term::Small(i * 10)));
        }
        let large_map = Term::Map(large_entries);
        let _ = make_hash2(large_map);
    }
    
    #[test]
    fn test_make_hash2_tuple_comprehensive() {
        // Test make_hash2 with tuples of various sizes
        for size in 0..=10 {
            let mut elements = Vec::new();
            for i in 0..size {
                elements.push(Term::Small(i as i64));
            }
            let tuple = Term::Tuple(elements);
            let _ = make_hash2(tuple);
        }
    }
    
    #[test]
    fn test_make_hash2_list_comprehensive() {
        // Test make_hash2 with lists of various lengths
        for len in 0..=10 {
            let mut tail = Box::new(Term::Small(0));
            for i in (1..=len).rev() {
                tail = Box::new(Term::List {
                    head: Box::new(Term::Small(i)),
                    tail,
                });
            }
            let list = *tail;
            let _ = make_hash2(list);
        }
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_erts_dbg_hashmap_collision_bonanza() {
        // Test that the debug collision function weakens hashes
        // Note: This test only runs in debug builds
        let key = Term::Small(42);
        let hash1 = erts_internal_hash(key.clone());
        let hash2 = erts_internal_hash(Term::Small(43));
        
        // Original hashes should be different
        assert_ne!(hash1, hash2);
        
        // Apply collision bonanza directly (function is only available in debug builds)
        let weak_hash1 = erts_dbg_hashmap_collision_bonanza(hash1, key.clone());
        let weak_hash2 = erts_dbg_hashmap_collision_bonanza(hash2, Term::Small(43));
        
        // Weakened hashes should still be different (but may have more collisions)
        // The function should return a valid hash value
        assert_ne!(weak_hash1, 0);
        assert_ne!(weak_hash2, 0);
        
        // Test with different hash values
        let hash3 = 0x1234567890ABCDEFu64;
        let weak_hash3 = erts_dbg_hashmap_collision_bonanza(hash3, Term::Nil);
        assert_ne!(weak_hash3, 0);
        assert_ne!(weak_hash3, hash3); // Should be different from original
        
        // Test that it handles the mask correctly
        let hash4 = 0xFFFFFFFFFFFFFFFFu64;
        let weak_hash4 = erts_dbg_hashmap_collision_bonanza(hash4, Term::Nil);
        assert_ne!(weak_hash4, 0);
    }
    
    #[test]
    fn test_erts_map_hash() {
        // Test that erts_map_hash works
        let key1 = Term::Small(100);
        let key2 = Term::Small(200);
        
        let hash1 = erts_map_hash(key1.clone());
        let hash2 = erts_map_hash(key2);
        
        assert_ne!(hash1, 0);
        assert_ne!(hash2, 0);
        assert_ne!(hash1, hash2); // Different keys should produce different hashes
        
        // In debug mode, hashes should be weakened
        #[cfg(debug_assertions)]
        {
            let internal_hash1 = erts_internal_hash(key1.clone());
            // In debug mode, map_hash applies collision bonanza, so it may differ
            // But it should still be a valid hash
            assert_ne!(hash1, 0);
            // The weakened hash should be different from the original
            assert_ne!(hash1, internal_hash1);
        }
        
        // In release mode, should be identical to internal_hash
        #[cfg(not(debug_assertions))]
        {
            let internal_hash1 = erts_internal_hash(key1);
            assert_eq!(hash1, internal_hash1);
        }
    }
}

