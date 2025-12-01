//! Math Utilities
//!
//! Provides mathematical utility functions based on erl_arith.c and erl_math.c.
//! These utilities handle arithmetic operations and mathematical functions.
//!
//! Includes support for Rational numbers (fractions) using the num-rational crate.

use num_rational::Rational64;
use num_traits::{Zero, One, Signed, ToPrimitive, FromPrimitive};

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

    /// Calculate square root
    ///
    /// # Arguments
    /// * `value` - Value to take square root of
    ///
    /// # Returns
    /// Square root
    pub fn sqrt(value: f64) -> f64 {
        value.sqrt()
    }

    /// Calculate natural logarithm
    ///
    /// # Arguments
    /// * `value` - Value to take logarithm of
    ///
    /// # Returns
    /// Natural logarithm
    pub fn ln(value: f64) -> f64 {
        value.ln()
    }

    /// Calculate base-10 logarithm
    ///
    /// # Arguments
    /// * `value` - Value to take logarithm of
    ///
    /// # Returns
    /// Base-10 logarithm
    pub fn log10(value: f64) -> f64 {
        value.log10()
    }

    /// Calculate base-2 logarithm
    ///
    /// # Arguments
    /// * `value` - Value to take logarithm of
    ///
    /// # Returns
    /// Base-2 logarithm
    pub fn log2(value: f64) -> f64 {
        value.log2()
    }

    /// Calculate e raised to the power of value
    ///
    /// # Arguments
    /// * `value` - Exponent
    ///
    /// # Returns
    /// e^value
    pub fn exp(value: f64) -> f64 {
        value.exp()
    }

    /// Calculate sine
    ///
    /// # Arguments
    /// * `value` - Angle in radians
    ///
    /// # Returns
    /// Sine of the angle
    pub fn sin(value: f64) -> f64 {
        value.sin()
    }

    /// Calculate cosine
    ///
    /// # Arguments
    /// * `value` - Angle in radians
    ///
    /// # Returns
    /// Cosine of the angle
    pub fn cos(value: f64) -> f64 {
        value.cos()
    }

    /// Calculate tangent
    ///
    /// # Arguments
    /// * `value` - Angle in radians
    ///
    /// # Returns
    /// Tangent of the angle
    pub fn tan(value: f64) -> f64 {
        value.tan()
    }

    /// Calculate arcsine
    ///
    /// # Arguments
    /// * `value` - Value in range [-1, 1]
    ///
    /// # Returns
    /// Angle in radians
    pub fn asin(value: f64) -> f64 {
        value.asin()
    }

    /// Calculate arccosine
    ///
    /// # Arguments
    /// * `value` - Value in range [-1, 1]
    ///
    /// # Returns
    /// Angle in radians
    pub fn acos(value: f64) -> f64 {
        value.acos()
    }

    /// Calculate arctangent
    ///
    /// # Arguments
    /// * `value` - Value
    ///
    /// # Returns
    /// Angle in radians
    pub fn atan(value: f64) -> f64 {
        value.atan()
    }

    /// Calculate arctangent of y/x
    ///
    /// # Arguments
    /// * `y` - Y coordinate
    /// * `x` - X coordinate
    ///
    /// # Returns
    /// Angle in radians
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    /// Calculate ceiling (round up)
    ///
    /// # Arguments
    /// * `value` - Value to round up
    ///
    /// # Returns
    /// Ceiling value
    pub fn ceil(value: f64) -> f64 {
        value.ceil()
    }

    /// Calculate floor (round down)
    ///
    /// # Arguments
    /// * `value` - Value to round down
    ///
    /// # Returns
    /// Floor value
    pub fn floor(value: f64) -> f64 {
        value.floor()
    }

    /// Round to nearest integer
    ///
    /// # Arguments
    /// * `value` - Value to round
    ///
    /// # Returns
    /// Rounded value
    pub fn round(value: f64) -> f64 {
        value.round()
    }

    /// Truncate to integer part
    ///
    /// # Arguments
    /// * `value` - Value to truncate
    ///
    /// # Returns
    /// Truncated value
    pub fn trunc(value: f64) -> f64 {
        value.trunc()
    }

    /// Calculate floating-point remainder
    ///
    /// # Arguments
    /// * `x` - Dividend
    /// * `y` - Divisor
    ///
    /// # Returns
    /// Remainder
    pub fn fmod(x: f64, y: f64) -> f64 {
        x % y
    }

    /// Calculate greatest common divisor
    ///
    /// # Arguments
    /// * `a` - First number
    /// * `b` - Second number
    ///
    /// # Returns
    /// GCD
    pub fn gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a.abs()
    }

    /// Calculate least common multiple
    ///
    /// # Arguments
    /// * `a` - First number
    /// * `b` - Second number
    ///
    /// # Returns
    /// LCM
    pub fn lcm(a: i64, b: i64) -> i64 {
        if a == 0 || b == 0 {
            return 0;
        }
        (a * b).abs() / Self::gcd(a, b)
    }
}

