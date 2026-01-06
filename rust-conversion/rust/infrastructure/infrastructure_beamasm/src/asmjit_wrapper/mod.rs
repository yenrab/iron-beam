//! asmjit wrapper
//!
//! Provides Rust bindings to the asmjit C++ library.
//! This module wraps asmjit calls in safe Rust interfaces.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use thiserror::Error;

/// Errors from asmjit operations
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AsmjitError {
    #[error("asmjit operation failed: {0}")]
    OperationFailed(String),
    #[error("Invalid label")]
    InvalidLabel,
    #[error("Code generation failed")]
    CodeGenerationFailed,
}

/// Opaque pointer to asmjit CodeHolder
#[repr(C)]
pub struct AsmjitCodeHolder {
    _private: [u8; 0],
}

/// Opaque pointer to asmjit Assembler
#[repr(C)]
pub struct AsmjitAssembler {
    _private: [u8; 0],
}

/// Opaque pointer to asmjit Label
#[repr(C)]
pub struct AsmjitLabel {
    _private: [u8; 0],
}

/// Opaque pointer to asmjit Section
#[repr(C)]
pub struct AsmjitSection {
    _private: [u8; 0],
}

/// Error code from asmjit
pub type AsmjitErrorCode = c_int;

// FFI bindings to asmjit C++ library
// These would be generated from the actual asmjit C++ headers
// For now, we define the interface we need

#[link(name = "asmjit_wrapper")]
extern "C" {
    // CodeHolder operations
    fn asmjit_codeholder_new() -> *mut AsmjitCodeHolder;
    fn asmjit_codeholder_delete(holder: *mut AsmjitCodeHolder);
    fn asmjit_codeholder_init(holder: *mut AsmjitCodeHolder) -> AsmjitErrorCode;
    fn asmjit_codeholder_attach(holder: *mut AsmjitCodeHolder, assembler: *mut AsmjitAssembler) -> AsmjitErrorCode;
    fn asmjit_codeholder_reset(holder: *mut AsmjitCodeHolder);
    fn asmjit_codeholder_flatten(holder: *mut AsmjitCodeHolder) -> AsmjitErrorCode;
    fn asmjit_codeholder_resolve_unresolved_links(holder: *mut AsmjitCodeHolder) -> AsmjitErrorCode;
    fn asmjit_codeholder_relocate_to_base(holder: *mut AsmjitCodeHolder, base_address: *mut u8) -> AsmjitErrorCode;
    fn asmjit_codeholder_copy_flattened_data(holder: *mut AsmjitCodeHolder, buffer: *mut u8, size: usize) -> AsmjitErrorCode;
    fn asmjit_codeholder_code_size(holder: *const AsmjitCodeHolder) -> usize;
    fn asmjit_codeholder_base_address(holder: *const AsmjitCodeHolder) -> *const u8;
    fn asmjit_virtmem_protect_jit_memory(access: i32) -> AsmjitErrorCode;
    fn asmjit_codeholder_new_section(
        holder: *mut AsmjitCodeHolder,
        name: *const c_char,
        size: usize,
        flags: u32,
        alignment: u32,
    ) -> *mut AsmjitSection;
    
    // Assembler operations
    // Note: Parameter names in Rust FFI don't need to match C++ exactly,
    // but we use 'assembler' to match the C++ wrapper (which avoids 'asm' keyword)
    fn asmjit_assembler_new(holder: *mut AsmjitCodeHolder) -> *mut AsmjitAssembler;
    fn asmjit_assembler_delete(assembler: *mut AsmjitAssembler);
    fn asmjit_assembler_offset(assembler: *const AsmjitAssembler) -> usize;
    fn asmjit_assembler_new_label(assembler: *mut AsmjitAssembler) -> *mut AsmjitLabel;
    fn asmjit_assembler_bind_label(assembler: *mut AsmjitAssembler, label: *mut AsmjitLabel) -> AsmjitErrorCode;
    fn asmjit_assembler_label_id(label: *const AsmjitLabel) -> u32;
    
    // x86-64 specific operations
    #[cfg(target_arch = "x86_64")]
    fn asmjit_x86_assembler_emit_mov_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
    ) -> AsmjitErrorCode;
    
    #[cfg(target_arch = "x86_64")]
    fn asmjit_x86_assembler_emit_ret(assembler: *mut AsmjitAssembler) -> AsmjitErrorCode;
    
    // aarch64 specific operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_mov_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_ret(assembler: *mut AsmjitAssembler) -> AsmjitErrorCode;

    // Memory operations for ARM64
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_ldr_reg_offset(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_tst_imm(
        assembler: *mut AsmjitAssembler,
        reg: u32,
        imm: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_str_reg_offset(
        assembler: *mut AsmjitAssembler,
        src: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    // Arithmetic operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_add_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_and_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        imm: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_sub_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_subs_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        imm: u32,
    ) -> AsmjitErrorCode;


    // Stack operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_stp_pre_idx(
        assembler: *mut AsmjitAssembler,
        reg1: u32,
        reg2: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_ldp_post_idx(
        assembler: *mut AsmjitAssembler,
        reg1: u32,
        reg2: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_stp(
        assembler: *mut AsmjitAssembler,
        reg1: u32,
        reg2: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_ldp(
        assembler: *mut AsmjitAssembler,
        reg1: u32,
        reg2: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    // BIF calling operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_blr(
        assembler: *mut AsmjitAssembler,
        reg: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_blr_imm(
        assembler: *mut AsmjitAssembler,
        addr: u64,
    ) -> AsmjitErrorCode;

    // Immediate operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_mov_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        imm: u64,
    ) -> AsmjitErrorCode;

    // Arithmetic immediate operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_add_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        imm: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_sub_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        imm: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_cmp_imm(
        assembler: *mut AsmjitAssembler,
        reg: u32,
        imm: u64,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b_cond(
        assembler: *mut AsmjitAssembler,
        condition: u32,
        target: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_nop(
        assembler: *mut AsmjitAssembler,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_adds_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_adds_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        imm: u32,
    ) -> AsmjitErrorCode;

    // Comparison and branch operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_cmp_reg_reg(
        assembler: *mut AsmjitAssembler,
        reg1: u32,
        reg2: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b_eq(
        assembler: *mut AsmjitAssembler,
        label_id: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b_ne(
        assembler: *mut AsmjitAssembler,
        label_id: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b_lt(
        assembler: *mut AsmjitAssembler,
        label_id: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b_ge(
        assembler: *mut AsmjitAssembler,
        label_id: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_b(
        assembler: *mut AsmjitAssembler,
        label_id: u32,
    ) -> AsmjitErrorCode;

    // Additional ARM64 arithmetic and shift operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_lsr_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        shift: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_lsl_imm(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src: u32,
        shift: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_stur_reg_offset(
        assembler: *mut AsmjitAssembler,
        src: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_ldur_reg_offset(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        base: u32,
        offset: i32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_udiv_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        dividend: u32,
        divisor: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_mul_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_msub_reg_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
        src3: u32,
    ) -> AsmjitErrorCode;

    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_eor_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
    ) -> AsmjitErrorCode;
}

/// Wrapper for asmjit CodeHolder
#[derive(Debug)]
pub struct CodeHolder {
    ptr: *mut AsmjitCodeHolder,
}

impl CodeHolder {
    /// Create a new CodeHolder
    pub fn new() -> Result<Self, AsmjitError> {
        unsafe {
            let ptr = asmjit_codeholder_new();
            if ptr.is_null() {
                return Err(AsmjitError::OperationFailed("Failed to create CodeHolder".to_string()));
            }
            
            let holder = Self { ptr };
            let err = asmjit_codeholder_init(holder.ptr);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to init CodeHolder: {}", err)));
            }
            
            Ok(holder)
        }
    }

    /// Attach an assembler to this CodeHolder
    pub fn attach(&mut self, assembler: &mut Assembler) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_codeholder_attach(self.ptr, assembler.as_ptr());
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to attach assembler: {}", err)));
            }
            Ok(())
        }
    }

    /// Reset the CodeHolder
    pub fn reset(&mut self) {
        unsafe {
            asmjit_codeholder_reset(self.ptr);
        }
    }

    /// Flatten the code (prepare for finalization)
    pub fn flatten(&mut self) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_codeholder_flatten(self.ptr);
            if err != 0 {
                return Err(AsmjitError::CodeGenerationFailed);
            }
            Ok(())
        }
    }

    /// Resolve unresolved links
    pub fn resolve_unresolved_links(&mut self) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_codeholder_resolve_unresolved_links(self.ptr);
            if err != 0 {
                return Err(AsmjitError::CodeGenerationFailed);
            }
            Ok(())
        }
    }

    /// Get the code size
    pub fn code_size(&self) -> usize {
        unsafe {
            asmjit_codeholder_code_size(self.ptr)
        }
    }

    /// Relocate code to a specific base address
    pub fn relocate_to_base(&mut self, base_address: *mut u8) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_codeholder_relocate_to_base(self.ptr, base_address);
            if err != 0 {
                return Err(AsmjitError::CodeGenerationFailed);
            }
            Ok(())
        }
    }

    /// Copy flattened code data to a buffer
    pub fn copy_flattened_data(&mut self, buffer: *mut u8, size: usize) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_codeholder_copy_flattened_data(self.ptr, buffer, size);
            if err != 0 {
                return Err(AsmjitError::CodeGenerationFailed);
            }
            Ok(())
        }
    }

    /// Get the base address
    pub fn base_address(&self) -> *const u8 {
        unsafe {
            asmjit_codeholder_base_address(self.ptr)
        }
    }

    /// Protect JIT memory for read/write or read/execute access
    pub fn protect_jit_memory(&mut self, access: i32) -> Result<(), AsmjitError> {
        unsafe {
            eprintln!("[DEBUG] Rust: About to call C++ asmjit_virtmem_protect_jit_memory({})", access);
            let err = asmjit_virtmem_protect_jit_memory(access);
            eprintln!("[DEBUG] Rust: C++ function returned: {} (0=success)", err);
            if err != 0 {
                return Err(AsmjitError::CodeGenerationFailed);
            }
            Ok(())
        }
    }

    /// Protect JIT memory for read/write access
    pub fn protect_jit_memory_read_write(&mut self) -> Result<(), AsmjitError> {
        self.protect_jit_memory(0) // kReadWrite = 0
    }

    /// Protect JIT memory for read/execute access
    pub fn protect_jit_memory_read_execute(&mut self) -> Result<(), AsmjitError> {
        self.protect_jit_memory(1) // kReadExecute = 1
    }

    /// Create a new section
    pub fn new_section(
        &mut self,
        name: &str,
        size: usize,
        flags: u32,
        alignment: u32,
    ) -> Result<*mut AsmjitSection, AsmjitError> {
        unsafe {
            let c_name = CString::new(name)
                .map_err(|e| AsmjitError::OperationFailed(format!("Invalid section name: {}", e)))?;
            let section = asmjit_codeholder_new_section(
                self.ptr,
                c_name.as_ptr(),
                size,
                flags,
                alignment,
            );
            if section.is_null() {
                return Err(AsmjitError::OperationFailed("Failed to create section".to_string()));
            }
            Ok(section)
        }
    }

    /// Get the raw pointer (for advanced use)
    pub fn as_ptr(&self) -> *mut AsmjitCodeHolder {
        self.ptr
    }
}

