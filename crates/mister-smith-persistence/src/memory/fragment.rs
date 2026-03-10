use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mister_smith_core::{
    AgentId, AgentType, ExecutionBranchId, ExecutionNodeId, MemoryFragmentId, TaskId,
};

/// Scope a managed-memory snapshot or fragment belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapshotScope {
    /// Context scoped to a specific runtime agent.
    Agent(AgentId),
    /// Context scoped to a single execution node.
    Node(ExecutionNodeId),
    /// Context scoped to a checkpointable execution branch.
    Branch(ExecutionBranchId),
    /// Context scoped to a workflow.
    Workflow(TaskId),
}

/// Class of managed-memory fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FragmentClass {
    /// Live working context still relevant to the active step.
    Working,
    /// Recent episodic memory captured from prior work.
    Episodic,
    /// Consolidated summary of older fragments.
    Summary,
    /// Resume-ready checkpoint context.
    Checkpoint,
    /// Audit or forensic context retained for visibility.
    Audit,
}

/// Recency and expiry metadata for a fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentFreshness {
    /// When the fragment was first recorded.
    pub recorded_at: DateTime<Utc>,
    /// When the fragment was last accessed.
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Optional expiry time.
    pub expires_at: Option<DateTime<Utc>>,
}

impl FragmentFreshness {
    /// Build a freshness policy with a TTL starting at `recorded_at`.
    pub fn ttl(recorded_at: DateTime<Utc>, ttl: Duration) -> Self {
        Self {
            recorded_at,
            last_accessed_at: None,
            expires_at: Some(recorded_at + ttl),
        }
    }

    /// Returns true when the fragment is expired at `now`.
    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at < now)
    }
}

/// Role and scope visibility policy for a fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// Roles allowed to consume the fragment.
    pub allowed_roles: Vec<AgentType>,
    /// Optional branch boundary this fragment must stay within.
    pub branch_id: Option<ExecutionBranchId>,
}

impl AccessPolicy {
    /// Create an access policy visible to the provided roles.
    pub fn for_roles(allowed_roles: Vec<AgentType>) -> Self {
        Self {
            allowed_roles,
            branch_id: None,
        }
    }

    /// Restrict the policy to a single branch.
    pub fn for_branch(mut self, branch_id: ExecutionBranchId) -> Self {
        self.branch_id = Some(branch_id);
        self
    }

    /// Returns true when the policy permits `role` for `scope`.
    pub fn allows(&self, role: AgentType, scope: &SnapshotScope) -> bool {
        let role_allowed = self.allowed_roles.contains(&role);
        let branch_allowed = match (self.branch_id, scope) {
            (Some(expected), SnapshotScope::Branch(actual)) => expected == *actual,
            (Some(_), _) => false,
            (None, _) => true,
        };

        role_allowed && branch_allowed
    }
}

/// Provenance metadata for a managed-memory fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentProvenance {
    /// Workflow the fragment belongs to.
    pub workflow_id: TaskId,
    /// Branch the fragment belongs to, when applicable.
    pub branch_id: Option<ExecutionBranchId>,
    /// Runtime agent that produced the fragment.
    pub source_agent_id: AgentId,
    /// Agent role that produced the fragment.
    pub source_role: AgentType,
    /// Logical source key or namespace.
    pub source_key: String,
    /// Fragments this one was derived from.
    pub derived_from: Vec<MemoryFragmentId>,
    /// Timestamp the fragment was recorded.
    pub recorded_at: DateTime<Utc>,
}

impl FragmentProvenance {
    /// Create new provenance metadata for a fragment.
    pub fn new(
        workflow_id: TaskId,
        branch_id: Option<ExecutionBranchId>,
        source_agent_id: AgentId,
        source_role: AgentType,
        source_key: impl Into<String>,
    ) -> Self {
        Self {
            workflow_id,
            branch_id,
            source_agent_id,
            source_role,
            source_key: source_key.into(),
            derived_from: Vec::new(),
            recorded_at: Utc::now(),
        }
    }
}

/// Lightweight fragment metadata suitable for task-level indexes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFragmentMetadata {
    /// Stable fragment identifier.
    pub fragment_id: MemoryFragmentId,
    /// Scope the fragment belongs to.
    pub scope: SnapshotScope,
    /// Runtime agent that produced the fragment.
    pub source_agent_id: AgentId,
    /// Fragment role/source type.
    pub source_role: AgentType,
    /// Logical source key or namespace.
    pub source_key: String,
    /// Fragment class.
    pub fragment_class: FragmentClass,
    /// Context units represented by the fragment.
    pub units: u64,
    /// Monotonic version captured in the index.
    pub version: u64,
    /// When the fragment was recorded.
    pub recorded_at: DateTime<Utc>,
}

/// Persisted managed-memory fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// Stable fragment identifier.
    pub fragment_id: MemoryFragmentId,
    /// Scope the fragment belongs to.
    pub scope: SnapshotScope,
    /// Stored context payload.
    pub content: Value,
    /// Provenance metadata.
    pub provenance: FragmentProvenance,
    /// Freshness and expiry metadata.
    pub freshness: FragmentFreshness,
    /// Visibility constraints for role-aware routing.
    pub access_policy: AccessPolicy,
    /// Monotonic version for future updates.
    pub version: u64,
    /// Semantic class of the fragment.
    pub fragment_class: FragmentClass,
    /// Estimated context units the fragment consumes.
    pub units: u64,
}

impl MemoryFragment {
    /// Create a new managed-memory fragment.
    pub fn new(
        scope: SnapshotScope,
        content: Value,
        units: u64,
        fragment_class: FragmentClass,
        provenance: FragmentProvenance,
        freshness: FragmentFreshness,
        access_policy: AccessPolicy,
    ) -> Self {
        Self {
            fragment_id: MemoryFragmentId::new(),
            scope,
            content,
            provenance,
            freshness,
            access_policy,
            version: 1,
            fragment_class,
            units,
        }
    }

    /// Returns true when this fragment is visible to `role` for `scope`.
    pub fn is_visible_to(
        &self,
        role: AgentType,
        scope: &SnapshotScope,
        now: DateTime<Utc>,
    ) -> bool {
        self.scope == *scope
            && !self.freshness.is_expired_at(now)
            && self.access_policy.allows(role, scope)
    }

    /// Build lightweight fragment metadata for task-level indexes.
    pub fn metadata(&self) -> MemoryFragmentMetadata {
        MemoryFragmentMetadata {
            fragment_id: self.fragment_id,
            scope: self.scope.clone(),
            source_agent_id: self.provenance.source_agent_id,
            source_role: self.provenance.source_role,
            source_key: self.provenance.source_key.clone(),
            fragment_class: self.fragment_class,
            units: self.units,
            version: self.version,
            recorded_at: self.provenance.recorded_at,
        }
    }
}
