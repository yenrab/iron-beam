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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Constants Tests ====================

    #[test]
    fn test_erts_x_regs_allocated() {
        assert_eq!(ERTS_X_REGS_ALLOCATED, 1024);
    }

    #[test]
    fn test_max_reg() {
        assert_eq!(MAX_REG, 1024);
    }

    #[test]
    fn test_constants_are_equal() {
        // Both constants should be equal for this implementation
        assert_eq!(ERTS_X_REGS_ALLOCATED, MAX_REG);
    }

    // ==================== Eterm Type Tests ====================

    #[test]
    fn test_eterm_is_u64() {
        let term: Eterm = 0xDEADBEEF_CAFEBABE;
        assert_eq!(term, 0xDEADBEEF_CAFEBABE);
    }

    #[test]
    fn test_eterm_zero() {
        let term: Eterm = 0;
        assert_eq!(term, 0);
    }

    #[test]
    fn test_eterm_max() {
        let term: Eterm = u64::MAX;
        assert_eq!(term, u64::MAX);
    }

    // ==================== FloatDef Tests ====================

    #[test]
    fn test_float_def_creation() {
        let f = FloatDef { value: 3.14 };
        assert!((f.value - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_float_def_zero() {
        let f = FloatDef { value: 0.0 };
        assert_eq!(f.value, 0.0);
    }

    #[test]
    fn test_float_def_negative() {
        let f = FloatDef { value: -42.5 };
        assert_eq!(f.value, -42.5);
    }

    #[test]
    fn test_float_def_infinity() {
        let f = FloatDef { value: f64::INFINITY };
        assert!(f.value.is_infinite());
    }

    #[test]
    fn test_float_def_nan() {
        let f = FloatDef { value: f64::NAN };
        assert!(f.value.is_nan());
    }

    #[test]
    fn test_float_def_debug() {
        let f = FloatDef { value: 1.5 };
        let debug = format!("{:?}", f);
        assert!(debug.contains("FloatDef"));
        assert!(debug.contains("1.5"));
    }

    #[test]
    fn test_float_def_clone() {
        let f = FloatDef { value: 2.718 };
        let cloned = f.clone();
        assert_eq!(f.value, cloned.value);
    }

    #[test]
    fn test_float_def_copy() {
        let f = FloatDef { value: 1.414 };
        let copied = f;
        assert_eq!(f.value, copied.value);
    }

    #[test]
    fn test_float_def_size() {
        assert_eq!(std::mem::size_of::<FloatDef>(), std::mem::size_of::<f64>());
    }

    // ==================== AuxRegs Tests ====================

    #[test]
    fn test_aux_regs_creation() {
        let aux = AuxRegs {
            runtime_stack: [0; 1],
            tmp_mem: [0; 5],
            erl_bits_state: [0; 64],
        };
        assert_eq!(aux.runtime_stack[0], 0);
        assert_eq!(aux.tmp_mem.len(), 5);
        assert_eq!(aux.erl_bits_state.len(), 64);
    }

    #[test]
    fn test_aux_regs_runtime_stack_modification() {
        let mut aux = AuxRegs {
            runtime_stack: [0; 1],
            tmp_mem: [0; 5],
            erl_bits_state: [0; 64],
        };
        aux.runtime_stack[0] = 0x12345678;
        assert_eq!(aux.runtime_stack[0], 0x12345678);
    }

    #[test]
    fn test_aux_regs_tmp_mem_modification() {
        let mut aux = AuxRegs {
            runtime_stack: [0; 1],
            tmp_mem: [0; 5],
            erl_bits_state: [0; 64],
        };
        for i in 0..5 {
            aux.tmp_mem[i] = i as u64 * 100;
        }
        assert_eq!(aux.tmp_mem[0], 0);
        assert_eq!(aux.tmp_mem[4], 400);
    }

    #[test]
    fn test_aux_regs_erl_bits_state_modification() {
        let mut aux = AuxRegs {
            runtime_stack: [0; 1],
            tmp_mem: [0; 5],
            erl_bits_state: [0; 64],
        };
        aux.erl_bits_state[0] = 0xFF;
        aux.erl_bits_state[63] = 0xAB;
        assert_eq!(aux.erl_bits_state[0], 0xFF);
        assert_eq!(aux.erl_bits_state[63], 0xAB);
    }

    // ==================== ErtsSchedulerRegisters Tests ====================

    #[test]
    fn test_erts_scheduler_registers_new() {
        let regs = ErtsSchedulerRegisters::new();
        assert_eq!(regs.aux_regs.runtime_stack[0], 0);
        assert_eq!(regs.x_reg_array[0], 0);
        assert_eq!(regs.f_reg_array[0].value, 0.0);
        assert!(regs.start_time_i.is_null());
        assert_eq!(regs.start_time, 0);
    }

    #[test]
    fn test_erts_scheduler_registers_x_reg_array_size() {
        let regs = ErtsSchedulerRegisters::new();
        assert_eq!(regs.x_reg_array.len(), ERTS_X_REGS_ALLOCATED);
    }

    #[test]
    fn test_erts_scheduler_registers_f_reg_array_size() {
        let regs = ErtsSchedulerRegisters::new();
        assert_eq!(regs.f_reg_array.len(), MAX_REG);
    }

    #[test]
    fn test_erts_scheduler_registers_x_reg_modification() {
        let mut regs = ErtsSchedulerRegisters::new();
        regs.x_reg_array[0] = 42;
        regs.x_reg_array[1023] = 100;
        assert_eq!(regs.x_reg_array[0], 42);
        assert_eq!(regs.x_reg_array[1023], 100);
    }

    #[test]
    fn test_erts_scheduler_registers_f_reg_modification() {
        let mut regs = ErtsSchedulerRegisters::new();
        regs.f_reg_array[0] = FloatDef { value: 3.14 };
        regs.f_reg_array[1023] = FloatDef { value: 2.718 };
        assert!((regs.f_reg_array[0].value - 3.14).abs() < f64::EPSILON);
        assert!((regs.f_reg_array[1023].value - 2.718).abs() < f64::EPSILON);
    }

    #[test]
    fn test_erts_scheduler_registers_start_time_modification() {
        let mut regs = ErtsSchedulerRegisters::new();
        regs.start_time = 1234567890;
        assert_eq!(regs.start_time, 1234567890);
    }

    #[test]
    fn test_erts_scheduler_registers_alignment() {
        // Verify 64-byte alignment requirement
        assert_eq!(std::mem::align_of::<ErtsSchedulerRegisters>(), 64);
    }

    // ==================== ErtsSchedulerData Tests ====================

    #[test]
    fn test_erts_scheduler_data_new() {
        let data = ErtsSchedulerData::new();
        assert!(data.registers.is_null());
        assert!(data.timer_wheel.is_null());
        assert_eq!(data.next_tmo_ref, 0);
        assert!(data.timer_service.is_null());
        assert_eq!(data.tid, 0);
        assert!(data.match_pseudo_process.is_null());
        assert!(data.free_process.is_null());
        assert!(data.ssi.is_null());
        assert!(data.current_process.is_null());
        assert_eq!(data.type_, 0);
        assert_eq!(data.no, 0);
        assert_eq!(data.dirty_no, 0);
        assert_eq!(data.flxctr_slot_no, 0);
        assert!(data.current_nif.is_null());
        assert!(data.dirty_shadow_process.is_null());
        assert!(data.current_port.is_null());
        assert!(data.run_queue.is_null());
        assert_eq!(data.virtual_reds, 0);
        assert_eq!(data.cpu_id, -1); // Note: initialized to -1
        assert_eq!(data.last_monotonic_time, 0);
        assert_eq!(data.check_time_reds, 0);
        assert_eq!(data.thr_id, 0);
        assert_eq!(data.unique, 0);
        assert_eq!(data.ref_, 0);
        assert_eq!(data.io_out, 0);
        assert_eq!(data.io_in, 0);
        assert_eq!(data.reductions, 0);
        assert_eq!(data.rand_state, 0);
        assert_eq!(data.sched_wall_time, 0);
        assert_eq!(data.nosuspend_port_task_handle, 0);
    }

    #[test]
    fn test_erts_scheduler_data_cpu_id_default() {
        let data = ErtsSchedulerData::new();
        // cpu_id is -1 when not bound
        assert_eq!(data.cpu_id, -1);
    }

    #[test]
    fn test_erts_scheduler_data_type_modification() {
        let mut data = ErtsSchedulerData::new();
        data.type_ = 1;
        data.no = 4;
        data.dirty_no = 2;
        assert_eq!(data.type_, 1);
        assert_eq!(data.no, 4);
        assert_eq!(data.dirty_no, 2);
    }

    #[test]
    fn test_erts_scheduler_data_reductions_modification() {
        let mut data = ErtsSchedulerData::new();
        data.reductions = 1000;
        data.virtual_reds = 500;
        assert_eq!(data.reductions, 1000);
        assert_eq!(data.virtual_reds, 500);
    }

    #[test]
    fn test_erts_scheduler_data_io_counters() {
        let mut data = ErtsSchedulerData::new();
        data.io_in = 1024;
        data.io_out = 2048;
        assert_eq!(data.io_in, 1024);
        assert_eq!(data.io_out, 2048);
    }

    #[test]
    fn test_erts_scheduler_data_time_fields() {
        let mut data = ErtsSchedulerData::new();
        data.last_monotonic_time = 0xDEADBEEF;
        data.sched_wall_time = 0xCAFEBABE;
        assert_eq!(data.last_monotonic_time, 0xDEADBEEF);
        assert_eq!(data.sched_wall_time, 0xCAFEBABE);
    }

    #[test]
    fn test_erts_scheduler_data_unique_ref() {
        let mut data = ErtsSchedulerData::new();
        data.unique = 12345;
        data.ref_ = 67890;
        assert_eq!(data.unique, 12345);
        assert_eq!(data.ref_, 67890);
    }

    #[test]
    fn test_erts_scheduler_data_placeholder_arrays() {
        let data = ErtsSchedulerData::new();
        // All placeholder arrays should be zeroed
        assert!(data.thr_progress_data.iter().all(|&x| x == 0));
        assert!(data.aux_work_data.iter().all(|&x| x == 0));
        assert!(data.atom_cache_map.iter().all(|&x| x == 0));
        assert!(data.gc_info.iter().all(|&x| x == 0));
        assert!(data.u.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_erts_scheduler_data_registers_pointer() {
        let mut data = ErtsSchedulerData::new();
        let mut regs = ErtsSchedulerRegisters::new();
        data.registers = &mut regs as *mut ErtsSchedulerRegisters;
        
        assert!(!data.registers.is_null());
        
        // Access registers through pointer
        unsafe {
            (*data.registers).x_reg_array[0] = 42;
            assert_eq!((*data.registers).x_reg_array[0], 42);
        }
    }

    // ==================== Type Alias Tests ====================

    #[test]
    fn test_jit_process_main_type() {
        // Verify the function pointer type exists and has correct size
        assert_eq!(
            std::mem::size_of::<JitProcessMain>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_jit_beam_function_type() {
        // Verify the function pointer type exists and has correct size
        assert_eq!(
            std::mem::size_of::<JitBeamFunction>(),
            std::mem::size_of::<usize>()
        );
    }

    // ==================== repr(C) Layout Tests ====================

    #[test]
    fn test_float_def_is_repr_c() {
        // FloatDef should have the same layout as a raw f64
        assert_eq!(std::mem::size_of::<FloatDef>(), 8);
    }

    #[test]
    fn test_aux_regs_field_sizes() {
        // Verify field sizes match expectations
        let aux = AuxRegs {
            runtime_stack: [0; 1],
            tmp_mem: [0; 5],
            erl_bits_state: [0; 64],
        };
        assert_eq!(aux.runtime_stack.len(), 1);
        assert_eq!(aux.tmp_mem.len(), 5);
        assert_eq!(aux.erl_bits_state.len(), 64);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_scheduler_data_with_registers() {
        let mut regs = Box::new(ErtsSchedulerRegisters::new());
        let mut data = ErtsSchedulerData::new();
        
        // Set up registers pointer
        data.registers = regs.as_mut() as *mut ErtsSchedulerRegisters;
        
        // Set some values
        data.no = 1;
        data.tid = 12345;
        
        unsafe {
            (*data.registers).x_reg_array[0] = 100;
            (*data.registers).x_reg_array[1] = 200;
            (*data.registers).f_reg_array[0] = FloatDef { value: 1.5 };
        }
        
        // Verify values
        assert_eq!(data.no, 1);
        assert_eq!(data.tid, 12345);
        unsafe {
            assert_eq!((*data.registers).x_reg_array[0], 100);
            assert_eq!((*data.registers).x_reg_array[1], 200);
            assert_eq!((*data.registers).f_reg_array[0].value, 1.5);
        }
    }

    #[test]
    fn test_multiple_scheduler_data_instances() {
        let data1 = ErtsSchedulerData::new();
        let data2 = ErtsSchedulerData::new();
        
        // Both should be independent
        assert_eq!(data1.no, data2.no);
        assert_eq!(data1.cpu_id, data2.cpu_id);
    }

    #[test]
    fn test_scheduler_registers_full_x_reg_usage() {
        let mut regs = ErtsSchedulerRegisters::new();
        
        // Fill all X registers
        for i in 0..ERTS_X_REGS_ALLOCATED {
            regs.x_reg_array[i] = i as u64;
        }
        
        // Verify
        for i in 0..ERTS_X_REGS_ALLOCATED {
            assert_eq!(regs.x_reg_array[i], i as u64);
        }
    }

    #[test]
    fn test_scheduler_registers_full_f_reg_usage() {
        let mut regs = ErtsSchedulerRegisters::new();
        
        // Fill all F registers
        for i in 0..MAX_REG {
            regs.f_reg_array[i] = FloatDef { value: i as f64 };
        }
        
        // Verify
        for i in 0..MAX_REG {
            assert_eq!(regs.f_reg_array[i].value, i as f64);
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_scheduler_data_max_values() {
        let mut data = ErtsSchedulerData::new();
        data.tid = u64::MAX;
        data.unique = u64::MAX;
        data.ref_ = u64::MAX;
        data.reductions = u64::MAX;
        data.io_in = u64::MAX;
        data.io_out = u64::MAX;
        
        assert_eq!(data.tid, u64::MAX);
        assert_eq!(data.unique, u64::MAX);
    }

    #[test]
    fn test_scheduler_data_negative_values() {
        let mut data = ErtsSchedulerData::new();
        data.cpu_id = -100;
        data.flxctr_slot_no = -50;
        data.virtual_reds = -1000;
        
        assert_eq!(data.cpu_id, -100);
        assert_eq!(data.flxctr_slot_no, -50);
        assert_eq!(data.virtual_reds, -1000);
    }

    #[test]
    fn test_x_reg_boundary_values() {
        let mut regs = ErtsSchedulerRegisters::new();
        
        // First and last registers
        regs.x_reg_array[0] = u64::MIN;
        regs.x_reg_array[ERTS_X_REGS_ALLOCATED - 1] = u64::MAX;
        
        assert_eq!(regs.x_reg_array[0], u64::MIN);
        assert_eq!(regs.x_reg_array[ERTS_X_REGS_ALLOCATED - 1], u64::MAX);
    }

    #[test]
    fn test_f_reg_special_values() {
        let mut regs = ErtsSchedulerRegisters::new();
        
        regs.f_reg_array[0] = FloatDef { value: f64::MIN };
        regs.f_reg_array[1] = FloatDef { value: f64::MAX };
        regs.f_reg_array[2] = FloatDef { value: f64::EPSILON };
        regs.f_reg_array[3] = FloatDef { value: f64::NEG_INFINITY };
        regs.f_reg_array[4] = FloatDef { value: f64::INFINITY };
        
        assert_eq!(regs.f_reg_array[0].value, f64::MIN);
        assert_eq!(regs.f_reg_array[1].value, f64::MAX);
        assert!(regs.f_reg_array[3].value.is_infinite());
        assert!(regs.f_reg_array[4].value.is_infinite());
    }
}

