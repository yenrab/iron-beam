//! Use Cases Layer: Process Scheduling
//!
//! Provides process scheduling functionality for the Erlang/OTP runtime system. This
//! crate implements business logic for process scheduling, run queue management,
//! and scheduler coordination.
//!
//! ## Overview
//!
//! The `usecases_scheduling` crate is part of the use cases layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides business logic for
//! scheduling Erlang processes, managing run queues, and coordinating schedulers.
//!
//! ## Modules
//!
//! - **[`run_queue`](run_queue/index.html)**: Run queue management with priority queues
//!   for scheduling processes at different priority levels
//!
//! - **[`scheduler`](scheduler/index.html)**: Scheduler functions including the main
//!   scheduler loop, scheduler wake/sleep, and scheduler state management
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_process.c`. It depends on:
//! - `entities_process` for Process structures
//! - `infrastructure_utilities` for process table access
//! - `usecases_process_management` for process state management
//!
//! ## Safety
//!
//! All code uses safe Rust patterns. Run queues use `Arc<Process>` for shared ownership
//! and `Mutex` for thread-safe access. No unsafe blocks are used.
//!
//! ## See Also
//!
//! - [`entities_process`](../../entities/entities_process/index.html): Process entity layer
//! - [`usecases_process_management`](../usecases_process_management/index.html): Process management use cases
//! - [`infrastructure_utilities`](../../infrastructure/infrastructure_utilities/index.html): Infrastructure utilities

pub mod run_queue;
pub mod scheduler;
pub mod initialization;
pub mod threads;

pub use run_queue::{RunQueue, RunPrioQueue, RunQueueInfo, Priority, dequeue_process, enqueue_process, check_requeue_process};
pub use scheduler::{Scheduler, schedule_process, erts_schedule, wake_scheduler, init_scheduler_suspend, ScheduleError};
pub use initialization::{erts_init_scheduling, get_global_schedulers};
pub use threads::{erts_start_schedulers, erts_stop_schedulers};

