//! Integration tests for infrastructure_utilities crate
//!
//! These tests verify that utility functions work correctly
//! and test end-to-end workflows for process table, atom table, and other utilities.

use infrastructure_utilities::*;
use entities_process::{Process, ProcessId};
use std::sync::Arc;

#[test]
fn test_process_table_creation() {
    let table = ProcessTable::new();
    assert_eq!(table.size(), 0);
    assert_eq!(table.max_size(), None);
}

#[test]
fn test_process_table_with_max_size() {
    let table = ProcessTable::with_max_size(1000);
    assert_eq!(table.size(), 0);
    assert_eq!(table.max_size(), Some(1000));
}

#[test]
fn test_process_table_insert_lookup() {
    let table = ProcessTable::new();
    let process = Arc::new(Process::new(1));
    
    // Insert process
    let previous = table.insert(1, Arc::clone(&process));
    assert!(previous.is_none()); // No previous process
    assert_eq!(table.size(), 1);
    
    // Lookup process
    let found = table.lookup(1);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), 1);
}

#[test]
fn test_process_table_remove() {
    let table = ProcessTable::new();
    let process = Arc::new(Process::new(2));
    
    // Insert and remove
    let previous = table.insert(2, Arc::clone(&process));
    assert!(previous.is_none());
    assert_eq!(table.size(), 1);
    
    let removed = table.remove(2);
    assert!(removed.is_some());
    assert_eq!(table.size(), 0);
    
    // Try to remove again
    let removed_again = table.remove(2);
    assert!(removed_again.is_none());
}

#[test]
fn test_process_table_multiple_processes() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=10 {
        let process = Arc::new(Process::new(i));
        table.insert(i, process);
    }
    
    assert_eq!(table.size(), 10);
    
    // Lookup all processes
    for i in 1..=10 {
        let found = table.lookup(i);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), i);
    }
}

#[test]
fn test_process_table_lookup_nonexistent() {
    let table = ProcessTable::new();
    
    // Lookup non-existent process
    let found = table.lookup(999);
    assert!(found.is_none());
}

#[test]
fn test_process_table_clear() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        table.insert(i, process);
    }
    
    assert_eq!(table.size(), 5);
    
    // Clear table
    table.clear();
    assert_eq!(table.size(), 0);
    
    // Verify all processes are gone
    for i in 1..=5 {
        assert!(table.lookup(i).is_none());
    }
}

#[test]
fn test_process_table_max_size_limit() {
    let table = ProcessTable::with_max_size(3);
    
    // Insert up to max size
    for i in 1..=3 {
        let process = Arc::new(Process::new(i));
        let previous = table.insert(i, process);
        assert!(previous.is_none());
    }
    
    assert_eq!(table.size(), 3);
    
    // Try to insert beyond max size - insert() doesn't check max_size, new_element() does
    // So we test new_element() instead
    let result = table.new_element(|_id| Arc::new(Process::new(0)));
    assert!(result.is_err());
    assert_eq!(table.size(), 3);
}

#[test]
fn test_get_global_process_table() {
    let table1 = get_global_process_table();
    let table2 = get_global_process_table();
    
    // Should return the same reference (singleton)
    assert!(std::ptr::eq(table1, table2));
}

#[test]
fn test_global_process_table_operations() {
    let table = get_global_process_table();
    
    // Insert a process
    let process = Arc::new(Process::new(100));
    let _previous = table.insert(100, Arc::clone(&process));
    
    // Lookup the process
    let found = table.lookup(100);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), 100);
    
    // Clean up
    table.remove(100);
}

#[test]
fn test_get_global_atom_table() {
    use entities_data_handling::AtomEncoding;
    
    let table1 = get_global_atom_table();
    let table2 = get_global_atom_table();
    
    // Should return the same reference (singleton)
    assert!(std::ptr::eq(table1, table2));
    
    // Test atom operations
    let index1 = table1.put_index(b"test_atom", AtomEncoding::Latin1, false).unwrap();
    let index2 = table2.put_index(b"test_atom", AtomEncoding::Latin1, false).unwrap();
    
    // Same atom should get same index
    assert_eq!(index1, index2);
}

