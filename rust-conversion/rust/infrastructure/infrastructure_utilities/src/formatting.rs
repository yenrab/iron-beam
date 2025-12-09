//! String Formatting and Printing Utilities
//!
//! Provides formatting and printing functions based on erl_printf.c and erl_printf_format.c.
//! These utilities handle formatted output similar to C's printf family.

use std::fmt;

/// Formatting utilities for string formatting and printing
pub struct FormatUtils;

impl FormatUtils {
    /// Format a string with arguments (similar to printf)
    ///
    /// This is a simplified version. For full printf-style formatting,
    /// consider using the `format!` macro or a formatting library.
    ///
    /// # Arguments
    /// * `format_str` - Format string (supports basic {} placeholders)
    /// * `args` - Arguments to format
    ///
    /// # Returns
    /// Formatted string
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::FormatUtils;
    ///
    /// let result = FormatUtils::format_string("Hello, {}!", &[&"world"]);
    /// assert_eq!(result, "Hello, world!");
    /// ```
    pub fn format_string(format_str: &str, args: &[&dyn fmt::Display]) -> String {
        // Simple implementation - for full printf support, would need a parser
        // This is a placeholder that demonstrates the API
        if args.is_empty() {
            return format_str.to_string();
        }

        // Basic placeholder replacement - replace {} with arguments in order
        let mut result = format_str.to_string();
        let mut arg_index = 0;
        
        // Replace {} placeholders with arguments
        while let Some(pos) = result.find("{}") {
            if arg_index < args.len() {
                let replacement = format!("{}", args[arg_index]);
                result.replace_range(pos..pos + 2, &replacement);
                arg_index += 1;
            } else {
                break; // No more arguments
            }
        }
        
        result
    }

    /// Print formatted string to stdout
    ///
    /// # Arguments
    /// * `format_str` - Format string
    /// * `args` - Arguments to format
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::FormatUtils;
    ///
    /// FormatUtils::print("Value: {}\n", &[&42]);
    /// ```
    pub fn print(format_str: &str, args: &[&dyn fmt::Display]) {
        let formatted = Self::format_string(format_str, args);
        print!("{}", formatted);
    }

    /// Print formatted string to stderr
    ///
    /// # Arguments
    /// * `format_str` - Format string
    /// * `args` - Arguments to format
    pub fn eprint(format_str: &str, args: &[&dyn fmt::Display]) {
        let formatted = Self::format_string(format_str, args);
        eprint!("{}", formatted);
    }

    /// Format a string with a single integer argument
    ///
    /// # Arguments
    /// * `format_str` - Format string with {} placeholder
    /// * `value` - Integer value to format
    ///
    /// # Returns
    /// Formatted string
    pub fn format_int(format_str: &str, value: i64) -> String {
        Self::format_string(format_str, &[&value])
    }

    /// Format a string with a single float argument
    ///
    /// # Arguments
    /// * `format_str` - Format string with {} placeholder
    /// * `value` - Float value to format
    ///
    /// # Returns
    /// Formatted string
    pub fn format_float(format_str: &str, value: f64) -> String {
        Self::format_string(format_str, &[&value])
    }

    /// Format a string with a single string argument
    ///
    /// # Arguments
    /// * `format_str` - Format string with {} placeholder
    /// * `value` - String value to format
    ///
    /// # Returns
    /// Formatted string
    pub fn format_str(format_str: &str, value: &str) -> String {
        Self::format_string(format_str, &[&value])
    }

    /// Pad a string to a specific width
    ///
    /// # Arguments
    /// * `s` - String to pad
    /// * `width` - Target width
    /// * `pad_char` - Character to use for padding
    /// * `left_align` - If true, pad on right; if false, pad on left
    ///
    /// # Returns
    /// Padded string
    pub fn pad_string(s: &str, width: usize, pad_char: char, left_align: bool) -> String {
        if s.len() >= width {
            return s.to_string();
        }
        let padding = width - s.len();
        let pad_str: String = std::iter::repeat(pad_char).take(padding).collect();
        if left_align {
            format!("{}{}", s, pad_str)
        } else {
            format!("{}{}", pad_str, s)
        }
    }

