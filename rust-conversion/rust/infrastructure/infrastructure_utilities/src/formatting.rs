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
}