#[test]
fn test_global_atom_table_operations() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Put atoms
    let index1 = table.put_index(b"atom1", AtomEncoding::Latin1, false).unwrap();
    let index2 = table.put_index(b"atom2", AtomEncoding::Latin1, false).unwrap();
    
    assert_ne!(index1, index2);
    
    // Get atom names
    let name1 = table.get_name(index1);
    assert!(name1.is_some());
    assert_eq!(name1.unwrap(), b"atom1");
    
    let name2 = table.get_name(index2);
    assert!(name2.is_some());
    assert_eq!(name2.unwrap(), b"atom2");
}

#[test]
fn test_process_table_error_cases() {
    let table = ProcessTable::with_max_size(1);
    
    // Insert first process using new_element (which checks max_size)
    let (id1, _process1) = table.new_element(|id| Arc::new(Process::new(id))).unwrap();
    assert_eq!(table.size(), 1);
    
    // Try to insert second process (should fail due to max size)
    let result = table.new_element(|id| Arc::new(Process::new(id)));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProcessTableError::TableFull);
    assert_eq!(table.size(), 1);
}

#[test]
fn test_process_table_concurrent_access() {
    use std::thread;
    use std::sync::Arc;
    
    let table = Arc::new(ProcessTable::new());
    
    // Spawn multiple threads inserting processes
    let mut handles = vec![];
    for i in 0..10 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            let process = Arc::new(Process::new(i));
            let _previous = table_clone.insert(i, process);
            i
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        let _ = handle.join().unwrap();
    }
    
    // Verify all processes were inserted
    assert_eq!(table.size(), 10);
    for i in 0..10 {
        assert!(table.lookup(i).is_some());
    }
}

#[test]
fn test_process_table_iteration() {
    let table = ProcessTable::new();
    
    // Insert multiple processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    // Get all IDs and verify count
    let all_ids = table.get_all_ids();
    assert_eq!(all_ids.len(), 5);
}

#[test]
fn test_atom_table_encoding_variants() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Test different encodings
    let index1 = table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
    let index2 = table.put_index(b"test", AtomEncoding::Latin1, false).unwrap();
    let index3 = table.put_index(b"test", AtomEncoding::Utf8, false).unwrap();
    
    // Same atom name with different encodings may or may not get same index
    // depending on implementation
    let _ = (index1, index2, index3);
}

#[test]
fn test_process_table_replace() {
    let table = ProcessTable::new();
    
    // Insert initial process (should return None for new entry)
    let process1 = Arc::new(Process::new(1));
    let first_insert = table.insert(1, Arc::clone(&process1));
    assert!(first_insert.is_none()); // First insert returns None
    
    // Replace with new process
    let process2 = Arc::new(Process::new(1));
    let old_process = table.insert(1, Arc::clone(&process2));
    
    // Should return old process (replacement)
    assert!(old_process.is_some());
    assert_eq!(table.size(), 1);
    
    // Lookup should return new process
    let found = table.lookup(1);
    assert!(found.is_some());
}

#[test]
fn test_process_table_size_after_operations() {
    let table = ProcessTable::new();
    
    assert_eq!(table.size(), 0);
    
    // Insert processes
    for i in 1..=5 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
        assert_eq!(table.size(), i as usize);
    }
    
    // Remove processes
    for i in 1..=5 {
        table.remove(i);
        assert_eq!(table.size(), (5 - i) as usize);
    }
}

#[test]
fn test_process_table_empty_after_clear() {
    let table = ProcessTable::new();
    
    // Insert processes
    for i in 1..=10 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    assert_eq!(table.size(), 10);
    assert!(!table.is_empty());
    
    // Clear
    table.clear();
    
    assert_eq!(table.size(), 0);
    assert!(table.is_empty());
}

#[test]
fn test_process_table_max_size_none() {
    let table = ProcessTable::new();
    assert_eq!(table.max_size(), None);
    
    // Should be able to insert many processes
    for i in 1..=100 {
        let process = Arc::new(Process::new(i));
        let _previous = table.insert(i, process);
    }
    
    assert_eq!(table.size(), 100);
}

#[test]
fn test_atom_table_get_nonexistent() {
    let table = get_global_atom_table();
    
    // Try to get non-existent atom
    let name = table.get_name(99999);
    assert!(name.is_none());
}

