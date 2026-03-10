use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mister_smith_core::{AgentType, ContextBudgetId, MemoryFragmentId, MemorySnapshotId};

use super::fragment::{MemoryFragment, SnapshotScope};

/// Inline summary attached to a snapshot when older context is reduced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySummary {
    /// Fragment lineage covered by the summary.
    pub derived_from: Vec<MemoryFragmentId>,
    /// Summary payload.
    pub content: Value,
    /// Units consumed by the summary.
    pub units: u64,
}

/// Lightweight snapshot metadata suitable for task-level indexes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySnapshotMetadata {
    /// Stable snapshot identifier.
    pub snapshot_id: MemorySnapshotId,
    /// Scope the snapshot reconstructs.
    pub target_scope: SnapshotScope,
    /// Role the snapshot was assembled for.
    pub role: AgentType,
    /// Context units delivered by the snapshot.
    pub delivered_units: u64,
    /// Candidate units before reduction.
    pub total_candidate_units: u64,
    /// Optional checkpoint fragment anchoring resume.
    pub checkpoint_fragment_id: Option<MemoryFragmentId>,
    /// Budget used to assemble the snapshot.
    pub budget_id: ContextBudgetId,
    /// Number of fragments captured in the snapshot.
    pub fragment_count: usize,
    /// Whether the snapshot included a reduced summary.
    pub has_summary: bool,
    /// Snapshot creation time.
    pub created_at: DateTime<Utc>,
}

/// Page request for persisted managed-memory metadata indexes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMetadataPageRequest {
    /// Zero-based starting offset into the filtered metadata set.
    pub offset: usize,
    /// Maximum number of entries to return.
    pub limit: usize,
    /// Optional scope filter.
    pub scope: Option<SnapshotScope>,
    /// Optional role filter.
    pub role: Option<AgentType>,
}

/// Page of persisted managed-memory metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMetadataPage<T> {
    /// Entries returned for the requested page.
    pub entries: Vec<T>,
    /// Zero-based starting offset used for this page.
    pub offset: usize,
    /// Maximum entries requested for the page.
    pub limit: usize,
    /// Total entries available after filtering.
    pub total_entries: usize,
    /// Offset to request the next page, when more entries remain.
    pub next_offset: Option<usize>,
}

/// Checkpoint-ready snapshot assembled from managed-memory fragments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Stable snapshot identifier.
    pub snapshot_id: MemorySnapshotId,
    /// Scope the snapshot reconstructs.
    pub target_scope: SnapshotScope,
    /// Role the snapshot was assembled for.
    pub role: AgentType,
    /// Included fragment identifiers.
    pub fragment_ids: Vec<MemoryFragmentId>,
    /// Optional inline summary for reduced history.
    pub summary: Option<MemorySummary>,
    /// When the snapshot was created.
    pub created_at: DateTime<Utc>,
    /// Budget used during assembly.
    pub budget_id: ContextBudgetId,
    /// Candidate units before reduction.
    pub total_candidate_units: u64,
    /// Delivered units after reduction.
    pub delivered_units: u64,
    /// Optional checkpoint fragment that can resume without replaying raw history.
    pub checkpoint_fragment_id: Option<MemoryFragmentId>,
}

impl MemorySnapshot {
    /// Build lightweight metadata for task-level indexes.
    pub fn metadata(&self) -> MemorySnapshotMetadata {
        MemorySnapshotMetadata {
            snapshot_id: self.snapshot_id,
            target_scope: self.target_scope.clone(),
            role: self.role,
            delivered_units: self.delivered_units,
            total_candidate_units: self.total_candidate_units,
            checkpoint_fragment_id: self.checkpoint_fragment_id,
            budget_id: self.budget_id,
            fragment_count: self.fragment_ids.len(),
            has_summary: self.summary.is_some(),
            created_at: self.created_at,
        }
    }
}

/// Source used to reconstruct a snapshot during resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResumeSource {
    /// The snapshot resumed from a dedicated checkpoint fragment.
    Checkpoint,
    /// The snapshot resumed from its selected fragment set.
    FragmentSelection,
}

/// Materialized snapshot payload returned during resume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializedSnapshot {
    /// Snapshot metadata.
    pub snapshot: MemorySnapshot,
    /// Fragments returned to the caller for reconstruction.
    pub fragments: Vec<MemoryFragment>,
    /// Reconstruction source actually used.
    pub resume_source: ResumeSource,
}
