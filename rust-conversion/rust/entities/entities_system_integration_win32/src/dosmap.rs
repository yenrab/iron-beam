//! DOS Map Module (Windows-specific)
//!
//! Provides Windows-specific DOS mapping functionality.
//! Maps Windows OS errors to Unix System V errno values.
//! Based on dosmap.c

/// POSIX errno values (Unix System V errno)
/// These correspond to standard POSIX error codes
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixErrno {
    /// Invalid argument
    EINVAL = 22,
    /// No such file or directory
    ENOENT = 2,
    /// Too many open files
    EMFILE = 24,
    /// Permission denied
    EACCES = 13,
    /// Bad file descriptor
    EBADF = 9,
    /// Out of memory
    ENOMEM = 12,
    /// Argument list too long
    E2BIG = 7,
    /// Exec format error
    ENOEXEC = 8,
    /// Cross-device link
    EXDEV = 18,
    /// File exists
    EEXIST = 17,
    /// Try again
    EAGAIN = 11,
    /// Broken pipe
    EPIPE = 32,
    /// No space left on device
    ENOSPC = 28,
    /// No child processes
    ECHILD = 10,
    /// Directory not empty
    ENOTEMPTY = 39,
}

impl From<PosixErrno> for i32 {
    fn from(errno: PosixErrno) -> i32 {
        errno as i32
    }
}

/// Windows error code constants
/// These correspond to Windows ERROR_* constants
pub mod win_error {
    /// ERROR_NOT_ENOUGH_QUOTA - special case error code
    pub const ERROR_NOT_ENOUGH_QUOTA: u32 = 1816;
}

