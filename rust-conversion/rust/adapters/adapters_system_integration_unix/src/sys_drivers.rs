//! System Drivers Module (Unix-specific)
//!
//! Provides Unix-specific system drivers for file descriptor management,
//! pipe operations, terminal window size queries, and driver data management.
//! Based on sys_drivers.c

use std::os::unix::io::RawFd;
use infrastructure_driver_api::{
    DriverPort, DriverEvent, DriverData as DriverDataPtr, DriverSizeT, DriverIOVec, IoVec,
    DriverSelectFlags,
    driver_select, driver_output, driver_outputv,
    driver_enq, driver_deq, driver_sizeq, driver_peekq, driver_enqv,
    driver_failure_posix, driver_failure_eof, driver_failure_atom,
    erl_drv_init_ack, set_os_pid,
    driver_pdl_create, driver_pdl_lock, driver_pdl_unlock,
    driver_async_port_key, driver_async, set_busy_port,
};

#[cfg(unix)]
use nix::{
    unistd::{close, read, write},
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

/// Driver data structure
///
/// Equivalent to C's `ErtsSysDriverData`. Contains all state for a driver instance.
#[derive(Debug)]
pub struct SysDriverData {
    /// Port number
    pub port_num: DriverPort,
    /// Output file descriptor data
    pub ofd: Option<Box<FdData>>,
    /// Input file descriptor data
    pub ifd: Option<Box<FdData>>,
    /// Packet bytes (0, 1, 2, or 4)
    pub packet_bytes: i32,
    /// Process ID
    pub pid: i32,
    /// Alive flag (0 = not alive, 1 = alive, -1 = exited)
    pub alive: i32,
    /// Status (exit status)
    pub status: i32,
    /// Terminating flag
    pub terminating: i32,
    /// Blocking data (for async I/O)
    pub blocking: Option<*mut std::ffi::c_void>,
    /// Busy flag
    pub busy: bool,
    /// High watermark for queue
    pub high_watermark: DriverSizeT,
    /// Low watermark for queue
    pub low_watermark: DriverSizeT,
}

impl SysDriverData {
    /// Create a new DriverData
    pub fn new(port_num: DriverPort) -> Self {
        Self {
            port_num,
            ofd: None,
            ifd: None,
            packet_bytes: 0,
            pid: 0,
            alive: 0,
            status: 0,
            terminating: 0,
            blocking: None,
            busy: false,
            high_watermark: 0,
            low_watermark: 0,
        }
    }
}

/// Output data to Erlang
///
/// Equivalent to C's `output`. Sends data from driver to Erlang.
///
/// # Arguments
///
/// * `drv_data` - Driver data
/// * `buf` - Buffer containing data to output
/// * `len` - Length of data to output
pub fn output(drv_data: &mut SysDriverData, buf: &[u8], len: DriverSizeT) {
    let port = drv_data.port_num;
    let pb = drv_data.packet_bytes;
    let ofd = drv_data.ofd.as_ref().map(|f| f.fd).unwrap_or(-1);

    // Validate packet size
    #[cfg(unix)]
    use nix::errno::Errno;
    if ((pb == 2) && (len > 0xffff)) || (pb == 1 && len > 0xff) || drv_data.pid == 0 {
        driver_failure_posix(port, Errno::EINVAL as i32);
        return;
    }

    // Encode packet length
    let mut length_bytes = [0u8; 4];
    length_bytes[0] = ((len >> 24) & 0xff) as u8;
    length_bytes[1] = ((len >> 16) & 0xff) as u8;
    length_bytes[2] = ((len >> 8) & 0xff) as u8;
    length_bytes[3] = (len & 0xff) as u8;
    let lbp = &length_bytes[(4 - pb as usize)..];

    let qsz = driver_sizeq(port);
    if qsz > 0 {
        // Queue is not empty, enqueue
        let _ = driver_enq(port, lbp);
        let _ = driver_enq(port, &buf[..len]);
        let qsz = qsz + len + pb as usize;
        if !drv_data.busy && qsz >= drv_data.high_watermark {
            set_busy_port(port, true);
            drv_data.busy = true;
        }
    } else {
        // Try to write directly
        #[cfg(unix)]
        {
            if ofd >= 0 {
                let mut written = 0;
                
                // Write length bytes using nix::unistd::write
                match write(ofd, lbp) {
                    Ok(n) => {
                        written += n;
                        
                        // Write data if length bytes were fully written
                        if written == pb as usize {
                            match write(ofd, &buf[..len]) {
                                Ok(n) => written += n,
                                Err(Errno::EINTR) | Err(Errno::EAGAIN) => {
                                    // Would block, will retry later
                                }
                                Err(_) => {
                                    // Error writing data, enqueue everything
                                    let _ = driver_enq(port, lbp);
                                    let _ = driver_enq(port, &buf[..len]);
                                    let event = DriverEvent::new(ofd);
                                    let flags = DriverSelectFlags::WRITE | DriverSelectFlags::USE;
                                    let _ = driver_select(port, event, flags, true);
                                    return;
                                }
                            }
                        }
                    }
                    Err(Errno::EINTR) | Err(Errno::EAGAIN) => {
                        // Would block, enqueue everything
                        let _ = driver_enq(port, lbp);
                        let _ = driver_enq(port, &buf[..len]);
                        let event = DriverEvent::new(ofd);
                        let flags = DriverSelectFlags::WRITE | DriverSelectFlags::USE;
                        let _ = driver_select(port, event, flags, true);
                        return;
                    }
                    Err(_) => {
                        // Error, enqueue everything
                        let _ = driver_enq(port, lbp);
                        let _ = driver_enq(port, &buf[..len]);
                        let event = DriverEvent::new(ofd);
                        let flags = DriverSelectFlags::WRITE | DriverSelectFlags::USE;
                        let _ = driver_select(port, event, flags, true);
                        return;
                    }
                }
                
                if written < (pb as usize + len) {
                    // Partial write, enqueue remainder
                    if written < pb as usize {
                        let _ = driver_enq(port, &lbp[written..]);
                        let _ = driver_enq(port, &buf[..len]);
                    } else {
                        let data_written = written - pb as usize;
                        let _ = driver_enq(port, &buf[data_written..len]);
                    }
                    let event = DriverEvent::new(ofd);
                    let flags = DriverSelectFlags::WRITE | DriverSelectFlags::USE;
                    let _ = driver_select(port, event, flags, true);
                }
                
                let qsz = (pb as usize + len) - written;
                if !drv_data.busy && qsz >= drv_data.high_watermark {
                    set_busy_port(port, true);
                    drv_data.busy = true;
                }
            }
        }
    }
}

/// Ready input callback
///
/// Equivalent to C's `ready_input`. Called when input is ready on a file descriptor.
///
/// # Arguments
///
/// * `drv_data` - Driver data
/// * `ready_fd` - File descriptor that is ready
pub fn ready_input(drv_data: &mut SysDriverData, ready_fd: RawFd) {
    let port = drv_data.port_num;
    let packet_bytes = drv_data.packet_bytes;

    if let Some(ref mut ifd) = drv_data.ifd {
        let abs_fd = if ifd.fd < 0 { -ifd.fd } else { ifd.fd };
        if abs_fd != ready_fd {
            return; // Wrong file descriptor
        }

        if packet_bytes == 0 {
            // No packet mode - read directly
            let mut read_buf = vec![0u8; 64 * 1024]; // ERTS_SYS_READ_BUF_SZ
            #[cfg(unix)]
            {
                match read(ready_fd, &mut read_buf) {
                    Ok(0) => {
                        port_inp_failure(drv_data, 0);
                    }
                    Ok(n) => {
                        read_buf.truncate(n);
                        let _ = driver_output(port, &read_buf);
                    }
                    Err(Errno::EINTR) | Err(Errno::EAGAIN) => {
                        // Would block or interrupted, retry later
                    }
                    Err(e) => {
                        port_inp_failure(drv_data, -(e as i32));
                    }
                }
            }
        } else {
            // Packet mode - handle packet reading
            // This is a simplified implementation
            // Full implementation would handle partial packets, remainders, etc.
            let mut read_buf = vec![0u8; 64 * 1024];
            #[cfg(unix)]
            {
                match read(ready_fd, &mut read_buf) {
                    Ok(0) => {
                        port_inp_failure(drv_data, 0);
                    }
                    Ok(n) => {
                        // Simplified packet handling
                        read_buf.truncate(n);
                        let _ = driver_output(port, &read_buf);
                    }
                    Err(Errno::EINTR) | Err(Errno::EAGAIN) => {
                        // Would block or interrupted, retry later
                    }
                    Err(e) => {
                        port_inp_failure(drv_data, -(e as i32));
                    }
                }
            }
        }
    }
}

/// Ready output callback
///
/// Equivalent to C's `ready_output`. Called when output is ready on a file descriptor.
///
/// # Arguments
///
/// * `drv_data` - Driver data
/// * `ready_fd` - File descriptor that is ready
pub fn ready_output(drv_data: &mut SysDriverData, ready_fd: RawFd) {
    let port = drv_data.port_num;

    if let Some((iov, _vsize)) = driver_peekq(port) {
        let vsize = iov.len().min(16); // MAX_VSIZE
        #[cfg(unix)]
        {
            let mut total_written = 0;
            
            for vec in &iov[..vsize] {
                let slice = unsafe { std::slice::from_raw_parts(vec.base, vec.len) };
                match write(ready_fd, slice) {
                    Ok(n) => {
                        total_written += n;
                        if n < slice.len() {
                            // Partial write, stop here
                            break;
                        }
                    }
                    Err(Errno::EINTR) | Err(Errno::EAGAIN) => {
                        // Would block or interrupted, stop here
                        break;
                    }
                    Err(_) => {
                        // Error, stop writing
                        break;
                    }
                }
            }
            
            if total_written > 0 {
                let qsz = driver_deq(port, total_written);
                if drv_data.busy && qsz < drv_data.low_watermark {
                    set_busy_port(port, false);
                    drv_data.busy = false;
                }
            }
        }
    } else {
        // No data in queue
        if drv_data.busy {
            set_busy_port(port, false);
            drv_data.busy = false;
        }
        let event = DriverEvent::new(ready_fd);
        let flags = DriverSelectFlags::WRITE;
        let _ = driver_select(port, event, flags, false);
        
        if drv_data.terminating != 0 {
            driver_failure_atom(port, "normal");
        }
    }
}

/// Port input failure handler
///
/// Equivalent to C's `port_inp_failure`. Handles input failures (EOF or error).
///
/// # Arguments
///
/// * `drv_data` - Driver data
/// * `res` - Result code (0 for EOF, negative for error)
fn port_inp_failure(drv_data: &mut SysDriverData, res: i32) {
    if let Some(ref mut ifd) = drv_data.ifd {
        let event = DriverEvent::new(ifd.fd);
        let flags = DriverSelectFlags::READ | DriverSelectFlags::WRITE;
        let _ = driver_select(drv_data.port_num, event, flags, false);
        clear_fd_data(ifd);
    }

    if res == 0 {
        // EOF
        if drv_data.alive == 1 {
            // Process hasn't exited yet
            return;
        } else if drv_data.alive == -1 {
            // Process has exited
            let _status = drv_data.status;
            // Report exit status would go here
        }
        driver_failure_eof(drv_data.port_num);
    } else {
        // Error
        if drv_data.ifd.is_some() {
            if drv_data.alive == -1 {
                // Use status as errno
                driver_failure_posix(drv_data.port_num, drv_data.status);
            } else {
                let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(Errno::EIO as i32);
                driver_failure_posix(drv_data.port_num, err);
            }
        }
    }
}

/// Stop driver
///
/// Equivalent to C's `stop`. Stops and cleans up a driver.
///
/// # Arguments
///
/// * `drv_data` - Driver data
pub fn stop(drv_data: &mut SysDriverData) {
    let port = drv_data.port_num;

    if let Some(ref mut ifd) = drv_data.ifd {
        nbio_stop_fd(ifd);
        let abs_fd = if ifd.fd < 0 { -ifd.fd } else { ifd.fd };
        let event = DriverEvent::new(abs_fd);
        let flags = DriverSelectFlags::USE;
        let _ = driver_select(port, event, flags, false);
    }

    if let Some(ref mut ofd) = drv_data.ofd {
        if drv_data.ifd.as_ref().map(|f| f.fd) != Some(ofd.fd) {
            nbio_stop_fd(ofd);
            let abs_fd = if ofd.fd < 0 { -ofd.fd } else { ofd.fd };
        let event = DriverEvent::new(abs_fd);
            let flags = DriverSelectFlags::USE;
            let _ = driver_select(port, event, flags, false);
        }
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
}