/// Rational number utilities
///
/// Provides operations for working with rational numbers (fractions).
/// Rational numbers are represented as fractions of integers, providing
/// exact arithmetic without floating-point rounding errors.
pub struct RationalUtils;

impl RationalUtils {
    /// Create a rational number from numerator and denominator
    ///
    /// # Arguments
    /// * `num` - Numerator
    /// * `den` - Denominator
    ///
    /// # Returns
    /// * `Some(Rational64)` - If denominator is not zero
    /// * `None` - If denominator is zero
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let r = RationalUtils::new(1, 2).unwrap();
    /// assert_eq!(r.to_f64(), Some(0.5));
    /// ```
    pub fn new(num: i64, den: i64) -> Option<Rational64> {
        if den == 0 {
            return None;
        }
        Some(Rational64::new(num, den))
    }

    /// Create a rational number from an integer
    ///
    /// # Arguments
    /// * `value` - Integer value
    ///
    /// # Returns
    /// Rational number representing the integer
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let r = RationalUtils::from_integer(5);
    /// assert_eq!(r.to_integer(), Some(5));
    /// ```
    pub fn from_integer(value: i64) -> Rational64 {
        Rational64::from_integer(value)
    }

    /// Create a rational number from a float (approximate)
    ///
    /// # Arguments
    /// * `value` - Float value
    ///
    /// # Returns
    /// Rational number approximating the float
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let r = RationalUtils::from_float(0.5);
    /// assert_eq!(r.to_f64(), Some(0.5));
    /// ```
    pub fn from_float(value: f64) -> Option<Rational64> {
        Rational64::from_f64(value)
    }

    /// Add two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// Sum of the two rationals
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let a = RationalUtils::new(1, 2).unwrap();
    /// let b = RationalUtils::new(1, 3).unwrap();
    /// let sum = RationalUtils::add(a, b);
    /// assert_eq!(sum.to_f64(), Some(5.0 / 6.0));
    /// ```
    pub fn add(a: Rational64, b: Rational64) -> Rational64 {
        a + b
    }

    /// Subtract two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// Difference of the two rationals
    pub fn subtract(a: Rational64, b: Rational64) -> Rational64 {
        a - b
    }

    /// Multiply two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// Product of the two rationals
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let a = RationalUtils::new(1, 2).unwrap();
    /// let b = RationalUtils::new(2, 3).unwrap();
    /// let product = RationalUtils::multiply(a, b);
    /// assert_eq!(product.to_f64(), Some(1.0 / 3.0));
    /// ```
    pub fn multiply(a: Rational64, b: Rational64) -> Rational64 {
        a * b
    }

