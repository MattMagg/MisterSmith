//! Error re-exports from `mister-smith-core`.
//!
//! All actor-related error types are defined canonically in the core crate.
//! This module re-exports them for convenience.

pub use mister_smith_core::{ActorError, SupervisionError, SystemError};
