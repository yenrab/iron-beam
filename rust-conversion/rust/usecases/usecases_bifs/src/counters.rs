//! Atomic Counters Built-in Functions
//!
//! Provides atomic counter operations with write concurrency support.
//! Each counter consists of multiple atomic instances (one per scheduler + base value)
//! to allow concurrent writes without contention.
//!
//! This module uses safe Rust atomic operations instead of unsafe FFI calls.

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

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

/// Counter reference - wraps atomic counters
///
/// Each counter has multiple atomic instances to support concurrent writes.
/// The first atomic is the base value, and subsequent ones are per-scheduler.
#[derive(Clone, Debug)]
pub struct CounterRef {
    /// Number of counters in this array
    arity: usize,
    /// Atomic values (base + per-scheduler instances)
    /// For simplicity, we use a single atomic per counter (not per-scheduler)
    /// This maintains the same API but with simpler implementation
    atomics: Arc<Vec<AtomicI64>>,
    /// Memory size in bytes
    memory_size: usize,
}

impl CounterRef {
    /// Create a new counter reference with the specified number of counters
    ///
    /// # Arguments
    /// * `count` - Number of counters to create (must be > 0)
    ///
    /// # Returns
    /// * `Ok(CounterRef)` if successful
    /// * `Err(CountersError)` if count is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CounterRef;
    ///
    /// // Create a counter array with 10 counters
    /// let counters = CounterRef::new(10).unwrap();
    /// assert_eq!(counters.arity(), 10);
    ///
    /// // Create a single counter
    /// let single = CounterRef::new(1).unwrap();
    /// assert_eq!(single.arity(), 1);
    ///
    /// // Invalid: zero counters
    /// assert!(CounterRef::new(0).is_err());
    /// ```
    pub fn new(count: usize) -> Result<Self, CountersError> {
        if count == 0 {
            return Err(CountersError::InvalidArgument(
                "Counter count must be greater than 0".to_string(),
            ));
        }

        // Check for overflow
        let max_count = usize::MAX / (std::mem::size_of::<AtomicI64>() * 2);
        if count > max_count {
            return Err(CountersError::SystemLimit(format!(
                "Counter count {} exceeds system limit {}",
                count, max_count
            )));
        }

        // Create atomic counters, all initialized to 0
        let atomics: Vec<AtomicI64> = (0..count)
            .map(|_| AtomicI64::new(0))
            .collect();

        // Calculate memory size (approximate)
        let memory_size = std::mem::size_of::<CounterRef>()
            + (count * std::mem::size_of::<AtomicI64>());

        Ok(CounterRef {
            arity: count,
            atomics: Arc::new(atomics),
            memory_size,
        })
    }

    /// Get the value of a counter
    ///
    /// Reads all atomic instances for the counter and returns the sum.
    /// In the simplified implementation, this just reads the single atomic.
    ///
    /// # Arguments
    /// * `index` - Counter index (1-based)
    ///
    /// # Returns
    /// * `Ok(i64)` - The counter value
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CounterRef;
    ///
    /// // Get initial value (should be 0)
    /// let counters = CounterRef::new(5).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), 0);
    ///
    /// // Get value after setting
    /// let counters = CounterRef::new(3).unwrap();
    /// counters.put(2, 42).unwrap();
    /// assert_eq!(counters.get(2).unwrap(), 42);
    ///
    /// // Get value after adding
    /// let counters = CounterRef::new(3).unwrap();
    /// counters.add(1, 10).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), 10);
    /// ```
    pub fn get(&self, index: usize) -> Result<i64, CountersError> {
        if index == 0 || index > self.arity {
            return Err(CountersError::InvalidArgument(format!(
                "Counter index {} out of range [1, {}]",
                index, self.arity
            )));
        }

        // Convert to 0-based index
        let idx = index - 1;
        let value = self.atomics[idx].load(Ordering::Relaxed);
        Ok(value)
    }