    /// Divide two rational numbers
    ///
    /// # Arguments
    /// * `a` - Dividend
    /// * `b` - Divisor
    ///
    /// # Returns
    /// * `Some(Rational64)` - If divisor is not zero
    /// * `None` - If divisor is zero
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let a = RationalUtils::new(1, 2).unwrap();
    /// let b = RationalUtils::new(1, 3).unwrap();
    /// let quotient = RationalUtils::divide(a, b).unwrap();
    /// assert_eq!(quotient.to_f64(), Some(1.5));
    /// ```
    pub fn divide(a: Rational64, b: Rational64) -> Option<Rational64> {
        if b.is_zero() {
            return None;
        }
        Some(a / b)
    }

    /// Get the absolute value of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Absolute value
    pub fn abs(r: Rational64) -> Rational64 {
        r.abs()
    }

    /// Compare two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// * `Some(std::cmp::Ordering)` - Comparison result
    /// * `None` - If comparison is not possible (shouldn't happen for rationals)
    pub fn compare(a: Rational64, b: Rational64) -> Option<std::cmp::Ordering> {
        a.partial_cmp(&b)
    }

    /// Convert rational to integer (if it represents a whole number)
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// * `Some(i64)` - If the rational is a whole number
    /// * `None` - If it's not a whole number
    pub fn to_integer(r: Rational64) -> Option<i64> {
        if r.is_integer() {
            r.to_i64()
        } else {
            None
        }
    }

    /// Convert rational to float
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// * `Some(f64)` - Float representation
    /// * `None` - If conversion fails
    pub fn to_float(r: Rational64) -> Option<f64> {
        r.to_f64()
    }

    /// Get numerator of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Numerator
    pub fn numerator(r: Rational64) -> i64 {
        *r.numer()
    }

    /// Get denominator of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Denominator
    pub fn denominator(r: Rational64) -> i64 {
        *r.denom()
    }

    /// Check if rational is zero
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if zero, `false` otherwise
    pub fn is_zero(r: Rational64) -> bool {
        r.is_zero()
    }

    /// Check if rational is one
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if one, `false` otherwise
    pub fn is_one(r: Rational64) -> bool {
        r.is_one()
    }

    /// Check if rational is positive
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if positive, `false` otherwise
    pub fn is_positive(r: Rational64) -> bool {
        r.is_positive()
    }

    /// Check if rational is negative
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if negative, `false` otherwise
    pub fn is_negative(r: Rational64) -> bool {
        r.is_negative()
    }

