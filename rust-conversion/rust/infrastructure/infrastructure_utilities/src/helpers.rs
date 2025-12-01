//! Helper Functions Module
//!
//! Provides helper functions for common operations.
//! These are utility functions that don't fit into specific categories.

/// Helper functions
pub struct HelperFunctions;

impl HelperFunctions {
    /// Check if a value is Some and satisfies a predicate
    ///
    /// # Arguments
    /// * `opt` - Optional value
    /// * `predicate` - Function to test the value
    ///
    /// # Returns
    /// `true` if value is Some and predicate returns true
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::HelperFunctions;
    ///
    /// assert!(HelperFunctions::option_satisfies(Some(5), |x| *x > 0));
    /// assert!(!HelperFunctions::option_satisfies(Some(-1), |x| *x > 0));
    /// assert!(!HelperFunctions::option_satisfies(None::<i32>, |x| *x > 0));
    /// ```
    pub fn option_satisfies<T, F>(opt: Option<T>, predicate: F) -> bool
    where
        F: FnOnce(&T) -> bool,
    {
        opt.map_or(false, |v| predicate(&v))
    }

    /// Convert a Result to Option, discarding the error
    ///
    /// # Arguments
    /// * `result` - Result to convert
    ///
    /// # Returns
    /// Some(value) if Ok, None if Err
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::HelperFunctions;
    ///
    /// assert_eq!(HelperFunctions::result_to_option(Ok(42)), Some(42));
    /// assert_eq!(HelperFunctions::result_to_option(Err("error")), None);
    /// ```
    pub fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
        result.ok()
    }

    /// Apply a function to a value if it's Some
    ///
    /// # Arguments
    /// * `opt` - Optional value
    /// * `f` - Function to apply
    ///
    /// # Returns
    /// Some(f(value)) if Some, None if None
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::HelperFunctions;
    ///
    /// assert_eq!(HelperFunctions::option_map(Some(5), |x| x * 2), Some(10));
    /// assert_eq!(HelperFunctions::option_map(None::<i32>, |x| x * 2), None);
    /// ```
    pub fn option_map<T, U, F>(opt: Option<T>, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        opt.map(f)
    }

    /// Unwrap an Option or return a default value
    ///
    /// # Arguments
    /// * `opt` - Optional value
    /// * `default` - Default value
    ///
    /// # Returns
    /// Value if Some, default if None
    pub fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
        opt.unwrap_or(default)
    }

    /// Unwrap an Option or compute a default value
    ///
    /// # Arguments
    /// * `opt` - Optional value
    /// * `default_fn` - Function to compute default
    ///
    /// # Returns
    /// Value if Some, default_fn() if None
    pub fn unwrap_or_else<T, F>(opt: Option<T>, default_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        opt.unwrap_or_else(default_fn)
    }

    /// Convert a Result to Option, mapping the error
    ///
    /// # Arguments
    /// * `result` - Result to convert
    /// * `mapper` - Function to map error
    ///
    /// # Returns
    /// Some(value) if Ok, None if Err
    pub fn result_to_option_with<T, E, F>(result: Result<T, E>, _mapper: F) -> Option<T>
    where
        F: FnOnce(E) -> (),
    {
        result.ok()
    }

    /// Chain two Options
    ///
    /// # Arguments
    /// * `opt1` - First option
    /// * `opt2` - Second option
    ///
    /// # Returns
    /// opt1 if Some, otherwise opt2
    pub fn or<T>(opt1: Option<T>, opt2: Option<T>) -> Option<T> {
        opt1.or(opt2)
    }

    /// Chain two Options with a function
    ///
    /// # Arguments
    /// * `opt` - Option
    /// * `f` - Function that returns an Option
    ///
    /// # Returns
    /// Result of f(value) if Some, None if None
    pub fn and_then<T, U, F>(opt: Option<T>, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        opt.and_then(f)
    }

    /// Filter an Option based on a predicate
    ///
    /// # Arguments
    /// * `opt` - Option to filter
    /// * `predicate` - Function that returns true to keep value
    ///
    /// # Returns
    /// Some(value) if predicate returns true, None otherwise
    pub fn filter<T, F>(opt: Option<T>, predicate: F) -> Option<T>
    where
        F: FnOnce(&T) -> bool,
    {
        opt.filter(predicate)
    }

    /// Zip two Options together
    ///
    /// # Arguments
    /// * `opt1` - First option
    /// * `opt2` - Second option
    ///
    /// # Returns
    /// Some((a, b)) if both are Some, None otherwise
    pub fn zip<T, U>(opt1: Option<T>, opt2: Option<U>) -> Option<(T, U)> {
        opt1.zip(opt2)
    }

    /// Unzip an Option of a tuple
    ///
    /// # Arguments
    /// * `opt` - Option of tuple
    ///
    /// # Returns
    /// (Some(a), Some(b)) if Some, (None, None) if None
    pub fn unzip<T, U>(opt: Option<(T, U)>) -> (Option<T>, Option<U>) {
        opt.map_or((None, None), |(a, b)| (Some(a), Some(b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_satisfies() {
        assert!(HelperFunctions::option_satisfies(Some(5), |x| *x > 0));
        assert!(!HelperFunctions::option_satisfies(Some(-1), |x| *x > 0));
        assert!(!HelperFunctions::option_satisfies(None::<i32>, |x| *x > 0));
    }

    #[test]
    fn test_result_to_option() {
        assert_eq!(HelperFunctions::result_to_option::<i32, &str>(Ok(42)), Some(42));
        assert_eq!(HelperFunctions::result_to_option::<i32, &str>(Err("error")), None);
    }

    #[test]
    fn test_option_map() {
        assert_eq!(HelperFunctions::option_map(Some(5), |x| x * 2), Some(10));
        assert_eq!(HelperFunctions::option_map(None::<i32>, |x| x * 2), None);
    }

    #[test]
    fn test_unwrap_or() {
        assert_eq!(HelperFunctions::unwrap_or(Some(5), 10), 5);
        assert_eq!(HelperFunctions::unwrap_or(None::<i32>, 10), 10);
    }

    #[test]
    fn test_unwrap_or_else() {
        assert_eq!(HelperFunctions::unwrap_or_else(Some(5), || 10), 5);
        assert_eq!(HelperFunctions::unwrap_or_else(None::<i32>, || 10), 10);
    }

    #[test]
    fn test_or() {
        assert_eq!(HelperFunctions::or(Some(5), Some(10)), Some(5));
        assert_eq!(HelperFunctions::or(None::<i32>, Some(10)), Some(10));
    }

    #[test]
    fn test_and_then() {
        assert_eq!(HelperFunctions::and_then(Some(5), |x| Some(x * 2)), Some(10));
        assert_eq!(HelperFunctions::and_then(None::<i32>, |x| Some(x * 2)), None);
    }

    #[test]
    fn test_filter() {
        assert_eq!(HelperFunctions::filter(Some(5), |x| *x > 0), Some(5));
        assert_eq!(HelperFunctions::filter(Some(-1), |x| *x > 0), None);
    }

    #[test]
    fn test_zip() {
        assert_eq!(HelperFunctions::zip(Some(5), Some(10)), Some((5, 10)));
        assert_eq!(HelperFunctions::zip(Some(5), None::<i32>), None);
    }

    #[test]
    fn test_unzip() {
        let (a, b) = HelperFunctions::unzip(Some((5, 10)));
        assert_eq!(a, Some(5));
        assert_eq!(b, Some(10));
        
        let (a, b) = HelperFunctions::unzip(None::<(i32, i32)>);
        assert_eq!(a, None);
        assert_eq!(b, None);
    }
}

