//! Helper functions for I/O operations
//!
//! Provides utility functions for I/O operations:
//! - Environment variable merging
//! - Argument conversion
//! - HTTP packet parsing helpers

use std::collections::HashMap;

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

/// Environment variable map
pub type Environment = HashMap<String, String>;

/// Merge global environment with key-value pairs (merge_global_environment)
///
/// Merges the global environment and the given key-value pairs into the
/// environment map. Keys with value 'false' or NIL are unset.
///
/// # Arguments
/// * `env` - Environment map to merge into
/// * `global_env` - Global environment to merge from
/// * `key_value_pairs` - List of (key, value) pairs to merge
///
/// # Returns
/// Ok(()) on success, Err on error
pub fn merge_global_environment(
    env: &mut Environment,
    global_env: &Environment,
    key_value_pairs: &[(String, Option<String>)],
) -> Result<(), HelperError> {
    // First, merge the global environment
    env.extend(global_env.iter().map(|(k, v)| (k.clone(), v.clone())));

    // Then, apply the key-value pairs
    for (key, value) in key_value_pairs {
        match value {
            Some(val) if val != "false" => {
                env.insert(key.clone(), val.clone());
            }
            _ => {
                // Unset the key (value is NIL or false)
                env.remove(key);
            }
        }
    }

    Ok(())
}

/// Convert arguments list to string vector (convert_args)
///
/// Converts a list of Erlang terms (strings/binaries) to a vector of
/// native strings. The first element is set to "default" if not provided.
///
/// # Arguments
/// * `args` - List of argument strings
///
/// # Returns
/// Vector of argument strings, with first element as "default"
pub fn convert_args(args: &[String]) -> Result<Vec<String>, HelperError> {
    if args.is_empty() {
        return Ok(vec!["default".to_string()]);
    }

    let mut result = vec!["default".to_string()];
    result.extend_from_slice(args);
    Ok(result)
}

/// Free arguments vector (free_args)
///
/// This is a no-op in Rust since Vec handles memory automatically.
/// Provided for API compatibility.
///
/// # Arguments
/// * `_args` - Arguments vector (ignored)
pub fn free_args(_args: Vec<String>) {
    // No-op in Rust - memory is automatically managed
}

/// HTTP packet parsing callback arguments
pub struct PacketCallbackArgs {
    /// Process ID
    pub process_id: u64,
    /// Result term
    pub result: Option<PacketResult>,
    /// String as binary flag
    pub string_as_bin: bool,
    /// Aligned pointer to binary data
    pub aligned_ptr: *const u8,
    /// Original binary
    pub original: Vec<u8>,
    /// Binary size
    pub bin_size: usize,
}

/// Packet parsing result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketResult {
    /// HTTP response
    HttpResponse {
        major: u8,
        minor: u8,
        status: u16,
        phrase: String,
    },
    /// HTTP request
    HttpRequest {
        method: String,
        uri: HttpUri,
        major: u8,
        minor: u8,
    },
    /// HTTP end of headers
    HttpEoh,
    /// HTTP header
    HttpHeader {
        bit: u8,
        name: String,
        original_name: String,
        value: String,
    },
    /// HTTP error
    HttpError {
        line: String,
    },
    /// SSL/TLS packet
    SslTls {
        content_type: u8,
        major: u8,
        minor: u8,
        data: Vec<u8>,
    },
}

/// HTTP URI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpUri {
    /// Absolute path
    AbsPath(String),
    /// Absolute URI
    AbsoluteUri {
        scheme: String,
        host: String,
        port: Option<u16>,
        path: String,
    },
    /// String URI
    String(String),
    /// Scheme URI
    Scheme {
        scheme: String,
        rest: String,
    },
    /// Star (OPTIONS *)
    Star,
}

/// Build HTTP string (http_bld_string)
///
/// Builds a string from packet data, either as a binary or as a string.
///
/// # Arguments
/// * `pca` - Packet callback arguments
/// * `str` - String data
/// * `len` - String length
///
/// # Returns
/// String or binary representation
pub fn http_bld_string(
    pca: &PacketCallbackArgs,
    str: &str,
    len: usize,
) -> Result<Vec<u8>, HelperError> {
    if pca.string_as_bin {
        // Return as binary
        Ok(str.as_bytes()[..len].to_vec())
    } else {
        // Return as string (UTF-8 encoded)
        Ok(str.as_bytes()[..len].to_vec())
    }
}