/// Error mapping table: Windows OS error -> POSIX errno
/// Position in table = Windows OS error code
/// This matches the errMapTable from dosmap.c
const ERR_MAP_TABLE: &[PosixErrno] = &[
    PosixErrno::EINVAL,  /* ERROR_SUCCESS                      0  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_FUNCTION             1  */
    PosixErrno::ENOENT,  /* ERROR_FILE_NOT_FOUND               2  */
    PosixErrno::ENOENT,  /* ERROR_PATH_NOT_FOUND               3  */
    PosixErrno::EMFILE,  /* ERROR_TOO_MANY_OPEN_FILES          4  */
    PosixErrno::EACCES,  /* ERROR_ACCESS_DENIED                5  */
    PosixErrno::EBADF,   /* ERROR_INVALID_HANDLE               6  */
    PosixErrno::ENOMEM,  /* ERROR_ARENA_TRASHED                7  */
    PosixErrno::ENOMEM,  /* ERROR_NOT_ENOUGH_MEMORY            8  */
    PosixErrno::ENOMEM,  /* ERROR_INVALID_BLOCK                9  */
    PosixErrno::E2BIG,   /* ERROR_BAD_ENVIRONMENT             10  */
    PosixErrno::ENOEXEC, /* ERROR_BAD_FORMAT                  11  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_ACCESS              12  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_DATA                13  */
    PosixErrno::EINVAL,  /* ERROR_OUTOFMEMORY                 14  */
    PosixErrno::ENOENT,  /* ERROR_INVALID_DRIVE               15  */
    PosixErrno::EACCES,  /* ERROR_CURRENT_DIRECTORY           16  */
    PosixErrno::EXDEV,   /* ERROR_NOT_SAME_DEVICE             17  */
    PosixErrno::ENOENT,  /* ERROR_NO_MORE_FILES               18  */
    PosixErrno::EACCES,  /* ERROR_WRITE_PROTECT               19  */
    PosixErrno::EACCES,  /* ERROR_BAD_UNIT                    20  */
    PosixErrno::EACCES,  /* ERROR_NOT_READY                   21  */
    PosixErrno::EACCES,  /* ERROR_BAD_COMMAND                 22  */
    PosixErrno::EACCES,  /* ERROR_CRC                         23  */
    PosixErrno::EACCES,  /* ERROR_BAD_LENGTH                  24  */
    PosixErrno::EACCES,  /* ERROR_SEEK                        25  */
    PosixErrno::EACCES,  /* ERROR_NOT_DOS_DISK                26  */
    PosixErrno::EACCES,  /* ERROR_SECTOR_NOT_FOUND            27  */
    PosixErrno::EACCES,  /* ERROR_OUT_OF_PAPER                28  */
    PosixErrno::EACCES,  /* ERROR_WRITE_FAULT                 29  */
    PosixErrno::EACCES,  /* ERROR_READ_FAULT                  30  */
    PosixErrno::EACCES,  /* ERROR_GEN_FAILURE                 31  */
    PosixErrno::EACCES,  /* ERROR_SHARING_VIOLATION           32  */
    PosixErrno::EACCES,  /* ERROR_LOCK_VIOLATION              33  */
    PosixErrno::EACCES,  /* ERROR_WRONG_DISK                  34  */
    PosixErrno::EACCES,  /*                                   35  */
    PosixErrno::EACCES,  /* ERROR_SHARING_BUFFER_EXCEEDED     36  */
    PosixErrno::EINVAL,  /*                                   37  */
    PosixErrno::EINVAL,  /* ERROR_HANDLE_EOF                  38  */
    PosixErrno::EINVAL,  /* ERROR_HANDLE_DISK_FULL            39  */
    PosixErrno::EINVAL,  /*                                   40  */
    PosixErrno::EINVAL,  /*                                   41  */
    PosixErrno::EINVAL,  /*                                   42  */
    PosixErrno::EINVAL,  /*                                   43  */
    PosixErrno::EINVAL,  /*                                   44  */
    PosixErrno::EINVAL,  /*                                   45  */
    PosixErrno::EINVAL,  /*                                   46  */
    PosixErrno::EINVAL,  /*                                   47  */
    PosixErrno::EINVAL,  /*                                   48  */
    PosixErrno::EINVAL,  /*                                   49  */
    PosixErrno::EINVAL,  /* ERROR_NOT_SUPPORTED               50  */
    PosixErrno::EINVAL,  /* ERROR_REM_NOT_LIST                51  */
    PosixErrno::EINVAL,  /* ERROR_DUP_NAME                    52  */
    PosixErrno::ENOENT,  /* ERROR_BAD_NETPATH                 53  */
    PosixErrno::EINVAL,  /* ERROR_NETWORK_BUSY                54  */
    PosixErrno::EINVAL,  /* ERROR_DEV_NOT_EXIST               55  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_CMDS               56  */
    PosixErrno::EINVAL,  /* ERROR_ADAP_HDW_ERR                57  */
    PosixErrno::EINVAL,  /* ERROR_BAD_NET_RESP                58  */
    PosixErrno::EINVAL,  /* ERROR_UNEXP_NET_ERR               59  */
    PosixErrno::EINVAL,  /* ERROR_BAD_REM_ADAP                60  */
    PosixErrno::EINVAL,  /* ERROR_PRINTQ_FULL                 61  */
    PosixErrno::EINVAL,  /* ERROR_NO_SPOOL_SPACE              62  */
    PosixErrno::EINVAL,  /* ERROR_PRINT_CANCELLED             63  */
    PosixErrno::EINVAL,  /* ERROR_NETNAME_DELETED             64  */
    PosixErrno::EACCES,  /* ERROR_NETWORK_ACCESS_DENIED       65  */
    PosixErrno::EINVAL,  /* ERROR_BAD_DEV_TYPE                66  */
    PosixErrno::ENOENT,  /* ERROR_BAD_NET_NAME                67  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_NAMES              68  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_SESS               69  */
    PosixErrno::EINVAL,  /* ERROR_SHARING_PAUSED              70  */
    PosixErrno::EINVAL,  /* ERROR_REQ_NOT_ACCEP               71  */
    PosixErrno::EINVAL,  /* ERROR_REDIR_PAUSED                72  */
    PosixErrno::EINVAL,  /*                                   73  */
    PosixErrno::EINVAL,  /*                                   74  */
    PosixErrno::EINVAL,  /*                                   75  */
    PosixErrno::EINVAL,  /*                                   76  */
    PosixErrno::EINVAL,  /*                                   77  */
    PosixErrno::EINVAL,  /*                                   78  */
    PosixErrno::EINVAL,  /*                                   79  */
    PosixErrno::EEXIST,  /* ERROR_FILE_EXISTS                 80  */
    PosixErrno::EINVAL,  /*                                   81  */
    PosixErrno::EACCES,  /* ERROR_CANNOT_MAKE                 82  */
    PosixErrno::EACCES,  /* ERROR_FAIL_I24                    83  */
    PosixErrno::EINVAL,  /* ERROR_OUT_OF_STRUCTURES           84  */
    PosixErrno::EINVAL,  /* ERROR_ALREADY_ASSIGNED            85  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_PASSWORD            86  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_PARAMETER           87  */
    PosixErrno::EINVAL,  /* ERROR_NET_WRITE_FAULT             88  */
    PosixErrno::EAGAIN,  /* ERROR_NO_PROC_SLOTS               89  */
    PosixErrno::EINVAL,  /*                                   90  */
    PosixErrno::EINVAL,  /*                                   91  */
    PosixErrno::EINVAL,  /*                                   92  */
    PosixErrno::EINVAL,  /*                                   93  */
    PosixErrno::EINVAL,  /*                                   94  */
    PosixErrno::EINVAL,  /*                                   95  */
    PosixErrno::EINVAL,  /*                                   96  */
    PosixErrno::EINVAL,  /*                                   97  */
    PosixErrno::EINVAL,  /*                                   98  */
    PosixErrno::EINVAL,  /*                                   99  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_SEMAPHORES        100  */
    PosixErrno::EINVAL,  /* ERROR_EXCL_SEM_ALREADY_OWNED     101  */
    PosixErrno::EINVAL,  /* ERROR_SEM_IS_SET                 102  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_SEM_REQUESTS      103  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_AT_INTERRUPT_TIME  104  */
    PosixErrno::EINVAL,  /* ERROR_SEM_OWNER_DIED             105  */
    PosixErrno::EINVAL,  /* ERROR_SEM_USER_LIMIT             106  */
    PosixErrno::EINVAL,  /* ERROR_DISK_CHANGE                107  */
    PosixErrno::EACCES,  /* ERROR_DRIVE_LOCKED               108  */
    PosixErrno::EPIPE,   /* ERROR_BROKEN_PIPE                109  */
    PosixErrno::EINVAL,  /* ERROR_OPEN_FAILED                110  */
    PosixErrno::EINVAL,  /* ERROR_BUFFER_OVERFLOW            111  */
    PosixErrno::ENOSPC,  /* ERROR_DISK_FULL                  112  */
    PosixErrno::EINVAL,  /* ERROR_NO_MORE_SEARCH_HANDLES     113  */
    PosixErrno::EBADF,   /* ERROR_INVALID_TARGET_HANDLE      114  */
    PosixErrno::EINVAL,  /*                                  115  */
    PosixErrno::EINVAL,  /*                                  116  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_CATEGORY           117  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_VERIFY_SWITCH      118  */
    PosixErrno::EINVAL,  /* ERROR_BAD_DRIVER_LEVEL           119  */
    PosixErrno::EINVAL,  /* ERROR_CALL_NOT_IMPLEMENTED       120  */
    PosixErrno::EINVAL,  /* ERROR_SEM_TIMEOUT                121  */
    PosixErrno::EINVAL,  /* ERROR_INSUFFICIENT_BUFFER        122  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_NAME               123  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_LEVEL              124  */
    PosixErrno::EINVAL,  /* ERROR_NO_VOLUME_LABEL            125  */
    PosixErrno::EINVAL,  /* ERROR_MOD_NOT_FOUND              126  */
    PosixErrno::EINVAL,  /* ERROR_PROC_NOT_FOUND             127  */
    PosixErrno::ECHILD,  /* ERROR_WAIT_NO_CHILDREN           128  */
    PosixErrno::ECHILD,  /* ERROR_CHILD_NOT_COMPLETE         129  */
    PosixErrno::EBADF,   /* ERROR_DIRECT_ACCESS_HANDLE       130  */
    PosixErrno::EINVAL,  /* ERROR_NEGATIVE_SEEK              131  */
    PosixErrno::EACCES,  /* ERROR_SEEK_ON_DEVICE             132  */
    PosixErrno::EINVAL,  /* ERROR_IS_JOIN_TARGET             133  */
    PosixErrno::EINVAL,  /* ERROR_IS_JOINED                  134  */
    PosixErrno::EINVAL,  /* ERROR_IS_SUBSTED                 135  */
    PosixErrno::EINVAL,  /* ERROR_NOT_JOINED                 136  */
    PosixErrno::EINVAL,  /* ERROR_NOT_SUBSTED                137  */
    PosixErrno::EINVAL,  /* ERROR_JOIN_TO_JOIN               138  */
    PosixErrno::EINVAL,  /* ERROR_SUBST_TO_SUBST             139  */
    PosixErrno::EINVAL,  /* ERROR_JOIN_TO_SUBST              140  */
    PosixErrno::EINVAL,  /* ERROR_SUBST_TO_JOIN              141  */
    PosixErrno::EINVAL,  /* ERROR_BUSY_DRIVE                 142  */
    PosixErrno::EINVAL,  /* ERROR_SAME_DRIVE                 143  */
    PosixErrno::EINVAL,  /* ERROR_DIR_NOT_ROOT               144  */
    PosixErrno::ENOTEMPTY, /* ERROR_DIR_NOT_EMPTY              145  */
    PosixErrno::EINVAL,  /* ERROR_IS_SUBST_PATH              146  */
    PosixErrno::EINVAL,  /* ERROR_IS_JOIN_PATH               147  */
    PosixErrno::EINVAL,  /* ERROR_PATH_BUSY                  148  */
    PosixErrno::EINVAL,  /* ERROR_IS_SUBST_TARGET            149  */
    PosixErrno::EINVAL,  /* ERROR_SYSTEM_TRACE               150  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_EVENT_COUNT        151  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_MUXWAITERS        152  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_LIST_FORMAT        153  */
    PosixErrno::EINVAL,  /* ERROR_LABEL_TOO_LONG             154  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_TCBS              155  */
    PosixErrno::EINVAL,  /* ERROR_SIGNAL_REFUSED             156  */
    PosixErrno::EINVAL,  /* ERROR_DISCARDED                  157  */
    PosixErrno::EACCES,  /* ERROR_NOT_LOCKED                 158  */
    PosixErrno::EINVAL,  /* ERROR_BAD_THREADID_ADDR          159  */
    PosixErrno::EINVAL,  /* ERROR_BAD_ARGUMENTS              160  */
    PosixErrno::ENOENT,  /* ERROR_BAD_PATHNAME               161  */
    PosixErrno::EINVAL,  /* ERROR_SIGNAL_PENDING             162  */
    PosixErrno::EINVAL,  /*                                  163  */
    PosixErrno::EAGAIN,  /* ERROR_MAX_THRDS_REACHED          164  */
    PosixErrno::EINVAL,  /*                                  165  */
    PosixErrno::EINVAL,  /*                                  166  */
    PosixErrno::EACCES,  /* ERROR_LOCK_FAILED                167  */
    PosixErrno::EINVAL,  /*                                  168  */
    PosixErrno::EINVAL,  /*                                  169  */
    PosixErrno::EINVAL,  /* ERROR_BUSY                       170  */
    PosixErrno::EINVAL,  /*                                  171  */
    PosixErrno::EINVAL,  /*                                  172  */
    PosixErrno::EINVAL,  /* ERROR_CANCEL_VIOLATION           173  */
    PosixErrno::EINVAL,  /* ERROR_ATOMIC_LOCKS_NOT_SUPPORTED 174  */
    PosixErrno::EINVAL,  /*                                  175  */
    PosixErrno::EINVAL,  /*                                  176  */
    PosixErrno::EINVAL,  /*                                  177  */
    PosixErrno::EINVAL,  /*                                  178  */
    PosixErrno::EINVAL,  /*                                  179  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_SEGMENT_NUMBER     180  */
    PosixErrno::EINVAL,  /*                                  181  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_ORDINAL            182  */
    PosixErrno::EEXIST,  /* ERROR_ALREADY_EXISTS             183  */
    PosixErrno::EINVAL,  /*                                  184  */
    PosixErrno::EINVAL,  /*                                  185  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_FLAG_NUMBER        186  */
    PosixErrno::EINVAL,  /* ERROR_SEM_NOT_FOUND              187  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_STARTING_CODESEG   188  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_STACKSEG           189  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_MODULETYPE         190  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_EXE_SIGNATURE      191  */
    PosixErrno::ENOEXEC, /* ERROR_EXE_MARKED_INVALID         192  */
    PosixErrno::ENOEXEC, /* ERROR_BAD_EXE_FORMAT             193  */
    PosixErrno::ENOEXEC, /* ERROR_ITERATED_DATA_EXCEEDS_64k  194  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_MINALLOCSIZE       195  */
    PosixErrno::ENOEXEC, /* ERROR_DYNLINK_FROM_INVALID_RING  196  */
    PosixErrno::ENOEXEC, /* ERROR_IOPL_NOT_ENABLED           197  */
    PosixErrno::ENOEXEC, /* ERROR_INVALID_SEGDPL             198  */
    PosixErrno::ENOEXEC, /* ERROR_AUTODATASEG_EXCEEDS_64k    199  */
    PosixErrno::ENOEXEC, /* ERROR_RING2SEG_MUST_BE_MOVABLE   200  */
    PosixErrno::ENOEXEC, /* ERROR_RELOC_CHAIN_XEEDS_SEGLIM   201  */
    PosixErrno::ENOEXEC, /* ERROR_INFLOOP_IN_RELOC_CHAIN     202  */
    PosixErrno::EINVAL,  /* ERROR_ENVVAR_NOT_FOUND           203  */
    PosixErrno::EINVAL,  /*                                  204  */
    PosixErrno::EINVAL,  /* ERROR_NO_SIGNAL_SENT             205  */
    PosixErrno::ENOENT,  /* ERROR_FILENAME_EXCED_RANGE       206  */
    PosixErrno::EINVAL,  /* ERROR_RING2_STACK_IN_USE         207  */
    PosixErrno::EINVAL,  /* ERROR_META_EXPANSION_TOO_LONG    208  */
    PosixErrno::EINVAL,  /* ERROR_INVALID_SIGNAL_NUMBER      209  */
    PosixErrno::EINVAL,  /* ERROR_THREAD_1_INACTIVE          210  */
    PosixErrno::EINVAL,  /*                                  211  */
    PosixErrno::EINVAL,  /* ERROR_LOCKED                     212  */
    PosixErrno::EINVAL,  /*                                  213  */
    PosixErrno::EINVAL,  /* ERROR_TOO_MANY_MODULES           214  */
    PosixErrno::EAGAIN,  /* ERROR_NESTING_NOT_ALLOWED        215  */
];

