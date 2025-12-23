//! asmjit wrapper
//!
//! Provides Rust bindings to the asmjit C++ library.
//! This module wraps asmjit calls in safe Rust interfaces.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use thiserror::Error;

/// Errors from asmjit operations
#[derive(Debug, Error)]
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
    fn asmjit_a64_assembler_emit_sub_reg_reg_reg(
        assembler: *mut AsmjitAssembler,
        dst: u32,
        src1: u32,
        src2: u32,
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

    // BIF calling operations
    #[cfg(target_arch = "aarch64")]
    fn asmjit_a64_assembler_emit_blr(
        assembler: *mut AsmjitAssembler,
        reg: u32,
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
}

/// Wrapper for asmjit CodeHolder
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
            let err = asmjit_virtmem_protect_jit_memory(access);
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
}

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

    // ==================== CodeHolder Tests (type system) ====================

    #[test]
    fn test_codeholder_send_sync() {
        // Verify CodeHolder implements Send and Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CodeHolder>();
        assert_sync::<CodeHolder>();
    }

    // ==================== Assembler Tests (type system) ====================

    #[test]
    fn test_assembler_send_sync() {
        // Verify Assembler implements Send and Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Assembler>();
        assert_sync::<Assembler>();
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
}

