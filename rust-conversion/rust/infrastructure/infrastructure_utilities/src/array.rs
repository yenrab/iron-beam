//! Array and Collection Utilities
//!
//! Provides utility functions for working with arrays, vectors, and collections.
//! These utilities handle common array/collection operations.

/// Array and collection utility functions
pub struct ArrayUtils;

impl ArrayUtils {
    /// Check if an array contains a value
    ///
    /// # Arguments
    /// * `arr` - Array to search
    /// * `value` - Value to find
    ///
    /// # Returns
    /// `true` if value is found
    pub fn contains<T: PartialEq>(arr: &[T], value: &T) -> bool {
        arr.contains(value)
    }

    /// Find the index of a value in an array
    ///
    /// # Arguments
    /// * `arr` - Array to search
    /// * `value` - Value to find
    ///
    /// # Returns
    /// * `Some(index)` - If value is found
    /// * `None` - If value is not found
    pub fn index_of<T: PartialEq>(arr: &[T], value: &T) -> Option<usize> {
        arr.iter().position(|x| x == value)
    }

    /// Reverse an array in place (returns new vector)
    ///
    /// # Arguments
    /// * `arr` - Array to reverse
    ///
    /// # Returns
    /// Reversed array
    pub fn reverse<T: Clone>(arr: &[T]) -> Vec<T> {
        let mut result: Vec<T> = arr.to_vec();
        result.reverse();
        result
    }

    /// Get the first element of an array
    ///
    /// # Arguments
    /// * `arr` - Array
    ///
    /// # Returns
    /// * `Some(element)` - First element
    /// * `None` - If array is empty
    pub fn first<T: Clone>(arr: &[T]) -> Option<T> {
        arr.first().cloned()
    }

    /// Get the last element of an array
    ///
    /// # Arguments
    /// * `arr` - Array
    ///
    /// # Returns
    /// * `Some(element)` - Last element
    /// * `None` - If array is empty
    pub fn last<T: Clone>(arr: &[T]) -> Option<T> {
        arr.last().cloned()
    }

    /// Get a slice of an array
    ///
    /// # Arguments
    /// * `arr` - Array
    /// * `start` - Start index (inclusive)
    /// * `end` - End index (exclusive)
    ///
    /// # Returns
    /// * `Some(slice)` - If indices are valid
    /// * `None` - If indices are out of bounds
    pub fn slice<T: Clone>(arr: &[T], start: usize, end: usize) -> Option<Vec<T>> {
        if start > end || end > arr.len() {
            return None;
        }
        Some(arr[start..end].to_vec())
    }

    /// Concatenate two arrays
    ///
    /// # Arguments
    /// * `arr1` - First array
    /// * `arr2` - Second array
    ///
    /// # Returns
    /// Concatenated array
    pub fn concat<T: Clone>(arr1: &[T], arr2: &[T]) -> Vec<T> {
        let mut result = arr1.to_vec();
        result.extend_from_slice(arr2);
        result
    }

    /// Remove duplicates from an array (preserving order)
    ///
    /// # Arguments
    /// * `arr` - Array with potential duplicates
    ///
    /// # Returns
    /// Array with duplicates removed
    pub fn unique<T: Clone + PartialEq>(arr: &[T]) -> Vec<T> {
        let mut result = Vec::new();
        for item in arr {
            if !result.contains(item) {
                result.push(item.clone());
            }
        }
        result
    }

    /// Filter an array based on a predicate
    ///
    /// # Arguments
    /// * `arr` - Array to filter
    /// * `predicate` - Function that returns true to keep element
    ///
    /// # Returns
    /// Filtered array
    pub fn filter<F, T: Clone>(arr: &[T], predicate: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        arr.iter().filter(|x| predicate(x)).cloned().collect()
    }

    /// Map an array to a new array
    ///
    /// # Arguments
    /// * `arr` - Array to map
    /// * `mapper` - Function to transform each element
    ///
    /// # Returns
    /// Mapped array
    pub fn map<F, T, U>(arr: &[T], mapper: F) -> Vec<U>
    where
        F: Fn(&T) -> U,
    {
        arr.iter().map(mapper).collect()
    }

