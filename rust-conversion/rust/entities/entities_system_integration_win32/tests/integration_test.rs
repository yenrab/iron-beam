//! Integration tests for entities_system_integration_win32 crate
//!
//! These tests verify Windows DOS mapping operations end-to-end.
//! Note: These tests only run on Windows platforms.

#[cfg(windows)]
use entities_system_integration_win32::*;

#[test]
#[cfg(windows)]
fn test_dos_map_error_code_ranges() {
    // Test mapping various error code ranges
    use dosmap::{DosMap, PosixErrno};
    
    // Test low error codes (0-50)
    assert_eq!(DosMap::map_error(0), PosixErrno::EINVAL as i32);
    assert_eq!(DosMap::map_error(2), PosixErrno::ENOENT as i32);
    assert_eq!(DosMap::map_error(5), PosixErrno::EACCES as i32);
    assert_eq!(DosMap::map_error(6), PosixErrno::EBADF as i32);
    assert_eq!(DosMap::map_error(8), PosixErrno::ENOMEM as i32);
    
    // Test middle range (50-150)
    assert_eq!(DosMap::map_error(80), PosixErrno::EEXIST as i32);
    assert_eq!(DosMap::map_error(89), PosixErrno::EAGAIN as i32);
    assert_eq!(DosMap::map_error(109), PosixErrno::EPIPE as i32);
    assert_eq!(DosMap::map_error(112), PosixErrno::ENOSPC as i32);
    
    // Test high range (150-215)
    assert_eq!(DosMap::map_error(145), PosixErrno::ENOTEMPTY as i32);
    assert_eq!(DosMap::map_error(183), PosixErrno::EEXIST as i32);
    assert_eq!(DosMap::map_error(215), PosixErrno::EAGAIN as i32);
}

#[test]
#[cfg(windows)]
fn test_dos_map_special_cases() {
    // Test special case error codes
    use dosmap::{DosMap, PosixErrno, win_error};
    
    // Test ERROR_NOT_ENOUGH_QUOTA (1816) - special case
    assert_eq!(
        DosMap::map_error(win_error::ERROR_NOT_ENOUGH_QUOTA),
        PosixErrno::ENOMEM as i32
    );
    
    // Test out-of-range errors map to EINVAL
    assert_eq!(DosMap::map_error(1000), PosixErrno::EINVAL as i32);
    assert_eq!(DosMap::map_error(5000), PosixErrno::EINVAL as i32);
    assert_eq!(DosMap::map_error(u32::MAX), PosixErrno::EINVAL as i32);
}

#[test]
#[cfg(windows)]
fn test_dos_map_consistency() {
    // Test that map_error and map_error_to_errno are consistent
    use dosmap::{DosMap, PosixErrno};
    
    let test_codes = vec![0, 2, 5, 6, 8, 80, 109, 112, 145, 183, 215];
    
    for code in test_codes {
        let errno_i32 = DosMap::map_error(code);
        let errno_enum = DosMap::map_error_to_errno(code);
        
        // They should match
        assert_eq!(errno_i32, errno_enum as i32);
    }
}

#[test]
#[cfg(windows)]
fn test_dos_map_error_types() {
    // Test mapping of different error types
    use dosmap::{DosMap, PosixErrno};
    
    // File system errors
    assert_eq!(DosMap::map_error(2), PosixErrno::ENOENT as i32); // FILE_NOT_FOUND
    assert_eq!(DosMap::map_error(3), PosixErrno::ENOENT as i32); // PATH_NOT_FOUND
    assert_eq!(DosMap::map_error(5), PosixErrno::EACCES as i32); // ACCESS_DENIED
    assert_eq!(DosMap::map_error(80), PosixErrno::EEXIST as i32); // FILE_EXISTS
    
    // Memory errors
    assert_eq!(DosMap::map_error(8), PosixErrno::ENOMEM as i32); // NOT_ENOUGH_MEMORY
    
    // I/O errors
    assert_eq!(DosMap::map_error(109), PosixErrno::EPIPE as i32); // BROKEN_PIPE
    assert_eq!(DosMap::map_error(112), PosixErrno::ENOSPC as i32); // DISK_FULL
    
    // Process errors
    assert_eq!(DosMap::map_error(128), PosixErrno::ECHILD as i32); // WAIT_NO_CHILDREN
    assert_eq!(DosMap::map_error(129), PosixErrno::ECHILD as i32); // CHILD_NOT_COMPLETE
}

#[test]
#[cfg(windows)]
fn test_dos_map_table_boundaries() {
    // Test boundaries of the error mapping table
    use dosmap::{DosMap, PosixErrno};
    
    // First entry
    assert_eq!(DosMap::map_error(0), PosixErrno::EINVAL as i32);
    
    // Last entry in table
    assert_eq!(DosMap::map_error(215), PosixErrno::EAGAIN as i32);
    
    // Just before out of range
    assert_eq!(DosMap::map_error(215), PosixErrno::EAGAIN as i32);
    
    // Just out of range (but not special case)
    assert_eq!(DosMap::map_error(216), PosixErrno::EINVAL as i32);
    assert_eq!(DosMap::map_error(217), PosixErrno::EINVAL as i32);
}

#[test]
#[cfg(windows)]
fn test_dos_map_posix_errno_values() {
    // Test that POSIX errno values are correct
    use dosmap::PosixErrno;
    
    // Verify some key errno values match standard POSIX values
    assert_eq!(PosixErrno::EINVAL as i32, 22);
    assert_eq!(PosixErrno::ENOENT as i32, 2);
    assert_eq!(PosixErrno::ENOMEM as i32, 12);
    assert_eq!(PosixErrno::EACCES as i32, 13);
    assert_eq!(PosixErrno::EBADF as i32, 9);
    assert_eq!(PosixErrno::EPIPE as i32, 32);
    assert_eq!(PosixErrno::ENOSPC as i32, 28);
    assert_eq!(PosixErrno::ECHILD as i32, 10);
    assert_eq!(PosixErrno::ENOTEMPTY as i32, 39);
    assert_eq!(PosixErrno::EEXIST as i32, 17);
    assert_eq!(PosixErrno::EAGAIN as i32, 11);
}

