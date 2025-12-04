//! Use Cases Layer: Memory Management
//!
//! Provides memory allocation strategies for the Erlang/OTP runtime system. This crate
//! implements various memory allocation algorithms that are used for managing heap memory
//! in the runtime.
//!
//! ## Overview
//!
//! The `usecases_memory_management` crate is part of the use cases layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides business logic for memory
//! allocation, implementing multiple allocation strategies optimized for different use cases.
//!
//! ## Allocation Strategies
//!
//! - **[`goodfit`](goodfit/index.html)**: Good-fit allocator - balances allocation speed
//!   and memory efficiency by finding blocks that are close to the requested size
//!
//! - **[`bestfit`](bestfit/index.html)**: Best-fit allocator - finds the smallest block
//!   that can satisfy the request, minimizing wasted memory
//!
//! - **[`afit`](afit/index.html)**: A-fit allocator - adaptive allocation strategy that
//!   adjusts based on allocation patterns
//!
//! - **[`firstfit`](firstfit/index.html)**: First-fit allocator - uses the first block
//!   that can satisfy the request, prioritizing allocation speed
//!
//! - **[`allocator`](allocator/index.html)**: Common allocator interface and types
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_goodfit_alloc.c` and related
//! allocation files. It depends on the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Core data handling
//! - [`entities_process`](../../entities/entities_process/index.html): Process heap management

pub mod allocator;
pub mod goodfit;
pub mod bestfit;
pub mod afit;
pub mod firstfit;

pub use allocator::{Allocator, AllocatorType, AllocationError};

