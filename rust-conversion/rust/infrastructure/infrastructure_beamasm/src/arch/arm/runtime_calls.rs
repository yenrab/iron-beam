//! Runtime Function Calls
//!
//! Provides type-safe calling of runtime C functions from JIT-generated code.
//! This handles argument passing, return values, and ARM64 ABI compliance.
//!
//! Based on `erts/emulator/beam/jit/x86/beam_asm.hpp` runtime_call/fragment_call

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, CodeHolder, a64};

/// Function pointer type for runtime functions
pub type FunctionPtr = *const std::ffi::c_void;

/// Argument types for runtime function calls
#[derive(Debug, Clone)]
pub enum RuntimeArg {
    /// Register argument (X register index)
    Register(u32),
    /// Immediate value
    Immediate(u64),
    /// Stack offset (for complex arguments)
    StackOffset(i32),
}

/// Runtime function call manager for ARM64 JIT
///
/// Handles calling Erlang runtime C functions from JIT-generated code.
/// Ensures proper argument passing, stack alignment, and return value handling.
pub struct RuntimeCallManager;

impl RuntimeCallManager {
    /// Call a runtime function with type-safe argument passing
    ///
    /// This generates code to call a C runtime function with proper ARM64 ABI compliance.
    /// Follows the Erlang JIT pattern: enter_runtime -> setup args -> call -> leave_runtime
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `func` - Function pointer to call
    /// * `args` - Arguments to pass to the function
    /// * `spec` - Runtime spec flags for enter/leave runtime calls
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn runtime_call(
        assembler: &mut Assembler,
        func: FunctionPtr,
        args: &[RuntimeArg],
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Runtime Call: Calling function {:p} with {} args, spec=0x{:x}",
                 func, args.len(), spec);

        // Validate argument count (ARM64 ABI supports up to 8 register args)
        if args.len() > 8 {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Too many arguments for runtime call: {} (max 8)", args.len())
            ));
        }

        // Enter runtime context (save process state)
        crate::RuntimeContextManager::emit_enter_runtime(assembler, spec)?;

        // Prepare arguments in ARM64 registers (X0-X7)
        Self::prepare_arguments(assembler, args)?;

        // Perform the function call
        Self::emit_dynamic_call(assembler, func)?;

        // Leave runtime context (restore process state)
        crate::RuntimeContextManager::emit_leave_runtime(assembler, spec)?;

        Ok(())
    }

    /// Call a runtime fragment function (simplified version)
    ///
    /// Similar to runtime_call but for simpler fragment functions that don't
    /// require complex argument marshalling. Uses basic runtime context.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `func` - Function pointer to call
    /// * `spec` - Runtime spec flags (typically just reductions)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn fragment_call(
        assembler: &mut Assembler,
        func: FunctionPtr,
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Runtime Call: Fragment call to {:p}, spec=0x{:x}", func, spec);

        // Enter runtime context for fragment call
        crate::RuntimeContextManager::emit_enter_runtime(assembler, spec)?;

        // Call the fragment function directly
        Self::emit_direct_call(assembler, func)?;

        // Leave runtime context
        crate::RuntimeContextManager::emit_leave_runtime(assembler, spec)?;

        Ok(())
    }

    /// Prepare arguments for runtime function call
    ///
    /// Moves arguments into the appropriate ARM64 registers according to ABI.
    /// X0-X7 are used for the first 8 arguments.
    fn prepare_arguments(
        assembler: &mut Assembler,
        args: &[RuntimeArg],
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        for (i, arg) in args.iter().enumerate() {
            if i >= 8 {
                return Err(BeamAssemblerError::CodeGenerationFailed(
                    "Too many arguments for register passing".to_string()
                ));
            }

            let target_reg = i as u32; // X0, X1, X2, ..., X7

            match arg {
                RuntimeArg::Register(src_reg) => {
                    // Move register value to argument register
                    if *src_reg != target_reg {
                        a64::emit_mov_reg_reg(assembler, target_reg, *src_reg)?;
                    }
                }
                RuntimeArg::Immediate(value) => {
                    // Load immediate value into argument register
                    a64::emit_mov_imm(assembler, target_reg, *value)?;
                }
                RuntimeArg::StackOffset(offset) => {
                    // Load from stack into argument register
                    a64::emit_ldr_reg_offset(assembler, target_reg, 31, *offset)?;
                }
            }
        }

        Ok(())
    }

    /// Emit a dynamic runtime call (function pointer in register)
    ///
    /// Calls a runtime function where the address is stored in a register.
    /// This is the primary method used for runtime calls in Erlang JIT.
    fn emit_dynamic_call(
        assembler: &mut Assembler,
        func: FunctionPtr,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Runtime Call: Emitting dynamic call to {:p}", func);

        // Load function pointer into a register and call it
        // In C++: a.mov(TMP1, Func); a.blr(TMP1);

        // For ARM64, we use TMP1 (x9) as the temporary register
        // Load function address into TMP1
        a64::emit_mov_imm(assembler, 9, func as u64)?;

        // Branch with link and return to the function
        a64::emit_blr(assembler, 9)?;

        Ok(())
    }

    /// Emit a direct call to a known function address
    ///
    /// Calls a function with a known address at compile time.
    fn emit_direct_call(
        assembler: &mut Assembler,
        func: FunctionPtr,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Runtime Call: Emitting direct call to {:p}", func);

        // For direct calls to known addresses, we can use BL with immediate
        // In C++: a.bl(func_address)

        a64::emit_blr_imm(assembler, func as u64)?;

        Ok(())
    }

    /// Assert stack consistency before runtime calls
    ///
    /// In debug builds, verifies that the stack is in a consistent state.
    fn emit_assert_stack_consistency(_assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        // In debug builds, we would add stack consistency checks here
        // For now, this is a no-op
        Ok(())
    }
}