#[test]
fn test_atom_table_duplicate_atoms() {
    use entities_data_handling::AtomEncoding;
    
    let table = get_global_atom_table();
    
    // Put same atom multiple times
    let index1 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    let index2 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    let index3 = table.put_index(b"duplicate", AtomEncoding::Latin1, false).unwrap();
    
    // All should get the same index
    assert_eq!(index1, index2);
    assert_eq!(index2, index3);
}

#[test]
fn test_process_table_error_enum() {
    // Test ProcessTableError enum
    let error = ProcessTableError::TableFull;
    let _ = format!("{:?}", error);
}

// Integration tests for HelperFunctions
#[test]
fn test_helper_functions_integration() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test helper functions in a realistic workflow
    // Simulate processing optional process IDs
    let process_ids: Vec<Option<u64>> = vec![Some(1), None, Some(2), Some(3), None];
    
    // Use option_satisfies to filter valid process IDs
    let valid_ids: Vec<u64> = process_ids
        .into_iter()
        .filter_map(|opt| {
            if HelperFunctions::option_satisfies(opt, |&id| id > 0) {
                opt
            } else {
                None
            }
        })
        .collect();
    
    assert_eq!(valid_ids, vec![1, 2, 3]);
}

#[test]
fn test_helper_functions_result_handling() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test result handling in a realistic scenario
    let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("error1"), Ok(2), Err("error2"), Ok(3)];
    
    // Convert results to options, discarding errors
    let values: Vec<i32> = results
        .into_iter()
        .filter_map(|result| HelperFunctions::result_to_option(result))
        .collect();
    
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_helper_functions_option_chaining() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test option chaining in a realistic workflow
    let opt1 = Some(5);
    let opt2 = None::<i32>;
    let opt3 = Some(10);
    
    // Chain options: use first Some, fallback to second
    let result1 = HelperFunctions::or(opt1, opt2);
    assert_eq!(result1, Some(5));
    
    let result2 = HelperFunctions::or(opt2, opt3);
    assert_eq!(result2, Some(10));
    
    // Test and_then for chaining operations
    let doubled = HelperFunctions::and_then(Some(5), |x| {
        if x > 0 {
            Some(x * 2)
        } else {
            None
        }
    });
    assert_eq!(doubled, Some(10));
}

#[test]
fn test_helper_functions_zip_unzip_workflow() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test zip/unzip in a realistic pairing scenario
    let ids = Some(1);
    let names = Some("process1");
    
    // Zip together
    let paired = HelperFunctions::zip(ids, names);
    assert_eq!(paired, Some((1, "process1")));
    
    // Unzip back
    let (unzipped_id, unzipped_name) = HelperFunctions::unzip(paired);
    assert_eq!(unzipped_id, Some(1));
    assert_eq!(unzipped_name, Some("process1"));
    
    // Test with None
    let (id, name) = HelperFunctions::unzip(None::<(u64, &str)>);
    assert_eq!(id, None);
    assert_eq!(name, None);
}

#[test]
fn test_helper_functions_filtering_workflow() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test filtering in a realistic scenario
    let values = vec![Some(1), Some(-5), Some(10), Some(-2), Some(3)];
    
    // Filter positive values
    let positive: Vec<i32> = values
        .into_iter()
        .filter_map(|opt| HelperFunctions::filter(opt, |&x| x > 0))
        .collect();
    
    assert_eq!(positive, vec![1, 10, 3]);
}

#[test]
fn test_helper_functions_mapping_workflow() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test mapping in a realistic transformation scenario
    let process_ids = vec![Some(1), None, Some(2), Some(3)];
    
    // Map process IDs to doubled values
    let doubled: Vec<Option<i32>> = process_ids
        .into_iter()
        .map(|opt| HelperFunctions::option_map(opt, |x| x * 2))
        .collect();
    
    assert_eq!(doubled, vec![Some(2), None, Some(4), Some(6)]);
}

