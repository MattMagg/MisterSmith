//! Managed memory fragments, snapshots, and consolidation helpers for Phase 10.

pub mod consolidation;
pub mod fragment;
pub mod manager;
pub mod snapshot;

pub use consolidation::{build_summary, consolidate_fragments};
pub use fragment::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, MemoryFragment,
    MemoryFragmentMetadata, SnapshotScope,
};
pub use manager::ManagedMemoryManager;
pub use snapshot::{
    MaterializedSnapshot, MemoryMetadataPage, MemoryMetadataPageRequest, MemorySnapshot,
    MemorySnapshotMetadata, MemorySummary, ResumeSource,
};
