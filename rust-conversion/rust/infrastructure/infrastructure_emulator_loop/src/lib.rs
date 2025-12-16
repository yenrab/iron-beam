//! Infrastructure Layer: Emulator Loop
//!
//! Provides the main emulator execution loop for BEAM instruction execution.
//! This crate implements the core process_main() function that executes BEAM
//! instructions for Erlang processes using JIT-compiled code.
//!
//! ## Overview
//!
//! The `infrastructure_emulator_loop` crate is part of the infrastructure layer
//! in the CLEAN architecture implementation of Erlang/OTP. It provides the main
//! execution loop that:
//! - Executes BEAM instructions for processes using JIT-compiled native code
//! - Manages process registers (X registers, heap, stack)
//! - Handles reduction counting
//! - Coordinates with the scheduler
//!
//! ## JIT Execution
//!
//! Unlike the C implementation which has a build switch between interpreter
//! (`beam_emu.c`) and JIT (`beamasm`), the Rust implementation uses **only**
//! BeamAsm JIT execution. All BEAM instructions are compiled to native code
//! at load time by `infrastructure_beamasm`, and this crate calls the JIT-compiled
//! code directly.
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
//! This crate is based on the C implementation in `beam_emu.c`, but uses JIT
//! execution exclusively. It depends on:
//! - `infrastructure_beamasm` for JIT code generation and execution
//! - `infrastructure_bif_dispatcher` for BIF call dispatching
//! - `usecases_scheduling` for process scheduling
//! - `entities_process` for Process structures
//!
//! The emulator loop coordinates with the BeamAsm JIT system to execute
//! native code and coordinate with other runtime components.
//!
//! ## REPL Integration
//!
//! The REPL (Read-Eval-Print Loop) uses this crate for execution in the same way as
//! the C version, but reengineered for Rust:
//!
//! 1. REPL expressions are scanned and parsed (`infrastructure_utilities::erl_scan`, `erl_parse`)
//! 2. Parsed expressions are compiled to BEAM bytecode
//! 3. BEAM bytecode is JIT-compiled via `infrastructure_beamasm::BeamAsmLoader`
//! 4. JIT-compiled code is executed via `process_main()` in this crate
//!
//! See `REPL_INTEGRATION.md` for detailed documentation on REPL integration.
//!
//! ## See Also
//!
//! - [`infrastructure_beamasm`](../infrastructure_beamasm/index.html): BeamAsm JIT compiler
//! - [`infrastructure_bif_dispatcher`](../infrastructure_bif_dispatcher/index.html): BIF dispatcher
//! - [`usecases_scheduling`](../../usecases/usecases_scheduling/index.html): Process scheduling
//! - [`entities_process`](../../entities/entities_process/index.html): Process entities
//! - [`REPL_INTEGRATION.md`](REPL_INTEGRATION.md): REPL integration documentation

pub mod emulator_loop;
pub mod registers;
pub mod process_executor_impl;

pub use emulator_loop::{process_main, init_emulator, EmulatorLoop, EmulatorLoopError};
pub use registers::{copy_in_registers, copy_out_registers, RegisterManager};
pub use process_executor_impl::EmulatorLoopExecutor;