    /// Add a value to a counter
    ///
    /// Atomically adds the increment to the counter.
    ///
    /// # Arguments
    /// * `index` - Counter index (1-based)
    /// * `increment` - Value to add (can be negative)
    ///
    /// # Returns
    /// * `Ok(i64)` - The new counter value after addition
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CounterRef;
    ///
    /// // Add positive value
    /// let counters = CounterRef::new(3).unwrap();
    /// let new_value = counters.add(1, 5).unwrap();
    /// assert_eq!(new_value, 5);
    /// assert_eq!(counters.get(1).unwrap(), 5);
    ///
    /// // Add negative value (decrement)
    /// let counters = CounterRef::new(3).unwrap();
    /// counters.put(2, 10).unwrap();
    /// let new_value = counters.add(2, -3).unwrap();
    /// assert_eq!(new_value, 7);
    ///
    /// // Multiple additions
    /// let counters = CounterRef::new(2).unwrap();
    /// counters.add(1, 1).unwrap();
    /// counters.add(1, 2).unwrap();
    /// counters.add(1, 3).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), 6);
    /// ```
    pub fn add(&self, index: usize, increment: i64) -> Result<i64, CountersError> {
        if index == 0 || index > self.arity {
            return Err(CountersError::InvalidArgument(format!(
                "Counter index {} out of range [1, {}]",
                index, self.arity
            )));
        }

        // Convert to 0-based index
        let idx = index - 1;
        let new_value = self.atomics[idx].fetch_add(increment, Ordering::Relaxed) + increment;
        Ok(new_value)
    }

    /// Set a counter to a specific value
    ///
    /// Atomically sets the counter to the specified value.
    ///
    /// # Arguments
    /// * `index` - Counter index (1-based)
    /// * `value` - New value for the counter
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CounterRef;
    ///
    /// // Set a counter value
    /// let counters = CounterRef::new(3).unwrap();
    /// counters.put(1, 42).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), 42);
    ///
    /// // Set multiple counters
    /// let counters = CounterRef::new(5).unwrap();
    /// counters.put(1, 10).unwrap();
    /// counters.put(2, 20).unwrap();
    /// counters.put(3, 30).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), 10);
    /// assert_eq!(counters.get(2).unwrap(), 20);
    /// assert_eq!(counters.get(3).unwrap(), 30);
    ///
    /// // Set negative value
    /// let counters = CounterRef::new(2).unwrap();
    /// counters.put(1, -100).unwrap();
    /// assert_eq!(counters.get(1).unwrap(), -100);
    /// ```
    pub fn put(&self, index: usize, value: i64) -> Result<(), CountersError> {
        if index == 0 || index > self.arity {
            return Err(CountersError::InvalidArgument(format!(
                "Counter index {} out of range [1, {}]",
                index, self.arity
            )));
        }

        // Convert to 0-based index
        let idx = index - 1;
        self.atomics[idx].store(value, Ordering::Relaxed);
        Ok(())
    }

    /// Get information about the counter array
    ///
    /// Returns a map with:
    /// - `size`: Number of counters
    /// - `memory`: Memory size in bytes
    ///
    /// # Returns
    /// Counter information as a map
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CounterRef;
    ///
    /// // Get info for small counter array
    /// let counters = CounterRef::new(5).unwrap();
    /// let info = counters.info();
    /// assert_eq!(info.size, 5);
    /// assert!(info.memory > 0);
    ///
    /// // Get info for larger counter array
    /// let counters = CounterRef::new(100).unwrap();
    /// let info = counters.info();
    /// assert_eq!(info.size, 100);
    /// assert!(info.memory > 0);
    ///
    /// // Get info for single counter
    /// let counters = CounterRef::new(1).unwrap();
    /// let info = counters.info();
    /// assert_eq!(info.size, 1);
    /// ```
    pub fn info(&self) -> CounterInfo {
        CounterInfo {
            size: self.arity,
            memory: self.memory_size,
        }
    }

    /// Get the number of counters
    pub fn arity(&self) -> usize {
        self.arity
    }
}

/// Counter information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterInfo {
    /// Number of counters
    pub size: usize,
    /// Memory size in bytes
    pub memory: usize,
}

/// Counters BIF operations
pub struct CountersBif;