/// Type-safe runtime function call builder
///
/// Provides a fluent API for building runtime function calls with proper typing.
pub struct RuntimeCallBuilder<'a> {
    assembler: &'a mut Assembler,
    func: Option<FunctionPtr>,
    args: Vec<RuntimeArg>,
    spec: u32,
}

impl<'a> RuntimeCallBuilder<'a> {
    /// Create a new runtime call builder
    pub fn new(assembler: &'a mut Assembler) -> Self {
        Self {
            assembler,
            func: None,
            args: Vec::new(),
            spec: 0,
        }
    }

    /// Set the function to call
    pub fn function(mut self, func: FunctionPtr) -> Self {
        self.func = Some(func);
        self
    }

    /// Set the runtime spec flags
    pub fn spec(mut self, spec: u32) -> Self {
        self.spec = spec;
        self
    }

    /// Add a register argument
    pub fn arg_register(mut self, reg: u32) -> Self {
        self.args.push(RuntimeArg::Register(reg));
        self
    }

    /// Add an immediate argument
    pub fn arg_immediate(mut self, value: u64) -> Self {
        self.args.push(RuntimeArg::Immediate(value));
        self
    }

    /// Add a stack offset argument
    pub fn arg_stack_offset(mut self, offset: i32) -> Self {
        self.args.push(RuntimeArg::StackOffset(offset));
        self
    }

    /// Execute the runtime call
    pub fn call(mut self) -> Result<(), BeamAssemblerError> {
        let func = self.func.ok_or_else(|| {
            BeamAssemblerError::CodeGenerationFailed("No function specified for runtime call".to_string())
        })?;

        RuntimeCallManager::runtime_call(self.assembler, func, &self.args, self.spec)
    }
}

/// Convenience functions for common runtime calls
impl RuntimeCallManager {
    /// Call a BIF (Built-In Function) dispatcher
    ///
    /// This is a specialized call for BIF execution that handles the common
    /// pattern of calling Erlang's BIF dispatcher with bif_num and arguments.
    pub fn call_bif_dispatcher(
        assembler: &mut Assembler,
        bif_func: FunctionPtr,
        arg1: u32,
        arg2: u32,
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        // Prepare arguments for BIF call
        let args = vec![
            RuntimeArg::Register(arg1),
            RuntimeArg::Register(arg2),
        ];

        // Call the BIF dispatcher
        Self::runtime_call(assembler, bif_func, &args, spec)
    }

    /// Call a garbage collection function
    pub fn call_gc_function(
        assembler: &mut Assembler,
        func: FunctionPtr,
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Runtime Call: Calling GC function {:p}", func);

        // GC functions may need special stack handling
        Self::fragment_call(assembler, func, spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_arg_creation() {
        let reg_arg = RuntimeArg::Register(5);
        let imm_arg = RuntimeArg::Immediate(42);
        let stack_arg = RuntimeArg::StackOffset(16);

        match reg_arg {
            RuntimeArg::Register(reg) => assert_eq!(reg, 5),
            _ => panic!("Wrong arg type"),
        }

        match imm_arg {
            RuntimeArg::Immediate(val) => assert_eq!(val, 42),
            _ => panic!("Wrong arg type"),
        }

        match stack_arg {
            RuntimeArg::StackOffset(offset) => assert_eq!(offset, 16),
            _ => panic!("Wrong arg type"),
        }
    }

    #[test]
    fn test_runtime_call_builder_creation() {
        // We can't test actual assembler calls without a real assembler,
        // but we can test the builder creation
        let code_holder = crate::asmjit_wrapper::CodeHolder::new().unwrap();
        let mut assembler = crate::asmjit_wrapper::Assembler::new(&code_holder).unwrap();
        let builder = RuntimeCallBuilder::new(&mut assembler);

        assert!(builder.func.is_none());
        assert!(builder.args.is_empty());
        assert_eq!(builder.spec, 0);
    }

    #[test]
    fn test_runtime_call_manager_creation() {
        // RuntimeCallManager has no state, just test creation
        let _manager = RuntimeCallManager;
    }
}
