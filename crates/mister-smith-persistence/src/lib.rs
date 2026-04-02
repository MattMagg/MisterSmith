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
pub use kv::fingerprints::{profile_fingerprint_key, ProfileFingerprintStore};
pub use kv::state::{
    branch_checkpoint_state_key, branch_resume_state_key, history_compaction_state_key,
    lifecycle_decision_state_key, workflow_effect_boundary_state_key, workflow_history_state_key,
    ConflictStrategy, Operation, StateChange, StateManager,
};
pub use memory::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, ManagedMemoryManager,
    MaterializedSnapshot, MemoryFragment, MemoryFragmentMetadata, MemoryMetadataPage,
    MemoryMetadataPageRequest, MemorySnapshot, MemorySnapshotMetadata, MemorySummary, ResumeSource,
    SnapshotScope,
};
pub use repository::session::SessionRepository;
pub use repository::task::{
    branch_resume_history, effect_boundary_records, history_compaction_records,
    latest_branch_checkpoint, latest_history_compaction, lifecycle_decision_history,
    merge_branch_checkpoint_metadata, merge_branch_resume_metadata, merge_effect_boundary_metadata,
    merge_history_compaction_metadata, merge_lifecycle_decision_metadata,
    merge_workflow_history_metadata, workflow_history, BranchCheckpointRecord, BranchResumeRecord,
    EffectBoundaryRecord, HistoryCompactionRecord, LifecycleDecisionRecord,
    WorkflowHistoryEventRecord,
};
pub use repository::Repository;

#[cfg(feature = "sqlx")]
pub use postgres::pool::PostgresConnection;
#[cfg(feature = "sqlx")]
pub use postgres::queries::{
    AgentRecord, AuditEntry, CheckpointRow, ConfigRecord, SessionRecord, SessionTurnRecord,
    StateRow,
};

// Audit persistence (security + sqlx feature gates)
#[cfg(all(feature = "security", feature = "sqlx"))]
pub use audit_persister::AuditPersister;
