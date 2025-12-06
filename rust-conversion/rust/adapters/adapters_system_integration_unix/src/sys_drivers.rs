//! System Drivers Module (Unix-specific)
//!
//! Provides Unix-specific system drivers for file descriptor management,
//! pipe operations, terminal window size queries, and file descriptor data management.
//! Based on sys_drivers.c

use std::os::unix::io::RawFd;

#[cfg(unix)]
use nix::{
    unistd::close,
    errno::Errno,
};

// Get terminal window size using direct ioctl (no File wrapper needed)
#[cfg(unix)]
fn get_winsize(fd: RawFd) -> Result<(u16, u16), Errno> {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        let result = libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws);
        if result == 0 {
            Ok((ws.ws_col, ws.ws_row))
        } else {
            Err(Errno::from_i32(std::io::Error::last_os_error().raw_os_error().unwrap_or(1)))
        }
    }
}

/// File descriptor data structure
///
/// Represents the state of a file descriptor for I/O operations.
/// Equivalent to `ErtsSysFdData` in the C implementation.
#[derive(Debug, Clone)]
pub struct FdData {
    /// File descriptor number
    pub fd: RawFd,
    /// Buffer for partial packet bytes
    pub pbuf: [u8; 4],
    /// Size of pbuf
    pub psz: usize,
    /// Main buffer
    pub buf: Option<Vec<u8>>,
    /// Current position in buffer (offset instead of raw pointer for safety)
    pub buf_offset: usize,
    /// Buffer size
    pub sz: usize,
    /// Remaining bytes to read
    pub remain: usize,
}

impl FdData {
    /// Create a new FdData with default values
    pub fn new(fd: RawFd) -> Self {
        Self {
            fd,
            pbuf: [0; 4],
            psz: 0,
            buf: None,
            buf_offset: 0,
            sz: 0,
            remain: 0,
        }
    }

    /// Get current position slice safely
    pub fn current_slice(&mut self) -> Option<&mut [u8]> {
        self.buf.as_mut().and_then(|buf| {
            if self.buf_offset < buf.len() {
                Some(&mut buf[self.buf_offset..])
            } else {
                None
            }
        })
    }

    /// Advance buffer position safely
    pub fn advance(&mut self, n: usize) {
        if let Some(ref buf) = self.buf {
            self.buf_offset = (self.buf_offset + n).min(buf.len());
        }
    }
}

/// Initialize file descriptor data
///
/// Equivalent to C's `init_fd_data`. Initializes an FdData structure
/// with the given file descriptor and default values.
///
/// # Arguments
///
/// * `fd_data` - Mutable reference to FdData to initialize
/// * `fd` - File descriptor number
pub fn init_fd_data(fd_data: &mut FdData, fd: RawFd) {
    fd_data.fd = fd;
    fd_data.buf = None;
    fd_data.buf_offset = 0;
    fd_data.remain = 0;
    fd_data.sz = 0;
    fd_data.psz = 0;
}

/// Close pipe file descriptors
///
/// Equivalent to C's `close_pipes`. Closes all file descriptors in
/// the input and output pipe arrays.
///
/// # Arguments
///
/// * `ifd` - Array of 2 input file descriptors [read, write]
/// * `ofd` - Array of 2 output file descriptors [read, write]
///
/// # Errors
///
/// Returns an error if any close operation fails.
#[cfg(unix)]
pub fn close_pipes(ifd: &[RawFd; 2], ofd: &[RawFd; 2]) -> Result<(), Errno> {
    close(ifd[0])?;
    close(ifd[1])?;
    close(ofd[0])?;
    close(ofd[1])?;
    Ok(())
}

#[cfg(not(unix))]
pub fn close_pipes(_ifd: &[RawFd; 2], _ofd: &[RawFd; 2]) -> Result<(), ()> {
    Ok(())
}

/// Get terminal window size
///
/// Equivalent to C's `fd_get_window_size`. Queries the terminal
/// window size using TIOCGWINSZ ioctl.
///
/// # Arguments
///
/// * `fd` - File descriptor (typically a TTY)
/// * `width` - Output parameter for window width in columns
/// * `height` - Output parameter for window height in rows
///
/// # Returns
///
/// Returns `Ok(())` if the window size was successfully retrieved,
/// `Err` otherwise.
#[cfg(unix)]
pub fn fd_get_window_size(fd: RawFd, width: &mut u32, height: &mut u32) -> Result<(), Errno> {
    let (cols, rows) = get_winsize(fd)?;
    *width = cols as u32;
    *height = rows as u32;
    Ok(())
}

#[cfg(not(unix))]
pub fn fd_get_window_size(_fd: RawFd, _width: &mut u32, _height: &mut u32) -> Result<(), ()> {
    Err(())
}

