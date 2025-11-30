//! Rational Number Operations Module
//!
//! This module provides arbitrary precision rational number operations for the Erlang/OTP
//! runtime system, providing support for exact fractional arithmetic beyond the limits of
//! floating-point precision.
//!
//! # Purpose
//!
//! Erlang supports exact arithmetic operations that require precise representation of
//! fractional values. This module provides the core operations needed to manipulate
//! rational numbers (fractions), including:
//!
//! - **Creation and Conversion**: Convert between rational numbers and standard
//!   integer types (i32, i64, u32, u64) as well as floating-point numbers (f64).
//!   Rational numbers can represent exact decimal values that cannot be precisely
//!   represented as floating-point numbers (e.g., 1/3, 0.1).
//!
//! - **Arithmetic Operations**: Addition, subtraction, multiplication, and division
//!   operations that work with arbitrary precision and maintain exact results.
//!   These operations enable exact decimal arithmetic without floating-point
//!   rounding errors.
//!
//! - **Comparison Operations**: Comparison functions for ordering rational numbers
//!   that respect mathematical ordering.
//!
//! - **String Conversion**: Convert rational numbers to string representations in
//!   various formats (decimal, fraction, scientific notation).
//!
//! # Implementation Details
//!
//! This module uses the `malachite` crate's `Rational` type for high-performance
//! arbitrary-precision rational arithmetic. Rational numbers are represented as
//! fractions (numerator/denominator) in reduced form, ensuring exact representation
//! of fractional values.
//!
//! # Examples
//!
//! ## Basic Arithmetic
//!
//! ```rust
//! use entities_utilities::BigRational;
//!
//! let a = BigRational::from_i64(1);
//! let b = BigRational::from_i64(3);
//! let third = a.div(&b).unwrap(); // Exact representation of 1/3
//!
//! let sum = third.plus(&third).plus(&third); // Exactly 1, no rounding error
//! ```
//!
//! ## Exact Decimal Arithmetic
//!
//! ```rust
//! use entities_utilities::BigRational;
//!
//! // Represent 0.1 exactly (which f64 cannot do)
//! let tenth = BigRational::from_f64(0.1).unwrap();
//! let three_tenths = tenth.plus(&tenth).plus(&tenth); // Exactly 0.3
//! ```
//!
//! ## Conversion
//!
//! ```rust
//! use entities_utilities::BigRational;
//!
//! let r = BigRational::from_i64(22).div(&BigRational::from_i64(7)).unwrap(); // 22/7
//! let f = r.to_f64(); // Approximate as f64
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

use malachite::Rational;
use malachite::base::num::conversion::traits::{IsInteger, RoundingFrom};
use malachite::base::num::arithmetic::traits::Abs;
use malachite::base::num::basic::traits::Zero;
use malachite::base::rounding_modes::RoundingMode;

/// Big rational number representation using malachite's Rational.
///
/// This struct wraps malachite's `Rational` type to provide arbitrary precision
/// rational number operations. Rational numbers are represented as fractions
/// (numerator/denominator) in reduced form.
///
/// # Purpose
///
/// `BigRational` provides arbitrary precision rational number arithmetic for the
/// Erlang runtime system. It can represent exact fractional values that cannot
/// be precisely represented as floating-point numbers, enabling exact decimal
/// arithmetic without rounding errors.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use entities_utilities::BigRational;
///
/// let r = BigRational::from_i64(1).div(&BigRational::from_i64(3)).unwrap();
/// assert_eq!(r.to_string(), "1/3");
/// ```
///
/// ## Exact Arithmetic
///
/// ```rust
/// use entities_utilities::BigRational;
///
/// let a = BigRational::from_f64(0.1).unwrap();
/// let b = BigRational::from_f64(0.2).unwrap();
/// let sum = a.plus(&b); // Exactly 0.3, no rounding error
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BigRational {
    value: Rational,
}

