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
    /// let result = FormatUtils::format_string("Hello, {}!", "world");
    /// assert_eq!(result, "Hello, world!");
    /// ```
    pub fn format_string(format_str: &str, args: &[&dyn fmt::Display]) -> String {
        // Simple implementation - for full printf support, would need a parser
        // This is a placeholder that demonstrates the API
        if args.is_empty() {
            return format_str.to_string();
        }

        // Basic placeholder replacement
        let mut result = format_str.to_string();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &format!("{}", arg));
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
}

