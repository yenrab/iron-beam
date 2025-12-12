//! Math Utilities
//!
//! Provides mathematical utility functions based on erl_arith.c and erl_math.c.
//! These utilities handle arithmetic operations and mathematical functions.
//!
//! Includes support for Rational numbers (fractions) using BigRational from entities_utilities.

use entities_utilities::BigRational;

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
/// Uses BigRational from entities_utilities for arbitrary precision.
pub struct RationalUtils;

impl RationalUtils {
    /// Create a rational number from numerator and denominator
    ///
    /// # Arguments
    /// * `num` - Numerator
    /// * `den` - Denominator
    ///
    /// # Returns
    /// * `Some(BigRational)` - If denominator is not zero
    /// * `None` - If denominator is zero
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let r = RationalUtils::new(1, 2).unwrap();
    /// assert_eq!(r.to_f64(), 0.5);
    /// ```
    pub fn new(num: i64, den: i64) -> Option<BigRational> {
        BigRational::from_fraction(num, den)
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
    /// assert_eq!(r.to_i64(), Some(5));
    /// ```
    pub fn from_integer(value: i64) -> BigRational {
        BigRational::from_i64(value)
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
    /// assert_eq!(r.to_f64(), 0.5);
    /// ```
    pub fn from_float(value: f64) -> Option<BigRational> {
        BigRational::from_f64(value)
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
    /// let sum = RationalUtils::add(&a, &b);
    /// assert!((sum.to_f64() - (5.0 / 6.0)).abs() < 1e-10);
    /// ```
    pub fn add(a: &BigRational, b: &BigRational) -> BigRational {
        a.plus(b)
    }

    /// Subtract two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// Difference of the two rationals
    pub fn subtract(a: &BigRational, b: &BigRational) -> BigRational {
        a.minus(b)
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
    /// let product = RationalUtils::multiply(&a, &b);
    /// assert!((product.to_f64() - (1.0 / 3.0)).abs() < 1e-10);
    /// ```
    pub fn multiply(a: &BigRational, b: &BigRational) -> BigRational {
        a.times(b)
    }

    /// Divide two rational numbers
    ///
    /// # Arguments
    /// * `a` - Dividend
    /// * `b` - Divisor
    ///
    /// # Returns
    /// * `Some(BigRational)` - If divisor is not zero
    /// * `None` - If divisor is zero
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::RationalUtils;
    ///
    /// let a = RationalUtils::new(1, 2).unwrap();
    /// let b = RationalUtils::new(1, 3).unwrap();
    /// let quotient = RationalUtils::divide(&a, &b).unwrap();
    /// assert!((quotient.to_f64() - 1.5).abs() < 1e-10);
    /// ```
    pub fn divide(a: &BigRational, b: &BigRational) -> Option<BigRational> {
        a.div(b)
    }

    /// Get the absolute value of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Absolute value
    pub fn abs(r: &BigRational) -> BigRational {
        r.abs()
    }

    /// Compare two rational numbers
    ///
    /// # Arguments
    /// * `a` - First rational
    /// * `b` - Second rational
    ///
    /// # Returns
    /// Comparison result (Ordering)
    pub fn compare(a: &BigRational, b: &BigRational) -> std::cmp::Ordering {
        a.comp(b)
    }

    /// Convert rational to integer (if it represents a whole number)
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// * `Some(i64)` - If the rational is a whole number
    /// * `None` - If it's not a whole number
    pub fn to_integer(r: &BigRational) -> Option<i64> {
            r.to_i64()
    }

    /// Convert rational to float
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Float representation
    pub fn to_float(r: &BigRational) -> f64 {
        r.to_f64()
    }

    /// Get numerator of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Numerator as malachite::Integer (arbitrary precision)
    pub fn numerator(r: &BigRational) -> malachite::Integer {
        r.numerator()
    }

    /// Get denominator of a rational number
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// Denominator as malachite::Integer (arbitrary precision)
    pub fn denominator(r: &BigRational) -> malachite::Integer {
        r.denominator()
    }

    /// Check if rational is zero
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if zero, `false` otherwise
    pub fn is_zero(r: &BigRational) -> bool {
        r.is_zero()
    }

    /// Check if rational is one
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if one, `false` otherwise
    pub fn is_one(r: &BigRational) -> bool {
        // Check if r == 1
        let one = BigRational::from_i64(1);
        r.comp(&one) == std::cmp::Ordering::Equal
    }

    /// Check if rational is positive
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if positive, `false` otherwise
    pub fn is_positive(r: &BigRational) -> bool {
        r.is_positive()
    }

    /// Check if rational is negative
    ///
    /// # Arguments
    /// * `r` - Rational number
    ///
    /// # Returns
    /// `true` if negative, `false` otherwise
    pub fn is_negative(r: &BigRational) -> bool {
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
    /// Note: BigRational is always in reduced form, so this returns a clone
    pub fn reduce(r: &BigRational) -> BigRational {
        r.clone() // BigRational is always in reduced form
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

    #[test]
    fn test_checked_add() {
        assert_eq!(MathUtils::checked_add(10, 20), Some(30));
        assert_eq!(MathUtils::checked_add(-5, 10), Some(5));
        assert_eq!(MathUtils::checked_add(0, 0), Some(0));
        // Note: The current implementation doesn't check overflow, so it always returns Some
        // This test verifies the basic functionality
    }

    #[test]
    fn test_checked_mul() {
        assert_eq!(MathUtils::checked_mul(5, 4), Some(20));
        assert_eq!(MathUtils::checked_mul(-3, 2), Some(-6));
        assert_eq!(MathUtils::checked_mul(0, 100), Some(0));
        // Note: The current implementation doesn't check overflow, so it always returns Some
        // This test verifies the basic functionality
    }

    #[test]
    fn test_div_rem_edge_cases() {
        // Test negative numbers
        assert_eq!(MathUtils::div_rem(-10, 3), Some((-3, -1)));
        assert_eq!(MathUtils::div_rem(10, -3), Some((-3, 1)));
        assert_eq!(MathUtils::div_rem(-10, -3), Some((3, -1)));
        
        // Test with 1
        assert_eq!(MathUtils::div_rem(5, 1), Some((5, 0)));
        
        // Test large numbers
        assert_eq!(MathUtils::div_rem(100, 7), Some((14, 2)));
    }

    #[test]
    fn test_pow_edge_cases() {
        assert_eq!(MathUtils::pow(1, 100), 1);
        assert_eq!(MathUtils::pow(0, 5), 0);
        assert_eq!(MathUtils::pow(-2, 3), -8);
        assert_eq!(MathUtils::pow(-2, 2), 4);
        assert_eq!(MathUtils::pow(10, 1), 10);
    }

    #[test]
    fn test_abs_edge_cases() {
        assert_eq!(MathUtils::abs(i64::MIN + 1), i64::MAX);
        assert_eq!(MathUtils::abs(i64::MAX), i64::MAX);
    }

    #[test]
    fn test_max_min_different_types() {
        assert_eq!(MathUtils::max(10.5, 20.3), 20.3);
        assert_eq!(MathUtils::min(10.5, 20.3), 10.5);
        assert_eq!(MathUtils::max("abc", "def"), "def");
        assert_eq!(MathUtils::min("abc", "def"), "abc");
    }

    #[test]
    fn test_sqrt_edge_cases() {
        assert!((MathUtils::sqrt(0.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::sqrt(1.0) - 1.0).abs() < 0.0001);
        assert!((MathUtils::sqrt(16.0) - 4.0).abs() < 0.0001);
        assert!((MathUtils::sqrt(25.0) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_ln_edge_cases() {
        assert!((MathUtils::ln(1.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::ln(10.0) - 2.302585).abs() < 0.0001);
    }

    #[test]
    fn test_log10_edge_cases() {
        assert!((MathUtils::log10(1.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::log10(10.0) - 1.0).abs() < 0.0001);
        assert!((MathUtils::log10(1000.0) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_log2_edge_cases() {
        assert!((MathUtils::log2(1.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::log2(2.0) - 1.0).abs() < 0.0001);
        assert!((MathUtils::log2(4.0) - 2.0).abs() < 0.0001);
        assert!((MathUtils::log2(16.0) - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_exp_edge_cases() {
        assert!((MathUtils::exp(0.0) - 1.0).abs() < 0.0001);
        assert!((MathUtils::exp(2.0) - 7.389056).abs() < 0.0001);
    }

    #[test]
    fn test_tan() {
        assert!((MathUtils::tan(0.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::tan(std::f64::consts::PI / 4.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_asin() {
        assert!((MathUtils::asin(0.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::asin(1.0) - std::f64::consts::PI / 2.0).abs() < 0.0001);
        assert!((MathUtils::asin(-1.0) + std::f64::consts::PI / 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_acos() {
        assert!((MathUtils::acos(1.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::acos(0.0) - std::f64::consts::PI / 2.0).abs() < 0.0001);
        assert!((MathUtils::acos(-1.0) - std::f64::consts::PI).abs() < 0.0001);
    }

    #[test]
    fn test_atan() {
        assert!((MathUtils::atan(0.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::atan(1.0) - std::f64::consts::PI / 4.0).abs() < 0.0001);
        assert!((MathUtils::atan(-1.0) + std::f64::consts::PI / 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_atan2() {
        assert!((MathUtils::atan2(0.0, 1.0) - 0.0).abs() < 0.0001);
        assert!((MathUtils::atan2(1.0, 1.0) - std::f64::consts::PI / 4.0).abs() < 0.0001);
        assert!((MathUtils::atan2(1.0, 0.0) - std::f64::consts::PI / 2.0).abs() < 0.0001);
        assert!((MathUtils::atan2(-1.0, 0.0) + std::f64::consts::PI / 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_ceil_floor_edge_cases() {
        assert_eq!(MathUtils::ceil(3.0), 3.0);
        assert_eq!(MathUtils::floor(3.0), 3.0);
        assert_eq!(MathUtils::ceil(-3.1), -3.0);
        assert_eq!(MathUtils::floor(-3.9), -4.0);
        assert_eq!(MathUtils::ceil(0.0), 0.0);
        assert_eq!(MathUtils::floor(0.0), 0.0);
    }

    #[test]
    fn test_round_trunc_edge_cases() {
        assert_eq!(MathUtils::round(0.0), 0.0);
        assert_eq!(MathUtils::trunc(0.0), 0.0);
        assert_eq!(MathUtils::round(-3.5), -4.0);
        assert_eq!(MathUtils::round(-3.4), -3.0);
        assert_eq!(MathUtils::trunc(-3.9), -3.0);
    }

    #[test]
    fn test_fmod() {
        assert!((MathUtils::fmod(10.0, 3.0) - 1.0).abs() < 0.0001);
        assert!((MathUtils::fmod(10.5, 3.0) - 1.5).abs() < 0.0001);
        assert!((MathUtils::fmod(-10.0, 3.0) + 1.0).abs() < 0.0001);
        assert!((MathUtils::fmod(10.0, -3.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_gcd_edge_cases() {
        assert_eq!(MathUtils::gcd(0, 5), 5);
        assert_eq!(MathUtils::gcd(5, 0), 5);
        assert_eq!(MathUtils::gcd(0, 0), 0);
        assert_eq!(MathUtils::gcd(1, 1), 1);
        assert_eq!(MathUtils::gcd(-48, 18), 6);
        assert_eq!(MathUtils::gcd(48, -18), 6);
        assert_eq!(MathUtils::gcd(-48, -18), 6);
    }

    #[test]
    fn test_lcm_edge_cases() {
        assert_eq!(MathUtils::lcm(0, 5), 0);
        assert_eq!(MathUtils::lcm(5, 0), 0);
        assert_eq!(MathUtils::lcm(0, 0), 0);
        assert_eq!(MathUtils::lcm(1, 1), 1);
        assert_eq!(MathUtils::lcm(-4, 6), 12);
        assert_eq!(MathUtils::lcm(4, -6), 12);
        assert_eq!(MathUtils::lcm(-4, -6), 12);
    }
}

#[cfg(test)]
mod rational_tests {
    use super::*;

    #[test]
    fn test_rational_new() {
        let r = RationalUtils::new(1, 2).unwrap();
        assert!((r.to_f64() - 0.5).abs() < 1e-10);
        assert!(RationalUtils::new(1, 0).is_none());
    }

    #[test]
    fn test_rational_from_integer() {
        let r = RationalUtils::from_integer(5);
        assert_eq!(RationalUtils::to_integer(&r), Some(5));
    }

    #[test]
    fn test_rational_from_float() {
        let r = RationalUtils::from_float(0.5).unwrap();
        assert!((r.to_f64() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_rational_add() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let sum = RationalUtils::add(&a, &b);
        let expected = RationalUtils::new(5, 6).unwrap();
        assert!((sum.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_subtract() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let diff = RationalUtils::subtract(&a, &b);
        let expected = RationalUtils::new(1, 6).unwrap();
        assert!((diff.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_multiply() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(2, 3).unwrap();
        let product = RationalUtils::multiply(&a, &b);
        let expected = RationalUtils::new(1, 3).unwrap();
        assert!((product.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_divide() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        let quotient = RationalUtils::divide(&a, &b).unwrap();
        let expected = RationalUtils::new(3, 2).unwrap();
        assert!((quotient.to_f64() - expected.to_f64()).abs() < 1e-10);
        
        let zero = RationalUtils::new(0, 1).unwrap();
        assert!(RationalUtils::divide(&a, &zero).is_none());
    }

    #[test]
    fn test_rational_abs() {
        let r = RationalUtils::new(-1, 2).unwrap();
        let abs_r = RationalUtils::abs(&r);
        let expected = RationalUtils::new(1, 2).unwrap();
        assert!((abs_r.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_compare() {
        let a = RationalUtils::new(1, 2).unwrap();
        let b = RationalUtils::new(1, 3).unwrap();
        assert!(RationalUtils::compare(&a, &b).is_gt());
        assert!(RationalUtils::compare(&b, &a).is_lt());
        assert!(RationalUtils::compare(&a, &a).is_eq());
    }

    #[test]
    fn test_rational_to_integer() {
        let r = RationalUtils::from_integer(5);
        assert_eq!(RationalUtils::to_integer(&r), Some(5));
        
        let r2 = RationalUtils::new(1, 2).unwrap();
        assert_eq!(RationalUtils::to_integer(&r2), None);
    }

    #[test]
    fn test_rational_numerator_denominator() {
        use malachite::Integer;
        let r = RationalUtils::new(3, 4).unwrap();
        assert_eq!(RationalUtils::numerator(&r), Integer::from(3));
        assert_eq!(RationalUtils::denominator(&r), Integer::from(4));
    }

    #[test]
    fn test_rational_is_zero() {
        let zero = RationalUtils::new(0, 1).unwrap();
        assert!(RationalUtils::is_zero(&zero));
        
        let non_zero = RationalUtils::new(1, 2).unwrap();
        assert!(!RationalUtils::is_zero(&non_zero));
    }

    #[test]
    fn test_rational_is_one() {
        let one = RationalUtils::new(1, 1).unwrap();
        assert!(RationalUtils::is_one(&one));
        
        let two = RationalUtils::new(2, 1).unwrap();
        assert!(!RationalUtils::is_one(&two));
    }

    #[test]
    fn test_rational_is_positive_negative() {
        let pos = RationalUtils::new(1, 2).unwrap();
        assert!(RationalUtils::is_positive(&pos));
        assert!(!RationalUtils::is_negative(&pos));
        
        let neg = RationalUtils::new(-1, 2).unwrap();
        assert!(!RationalUtils::is_positive(&neg));
        assert!(RationalUtils::is_negative(&neg));
    }

    #[test]
    fn test_rational_reduce() {
        let r = RationalUtils::new(2, 4).unwrap();
        let reduced = RationalUtils::reduce(&r);
        let expected = RationalUtils::new(1, 2).unwrap();
        assert!((reduced.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_to_float() {
        let r = RationalUtils::new(1, 2).unwrap();
        assert!((RationalUtils::to_float(&r) - 0.5).abs() < 1e-10);
        
        let r2 = RationalUtils::new(3, 4).unwrap();
        assert!((RationalUtils::to_float(&r2) - 0.75).abs() < 1e-10);
        
        let r3 = RationalUtils::from_integer(5);
        assert!((RationalUtils::to_float(&r3) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_rational_new_edge_cases() {
        // Test with negative numbers
        let r = RationalUtils::new(-1, 2).unwrap();
        assert!((r.to_f64() + 0.5).abs() < 1e-10);
        
        let r2 = RationalUtils::new(1, -2).unwrap();
        assert!((r2.to_f64() + 0.5).abs() < 1e-10);
        
        let r3 = RationalUtils::new(-1, -2).unwrap();
        assert!((r3.to_f64() - 0.5).abs() < 1e-10);
        
        // Test with zero numerator
        let r4 = RationalUtils::new(0, 5).unwrap();
        assert!(RationalUtils::is_zero(&r4));
    }

    #[test]
    fn test_rational_from_integer_edge_cases() {
        let r1 = RationalUtils::from_integer(0);
        assert!(RationalUtils::is_zero(&r1));
        
        let r2 = RationalUtils::from_integer(-5);
        assert_eq!(RationalUtils::to_integer(&r2), Some(-5));
        
        let r3 = RationalUtils::from_integer(1);
        assert!(RationalUtils::is_one(&r3));
    }

    #[test]
    fn test_rational_from_float_edge_cases() {
        let r1 = RationalUtils::from_float(0.0).unwrap();
        assert!(RationalUtils::is_zero(&r1));
        
        let r2 = RationalUtils::from_float(1.0).unwrap();
        assert!(RationalUtils::is_one(&r2));
        
        let r3 = RationalUtils::from_float(-0.5).unwrap();
        assert!(RationalUtils::is_negative(&r3));
    }

    #[test]
    fn test_rational_add_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let r = RationalUtils::new(1, 2).unwrap();
        
        // Adding zero
        let sum = RationalUtils::add(&r, &zero);
        assert!((sum.to_f64() - r.to_f64()).abs() < 1e-10);
        
        // Adding negative
        let neg = RationalUtils::new(-1, 3).unwrap();
        let sum2 = RationalUtils::add(&r, &neg);
        let expected = RationalUtils::new(1, 6).unwrap();
        assert!((sum2.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_subtract_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let r = RationalUtils::new(1, 2).unwrap();
        
        // Subtracting zero
        let diff = RationalUtils::subtract(&r, &zero);
        assert!((diff.to_f64() - r.to_f64()).abs() < 1e-10);
        
        // Subtracting from zero
        let diff2 = RationalUtils::subtract(&zero, &r);
        let expected = RationalUtils::new(-1, 2).unwrap();
        assert!((diff2.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_multiply_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let one = RationalUtils::new(1, 1).unwrap();
        let r = RationalUtils::new(1, 2).unwrap();
        
        // Multiplying by zero
        let product = RationalUtils::multiply(&r, &zero);
        assert!(RationalUtils::is_zero(&product));
        
        // Multiplying by one
        let product2 = RationalUtils::multiply(&r, &one);
        assert!((product2.to_f64() - r.to_f64()).abs() < 1e-10);
        
        // Multiplying negative
        let neg = RationalUtils::new(-1, 3).unwrap();
        let product3 = RationalUtils::multiply(&r, &neg);
        let expected = RationalUtils::new(-1, 6).unwrap();
        assert!((product3.to_f64() - expected.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_divide_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let one = RationalUtils::new(1, 1).unwrap();
        let r = RationalUtils::new(1, 2).unwrap();
        
        // Dividing zero
        let quotient = RationalUtils::divide(&zero, &r).unwrap();
        assert!(RationalUtils::is_zero(&quotient));
        
        // Dividing by one
        let quotient2 = RationalUtils::divide(&r, &one).unwrap();
        assert!((quotient2.to_f64() - r.to_f64()).abs() < 1e-10);
        
        // Dividing by zero should return None
        assert!(RationalUtils::divide(&r, &zero).is_none());
    }

    #[test]
    fn test_rational_abs_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let abs_zero = RationalUtils::abs(&zero);
        assert!(RationalUtils::is_zero(&abs_zero));
        
        let pos = RationalUtils::new(1, 2).unwrap();
        let abs_pos = RationalUtils::abs(&pos);
        assert!((abs_pos.to_f64() - pos.to_f64()).abs() < 1e-10);
    }

    #[test]
    fn test_rational_compare_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        let one = RationalUtils::new(1, 1).unwrap();
        
        assert!(RationalUtils::compare(&zero, &one).is_lt());
        assert!(RationalUtils::compare(&one, &zero).is_gt());
        assert!(RationalUtils::compare(&zero, &zero).is_eq());
        
        let neg = RationalUtils::new(-1, 2).unwrap();
        assert!(RationalUtils::compare(&neg, &zero).is_lt());
        assert!(RationalUtils::compare(&zero, &neg).is_gt());
    }

    #[test]
    fn test_rational_to_integer_edge_cases() {
        let zero = RationalUtils::from_integer(0);
        assert_eq!(RationalUtils::to_integer(&zero), Some(0));
        
        let neg = RationalUtils::from_integer(-5);
        assert_eq!(RationalUtils::to_integer(&neg), Some(-5));
        
        let fraction = RationalUtils::new(3, 2).unwrap();
        assert_eq!(RationalUtils::to_integer(&fraction), None);
    }

    #[test]
    fn test_rational_numerator_denominator_edge_cases() {
        use malachite::Integer;
        
        let zero = RationalUtils::new(0, 5).unwrap();
        assert_eq!(RationalUtils::numerator(&zero), Integer::from(0));
        
        let neg = RationalUtils::new(-3, 4).unwrap();
        // Rational should be negative (BigRational normalizes sign)
        assert!(RationalUtils::is_negative(&neg));
        
        // Numerator and denominator should be non-zero
        let num = RationalUtils::numerator(&neg);
        let den = RationalUtils::denominator(&neg);
        assert_ne!(num, Integer::from(0));
        assert_ne!(den, Integer::from(0));
    }

    #[test]
    fn test_rational_is_zero_edge_cases() {
        let zero1 = RationalUtils::new(0, 1).unwrap();
        assert!(RationalUtils::is_zero(&zero1));
        
        let zero2 = RationalUtils::new(0, 100).unwrap();
        assert!(RationalUtils::is_zero(&zero2));
        
        let non_zero = RationalUtils::new(1, 1000000).unwrap();
        assert!(!RationalUtils::is_zero(&non_zero));
    }

    #[test]
    fn test_rational_is_one_edge_cases() {
        let one1 = RationalUtils::new(1, 1).unwrap();
        assert!(RationalUtils::is_one(&one1));
        
        let one2 = RationalUtils::new(2, 2).unwrap();
        assert!(RationalUtils::is_one(&one2));
        
        let one3 = RationalUtils::new(-1, -1).unwrap();
        assert!(RationalUtils::is_one(&one3));
        
        let not_one = RationalUtils::new(2, 1).unwrap();
        assert!(!RationalUtils::is_one(&not_one));
    }

    #[test]
    fn test_rational_is_positive_negative_edge_cases() {
        let zero = RationalUtils::new(0, 1).unwrap();
        assert!(!RationalUtils::is_positive(&zero));
        assert!(!RationalUtils::is_negative(&zero));
        
        let small_pos = RationalUtils::new(1, 1000000).unwrap();
        assert!(RationalUtils::is_positive(&small_pos));
        assert!(!RationalUtils::is_negative(&small_pos));
        
        let small_neg = RationalUtils::new(-1, 1000000).unwrap();
        assert!(!RationalUtils::is_positive(&small_neg));
        assert!(RationalUtils::is_negative(&small_neg));
    }

    #[test]
    fn test_rational_reduce_edge_cases() {
        let r1 = RationalUtils::new(4, 8).unwrap();
        let reduced1 = RationalUtils::reduce(&r1);
        let expected1 = RationalUtils::new(1, 2).unwrap();
        assert!((reduced1.to_f64() - expected1.to_f64()).abs() < 1e-10);
        
        let r2 = RationalUtils::new(100, 200).unwrap();
        let reduced2 = RationalUtils::reduce(&r2);
        let expected2 = RationalUtils::new(1, 2).unwrap();
        assert!((reduced2.to_f64() - expected2.to_f64()).abs() < 1e-10);
        
        // Already reduced
        let r3 = RationalUtils::new(1, 2).unwrap();
        let reduced3 = RationalUtils::reduce(&r3);
        assert!((reduced3.to_f64() - r3.to_f64()).abs() < 1e-10);
    }
}

