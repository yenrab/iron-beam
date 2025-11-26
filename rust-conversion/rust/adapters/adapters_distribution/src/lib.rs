//! Adapters Layer: Distribution
//!
//! Provides distribution functionality:
//! - External term format
//! - UDS (Unix Domain Socket) distribution
//!
//! Based on external.c and uds_drv.c
//! Depends on Entities layer.

pub mod external;
pub mod uds;

pub use external::ExternalTerm;
pub use uds::UdsDistribution;

