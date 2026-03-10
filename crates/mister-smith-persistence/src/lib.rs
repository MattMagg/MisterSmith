//! Persistence and state management for the Mister Smith framework.
//!
//! Implements a dual-store architecture:
//! - **PostgreSQL** (via sqlx) for authoritative relational persistence
//! - **JetStream KV** (via async-nats) for fast distributed state
//!
//! A [`HybridStateManager`] routes reads/writes
//! to the appropriate backend, with dirty-key tracking, configurable flush thresholds,
//! state hydration on startup, and graceful degradation when a backend is unavailable.

pub mod config;
pub mod error;
pub mod health;
pub mod hybrid;
pub mod kv;
pub mod memory;
#[cfg(feature = "sqlx")]
pub mod postgres;
pub mod repository;

#[cfg(all(feature = "security", feature = "sqlx"))]
pub mod audit_persister;

// Re-exports for convenience
pub use config::{CheckpointConfig, FlushConfig, KvConfig, PersistenceConfig, PostgresConfig};
pub use error::from_kv_error;
#[cfg(feature = "sqlx")]
pub use error::from_sqlx_error;

// Core types
pub use hybrid::manager::HybridStateManager;
pub use hybrid::router::{DataRouter, DataType, StorageLayer};
pub use kv::state::{ConflictStrategy, Operation, StateChange, StateManager};
pub use memory::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, ManagedMemoryManager,
    MaterializedSnapshot, MemoryFragment, MemoryFragmentMetadata, MemoryMetadataPage,
    MemoryMetadataPageRequest, MemorySnapshot, MemorySnapshotMetadata, MemorySummary,
    ResumeSource, SnapshotScope,
};
pub use repository::Repository;

#[cfg(feature = "sqlx")]
pub use postgres::pool::PostgresConnection;
#[cfg(feature = "sqlx")]
pub use postgres::queries::{AgentRecord, AuditEntry, CheckpointRow, ConfigRecord, StateRow};

// Audit persistence (security + sqlx feature gates)
#[cfg(all(feature = "security", feature = "sqlx"))]
pub use audit_persister::AuditPersister;