/// Build HTTP URI (http_bld_uri)
///
/// Builds an HTTP URI representation from parsed URI data.
///
/// # Arguments
/// * `pca` - Packet callback arguments
/// * `uri` - HTTP URI data
///
/// # Returns
/// HTTP URI representation
pub fn http_bld_uri(
    pca: &PacketCallbackArgs,
    uri: &HttpUri,
) -> Result<HttpUri, HelperError> {
    match uri {
        HttpUri::Star => Ok(HttpUri::Star),
        HttpUri::AbsPath(path) => {
            let s1 = String::from_utf8_lossy(
                &http_bld_string(pca, path, path.len())?
            ).to_string();
            Ok(HttpUri::AbsPath(s1))
        }
        HttpUri::AbsoluteUri {
            scheme,
            host,
            port,
            path,
        } => {
            let s1 = String::from_utf8_lossy(
                &http_bld_string(pca, host, host.len())?
            ).to_string();
            let s2 = String::from_utf8_lossy(
                &http_bld_string(pca, path, path.len())?
            ).to_string();
            Ok(HttpUri::AbsoluteUri {
                scheme: scheme.clone(),
                host: s1,
                port: *port,
                path: s2,
            })
        }
        HttpUri::String(s) => {
            let s1 = String::from_utf8_lossy(
                &http_bld_string(pca, s, s.len())?
            ).to_string();
            Ok(HttpUri::String(s1))
        }
        HttpUri::Scheme { scheme, rest } => {
            let s1 = String::from_utf8_lossy(
                &http_bld_string(pca, scheme, scheme.len())?
            ).to_string();
            let s2 = String::from_utf8_lossy(
                &http_bld_string(pca, rest, rest.len())?
            ).to_string();
            Ok(HttpUri::Scheme {
                scheme: s1,
                rest: s2,
            })
        }
    }
}

/// Helper errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperError {
    /// Invalid argument
    InvalidArgument,
    /// UTF-8 encoding error
    Utf8Error,
    /// Out of memory
    OutOfMemory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_global_environment() {
        let mut env = Environment::new();
        let global_env: Environment = [
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        let key_value_pairs = vec![
            ("PATH".to_string(), Some("/custom/path".to_string())),
            ("HOME".to_string(), None), // Unset
            ("NEW_VAR".to_string(), Some("new_value".to_string())),
        ];

        merge_global_environment(&mut env, &global_env, &key_value_pairs).unwrap();

        assert_eq!(env.get("PATH"), Some(&"/custom/path".to_string()));
        assert_eq!(env.get("HOME"), None); // Should be unset
        assert_eq!(env.get("NEW_VAR"), Some(&"new_value".to_string()));
    }

    #[test]
    fn test_convert_args() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let result = convert_args(&args).unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "default");
        assert_eq!(result[1], "arg1");
        assert_eq!(result[2], "arg2");
    }

    #[test]
    fn test_convert_args_empty() {
        let result = convert_args(&[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "default");
    }

    #[test]
    fn test_http_bld_string() {
        let pca = PacketCallbackArgs {
            process_id: 1,
            result: None,
            string_as_bin: false,
            aligned_ptr: std::ptr::null(),
            original: Vec::new(),
            bin_size: 0,
        };

        let result = http_bld_string(&pca, "hello", 5).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_http_bld_uri_star() {
        let pca = PacketCallbackArgs {
            process_id: 1,
            result: None,
            string_as_bin: false,
            aligned_ptr: std::ptr::null(),
            original: Vec::new(),
            bin_size: 0,
        };

        let uri = HttpUri::Star;
        let result = http_bld_uri(&pca, &uri).unwrap();
        assert_eq!(result, HttpUri::Star);
    }

    #[test]
    fn test_http_bld_uri_abs_path() {
        let pca = PacketCallbackArgs {
            process_id: 1,
            result: None,
            string_as_bin: false,
            aligned_ptr: std::ptr::null(),
            original: Vec::new(),
            bin_size: 0,
        };

        let uri = HttpUri::AbsPath("/path/to/resource".to_string());
        let result = http_bld_uri(&pca, &uri).unwrap();
        
        match result {
            HttpUri::AbsPath(path) => assert_eq!(path, "/path/to/resource"),
            _ => panic!("Expected AbsPath"),
        }
    }
}