/// Size of the error mapping table
const ERR_MAP_TABLE_SIZE: usize = ERR_MAP_TABLE.len();

/// Windows-specific DOS mapping operations
pub struct DosMap;

impl DosMap {
    /// Map a Windows error code to a POSIX errno value
    ///
    /// Takes a Windows error number and maps it to a Unix System V errno.
    /// This matches the behavior of `_dosmaperr` in dosmap.c.
    ///
    /// # Arguments
    /// * `winerrno` - Windows error code
    ///
    /// # Returns
    /// POSIX errno value as i32
    ///
    /// # Special Cases
    /// - ERROR_NOT_ENOUGH_QUOTA (1816) maps to ENOMEM
    /// - Error codes >= table size map to EINVAL
    pub fn map_error(winerrno: u32) -> i32 {
        if winerrno >= ERR_MAP_TABLE_SIZE as u32 {
            // Special case: ERROR_NOT_ENOUGH_QUOTA (1816) maps to ENOMEM
            if winerrno == win_error::ERROR_NOT_ENOUGH_QUOTA {
                return PosixErrno::ENOMEM as i32;
            }
            // All other out-of-range errors map to EINVAL
            return PosixErrno::EINVAL as i32;
        }
        
        // Look up in table
        ERR_MAP_TABLE[winerrno as usize] as i32
    }