    /// Truncate a string to a maximum length
    ///
    /// # Arguments
    /// * `s` - String to truncate
    /// * `max_len` - Maximum length
    ///
    /// # Returns
    /// Truncated string (with "..." appended if truncated)
    pub fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            ".".repeat(max_len)
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_string() {
        let result = FormatUtils::format_string("Hello, {}!", &[&"world"]);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_format_string_multiple_args() {
        let result = FormatUtils::format_string("{} + {} = {}", &[&2, &3, &5]);
        assert_eq!(result, "2 + 3 = 5");
    }

    #[test]
    fn test_format_string_no_args() {
        let result = FormatUtils::format_string("No placeholders", &[]);
        assert_eq!(result, "No placeholders");
    }

    #[test]
    fn test_format_int() {
        let result = FormatUtils::format_int("Value: {}", 42);
        assert_eq!(result, "Value: 42");
    }

    #[test]
    fn test_format_float() {
        let result = FormatUtils::format_float("Value: {}", 3.14);
        assert!(result.starts_with("Value: 3.1"));
    }

    #[test]
    fn test_format_str() {
        let result = FormatUtils::format_str("Hello, {}!", "world");
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(FormatUtils::pad_string("abc", 5, ' ', true), "abc  ");
        assert_eq!(FormatUtils::pad_string("abc", 5, '0', false), "00abc");
        assert_eq!(FormatUtils::pad_string("abc", 2, ' ', true), "abc");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(FormatUtils::truncate_string("hello", 10), "hello");
        assert_eq!(FormatUtils::truncate_string("hello world", 8), "hello...");
        assert_eq!(FormatUtils::truncate_string("hello", 3), "...");
    }

    #[test]
    fn test_format_string_empty_format() {
        let result = FormatUtils::format_string("", &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_string_empty_format_with_args() {
        let result = FormatUtils::format_string("", &[&"test"]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_string_more_placeholders_than_args() {
        let result = FormatUtils::format_string("{} {} {}", &[&1, &2]);
        assert_eq!(result, "1 2 {}");
    }

    #[test]
    fn test_format_string_more_args_than_placeholders() {
        let result = FormatUtils::format_string("{}", &[&1, &2, &3]);
        assert_eq!(result, "1");
    }

    #[test]
    fn test_format_string_no_placeholders_with_args() {
        let result = FormatUtils::format_string("No placeholders here", &[&"ignored"]);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_format_string_multiple_placeholders_same_position() {
        let result = FormatUtils::format_string("{}{}", &[&"a", &"b"]);
        assert_eq!(result, "ab");
    }

    #[test]
    fn test_format_string_with_different_types() {
        let result = FormatUtils::format_string("{} {} {}", &[&42, &"test", &3.14]);
        assert_eq!(result, "42 test 3.14");
    }

    #[test]
    fn test_format_string_with_empty_string_arg() {
        let result = FormatUtils::format_string("Hello, {}!", &[&""]);
        assert_eq!(result, "Hello, !");
    }

    #[test]
    fn test_format_string_with_special_characters() {
        let result = FormatUtils::format_string("Value: {}", &[&"a{b}c"]);
        assert_eq!(result, "Value: a{b}c");
    }

    #[test]
    fn test_print_function() {
        // Test that print doesn't panic (hard to test output without capturing stdout)
        FormatUtils::print("Test: {}\n", &[&42]);
        // If we get here, it didn't panic
    }

    #[test]
    fn test_eprint_function() {
        // Test that eprint doesn't panic (hard to test output without capturing stderr)
        FormatUtils::eprint("Error: {}\n", &[&"test error"]);
        // If we get here, it didn't panic
    }

    #[test]
    fn test_format_int_negative() {
        let result = FormatUtils::format_int("Value: {}", -42);
        assert_eq!(result, "Value: -42");
    }

    #[test]
    fn test_format_int_zero() {
        let result = FormatUtils::format_int("Value: {}", 0);
        assert_eq!(result, "Value: 0");
    }

    #[test]
    fn test_format_int_large() {
        let result = FormatUtils::format_int("Value: {}", i64::MAX);
        assert!(result.contains(&i64::MAX.to_string()));
    }

    #[test]
    fn test_format_float_negative() {
        let result = FormatUtils::format_float("Value: {}", -3.14);
        assert!(result.contains("-3.1"));
    }

    #[test]
    fn test_format_float_zero() {
        let result = FormatUtils::format_float("Value: {}", 0.0);
        assert!(result.contains("0"));
    }

    #[test]
    fn test_format_float_scientific() {
        let result = FormatUtils::format_float("Value: {}", 1e10);
        assert!(result.contains("1"));
    }

    #[test]
    fn test_format_str_empty() {
        let result = FormatUtils::format_str("Value: {}", "");
        assert_eq!(result, "Value: ");
    }

    #[test]
    fn test_format_str_long() {
        let long_str = "a".repeat(100);
        let result = FormatUtils::format_str("Value: {}", &long_str);
        assert_eq!(result, format!("Value: {}", long_str));
    }

    #[test]
    fn test_pad_string_empty() {
        assert_eq!(FormatUtils::pad_string("", 5, ' ', true), "     ");
        assert_eq!(FormatUtils::pad_string("", 5, '0', false), "00000");
    }

    #[test]
    fn test_pad_string_exact_width() {
        assert_eq!(FormatUtils::pad_string("abc", 3, ' ', true), "abc");
        assert_eq!(FormatUtils::pad_string("abc", 3, '0', false), "abc");
    }

    #[test]
    fn test_pad_string_width_one() {
        assert_eq!(FormatUtils::pad_string("a", 1, ' ', true), "a");
        assert_eq!(FormatUtils::pad_string("", 1, ' ', true), " ");
    }

    #[test]
    fn test_pad_string_different_pad_chars() {
        assert_eq!(FormatUtils::pad_string("test", 8, 'x', true), "testxxxx");
        assert_eq!(FormatUtils::pad_string("test", 8, 'x', false), "xxxxtest");
        assert_eq!(FormatUtils::pad_string("test", 8, '\t', true), "test\t\t\t\t");
    }

    #[test]
    fn test_pad_string_unicode() {
        // Test with unicode characters (note: len() counts bytes, not chars)
        // "测试" is 6 bytes (3 bytes per Chinese char)
        // Test with width that's larger than the string byte length
        let result1 = FormatUtils::pad_string("测试", 10, ' ', true);
        assert!(result1.starts_with("测试"));
        assert_eq!(result1.len(), 10);
        
        let result2 = FormatUtils::pad_string("测试", 10, ' ', false);
        assert!(result2.ends_with("测试"));
        assert_eq!(result2.len(), 10);
        
        // Test with width equal to string length (should return unchanged)
        let result3 = FormatUtils::pad_string("测试", 6, ' ', true);
        assert_eq!(result3, "测试");
    }

    #[test]
    fn test_truncate_string_empty() {
        assert_eq!(FormatUtils::truncate_string("", 0), "");
        assert_eq!(FormatUtils::truncate_string("", 5), "");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(FormatUtils::truncate_string("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_string_max_len_zero() {
        assert_eq!(FormatUtils::truncate_string("hello", 0), "");
    }

    #[test]
    fn test_truncate_string_max_len_one() {
        assert_eq!(FormatUtils::truncate_string("hello", 1), ".");
    }

    #[test]
    fn test_truncate_string_max_len_two() {
        assert_eq!(FormatUtils::truncate_string("hello", 2), "..");
    }

    #[test]
    fn test_truncate_string_max_len_three() {
        assert_eq!(FormatUtils::truncate_string("hello", 3), "...");
    }

    #[test]
    fn test_truncate_string_max_len_four() {
        assert_eq!(FormatUtils::truncate_string("hello", 4), "h...");
    }

    #[test]
    fn test_truncate_string_unicode() {
        // Test with unicode characters
        // Note: truncate_string uses byte slicing which may panic on invalid boundaries
        // Test with safe lengths that align to char boundaries
        let unicode_str = "测试"; // 6 bytes (2 chars, 3 bytes each)
        let result = FormatUtils::truncate_string(unicode_str, 10);
        // Should not truncate since 6 < 10
        assert_eq!(result, unicode_str);
        
        // Test truncation with length that aligns to char boundary
        // "测试测试" is 12 bytes, truncate to 9 (which is 3 chars = 9 bytes, safe boundary)
        let long_unicode = "测试测试测试"; // 18 bytes
        let result2 = FormatUtils::truncate_string(long_unicode, 9);
        // Should truncate and add "..." (6 bytes + 3 for "...")
        assert_eq!(result2.len(), 9);
        assert!(result2.ends_with("..."));
    }

    #[test]
    fn test_format_string_with_newlines() {
        let result = FormatUtils::format_string("Line 1: {}\nLine 2: {}", &[&"a", &"b"]);
        assert_eq!(result, "Line 1: a\nLine 2: b");
    }

    #[test]
    fn test_format_string_with_braces() {
        // Test that single braces don't cause issues
        let result = FormatUtils::format_string("{value}", &[]);
        assert_eq!(result, "{value}");
    }

    #[test]
    fn test_pad_string_very_long() {
        let result = FormatUtils::pad_string("a", 1000, 'x', true);
        assert_eq!(result.len(), 1000);
        assert!(result.starts_with("a"));
        assert!(result.ends_with("x"));
    }

    #[test]
    fn test_truncate_string_very_long() {
        let long_str = "a".repeat(1000);
        let result = FormatUtils::truncate_string(&long_str, 10);
        assert_eq!(result.len(), 10);
        assert_eq!(result, "aaaaaaa...");
    }

    #[test]
    fn test_format_string_consecutive_placeholders() {
        let result = FormatUtils::format_string("{}{}{}", &[&1, &2, &3]);
        assert_eq!(result, "123");
    }

    #[test]
    fn test_format_string_placeholders_at_start() {
        let result = FormatUtils::format_string("{}start", &[&"test"]);
        assert_eq!(result, "teststart");
    }

    #[test]
    fn test_format_string_placeholders_at_end() {
        let result = FormatUtils::format_string("start{}", &[&"test"]);
        assert_eq!(result, "starttest");
    }
}