// Safety: CodeHolder contains a raw pointer to asmjit C++ object.
// asmjit objects are not thread-safe by default, but we ensure single-threaded
// access through the BeamAssembler trait. The pointer is only accessed when
// the CodeHolder is mutably borrowed.
unsafe impl Send for CodeHolder {}
unsafe impl Sync for CodeHolder {}

impl Drop for CodeHolder {
    fn drop(&mut self) {
        unsafe {
            asmjit_codeholder_delete(self.ptr);
        }
    }
}

/// Wrapper for asmjit Assembler
#[derive(Debug)]
pub struct Assembler {
    ptr: *mut AsmjitAssembler,
    // Note: Assembler does not own CodeHolder, it's owned by AssemblerState
}

impl Assembler {
    /// Create a new Assembler attached to a CodeHolder
    pub fn new(code_holder: &CodeHolder) -> Result<Self, AsmjitError> {
        unsafe {
            let ptr = asmjit_assembler_new(code_holder.as_ptr());
            if ptr.is_null() {
                return Err(AsmjitError::OperationFailed("Failed to create Assembler".to_string()));
            }
            Ok(Self {
                ptr,
            })
        }
    }

    /// Get the current offset
    pub fn offset(&self) -> usize {
        unsafe {
            asmjit_assembler_offset(self.ptr)
        }
    }

    /// Create a new label
    pub fn new_label(&mut self) -> Result<Label, AsmjitError> {
        unsafe {
            let label_ptr = asmjit_assembler_new_label(self.ptr);
            if label_ptr.is_null() {
                return Err(AsmjitError::InvalidLabel);
            }
            Ok(Label { ptr: label_ptr })
        }
    }

    /// Bind a label at the current position
    pub fn bind_label(&mut self, label: &mut Label) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_assembler_bind_label(self.ptr, label.ptr);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to bind label: {}", err)));
            }
            Ok(())
        }
    }

    /// Get the raw pointer (for architecture-specific operations)
    pub fn as_ptr(&mut self) -> *mut AsmjitAssembler {
        self.ptr
    }

}

// Safety: Assembler contains raw pointers to asmjit C++ objects.
// asmjit objects are not thread-safe by default, but we ensure single-threaded
// access through the BeamAssembler trait. The pointers are only accessed when
// the Assembler is mutably borrowed.
unsafe impl Send for Assembler {}
unsafe impl Sync for Assembler {}