/// Clear file descriptor data
///
/// Equivalent to C's `clear_fd_data`. Frees any allocated buffers
/// and resets the FdData structure to its initial state.
///
/// # Arguments
///
/// * `fd_data` - Mutable reference to FdData to clear
pub fn clear_fd_data(fd_data: &mut FdData) {
    if fd_data.sz > 0 {
        // Free the buffer if it was allocated
        fd_data.buf = None;
    }
    fd_data.buf = None;
    fd_data.sz = 0;
    fd_data.remain = 0;
    fd_data.buf_offset = 0;
    fd_data.psz = 0;
}

/// Set file descriptor to blocking mode
///
/// Sets a file descriptor to blocking mode. This is a helper function
/// used by nbio_stop_fd.
///
/// # Arguments
///
/// * `fd` - File descriptor to set to blocking mode
///
/// # Errors
///
/// Returns an error if the fcntl operations fail.
#[cfg(unix)]
fn set_blocking(fd: RawFd) -> Result<(), Errno> {
    // Direct fcntl call (no File wrapper needed)
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags >= 0 {
            let _ = libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_blocking(_fd: RawFd) -> Result<(), ()> {
    Ok(())
}

/// Stop non-blocking I/O on a file descriptor
///
/// Equivalent to C's `nbio_stop_fd`. Clears the file descriptor data
/// and sets the file descriptor to blocking mode.
///
/// # Arguments
///
/// * `fd_data` - Mutable reference to FdData to stop
pub fn nbio_stop_fd(fd_data: &mut FdData) {
    clear_fd_data(fd_data);
    // Use absolute value of fd (C code uses abs() macro)
    let abs_fd = if fd_data.fd < 0 { -fd_data.fd } else { fd_data.fd };
    let _ = set_blocking(abs_fd); // Ignore errors on cleanup
}

/// Driver flush operation
///
/// Equivalent to C's `fd_flush`. Marks the driver as terminating
/// to ensure all data is flushed before shutdown.
///
/// # Arguments
///
/// * `terminating` - Mutable reference to terminating flag
pub fn fd_flush(terminating: &mut bool) {
    if !*terminating {
        *terminating = true;
    }
}

/// System drivers for Unix
pub struct SysDrivers;

impl SysDrivers {
    /// Initialize system drivers
    pub fn init() -> Result<(), DriverError> {
        // System drivers are initialized on first use
        Ok(())
    }
}

/// Driver operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// Initialization failed
    InitFailed,
    /// Invalid file descriptor
    InvalidFd,
    /// Operation not supported
    NotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_init_fd_data() {
        let mut fd_data = FdData::new(0);
        init_fd_data(&mut fd_data, 5);
        assert_eq!(fd_data.fd, 5);
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert!(fd_data.buf.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_clear_fd_data() {
        let mut fd_data = FdData::new(1);
        fd_data.sz = 100;
        fd_data.remain = 50;
        fd_data.psz = 2;
        
        clear_fd_data(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert!(fd_data.buf.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_fd_get_window_size() {
        // Test with stdin (may or may not be a TTY)
        let mut width = 0;
        let mut height = 0;
        let result = fd_get_window_size(0, &mut width, &mut height);
        // Result depends on whether stdin is a TTY, so we just check it doesn't panic
        // If it's a TTY, width and height should be set
        if result.is_ok() {
            assert!(width > 0);
            assert!(height > 0);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_fd_flush() {
        let mut terminating = false;
        fd_flush(&mut terminating);
        assert!(terminating);
        
        // Calling again should not change the value
        fd_flush(&mut terminating);
        assert!(terminating);
    }

    #[test]
    #[cfg(unix)]
    fn test_nbio_stop_fd() {
        let mut fd_data = FdData::new(1);
        fd_data.sz = 100;
        fd_data.remain = 50;
        
        nbio_stop_fd(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
    }

    #[test]
    #[cfg(unix)]
    fn test_sys_drivers() {
        let result = SysDrivers::init();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_fd_data_new() {
        let fd_data = FdData::new(42);
        assert_eq!(fd_data.fd, 42);
        assert_eq!(fd_data.pbuf, [0; 4]);
        assert_eq!(fd_data.psz, 0);
        assert!(fd_data.buf.is_none());
        assert_eq!(fd_data.buf_offset, 0);
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
    }
    
    #[test]
    fn test_fd_data_current_slice_with_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![1, 2, 3, 4, 5]);
        fd_data.buf_offset = 2;
        
        let slice = fd_data.current_slice();
        assert!(slice.is_some());
        assert_eq!(slice.unwrap(), &[3, 4, 5]);
    }
    
    #[test]
    fn test_fd_data_current_slice_no_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = None;
        
        let slice = fd_data.current_slice();
        assert!(slice.is_none());
    }
    
    #[test]
    fn test_fd_data_current_slice_offset_at_end() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.buf_offset = 3; // At end
        
        let slice = fd_data.current_slice();
        assert!(slice.is_none());
    }
    
    #[test]
    fn test_fd_data_current_slice_offset_past_end() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.buf_offset = 5; // Past end
        
        let slice = fd_data.current_slice();
        assert!(slice.is_none());
    }
    
    #[test]
    fn test_fd_data_advance_with_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![0; 100]);
        fd_data.buf_offset = 10;
        
        fd_data.advance(20);
        assert_eq!(fd_data.buf_offset, 30);
    }
    
    #[test]
    fn test_fd_data_advance_no_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = None;
        fd_data.buf_offset = 10;
        
        fd_data.advance(20);
        // Should not panic, offset should remain unchanged
        assert_eq!(fd_data.buf_offset, 10);
    }
    
    #[test]
    fn test_fd_data_advance_past_end() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![0; 50]);
        fd_data.buf_offset = 40;
        
        fd_data.advance(20); // Would go past end
        assert_eq!(fd_data.buf_offset, 50); // Should clamp to buf.len()
    }
    
    #[test]
    fn test_fd_data_advance_to_exact_end() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![0; 100]);
        fd_data.buf_offset = 90;
        
        fd_data.advance(10);
        assert_eq!(fd_data.buf_offset, 100);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_init_fd_data_preserves_fd() {
        let mut fd_data = FdData::new(10);
        fd_data.sz = 100;
        fd_data.remain = 50;
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.buf_offset = 2;
        fd_data.psz = 3;
        
        init_fd_data(&mut fd_data, 15);
        
        assert_eq!(fd_data.fd, 15);
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert!(fd_data.buf.is_none());
        assert_eq!(fd_data.buf_offset, 0);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_clear_fd_data_with_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![1, 2, 3, 4, 5]);
        fd_data.sz = 100;
        fd_data.remain = 50;
        fd_data.buf_offset = 2;
        fd_data.psz = 3;
        fd_data.pbuf[0] = 10;
        fd_data.pbuf[1] = 20;
        fd_data.pbuf[2] = 30;
        
        clear_fd_data(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert_eq!(fd_data.buf_offset, 0);
        assert!(fd_data.buf.is_none());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_clear_fd_data_without_buf() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = None;
        fd_data.sz = 0;
        fd_data.remain = 0;
        
        clear_fd_data(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert!(fd_data.buf.is_none());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_fd_get_window_size_invalid_fd() {
        let mut width = 0;
        let mut height = 0;
        // Use an invalid file descriptor
        let result = fd_get_window_size(-1, &mut width, &mut height);
        // Should fail on invalid FD
        assert!(result.is_err());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_fd_get_window_size_with_tty() {
        use std::fs::OpenOptions;
        use std::os::fd::AsRawFd;
        
        // Try to open /dev/tty if available
        if let Ok(tty) = OpenOptions::new().read(true).write(true).open("/dev/tty") {
            let fd = tty.as_raw_fd();
            let mut width = 0;
            let mut height = 0;
            let result = fd_get_window_size(fd, &mut width, &mut height);
            if result.is_ok() {
                assert!(width > 0);
                assert!(height > 0);
            }
        }
    }
    
    #[test]
    #[cfg(unix)]
    fn test_fd_flush_false_to_true() {
        let mut terminating = false;
        fd_flush(&mut terminating);
        assert!(terminating);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_fd_flush_already_true() {
        let mut terminating = true;
        fd_flush(&mut terminating);
        assert!(terminating);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_nbio_stop_fd_with_negative_fd() {
        let mut fd_data = FdData::new(-5);
        fd_data.sz = 100;
        fd_data.remain = 50;
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.buf_offset = 1;
        fd_data.psz = 2;
        
        nbio_stop_fd(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert_eq!(fd_data.buf_offset, 0);
        assert!(fd_data.buf.is_none());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_nbio_stop_fd_with_positive_fd() {
        let mut fd_data = FdData::new(10);
        fd_data.sz = 200;
        fd_data.remain = 100;
        fd_data.buf = Some(vec![1, 2, 3, 4, 5]);
        fd_data.buf_offset = 3;
        
        nbio_stop_fd(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert_eq!(fd_data.buf_offset, 0);
        assert!(fd_data.buf.is_none());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_nbio_stop_fd_with_zero_fd() {
        let mut fd_data = FdData::new(0);
        fd_data.sz = 50;
        fd_data.remain = 25;
        
        nbio_stop_fd(&mut fd_data);
        
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_close_pipes_success() {
        use nix::unistd::pipe;
        
        // Create pipes
        let (read1, write1) = pipe().unwrap();
        let (read2, write2) = pipe().unwrap();
        
        let ifd = [read1, write1];
        let ofd = [read2, write2];
        
        // Close pipes - should succeed
        let result = close_pipes(&ifd, &ofd);
        assert!(result.is_ok());
    }
    
    #[test]
    #[cfg(unix)]
    fn test_close_pipes_already_closed() {
        use nix::unistd::pipe;
        
        // Create pipes
        let (read1, write1) = pipe().unwrap();
        let (read2, write2) = pipe().unwrap();
        
        let ifd = [read1, write1];
        let ofd = [read2, write2];
        
        // Close them once
        close_pipes(&ifd, &ofd).unwrap();
        
        // Try to close again - should fail
        let result = close_pipes(&ifd, &ofd);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_driver_error_variants() {
        let error1 = DriverError::InitFailed;
        let error2 = DriverError::InvalidFd;
        let error3 = DriverError::NotSupported;
        
        assert!(matches!(error1, DriverError::InitFailed));
        assert!(matches!(error2, DriverError::InvalidFd));
        assert!(matches!(error3, DriverError::NotSupported));
    }
    
    #[test]
    fn test_driver_error_debug() {
        let error = DriverError::InitFailed;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InitFailed"));
    }
    
    #[test]
    fn test_driver_error_clone() {
        let error = DriverError::InvalidFd;
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }
    
    #[test]
    fn test_driver_error_partial_eq() {
        let error1 = DriverError::InitFailed;
        let error2 = DriverError::InitFailed;
        let error3 = DriverError::InvalidFd;
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
    
    #[test]
    fn test_fd_data_debug() {
        let fd_data = FdData::new(42);
        let debug_str = format!("{:?}", fd_data);
        assert!(debug_str.contains("42"));
    }
    
    #[test]
    fn test_fd_data_clone() {
        let mut fd_data = FdData::new(10);
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.sz = 100;
        fd_data.remain = 50;
        fd_data.buf_offset = 1;
        fd_data.psz = 2;
        fd_data.pbuf[0] = 10;
        
        let cloned = fd_data.clone();
        
        assert_eq!(cloned.fd, 10);
        assert_eq!(cloned.sz, 100);
        assert_eq!(cloned.remain, 50);
        assert_eq!(cloned.buf_offset, 1);
        assert_eq!(cloned.psz, 2);
        assert_eq!(cloned.pbuf[0], 10);
        assert_eq!(cloned.buf, Some(vec![1, 2, 3]));
    }
    
    #[test]
    fn test_fd_data_with_large_buffer() {
        let mut fd_data = FdData::new(1);
        let large_buf = vec![0u8; 10000];
        fd_data.buf = Some(large_buf);
        fd_data.buf_offset = 5000;
        fd_data.sz = 10000;
        
        let slice = fd_data.current_slice();
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 5000);
    }
    
    #[test]
    fn test_fd_data_advance_large_amount() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![0; 1000]);
        fd_data.buf_offset = 0;
        
        fd_data.advance(500);
        assert_eq!(fd_data.buf_offset, 500);
        
        fd_data.advance(300);
        assert_eq!(fd_data.buf_offset, 800);
        
        fd_data.advance(300); // Would exceed buffer
        assert_eq!(fd_data.buf_offset, 1000); // Clamped
    }
    
    #[test]
    #[cfg(unix)]
    fn test_init_fd_data_resets_all_fields() {
        let mut fd_data = FdData {
            fd: 99,
            pbuf: [1, 2, 3, 4],
            psz: 4,
            buf: Some(vec![1, 2, 3]),
            buf_offset: 2,
            sz: 100,
            remain: 50,
        };
        
        init_fd_data(&mut fd_data, 5);
        
        assert_eq!(fd_data.fd, 5);
        assert_eq!(fd_data.sz, 0);
        assert_eq!(fd_data.remain, 0);
        assert_eq!(fd_data.psz, 0);
        assert_eq!(fd_data.buf_offset, 0);
        assert!(fd_data.buf.is_none());
        // pbuf is not reset by init_fd_data
    }
    
    #[test]
    #[cfg(unix)]
    fn test_clear_fd_data_multiple_times() {
        let mut fd_data = FdData::new(1);
        fd_data.buf = Some(vec![1, 2, 3]);
        fd_data.sz = 100;
        
        clear_fd_data(&mut fd_data);
        clear_fd_data(&mut fd_data); // Should be idempotent
        
        assert_eq!(fd_data.sz, 0);
        assert!(fd_data.buf.is_none());
    }
}
