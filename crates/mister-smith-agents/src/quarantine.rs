//! Agent-facing quarantine re-exports.
//!
//! The concrete quarantine actor lives in `mister-smith-security` so the same
//! implementation can protect both agent messaging boundaries and persistence
//! shared-state boundaries without a crate cycle.

pub use mister_smith_security::{QuarantineActor, QuarantineTransfer, SharedStateAccess};