impl CountersBif {
    /// Create a new counter array
    ///
    /// Equivalent to `counters:new/1` in Erlang.
    ///
    /// # Arguments
    /// * `count` - Number of counters to create
    ///
    /// # Returns
    /// * `Ok(CounterRef)` - New counter reference
    /// * `Err(CountersError)` - If creation fails
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CountersBif;
    ///
    /// // Create a counter array with 10 counters
    /// let counters = CountersBif::new(10).unwrap();
    /// assert_eq!(counters.arity(), 10);
    ///
    /// // Create a single counter
    /// let single = CountersBif::new(1).unwrap();
    /// assert_eq!(single.arity(), 1);
    ///
    /// // Invalid: zero counters
    /// assert!(CountersBif::new(0).is_err());
    /// ```
    pub fn new(count: usize) -> Result<CounterRef, CountersError> {
        CounterRef::new(count)
    }

    /// Get a counter value
    ///
    /// Equivalent to `counters:get/2` in Erlang.
    ///
    /// # Arguments
    /// * `counter_ref` - Counter reference
    /// * `index` - Counter index (1-based)
    ///
    /// # Returns
    /// * `Ok(i64)` - Counter value
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::{CountersBif, CounterRef};
    ///
    /// // Get initial value
    /// let counters = CountersBif::new(3).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 0);
    ///
    /// // Get value after setting
    /// let counters = CountersBif::new(3).unwrap();
    /// CountersBif::put(&counters, 2, 100).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 2).unwrap(), 100);
    ///
    /// // Get value after adding
    /// let counters = CountersBif::new(2).unwrap();
    /// CountersBif::add(&counters, 1, 25).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 25);
    /// ```
    pub fn get(counter_ref: &CounterRef, index: usize) -> Result<i64, CountersError> {
        counter_ref.get(index)
    }

    /// Add to a counter
    ///
    /// Equivalent to `counters:add/3` in Erlang.
    ///
    /// # Arguments
    /// * `counter_ref` - Counter reference
    /// * `index` - Counter index (1-based)
    /// * `increment` - Value to add
    ///
    /// # Returns
    /// * `Ok(i64)` - New counter value
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CountersBif;
    ///
    /// // Add positive value
    /// let counters = CountersBif::new(3).unwrap();
    /// let new_value = CountersBif::add(&counters, 1, 10).unwrap();
    /// assert_eq!(new_value, 10);
    ///
    /// // Add negative value (decrement)
    /// let counters = CountersBif::new(2).unwrap();
    /// CountersBif::put(&counters, 1, 50).unwrap();
    /// let new_value = CountersBif::add(&counters, 1, -15).unwrap();
    /// assert_eq!(new_value, 35);
    ///
    /// // Multiple additions
    /// let counters = CountersBif::new(2).unwrap();
    /// CountersBif::add(&counters, 1, 5).unwrap();
    /// CountersBif::add(&counters, 1, 3).unwrap();
    /// CountersBif::add(&counters, 1, 2).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 10);
    /// ```
    pub fn add(counter_ref: &CounterRef, index: usize, increment: i64) -> Result<i64, CountersError> {
        counter_ref.add(index, increment)
    }

    /// Set a counter value
    ///
    /// Equivalent to `counters:put/3` in Erlang.
    ///
    /// # Arguments
    /// * `counter_ref` - Counter reference
    /// * `index` - Counter index (1-based)
    /// * `value` - New value
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    /// * `Err(CountersError)` - If index is invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CountersBif;
    ///
    /// // Set a counter value
    /// let counters = CountersBif::new(3).unwrap();
    /// CountersBif::put(&counters, 1, 42).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 42);
    ///
    /// // Set multiple counters
    /// let counters = CountersBif::new(5).unwrap();
    /// CountersBif::put(&counters, 1, 10).unwrap();
    /// CountersBif::put(&counters, 2, 20).unwrap();
    /// CountersBif::put(&counters, 3, 30).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 10);
    /// assert_eq!(CountersBif::get(&counters, 2).unwrap(), 20);
    ///
    /// // Overwrite existing value
    /// let counters = CountersBif::new(2).unwrap();
    /// CountersBif::put(&counters, 1, 100).unwrap();
    /// CountersBif::put(&counters, 1, 200).unwrap();
    /// assert_eq!(CountersBif::get(&counters, 1).unwrap(), 200);
    /// ```
    pub fn put(counter_ref: &CounterRef, index: usize, value: i64) -> Result<(), CountersError> {
        counter_ref.put(index, value)
    }