#[test]
fn test_helper_functions_default_values() {
    use infrastructure_utilities::helpers::HelperFunctions;
    
    // Test default value handling in a realistic scenario
    let config_value = Some(42);
    let default_config = 100;
    
    let result1 = HelperFunctions::unwrap_or(config_value, default_config);
    assert_eq!(result1, 42);
    
    let missing_value = None::<i32>;
    let result2 = HelperFunctions::unwrap_or(missing_value, default_config);
    assert_eq!(result2, 100);
    
    // Test with computed default
    let result3 = HelperFunctions::unwrap_or_else(None::<i32>, || {
        // Compute default based on some logic
        200
    });
    assert_eq!(result3, 200);
}

// Integration tests for MathUtils
#[test]
fn test_math_utils_integration() {
    use infrastructure_utilities::MathUtils;
    
    // Test arithmetic operations in a realistic workflow
    let values = vec![10, 20, 30, 40];
    
    // Calculate sum using checked_add
    let mut sum = 0;
    for &val in &values {
        if let Some(result) = MathUtils::checked_add(sum, val) {
            sum = result;
        }
    }
    assert_eq!(sum, 100);
    
    // Calculate product using checked_mul
    let mut product = 1;
    for &val in &values[..2] {
        if let Some(result) = MathUtils::checked_mul(product, val) {
            product = result;
        }
    }
    assert_eq!(product, 200);
    
    // Find max and min
    let max_val = values.iter().fold(values[0], |acc, &x| MathUtils::max(acc, x));
    let min_val = values.iter().fold(values[0], |acc, &x| MathUtils::min(acc, x));
    assert_eq!(max_val, 40);
    assert_eq!(min_val, 10);
}

#[test]
fn test_math_utils_division_workflow() {
    use infrastructure_utilities::MathUtils;
    
    // Test division and remainder in a realistic scenario
    let total = 100;
    let divisor = 7;
    
    if let Some((quotient, remainder)) = MathUtils::div_rem(total, divisor) {
        assert_eq!(quotient, 14);
        assert_eq!(remainder, 2);
        // Verify: quotient * divisor + remainder = total
        assert_eq!(quotient * divisor + remainder, total);
    }
    
    // Test with zero divisor
    assert_eq!(MathUtils::div_rem(total, 0), None);
}

#[test]
fn test_math_utils_power_calculations() {
    use infrastructure_utilities::MathUtils;
    
    // Test power calculations for different bases
    let bases = vec![2, 3, 5, 10];
    let exponent = 3;
    
    for base in bases {
        let result = MathUtils::pow(base, exponent);
        let expected = base * base * base;
        assert_eq!(result, expected);
    }
    
    // Test with exponent 0
    assert_eq!(MathUtils::pow(5, 0), 1);
    assert_eq!(MathUtils::pow(100, 0), 1);
}

#[test]
fn test_math_utils_gcd_lcm_workflow() {
    use infrastructure_utilities::MathUtils;
    
    // Test GCD and LCM in a realistic scenario
    let pairs = vec![(48, 18), (17, 13), (100, 25), (21, 14)];
    
    for (a, b) in pairs {
        let gcd = MathUtils::gcd(a, b);
        let lcm = MathUtils::lcm(a, b);
        
        // Verify: gcd * lcm = |a * b| (for non-zero values)
        if a != 0 && b != 0 {
            assert_eq!(gcd * lcm, (a * b).abs());
        }
    }
}

#[test]
fn test_math_utils_floating_point_operations() {
    use infrastructure_utilities::MathUtils;
    
    // Test floating point operations in a realistic workflow
    let values = vec![3.7, 4.2, 5.9, 2.1];
    
    // Test rounding operations
    let rounded: Vec<f64> = values.iter().map(|&x| MathUtils::round(x)).collect();
    assert_eq!(rounded, vec![4.0, 4.0, 6.0, 2.0]);
    
    // Test ceiling
    let ceiled: Vec<f64> = values.iter().map(|&x| MathUtils::ceil(x)).collect();
    assert_eq!(ceiled, vec![4.0, 5.0, 6.0, 3.0]);
    
    // Test floor
    let floored: Vec<f64> = values.iter().map(|&x| MathUtils::floor(x)).collect();
    assert_eq!(floored, vec![3.0, 4.0, 5.0, 2.0]);
    
    // Test sqrt
    let sqrt_values: Vec<f64> = vec![4.0, 9.0, 16.0, 25.0];
    for &val in &sqrt_values {
        let sqrt_result = MathUtils::sqrt(val);
        let expected = val.sqrt();
        assert!((sqrt_result - expected).abs() < 0.0001);
    }
}

