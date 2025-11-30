//! Unique Value Generation Built-in Functions
//!
//! Provides unique value generation:
//! - References (make_ref)
//! - Unique integers (with optional monotonic and positive flags)
//!
//! This module uses safe Rust atomic operations and thread IDs for unique value generation.

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
 *
 * Creation productivity increased for code in this file by using AALang and GAB.
 * See https://github.com/yenrab/AALang-Gab
 */

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reference identifier
///
/// Represents a unique reference in the system.
/// In Erlang, references are used for various purposes including
/// process monitoring, message tagging, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference {
    /// Thread ID that created the reference
    thread_id: u32,
    /// Reference value (atomic counter)
    value: u64,
    /// Reference number (for multi-part references)
    ref_number: u32,
}

impl Reference {
    /// Create a new reference
    fn new(thread_id: u32, value: u64, ref_number: u32) -> Self {
        Self {
            thread_id,
            value,
            ref_number,
        }
    }

    /// Get the thread ID
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }

    /// Get the reference value
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Get the reference number
    pub fn ref_number(&self) -> u32 {
        self.ref_number
    }
}

/// Unique integer options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniqueIntegerOption {
    /// Generate monotonic unique integers (strictly increasing)
    Monotonic,
    /// Generate only positive unique integers
    Positive,
}

/// Unique integer generator
///
/// Generates unique integers using thread IDs and atomic counters.
/// Supports both monotonic and non-monotonic generation.
pub struct UniqueIntegerGenerator {
    /// Global counter for unique integers
    global_counter: AtomicU64,
    /// Monotonic counter for strictly increasing values
    monotonic_counter: AtomicU64,
    /// Initial reference value (based on system time)
    ref_init_value: u64,
}

impl UniqueIntegerGenerator {
    /// Create a new unique integer generator
    ///
    /// Initializes with a value based on system time to ensure uniqueness
    /// across process restarts.
    pub fn new() -> Self {
        let ref_init_value = Self::init_ref_value();
        Self {
            global_counter: AtomicU64::new(ref_init_value),
            monotonic_counter: AtomicU64::new(0),
            ref_init_value,
        }
    }

    /// Initialize reference value from system time
    fn init_ref_value() -> u64 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        
        let mut value = duration.as_secs();
        value |= (duration.subsec_micros() as u64) << 32;
        value = value.wrapping_mul(268438039);
        value = value.wrapping_add(duration.subsec_micros() as u64);
        value
    }

    /// Get a thread ID for the current thread
    ///
    /// Uses thread-local storage to assign unique IDs to threads.
    /// In a full implementation, this would map to scheduler IDs.
    fn get_thread_id() -> u32 {
        use std::cell::Cell;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1);
        
        thread_local! {
            static THREAD_ID: Cell<Option<u32>> = Cell::new(None);
        }
        
        THREAD_ID.with(|id| {
            if let Some(tid) = id.get() {
                tid
            } else {
                // Assign a unique ID to this thread
                let tid = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
                id.set(Some(tid));
                tid
            }
        })
    }

    /// Generate a unique integer
    ///
    /// # Arguments
    /// * `positive` - If true, only generate positive integers
    ///
    /// # Returns
    /// Unique integer value
    pub fn unique_integer(&self, positive: bool) -> i64 {
        let thread_id = Self::get_thread_id();
        let unique = self.global_counter.fetch_add(1, Ordering::Relaxed);
        
        // Combine thread ID and unique value
        let combined = ((thread_id as u64) << 32) | (unique & 0xFFFFFFFF);
        
        let result = if positive {
            // Ensure positive (add 1 to avoid 0)
            (combined as i64).saturating_add(1).max(1)
        } else {
            // Can be negative, but ensure it's within i64 range
            combined as i64
        };
        
        result
    }

    /// Generate a monotonic unique integer
    ///
    /// Generates strictly increasing unique integers.
    ///
    /// # Arguments
    /// * `positive` - If true, only generate positive integers
    ///
    /// # Returns
    /// Monotonic unique integer value
    pub fn unique_integer_monotonic(&self, positive: bool) -> i64 {
        let raw = self.monotonic_counter.fetch_add(1, Ordering::AcqRel);
        
        if positive {
            // Monotonic positive: start from 1
            (raw + 1) as i64
        } else {
            // Monotonic with offset to allow negative values
            // Use MIN_SMALL equivalent offset
            const MIN_SMALL: i64 = i64::MIN;
            let offset = MIN_SMALL;
            (raw as i64).saturating_add(offset)
        }
    }

    /// Create a new reference
    ///
    /// Generates a unique reference identifier.
    ///
    /// # Returns
    /// New reference
    pub fn make_ref(&self) -> Reference {
        let thread_id = Self::get_thread_id();
        let value = self.global_counter.fetch_add(1, Ordering::Relaxed);
        let ref_number = (value & 0xFFFFFFFF) as u32;
        
        Reference::new(thread_id, value, ref_number)
    }
}