    /// Get counter information
    ///
    /// Equivalent to `counters:info/1` in Erlang.
    ///
    /// # Arguments
    /// * `counter_ref` - Counter reference
    ///
    /// # Returns
    /// Counter information
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CountersBif;
    ///
    /// // Get info for small counter array
    /// let counters = CountersBif::new(5).unwrap();
    /// let info = CountersBif::info(&counters);
    /// assert_eq!(info.size, 5);
    /// assert!(info.memory > 0);
    ///
    /// // Get info for larger counter array
    /// let counters = CountersBif::new(100).unwrap();
    /// let info = CountersBif::info(&counters);
    /// assert_eq!(info.size, 100);
    /// assert!(info.memory > 0);
    ///
    /// // Get info for single counter
    /// let counters = CountersBif::new(1).unwrap();
    /// let info = CountersBif::info(&counters);
    /// assert_eq!(info.size, 1);
    /// ```
    pub fn info(counter_ref: &CounterRef) -> CounterInfo {
        counter_ref.info()
    }

    /// Calculate ceiling division
    ///
    /// Helper function: (dividend + divisor - 1) / divisor
    ///
    /// # Arguments
    /// * `dividend` - Number to divide
    /// * `divisor` - Divisor
    ///
    /// # Returns
    /// Ceiling of dividend / divisor
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::counters::CountersBif;
    ///
    /// // Exact division
    /// assert_eq!(CountersBif::div_ceil(10, 2), 5);
    ///
    /// // Ceiling division (rounds up)
    /// assert_eq!(CountersBif::div_ceil(10, 3), 4);
    /// assert_eq!(CountersBif::div_ceil(11, 3), 4);
    ///
    /// // Edge cases
    /// assert_eq!(CountersBif::div_ceil(0, 5), 0);
    /// assert_eq!(CountersBif::div_ceil(1, 5), 1);
    /// ```
    pub fn div_ceil(dividend: usize, divisor: usize) -> usize {
        if divisor == 0 {
            return 0; // Avoid division by zero
        }
        (dividend + divisor - 1) / divisor
    }
}

/// Error type for counter operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountersError {
    /// Invalid argument provided
    InvalidArgument(String),
    /// System limit exceeded
    SystemLimit(String),
    /// Counter reference not found
    NotFound,
}

impl std::fmt::Display for CountersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CountersError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            CountersError::SystemLimit(msg) => write!(f, "System limit: {}", msg),
            CountersError::NotFound => write!(f, "Counter reference not found"),
        }
    }
}

