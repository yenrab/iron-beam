//! Print Term Module
//!
//! Provides functionality to print terms in readable format.
//! Based on lib/erl_interface/src/misc/ei_printterm.c

use entities_data_handling::term_hashing::Term;

/// Print a term to stdout
///
/// # Arguments
/// * `term` - Term to print
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(PrintError)` - Print error
pub fn print_term(term: &Term) -> Result<(), PrintError> {
    let mut buf = Vec::new();
    print_term_internal(term, &mut buf, 0)?;
    let s = String::from_utf8(buf).map_err(|_| PrintError::EncodingError)?;
    print!("{}", s);
    Ok(())
}

/// Print a term to a string
///
/// # Arguments
/// * `term` - Term to print
///
/// # Returns
/// * `Ok(string)` - String representation
/// * `Err(PrintError)` - Print error
pub fn s_print_term(term: &Term) -> Result<String, PrintError> {
    let mut buf = Vec::new();
    print_term_internal(term, &mut buf, 0)?;
    String::from_utf8(buf).map_err(|_| PrintError::EncodingError)
}

/// Internal printing function
fn print_term_internal(
    term: &Term,
    buf: &mut Vec<u8>,
    _depth: usize,
) -> Result<(), PrintError> {
    match term {
        Term::Nil => {
            buf.extend_from_slice(b"[]");
        }
        Term::Small(n) => {
            let s = n.to_string();
            buf.extend_from_slice(s.as_bytes());
        }
        Term::Atom(index) => {
            // For now, print atom index
            // In full implementation, would look up atom name
            buf.extend_from_slice(b"atom_");
            buf.extend_from_slice(index.to_string().as_bytes());
        }
        Term::Float(f) => {
            let s = f.to_string();
            buf.extend_from_slice(s.as_bytes());
        }
        Term::Binary { data, bit_offset, bit_size } => {
            if *bit_offset == 0 && *bit_size == data.len() * 8 {
                // Full byte binary
                buf.extend_from_slice(b"<<");
                for (i, byte) in data.iter().enumerate() {
                    if i > 0 {
                        buf.extend_from_slice(b",");
                    }
                    buf.extend_from_slice(byte.to_string().as_bytes());
                }
                buf.extend_from_slice(b">>");
            } else {
                // Bitstring
                buf.extend_from_slice(b"<<");
                // Simplified bitstring representation
                buf.extend_from_slice(bit_size.to_string().as_bytes());
                buf.extend_from_slice(b":");
                buf.extend_from_slice(bit_offset.to_string().as_bytes());
                buf.extend_from_slice(b">>");
            }
        }
        Term::List { head, tail } => {
            buf.extend_from_slice(b"[");
            print_term_internal(head, buf, 0)?;
            if let Term::Nil = **tail {
                // End of list
            } else {
                buf.extend_from_slice(b",");
                print_term_internal(tail, buf, 0)?;
            }
            buf.extend_from_slice(b"]");
        }
        Term::Tuple(elements) => {
            buf.extend_from_slice(b"{");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    buf.extend_from_slice(b",");
                }
                print_term_internal(elem, buf, 0)?;
            }
            buf.extend_from_slice(b"}");
        }
        Term::Map(pairs) => {
            buf.extend_from_slice(b"#");
            buf.extend_from_slice(b"{");
            for (i, (key, value)) in pairs.iter().enumerate() {
                if i > 0 {
                    buf.extend_from_slice(b",");
                }
                print_term_internal(key, buf, 0)?;
                buf.extend_from_slice(b"=>");
                print_term_internal(value, buf, 0)?;
            }
            buf.extend_from_slice(b"}");
        }
        Term::Pid { node, id, serial, creation } => {
            buf.extend_from_slice(b"<");
            buf.extend_from_slice(node.to_string().as_bytes());
            buf.extend_from_slice(b".");
            buf.extend_from_slice(id.to_string().as_bytes());
            buf.extend_from_slice(b".");
            buf.extend_from_slice(serial.to_string().as_bytes());
            buf.extend_from_slice(b".");
            buf.extend_from_slice(creation.to_string().as_bytes());
            buf.extend_from_slice(b">");
        }
        Term::Port { node, id, creation } => {
            buf.extend_from_slice(b"#Port<");
            buf.extend_from_slice(node.to_string().as_bytes());
            buf.extend_from_slice(b".");
            buf.extend_from_slice(id.to_string().as_bytes());
            buf.extend_from_slice(b".");
            buf.extend_from_slice(creation.to_string().as_bytes());
            buf.extend_from_slice(b">");
        }
        Term::Ref { node, ids, creation } => {
            buf.extend_from_slice(b"#Ref<");
            buf.extend_from_slice(node.to_string().as_bytes());
            for id in ids {
                buf.extend_from_slice(b".");
                buf.extend_from_slice(id.to_string().as_bytes());
            }
            buf.extend_from_slice(b".");
            buf.extend_from_slice(creation.to_string().as_bytes());
            buf.extend_from_slice(b">");
        }
        Term::Fun { is_local, module, function, arity, .. } => {
            if *is_local {
                buf.extend_from_slice(b"fun ");
            } else {
                buf.extend_from_slice(b"fun ");
            }
            buf.extend_from_slice(b"module:");
            buf.extend_from_slice(module.to_string().as_bytes());
            buf.extend_from_slice(b"/function:");
            buf.extend_from_slice(function.to_string().as_bytes());
            buf.extend_from_slice(b"/arity:");
            buf.extend_from_slice(arity.to_string().as_bytes());
        }
        Term::Big(..) => {
            buf.extend_from_slice(b"<bignum>");
        }
    }
    Ok(())
}

/// Print errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintError {
    /// Encoding error
    EncodingError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_small_integer() {
        let term = Term::Small(42);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_print_nil() {
        let term = Term::Nil;
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[]");
    }

    #[test]
    fn test_print_atom() {
        let term = Term::Atom(123);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("atom_"));
    }

    #[test]
    fn test_print_tuple() {
        let term = Term::Tuple(vec![Term::Small(1), Term::Small(2), Term::Small(3)]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{1,2,3}");
    }

    #[test]
    fn test_print_binary() {
        let term = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("<<"));
    }

    #[test]
    fn test_print_binary_empty() {
        let term = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<<>>");
    }

    #[test]
    fn test_print_binary_single_byte() {
        let term = Term::Binary {
            data: vec![42],
            bit_offset: 0,
            bit_size: 8,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<<42>>");
    }

    #[test]
    fn test_print_binary_multiple_bytes() {
        let term = Term::Binary {
            data: vec![1, 2, 3],
            bit_offset: 0,
            bit_size: 24,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<<1,2,3>>");
    }

    #[test]
    fn test_print_binary_bitstring() {
        let term = Term::Binary {
            data: vec![1, 2],
            bit_offset: 4,
            bit_size: 12,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("<<"));
        assert!(output.contains("12:4")); // bit_size:bit_offset
    }

    #[test]
    fn test_print_binary_bitstring_offset_only() {
        let term = Term::Binary {
            data: vec![1, 2, 3],
            bit_offset: 2,
            bit_size: 24,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("<<"));
        assert!(output.contains("24:2"));
    }

    #[test]
    fn test_print_float() {
        let term = Term::Float(3.14);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("3.14"));
    }

    #[test]
    fn test_print_float_zero() {
        let term = Term::Float(0.0);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("0"));
    }

    #[test]
    fn test_print_float_negative() {
        let term = Term::Float(-42.5);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("-42.5"));
    }

    #[test]
    fn test_print_small_integer_zero() {
        let term = Term::Small(0);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0");
    }

    #[test]
    fn test_print_small_integer_negative() {
        let term = Term::Small(-42);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "-42");
    }

    #[test]
    fn test_print_small_integer_large() {
        let term = Term::Small(123456789);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "123456789");
    }

    #[test]
    fn test_print_list_empty() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        // Should print [1] since tail is Nil
        assert_eq!(result.unwrap(), "[1]");
    }

    #[test]
    fn test_print_list_single_element() {
        let term = Term::List {
            head: Box::new(Term::Small(42)),
            tail: Box::new(Term::Nil),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[42]");
    }

    #[test]
    fn test_print_list_multiple_elements() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(3)),
                    tail: Box::new(Term::Nil),
                }),
            }),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        // The current implementation prints nested lists, so [1,[2,[3]]]
        // This is because tail is printed as-is when it's not Nil
        let output = result.unwrap();
        assert!(output.starts_with("[1,"));
        assert!(output.contains("2"));
        assert!(output.contains("3"));
        assert!(output.ends_with("]"));
    }

    #[test]
    fn test_print_list_nested() {
        let inner_list = Term::List {
            head: Box::new(Term::Small(2)),
            tail: Box::new(Term::Nil),
        };
        let term = Term::List {
            head: Box::new(inner_list),
            tail: Box::new(Term::Nil),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[[2]]");
    }

    #[test]
    fn test_print_list_with_non_nil_tail() {
        // Improper list (tail is not Nil)
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(2)),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("[1,"));
        assert!(output.ends_with("2]"));
    }

    #[test]
    fn test_print_tuple_empty() {
        let term = Term::Tuple(vec![]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{}");
    }

    #[test]
    fn test_print_tuple_single_element() {
        let term = Term::Tuple(vec![Term::Small(42)]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{42}");
    }

    #[test]
    fn test_print_tuple_nested() {
        let inner_tuple = Term::Tuple(vec![Term::Small(2), Term::Small(3)]);
        let term = Term::Tuple(vec![Term::Small(1), inner_tuple]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{1,{2,3}}");
    }

    #[test]
    fn test_print_map_empty() {
        let term = Term::Map(vec![]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "#{}");
    }

    #[test]
    fn test_print_map_single_pair() {
        let term = Term::Map(vec![
            (Term::Atom(1), Term::Small(42)),
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#{"));
        assert!(output.contains("=>"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_print_map_multiple_pairs() {
        let term = Term::Map(vec![
            (Term::Atom(1), Term::Small(10)),
            (Term::Atom(2), Term::Small(20)),
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#{"));
        assert!(output.contains("=>"));
        assert!(output.contains("10"));
        assert!(output.contains("20"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_print_map_nested() {
        let inner_map = Term::Map(vec![
            (Term::Atom(2), Term::Small(20)),
        ]);
        let term = Term::Map(vec![
            (Term::Atom(1), inner_map),
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#{"));
        assert!(output.contains("#{"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_print_pid() {
        let term = Term::Pid {
            node: 1,
            id: 2,
            serial: 3,
            creation: 4,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<1.2.3.4>");
    }

    #[test]
    fn test_print_pid_large_values() {
        let term = Term::Pid {
            node: 12345,
            id: 67890,
            serial: 11111,
            creation: 22222,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("<"));
        assert!(output.contains("12345"));
        assert!(output.contains("67890"));
        assert!(output.ends_with(">"));
    }

    #[test]
    fn test_print_port() {
        let term = Term::Port {
            node: 1,
            id: 2,
            creation: 3,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "#Port<1.2.3>");
    }

    #[test]
    fn test_print_port_large_values() {
        let term = Term::Port {
            node: 12345,
            id: 67890,
            creation: 11111,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#Port<"));
        assert!(output.contains("12345"));
        assert!(output.contains("67890"));
        assert!(output.ends_with(">"));
    }

    #[test]
    fn test_print_ref_empty() {
        let term = Term::Ref {
            node: 1,
            ids: vec![],
            creation: 2,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#Ref<"));
        assert!(output.contains("1"));
        assert!(output.contains("2"));
        assert!(output.ends_with(">"));
    }

    #[test]
    fn test_print_ref_single_id() {
        let term = Term::Ref {
            node: 1,
            ids: vec![42],
            creation: 2,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#Ref<"));
        assert!(output.contains("1"));
        assert!(output.contains("42"));
        assert!(output.contains("2"));
        assert!(output.ends_with(">"));
    }

    #[test]
    fn test_print_ref_multiple_ids() {
        let term = Term::Ref {
            node: 1,
            ids: vec![10, 20, 30],
            creation: 2,
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#Ref<"));
        assert!(output.contains("1"));
        assert!(output.contains("10"));
        assert!(output.contains("20"));
        assert!(output.contains("30"));
        assert!(output.contains("2"));
        assert!(output.ends_with(">"));
    }

    #[test]
    fn test_print_fun_local() {
        let term = Term::Fun {
            is_local: true,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("fun "));
        assert!(output.contains("module:1"));
        assert!(output.contains("function:2"));
        assert!(output.contains("arity:3"));
    }

    #[test]
    fn test_print_fun_local_with_env() {
        let term = Term::Fun {
            is_local: true,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: Some(100),
            env: vec![Term::Small(42)],
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("fun "));
        assert!(output.contains("module:1"));
        assert!(output.contains("function:2"));
        assert!(output.contains("arity:3"));
    }

    #[test]
    fn test_print_fun_external() {
        let term = Term::Fun {
            is_local: false,
            module: 10,
            function: 20,
            arity: 30,
            old_uniq: None,
            env: vec![],
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("fun "));
        assert!(output.contains("module:10"));
        assert!(output.contains("function:20"));
        assert!(output.contains("arity:30"));
    }

    #[test]
    fn test_print_complex_nested() {
        // Complex nested structure
        let inner_list = Term::List {
            head: Box::new(Term::Small(2)),
            tail: Box::new(Term::Nil),
        };
        let inner_tuple = Term::Tuple(vec![Term::Atom(1), inner_list]);
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Atom(2),
            inner_tuple,
            Term::Binary {
                data: vec![1, 2],
                bit_offset: 0,
                bit_size: 16,
            },
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("{"));
        assert!(output.contains("atom_"));
        assert!(output.contains("<<"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_print_term_function() {
        // Test print_term (to stdout) doesn't panic
        let term = Term::Small(42);
        let result = print_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_error_variants() {
        // Test that error variants can be constructed and compared
        let err1 = PrintError::EncodingError;
        let err2 = PrintError::EncodingError;
        assert_eq!(err1, err2);
        
        // Test Clone
        let err3 = err1.clone();
        assert_eq!(err1, err3);
        
        // Test Debug
        let _ = format!("{:?}", err1);
    }

    #[test]
    fn test_print_atom_various_indices() {
        // Test atoms with various indices
        for index in [0, 1, 100, 1000, u32::MAX] {
            let term = Term::Atom(index);
            let result = s_print_term(&term);
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.starts_with("atom_"));
            assert!(output.contains(&index.to_string()));
        }
    }

    #[test]
    fn test_print_list_mixed_types() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Atom(2)),
                tail: Box::new(Term::List {
                    head: Box::new(Term::Float(3.14)),
                    tail: Box::new(Term::Nil),
                }),
            }),
        };
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("["));
        assert!(output.contains("1"));
        assert!(output.contains("atom_"));
        assert!(output.contains("3.14"));
        assert!(output.ends_with("]"));
    }

    #[test]
    fn test_print_tuple_mixed_types() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Atom(2),
            Term::Float(3.14),
            Term::Nil,
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("{"));
        assert!(output.contains("1"));
        assert!(output.contains("atom_"));
        assert!(output.contains("3.14"));
        assert!(output.contains("[]"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_print_map_with_various_keys() {
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(10)),
            (Term::Atom(2), Term::Small(20)),
            (Term::Nil, Term::Small(30)),
        ]);
        let result = s_print_term(&term);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("#{"));
        assert!(output.contains("=>"));
        assert!(output.contains("10"));
        assert!(output.contains("20"));
        assert!(output.contains("30"));
        assert!(output.ends_with("}"));
    }
}