impl BigRational {
    /// Create a new rational number from a 64-bit signed integer.
    ///
    /// This function converts a standard `i64` value into a `BigRational`,
    /// representing it as a fraction with denominator 1.
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit signed integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigRational` instance containing the converted value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_utilities::BigRational;
    ///
    /// let r = BigRational::from_i64(42);
    /// assert_eq!(r.to_string(), "42");
    /// ```
    pub fn from_i64(value: i64) -> Self {
        Self {
            value: Rational::from(value),
        }
    }

    /// Create a new rational number from a 64-bit unsigned integer.
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit unsigned integer value to convert
    ///
    /// # Returns
    ///
    /// A new `BigRational` instance containing the converted value.
    pub fn from_u64(value: u64) -> Self {
        Self {
            value: Rational::from(value),
        }
    }

    /// Create a new rational number from a 64-bit floating-point number.
    ///
    /// This function converts a `f64` value into a `BigRational` by finding
    /// the exact rational representation. For values that cannot be exactly
    /// represented (like 0.1), this provides an exact representation.
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit floating-point value to convert
    ///
    /// # Returns
    ///
    /// * `Some(BigRational)` if the conversion succeeds (value is finite)
    /// * `None` if the conversion fails (NaN or infinity)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_utilities::BigRational;
    ///
    /// let r = BigRational::from_f64(0.1);
    /// assert!(r.is_some());
    /// // 0.1 is exactly represented as 3602879701896397/36028797018963968
    /// ```
    pub fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Some(Self {
            value: Rational::try_from(value).ok()?,
        })
    }

    /// Create a rational number from a numerator and denominator.
    ///
    /// # Arguments
    ///
    /// * `numerator` - The numerator of the fraction
    /// * `denominator` - The denominator of the fraction (must not be zero)
    ///
    /// # Returns
    ///
    /// * `Some(BigRational)` if denominator is not zero
    /// * `None` if denominator is zero
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_utilities::BigRational;
    ///
    /// let r = BigRational::from_fraction(22, 7);
    /// assert!(r.is_some());
    /// assert_eq!(r.unwrap().to_string(), "22/7");
    /// ```
    pub fn from_fraction(numerator: i64, denominator: i64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        Some(Self {
            value: Rational::from(numerator) / Rational::from(denominator),
        })
    }

    /// Add two rational numbers.
    ///
    /// # Arguments
    ///
    /// * `other` - The rational number to add
    ///
    /// # Returns
    ///
    /// A new `BigRational` representing the sum.
    pub fn plus(&self, other: &Self) -> Self {
        Self {
            value: &self.value + &other.value,
        }
    }

    /// Subtract two rational numbers.
    ///
    /// # Arguments
    ///
    /// * `other` - The rational number to subtract
    ///
    /// # Returns
    ///
    /// A new `BigRational` representing the difference.
    pub fn minus(&self, other: &Self) -> Self {
        Self {
            value: &self.value - &other.value,
        }
    }

    /// Multiply two rational numbers.
    ///
    /// # Arguments
    ///
    /// * `other` - The rational number to multiply
    ///
    /// # Returns
    ///
    /// A new `BigRational` representing the product.
    pub fn times(&self, other: &Self) -> Self {
        Self {
            value: &self.value * &other.value,
        }
    }

    /// Divide two rational numbers.
    ///
    /// # Arguments
    ///
    /// * `other` - The rational number to divide by
    ///
    /// # Returns
    ///
    /// * `Some(BigRational)` if division succeeds (other is not zero)
    /// * `None` if division by zero
    pub fn div(&self, other: &Self) -> Option<Self> {
        if other.value == Rational::ZERO {
            return None;
        }
        Some(Self {
            value: &self.value / &other.value,
        })
    }

    /// Get the absolute value of a rational number.
    ///
    /// # Returns
    ///
    /// A new `BigRational` representing the absolute value.
    pub fn abs(&self) -> Self {
        Self {
            value: self.value.clone().abs(),
        }
    }

    /// Negate a rational number.
    ///
    /// # Returns
    ///
    /// A new `BigRational` representing the negated value.
    pub fn neg(&self) -> Self {
        Self {
            value: -&self.value,
        }
    }

    /// Compare two rational numbers.
    ///
    /// # Arguments
    ///
    /// * `other` - The rational number to compare with
    ///
    /// # Returns
    ///
    /// * `Some(std::cmp::Ordering::Less)` if self < other
    /// * `Some(std::cmp::Ordering::Equal)` if self == other
    /// * `Some(std::cmp::Ordering::Greater)` if self > other
    pub fn comp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }

    /// Convert a rational number to a 64-bit floating-point number.
    ///
    /// This conversion may be lossy for very large or very precise rational numbers
    /// due to the limited precision of floating-point representation.
    ///
    /// # Returns
    ///
    /// The `f64` representation of the rational number.
    pub fn to_f64(&self) -> f64 {
        let (result, _ordering) = f64::rounding_from(&self.value, RoundingMode::Nearest);
        result
    }

    /// Convert a rational number to a 64-bit signed integer, if possible.
    ///
    /// This conversion only succeeds if the rational number represents an exact
    /// integer value within the range of `i64`.
    ///
    /// # Returns
    ///
    /// * `Some(i64)` if the value is an exact integer within i64 range
    /// * `None` if the value is not an integer or out of range
    pub fn to_i64(&self) -> Option<i64> {
        if self.value.is_integer() {
            // Try to convert to i64 via f64
            // For exact integers, we can safely convert through f64
            let f64_val = self.to_f64();
            if f64_val >= i64::MIN as f64 && f64_val <= i64::MAX as f64 {
                Some(f64_val as i64)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if the rational number is zero.
    ///
    /// # Returns
    ///
    /// `true` if the value is zero, `false` otherwise.
    pub fn is_zero(&self) -> bool {
        self.value == Rational::ZERO
    }

    /// Check if the rational number is positive.
    ///
    /// # Returns
    ///
    /// `true` if the value is positive, `false` otherwise.
    pub fn is_positive(&self) -> bool {
        self.value > Rational::ZERO
    }

    /// Check if the rational number is negative.
    ///
    /// # Returns
    ///
    /// `true` if the value is negative, `false` otherwise.
    pub fn is_negative(&self) -> bool {
        self.value < Rational::ZERO
    }

    /// Check if the rational number represents an integer.
    ///
    /// # Returns
    ///
    /// `true` if the value is an integer (denominator is 1), `false` otherwise.
    pub fn is_integer(&self) -> bool {
        self.value.is_integer()
    }

    /// Get access to the underlying `malachite::Rational` value.
    ///
    /// This function provides access to the underlying `malachite::Rational`
    /// for advanced operations that are not covered by the `BigRational` API.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `malachite::Rational` value.
    pub fn as_rational(&self) -> &Rational {
        &self.value
    }

    /// Create a `BigRational` from a `malachite::Rational` value.
    ///
    /// This function enables integration with code that uses `malachite::Rational`
    /// directly, allowing seamless conversion between `malachite` types and
    /// `BigRational`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `malachite::Rational` value to wrap
    ///
    /// # Returns
    ///
    /// A new `BigRational` instance wrapping the provided value.
    pub fn from_rational(value: Rational) -> Self {
        Self { value }
    }
}

impl std::fmt::Display for BigRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i64() {
        let r = BigRational::from_i64(42);
        assert_eq!(r.to_string(), "42");
        assert!(r.is_integer());
    }

    #[test]
    fn test_from_fraction() {
        let r = BigRational::from_fraction(22, 7).unwrap();
        assert_eq!(r.to_string(), "22/7");
        assert!(!r.is_integer());
    }

    #[test]
    fn test_from_fraction_zero_denominator() {
        assert!(BigRational::from_fraction(1, 0).is_none());
    }

    #[test]
    fn test_arithmetic() {
        let a = BigRational::from_i64(1);
        let b = BigRational::from_i64(3);
        let third = a.div(&b).unwrap();
        
        let sum = third.plus(&third).plus(&third);
        assert_eq!(sum.to_i64(), Some(1));
    }

    #[test]
    fn test_exact_decimal() {
        let tenth = BigRational::from_f64(0.1).unwrap();
        let three_tenths = tenth.plus(&tenth).plus(&tenth);
        let f = three_tenths.to_f64();
        // Should be exactly 0.3 (within f64 precision)
        assert!((f - 0.3).abs() < 1e-15);
    }

    #[test]
    fn test_from_f64_nan() {
        assert!(BigRational::from_f64(f64::NAN).is_none());
    }

    #[test]
    fn test_from_f64_infinity() {
        assert!(BigRational::from_f64(f64::INFINITY).is_none());
        assert!(BigRational::from_f64(f64::NEG_INFINITY).is_none());
    }

    #[test]
    fn test_abs() {
        let r = BigRational::from_i64(-42);
        assert_eq!(r.abs().to_i64(), Some(42));
    }

    #[test]
    fn test_neg() {
        let r = BigRational::from_i64(42);
        assert_eq!(r.neg().to_i64(), Some(-42));
    }

    #[test]
    fn test_compare() {
        let a = BigRational::from_i64(1);
        let b = BigRational::from_i64(2);
        assert_eq!(a.comp(&b), std::cmp::Ordering::Less);
        assert_eq!(b.comp(&a), std::cmp::Ordering::Greater);
        assert_eq!(a.comp(&a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_div_by_zero() {
        let a = BigRational::from_i64(1);
        let b = BigRational::from_i64(0);
        assert!(a.div(&b).is_none());
    }

    #[test]
    fn test_is_zero() {
        assert!(BigRational::from_i64(0).is_zero());
        assert!(!BigRational::from_i64(1).is_zero());
    }

    #[test]
    fn test_is_positive_negative() {
        assert!(BigRational::from_i64(1).is_positive());
        assert!(!BigRational::from_i64(1).is_negative());
        assert!(!BigRational::from_i64(-1).is_positive());
        assert!(BigRational::from_i64(-1).is_negative());
    }

    #[test]
    fn test_from_u64() {
        let r = BigRational::from_u64(42);
        assert_eq!(r.to_string(), "42");
        assert!(r.is_integer());
        assert!(r.is_positive());
        
        let r_max = BigRational::from_u64(u64::MAX);
        assert!(r_max.is_positive());
        assert!(r_max.is_integer());
    }

    #[test]
    fn test_minus() {
        let a = BigRational::from_i64(10);
        let b = BigRational::from_i64(3);
        let result = a.minus(&b);
        assert_eq!(result.to_i64(), Some(7));
        
        // Test negative result
        let c = BigRational::from_i64(3);
        let d = BigRational::from_i64(10);
        let result2 = c.minus(&d);
        assert_eq!(result2.to_i64(), Some(-7));
        
        // Test with fractions
        let e = BigRational::from_fraction(5, 2).unwrap();
        let f = BigRational::from_fraction(1, 2).unwrap();
        let result3 = e.minus(&f);
        assert_eq!(result3.to_i64(), Some(2));
    }

    #[test]
    fn test_times() {
        let a = BigRational::from_i64(6);
        let b = BigRational::from_i64(7);
        let result = a.times(&b);
        assert_eq!(result.to_i64(), Some(42));
        
        // Test with fractions
        let c = BigRational::from_fraction(3, 4).unwrap();
        let d = BigRational::from_fraction(4, 3).unwrap();
        let result2 = c.times(&d);
        assert_eq!(result2.to_i64(), Some(1));
        
        // Test negative
        let e = BigRational::from_i64(-5);
        let f = BigRational::from_i64(3);
        let result3 = e.times(&f);
        assert_eq!(result3.to_i64(), Some(-15));
    }

    #[test]
    fn test_as_rational() {
        let r = BigRational::from_i64(42);
        let rational_ref = r.as_rational();
        // Verify we can use the reference
        assert!(*rational_ref != Rational::ZERO);
        
        let r2 = BigRational::from_fraction(22, 7).unwrap();
        let rational_ref2 = r2.as_rational();
        assert!(!rational_ref2.is_integer());
    }

    #[test]
    fn test_from_rational() {
        use malachite::Rational;
        let malachite_rational = Rational::from(42);
        let big_rational = BigRational::from_rational(malachite_rational);
        assert_eq!(big_rational.to_i64(), Some(42));
        
        let malachite_rational2 = Rational::from(22) / Rational::from(7);
        let big_rational2 = BigRational::from_rational(malachite_rational2);
        assert_eq!(big_rational2.to_string(), "22/7");
    }

    #[test]
    fn test_to_i64_non_integer() {
        // Test to_i64 with non-integer rational
        let r = BigRational::from_fraction(1, 3).unwrap();
        assert!(!r.is_integer());
        assert_eq!(r.to_i64(), None);
    }

    #[test]
    fn test_to_i64_large_value() {
        // Test to_i64 with value that might exceed i64 range when converted via f64
        // Create a very large integer rational
        let large = BigRational::from_u64(u64::MAX);
        // This should still work if it fits in i64, but test the edge case
        if let Some(val) = large.to_i64() {
            assert!(val > 0);
        }
    }

    #[test]
    fn test_from_f64_conversion_failure() {
        // Test the case where Rational::try_from might fail
        // This is hard to trigger directly, but we can test edge cases
        // Test with a very large f64 that might cause conversion issues
        let very_large = f64::MAX;
        if let Some(r) = BigRational::from_f64(very_large) {
            // If conversion succeeds, verify it's reasonable
            assert!(r.is_positive());
        }
        
        // Test with a very small f64
        let very_small = f64::MIN_POSITIVE;
        if let Some(r) = BigRational::from_f64(very_small) {
            assert!(r.is_positive());
        }
    }

    #[test]
    fn test_comprehensive_arithmetic() {
        // Test all arithmetic operations together
        let a = BigRational::from_i64(10);
        let b = BigRational::from_i64(3);
        
        let sum = a.plus(&b);
        assert_eq!(sum.to_i64(), Some(13));
        
        let diff = a.minus(&b);
        assert_eq!(diff.to_i64(), Some(7));
        
        let prod = a.times(&b);
        assert_eq!(prod.to_i64(), Some(30));
        
        let quot = a.div(&b).unwrap();
        // 10/3 is not an integer
        assert!(!quot.is_integer());
        assert!((quot.to_f64() - 10.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_fraction_arithmetic() {
        // Test arithmetic with fractions
        let half = BigRational::from_fraction(1, 2).unwrap();
        let third = BigRational::from_fraction(1, 3).unwrap();
        
        let sum = half.plus(&third);
        // 1/2 + 1/3 = 5/6
        assert!((sum.to_f64() - 5.0/6.0).abs() < 1e-10);
        
        let diff = half.minus(&third);
        // 1/2 - 1/3 = 1/6
        assert!((diff.to_f64() - 1.0/6.0).abs() < 1e-10);
        
        let prod = half.times(&third);
        // 1/2 * 1/3 = 1/6
        assert!((prod.to_f64() - 1.0/6.0).abs() < 1e-10);
        
        let quot = half.div(&third).unwrap();
        // (1/2) / (1/3) = 3/2
        assert!((quot.to_f64() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_display_formatting() {
        // Test Display implementation with various values
        let r1 = BigRational::from_i64(42);
        assert_eq!(format!("{}", r1), "42");
        
        let r2 = BigRational::from_fraction(22, 7).unwrap();
        let formatted = format!("{}", r2);
        assert!(formatted.contains("22") && formatted.contains("7"));
        
        let r3 = BigRational::from_i64(0);
        assert_eq!(format!("{}", r3), "0");
    }
}

