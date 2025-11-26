//! Adapters Layer: Time Management
//!
//! Provides time management functionality:
//! - Timers
//! - Time slice management
//!
//! Based on timer_drv.c and consume_timeslice_drv.c
//! Depends on Entities layer.

pub mod timer;
pub mod timeslice;

pub use timer::Timer;
pub use timeslice::TimeSlice;