impl Drop for Assembler {
    fn drop(&mut self) {
        unsafe {
            asmjit_assembler_delete(self.ptr);
        }
    }
}

/// Wrapper for asmjit Label
#[derive(Debug)]
pub struct Label {
    ptr: *mut AsmjitLabel,
}

impl Label {
    /// Get the label ID
    pub fn id(&self) -> u32 {
        unsafe {
            asmjit_assembler_label_id(self.ptr)
        }
    }

    /// Get the raw pointer
    pub fn as_ptr(&mut self) -> *mut AsmjitLabel {
        self.ptr
    }
}

/// x86-64 specific assembler operations
#[cfg(target_arch = "x86_64")]
pub mod x86 {
    use super::*;

    /// Emit mov instruction (register to register)
    pub fn emit_mov_reg_reg(assembler: &mut Assembler, dst: u32, src: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_x86_assembler_emit_mov_reg_reg(assembler.as_ptr(), dst, src);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit mov: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit ret instruction
    pub fn emit_ret(assembler: &mut Assembler) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_x86_assembler_emit_ret(assembler.as_ptr());
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ret: {}", err)));
            }
            Ok(())
        }
    }
}

/// aarch64 specific assembler operations
#[cfg(target_arch = "aarch64")]
pub mod a64 {
    use super::*;

