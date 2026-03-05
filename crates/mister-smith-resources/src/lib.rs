//! Generic connection pooling, pool sizing, health checks, and resource lifecycle management.
//!
//! This crate provides:
//!
//! - **[`pool`]** — [`ConnectionPool<R>`](pool::ConnectionPool), a generic async-aware
//!   connection pool with RAII checkout via [`PooledResource`].
//! - **[`sizing`]** — Pool sizing algorithms using Little's Law and environment templates.
//! - **[`health`]** — [`PoolHealthReport`] for pool observability.
//! - **[`manager`]** — [`ResourceManager`] for heterogeneous pool
//!   registration and lookup.

pub mod health;
pub mod manager;
pub mod pool;
pub mod sizing;

// Re-export the core Resource trait for convenience.
pub use mister_smith_core::Resource;

// Re-export primary types at crate root.
pub use health::PoolHealthReport;
pub use manager::ResourceManager;
pub use pool::{ConnectionPool, PoolConfig, PoolError, PooledResource};
pub use sizing::{
    get_environment_template, ConnectionPoolSizer, Environment, PoolSizeRecommendation,
    PoolSizeTemplate,
};
