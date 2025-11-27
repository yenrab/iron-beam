//! Big Number Operations Module
//!
//! This module provides arbitrary precision integer operations for the Erlang/OTP
//! runtime system, providing support for integers of any size beyond the limits of standard
//! machine word types.
//!
//! # Purpose
//!
//! Erlang supports arbitrary precision integers (bignums) that can represent
//! values of any size. This module provides the core operations needed to
//! manipulate these big numbers, including:
//!
//! - **Creation and Conversion**: Convert between big numbers and standard
//!   integer types (i32, i64, u32, u64) as well as floating-point numbers (f64).
//!   The conversion from f64 handles any finite f64 value by using digit-based conversion, allowing
//!   conversion of very large floating-point values that exceed i64 range.
//!
//! - **Arithmetic Operations**: Addition, subtraction, multiplication, division,
//!   and remainder operations that work with arbitrary precision. These operations
//!   maintain mathematical correctness for values of any size, enabling Erlang's
//!   ability to perform calculations on numbers that exceed machine word limits.
//!
//! - **Bitwise Operations**: Bitwise AND, OR, XOR, NOT, and shift operations that
//!   maintain two's complement semantics matching the C implementation exactly.
//!   This ensures that bitwise operations produce the same results as the original
//!   C code, which is critical for compatibility with existing Erlang code.
//!
//! - **Comparison Operations**: Signed and unsigned comparison functions for
//!   ordering big numbers. The signed comparison (`comp`) respects the sign of
//!   numbers, while unsigned comparison (`ucomp`) compares absolute values,
//!   ignoring sign.
//!
//! - **String Conversion**: Convert big numbers to string representations in
//!   various bases (2-36), supporting binary, octal, decimal, hexadecimal, and
//!   arbitrary base conversions. This is essential for displaying big numbers
//!   to users and for serialization purposes.
//!
//! # Implementation Details
//!
//! This module uses the `malachite` crate for high-performance arbitrary-precision
//! arithmetic. Malachite uses two's complement representation internally, which
//! matches the C code's bitwise operation semantics exactly. This ensures that
//! operations produce the same results as the original implementation.
//!
//! The conversion from f64 to big number handles any finite f64 value by using digit-based conversion. This allows
//! conversion of very large floating-point values that exceed i64 range. The algorithm
//! works by:
//!
//! 1. Extracting the sign and making the value positive
//! 2. Scaling down by digit base (2^32) to count digits needed
//! 3. Extracting digits iteratively by multiplying, truncating, and subtracting
//! 4. Building the Integer from the extracted digits
//!
//! # Examples
//!
//! ## Basic Arithmetic
//!
//! ```rust
//! use entities_utilities::BigNumber;
//!
//! let a = BigNumber::from_i64(100);
//! let b = BigNumber::from_i64(50);
//!
//! let sum = a.plus(&b);        // 150
//! let diff = a.minus(&b);      // 50
//! let prod = a.times(&b);      // 5000
//! let quot = a.div(&b).unwrap(); // 2
//! ```
//!
//! ## Large Numbers
//!
//! ```rust
//! use entities_utilities::BigNumber;
//!
//! // Create numbers larger than i64::MAX
//! let large = BigNumber::from_u64(u64::MAX);
//! let larger = large.plus(&BigNumber::from_u64(1));
//!
//! // These operations work with arbitrary precision
//! let result = large.times(&larger);
//! ```
//!
//! ## String Conversion
//!
//! ```rust
//! use entities_utilities::BigNumber;
//!
//! let num = BigNumber::from_i64(255);
//! assert_eq!(num.to_string_base(16), "ff");
//! assert_eq!(num.to_string_base(2), "11111111");
//! assert_eq!(num.to_string_base(10), "255");
//! ```

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

use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

/// Big number representation using malachite's Integer.
///
/// This struct wraps malachite's `Integer` type to provide arbitrary precision
/// integer operations. Malachite uses two's complement representation internally,
/// which ensures correct bitwise operation semantics.
///
/// # Purpose
///
/// `BigNumber` provides arbitrary precision integer arithmetic for the
/// Erlang runtime system. It can represent integers of any size, from
/// small values that fit in machine words to extremely large numbers
/// that require multiple words of storage. This enables Erlang's bignum
/// support, allowing calculations on numbers that exceed the limits of
/// standard integer types.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use entities_utilities::BigNumber;
///
/// let num = BigNumber::from_i64(12345);
/// assert_eq!(num.to_i64(), Some(12345));
/// ```
///
/// ## Large Numbers
///
/// ```rust
/// use entities_utilities::BigNumber;
///
/// // Create a number larger than i64::MAX
/// let large = BigNumber::from_u64(u64::MAX);
/// let larger = large.plus(&BigNumber::from_u64(1));
/// ```
///
/// ## Arithmetic Operations
///
/// ```rust
/// use entities_utilities::BigNumber;
///
/// let a = BigNumber::from_i64(100);
/// let b = BigNumber::from_i64(50);
/// let sum = a.plus(&b);
/// assert_eq!(sum.to_i64(), Some(150));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BigNumber {
    value: Integer,
}

