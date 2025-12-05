//! Driver API Type Definitions
//!
//! Provides type definitions equivalent to C's erl_driver.h types.

use std::os::unix::io::RawFd;

/// Driver port identifier
///
/// Equivalent to C's `ErlDrvPort`. Represents a port in the Erlang runtime system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverPort {
    id: u64,
}

impl DriverPort {
    /// Create a new driver port from an ID
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the port ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Driver data pointer
///
/// Equivalent to C's `ErlDrvData`. Opaque pointer to driver-specific data.
///
/// This is a wrapper around a raw pointer to ensure it can be safely
/// sent between threads. The driver infrastructure ensures proper synchronization.
#[derive(Debug, Clone, Copy)]
pub struct DriverData(*mut std::ffi::c_void);

impl DriverData {
    /// Create a new DriverData from a raw pointer
    pub fn new(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a null DriverData
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if this is null
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

// Safety: DriverData is an opaque pointer that drivers manage.
// The driver infrastructure ensures proper synchronization.
unsafe impl Send for DriverData {}
unsafe impl Sync for DriverData {}

impl From<*mut std::ffi::c_void> for DriverData {
    fn from(ptr: *mut std::ffi::c_void) -> Self {
        Self::new(ptr)
    }
}

impl From<DriverData> for *mut std::ffi::c_void {
    fn from(data: DriverData) -> Self {
        data.as_ptr()
    }
}

/// Driver event (file descriptor)
///
/// Equivalent to C's `ErlDrvEvent`. Represents an event (typically a file descriptor)
/// that can be selected on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverEvent {
    fd: RawFd,
}

impl DriverEvent {
    /// Create a new driver event from a file descriptor
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }

    /// Get the file descriptor
    pub fn fd(&self) -> RawFd {
        self.fd
    }
}

impl From<RawFd> for DriverEvent {
    fn from(fd: RawFd) -> Self {
        Self::new(fd)
    }
}

impl From<DriverEvent> for RawFd {
    fn from(event: DriverEvent) -> Self {
        event.fd
    }
}

/// Driver size type
///
/// Equivalent to C's `ErlDrvSizeT`. Used for sizes and lengths in driver operations.
pub type DriverSizeT = usize;

/// Driver signed size type
///
/// Equivalent to C's `ErlDrvSSizeT`. Used for signed sizes and return values.
pub type DriverSSizeT = isize;

/// Driver flags for selection
///
/// Equivalent to C's driver selection flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverSelectFlags(u32);

impl DriverSelectFlags {
    /// Read flag
    pub const READ: Self = Self(1 << 0);
    /// Write flag
    pub const WRITE: Self = Self(1 << 1);
    /// Use flag (keep event registered)
    pub const USE: Self = Self(1 << 2);
    /// Use without callback flag
    pub const USE_NO_CALLBACK: Self = Self(1 << 3);

    /// Create new flags
    pub fn new() -> Self {
        Self(0)
    }

    /// Add a flag
    pub fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }

    /// Check if a flag is set
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Get raw value
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for DriverSelectFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// I/O vector for scatter/gather operations
///
/// Equivalent to C's `ErlIOVec`.
#[derive(Debug)]
pub struct DriverIOVec {
    /// Number of vectors
    pub vsize: usize,
    /// Total size in bytes
    pub size: DriverSizeT,
    /// I/O vectors
    pub iov: Vec<IoVec>,
}

/// Single I/O vector entry
///
/// Equivalent to C's `SysIOVec`.
#[derive(Debug, Clone)]
pub struct IoVec {
    /// Base pointer
    pub base: *mut u8,
    /// Length
    pub len: usize,
}

unsafe impl Send for IoVec {}
unsafe impl Sync for IoVec {}

/// Driver binary
///
/// Equivalent to C's `ErlDrvBinary`.
#[derive(Debug)]
pub struct DriverBinary {
    /// Original size
    pub orig_size: DriverSSizeT,
    /// Data bytes
    pub data: Vec<u8>,
}

impl DriverBinary {
    /// Create a new driver binary
    pub fn new(data: Vec<u8>) -> Self {
        let orig_size = data.len() as DriverSSizeT;
        Self { orig_size, data }
    }

    /// Get the data as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the data as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Error codes that can be returned from driver start functions
///
/// Equivalent to C's `ERL_DRV_ERROR_*` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverErrorCode {
    /// General error
    General,
    /// System error (errno)
    Errno,
    /// Bad argument
    BadArg,
}

impl DriverErrorCode {
    /// Convert to driver data error value
    pub fn to_driver_data(self) -> DriverData {
        match self {
            DriverErrorCode::General => DriverData::new((-1isize) as *mut std::ffi::c_void),
            DriverErrorCode::Errno => DriverData::new((-2isize) as *mut std::ffi::c_void),
            DriverErrorCode::BadArg => DriverData::new((-3isize) as *mut std::ffi::c_void),
        }
    }
}