    /// Sum all elements in an array of numbers
    ///
    /// # Arguments
    /// * `arr` - Array of numbers
    ///
    /// # Returns
    /// Sum of all elements
    pub fn sum(arr: &[i64]) -> i64 {
        arr.iter().sum()
    }

    /// Calculate the average of an array of numbers
    ///
    /// # Arguments
    /// * `arr` - Array of numbers
    ///
    /// # Returns
    /// * `Some(average)` - If array is not empty
    /// * `None` - If array is empty
    pub fn average(arr: &[i64]) -> Option<f64> {
        if arr.is_empty() {
            return None;
        }
        Some(arr.iter().sum::<i64>() as f64 / arr.len() as f64)
    }

    /// Find the maximum value in an array
    ///
    /// # Arguments
    /// * `arr` - Array of comparable values
    ///
    /// # Returns
    /// * `Some(max)` - Maximum value
    /// * `None` - If array is empty
    pub fn max<T: PartialOrd + Clone>(arr: &[T]) -> Option<T> {
        if arr.is_empty() {
            return None;
        }
        let mut max = &arr[0];
        for item in arr.iter().skip(1) {
            if item > max {
                max = item;
            }
        }
        Some(max.clone())
    }

    /// Find the minimum value in an array
    ///
    /// # Arguments
    /// * `arr` - Array of comparable values
    ///
    /// # Returns
    /// * `Some(min)` - Minimum value
    /// * `None` - If array is empty
    pub fn min<T: PartialOrd + Clone>(arr: &[T]) -> Option<T> {
        if arr.is_empty() {
            return None;
        }
        let mut min = &arr[0];
        for item in arr.iter().skip(1) {
            if item < min {
                min = item;
            }
        }
        Some(min.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let arr = [1, 2, 3, 4, 5];
        assert!(ArrayUtils::contains(&arr, &3));
        assert!(!ArrayUtils::contains(&arr, &6));
    }

    #[test]
    fn test_index_of() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(ArrayUtils::index_of(&arr, &3), Some(2));
        assert_eq!(ArrayUtils::index_of(&arr, &6), None);
    }

    #[test]
    fn test_reverse() {
        let arr = [1, 2, 3];
        let reversed = ArrayUtils::reverse(&arr);
        assert_eq!(reversed, vec![3, 2, 1]);
    }

    #[test]
    fn test_first_last() {
        let arr = [1, 2, 3];
        assert_eq!(ArrayUtils::first(&arr), Some(1));
        assert_eq!(ArrayUtils::last(&arr), Some(3));
        assert_eq!(ArrayUtils::first(&[] as &[i32]), None);
    }

    #[test]
    fn test_slice() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(ArrayUtils::slice(&arr, 1, 3), Some(vec![2, 3]));
        assert_eq!(ArrayUtils::slice(&arr, 0, 10), None);
    }

    #[test]
    fn test_concat() {
        let arr1 = [1, 2, 3];
        let arr2 = [4, 5, 6];
        let result = ArrayUtils::concat(&arr1, &arr2);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_unique() {
        let arr = [1, 2, 2, 3, 3, 3];
        let unique = ArrayUtils::unique(&arr);
        assert_eq!(unique, vec![1, 2, 3]);
    }

    #[test]
    fn test_filter() {
        let arr = [1, 2, 3, 4, 5];
        let filtered = ArrayUtils::filter(&arr, |x| *x % 2 == 0);
        assert_eq!(filtered, vec![2, 4]);
    }

    #[test]
    fn test_map() {
        let arr = [1, 2, 3];
        let mapped = ArrayUtils::map(&arr, |x| x * 2);
        assert_eq!(mapped, vec![2, 4, 6]);
    }

    #[test]
    fn test_sum() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(ArrayUtils::sum(&arr), 15);
    }

    #[test]
    fn test_average() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(ArrayUtils::average(&arr), Some(3.0));
        assert_eq!(ArrayUtils::average(&[] as &[i64]), None);
    }

    #[test]
    fn test_max_min() {
        let arr = [1, 5, 3, 2, 4];
        assert_eq!(ArrayUtils::max(&arr), Some(5));
        assert_eq!(ArrayUtils::min(&arr), Some(1));
        assert_eq!(ArrayUtils::max(&[] as &[i32]), None);
    }
}