impl BigNumber {
    /// Create a new big number from a 64-bit signed integer.
    ///
    /// This function converts a standard `i64` value into a `BigNumber`,
    /// enabling it to participate in arbitrary precision arithmetic operations.
    /// The conversion is lossless and preserves the sign of the input value.
    ///
    /// # Purpose
    ///
    /// In Erlang, integers that exceed the range of small integers (typically
    /// fitting in a machine word) are represented as bignums. This function
    /// provides the primary way to create big numbers from standard integer
    /// types, allowing seamless integration with existing integer values.
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit signed integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigNumber` instance containing the converted value.
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// assert_eq!(num.to_i64(), Some(12345));
    /// ```
    ///
    /// ## Negative Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let neg = BigNumber::from_i64(-12345);
    /// assert_eq!(neg.to_i64(), Some(-12345));
    /// assert!(!neg.is_positive());
    /// ```
    ///
    /// ## Boundary Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let max = BigNumber::from_i64(i64::MAX);
    /// let min = BigNumber::from_i64(i64::MIN);
    /// assert_eq!(max.to_i64(), Some(i64::MAX));
    /// assert_eq!(min.to_i64(), Some(i64::MIN));
    /// ```
    pub fn from_i64(value: i64) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from a 64-bit unsigned integer.
    ///
    /// This function converts a standard `u64` value into a `BigNumber`,
    /// enabling it to participate in arbitrary precision arithmetic operations.
    /// The conversion is lossless and always produces a non-negative result.
    ///
    /// # Purpose
    ///
    /// This function is useful when working with unsigned integer values that
    /// may exceed the range of signed integers. It allows creation of big
    /// numbers from unsigned types without sign extension issues.
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit unsigned integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigNumber` instance containing the converted value (always non-negative).
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_u64(12345);
    /// assert_eq!(num.to_i64(), Some(12345));
    /// assert!(num.is_positive());
    /// ```
    ///
    /// ## Large Unsigned Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // Create from u64::MAX (larger than i64::MAX)
    /// let max = BigNumber::from_u64(u64::MAX);
    /// assert!(max.is_positive());
    /// // Cannot convert back to i64 (too large)
    /// assert!(max.to_i64().is_none());
    /// ```
    ///
    /// ## Arithmetic with Unsigned Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let a = BigNumber::from_u64(100);
    /// let b = BigNumber::from_u64(50);
    /// let sum = a.plus(&b);
    /// assert_eq!(sum.to_i64(), Some(150));
    /// ```
    pub fn from_u64(value: u64) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from a 32-bit unsigned integer.
    ///
    /// This function converts a standard `u32` value into a `BigNumber`,
    /// enabling it to participate in arbitrary precision arithmetic operations.
    /// The conversion is lossless and always produces a non-negative result.
    ///
    /// # Purpose
    ///
    /// This function provides a convenient way to create big numbers from
    /// 32-bit unsigned integers, which are commonly used in system programming
    /// and network protocols.
    ///
    /// # Arguments
    ///
    /// * `value` - The 32-bit unsigned integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigNumber` instance containing the converted value (always non-negative).
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_u32(12345);
    /// assert_eq!(num.to_u32(), Some(12345));
    /// ```
    ///
    /// ## Maximum u32 Value
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let max = BigNumber::from_u32(u32::MAX);
    /// assert_eq!(max.to_u32(), Some(u32::MAX));
    /// assert!(max.is_positive());
    /// ```
    ///
    /// ## Conversion to Other Types
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_u32(1000);
    /// assert_eq!(num.to_i64(), Some(1000));
    /// assert_eq!(num.to_u32(), Some(1000));
    /// ```
    pub fn from_u32(value: u32) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from a 32-bit signed integer.
    ///
    /// This function converts a standard `i32` value into a `BigNumber`,
    /// enabling it to participate in arbitrary precision arithmetic operations.
    /// The conversion is lossless and preserves the sign of the input value.
    ///
    /// # Purpose
    ///
    /// This function provides a convenient way to create big numbers from
    /// 32-bit signed integers, which are commonly used in system programming
    /// and application logic.
    ///
    /// # Arguments
    ///
    /// * `value` - The 32-bit signed integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigNumber` instance containing the converted value.
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i32(12345);
    /// assert_eq!(num.to_i32(), Some(12345));
    /// ```
    ///
    /// ## Negative Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let neg = BigNumber::from_i32(-12345);
    /// assert_eq!(neg.to_i64(), Some(-12345));
    /// assert!(!neg.is_positive());
    /// ```
    ///
    /// ## Boundary Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let max = BigNumber::from_i32(i32::MAX);
    /// let min = BigNumber::from_i32(i32::MIN);
    /// assert_eq!(max.to_i64(), Some(i32::MAX as i64));
    /// assert_eq!(min.to_i64(), Some(i32::MIN as i64));
    /// ```
    pub fn from_i32(value: i32) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from a 64-bit floating-point number.
    ///
    /// This function converts a standard `f64` value into a `BigNumber` by
    /// truncating the fractional part. The conversion handles any finite
    /// floating-point value, including those that exceed the range of `i64`.
    ///
    /// # Purpose
    ///
    /// This function enables conversion from floating-point numbers to big
    /// integers, which is essential for Erlang's type system where numbers
    /// can be represented as either floats or integers. The implementation
    /// handles very large
    /// floating-point values that exceed `i64` range by using digit-based
    /// conversion.
    ///
    /// The algorithm works by:
    /// 1. Extracting the sign and making the value positive
    /// 2. Scaling down by digit base (2^32) to count digits needed
    /// 3. Extracting digits iteratively by multiplying, truncating, and subtracting
    /// 4. Building the Integer from the extracted digits
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit floating-point value to convert
    ///
    /// # Returns
    ///
    /// * `Some(BigNumber)` if the conversion succeeds (value is finite)
    /// * `None` if the conversion fails (NaN, infinity, or out of range)
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_f64(123.456);
    /// assert_eq!(num.unwrap().to_i64(), Some(123)); // Fractional part truncated
    /// ```
    ///
    /// ## Large Floating-Point Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // Convert very large floating-point value (exceeds i64 range)
    /// let large = BigNumber::from_f64(1e20);
    /// assert!(large.is_some());
    /// assert!(large.unwrap().is_positive());
    /// ```
    ///
    /// ## Invalid Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // NaN and infinity cannot be converted
    /// assert!(BigNumber::from_f64(f64::NAN).is_none());
    /// assert!(BigNumber::from_f64(f64::INFINITY).is_none());
    /// assert!(BigNumber::from_f64(f64::NEG_INFINITY).is_none());
    /// ```
    ///
    /// **Note**: When malachite ships a version with native `from_f64` support
    /// (e.g., via `TryFrom<f64>` or a dedicated `Integer::from_f64()` method),
    /// this implementation should be replaced with a call to that function
    /// for better maintainability and potential performance improvements.
    pub fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }

        // Fast path for values that fit in i64 (common case)
        let truncated = value.trunc();
        if truncated >= i64::MIN as f64 && truncated <= i64::MAX as f64 {
            // Check if we can use the fast path without precision loss
            // For values in i64 range, direct conversion is safe and faster
            return Some(Self {
                value: Integer::from(truncated as i64),
            });
        }

        // Slow path: Use algorithm for values outside i64 range
        
        // Extract sign and make positive
        let is_negative = value < 0.0;
        let x = if is_negative { -value } else { value };
        
        // Digit base: 2^32 (matches C's D_MASK + 1 where D_EXP = 32 on typical systems)
        // Using 2^32 instead of 2^64 to avoid f64 precision issues
        const DIGIT_BASE: f64 = 4294967296.0; // 2^32
        
        // Step 1: Count how many digits we need by scaling down
        // This is the "unscale" step in the C code (lines 1969-1975)
        let mut digit_count = 0;
        let mut x_scaled = x;
        while x_scaled >= 1.0 {
            x_scaled /= DIGIT_BASE;
            digit_count += 1;
        }
        
        // If we need 0 digits, the value is 0
        if digit_count == 0 {
            return Some(Self {
                value: Integer::from(0),
            });
        }
        
        // Step 2: Extract digits by scaling up and truncating
        // This matches C code lines 1983-1990
        // We work backwards from the most significant digit (ds-1 down to 0)
        let mut digits = Vec::with_capacity(digit_count);
        
        // x_scaled is now < 1.0 after the scaling down loop
        // Extract digits from most significant (digit_count-1) to least significant (0)
        for _ in 0..digit_count {
            x_scaled *= DIGIT_BASE; // "shift" left (line 1986)
            let digit = x_scaled.trunc() as u32; // trunc (line 1987)
            digits.push(digit);
            x_scaled -= digit as f64; // remove integer part (line 1989)
        }
        
        // Step 3: Build Integer from digits (most significant first)
        // Start with the most significant digit and multiply by base, adding next digits
        let mut result = Integer::from(0);
        let base_int = Integer::from(DIGIT_BASE as u64);
        
        for &digit in &digits {
            result = &result * &base_int;
            result = &result + Integer::from(digit as u64);
        }
        
        // Apply sign (matches C lines 1992-1996)
        if is_negative {
            result = -result;
        }
        
        Some(Self { value: result })
    }

    /// Convert a big number to a 64-bit floating-point number.
    ///
    /// This function converts a `BigNumber` to an `f64` value. The conversion
    /// is lossy for very large numbers due to the limited precision of floating-point
    /// representation. If the value is too large to represent as a finite `f64`,
    /// the function returns `None`.
    ///
    /// # Purpose
    ///
    /// This function enables conversion from big integers to floating-point numbers,
    /// which is essential for Erlang's type system where numbers can be represented
    /// as either floats or integers. The conversion uses malachite's `RoundingFrom`
    /// trait with `Exact` rounding mode to ensure precision when possible.
    ///
    /// # Returns
    ///
    /// * `Some(f64)` if the value can be represented as a finite floating-point number
    /// * `None` if the value is too large (would result in infinity)
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// assert_eq!(num.to_f64(), Some(12345.0));
    /// ```
    ///
    /// ## Negative Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let neg = BigNumber::from_i64(-12345);
    /// assert_eq!(neg.to_f64(), Some(-12345.0));
    /// ```
    ///
    /// ## Very Large Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // Numbers that exceed f64 range return None
    /// let huge = BigNumber::from_f64(1e100).unwrap();
    /// // May return None if too large for f64
    /// let result = huge.to_f64();
    /// ```
    ///
    /// Returns -1 (error) if
    /// the result is not finite.
    pub fn to_f64(&self) -> Option<f64> {
        // Use malachite's RoundingFrom trait for efficient conversion
        // This is more efficient than string conversion and handles overflow correctly
        let (result, _ordering) = f64::rounding_from(&self.value, RoundingMode::Exact);
        
        // Check if result is finite (returns -1 if not finite)
        if result.is_finite() {
            Some(result)
        } else {
            None  // Overflow case - value too large for f64
        }
    }

    /// Convert a big number to a 32-bit unsigned integer.
    ///
    /// This function converts a `BigNumber` to a `u32` value. The conversion
    /// only succeeds if the value is non-negative and within the range of `u32`
    /// (0 to 4,294,967,295).
    ///
    /// # Purpose
    ///
    /// This function provides a way to extract a standard unsigned integer value
    /// from a big number when the value is known to be within the `u32` range.
    /// This is useful for system programming and network protocols that use
    /// 32-bit unsigned integers.
    ///
    /// # Returns
    ///
    /// * `Some(u32)` if the value is non-negative and within `u32` range
    /// * `None` if the value is negative or exceeds `u32::MAX`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// assert_eq!(num.to_u32(), Some(12345));
    /// ```
    ///
    /// ## Maximum u32 Value
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let max = BigNumber::from_u32(u32::MAX);
    /// assert_eq!(max.to_u32(), Some(u32::MAX));
    /// ```
    ///
    /// ## Out of Range
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // Negative numbers cannot be converted to u32
    /// let neg = BigNumber::from_i64(-100);
    /// assert_eq!(neg.to_u32(), None);
    ///
    /// // Values exceeding u32::MAX return None
    /// let too_large = BigNumber::from_u64(u32::MAX as u64 + 1);
    /// assert_eq!(too_large.to_u32(), None);
    /// ```
    pub fn to_u32(&self) -> Option<u32> {
        if self.value >= 0 {
            // Try converting via string for now
            let s = self.value.to_string();
            if let Ok(val) = s.parse::<u64>() {
                if val <= u32::MAX as u64 {
                    Some(val as u32)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Convert a big number to a 64-bit signed integer.
    ///
    /// This function converts a `BigNumber` to an `i64` value. The conversion
    /// only succeeds if the value is within the range of `i64`
    /// (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807).
    ///
    /// # Purpose
    ///
    /// This function provides a way to extract a standard signed integer value
    /// from a big number when the value is known to be within the `i64` range.
    /// This is the most common conversion for big numbers that started as
    /// standard integers.
    ///
    /// # Returns
    ///
    /// * `Some(i64)` if the value is within `i64` range
    /// * `None` if the value exceeds `i64::MAX` or is less than `i64::MIN`
    ///
    /// # Examples
    ///
    /// ## Basic Conversion
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// assert_eq!(num.to_i64(), Some(12345));
    /// ```
    ///
    /// ## Boundary Values
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let max = BigNumber::from_i64(i64::MAX);
    /// let min = BigNumber::from_i64(i64::MIN);
    /// assert_eq!(max.to_i64(), Some(i64::MAX));
    /// assert_eq!(min.to_i64(), Some(i64::MIN));
    /// ```
    ///
    /// ## Out of Range
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// // Values exceeding i64 range return None
    /// let too_large = BigNumber::from_u64(u64::MAX);
    /// assert_eq!(too_large.to_i64(), None);
    /// ```
    pub fn to_i64(&self) -> Option<i64> {
        let s = self.value.to_string();
        s.parse::<i64>().ok()
    }

    /// Check if the big number is positive or zero.
    ///
    /// This function determines whether the big number has a non-negative value.
    /// Zero is considered positive for the purposes of this function.
    ///
    /// # Purpose
    ///
    /// This function provides a quick way to check the sign of a big number
    /// without needing to compare it to zero. It's useful for conditional logic
    /// and validation checks.
    ///
    /// # Returns
    ///
    /// * `true` if the value is greater than or equal to zero
    /// * `false` if the value is negative
    ///
    /// # Examples
    //
    /// ## Positive Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let pos = BigNumber::from_i64(100);
    /// assert!(pos.is_positive());
    /// ```
    //
    /// ## Zero
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let zero = BigNumber::from_i64(0);
    /// assert!(zero.is_positive()); // Zero is considered positive
    /// ```
    //
    /// ## Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let neg = BigNumber::from_i64(-100);
    /// assert!(!neg.is_positive());
    /// ```
    pub fn is_positive(&self) -> bool {
        self.value >= 0
    }

    /// Check if the big number is zero.
    ///
    /// This function determines whether the big number has a value of exactly zero.
    ///
    /// # Purpose
    ///
    /// This function provides a quick way to check if a big number is zero,
    /// which is useful for validation, division checks, and conditional logic.
    ///
    /// # Returns
    ///
    /// * `true` if the value is exactly zero
    /// * `false` otherwise
    ///
    /// # Examples
    //
    /// ## Zero Check
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let zero = BigNumber::from_i64(0);
    /// assert!(zero.is_zero());
    /// ```
    //
    /// ## Non-Zero Values
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(100);
    /// assert!(!num.is_zero());
    //
    /// let neg = BigNumber::from_i64(-100);
    /// assert!(!neg.is_zero());
    /// ```
    //
    /// ## Division Safety Check
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let dividend = BigNumber::from_i64(100);
    /// let divisor = BigNumber::from_i64(0);
    //
    /// if divisor.is_zero() {
    ///     // Handle division by zero error
    ///     println!("Cannot divide by zero!");
    /// } else {
    ///     let result = dividend.div(&divisor);
    /// }
    /// ```
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Add two big numbers: computes `self + other`.
    ///
    /// This function performs arbitrary precision addition of two big numbers,
    /// returning a new `BigNumber` containing the sum. The operation supports
    /// numbers of any size and correctly handles both positive and negative values.
    ///
    /// # Purpose
    ///
    /// Addition is one of the fundamental arithmetic operations needed for
    /// Erlang's bignum support. This function enables calculations on numbers
    /// that exceed the range of standard integer types, maintaining mathematical
    /// correctness for values of any size.
    ///
    /// # Arguments
    ///
    /// * `other` - The big number to add to `self`
    ///
    /// # Returns
    ///
    /// A new `BigNumber` containing the sum of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ## Basic Addition
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// let sum = a.plus(&b);
    /// assert_eq!(sum.to_i64(), Some(150));
    /// ```
    ///
    /// ## Negative Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(-50);
    /// let sum = a.plus(&b);
    /// assert_eq!(sum.to_i64(), Some(50));
    /// ```
    ///
    /// ## Large Numbers
    ///
    /// ```rust
    /// use entities_utilities::BigNumber;
    ///
    /// let a = BigNumber::from_u64(u64::MAX);
    /// let b = BigNumber::from_u64(1);
    /// let sum = a.plus(&b);
    /// // Result exceeds u64::MAX, so cannot convert back
    /// assert!(sum.to_i64().is_none());
    /// ```
    pub fn plus(&self, other: &Self) -> Self {
        Self {
            value: &self.value + &other.value,
        }
    }

    /// Subtract two big numbers: computes `self - other`.
    ///
    /// This function performs arbitrary precision subtraction of two big numbers,
    /// returning a new `BigNumber` containing the difference. The operation supports
    /// numbers of any size and correctly handles both positive and negative values.
    ///
    /// # Purpose
    //
    /// Subtraction is one of the fundamental arithmetic operations needed for
    /// Erlang's bignum support. This function enables calculations on numbers
    /// that exceed the range of standard integer types, maintaining mathematical
    /// correctness for values of any size.
    //
    /// # Arguments
    //
    /// * `other` - The big number to subtract from `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the difference of `self` and `other`.
    //
    /// # Examples
    //
    /// ## Basic Subtraction
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// let diff = a.minus(&b);
    /// assert_eq!(diff.to_i64(), Some(50));
    /// ```
    //
    /// ## Negative Results
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(50);
    /// let b = BigNumber::from_i64(100);
    /// let diff = a.minus(&b);
    /// assert_eq!(diff.to_i64(), Some(-50));
    /// ```
    //
    /// ## Subtracting Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(-50);
    /// let diff = a.minus(&b); // 100 - (-50) = 150
    /// assert_eq!(diff.to_i64(), Some(150));
    /// ```
    pub fn minus(&self, other: &Self) -> Self {
        Self {
            value: &self.value - &other.value,
        }
    }

    /// Multiply two big numbers: computes `self * other`.
    ///
    /// This function performs arbitrary precision multiplication of two big numbers,
    /// returning a new `BigNumber` containing the product. The operation supports
    /// numbers of any size and correctly handles both positive and negative values.
    ///
    /// # Purpose
    //
    /// Multiplication is one of the fundamental arithmetic operations needed for
    /// Erlang's bignum support. This function enables calculations on numbers
    /// that exceed the range of standard integer types, maintaining mathematical
    /// correctness for values of any size. The result can be much larger than
    /// either operand, which is why arbitrary precision is essential.
    //
    /// # Arguments
    //
    /// * `other` - The big number to multiply with `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the product of `self` and `other`.
    //
    /// # Examples
    //
    /// ## Basic Multiplication
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// let prod = a.times(&b);
    /// assert_eq!(prod.to_i64(), Some(5000));
    /// ```
    //
    /// ## Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(-50);
    /// let prod = a.times(&b);
    /// assert_eq!(prod.to_i64(), Some(-5000));
    /// ```
    //
    /// ## Large Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_u64(u64::MAX);
    /// let b = BigNumber::from_u64(2);
    /// let prod = a.times(&b);
    /// // Result exceeds u64::MAX, so cannot convert back
    /// assert!(prod.to_i64().is_none());
    /// ```
    pub fn times(&self, other: &Self) -> Self {
        Self {
            value: &self.value * &other.value,
        }
    }

    /// Divide two big numbers: computes `self / other`.
    ///
    /// This function performs arbitrary precision integer division of two big numbers,
    /// returning a new `BigNumber` containing the quotient. The operation performs
    /// integer division (truncating toward zero), matching Erlang's division semantics.
    ///
    /// # Purpose
    //
    /// Division is one of the fundamental arithmetic operations needed for
    /// Erlang's bignum support. This function enables calculations on numbers
    /// that exceed the range of standard integer types, maintaining mathematical
    /// correctness for values of any size. The result is always an integer,
    /// with any fractional part discarded.
    //
    /// # Arguments
    //
    /// * `other` - The big number to divide `self` by
    //
    /// # Returns
    //
    /// * `Some(BigNumber)` containing the quotient if `other` is not zero
    /// * `None` if dividing by zero
    //
    /// # Examples
    //
    /// ## Basic Division
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// let quot = a.div(&b).unwrap();
    /// assert_eq!(quot.to_i64(), Some(2));
    /// ```
    //
    /// ## Division with Remainder
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(17);
    /// let b = BigNumber::from_i64(5);
    /// let quot = a.div(&b).unwrap();
    /// assert_eq!(quot.to_i64(), Some(3)); // 17 / 5 = 3 (integer division)
    /// ```
    //
    /// ## Division by Zero
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let zero = BigNumber::from_i64(0);
    /// let result = a.div(&zero);
    /// assert!(result.is_none()); // Division by zero returns None
    /// ```
    pub fn div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }
        Some(Self {
            value: &self.value / &other.value,
        })
    }

    /// Compute the remainder of division: computes `self % other`.
    ///
    /// This function performs the modulo operation on two big numbers,
    /// returning the remainder after dividing `self` by `other`. The result
    /// has the same sign as `self`, matching Erlang's modulo semantics.
    ///
    /// # Purpose
    //
    /// The remainder operation is essential for modular arithmetic, which is
    /// commonly used in cryptography, hashing, and cyclic calculations. This
    /// function enables modulo operations on numbers of any size, maintaining
    /// mathematical correctness for values that exceed standard integer types.
    //
    /// # Arguments
    //
    /// * `other` - The big number to divide `self` by (the modulus)
    //
    /// # Returns
    //
    /// * `Some(BigNumber)` containing the remainder if `other` is not zero
    /// * `None` if dividing by zero
    //
    /// # Examples
    //
    /// ## Basic Remainder
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(17);
    /// let b = BigNumber::from_i64(5);
    /// let rem = a.rem(&b).unwrap();
    /// assert_eq!(rem.to_i64(), Some(2)); // 17 % 5 = 2
    /// ```
    //
    /// ## Exact Division
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// let rem = a.rem(&b).unwrap();
    /// assert_eq!(rem.to_i64(), Some(0)); // 100 % 50 = 0 (exact division)
    /// ```
    //
    /// ## Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(-17);
    /// let b = BigNumber::from_i64(5);
    /// let rem = a.rem(&b).unwrap();
    /// assert_eq!(rem.to_i64(), Some(-2)); // -17 % 5 = -2 (same sign as dividend)
    /// ```
    pub fn rem(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }
        Some(Self {
            value: &self.value % &other.value,
        })
    }

    /// Multiply and add in a single operation: computes `self * y + z`.
    ///
    /// This function performs a fused multiply-add operation, which is more
    /// efficient than performing multiplication and addition separately. It
    /// computes the result of multiplying `self` by `y` and then adding `z`
    /// to the product, all in one operation.
    ///
    /// # Purpose
    //
    /// The multiply-add operation is a common pattern in mathematical computations,
    /// particularly in linear algebra and signal processing. Performing it as a
    /// single operation can be more efficient and can avoid intermediate rounding
    /// errors in floating-point arithmetic (though for integers, this is less
    /// of a concern).
    //
    /// # Arguments
    //
    /// * `y` - The big number to multiply with `self`
    /// * `z` - The big number to add to the product
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the result of `self * y + z`.
    //
    /// # Examples
    //
    /// ## Basic Multiply-Add
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let x = BigNumber::from_i64(10);
    /// let y = BigNumber::from_i64(5);
    /// let z = BigNumber::from_i64(3);
    /// let result = x.mul_add(&y, &z);
    /// assert_eq!(result.to_i64(), Some(53)); // 10 * 5 + 3 = 53
    /// ```
    //
    /// ## Linear Function Evaluation
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// // Evaluate y = mx + b where m=2, x=10, b=5
    /// let m = BigNumber::from_i64(2);
    /// let x = BigNumber::from_i64(10);
    /// let b = BigNumber::from_i64(5);
    /// let y = x.mul_add(&m, &b);
    /// assert_eq!(y.to_i64(), Some(25)); // 10 * 2 + 5 = 25
    /// ```
    //
    /// ## Large Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let x = BigNumber::from_u64(u64::MAX);
    /// let y = BigNumber::from_u64(2);
    /// let z = BigNumber::from_u64(1);
    /// let result = x.mul_add(&y, &z);
    /// // Result exceeds u64::MAX, so cannot convert back
    /// assert!(result.to_i64().is_none());
    /// ```
    pub fn mul_add(&self, y: &Self, z: &Self) -> Self {
        Self {
            value: &self.value * &y.value + &z.value,
        }
    }

    /// Add a small unsigned integer to a big number: computes `self + y`.
    ///
    /// This function performs addition of a big number with a standard `u32`
    /// value. It's optimized for the common case where a small constant is
    /// being added to a big number, avoiding the need to create a `BigNumber`
    /// from the small value first.
    //
    /// # Purpose
    //
    /// This function provides an optimized path for adding small constants to
    /// big numbers, which is a common operation in loops, counters, and
    /// incremental calculations. By accepting a `u32` directly, it avoids
    /// the overhead of creating a temporary `BigNumber` for the small value.
    //
    /// # Arguments
    //
    /// * `y` - The 32-bit unsigned integer to add to `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the sum of `self` and `y`.
    //
    /// # Examples
    //
    /// ## Basic Addition
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let x = BigNumber::from_i64(100);
    /// let result = x.plus_small(50);
    /// assert_eq!(result.to_i64(), Some(150));
    /// ```
    //
    /// ## Incrementing Large Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let large = BigNumber::from_u64(u64::MAX);
    /// let incremented = large.plus_small(1);
    /// // Result exceeds u64::MAX
    /// assert!(incremented.to_i64().is_none());
    /// ```
    //
    /// ## Counter Pattern
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let mut counter = BigNumber::from_i64(0);
    /// for i in 0..10 {
    ///     counter = counter.plus_small(1);
    /// }
    /// assert_eq!(counter.to_i64(), Some(10));
    /// ```
    pub fn plus_small(&self, y: u32) -> Self {
        Self {
            value: &self.value + Integer::from(y),
        }
    }

    /// Perform bitwise AND operation: computes `self & other`.
    ///
    /// This function performs a bitwise AND operation between two big numbers,
    /// returning a new `BigNumber` where each bit is set to 1 only if the
    /// corresponding bits in both operands are 1. The operation uses two's
    /// complement representation, matching the C implementation exactly.
    //
    /// # Purpose
    //
    /// Bitwise AND is essential for masking operations, flag checking, and
    /// low-level bit manipulation. This function enables bitwise operations
    /// on numbers of any size, maintaining compatibility with Erlang's bitwise
    /// operators.
    //
    /// # Arguments
    //
    /// * `other` - The big number to perform AND with `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the bitwise AND of `self` and `other`.
    //
    /// # Examples
    //
    /// ## Basic Bitwise AND
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(0b1010); // 10 in binary
    /// let b = BigNumber::from_i64(0b1100); // 12 in binary
    /// let result = a.bitand(&b);
    /// assert_eq!(result.to_i64(), Some(0b1000)); // 8 in binary
    /// ```
    //
    /// ## Masking Bits
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// // Extract lower 8 bits
    /// let value = BigNumber::from_i64(0x1234);
    /// let mask = BigNumber::from_i64(0xFF);
    /// let lower_bits = value.bitand(&mask);
    /// assert_eq!(lower_bits.to_i64(), Some(0x34));
    /// ```
    //
    /// ## Checking Flags
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let flags = BigNumber::from_i64(0b1010);
    /// let flag_mask = BigNumber::from_i64(0b0010);
    /// let has_flag = flags.bitand(&flag_mask);
    /// assert_eq!(has_flag.to_i64(), Some(0b0010)); // Flag is set
    /// ```
    pub fn bitand(&self, other: &Self) -> Self {
        Self {
            value: &self.value & &other.value,
        }
    }

    /// Perform bitwise OR operation: computes `self | other`.
    //
    /// This function performs a bitwise OR operation between two big numbers,
    /// returning a new `BigNumber` where each bit is set to 1 if the
    /// corresponding bit in either operand is 1. The operation uses two's
    /// complement representation, matching the C implementation exactly.
    //
    /// # Purpose
    //
    /// Bitwise OR is essential for setting flags, combining bit patterns, and
    /// low-level bit manipulation. This function enables bitwise operations
    /// on numbers of any size, maintaining compatibility with Erlang's bitwise
    /// operators.
    //
    /// # Arguments
    //
    /// * `other` - The big number to perform OR with `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the bitwise OR of `self` and `other`.
    //
    /// # Examples
    //
    /// ## Basic Bitwise OR
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(0b1010); // 10 in binary
    /// let b = BigNumber::from_i64(0b1100); // 12 in binary
    /// let result = a.bitor(&b);
    /// assert_eq!(result.to_i64(), Some(0b1110)); // 14 in binary
    /// ```
    //
    /// ## Setting Flags
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let flags = BigNumber::from_i64(0b1000);
    /// let new_flag = BigNumber::from_i64(0b0010);
    /// let combined = flags.bitor(&new_flag);
    /// assert_eq!(combined.to_i64(), Some(0b1010)); // Both flags set
    /// ```
    //
    /// ## Combining Bit Patterns
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let high = BigNumber::from_i64(0x1200);
    /// let low = BigNumber::from_i64(0x0034);
    /// let combined = high.bitor(&low);
    /// assert_eq!(combined.to_i64(), Some(0x1234));
    /// ```
    pub fn bitor(&self, other: &Self) -> Self {
        Self {
            value: &self.value | &other.value,
        }
    }

    /// Perform bitwise XOR (exclusive OR) operation: computes `self ^ other`.
    //
    /// This function performs a bitwise XOR operation between two big numbers,
    /// returning a new `BigNumber` where each bit is set to 1 if the
    /// corresponding bits in the operands differ. The operation uses two's
    /// complement representation, matching the C implementation exactly.
    //
    /// # Purpose
    //
    /// Bitwise XOR is essential for toggling bits, implementing simple
    /// encryption, and detecting differences between bit patterns. This
    /// function enables bitwise operations on numbers of any size, maintaining
    /// compatibility with Erlang's bitwise operators.
    //
    /// # Arguments
    //
    /// * `other` - The big number to perform XOR with `self`
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the bitwise XOR of `self` and `other`.
    //
    /// # Examples
    //
    /// ## Basic Bitwise XOR
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(0b1010); // 10 in binary
    /// let b = BigNumber::from_i64(0b1100); // 12 in binary
    /// let result = a.bitxor(&b);
    /// assert_eq!(result.to_i64(), Some(0b0110)); // 6 in binary
    /// ```
    //
    /// ## Toggling Bits
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let value = BigNumber::from_i64(0b1010);
    /// let toggle = BigNumber::from_i64(0b0011);
    /// let toggled = value.bitxor(&toggle);
    /// assert_eq!(toggled.to_i64(), Some(0b1001)); // Bits 0 and 1 toggled
    /// ```
    //
    /// ## XOR Properties
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(0b1010);
    /// let b = BigNumber::from_i64(0b1100);
    /// let xor1 = a.bitxor(&b);
    /// let xor2 = b.bitxor(&a);
    /// assert_eq!(xor1, xor2); // XOR is commutative
    //
    /// // XOR with self gives zero
    /// let zero = a.bitxor(&a);
    /// assert_eq!(zero.to_i64(), Some(0));
    /// ```
    pub fn bitxor(&self, other: &Self) -> Self {
        Self {
            value: &self.value ^ &other.value,
        }
    }

    /// Perform bitwise NOT operation: computes `!self`.
    //
    /// This function performs a bitwise NOT (one's complement) operation on a
    /// big number, inverting all bits. The operation uses two's complement
    /// representation, matching Erlang's `bnot` operator semantics exactly.
    //
    /// # Purpose
    //
    /// Bitwise NOT is essential for inverting bit patterns, implementing
    /// bitwise negation, and low-level bit manipulation. In Erlang, the behavior
    /// follows two's complement semantics: `bnot -X == (X - 1)` and
    /// `bnot +X == -(X + 1)`. This function maintains that exact behavior.
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the bitwise NOT of `self`.
    //
    /// # Examples
    //
    /// ## Basic Bitwise NOT
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(10);
    /// let not_num = num.bitnot();
    /// // In two's complement: !10 = -11
    /// assert_eq!(not_num.to_i64(), Some(-11));
    /// ```
    //
    /// ## NOT of Zero
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let zero = BigNumber::from_i64(0);
    /// let not_zero = zero.bitnot();
    /// // In two's complement: !0 = -1
    /// assert_eq!(not_zero.to_i64(), Some(-1));
    /// ```
    //
    /// ## NOT of Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let neg = BigNumber::from_i64(-5);
    /// let not_neg = neg.bitnot();
    /// // In two's complement: !(-5) = 4
    /// assert_eq!(not_neg.to_i64(), Some(4));
    /// ```
    pub fn bitnot(&self) -> Self {
        Self {
            value: !&self.value,
        }
    }

    /// Perform bitwise shift operation: computes `self << shift` or `self >> -shift`.
    //
    /// This function performs a bitwise shift operation on a big number. If `shift`
    /// is positive, it performs a left shift (multiply by 2^shift). If `shift`
    /// is negative, it performs a right shift (divide by 2^|shift|). The operation
    /// uses arithmetic right shift for signed numbers, matching Erlang's shift
    /// operator semantics.
    //
    /// # Purpose
    //
    /// Bitwise shifts are essential for efficient multiplication and division by
    /// powers of two, bit manipulation, and low-level operations. This function
    /// enables shift operations on numbers of any size, maintaining compatibility
    /// with Erlang's shift operators.
    //
    /// # Arguments
    //
    /// * `shift` - The number of bits to shift:
    ///   - Positive values: left shift (multiply by 2^shift)
    ///   - Negative values: right shift (divide by 2^|shift|)
    //
    /// # Returns
    //
    /// A new `BigNumber` containing the shifted value.
    //
    /// # Examples
    //
    /// ## Left Shift
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(10);
    /// let shifted = num.lshift(2); // 10 << 2 = 40
    /// assert_eq!(shifted.to_i64(), Some(40));
    /// ```
    //
    /// ## Right Shift
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(40);
    /// let shifted = num.lshift(-2); // 40 >> 2 = 10
    /// assert_eq!(shifted.to_i64(), Some(10));
    /// ```
    //
    /// ## Large Shifts
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(1);
    /// let shifted = num.lshift(32); // 1 << 32
    /// assert_eq!(shifted.to_i64(), Some(4294967296));
    /// ```
    pub fn lshift(&self, shift: i32) -> Self {
        if shift >= 0 {
            Self {
                value: &self.value << shift as u64,
            }
        } else {
            Self {
                value: &self.value >> (-shift) as u64,
            }
        }
    }

    /// Compare two big numbers using signed comparison.
    //
    /// This function performs a signed comparison between two big numbers,
    /// taking into account the sign of the numbers. Negative numbers are
    /// considered less than positive numbers, and zero is considered equal
    /// to zero.
    //
    /// # Purpose
    //
    /// Signed comparison is essential for ordering numbers, implementing
    /// sorting algorithms, and conditional logic based on numeric values.
    /// This function enables comparison of numbers of any size, maintaining
    /// mathematical correctness for values that exceed standard integer types.
    //
    /// # Arguments
    //
    /// * `other` - The big number to compare with `self`
    //
    /// # Returns
    //
    /// * `-1` if `self < other`
    /// * `0` if `self == other`
    /// * `1` if `self > other`
    //
    /// # Examples
    //
    /// ## Basic Comparison
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(50);
    /// assert_eq!(a.comp(&b), 1);  // 100 > 50
    /// assert_eq!(b.comp(&a), -1); // 50 < 100
    /// ```
    //
    /// ## Equality
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let a = BigNumber::from_i64(100);
    /// let b = BigNumber::from_i64(100);
    /// assert_eq!(a.comp(&b), 0); // 100 == 100
    /// ```
    //
    /// ## Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let neg = BigNumber::from_i64(-100);
    /// let pos = BigNumber::from_i64(100);
    /// assert_eq!(neg.comp(&pos), -1); // -100 < 100
    /// assert_eq!(pos.comp(&neg), 1);   // 100 > -100
    /// ```
    pub fn comp(&self, other: &Self) -> i32 {
        match self.value.cmp(&other.value) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    /// Compare two big numbers using unsigned comparison (absolute values).
    //
    /// This function performs an unsigned comparison between two big numbers,
    /// comparing their absolute values and ignoring the sign. This is useful
    /// when you want to compare the magnitude of numbers regardless of whether
    /// they are positive or negative.
    //
    /// # Purpose
    //
    /// Unsigned comparison is useful when you need to compare the magnitude
    /// or absolute value of numbers, such as when finding the maximum absolute
    /// value or when implementing algorithms that treat positive and negative
    /// numbers of the same magnitude as equivalent.
    //
    /// # Arguments
    //
    /// * `other` - The big number to compare with `self`
    //
    /// # Returns
    //
    /// * `-1` if `|self| < |other|`
    /// * `0` if `|self| == |other|`
    /// * `1` if `|self| > |other|`
    //
    /// # Examples
    //
    /// ## Same Magnitude, Different Signs
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let neg = BigNumber::from_i64(-100);
    /// let pos = BigNumber::from_i64(100);
    /// assert_eq!(neg.ucomp(&pos), 0); // |-100| == |100|
    /// ```
    //
    /// ## Different Magnitudes
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let small = BigNumber::from_i64(-50);
    /// let large = BigNumber::from_i64(100);
    /// assert_eq!(small.ucomp(&large), -1); // |-50| < |100|
    /// assert_eq!(large.ucomp(&small), 1);  // |100| > |-50|
    /// ```
    //
    /// ## Negative vs Negative
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let small_neg = BigNumber::from_i64(-50);
    /// let large_neg = BigNumber::from_i64(-100);
    /// assert_eq!(small_neg.ucomp(&large_neg), -1); // |-50| < |-100|
    /// ```
    pub fn ucomp(&self, other: &Self) -> i32 {
        // Get absolute values by comparing with zero and negating if needed
        let abs_self = if self.value < 0 {
            -self.value.clone()
        } else {
            self.value.clone()
        };
        let abs_other = if other.value < 0 {
            -other.value.clone()
        } else {
            other.value.clone()
        };
        match abs_self.cmp(&abs_other) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    /// Convert a big number to a string representation in the given base.
    //
    /// This function converts a `BigNumber` to a string representation using
    /// the specified base (radix). The base must be between 2 and 36, where
    /// bases 2-10 use digits 0-9, and bases 11-36 use digits 0-9 and letters
    /// a-z. For negative numbers, a minus sign is prepended.
    //
    /// # Purpose
    //
    /// String conversion is essential for displaying big numbers to users,
    /// serialization, and debugging. This function supports common bases
    /// (binary, octal, decimal, hexadecimal) as well as arbitrary bases up
    /// to 36, matching Erlang's integer-to-string conversion capabilities.
    //
    /// # Arguments
    //
    /// * `base` - The base (radix) for the string representation (2-36)
    //
    /// # Returns
    //
    /// A `String` containing the number in the specified base.
    //
    /// # Panics
    //
    /// Panics if `base` is less than 2 or greater than 36.
    //
    /// # Examples
    //
    /// ## Common Bases
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(255);
    /// assert_eq!(num.to_string_base(10), "255");  // Decimal
    /// assert_eq!(num.to_string_base(16), "ff");  // Hexadecimal
    /// assert_eq!(num.to_string_base(2), "11111111"); // Binary
    /// assert_eq!(num.to_string_base(8), "377");  // Octal
    /// ```
    //
    /// ## Arbitrary Bases
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(100);
    /// assert_eq!(num.to_string_base(3), "10201");  // Base 3
    /// assert_eq!(num.to_string_base(5), "400");    // Base 5
    /// assert_eq!(num.to_string_base(12), "84");     // Base 12
    /// ```
    //
    /// ## Negative Numbers
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let neg = BigNumber::from_i64(-100);
    /// assert_eq!(neg.to_string_base(10), "-100");
    /// assert_eq!(neg.to_string_base(16), "-64");
    /// ```
    pub fn to_string_base(&self, base: u32) -> String {
        if base < 2 || base > 36 {
            panic!("Base must be between 2 and 36");
        }
        
        // Use optimized formatting for common bases
        match base {
            10 => self.value.to_string(),
            16 => format!("{:x}", self.value),
            2 => format!("{:b}", self.value),
            8 => format!("{:o}", self.value),
            _ => {
                // Manual conversion for arbitrary bases (2-36)
                // Algorithm: repeatedly divide by base, collecting remainders
                // This matches the C implementation's write_big function
                let mut result = String::new();
                let mut n = self.value.clone();
                let zero = Integer::from(0);
                let base_int = Integer::from(base);
                
                // Handle negative numbers (sign comes first in output)
                if n < zero {
                    result.push('-');
                    n = -n;
                }
                
                // Handle zero
                if n == zero {
                    return "0".to_string();
                }
                
                // Convert by repeatedly dividing by base
                // Collect digits (least significant first, we'll reverse later)
                let mut digits = Vec::new();
                while n > zero {
                    let remainder = &n % &base_int;
                    // Convert remainder to u32
                    // Since base <= 36, remainder will always be < base, so safe to convert
                    let digit = if remainder == zero {
                        0
                    } else {
                        // Convert small remainder to u32 via string (safe for base <= 36)
                        remainder.to_string().parse::<u32>().unwrap_or(0)
                    };
                    digits.push(digit);
                    n = &n / &base_int;
                }
                
                // Build string from most significant to least significant (reverse digits)
                for &digit in digits.iter().rev() {
                    if digit < 10 {
                        result.push((b'0' + digit as u8) as char);
                    } else {
                        // For bases > 10, use lowercase letters a-z
                        result.push((b'a' + (digit - 10) as u8) as char);
                    }
                }
                result
            }
        }
    }

    /// Get a reference to the internal `Integer` value.
    //
    /// This function provides access to the underlying `malachite::Integer`
    /// value for advanced use cases where direct manipulation of the `Integer`
    /// type is needed. This is useful for operations not exposed by the
    /// `BigNumber` API or for integration with other code that uses `malachite`.
    //
    /// # Purpose
    //
    /// This function enables advanced users to access the full power of the
    /// `malachite` crate when needed, while still providing a simplified API
    /// through `BigNumber` for common operations.
    //
    /// # Returns
    //
    /// A reference to the internal `Integer` value.
    //
    /// # Examples
    //
    /// ## Accessing Internal Value
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(12345);
    /// let integer_ref = num.as_integer();
    /// assert_eq!(integer_ref.to_string(), "12345");
    /// ```
    //
    /// ## Advanced Operations
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    /// use malachite::Integer;
    //
    /// let num = BigNumber::from_i64(100);
    /// let integer = num.as_integer();
    /// // Perform advanced malachite operations if needed
    /// let doubled = integer * Integer::from(2);
    /// ```
    //
    /// ## Creating New BigNumber
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(12345);
    /// let integer_ref = num.as_integer();
    /// let new_num = BigNumber::from_integer(integer_ref.clone());
    /// assert_eq!(num, new_num);
    /// ```
    pub fn as_integer(&self) -> &Integer {
        &self.value
    }

    /// Create a `BigNumber` from a `malachite::Integer` value.
    //
    /// This function provides a way to create a `BigNumber` directly from a
    /// `malachite::Integer`, which is useful for advanced use cases where you
    /// need to work with `malachite` types directly and then convert to `BigNumber`.
    //
    /// # Purpose
    //
    /// This function enables integration with code that uses `malachite::Integer`
    /// directly, allowing seamless conversion between `malachite` types and
    /// `BigNumber` for use in the Erlang runtime system.
    //
    /// # Arguments
    //
    /// * `value` - The `malachite::Integer` value to wrap
    //
    /// # Returns
    //
    /// A new `BigNumber` instance containing the provided `Integer` value.
    //
    /// # Examples
    //
    /// ## Creating from Integer
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    /// use malachite::Integer;
    //
    /// let integer = Integer::from(12345);
    /// let num = BigNumber::from_integer(integer);
    /// assert_eq!(num.to_i64(), Some(12345));
    /// ```
    //
    /// ## Advanced Malachite Operations
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    /// use malachite::Integer;
    //
    /// // Perform complex malachite operations
    /// let a = Integer::from(100);
    /// let b = Integer::from(50);
    /// let result = &a * &b + Integer::from(10);
    //
    /// // Convert to BigNumber
    /// let num = BigNumber::from_integer(result);
    /// assert_eq!(num.to_i64(), Some(5010));
    /// ```
    //
    /// ## Round-Trip Conversion
    //
    /// ```rust
    /// use entities_utilities::BigNumber;
    //
    /// let num = BigNumber::from_i64(12345);
    /// let integer = num.as_integer().clone();
    /// let new_num = BigNumber::from_integer(integer);
    /// assert_eq!(num, new_num);
    /// ```
    pub fn from_integer(value: Integer) -> Self {
        Self { value }
    }
}

impl From<i64> for BigNumber {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl From<u64> for BigNumber {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<i32> for BigNumber {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<u32> for BigNumber {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_number_creation() {
        let big = BigNumber::from_i64(12345);
        assert!(big.is_positive());
        assert!(!big.is_zero());

        let big_neg = BigNumber::from_i64(-12345);
        assert!(!big_neg.is_positive());
        
        // Test zero
        let zero = BigNumber::from_i64(0);
        assert!(zero.is_zero());
        assert!(zero.is_positive());
        
        // Test from_u64
        let big_u64 = BigNumber::from_u64(12345);
        assert_eq!(big_u64.to_i64(), Some(12345));
        
        // Test from_u32
        let big_u32 = BigNumber::from_u32(12345);
        assert_eq!(big_u32.to_i64(), Some(12345));
        
        // Test from_i32
        let big_i32 = BigNumber::from_i32(12345);
        assert_eq!(big_i32.to_i64(), Some(12345));
        let big_i32_neg = BigNumber::from_i32(-12345);
        assert_eq!(big_i32_neg.to_i64(), Some(-12345));
        
        // Test From trait implementations
        let from_i64: BigNumber = 12345i64.into();
        assert_eq!(from_i64.to_i64(), Some(12345));
        let from_u64: BigNumber = 12345u64.into();
        assert_eq!(from_u64.to_i64(), Some(12345));
        let from_i32: BigNumber = 12345i32.into();
        assert_eq!(from_i32.to_i64(), Some(12345));
        let from_u32: BigNumber = 12345u32.into();
        assert_eq!(from_u32.to_i64(), Some(12345));
    }

    #[test]
    fn test_arithmetic_operations() {
        let a = BigNumber::from_i64(100);
        let b = BigNumber::from_i64(50);

        assert_eq!(a.plus(&b).to_i64(), Some(150));
        assert_eq!(a.minus(&b).to_i64(), Some(50));
        assert_eq!(a.times(&b).to_i64(), Some(5000));
        assert_eq!(a.div(&b).unwrap().to_i64(), Some(2));
        assert_eq!(a.rem(&b).unwrap().to_i64(), Some(0));
        
        // Test division by zero
        let zero = BigNumber::from_i64(0);
        assert!(a.div(&zero).is_none());
        assert!(a.rem(&zero).is_none());
        
        // Test remainder with non-zero result
        let c = BigNumber::from_i64(17);
        let d = BigNumber::from_i64(5);
        assert_eq!(c.rem(&d).unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_bitwise_operations() {
        let a = BigNumber::from_i64(0b1010); // 10
        let b = BigNumber::from_i64(0b1100); // 12

        assert_eq!(a.bitand(&b).to_i64(), Some(0b1000)); // 8
        assert_eq!(a.bitor(&b).to_i64(), Some(0b1110)); // 14
        assert_eq!(a.bitxor(&b).to_i64(), Some(0b0110)); // 6
        
        // Test bitnot
        let c = BigNumber::from_i64(10);
        let not_c = c.bitnot();
        // bitnot(10) = !10 = -11 in two's complement
        assert_eq!(not_c.to_i64(), Some(-11));
        
        // Test bitnot with zero
        let zero = BigNumber::from_i64(0);
        let not_zero = zero.bitnot();
        assert_eq!(not_zero.to_i64(), Some(-1));
        
        // Test bitnot with negative
        let neg = BigNumber::from_i64(-5);
        let not_neg = neg.bitnot();
        assert_eq!(not_neg.to_i64(), Some(4));
    }

    #[test]
    fn test_shift_operations() {
        let a = BigNumber::from_i64(10);

        assert_eq!(a.lshift(2).to_i64(), Some(40)); // 10 << 2 = 40
        assert_eq!(a.lshift(-1).to_i64(), Some(5)); // 10 >> 1 = 5
        
        // Test larger shifts
        let b = BigNumber::from_i64(1);
        assert_eq!(b.lshift(10).to_i64(), Some(1024)); // 1 << 10 = 1024
        assert_eq!(b.lshift(32).to_i64(), Some(4294967296)); // 1 << 32
        
        // Test right shift
        let c = BigNumber::from_i64(1024);
        assert_eq!(c.lshift(-10).to_i64(), Some(1)); // 1024 >> 10 = 1
        
        // Test negative number shifts
        let d = BigNumber::from_i64(-10);
        assert_eq!(d.lshift(2).to_i64(), Some(-40)); // -10 << 2 = -40
        assert_eq!(d.lshift(-1).to_i64(), Some(-5)); // -10 >> 1 = -5
    }

    #[test]
    fn test_comparison() {
        let a = BigNumber::from_i64(100);
        let b = BigNumber::from_i64(50);
        let c = BigNumber::from_i64(100);

        assert_eq!(a.comp(&b), 1);
        assert_eq!(b.comp(&a), -1);
        assert_eq!(a.comp(&c), 0);
        
        // Test unsigned comparison
        let d = BigNumber::from_i64(-100);
        let e = BigNumber::from_i64(100);
        // ucomp compares absolute values
        assert_eq!(d.ucomp(&e), 0); // |-100| == |100|
        assert_eq!(d.ucomp(&b), 1); // |-100| > |50|
        
        // Test negative comparisons
        let f = BigNumber::from_i64(-50);
        assert_eq!(d.comp(&f), -1); // -100 < -50
        assert_eq!(f.comp(&d), 1); // -50 > -100
    }

    #[test]
    fn test_negative_operations() {
        let a = BigNumber::from_i64(-10);
        let b = BigNumber::from_i64(5);

        assert_eq!(a.plus(&b).to_i64(), Some(-5));
        assert_eq!(a.minus(&b).to_i64(), Some(-15));
        assert_eq!(a.times(&b).to_i64(), Some(-50));
    }

    #[test]
    fn test_mul_add() {
        let x = BigNumber::from_i64(10);
        let y = BigNumber::from_i64(5);
        let z = BigNumber::from_i64(3);

        assert_eq!(x.mul_add(&y, &z).to_i64(), Some(53)); // 10*5 + 3
    }

    #[test]
    fn test_plus_small() {
        let x = BigNumber::from_i64(100);
        assert_eq!(x.plus_small(50).to_i64(), Some(150));
    }

    #[test]
    fn test_conversion() {
        let big = BigNumber::from_i64(12345);
        assert_eq!(big.to_u32(), Some(12345));
        assert_eq!(big.to_i64(), Some(12345));

        let big_neg = BigNumber::from_i64(-12345);
        assert_eq!(big_neg.to_u32(), None); // Negative can't be u32
        assert_eq!(big_neg.to_i64(), Some(-12345));
        
        // Test u32 max
        let u32_max = BigNumber::from_u64(u32::MAX as u64);
        assert_eq!(u32_max.to_u32(), Some(u32::MAX));
        
        // Test u32 overflow
        let too_large = BigNumber::from_u64(u32::MAX as u64 + 1);
        assert_eq!(too_large.to_u32(), None);
        
        // Test f64 conversion
        let f64_test = BigNumber::from_i64(12345);
        assert_eq!(f64_test.to_f64(), Some(12345.0));
        
        // Test from_f64
        let from_f64 = BigNumber::from_f64(12345.5);
        assert_eq!(from_f64.unwrap().to_i64(), Some(12345)); // Truncated
        
        // Test from_f64 with negative
        let from_f64_neg = BigNumber::from_f64(-12345.5);
        assert_eq!(from_f64_neg.unwrap().to_i64(), Some(-12345));
        
        // Test from_f64 with NaN
        assert!(BigNumber::from_f64(f64::NAN).is_none());
        
        // Test from_f64 with infinity
        assert!(BigNumber::from_f64(f64::INFINITY).is_none());
        assert!(BigNumber::from_f64(f64::NEG_INFINITY).is_none());
        
        // Test zero conversion
        let zero = BigNumber::from_i64(0);
        assert_eq!(zero.to_u32(), Some(0));
        assert_eq!(zero.to_i64(), Some(0));
        assert_eq!(zero.to_f64(), Some(0.0));
        
        // Test to_f64 with negative
        let neg_f64 = BigNumber::from_i64(-12345);
        assert_eq!(neg_f64.to_f64(), Some(-12345.0));
        
        // Test to_f64 with very large number (should still work if within f64 range)
        let large = BigNumber::from_f64(1e20);
        assert!(large.is_some());
        let large_f64 = large.unwrap().to_f64();
        assert!(large_f64.is_some());
        // Verify it's approximately correct (within f64 precision)
        assert!((large_f64.unwrap() - 1e20).abs() < 1e10); // Allow some precision loss
    }

    #[test]
    fn test_string_conversion() {
        let big = BigNumber::from_i64(255);
        assert_eq!(big.to_string_base(16), "ff");
        assert_eq!(big.to_string_base(10), "255");
        assert_eq!(big.to_string_base(2), "11111111");
        
        // Test arbitrary bases
        let big2 = BigNumber::from_i64(100);
        assert_eq!(big2.to_string_base(3), "10201"); // 100 in base 3
        assert_eq!(big2.to_string_base(5), "400");   // 100 in base 5
        assert_eq!(big2.to_string_base(12), "84");   // 100 in base 12
        
        // Test base 36 (uses all digits 0-9 and a-z)
        let big3 = BigNumber::from_i64(1295); // 1295 = 36^2 - 1 = zz in base 36
        assert_eq!(big3.to_string_base(36), "zz");
        
        // Test negative numbers
        let big4 = BigNumber::from_i64(-100);
        assert_eq!(big4.to_string_base(10), "-100");
        assert_eq!(big4.to_string_base(16), "-64");
        
        // Test base 8 (octal)
        let big5 = BigNumber::from_i64(64);
        assert_eq!(big5.to_string_base(8), "100");
        
        // Test zero in various bases
        let zero = BigNumber::from_i64(0);
        assert_eq!(zero.to_string_base(2), "0");
        assert_eq!(zero.to_string_base(10), "0");
        assert_eq!(zero.to_string_base(16), "0");
        assert_eq!(zero.to_string_base(36), "0");
        
        // Test all bases from 2 to 36
        let test_val = BigNumber::from_i64(35);
        assert_eq!(test_val.to_string_base(36), "z"); // Last digit in base 36
        
        // Test base 11-35 (various arbitrary bases)
        let test_val2 = BigNumber::from_i64(100);
        assert_eq!(test_val2.to_string_base(11), "91");
        assert_eq!(test_val2.to_string_base(20), "50");
        assert_eq!(test_val2.to_string_base(30), "3a");
    }
    
    #[test]
    #[should_panic(expected = "Base must be between 2 and 36")]
    fn test_string_conversion_invalid_base_too_small() {
        let big = BigNumber::from_i64(100);
        let _ = big.to_string_base(1);
    }
    
    #[test]
    #[should_panic(expected = "Base must be between 2 and 36")]
    fn test_string_conversion_invalid_base_too_large() {
        let big = BigNumber::from_i64(100);
        let _ = big.to_string_base(37);
    }

    #[test]
    fn test_large_numbers() {
        // Test with numbers larger than i64
        let a = BigNumber::from_u64(u64::MAX);
        let b = BigNumber::from_u64(1);
        let sum = a.plus(&b);
        
        // Should handle overflow correctly
        assert!(sum.to_i64().is_none()); // Too large for i64
        
        // Test as_integer and from_integer
        let big = BigNumber::from_i64(12345);
        let integer_ref = big.as_integer();
        assert_eq!(integer_ref.to_string(), "12345");
        
        let new_big = BigNumber::from_integer(integer_ref.clone());
        assert_eq!(new_big.to_i64(), Some(12345));
    }
    
    #[test]
    fn test_edge_cases() {
        // Test very large numbers
        let huge = BigNumber::from_u64(u64::MAX);
        assert!(huge.to_u32().is_none()); // Too large for u32
        
        // Test i64 boundaries
        let i64_max = BigNumber::from_i64(i64::MAX);
        assert_eq!(i64_max.to_i64(), Some(i64::MAX));
        
        let i64_min = BigNumber::from_i64(i64::MIN);
        assert_eq!(i64_min.to_i64(), Some(i64::MIN));
        assert!(i64_min.to_u32().is_none()); // Negative can't be u32
        
        // Test from_f64 edge cases
        let f64_max_int = BigNumber::from_f64(i64::MAX as f64);
        assert!(f64_max_int.is_some());
        
        let f64_min_int = BigNumber::from_f64(i64::MIN as f64);
        assert!(f64_min_int.is_some());
        
        // Test from_f64 with values outside i64 range
        // Now that we implement the C algorithm, we can handle these!
        let too_large = BigNumber::from_f64(1e20);
        assert!(too_large.is_some()); // Should now work with C algorithm
        // Verify it's correct: 1e20 = 100000000000000000000
        // Convert back to string to verify
        let result = too_large.as_ref().unwrap();
        let result_str = result.to_string_base(10);
        assert!(result_str.starts_with("10000000000000000000")); // Should start with 1e20
        assert!(result.is_positive());
        
        // Test even larger value
        let very_large = BigNumber::from_f64(1e30);
        assert!(very_large.is_some());
        assert!(very_large.as_ref().unwrap().is_positive());
        
        // Test Clone, Debug, PartialEq, Eq, Hash
        let a = BigNumber::from_i64(100);
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(format!("{:?}", a), format!("{:?}", b));
        
        // Test Hash (via HashMap)
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(a.clone(), "test");
        assert_eq!(map.get(&b), Some(&"test"));
    }
}
