//! Use Cases Layer: Memory Management
//!
//! Provides memory allocation strategies:
//! - Good-fit allocator
//! - Best-fit allocator
//! - A-fit allocator
//! - First-fit allocator
//!
//! Based on erl_goodfit_alloc.c and related allocation files.
//! Depends on Entities layer.

pub mod allocator;
pub mod goodfit;
pub mod bestfit;
pub mod afit;
pub mod firstfit;

pub use allocator::{Allocator, AllocatorType, AllocationError};