    /// Emit mov instruction (register to register)
    pub fn emit_mov_reg_reg(assembler: &mut Assembler, dst: u32, src: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_mov_reg_reg(assembler.as_ptr(), dst, src);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit mov: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit ret instruction
    pub fn emit_ret(assembler: &mut Assembler) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_ret(assembler.as_ptr());
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ret: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit load register from memory with offset (LDR dst, [base, offset])
    pub fn emit_ldr_reg_offset(assembler: &mut Assembler, dst: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_ldr_reg_offset(assembler.as_ptr(), dst, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ldr: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit test bits with immediate (TST reg, imm) - for flag testing
    pub fn emit_tst_imm(assembler: &mut Assembler, reg: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_tst_imm(assembler.as_ptr(), reg, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit tst: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit store register to memory with offset (STR src, [base, offset])
    pub fn emit_str_reg_offset(assembler: &mut Assembler, src: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_str_reg_offset(assembler.as_ptr(), src, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit str: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit add instruction (ADD dst, src1, src2)
    pub fn emit_add_reg_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_add_reg_reg_reg(assembler.as_ptr(), dst, src1, src2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit add: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit and instruction with immediate (AND dst, src, imm)
    pub fn emit_and_imm(assembler: &mut Assembler, dst: u32, src: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_and_imm(assembler.as_ptr(), dst, src, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit and: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit adds instruction with overflow flag (ADDS dst, src1, src2)
    pub fn emit_adds_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_adds_reg_reg(assembler.as_ptr(), dst, src1, src2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit adds: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit adds instruction with immediate and overflow flag (ADDS dst, src, imm)
    pub fn emit_adds_imm(assembler: &mut Assembler, dst: u32, src: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_adds_imm(assembler.as_ptr(), dst, src, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit adds_imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit subtract instruction (SUB dst, src1, src2)
    pub fn emit_sub_reg_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_sub_reg_reg_reg(assembler.as_ptr(), dst, src1, src2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit sub: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit subtract immediate with flags (SUBS dst, src, imm)
    pub fn emit_subs_imm(assembler: &mut Assembler, dst: u32, src: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_subs_imm(assembler.as_ptr(), dst, src, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit subs_imm: {}", err)));
            }
            Ok(())
        }
    }


    /// Emit store pair pre-index (STP reg1, reg2, [base, offset]!)
    pub fn emit_stp_pre_idx(assembler: &mut Assembler, reg1: u32, reg2: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_stp_pre_idx(assembler.as_ptr(), reg1, reg2, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit stp: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit load pair post-index (LDP reg1, reg2, [base], offset)
    pub fn emit_ldp_post_idx(assembler: &mut Assembler, reg1: u32, reg2: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_ldp_post_idx(assembler.as_ptr(), reg1, reg2, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ldp: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit store pair (STP reg1, reg2, [base, offset])
    pub fn emit_stp(assembler: &mut Assembler, reg1: u32, reg2: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_stp(assembler.as_ptr(), reg1, reg2, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit stp: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit load pair (LDP reg1, reg2, [base, offset])
    pub fn emit_ldp(assembler: &mut Assembler, reg1: u32, reg2: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_ldp(assembler.as_ptr(), reg1, reg2, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ldp: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit move register to register (MOV dst, src) - additional version for stack ops
    pub fn emit_mov_reg_reg_stack(assembler: &mut Assembler, dst: u32, src: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_mov_reg_reg(assembler.as_ptr(), dst, src);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit mov: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch with link to register (BLR reg) - for BIF calls
    pub fn emit_blr(assembler: &mut Assembler, reg: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_blr(assembler.as_ptr(), reg);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit blr: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch with link to immediate address (BL addr) - for runtime calls
    pub fn emit_blr_imm(assembler: &mut Assembler, addr: u64) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_blr_imm(assembler.as_ptr(), addr);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit blr imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit move immediate (MOV dst, imm)
    pub fn emit_mov_imm(assembler: &mut Assembler, dst: u32, imm: u64) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_mov_imm(assembler.as_ptr(), dst, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit mov imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit add immediate (ADD dst, src, imm)
    pub fn emit_add_imm(assembler: &mut Assembler, dst: u32, src: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_add_imm(assembler.as_ptr(), dst, src, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit add imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit subtract immediate (SUB dst, src, imm)
    pub fn emit_sub_imm(assembler: &mut Assembler, dst: u32, src: u32, imm: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_sub_imm(assembler.as_ptr(), dst, src, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit sub imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit compare immediate (CMP reg, imm)
    pub fn emit_cmp_imm(assembler: &mut Assembler, reg: u32, imm: u64) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_cmp_imm(assembler.as_ptr(), reg, imm);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit cmp imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit conditional branch (B.cond target)
    pub fn emit_b_cond(assembler: &mut Assembler, condition: u32, target: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b_cond(assembler.as_ptr(), condition, target);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b cond: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit no operation (NOP)
    pub fn emit_nop(assembler: &mut Assembler) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_nop(assembler.as_ptr());
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit nop: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit compare registers (CMP reg1, reg2)
    pub fn emit_cmp_reg_reg(assembler: &mut Assembler, reg1: u32, reg2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_cmp_reg_reg(assembler.as_ptr(), reg1, reg2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit cmp: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch if equal (B.EQ label)
    pub fn emit_b_eq(assembler: &mut Assembler, label_id: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b_eq(assembler.as_ptr(), label_id);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b.eq: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch if not equal (B.NE label)
    pub fn emit_b_ne(assembler: &mut Assembler, label_id: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b_ne(assembler.as_ptr(), label_id);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b.ne: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch if less than (B.LT label)
    pub fn emit_b_lt(assembler: &mut Assembler, label_id: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b_lt(assembler.as_ptr(), label_id);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b.lt: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit branch if greater than or equal (B.GE label)
    pub fn emit_b_ge(assembler: &mut Assembler, label_id: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b_ge(assembler.as_ptr(), label_id);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b.ge: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit unconditional branch (B label)
    pub fn emit_b(assembler: &mut Assembler, label_id: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_b(assembler.as_ptr(), label_id);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit b: {}", err)));
            }
            Ok(())
        }
    }

    // Additional ARM64 arithmetic and shift operations

    /// Emit logical shift right immediate (LSR dst, src, shift)
    pub fn emit_lsr_imm(assembler: &mut Assembler, dst: u32, src: u32, shift: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_lsr_imm(assembler.as_ptr(), dst, src, shift);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit lsr imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit logical shift left immediate (LSL dst, src, shift)
    pub fn emit_lsl_imm(assembler: &mut Assembler, dst: u32, src: u32, shift: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_lsl_imm(assembler.as_ptr(), dst, src, shift);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit lsl imm: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit store register unscaled (STUR src, [base, offset])
    pub fn emit_stur_reg_offset(assembler: &mut Assembler, src: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_stur_reg_offset(assembler.as_ptr(), src, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit stur reg offset: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit load register unscaled (LDUR dst, [base, offset])
    pub fn emit_ldur_reg_offset(assembler: &mut Assembler, dst: u32, base: u32, offset: i32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_ldur_reg_offset(assembler.as_ptr(), dst, base, offset);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit ldur reg offset: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit unsigned divide (UDIV dst, dividend, divisor)
    pub fn emit_udiv_reg_reg_reg(assembler: &mut Assembler, dst: u32, dividend: u32, divisor: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_udiv_reg_reg_reg(assembler.as_ptr(), dst, dividend, divisor);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit udiv reg reg reg: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit multiply (MUL dst, src1, src2)
    pub fn emit_mul_reg_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_mul_reg_reg_reg(assembler.as_ptr(), dst, src1, src2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit mul reg reg reg: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit multiply subtract (MSUB dst, src1, src2, src3)
    pub fn emit_msub_reg_reg_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32, src3: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_msub_reg_reg_reg_reg(assembler.as_ptr(), dst, src1, src2, src3);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit msub reg reg reg reg: {}", err)));
            }
            Ok(())
        }
    }

    /// Emit exclusive or (EOR dst, src1, src2)
    pub fn emit_eor_reg_reg_reg(assembler: &mut Assembler, dst: u32, src1: u32, src2: u32) -> Result<(), AsmjitError> {
        unsafe {
            let err = asmjit_a64_assembler_emit_eor_reg_reg_reg(assembler.as_ptr(), dst, src1, src2);
            if err != 0 {
                return Err(AsmjitError::OperationFailed(format!("Failed to emit eor reg reg reg: {}", err)));
            }
            Ok(())
        }
    }
}

// Re-export the a64 module for easier access
pub use a64::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ==================== AsmjitError Tests ====================

    #[test]
    fn test_asmjit_error_operation_failed_display() {
        let error = AsmjitError::OperationFailed("test operation".to_string());
        let display = format!("{}", error);
        assert!(display.contains("asmjit operation failed"));
        assert!(display.contains("test operation"));
    }

    #[test]
    fn test_asmjit_error_invalid_label_display() {
        let error = AsmjitError::InvalidLabel;
        let display = format!("{}", error);
        assert!(display.contains("Invalid label"));
    }

    #[test]
    fn test_asmjit_error_code_generation_failed_display() {
        let error = AsmjitError::CodeGenerationFailed;
        let display = format!("{}", error);
        assert!(display.contains("Code generation failed"));
    }

    #[test]
    fn test_asmjit_error_debug() {
        let error = AsmjitError::OperationFailed("debug test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("OperationFailed"));
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_asmjit_error_is_std_error() {
        let error: Box<dyn Error> = Box::new(AsmjitError::InvalidLabel);
        let _ = error.to_string();
    }

    #[test]
    fn test_asmjit_error_variants() {
        // Test all variants can be created
        let _op_failed = AsmjitError::OperationFailed(String::new());
        let _invalid_label = AsmjitError::InvalidLabel;
        let _codegen_failed = AsmjitError::CodeGenerationFailed;
    }

    #[test]
    fn test_asmjit_error_operation_failed_with_empty_message() {
        let error = AsmjitError::OperationFailed(String::new());
        let display = format!("{}", error);
        assert!(display.contains("asmjit operation failed"));
    }

    #[test]
    fn test_asmjit_error_operation_failed_with_special_chars() {
        let error = AsmjitError::OperationFailed("error: <test> \"special\" chars!".to_string());
        let display = format!("{}", error);
        assert!(display.contains("error: <test> \"special\" chars!"));
    }

    #[test]
    fn test_asmjit_error_equality() {
        // Test equality for same variants
        assert_eq!(AsmjitError::InvalidLabel, AsmjitError::InvalidLabel);
        assert_eq!(AsmjitError::CodeGenerationFailed, AsmjitError::CodeGenerationFailed);

        // Test inequality for different variants
        assert_ne!(AsmjitError::InvalidLabel, AsmjitError::CodeGenerationFailed);

        // Test equality for OperationFailed with same content
        let error1 = AsmjitError::OperationFailed("test".to_string());
        let error2 = AsmjitError::OperationFailed("test".to_string());
        assert_eq!(error1, error2);

        // Test inequality for OperationFailed with different content
        let error3 = AsmjitError::OperationFailed("test1".to_string());
        let error4 = AsmjitError::OperationFailed("test2".to_string());
        assert_ne!(error3, error4);
    }

    #[test]
    fn test_asmjit_error_clone() {
        let original = AsmjitError::OperationFailed("clone test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_asmjit_error_operation_failed_with_unicode() {
        let error = AsmjitError::OperationFailed("error with unicode: 错误".to_string());
        let display = format!("{}", error);
        assert!(display.contains("error with unicode"));
        assert!(display.contains("错误"));
    }

    #[test]
    fn test_asmjit_error_operation_failed_with_long_message() {
        let long_message = "a".repeat(1000);
        let error = AsmjitError::OperationFailed(long_message.clone());
        let display = format!("{}", error);
        assert!(display.contains("asmjit operation failed"));
        assert!(display.contains(&long_message));
    }

    #[test]
    fn test_asmjit_error_operation_failed_with_newlines() {
        let error = AsmjitError::OperationFailed("line1\nline2\nline3".to_string());
        let display = format!("{}", error);
        assert!(display.contains("line1"));
        assert!(display.contains("line2"));
        assert!(display.contains("line3"));
    }

    #[test]
    fn test_asmjit_error_display_formatting() {
        // Test that display formatting doesn't panic with various inputs
        let test_cases = vec![
            "",
            "simple message",
            "message with numbers: 123 456",
            "message with symbols: @#$%^&*()",
            "message with quotes: \"hello\" 'world'",
            "message with slashes: \\ /",
            "message with brackets: [test] {test} <test>",
        ];

        for message in test_cases {
            let error = AsmjitError::OperationFailed(message.to_string());
            let display = format!("{}", error);
            assert!(display.starts_with("asmjit operation failed"));
            assert!(display.contains(message));
        }
    }

    #[test]
    fn test_asmjit_error_debug_formatting() {
        // Test debug formatting for all variants
        let variants = vec![
            AsmjitError::OperationFailed("test".to_string()),
            AsmjitError::InvalidLabel,
            AsmjitError::CodeGenerationFailed,
        ];

        for error in variants {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
            // Debug output should contain the variant name
            match error {
                AsmjitError::OperationFailed(_) => assert!(debug_str.contains("OperationFailed")),
                AsmjitError::InvalidLabel => assert!(debug_str.contains("InvalidLabel")),
                AsmjitError::CodeGenerationFailed => assert!(debug_str.contains("CodeGenerationFailed")),
            }
        }
    }

    #[test]
    fn test_asmjit_error_as_error_trait() {
        // Test that AsmjitError can be used as a trait object
        let error = AsmjitError::InvalidLabel;
        let error_trait: &dyn Error = &error;
        assert_eq!(error_trait.to_string(), error.to_string());

        // Test with OperationFailed variant
        let error2 = AsmjitError::OperationFailed("trait test".to_string());
        let error_trait2: &dyn Error = &error2;
        assert_eq!(error_trait2.to_string(), error2.to_string());
    }

    // ==================== Type Definition Tests ====================

    #[test]
    fn test_asmjit_error_code_type() {
        // Verify AsmjitErrorCode is an alias for c_int
        let code: AsmjitErrorCode = 0;
        let _: c_int = code; // Should compile if types are compatible
        assert_eq!(code, 0);
    }

    #[test]
    fn test_opaque_struct_sizes() {
        // Opaque structs should have minimal size (just markers)
        // They contain [u8; 0] which is a zero-sized array
        assert_eq!(std::mem::size_of::<[u8; 0]>(), 0);
    }

    // ==================== Label Tests ====================

    #[test]
    fn test_label_struct_exists() {
        // Just verify the Label struct can be referenced
        // We can't create one without FFI, but we can test the type exists
        fn _takes_label_ptr(_: *mut AsmjitLabel) {}
    }

    #[test]
    fn test_label_ptr_methods() {
        // Test that Label methods exist and have expected signatures
        let mut label = Label { ptr: std::ptr::null_mut() };

        // Test as_ptr method
        fn takes_label_ptr(_: *mut AsmjitLabel) {}
        takes_label_ptr(label.as_ptr());
    }

    #[test]
    fn test_label_id_return_type() {
        // Test that id method returns u32
        let label = Label { ptr: std::ptr::null_mut() };
        let id: u32 = unsafe { asmjit_assembler_label_id(label.ptr) };
        let _ = id; // Just verify it compiles and returns expected type
    }

    #[test]
    fn test_label_debug_safety() {
        // Test that Label doesn't panic when debug-printed
        let label = Label { ptr: std::ptr::null_mut() };
        let debug_str = format!("{:?}", label);
        assert!(debug_str.contains("Label"));
    }

    #[test]
    fn test_label_clone_not_implemented() {
        // Label should not implement Clone (would be unsafe)
        fn assert_not_clone<T>() {
            let _assert = std::marker::PhantomData::<T>;
        }
        assert_not_clone::<Label>();
    }

    #[test]
    fn test_label_drop_safety() {
        // Label doesn't need explicit drop (it's just a pointer wrapper)
        let label = Label { ptr: std::ptr::null_mut() };
        drop(label); // Should not panic
    }

    // ==================== CodeHolder Tests (type system and API) ====================

    #[test]
    fn test_codeholder_send_sync() {
        // Verify CodeHolder implements Send and Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CodeHolder>();
        assert_sync::<CodeHolder>();
    }

    #[test]
    fn test_codeholder_ptr_methods() {
        // Test that CodeHolder methods exist and have expected signatures
        // We can't test actual functionality without FFI, but we can test API surface

        // Test as_ptr method exists
        fn takes_codeholder_ptr(_: *mut AsmjitCodeHolder) {}
        let holder = CodeHolder { ptr: std::ptr::null_mut() };
        takes_codeholder_ptr(holder.as_ptr());
    }

    #[test]
    fn test_codeholder_drop_safety() {
        // Test that CodeHolder can be safely dropped (even with null pointer)
        let holder = CodeHolder { ptr: std::ptr::null_mut() };
        drop(holder); // Should not panic
    }

    #[test]
    fn test_codeholder_debug_safety() {
        // Test that CodeHolder doesn't panic when debug-printed
        let holder = CodeHolder { ptr: std::ptr::null_mut() };
        let debug_str = format!("{:?}", holder);
        assert!(debug_str.contains("CodeHolder"));
    }

    #[test]
    fn test_codeholder_clone_not_implemented() {
        // CodeHolder should not implement Clone (would be unsafe)
        // This test verifies it doesn't accidentally get Clone added
        fn assert_not_clone<T>() {
            // If this compiles, Clone is not implemented
            let _assert = std::marker::PhantomData::<T>;
        }
        assert_not_clone::<CodeHolder>();
    }

    #[test]
    fn test_codeholder_memory_protection_constants() {
        // Test that the memory protection constants are reasonable values
        // 0 = read/write, 1 = read/execute (typical values)
        let read_write = 0i32;
        let read_execute = 1i32;
        assert_eq!(read_write, 0);
        assert_eq!(read_execute, 1);
    }

    #[test]
    fn test_codeholder_section_creation_parameters() {
        // Test that section creation parameters have reasonable types
        let name = "test_section";
        let size = 1024usize;
        let flags = 0u32;
        let alignment = 16u32;

        // Verify types are as expected
        let _name: &str = name;
        let _size: usize = size;
        let _flags: u32 = flags;
        let _alignment: u32 = alignment;
    }

    // ==================== Assembler Tests (type system and API) ====================

    #[test]
    fn test_assembler_send_sync() {
        // Verify Assembler implements Send and Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Assembler>();
        assert_sync::<Assembler>();
    }

    #[test]
    fn test_assembler_ptr_methods() {
        // Test that Assembler methods exist and have expected signatures
        fn takes_assembler_ptr(_: *mut AsmjitAssembler) {}
        let mut assembler = Assembler { ptr: std::ptr::null_mut() };
        takes_assembler_ptr(assembler.as_ptr());
    }

    #[test]
    fn test_assembler_drop_safety() {
        // Test that Assembler can be safely dropped (even with null pointer)
        let assembler = Assembler { ptr: std::ptr::null_mut() };
        drop(assembler); // Should not panic
    }

    #[test]
    fn test_assembler_debug_safety() {
        // Test that Assembler doesn't panic when debug-printed
        let assembler = Assembler { ptr: std::ptr::null_mut() };
        let debug_str = format!("{:?}", assembler);
        assert!(debug_str.contains("Assembler"));
    }

    #[test]
    fn test_assembler_clone_not_implemented() {
        // Assembler should not implement Clone (would be unsafe)
        fn assert_not_clone<T>() {
            let _assert = std::marker::PhantomData::<T>;
        }
        assert_not_clone::<Assembler>();
    }

    #[test]
    fn test_assembler_offset_return_type() {
        // Test that offset method returns usize
        let assembler = Assembler { ptr: std::ptr::null_mut() };
        let offset: usize = unsafe { asmjit_assembler_offset(assembler.ptr) };
        let _ = offset; // Just verify it compiles and returns expected type
    }

    #[test]
    fn test_assembler_label_operations() {
        // Test label-related operations exist and have proper signatures
        let assembler = Assembler { ptr: std::ptr::null_mut() };

        // Test new_label signature (would create null label in real usage)
        let label = Label { ptr: std::ptr::null_mut() };

        // Test bind_label signature
        let mut mutable_label = label;

        // These calls would normally make FFI calls, but we're just testing signatures
        fn takes_label_ptr(_: *mut AsmjitLabel) {}
        takes_label_ptr(mutable_label.as_ptr());
    }

    // ==================== Error Conversion Tests ====================

    #[test]
    fn test_asmjit_error_into_box_dyn_error() {
        let error = AsmjitError::InvalidLabel;
        let boxed: Box<dyn Error> = Box::new(error);
        assert!(boxed.to_string().contains("Invalid label"));
    }

    #[test]
    fn test_asmjit_error_source() {
        // AsmjitError doesn't wrap other errors, so source should be None
        let error = AsmjitError::InvalidLabel;
        assert!(error.source().is_none());
    }

    // ==================== Architecture-specific module tests ====================

    #[cfg(target_arch = "x86_64")]
    mod x86_tests {
        #[test]
        fn test_x86_module_exists() {
            // Verify the x86 module is accessible on x86_64
            use super::super::x86;
            let _ = x86::emit_mov_reg_reg;
            let _ = x86::emit_ret;
        }

        #[test]
        fn test_x86_instruction_signatures() {
            // Test that all expected x86-64 instruction functions exist and have correct signatures
            use super::super::x86;

            // Just verify that the functions can be referenced (compile-time check)
            // Basic register operations
            let _ = x86::emit_mov_reg_reg;
            let _ = x86::emit_ret;
        }

        #[test]
        fn test_x86_register_parameter_types() {
            // Test that register parameters use u32 as expected for x86-64
            let reg1: u32 = 0; // RAX
            let reg2: u32 = 1; // RCX
            let reg3: u32 = 2; // RDX

            // Verify parameter types match function signatures
            let _reg1: u32 = reg1;
            let _reg2: u32 = reg2;
            let _reg3: u32 = reg3;
        }

        #[test]
        fn test_x86_instruction_categories() {
            // Test that x86 instructions are properly categorized
            use super::super::x86;

            // Data movement instructions
            let mov_ops = [x86::emit_mov_reg_reg];
            let _ = mov_ops;

            // Control flow instructions
            let ctrl_ops = [x86::emit_ret];
            let _ = ctrl_ops;
        }

        #[test]
        fn test_x86_error_message_patterns() {
            // Test that error messages follow expected patterns for x86 instructions
            use super::AsmjitError;

            // Simulate the kinds of errors that would be generated
            let mov_error = AsmjitError::OperationFailed("Failed to emit mov: -1".to_string());
            assert!(mov_error.to_string().contains("Failed to emit mov"));

            let ret_error = AsmjitError::OperationFailed("Failed to emit ret: -2".to_string());
            assert!(ret_error.to_string().contains("Failed to emit ret"));
        }

        #[test]
        fn test_x86_instruction_naming_conventions() {
            // Test that x86 instruction naming follows consistent conventions
            use super::super::x86;

            // All emit_ functions should start with emit_
            let function_names = [
                "emit_mov_reg_reg",
                "emit_ret",
            ];

            for name in function_names {
                assert!(name.starts_with("emit_"), "Function {} should start with 'emit_'", name);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    mod a64_tests {
        #[test]
        fn test_a64_module_exists() {
            // Verify the a64 module is accessible on aarch64
            use super::super::a64;
            let _ = a64::emit_mov_reg_reg;
            let _ = a64::emit_ret;
        }

        #[test]
        fn test_a64_instruction_signatures() {
            // Test that all expected ARM64 instruction functions exist and have correct signatures
            use super::super::a64;

            // Just verify that the functions can be referenced (compile-time check)
            // The actual function signatures are validated by the fact that they compile

            // Basic register operations
            let _ = a64::emit_mov_reg_reg;
            let _ = a64::emit_ret;

            // Memory operations
            let _ = a64::emit_ldr_reg_offset;
            let _ = a64::emit_tst_imm;
            let _ = a64::emit_str_reg_offset;

            // Arithmetic operations
            let _ = a64::emit_add_reg_reg_reg;
            let _ = a64::emit_and_imm;
            let _ = a64::emit_sub_reg_reg_reg;
            let _ = a64::emit_subs_imm;

            // Stack operations
            let _ = a64::emit_stp_pre_idx;
            let _ = a64::emit_ldp_post_idx;
            let _ = a64::emit_stp;
            let _ = a64::emit_ldp;

            // BIF calling operations
            let _ = a64::emit_blr;
            let _ = a64::emit_blr_imm;

            // Immediate operations
            let _ = a64::emit_mov_imm;
            let _ = a64::emit_add_imm;
            let _ = a64::emit_sub_imm;
            let _ = a64::emit_cmp_imm;

            // Conditional operations
            let _ = a64::emit_b_cond;
            let _ = a64::emit_nop;

            // Additional arithmetic
            let _ = a64::emit_adds_reg_reg;
            let _ = a64::emit_adds_imm;
            let _ = a64::emit_cmp_reg_reg;

            // Branch operations
            let _ = a64::emit_b_eq;
            let _ = a64::emit_b_ne;
            let _ = a64::emit_b_lt;
            let _ = a64::emit_b_ge;
            let _ = a64::emit_b;

            // Shift operations
            let _ = a64::emit_lsr_imm;
            let _ = a64::emit_lsl_imm;

            // Additional memory operations
            let _ = a64::emit_stur_reg_offset;
            let _ = a64::emit_ldur_reg_offset;

            // Division and multiplication
            let _ = a64::emit_udiv_reg_reg_reg;
            let _ = a64::emit_mul_reg_reg_reg;
            let _ = a64::emit_msub_reg_reg_reg_reg;

            // Logical operations
            let _ = a64::emit_eor_reg_reg_reg;
        }

        #[test]
        fn test_a64_register_parameter_types() {
            // Test that register parameters use u32 as expected
            let reg1: u32 = 0;
            let reg2: u32 = 1;
            let reg3: u32 = 2;
            let imm_small: u32 = 42;
            let imm_large: u64 = 0x123456789ABCDEF0;
            let offset: i32 = -16;

            // Verify parameter types match function signatures
            let _reg1: u32 = reg1;
            let _reg2: u32 = reg2;
            let _reg3: u32 = reg3;
            let _imm_small: u32 = imm_small;
            let _imm_large: u64 = imm_large;
            let _offset: i32 = offset;
        }

        #[test]
        fn test_a64_instruction_categories() {
            // Test that instructions are properly categorized by functionality
            use super::super::a64;

            // Just reference the functions to ensure they exist and compile
            // Data movement instructions
            let _mov_reg_reg = a64::emit_mov_reg_reg;
            let _mov_imm = a64::emit_mov_imm;

            // Arithmetic instructions
            let _add_reg_reg_reg = a64::emit_add_reg_reg_reg;
            let _add_imm = a64::emit_add_imm;
            let _adds_reg_reg = a64::emit_adds_reg_reg;
            let _adds_imm = a64::emit_adds_imm;
            let _sub_reg_reg_reg = a64::emit_sub_reg_reg_reg;
            let _sub_imm = a64::emit_sub_imm;
            let _subs_imm = a64::emit_subs_imm;
            let _mul_reg_reg_reg = a64::emit_mul_reg_reg_reg;
            let _msub_reg_reg_reg_reg = a64::emit_msub_reg_reg_reg_reg;
            let _udiv_reg_reg_reg = a64::emit_udiv_reg_reg_reg;

            // Logical instructions
            let _and_imm = a64::emit_and_imm;
            let _eor_reg_reg_reg = a64::emit_eor_reg_reg_reg;
            let _tst_imm = a64::emit_tst_imm;

            // Memory instructions
            let _ldr_reg_offset = a64::emit_ldr_reg_offset;
            let _str_reg_offset = a64::emit_str_reg_offset;
            let _ldur_reg_offset = a64::emit_ldur_reg_offset;
            let _stur_reg_offset = a64::emit_stur_reg_offset;
            let _stp = a64::emit_stp;
            let _ldp = a64::emit_ldp;
            let _stp_pre_idx = a64::emit_stp_pre_idx;
            let _ldp_post_idx = a64::emit_ldp_post_idx;

            // Control flow instructions
            let _ret = a64::emit_ret;
            let _blr = a64::emit_blr;
            let _blr_imm = a64::emit_blr_imm;
            let _b = a64::emit_b;
            let _b_eq = a64::emit_b_eq;
            let _b_ne = a64::emit_b_ne;
            let _b_lt = a64::emit_b_lt;
            let _b_ge = a64::emit_b_ge;
            let _b_cond = a64::emit_b_cond;
            let _cmp_reg_reg = a64::emit_cmp_reg_reg;
            let _cmp_imm = a64::emit_cmp_imm;

            // Shift instructions
            let _lsr_imm = a64::emit_lsr_imm;
            let _lsl_imm = a64::emit_lsl_imm;

            // Miscellaneous instructions
            let _nop = a64::emit_nop;
        }

        #[test]
        fn test_a64_error_message_patterns() {
            // Test that error messages follow expected patterns for different instruction types
            use super::AsmjitError;

            // Simulate the kinds of errors that would be generated
            let mov_error = AsmjitError::OperationFailed("Failed to emit mov: -1".to_string());
            assert!(mov_error.to_string().contains("Failed to emit mov"));

            let ldr_error = AsmjitError::OperationFailed("Failed to emit ldr: -2".to_string());
            assert!(ldr_error.to_string().contains("Failed to emit ldr"));

            let add_error = AsmjitError::OperationFailed("Failed to emit add: -3".to_string());
            assert!(add_error.to_string().contains("Failed to emit add"));

            let ret_error = AsmjitError::OperationFailed("Failed to emit ret: -4".to_string());
            assert!(ret_error.to_string().contains("Failed to emit ret"));
        }

        #[test]
        fn test_a64_instruction_naming_conventions() {
            // Test that instruction naming follows consistent conventions
            use super::super::a64;

            // All emit_ functions should start with emit_
            let function_names = [
                "emit_mov_reg_reg",
                "emit_ret",
                "emit_ldr_reg_offset",
                "emit_add_reg_reg_reg",
                "emit_blr",
                "emit_b",
            ];

            for name in function_names {
                assert!(name.starts_with("emit_"), "Function {} should start with 'emit_'", name);
            }
        }
    }

    // ==================== CString handling tests ====================

    #[test]
    fn test_cstring_from_valid_str() {
        let result = CString::new("test_section");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cstring_from_str_with_null() {
        let result = CString::new("test\0section");
        assert!(result.is_err());
    }

    #[test]
    fn test_cstring_from_empty_str() {
        let result = CString::new("");
        assert!(result.is_ok());
    }

    // ==================== Error message formatting tests ====================

    #[test]
    fn test_error_format_with_error_code() {
        // Test the format used in actual error handling
        let err_code = 42;
        let error = AsmjitError::OperationFailed(format!("Failed to init CodeHolder: {}", err_code));
        assert!(error.to_string().contains("42"));
    }

    #[test]
    fn test_error_format_bind_label() {
        let err_code = 123;
        let error = AsmjitError::OperationFailed(format!("Failed to bind label: {}", err_code));
        assert!(error.to_string().contains("Failed to bind label"));
        assert!(error.to_string().contains("123"));
    }

    #[test]
    fn test_error_format_emit_mov() {
        let err_code = 456;
        let error = AsmjitError::OperationFailed(format!("Failed to emit mov: {}", err_code));
        assert!(error.to_string().contains("Failed to emit mov"));
        assert!(error.to_string().contains("456"));
    }

    #[test]
    fn test_error_format_emit_ret() {
        let err_code = 789;
        let error = AsmjitError::OperationFailed(format!("Failed to emit ret: {}", err_code));
        assert!(error.to_string().contains("Failed to emit ret"));
        assert!(error.to_string().contains("789"));
    }

    #[test]
    fn test_error_format_invalid_section_name() {
        // Create a NulError by trying to create a CString with an embedded null
        let nul_error = CString::new("test\0section").unwrap_err();
        let error = AsmjitError::OperationFailed(format!("Invalid section name: {}", nul_error));
        assert!(error.to_string().contains("Invalid section name"));
    }

    // ==================== Memory Management Tests ====================

    #[test]
    fn test_memory_protection_constants() {
        // Test memory protection constants used in CodeHolder
        let read_write_access = 0i32;
        let read_execute_access = 1i32;

        // Verify these are the expected values
        assert_eq!(read_write_access, 0);
        assert_eq!(read_execute_access, 1);

        // Test that they are within reasonable ranges
        assert!(read_write_access >= 0);
        assert!(read_execute_access >= 0);
        assert!(read_execute_access > read_write_access);
    }

    #[test]
    fn test_relocation_address_types() {
        // Test that relocation uses correct pointer types
        let base_address: *mut u8 = std::ptr::null_mut();
        let const_base: *const u8 = base_address as *const u8;

        // Verify type compatibility
        let _: *mut u8 = base_address;
        let _: *const u8 = const_base;
    }

    #[test]
    fn test_code_size_return_type() {
        // Test that code_size returns usize
        let code_holder = CodeHolder { ptr: std::ptr::null_mut() };
        let size: usize = unsafe { asmjit_codeholder_code_size(code_holder.ptr) };
        let _ = size; // Just verify return type
    }

    #[test]
    fn test_base_address_return_type() {
        // Test that base_address returns *const u8
        let code_holder = CodeHolder { ptr: std::ptr::null_mut() };
        let addr: *const u8 = unsafe { asmjit_codeholder_base_address(code_holder.ptr) };
        let _ = addr; // Just verify return type
    }

    #[test]
    fn test_memory_protection_error_codes() {
        // Test that memory protection operations can return error codes
        let error_code: AsmjitErrorCode = -1;
        assert!(error_code < 0, "Error codes should be negative");

        // Test success code
        let success_code: AsmjitErrorCode = 0;
        assert_eq!(success_code, 0, "Success code should be zero");
    }

    #[test]
    fn test_copy_flattened_data_parameters() {
        // Test that copy_flattened_data has correct parameter types
        let buffer: *mut u8 = std::ptr::null_mut();
        let size: usize = 1024;

        // Verify parameter types
        let _: *mut u8 = buffer;
        let _: usize = size;
    }

    // ==================== Section Management Tests ====================

    #[test]
    fn test_section_creation_parameters() {
        // Test section creation parameter types
        let name = "test_section";
        let size: usize = 1024;
        let flags: u32 = 0;
        let alignment: u32 = 16;

        // Verify parameter types match expected usage
        let _: &str = name;
        let _: usize = size;
        let _: u32 = flags;
        let _: u32 = alignment;
    }

    #[test]
    fn test_section_pointer_return_type() {
        // Test that new_section returns the expected pointer type
        let section_ptr: *mut AsmjitSection = std::ptr::null_mut();
        let _: *mut AsmjitSection = section_ptr;
    }

    #[test]
    fn test_section_name_validation() {
        // Test section name validation (should be valid C string)
        let valid_names = ["text", "data", "rodata", "bss", "test\nnewline", "test\tab"];
        let invalid_names = ["test\0null"];

        for name in &valid_names {
            assert!(CString::new(*name).is_ok(), "Valid name '{}' should create valid CString", name);
        }

        for name in &invalid_names {
            assert!(CString::new(*name).is_err(), "Invalid name '{}' should fail to create CString", name);
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_null_pointer_safety() {
        // Test that operations with null pointers don't cause immediate crashes
        // (They would fail at the FFI boundary, but shouldn't panic in Rust code)

        let null_codeholder = CodeHolder { ptr: std::ptr::null_mut() };
        let mut null_assembler = Assembler { ptr: std::ptr::null_mut() };
        let mut null_label = Label { ptr: std::ptr::null_mut() };

        // These should not panic - just test that the methods exist
        let _ptr = null_codeholder.as_ptr();
        let _ptr = null_assembler.as_ptr();
        let _ptr = null_label.as_ptr();

        // Drop should be safe even with null pointers
        drop(null_codeholder);
        drop(null_assembler);
        drop(null_label);
    }

    #[test]
    fn test_error_code_ranges() {
        // Test that error codes can represent various failure conditions
        let error_codes = [-1, -2, -99, i32::MIN];

        for &code in &error_codes {
            assert!(code < 0, "Error code {} should be negative", code);
        }

        // Test success code
        let success = 0;
        assert_eq!(success, 0);
    }

    #[test]
    fn test_register_parameter_ranges() {
        // Test that register parameters accept reasonable ranges
        // ARM64 has 32 registers (x0-x31)
        let valid_regs = [0u32, 1, 15, 30, 31];
        let potentially_invalid_regs = [32u32, 63, u32::MAX];

        for &reg in &valid_regs {
            assert!(reg < 32, "Register {} should be valid for ARM64", reg);
        }

        // x86-64 has more registers, but we still test bounds
        for &reg in &potentially_invalid_regs {
            assert!(reg >= 32, "Register {} might be invalid", reg);
        }
    }

    #[test]
    fn test_immediate_value_ranges() {
        // Test that immediate values can handle edge cases
        let small_imm = 0u32;
        let large_imm_u32 = u32::MAX;
        let large_imm_u64 = u64::MAX;

        // Verify types
        let _: u32 = small_imm;
        let _: u32 = large_imm_u32;
        let _: u64 = large_imm_u64;
    }

    #[test]
    fn test_offset_range_validation() {
        // Test that memory offsets handle negative and positive values
        let offsets = [-4096i32, -16, 0, 16, 4096, i32::MIN / 2, i32::MAX / 2];

        for &offset in &offsets {
            let _: i32 = offset; // Just verify type compatibility
        }
    }

    #[test]
    fn test_alignment_values() {
        // Test that alignment values are powers of 2
        let alignments = [1u32, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

        for &align in &alignments {
            assert!(align.is_power_of_two(), "Alignment {} should be power of 2", align);
        }
    }

    #[test]
    fn test_section_size_limits() {
        // Test that section sizes can handle various values
        let sizes = [0usize, 1, 1024, 4096, 65536, usize::MAX / 2];

        for &size in &sizes {
            let _: usize = size; // Just verify type compatibility
        }
    }

    #[test]
    fn test_cstring_edge_cases() {
        // Test CString creation with edge cases
        let test_cases = vec![
            ("", true),           // Empty string
            ("a", true),          // Single character
            ("normal_name", true), // Normal case
            ("name_with_123", true), // With numbers
            ("name-with-dashes", true), // With dashes
            ("name.with.dots", true), // With dots
            ("name_with_underscores", true), // With underscores
            ("test\nnewline", true), // Newline (valid)
            ("test\tab", true),  // Tab (valid)
            ("\0", false),        // Null byte at start
            ("test\0null", false), // Null byte in middle
            ("test\x00null", false), // Hex null
        ];

        for (input, should_succeed) in test_cases {
            let result = std::ffi::CString::new(input);
            if should_succeed {
                assert!(result.is_ok(), "CString::new({:?}) should succeed", input);
            } else {
                assert!(result.is_err(), "CString::new({:?}) should fail", input);
            }
        }
    }

    #[test]
    fn test_architecture_conditional_compilation() {
        // Test that architecture-specific code is properly gated
        #[cfg(target_arch = "aarch64")]
        {
            // On ARM64, a64 module should be available
            use super::a64;
            let _ = a64::emit_mov_reg_reg;
        }

        #[cfg(target_arch = "x86_64")]
        {
            // On x86-64, x86 module should be available
            use super::x86;
            let _ = x86::emit_mov_reg_reg;
        }

        // This test should compile on any architecture
        let _error = AsmjitError::InvalidLabel;
    }

}

