//! Infrastructure Layer: Emulator Loop
//!
//! Provides the main emulator execution loop for BEAM instruction execution.
//! This crate implements the core process_main() function that executes BEAM
//! instructions for Erlang processes.
//!
//! ## Overview
//!
//! The `infrastructure_emulator_loop` crate is part of the infrastructure layer
//! in the CLEAN architecture implementation of Erlang/OTP. It provides the main
//! execution loop that:
//! - Executes BEAM instructions for processes
//! - Manages process registers (X registers, heap, stack)
//! - Handles reduction counting
//! - Coordinates with the scheduler
//!
//! ## Modules
//!
//! - **[`emulator_loop`](emulator_loop/index.html)**: Main emulator loop
//!   (process_main, init_emulator)
//!
//! - **[`registers`](registers/index.html)**: Register management functions
//!   (copy_in_registers, copy_out_registers)
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `beam_emu.c`. It depends on:
//! - `infrastructure_bif_dispatcher` for BIF call dispatching
//! - `usecases_scheduling` for process scheduling
//! - `entities_process` for Process structures
//!
//! The emulator loop is the heart of the BEAM virtual machine, executing
//! instructions and coordinating with other runtime components.
//!
//! ## See Also
//!
//! - [`infrastructure_bif_dispatcher`](../infrastructure_bif_dispatcher/index.html): BIF dispatcher
//! - [`usecases_scheduling`](../../usecases/usecases_scheduling/index.html): Process scheduling
//! - [`entities_process`](../../entities/entities_process/index.html): Process entities

pub mod emulator_loop;
pub mod registers;
pub mod instruction_execution;
pub mod instruction_decoder;
pub mod process_executor_impl;

#[cfg(test)]
mod test_code;

pub use emulator_loop::{process_main, init_emulator, EmulatorLoop, EmulatorLoopError};
pub use registers::{copy_in_registers, copy_out_registers, RegisterManager};
pub use instruction_execution::{InstructionResult, InstructionExecutor, DefaultInstructionExecutor, is_valid_instruction, next_instruction};
pub use instruction_decoder::{decode_instruction, get_instruction_size, opcodes};
pub use process_executor_impl::EmulatorLoopExecutor;


