//! Regular Expression BIF Module
//!
//! Provides regular expression built-in functions.
//! Based on erl_bif_re.c
//!
//! This module implements regex operations using the Rust `regex` crate.
//! Note: The C code uses PCRE2, but we use Rust's regex crate for safe Rust implementation.
//! For internal usecases, this provides equivalent functionality.

use regex::{Regex, RegexBuilder};
use std::sync::Arc;

/// Regular expression BIF operations
pub struct RegexBif;

impl RegexBif {
    /// Compile a regular expression pattern
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern string
    /// * `case_insensitive` - Whether to match case-insensitively
    /// * `multiline` - Whether ^ and $ match at line boundaries
    /// * `dot_matches_newline` - Whether . matches newline
    ///
    /// # Returns
    /// Compiled regex or error
    pub fn compile(
        pattern: &str,
        case_insensitive: bool,
        multiline: bool,
        dot_matches_newline: bool,
    ) -> Result<CompiledRegex, RegexError> {
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(case_insensitive);
        builder.multi_line(multiline);
        builder.dot_matches_new_line(dot_matches_newline);
        
        builder
            .build()
            .map(|re| CompiledRegex {
                regex: Arc::new(re),
            })
            .map_err(|e| RegexError::InvalidPattern(e.to_string()))
    }

    /// Compile a regular expression with default options
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern string
    ///
    /// # Returns
    /// Compiled regex or error
    pub fn compile_simple(pattern: &str) -> Result<CompiledRegex, RegexError> {
        Self::compile(pattern, false, false, false)
    }

    /// Get the regex library version
    ///
    /// # Returns
    /// Version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Run a regex match on text
    ///
    /// # Arguments
    /// * `regex` - Compiled regex
    /// * `text` - Text to search
    /// * `start_offset` - Starting offset in bytes
    ///
    /// # Returns
    /// Match result with captures
    pub fn run(
        regex: &CompiledRegex,
        text: &str,
        start_offset: usize,
    ) -> Result<MatchResult, RegexError> {
        if start_offset > text.len() {
            return Err(RegexError::InvalidOffset);
        }

        let search_text = &text[start_offset..];
        let regex_ref = regex.regex.as_ref();

        if let Some(captures) = regex_ref.captures(search_text) {
            let mut groups = Vec::new();
            
            // Full match (group 0)
            if let Some(m) = captures.get(0) {
                groups.push(Capture {
                    start: m.start() + start_offset,
                    end: m.end() + start_offset,
                    text: m.as_str().to_string(),
                });
            }

            // Named and numbered captures
            for i in 1..captures.len() {
                if let Some(m) = captures.get(i) {
                    groups.push(Capture {
                        start: m.start() + start_offset,
                        end: m.end() + start_offset,
                        text: m.as_str().to_string(),
                    });
                } else {
                    groups.push(Capture {
                        start: start_offset,
                        end: start_offset,
                        text: String::new(),
                    });
                }
            }

            Ok(MatchResult {
                matched: true,
                captures: groups,
            })
        } else {
            Ok(MatchResult {
                matched: false,
                captures: Vec::new(),
            })
        }
    }

    /// Find all matches in text (global match)
    ///
    /// # Arguments
    /// * `regex` - Compiled regex
    /// * `text` - Text to search
    ///
    /// # Returns
    /// Vector of match results
    pub fn find_all(regex: &CompiledRegex, text: &str) -> Vec<MatchResult> {
        let regex_ref = regex.regex.as_ref();
        let mut results = Vec::new();

        for captures in regex_ref.captures_iter(text) {
            let mut groups = Vec::new();
            
            if let Some(m) = captures.get(0) {
                groups.push(Capture {
                    start: m.start(),
                    end: m.end(),
                    text: m.as_str().to_string(),
                });
            }

            for i in 1..captures.len() {
                if let Some(m) = captures.get(i) {
                    groups.push(Capture {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    });
                } else {
                    groups.push(Capture {
                        start: 0,
                        end: 0,
                        text: String::new(),
                    });
                }
            }

            results.push(MatchResult {
                matched: true,
                captures: groups,
            });
        }

        results
    }
}

