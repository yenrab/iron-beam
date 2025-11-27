//! Big Number Operations
//!
//! Provides arbitrary precision integer operations.
//! Based on big.c
//!
//! This module uses the `malachite` crate for high-performance
//! arbitrary-precision arithmetic, which provides behavior compatible
//! with the C implementation's two's complement semantics for operations.

use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

/// Big number representation using malachite's Integer
///
/// This wraps malachite's Integer type to provide the same API
/// as the C big.c implementation. Malachite uses two's complement
/// representation internally, which matches the C code's bitwise
/// operation semantics.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BigNumber {
    value: Integer,
}

impl BigNumber {
    /// Create a new big number from i64
    pub fn from_i64(value: i64) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from u64
    pub fn from_u64(value: u64) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from u32
    pub fn from_u32(value: u32) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from i32
    pub fn from_i32(value: i32) -> Self {
        Self {
            value: Integer::from(value),
        }
    }

    /// Create a new big number from double (f64)
    ///
    /// Returns None if the conversion fails (NaN, infinity, or out of range)
    ///
    /// This implements the C `double_to_big` algorithm, which handles any finite f64 value
    /// by using digit-based conversion. The algorithm:
    /// 1. Extracts sign and makes the value positive
    /// 2. Scales down by digit base (2^32) to count digits needed
    /// 3. Extracts digits iteratively by multiplying, truncating, and subtracting
    /// 4. Builds the Integer from the extracted digits
    ///
    /// **TODO: When malachite ships a version with native `from_f64` support (e.g., via `TryFrom<f64>`**
    /// **or a dedicated `Integer::from_f64()` method), replace this implementation with a call to**
    /// **that function for better maintainability and potential performance improvements.**
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

        // Slow path: Use C algorithm for values outside i64 range
        // This matches the C `double_to_big` implementation exactly
        
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

    /// Convert to f64
    ///
    /// Returns None if the value is too large to represent as f64 (overflow/infinity)
    ///
    /// This uses malachite's `RoundingFrom` trait to convert the Integer to f64.
    /// The conversion uses `Exact` rounding mode, which ensures precision when possible.
    /// If the value is too large to fit in f64, the result will be infinity, which
    /// we detect and return None.
    ///
    /// This matches the C `big_to_double` behavior, which returns -1 (error) if
    /// the result is not finite.
    pub fn to_f64(&self) -> Option<f64> {
        // Use malachite's RoundingFrom trait for efficient conversion
        // This is more efficient than string conversion and handles overflow correctly
        let (result, _ordering) = f64::rounding_from(&self.value, RoundingMode::Exact);
        
        // Check if result is finite (C's big_to_double returns -1 if not finite)
        if result.is_finite() {
            Some(result)
        } else {
            None  // Overflow case - value too large for f64
        }
    }

    /// Convert to u32
    ///
    /// Returns None if the value is negative or too large
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

    /// Convert to i64
    ///
    /// Returns None if the value is out of range
    pub fn to_i64(&self) -> Option<i64> {
        let s = self.value.to_string();
        s.parse::<i64>().ok()
    }

    /// Get the sign of the number
    ///
    /// Returns true if positive or zero, false if negative
    pub fn is_positive(&self) -> bool {
        self.value >= 0
    }

    /// Check if the number is zero
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Add two big numbers: x + y
    pub fn plus(&self, other: &Self) -> Self {
        Self {
            value: &self.value + &other.value,
        }
    }

    /// Subtract two big numbers: x - y
    pub fn minus(&self, other: &Self) -> Self {
        Self {
            value: &self.value - &other.value,
        }
    }

    /// Multiply two big numbers: x * y
    pub fn times(&self, other: &Self) -> Self {
        Self {
            value: &self.value * &other.value,
        }
    }

    /// Divide two big numbers: x / y
    ///
    /// Returns None if dividing by zero
    pub fn div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }
        Some(Self {
            value: &self.value / &other.value,
        })
    }

    /// Remainder of division: x % y
    ///
    /// Returns None if dividing by zero
    pub fn rem(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }
        Some(Self {
            value: &self.value % &other.value,
        })
    }

    /// Multiply and add: x * y + z
    pub fn mul_add(&self, y: &Self, z: &Self) -> Self {
        Self {
            value: &self.value * &y.value + &z.value,
        }
    }

    /// Add a small unsigned integer: x + y
    pub fn plus_small(&self, y: u32) -> Self {
        Self {
            value: &self.value + Integer::from(y),
        }
    }

    /// Bitwise AND: x & y
    pub fn bitand(&self, other: &Self) -> Self {
        Self {
            value: &self.value & &other.value,
        }
    }

    /// Bitwise OR: x | y
    pub fn bitor(&self, other: &Self) -> Self {
        Self {
            value: &self.value | &other.value,
        }
    }

    /// Bitwise XOR: x ^ y
    pub fn bitxor(&self, other: &Self) -> Self {
        Self {
            value: &self.value ^ &other.value,
        }
    }

    /// Bitwise NOT: !x
    ///
    /// In Erlang, bnot -X == (X - 1) and bnot +X == -(X + 1)
    /// This matches two's complement semantics
    pub fn bitnot(&self) -> Self {
        Self {
            value: !&self.value,
        }
    }

    /// Left shift: x << y
    ///
    /// y can be negative for right shift
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

    /// Compare two big numbers (signed comparison)
    ///
    /// Returns:
    /// - -1 if self < other
    /// - 0 if self == other
    /// - 1 if self > other
    pub fn comp(&self, other: &Self) -> i32 {
        match self.value.cmp(&other.value) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    /// Unsigned comparison of two big numbers
    ///
    /// Compares absolute values, ignoring sign
    /// Returns:
    /// - -1 if |self| < |other|
    /// - 0 if |self| == |other|
    /// - 1 if |self| > |other|
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

    /// Convert to string representation in given base
    ///
    /// Base must be between 2 and 36
    /// Uses standard formatting for common bases (2, 8, 10, 16) and
    /// manual conversion for other bases.
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

    /// Get the internal Integer value (for advanced use)
    pub fn as_integer(&self) -> &Integer {
        &self.value
    }

    /// Create from Integer (for advanced use)
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
