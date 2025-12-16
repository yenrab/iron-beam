//! Scheduler data structures
//!
//! Equivalent to C ErtsSchedulerData and ErtsSchedulerRegisters structures.
//! These structures match the C definitions exactly to ensure compatibility
//! with generated JIT code.

use std::os::raw::c_void;

/// Maximum number of X registers allocated
pub const ERTS_X_REGS_ALLOCATED: usize = 1024;

/// Maximum number of floating point registers
pub const MAX_REG: usize = 1024;

/// Eterm type (64-bit value)
pub type Eterm = u64;

/// Float definition (matches C FloatDef)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FloatDef {
    pub value: f64,
}

/// Auxiliary registers structure
///
/// Matches C struct aux_regs__ in ErtsSchedulerRegisters.
#[repr(C)]
pub struct AuxRegs {
    /// Runtime stack (on normal schedulers)
    pub runtime_stack: [u64; 1],
    
    /// Temporary memory used by beamasm for allocations within instructions
    pub tmp_mem: [u64; 5],
    
    /// erl_bits.c state
    /// Note: This is a placeholder - actual structure would match C erl_bits_state
    pub erl_bits_state: [u8; 64], // Placeholder size
}

/// Scheduler registers structure
///
/// Matches C ErtsSchedulerRegisters structure exactly.
/// This structure is allocated on the stack by process_main.
#[repr(C, align(64))] // Cache line aligned
pub struct ErtsSchedulerRegisters {
    /// Auxiliary registers
    pub aux_regs: AuxRegs,
    
    /// X register array (Eterm registers)
    pub x_reg_array: [Eterm; ERTS_X_REGS_ALLOCATED],
    
    /// Floating point register array
    pub f_reg_array: [FloatDef; MAX_REG],
    
    /// Start time instruction pointer (seldom-used scheduler-specific data)
    pub start_time_i: *const c_void,
    
    /// Start time (seldom-used scheduler-specific data)
    pub start_time: u64,
}

impl ErtsSchedulerRegisters {
    /// Create a new scheduler registers structure
    pub fn new() -> Self {
        Self {
            aux_regs: AuxRegs {
                runtime_stack: [0; 1],
                tmp_mem: [0; 5],
                erl_bits_state: [0; 64],
            },
            x_reg_array: [0; ERTS_X_REGS_ALLOCATED],
            f_reg_array: [FloatDef { value: 0.0 }; MAX_REG],
            start_time_i: std::ptr::null(),
            start_time: 0,
        }
    }
}

/// Scheduler data structure
///
/// Matches C ErtsSchedulerData structure.
/// This is the main structure passed to process_main.
#[repr(C)]
pub struct ErtsSchedulerData {
    /// Pointer to scheduler registers (allocated on stack by process_main)
    pub registers: *mut ErtsSchedulerRegisters,
    
    /// Timer wheel
    pub timer_wheel: *mut c_void,
    
    /// Next timeout reference
    pub next_tmo_ref: u64,
    
    /// Timer service
    pub timer_service: *mut c_void,
    
    /// Thread ID
    pub tid: u64,
    
    /// Match pseudo process
    pub match_pseudo_process: *mut c_void,
    
    /// Free process
    pub free_process: *mut c_void, // Process*
    
    /// Thread progress data
    pub thr_progress_data: [u8; 64], // Placeholder
    
    /// Scheduler sleep info
    pub ssi: *mut c_void,
    
    /// Current process
    pub current_process: *mut c_void, // Process*
    
    /// Scheduler type
    pub type_: u32,
    
    /// Scheduler number for normal schedulers
    pub no: u32,
    
    /// Scheduler number for dirty schedulers
    pub dirty_no: u32,
    
    /// Flex counter slot number
    pub flxctr_slot_no: i32,
    
    /// Current NIF
    pub current_nif: *mut c_void,
    
    /// Dirty shadow process
    pub dirty_shadow_process: *mut c_void, // Process*
    
    /// Current port
    pub current_port: *mut c_void, // Port*
    
    /// Run queue
    pub run_queue: *mut c_void, // ErtsRunQueue*
    
    /// Virtual reductions
    pub virtual_reds: i32,
    
    /// CPU ID (>= 0 when bound)
    pub cpu_id: i32,
    
    /// Aux work data
    pub aux_work_data: [u8; 64], // Placeholder
    
    /// Atom cache map
    pub atom_cache_map: [u8; 64], // Placeholder
    
    /// Last monotonic time
    pub last_monotonic_time: u64,
    
    /// Check time reductions
    pub check_time_reds: i32,
    
    /// Thread ID
    pub thr_id: u32,
    
    /// Unique ID
    pub unique: u64,
    
    /// Reference
    pub ref_: u64,
    
    /// IO counters
    pub io_out: u64,
    pub io_in: u64,
    
    /// Reductions
    pub reductions: u64,
    
    /// Random state
    pub rand_state: u64,
    
    /// Scheduler wall time
    pub sched_wall_time: u64,
    
    /// GC info
    pub gc_info: [u8; 64], // Placeholder
    
    /// Port task handle
    pub nosuspend_port_task_handle: u64,
    
    /// Union field (ETS tables or dirty NIF halt info)
    pub u: [u8; 64], // Placeholder
}

impl ErtsSchedulerData {
    /// Create a new scheduler data structure
    pub fn new() -> Self {
        Self {
            registers: std::ptr::null_mut(),
            timer_wheel: std::ptr::null_mut(),
            next_tmo_ref: 0,
            timer_service: std::ptr::null_mut(),
            tid: 0,
            match_pseudo_process: std::ptr::null_mut(),
            free_process: std::ptr::null_mut(),
            thr_progress_data: [0; 64],
            ssi: std::ptr::null_mut(),
            current_process: std::ptr::null_mut(),
            type_: 0,
            no: 0,
            dirty_no: 0,
            flxctr_slot_no: 0,
            current_nif: std::ptr::null_mut(),
            dirty_shadow_process: std::ptr::null_mut(),
            current_port: std::ptr::null_mut(),
            run_queue: std::ptr::null_mut(),
            virtual_reds: 0,
            cpu_id: -1,
            aux_work_data: [0; 64],
            atom_cache_map: [0; 64],
            last_monotonic_time: 0,
            check_time_reds: 0,
            thr_id: 0,
            unique: 0,
            ref_: 0,
            io_out: 0,
            io_in: 0,
            reductions: 0,
            rand_state: 0,
            sched_wall_time: 0,
            gc_info: [0; 64],
            nosuspend_port_task_handle: 0,
            u: [0; 64],
        }
    }
}

/// JIT calling convention function pointer type for process_main
///
/// Matches C: void(ERTS_CCONV_JIT *)(ErtsSchedulerData *)
pub type JitProcessMain = unsafe extern "C" fn(*mut ErtsSchedulerData);

/// BEAM function entry point type
///
/// BEAM functions don't return - they jump to other functions or call runtime functions.
/// The function signature matches how BEAM functions access process and registers.
pub type JitBeamFunction = unsafe extern "C" fn(*mut c_void, *mut Eterm); // Process*, Eterm*

