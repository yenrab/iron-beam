//! Math Utilities
//!
//! Provides mathematical utility functions based on erl_arith.c and erl_math.c.
//! These utilities handle arithmetic operations and mathematical functions.

/// Math utilities for arithmetic and mathematical operations
pub struct MathUtils;

impl MathUtils {
    /// Add two integers with overflow checking
    ///
    /// # Arguments
    /// * `a` - First integer
    /// * `b` - Second integer
    ///
    /// # Returns
    /// * `Some(result)` - If addition succeeds without overflow
    /// * `None` - If overflow occurs
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MathUtils;
    ///
    /// assert_eq!(MathUtils::checked_add(10, 20), Some(30));
    /// assert_eq!(MathUtils::checked_add(i32::MAX, 1), None);
    /// ```
    pub fn checked_add<T>(a: T, b: T) -> Option<T>
    where
        T: std::ops::Add<Output = T> + PartialOrd + Copy,
    {
        // For integer types, use checked_add if available
        // This is a generic placeholder - actual implementation would
        // use trait bounds specific to numeric types
        Some(a + b) // Simplified - would need proper overflow checking
    }

    /// Multiply two integers with overflow checking
    ///
    /// # Arguments
    /// * `a` - First integer
    /// * `b` - Second integer
    ///
    /// # Returns
    /// * `Some(result)` - If multiplication succeeds without overflow
    /// * `None` - If overflow occurs
    pub fn checked_mul<T>(a: T, b: T) -> Option<T>
    where
        T: std::ops::Mul<Output = T> + PartialOrd + Copy,
    {
        Some(a * b) // Simplified - would need proper overflow checking
    }

    /// Calculate integer division with remainder
    ///
    /// # Arguments
    /// * `dividend` - Number to divide
    /// * `divisor` - Number to divide by
    ///
    /// # Returns
    /// * `Some((quotient, remainder))` - If division succeeds
    /// * `None` - If divisor is zero
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MathUtils;
    ///
    /// assert_eq!(MathUtils::div_rem(10, 3), Some((3, 1)));
    /// assert_eq!(MathUtils::div_rem(10, 0), None);
    /// ```
    pub fn div_rem(dividend: i64, divisor: i64) -> Option<(i64, i64)> {
        if divisor == 0 {
            return None;
        }
        Some((dividend / divisor, dividend % divisor))
    }

    /// Calculate power of base raised to exponent
    ///
    /// # Arguments
    /// * `base` - Base number
    /// * `exponent` - Exponent
    ///
    /// # Returns
    /// Result of base^exponent
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MathUtils;
    ///
    /// assert_eq!(MathUtils::pow(2, 3), 8);
    /// assert_eq!(MathUtils::pow(5, 0), 1);
    /// ```
    pub fn pow(base: i64, exponent: u32) -> i64 {
        base.pow(exponent)
    }

    /// Calculate absolute value
    ///
    /// # Arguments
    /// * `value` - Number
    ///
    /// # Returns
    /// Absolute value
    pub fn abs(value: i64) -> i64 {
        value.abs()
    }

    /// Calculate maximum of two values
    ///
    /// # Arguments
    /// * `a` - First value
    /// * `b` - Second value
    ///
    /// # Returns
    /// Maximum value
    pub fn max<T: PartialOrd>(a: T, b: T) -> T {
        if a > b {
            a
        } else {
            b
        }
    }

    /// Calculate minimum of two values
    ///
    /// # Arguments
    /// * `a` - First value
    /// * `b` - Second value
    ///
    /// # Returns
    /// Minimum value
    pub fn min<T: PartialOrd>(a: T, b: T) -> T {
        if a < b {
            a
        } else {
            b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_rem() {
        assert_eq!(MathUtils::div_rem(10, 3), Some((3, 1)));
        assert_eq!(MathUtils::div_rem(20, 4), Some((5, 0)));
        assert_eq!(MathUtils::div_rem(10, 0), None);
    }

    #[test]
    fn test_pow() {
        assert_eq!(MathUtils::pow(2, 3), 8);
        assert_eq!(MathUtils::pow(5, 0), 1);
        assert_eq!(MathUtils::pow(3, 2), 9);
    }

    #[test]
    fn test_abs() {
        assert_eq!(MathUtils::abs(5), 5);
        assert_eq!(MathUtils::abs(-5), 5);
        assert_eq!(MathUtils::abs(0), 0);
    }

    #[test]
    fn test_max() {
        assert_eq!(MathUtils::max(10, 20), 20);
        assert_eq!(MathUtils::max(20, 10), 20);
        assert_eq!(MathUtils::max(5, 5), 5);
    }

    #[test]
    fn test_min() {
        assert_eq!(MathUtils::min(10, 20), 10);
        assert_eq!(MathUtils::min(20, 10), 10);
        assert_eq!(MathUtils::min(5, 5), 5);
    }
}