impl std::error::Error for CountersError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_counter() {
        let counters = CountersBif::new(10).unwrap();
        assert_eq!(counters.arity(), 10);
    }

    #[test]
    fn test_new_counter_zero() {
        let result = CountersBif::new(0);
        assert!(matches!(result, Err(CountersError::InvalidArgument(_))));
    }

    #[test]
    fn test_get_counter() {
        let counters = CountersBif::new(5).unwrap();
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 0); // Initialized to 0
    }

    #[test]
    fn test_get_counter_invalid_index() {
        let counters = CountersBif::new(5).unwrap();
        assert!(CountersBif::get(&counters, 0).is_err());
        assert!(CountersBif::get(&counters, 6).is_err());
    }

    #[test]
    fn test_add_counter() {
        let counters = CountersBif::new(5).unwrap();
        let new_value = CountersBif::add(&counters, 1, 10).unwrap();
        assert_eq!(new_value, 10);
        
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 10);
    }

    #[test]
    fn test_add_counter_multiple() {
        let counters = CountersBif::new(5).unwrap();
        CountersBif::add(&counters, 1, 5).unwrap();
        CountersBif::add(&counters, 1, 3).unwrap();
        CountersBif::add(&counters, 1, -2).unwrap();
        
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 6);
    }

    #[test]
    fn test_add_counter_negative() {
        let counters = CountersBif::new(5).unwrap();
        CountersBif::add(&counters, 1, 10).unwrap();
        CountersBif::add(&counters, 1, -5).unwrap();
        
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 5);
    }

    #[test]
    fn test_put_counter() {
        let counters = CountersBif::new(5).unwrap();
        CountersBif::put(&counters, 1, 42).unwrap();
        
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_put_counter_overwrite() {
        let counters = CountersBif::new(5).unwrap();
        CountersBif::add(&counters, 1, 10).unwrap();
        CountersBif::put(&counters, 1, 100).unwrap();
        
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_put_counter_invalid_index() {
        let counters = CountersBif::new(5).unwrap();
        assert!(CountersBif::put(&counters, 0, 10).is_err());
        assert!(CountersBif::put(&counters, 6, 10).is_err());
    }

    #[test]
    fn test_info() {
        let counters = CountersBif::new(10).unwrap();
        let info = CountersBif::info(&counters);
        assert_eq!(info.size, 10);
        assert!(info.memory > 0);
    }

    #[test]
    fn test_multiple_counters() {
        let counters = CountersBif::new(3).unwrap();
        
        CountersBif::put(&counters, 1, 10).unwrap();
        CountersBif::put(&counters, 2, 20).unwrap();
        CountersBif::put(&counters, 3, 30).unwrap();
        
        assert_eq!(CountersBif::get(&counters, 1).unwrap(), 10);
        assert_eq!(CountersBif::get(&counters, 2).unwrap(), 20);
        assert_eq!(CountersBif::get(&counters, 3).unwrap(), 30);
    }

    #[test]
    fn test_div_ceil() {
        assert_eq!(CountersBif::div_ceil(10, 3), 4);
        assert_eq!(CountersBif::div_ceil(10, 5), 2);
        assert_eq!(CountersBif::div_ceil(10, 4), 3);
        assert_eq!(CountersBif::div_ceil(0, 5), 0);
    }

    #[test]
    fn test_div_ceil_exact() {
        assert_eq!(CountersBif::div_ceil(10, 2), 5);
        assert_eq!(CountersBif::div_ceil(20, 4), 5);
    }

    #[test]
    fn test_counter_ref_clone() {
        let counters1 = CountersBif::new(5).unwrap();
        let counters2 = counters1.clone();
        
        CountersBif::put(&counters1, 1, 100).unwrap();
        
        // Both should see the same value (shared atomics)
        assert_eq!(CountersBif::get(&counters1, 1).unwrap(), 100);
        assert_eq!(CountersBif::get(&counters2, 1).unwrap(), 100);
    }

    #[test]
    fn test_counter_error_display() {
        let err1 = CountersError::InvalidArgument("test".to_string());
        let display = format!("{}", err1);
        assert!(display.contains("Invalid argument"));
        assert!(display.contains("test"));
        
        let err2 = CountersError::SystemLimit("limit".to_string());
        let display = format!("{}", err2);
        assert!(display.contains("System limit"));
        assert!(display.contains("limit"));
        
        let err3 = CountersError::NotFound;
        let display = format!("{}", err3);
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_add_returns_new_value() {
        let counters = CountersBif::new(5).unwrap();
        let new_value = CountersBif::add(&counters, 1, 5).unwrap();
        assert_eq!(new_value, 5);
        
        // Verify it matches get
        assert_eq!(CountersBif::get(&counters, 1).unwrap(), 5);
    }

    #[test]
    fn test_concurrent_operations() {
        use std::thread;
        
        let counters = CountersBif::new(1).unwrap();
        let counters_clone = counters.clone();
        
        // Spawn multiple threads adding to the same counter
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = counters_clone.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        CountersBif::add(&c, 1, 1).unwrap();
                    }
                })
            })
            .collect();
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Should have added 10 * 100 = 1000
        let value = CountersBif::get(&counters, 1).unwrap();
        assert_eq!(value, 1000);
    }

    #[test]
    fn test_large_counter_array() {
        let counters = CountersBif::new(1000).unwrap();
        assert_eq!(counters.arity(), 1000);
        
        // Test first and last counter
        CountersBif::put(&counters, 1, 1).unwrap();
        CountersBif::put(&counters, 1000, 1000).unwrap();
        
        assert_eq!(CountersBif::get(&counters, 1).unwrap(), 1);
        assert_eq!(CountersBif::get(&counters, 1000).unwrap(), 1000);
    }

    #[test]
    fn test_info_memory_size() {
        let counters1 = CountersBif::new(10).unwrap();
        let info1 = CountersBif::info(&counters1);
        
        let counters2 = CountersBif::new(100).unwrap();
        let info2 = CountersBif::info(&counters2);
        
        // Larger array should use more memory
        assert!(info2.memory > info1.memory);
        assert_eq!(info2.size, 100);
        assert_eq!(info1.size, 10);
    }
}