    /// Map a Windows error code to a PosixErrno enum value
    ///
    /// # Arguments
    /// * `winerrno` - Windows error code
    ///
    /// # Returns
    /// PosixErrno enum value
    pub fn map_error_to_errno(winerrno: u32) -> PosixErrno {
        if winerrno >= ERR_MAP_TABLE_SIZE as u32 {
            if winerrno == win_error::ERROR_NOT_ENOUGH_QUOTA {
                return PosixErrno::ENOMEM;
            }
            return PosixErrno::EINVAL;
        }
        
        ERR_MAP_TABLE[winerrno as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_dosmap_basic_errors() {
        // Test ERROR_FILE_NOT_FOUND (2) -> ENOENT
        assert_eq!(DosMap::map_error(2), PosixErrno::ENOENT as i32);
        assert_eq!(DosMap::map_error_to_errno(2), PosixErrno::ENOENT);

        // Test ERROR_PATH_NOT_FOUND (3) -> ENOENT
        assert_eq!(DosMap::map_error(3), PosixErrno::ENOENT as i32);
        assert_eq!(DosMap::map_error_to_errno(3), PosixErrno::ENOENT);

        // Test ERROR_ACCESS_DENIED (5) -> EACCES
        assert_eq!(DosMap::map_error(5), PosixErrno::EACCES as i32);
        assert_eq!(DosMap::map_error_to_errno(5), PosixErrno::EACCES);

        // Test ERROR_INVALID_HANDLE (6) -> EBADF
        assert_eq!(DosMap::map_error(6), PosixErrno::EBADF as i32);
        assert_eq!(DosMap::map_error_to_errno(6), PosixErrno::EBADF);

        // Test ERROR_NOT_ENOUGH_MEMORY (8) -> ENOMEM
        assert_eq!(DosMap::map_error(8), PosixErrno::ENOMEM as i32);
        assert_eq!(DosMap::map_error_to_errno(8), PosixErrno::ENOMEM);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_special_cases() {
        // Test ERROR_SUCCESS (0) -> EINVAL
        assert_eq!(DosMap::map_error(0), PosixErrno::EINVAL as i32);
        assert_eq!(DosMap::map_error_to_errno(0), PosixErrno::EINVAL);

        // Test ERROR_FILE_EXISTS (80) -> EEXIST
        assert_eq!(DosMap::map_error(80), PosixErrno::EEXIST as i32);
        assert_eq!(DosMap::map_error_to_errno(80), PosixErrno::EEXIST);

        // Test ERROR_ALREADY_EXISTS (183) -> EEXIST
        assert_eq!(DosMap::map_error(183), PosixErrno::EEXIST as i32);
        assert_eq!(DosMap::map_error_to_errno(183), PosixErrno::EEXIST);

        // Test ERROR_BROKEN_PIPE (109) -> EPIPE
        assert_eq!(DosMap::map_error(109), PosixErrno::EPIPE as i32);
        assert_eq!(DosMap::map_error_to_errno(109), PosixErrno::EPIPE);

        // Test ERROR_DISK_FULL (112) -> ENOSPC
        assert_eq!(DosMap::map_error(112), PosixErrno::ENOSPC as i32);
        assert_eq!(DosMap::map_error_to_errno(112), PosixErrno::ENOSPC);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_error_not_enough_quota() {
        // Special case: ERROR_NOT_ENOUGH_QUOTA (1816) -> ENOMEM
        assert_eq!(DosMap::map_error(win_error::ERROR_NOT_ENOUGH_QUOTA), PosixErrno::ENOMEM as i32);
        assert_eq!(DosMap::map_error_to_errno(win_error::ERROR_NOT_ENOUGH_QUOTA), PosixErrno::ENOMEM);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_out_of_range_errors() {
        // Error codes >= table size should map to EINVAL
        let table_size = ERR_MAP_TABLE_SIZE as u32;
        
        // Test just beyond table size
        assert_eq!(DosMap::map_error(table_size), PosixErrno::EINVAL as i32);
        assert_eq!(DosMap::map_error_to_errno(table_size), PosixErrno::EINVAL);

        // Test a large error code (but not ERROR_NOT_ENOUGH_QUOTA)
        assert_eq!(DosMap::map_error(10000), PosixErrno::EINVAL as i32);
        assert_eq!(DosMap::map_error_to_errno(10000), PosixErrno::EINVAL);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_table_boundary() {
        // Test the last entry in the table (215)
        assert_eq!(DosMap::map_error(215), PosixErrno::EAGAIN as i32);
        assert_eq!(DosMap::map_error_to_errno(215), PosixErrno::EAGAIN);

        // Test one before the last entry
        assert_eq!(DosMap::map_error(214), PosixErrno::EINVAL as i32);
        assert_eq!(DosMap::map_error_to_errno(214), PosixErrno::EINVAL);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_various_error_types() {
        // Test ENOEXEC errors (188-202)
        assert_eq!(DosMap::map_error_to_errno(188), PosixErrno::ENOEXEC);
        assert_eq!(DosMap::map_error_to_errno(193), PosixErrno::ENOEXEC);
        assert_eq!(DosMap::map_error_to_errno(202), PosixErrno::ENOEXEC);

        // Test ECHILD errors (128-129)
        assert_eq!(DosMap::map_error_to_errno(128), PosixErrno::ECHILD);
        assert_eq!(DosMap::map_error_to_errno(129), PosixErrno::ECHILD);

        // Test ENOTEMPTY error (145)
        assert_eq!(DosMap::map_error_to_errno(145), PosixErrno::ENOTEMPTY);

        // Test EAGAIN errors (89, 164, 215)
        assert_eq!(DosMap::map_error_to_errno(89), PosixErrno::EAGAIN);
        assert_eq!(DosMap::map_error_to_errno(164), PosixErrno::EAGAIN);
        assert_eq!(DosMap::map_error_to_errno(215), PosixErrno::EAGAIN);
    }

    #[test]
    #[cfg(windows)]
    fn test_dosmap_table_size() {
        // Verify table size matches expected value (216 entries, indices 0-215)
        assert_eq!(ERR_MAP_TABLE_SIZE, 216);
        assert_eq!(ERR_MAP_TABLE.len(), 216);
    }

    #[test]
    #[cfg(windows)]
    fn test_posix_errno_values() {
        // Verify some key POSIX errno values
        assert_eq!(PosixErrno::EINVAL as i32, 22);
        assert_eq!(PosixErrno::ENOENT as i32, 2);
        assert_eq!(PosixErrno::ENOMEM as i32, 12);
        assert_eq!(PosixErrno::EACCES as i32, 13);
        assert_eq!(PosixErrno::EBADF as i32, 9);
    }
}