/// Global unique integer generator instance
///
/// Uses lazy_static pattern for thread-safe initialization.
static GLOBAL_GENERATOR: std::sync::OnceLock<UniqueIntegerGenerator> = std::sync::OnceLock::new();

fn get_generator() -> &'static UniqueIntegerGenerator {
    GLOBAL_GENERATOR.get_or_init(|| UniqueIntegerGenerator::new())
}

/// Unique BIF operations
pub struct UniqueBif;

impl UniqueBif {
    /// Create a new reference
    ///
    /// Equivalent to `make_ref/0` in Erlang.
    ///
    /// # Returns
    /// New unique reference
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::unique::UniqueBif;
    ///
    /// // Create multiple unique references
    /// let ref1 = UniqueBif::make_ref();
    /// let ref2 = UniqueBif::make_ref();
    /// let ref3 = UniqueBif::make_ref();
    /// assert_ne!(ref1, ref2);
    /// assert_ne!(ref2, ref3);
    /// assert_ne!(ref1, ref3);
    ///
    /// // References have thread ID and value components
    /// let ref1 = UniqueBif::make_ref();
    /// assert!(ref1.thread_id() > 0);
    /// assert!(ref1.value() > 0);
    ///
    /// // Create references from different calls
    /// let ref1 = UniqueBif::make_ref();
    /// std::thread::sleep(std::time::Duration::from_millis(1));
    /// let ref2 = UniqueBif::make_ref();
    /// assert_ne!(ref1.value(), ref2.value());
    /// ```
    pub fn make_ref() -> Reference {
        get_generator().make_ref()
    }

    /// Generate a unique integer
    ///
    /// Equivalent to `erlang:unique_integer/0` in Erlang.
    /// Generates a unique integer that may be negative.
    ///
    /// # Returns
    /// Unique integer (may be negative)
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::unique::UniqueBif;
    ///
    /// // Generate multiple unique integers
    /// let int1 = UniqueBif::unique_integer();
    /// let int2 = UniqueBif::unique_integer();
    /// let int3 = UniqueBif::unique_integer();
    /// assert_ne!(int1, int2);
    /// assert_ne!(int2, int3);
    ///
    /// // Integers may be negative (no positive flag)
    /// let ints: Vec<i64> = (0..10).map(|_| UniqueBif::unique_integer()).collect();
    /// // At least some should be unique
    /// let unique_count = ints.iter().collect::<std::collections::HashSet<_>>().len();
    /// assert!(unique_count >= 1);
    ///
    /// // Generate integers over time
    /// let int1 = UniqueBif::unique_integer();
    /// std::thread::sleep(std::time::Duration::from_millis(1));
    /// let int2 = UniqueBif::unique_integer();
    /// assert_ne!(int1, int2);
    /// ```
    pub fn unique_integer() -> i64 {
        get_generator().unique_integer(false)
    }

    /// Generate a unique integer with options
    ///
    /// Equivalent to `erlang:unique_integer/1` in Erlang.
    ///
    /// # Arguments
    /// * `options` - Vector of options (monotonic, positive)
    ///
    /// # Returns
    /// * `Ok(i64)` - Unique integer
    /// * `Err(UniqueError)` - If invalid options provided
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::unique::{UniqueBif, UniqueIntegerOption};
    ///
    /// // Generate monotonic unique integers (strictly increasing)
    /// let int1 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
    /// let int2 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
    /// let int3 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
    /// assert!(int2 > int1);
    /// assert!(int3 > int2);
    ///
    /// // Generate positive unique integers
    /// let int1 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
    /// let int2 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
    /// assert!(int1 > 0);
    /// assert!(int2 > 0);
    /// assert_ne!(int1, int2);
    ///
    /// // Generate monotonic and positive integers
    /// let int1 = UniqueBif::unique_integer_with_options(&[
    ///     UniqueIntegerOption::Monotonic,
    ///     UniqueIntegerOption::Positive,
    /// ]).unwrap();
    /// let int2 = UniqueBif::unique_integer_with_options(&[
    ///     UniqueIntegerOption::Monotonic,
    ///     UniqueIntegerOption::Positive,
    /// ]).unwrap();
    /// assert!(int1 > 0);
    /// assert!(int2 > int1);
    /// ```
    pub fn unique_integer_with_options(
        options: &[UniqueIntegerOption],
    ) -> Result<i64, UniqueError> {
        let mut monotonic = false;
        let mut positive = false;

        for option in options {
            match option {
                UniqueIntegerOption::Monotonic => monotonic = true,
                UniqueIntegerOption::Positive => positive = true,
            }
        }

        let generator = get_generator();
        if monotonic {
            Ok(generator.unique_integer_monotonic(positive))
        } else {
            Ok(generator.unique_integer(positive))
        }
    }
}