#[test]
fn test_math_utils_trigonometric_functions() {
    use infrastructure_utilities::MathUtils;
    
    // Test trigonometric functions
    let angles = vec![0.0, std::f64::consts::PI / 4.0, std::f64::consts::PI / 2.0];
    
    for angle in angles {
        let sin_val = MathUtils::sin(angle);
        let cos_val = MathUtils::cos(angle);
        let tan_val = MathUtils::tan(angle);
        
        // Verify: sin^2 + cos^2 = 1
        let sum_squares = sin_val * sin_val + cos_val * cos_val;
        assert!((sum_squares - 1.0).abs() < 0.0001);
        
        // Verify: tan = sin / cos (when cos != 0)
        if cos_val.abs() > 0.0001 {
            let tan_calculated = sin_val / cos_val;
            assert!((tan_val - tan_calculated).abs() < 0.0001);
        }
    }
}

#[test]
fn test_rational_utils_integration() {
    use infrastructure_utilities::RationalUtils;
    
    // Test rational number operations in a realistic workflow
    let r1 = RationalUtils::new(1, 2).unwrap();
    let r2 = RationalUtils::new(1, 3).unwrap();
    
    // Add and verify
    let sum = RationalUtils::add(&r1, &r2);
    let expected_sum = RationalUtils::new(5, 6).unwrap();
    assert!((sum.to_f64() - expected_sum.to_f64()).abs() < 1e-10);
    
    // Multiply and verify
    let product = RationalUtils::multiply(&r1, &r2);
    let expected_product = RationalUtils::new(1, 6).unwrap();
    assert!((product.to_f64() - expected_product.to_f64()).abs() < 1e-10);
    
    // Divide and verify
    let quotient = RationalUtils::divide(&r1, &r2).unwrap();
    let expected_quotient = RationalUtils::new(3, 2).unwrap();
    assert!((quotient.to_f64() - expected_quotient.to_f64()).abs() < 1e-10);
}

#[test]
fn test_rational_utils_complex_calculations() {
    use infrastructure_utilities::RationalUtils;
    
    // Test complex rational number calculations
    let a = RationalUtils::new(1, 2).unwrap();
    let b = RationalUtils::new(1, 3).unwrap();
    let c = RationalUtils::new(1, 4).unwrap();
    
    // Calculate: (a + b) * c
    let sum_ab = RationalUtils::add(&a, &b);
    let result1 = RationalUtils::multiply(&sum_ab, &c);
    let expected1 = RationalUtils::new(5, 24).unwrap();
    assert!((result1.to_f64() - expected1.to_f64()).abs() < 1e-10);
    
    // Calculate: a / (b + c)
    let sum_bc = RationalUtils::add(&b, &c);
    let result2 = RationalUtils::divide(&a, &sum_bc).unwrap();
    let expected2 = RationalUtils::new(6, 7).unwrap();
    assert!((result2.to_f64() - expected2.to_f64()).abs() < 1e-10);
}

#[test]
fn test_rational_utils_comparison_workflow() {
    use infrastructure_utilities::RationalUtils;
    
    // Test comparison operations in a realistic scenario
    let values = vec![
        RationalUtils::new(1, 2).unwrap(),
        RationalUtils::new(1, 3).unwrap(),
        RationalUtils::new(3, 4).unwrap(),
        RationalUtils::new(1, 4).unwrap(),
    ];
    
    // Find maximum
    let max = values.iter().fold(&values[0], |acc, x| {
        if RationalUtils::compare(x, acc).is_gt() {
            x
        } else {
            acc
        }
    });
    assert!((max.to_f64() - 0.75).abs() < 1e-10);
    
    // Find minimum
    let min = values.iter().fold(&values[0], |acc, x| {
        if RationalUtils::compare(x, acc).is_lt() {
            x
        } else {
            acc
        }
    });
    assert!((min.to_f64() - 0.25).abs() < 1e-10);
}