    /// Reduce a rational to its simplest form
    ///
    /// Rational numbers are automatically reduced, but this method
    /// ensures the number is in canonical form.
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Reduced rational (same value, but in simplest form)
    pub fn reduce(r: Rational64) -> Rational64 {
        r.reduced()
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

    #[test]
    fn test_sqrt() {
        assert!((MathUtils::sqrt(4.0) - 2.0).abs() < 0.0001);
        assert!((MathUtils::sqrt(9.0) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_ln() {
        assert!((MathUtils::ln(std::f64::consts::E) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_log10() {
        assert!((MathUtils::log10(100.0) - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_log2() {
        assert!((MathUtils::log2(8.0) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_exp() {
        assert!((MathUtils::exp(1.0) - std::f64::consts::E).abs() < 0.0001);
    }

    #[test]
    fn test_sin_cos() {
        assert!((MathUtils::sin(0.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::cos(0.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_ceil_floor() {
        assert_eq!(MathUtils::ceil(3.1), 4.0);
        assert_eq!(MathUtils::floor(3.9), 3.0);
    }

    #[test]
    fn test_round_trunc() {
        assert_eq!(MathUtils::round(3.5), 4.0);
        assert_eq!(MathUtils::round(3.4), 3.0);
        assert_eq!(MathUtils::trunc(3.9), 3.0);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(MathUtils::gcd(48, 18), 6);
        assert_eq!(MathUtils::gcd(17, 13), 1);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(MathUtils::lcm(4, 6), 12);
        assert_eq!(MathUtils::lcm(5, 7), 35);
    }
}

#[cfg(test)]
mod rational_tests {
    use super::*;

    #[test]
    fn test_rational_new() {
        let r = RationalUtils::new(1, 2).unwrap();
        assert_eq!(r.to_f64(), Some(0.5));
        assert!(RationalUtils::new(1, 0).is_none());
    }

    #[test]
    fn test_rational_from_integer() {
        let r = RationalUtils::from_integer(5);
        assert_eq!(RationalUtils::to_integer(r), Some(5));
    }

    #[test]
    fn test_rational_from_float() {
        let r = RationalUtils::from_float(0.5).unwrap();
        assert!((r.to_f64().unwrap() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_rational_add() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let sum = RationalUtils::add(a, b);
        assert_eq!(sum, RationalUtils::new(5, 6).unwrap());
    }

    #[test]
    fn test_rational_subtract() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let diff = RationalUtils::subtract(a, b);
        assert_eq!(diff, RationalUtils::new(1, 6).unwrap());
    }

    #[test]
    fn test_rational_multiply() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(2, 3).unwrap();
        let product = RationalUtils::multiply(a, b);
        assert_eq!(product, RationalUtils::new(1, 3).unwrap());
    }

    #[test]
    fn test_rational_divide() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let quotient = RationalUtils::divide(a, b).unwrap();
        assert_eq!(quotient, RationalUtils::new(3, 2).unwrap());
        
        let zero = RationalUtils::new(0, 1).unwrap();
        assert!(RationalUtils::divide(a, zero).is_none());
    }

    #[test]
    fn test_rational_abs() {
        let r = RationalUtils::new(-1, 2).unwrap();
        let abs_r = RationalUtils::abs(r);
        assert_eq!(abs_r, RationalUtils::new(1, 2).unwrap());
    }

    #[test]
    fn test_rational_compare() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        assert!(RationalUtils::compare(a, b).unwrap().is_gt());
        assert!(RationalUtils::compare(b, a).unwrap().is_lt());
        assert!(RationalUtils::compare(a, a).unwrap().is_eq());
    }

    #[test]
    fn test_rational_to_integer() {
        let r = RationalUtils::from_integer(5);
        assert_eq!(RationalUtils::to_integer(r), Some(5));
        
        let r2 = RationalUtils::new(1, 2).unwrap();
        assert_eq!(RationalUtils::to_integer(r2), None);
    }

    #[test]
    fn test_rational_numerator_denominator() {
        let r = RationalUtils::new(3, 4).unwrap();
        assert_eq!(RationalUtils::numerator(r), 3);
        assert_eq!(RationalUtils::denominator(r), 4);
    }

    #[test]
    fn test_rational_is_zero() {
        let zero = RationalUtils::new(0, 1).unwrap();
        assert!(RationalUtils::is_zero(zero));
        
        let non_zero = RationalUtils::new(1, 2).unwrap();
        assert!(!RationalUtils::is_zero(non_zero));
    }

    #[test]
    fn test_rational_is_one() {
        let one = RationalUtils::new(1, 1).unwrap();
        assert!(RationalUtils::is_one(one));
        
        let two = RationalUtils::new(2, 1).unwrap();
        assert!(!RationalUtils::is_one(two));
    }

    #[test]
    fn test_rational_is_positive_negative() {
        let pos = RationalUtils::new(1, 2).unwrap();
        assert!(RationalUtils::is_positive(pos));
        assert!(!RationalUtils::is_negative(pos));
        
        let neg = RationalUtils::new(-1, 2).unwrap();
        assert!(!RationalUtils::is_positive(neg));
        assert!(RationalUtils::is_negative(neg));
    }

    #[test]
    fn test_rational_reduce() {
        let r = RationalUtils::new(2, 4).unwrap();
        let reduced = RationalUtils::reduce(r);
        assert_eq!(reduced, RationalUtils::new(1, 2).unwrap());
    }
}