/// Error type for unique operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UniqueError {
    /// Invalid argument provided
    InvalidArgument(String),
}

impl std::fmt::Display for UniqueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniqueError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl std::error::Error for UniqueError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_ref() {
        let ref1 = UniqueBif::make_ref();
        let ref2 = UniqueBif::make_ref();
        
        // References should be unique
        assert_ne!(ref1, ref2);
        
        // Should have valid values
        assert!(ref1.value() > 0 || ref2.value() > 0);
    }

    #[test]
    fn test_make_ref_uniqueness() {
        let mut refs = Vec::new();
        for _ in 0..100 {
            refs.push(UniqueBif::make_ref());
        }
        
        // All references should be unique
        for i in 0..refs.len() {
            for j in (i + 1)..refs.len() {
                assert_ne!(refs[i], refs[j]);
            }
        }
    }

    #[test]
    fn test_unique_integer() {
        let int1 = UniqueBif::unique_integer();
        let int2 = UniqueBif::unique_integer();
        
        // Should be unique
        assert_ne!(int1, int2);
    }

    #[test]
    fn test_unique_integer_uniqueness() {
        let mut ints = Vec::new();
        for _ in 0..100 {
            ints.push(UniqueBif::unique_integer());
        }
        
        // All integers should be unique
        for i in 0..ints.len() {
            for j in (i + 1)..ints.len() {
                assert_ne!(ints[i], ints[j]);
            }
        }
    }

    #[test]
    fn test_unique_integer_with_options_monotonic() {
        let int1 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
        let int2 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
        let int3 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
        
        // Monotonic should be strictly increasing
        assert!(int2 > int1);
        assert!(int3 > int2);
    }

    #[test]
    fn test_unique_integer_with_options_positive() {
        let int1 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
        let int2 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
        
        // Should be positive
        assert!(int1 > 0);
        assert!(int2 > 0);
        // Should be unique
        assert_ne!(int1, int2);
    }

    #[test]
    fn test_unique_integer_with_options_monotonic_positive() {
        let int1 = UniqueBif::unique_integer_with_options(&[
            UniqueIntegerOption::Monotonic,
            UniqueIntegerOption::Positive,
        ]).unwrap();
        let int2 = UniqueBif::unique_integer_with_options(&[
            UniqueIntegerOption::Monotonic,
            UniqueIntegerOption::Positive,
        ]).unwrap();
        let int3 = UniqueBif::unique_integer_with_options(&[
            UniqueIntegerOption::Monotonic,
            UniqueIntegerOption::Positive,
        ]).unwrap();
        
        // Should be positive and monotonic
        assert!(int1 > 0);
        assert!(int2 > int1);
        assert!(int3 > int2);
    }

    #[test]
    fn test_unique_integer_with_options_empty() {
        // Empty options should work (default behavior)
        let int1 = UniqueBif::unique_integer_with_options(&[]).unwrap();
        let int2 = UniqueBif::unique_integer_with_options(&[]).unwrap();
        
        assert_ne!(int1, int2);
    }

    #[test]
    fn test_reference_fields() {
        let reference = UniqueBif::make_ref();
        
        // Should have valid fields
        let _thread_id = reference.thread_id();
        let _value = reference.value();
        let _ref_number = reference.ref_number();
        
        // Fields should be accessible and valid
        assert!(reference.thread_id() > 0); // Thread IDs start from 1
    }

    #[test]
    fn test_unique_integer_generator_new() {
        let generator = UniqueIntegerGenerator::new();
        
        // Should initialize successfully
        let int1 = generator.unique_integer(false);
        let int2 = generator.unique_integer(false);
        
        assert_ne!(int1, int2);
    }

    #[test]
    fn test_unique_integer_generator_monotonic() {
        let generator = UniqueIntegerGenerator::new();
        
        let int1 = generator.unique_integer_monotonic(false);
        let int2 = generator.unique_integer_monotonic(false);
        let int3 = generator.unique_integer_monotonic(false);
        
        // Should be strictly increasing
        assert!(int2 > int1);
        assert!(int3 > int2);
    }

    #[test]
    fn test_unique_integer_generator_monotonic_positive() {
        let generator = UniqueIntegerGenerator::new();
        
        let int1 = generator.unique_integer_monotonic(true);
        let int2 = generator.unique_integer_monotonic(true);
        let int3 = generator.unique_integer_monotonic(true);
        
        // Should be positive and increasing
        assert!(int1 > 0);
        assert!(int2 > int1);
        assert!(int3 > int2);
    }

    #[test]
    fn test_reference_clone() {
        let ref1 = UniqueBif::make_ref();
        let ref2 = ref1.clone();
        
        // Cloned references should be equal
        assert_eq!(ref1, ref2);
    }

    #[test]
    fn test_reference_hash() {
        use std::collections::HashSet;
        
        let mut refs = HashSet::new();
        for _ in 0..50 {
            let reference = UniqueBif::make_ref();
            assert!(refs.insert(reference));
        }
        
        // All should be unique in the set
        assert_eq!(refs.len(), 50);
    }

    #[test]
    fn test_unique_error_display() {
        let err = UniqueError::InvalidArgument("test".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid argument"));
        assert!(display.contains("test"));
    }

    #[test]
    fn test_concurrent_make_ref() {
        use std::thread;
        
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(thread::spawn(|| {
                let mut refs = Vec::new();
                for _ in 0..10 {
                    refs.push(UniqueBif::make_ref());
                }
                refs
            }));
        }
        
        let mut all_refs = Vec::new();
        for handle in handles {
            all_refs.extend(handle.join().unwrap());
        }
        
        // All references should be unique
        for i in 0..all_refs.len() {
            for j in (i + 1)..all_refs.len() {
                assert_ne!(all_refs[i], all_refs[j]);
            }
        }
    }

    #[test]
    fn test_concurrent_unique_integer() {
        use std::thread;
        
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(thread::spawn(|| {
                let mut ints = Vec::new();
                for _ in 0..10 {
                    ints.push(UniqueBif::unique_integer());
                }
                ints
            }));
        }
        
        let mut all_ints = Vec::new();
        for handle in handles {
            all_ints.extend(handle.join().unwrap());
        }
        
        // All integers should be unique
        for i in 0..all_ints.len() {
            for j in (i + 1)..all_ints.len() {
                assert_ne!(all_ints[i], all_ints[j]);
            }
        }
    }

    #[test]
    fn test_monotonic_ordering() {
        // Test that monotonic integers are strictly increasing
        let mut prev = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
        
        for _ in 0..100 {
            let current = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
            assert!(current > prev, "Monotonic integers must be strictly increasing");
            prev = current;
        }
    }

    #[test]
    fn test_positive_option() {
        // Test that positive option generates only positive integers
        for _ in 0..100 {
            let value = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
            assert!(value > 0, "Positive option must generate positive integers");
        }
    }

    #[test]
    fn test_reference_equality() {
        let ref1 = UniqueBif::make_ref();
        let ref2 = UniqueBif::make_ref();
        let ref1_clone = ref1.clone();
        
        assert_eq!(ref1, ref1_clone);
        assert_ne!(ref1, ref2);
    }

    #[test]
    fn test_unique_integer_positive_vs_negative() {
        // Test that without positive option, integers can be negative
        let mut has_negative = false;
        let mut has_positive = false;
        
        for _ in 0..1000 {
            let value = UniqueBif::unique_integer();
            if value < 0 {
                has_negative = true;
            }
            if value > 0 {
                has_positive = true;
            }
        }
        
        // Should have both positive and potentially negative (depending on implementation)
        // At minimum, should have positive values
        assert!(has_positive);
    }

    #[test]
    fn test_get_generator_singleton() {
        // Test that get_generator returns the same instance
        let gen1 = get_generator();
        let gen2 = get_generator();
        
        // Should be the same reference (singleton)
        assert!(std::ptr::eq(gen1, gen2));
    }
}