/// Compiled regular expression
pub struct CompiledRegex {
    regex: Arc<Regex>,
}

impl CompiledRegex {
    /// Get a reference to the underlying regex
    pub fn as_ref(&self) -> &Regex {
        self.regex.as_ref()
    }
}

/// Capture group information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Matched text
    pub text: String,
}

/// Match result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// Whether a match was found
    pub matched: bool,
    /// Capture groups (group 0 is full match, then numbered groups)
    pub captures: Vec<Capture>,
}

/// Regex operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexError {
    /// Invalid pattern
    InvalidPattern(String),
    /// Invalid offset
    InvalidOffset,
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            RegexError::InvalidOffset => write!(f, "Invalid offset"),
        }
    }
}

impl std::error::Error for RegexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let re = RegexBif::compile_simple(r"hello");
        assert!(re.is_ok());
    }

    #[test]
    fn test_compile_invalid() {
        let re = RegexBif::compile_simple(r"[invalid");
        assert!(re.is_err());
        if let Err(RegexError::InvalidPattern(_)) = re {
            // Expected
        } else {
            panic!("Expected InvalidPattern error");
        }
    }

    #[test]
    fn test_run_match() {
        let re = RegexBif::compile_simple(r"hello").unwrap();
        let result = RegexBif::run(&re, "hello world", 0).unwrap();
        assert!(result.matched);
        assert_eq!(result.captures.len(), 1);
        assert_eq!(result.captures[0].text, "hello");
    }

    #[test]
    fn test_run_no_match() {
        let re = RegexBif::compile_simple(r"goodbye").unwrap();
        let result = RegexBif::run(&re, "hello world", 0).unwrap();
        assert!(!result.matched);
    }

    #[test]
    fn test_run_with_captures() {
        let re = RegexBif::compile_simple(r"(\w+) (\w+)").unwrap();
        let result = RegexBif::run(&re, "hello world", 0).unwrap();
        assert!(result.matched);
        assert_eq!(result.captures.len(), 3); // Full match + 2 groups
        assert_eq!(result.captures[0].text, "hello world");
        assert_eq!(result.captures[1].text, "hello");
        assert_eq!(result.captures[2].text, "world");
    }

    #[test]
    fn test_run_with_offset() {
        let re = RegexBif::compile_simple(r"world").unwrap();
        let result = RegexBif::run(&re, "hello world", 6).unwrap();
        assert!(result.matched);
        assert_eq!(result.captures[0].start, 6);
        assert_eq!(result.captures[0].text, "world");
    }

    #[test]
    fn test_find_all() {
        let re = RegexBif::compile_simple(r"\d+").unwrap();
        let results = RegexBif::find_all(&re, "123 abc 456 def 789");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].captures[0].text, "123");
        assert_eq!(results[1].captures[0].text, "456");
        assert_eq!(results[2].captures[0].text, "789");
    }

    #[test]
    fn test_case_insensitive() {
        let re = RegexBif::compile("hello", true, false, false).unwrap();
        let result = RegexBif::run(&re, "HELLO world", 0).unwrap();
        assert!(result.matched);
    }

    #[test]
    fn test_multiline() {
        let re = RegexBif::compile("^hello", false, true, false).unwrap();
        let result = RegexBif::run(&re, "world\nhello", 0).unwrap();
        assert!(result.matched);
    }

    #[test]
    fn test_version() {
        let version = RegexBif::version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_run_invalid_offset() {
        let re = RegexBif::compile_simple(r"hello").unwrap();
        let result = RegexBif::run(&re, "hello", 10); // offset > text.len()
        assert!(result.is_err());
        if let Err(RegexError::InvalidOffset) = result {
            // Expected
        } else {
            panic!("Expected InvalidOffset error");
        }
    }

    #[test]
    fn test_error_display() {
        let err1 = RegexError::InvalidPattern("test error".to_string());
        let err2 = RegexError::InvalidOffset;
        let s1 = format!("{}", err1);
        let s2 = format!("{}", err2);
        assert!(s1.contains("Invalid pattern"));
        assert!(s1.contains("test error"));
        assert!(s2.contains("Invalid offset"));
    }

    #[test]
    fn test_error_trait() {
        use std::error::Error;
        let err = RegexError::InvalidPattern("test".to_string());
        let source = err.source();
        assert!(source.is_none()); // Our error doesn't have a source
        let _ = format!("{:?}", err);
    }

    #[test]
    fn test_run_with_optional_captures() {
        // Pattern with optional group that doesn't match
        let re = RegexBif::compile_simple(r"(\d+)?(hello)").unwrap();
        let result = RegexBif::run(&re, "hello", 0).unwrap();
        assert!(result.matched);
        // First group is optional and doesn't match, should hit else branch
        assert_eq!(result.captures.len(), 3); // Full match + 2 groups
        // First capture group should be empty (hits line 107-111)
        assert_eq!(result.captures[1].text, "");
        assert_eq!(result.captures[1].start, 0);
        assert_eq!(result.captures[2].text, "hello");
    }

    #[test]
    fn test_find_all_with_optional_captures() {
        let re = RegexBif::compile_simple(r"(\d+)?(hello)").unwrap();
        let results = RegexBif::find_all(&re, "hello world hello");
        assert_eq!(results.len(), 2);
        // Each result should have optional groups that don't match
        // This will hit the else branch at line 157-162
        for result in &results {
            assert!(result.matched);
            assert_eq!(result.captures.len(), 3);
            assert_eq!(result.captures[1].text, ""); // Optional group doesn't match
            assert_eq!(result.captures[2].text, "hello");
        }
    }

    #[test]
    fn test_compile_all_options() {
        // Test all 8 combinations of the 3 boolean options
        let _ = RegexBif::compile("test", false, false, false).unwrap();
        let _ = RegexBif::compile("test", true, false, false).unwrap();
        let _ = RegexBif::compile("test", false, true, false).unwrap();
        let _ = RegexBif::compile("test", false, false, true).unwrap();
        let _ = RegexBif::compile("test", true, true, false).unwrap();
        let _ = RegexBif::compile("test", true, false, true).unwrap();
        let _ = RegexBif::compile("test", false, true, true).unwrap();
        let _ = RegexBif::compile("test", true, true, true).unwrap();
    }

    #[test]
    fn test_run_offset_at_end() {
        let re = RegexBif::compile_simple(r"hello").unwrap();
        let result = RegexBif::run(&re, "hello", 5).unwrap(); // offset at end
        assert!(!result.matched);
        assert_eq!(result.captures.len(), 0);
    }

    #[test]
    fn test_run_empty_string() {
        let re = RegexBif::compile_simple(r"^$").unwrap();
        let result = RegexBif::run(&re, "", 0).unwrap();
        assert!(result.matched);
        assert_eq!(result.captures.len(), 1);
    }

    #[test]
    fn test_compiled_regex_as_ref() {
        let re = RegexBif::compile_simple(r"test").unwrap();
        let regex_ref = re.as_ref();
        assert!(regex_ref.is_match("test"));
        assert!(!regex_ref.is_match("no match"));
    }

    #[test]
    fn test_find_all_no_matches() {
        let re = RegexBif::compile_simple(r"\d+").unwrap();
        let results = RegexBif::find_all(&re, "abc def");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_error_display_special_chars() {
        let err = RegexError::InvalidPattern("error with \"quotes\" and\nnewlines".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Invalid pattern"));
        assert!(s.contains("error with"));
    }

    #[test]
    fn test_run_with_offset_boundary() {
        // Test offset exactly at text.len() (should be valid, just empty search)
        let re = RegexBif::compile_simple(r"hello").unwrap();
        let result = RegexBif::run(&re, "hello", 5).unwrap();
        assert!(!result.matched);
    }

    #[test]
    fn test_find_all_empty_text() {
        let re = RegexBif::compile_simple(r"\d+").unwrap();
        let results = RegexBif::find_all(&re, "");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_run_captures_with_offset() {
        // Test that capture offsets are correctly adjusted with start_offset
        let re = RegexBif::compile_simple(r"(\w+)").unwrap();
        let result = RegexBif::run(&re, "prefix hello", 7).unwrap();
        assert!(result.matched);
        assert_eq!(result.captures[0].start, 7);
        assert_eq!(result.captures[0].text, "hello");
    }
}
