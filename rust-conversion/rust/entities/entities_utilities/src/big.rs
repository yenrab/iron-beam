//! Big Number Operations
//!
//! Provides arbitrary precision integer operations.
//! Based on big.c
//!
//! This module uses the `malachite` crate for high-performance
//! arbitrary-precision arithmetic, which provides behavior compatible
//! with the C implementation's two's complement semantics for operations.

use malachite::Integer;

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
    pub fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        // Convert to integer by truncating
        // Note: C code does similar conversion in double_to_big
        // Malachite doesn't have direct from_f64, so we convert via i64 first
        // For values outside i64 range, we'd need a more complex conversion
        let truncated = value.trunc();
        if truncated >= i64::MIN as f64 && truncated <= i64::MAX as f64 {
            Some(Self {
                value: Integer::from(truncated as i64),
            })
        } else {
            None
        }
    }

    /// Convert to f64
    ///
    /// Returns None if the value is too large to represent as f64
    /// Note: This is a simplified conversion - for full precision,
    /// a more complex implementation would be needed
    pub fn to_f64(&self) -> Option<f64> {
        // For now, convert via string representation for very large numbers
        // This is a limitation - proper implementation would need more work
        let s = self.value.to_string();
        s.parse::<f64>().ok()
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
    /// Note: This uses a simple conversion - for full base conversion
    /// support, additional implementation would be needed
    pub fn to_string_base(&self, base: u32) -> String {
        if base < 2 || base > 36 {
            panic!("Base must be between 2 and 36");
        }
        // For bases 10, 16, 2, 8 we can use standard formatting
        // For other bases, convert via base 10 for now
        match base {
            10 => self.value.to_string(),
            16 => format!("{:x}", self.value),
            2 => format!("{:b}", self.value),
            8 => format!("{:o}", self.value),
            _ => {
                // For other bases, use a simple conversion
                // This is a placeholder - proper implementation needed
                let mut result = String::new();
                let mut n = self.value.clone();
                let zero = Integer::from(0);
                let base_int = Integer::from(base);
                
                if n < zero {
                    result.push('-');
                    n = -n;
                }
                
                if n == zero {
                    return "0".to_string();
                }
                
                let mut digits = Vec::new();
                while n > zero {
                    let rem = &n % &base_int;
                    let digit = rem.to_string().parse::<u32>().unwrap_or(0);
                    digits.push(digit);
                    n = &n / &base_int;
                }
                
                for &digit in digits.iter().rev() {
                    if digit < 10 {
                        result.push((b'0' + digit as u8) as char);
                    } else {
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
    }

    #[test]
    fn test_bitwise_operations() {
        let a = BigNumber::from_i64(0b1010); // 10
        let b = BigNumber::from_i64(0b1100); // 12

        assert_eq!(a.bitand(&b).to_i64(), Some(0b1000)); // 8
        assert_eq!(a.bitor(&b).to_i64(), Some(0b1110)); // 14
        assert_eq!(a.bitxor(&b).to_i64(), Some(0b0110)); // 6
    }

    #[test]
    fn test_shift_operations() {
        let a = BigNumber::from_i64(10);

        assert_eq!(a.lshift(2).to_i64(), Some(40)); // 10 << 2 = 40
        assert_eq!(a.lshift(-1).to_i64(), Some(5)); // 10 >> 1 = 5
    }

    #[test]
    fn test_comparison() {
        let a = BigNumber::from_i64(100);
        let b = BigNumber::from_i64(50);
        let c = BigNumber::from_i64(100);

        assert_eq!(a.comp(&b), 1);
        assert_eq!(b.comp(&a), -1);
        assert_eq!(a.comp(&c), 0);
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
    }

    #[test]
    fn test_string_conversion() {
        let big = BigNumber::from_i64(255);
        assert_eq!(big.to_string_base(16), "ff");
        assert_eq!(big.to_string_base(10), "255");
        assert_eq!(big.to_string_base(2), "11111111");
    }

    #[test]
    fn test_large_numbers() {
        // Test with numbers larger than i64
        let a = BigNumber::from_u64(u64::MAX);
        let b = BigNumber::from_u64(1);
        let sum = a.plus(&b);
        
        // Should handle overflow correctly
        assert!(sum.to_i64().is_none()); // Too large for i64
    }
}
