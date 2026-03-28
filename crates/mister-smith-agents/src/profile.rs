//! Supervisory profile helpers for Guard / Advisor decisions.

use chrono::Utc;
use mister_smith_core::ExecutionBranchId;
use mister_smith_core::{
    GuardTarget, HealthState, ProfileFingerprint, ProfileSnapshot, ProfileSnapshotId,
    ProfileTarget, SemanticSignal,
};
use mister_smith_persistence::ProfileFingerprintStore;

use crate::execution_graph::ExecutionGraph;

/// Profile data plus operator-facing notes gathered before a Guard decision.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileAssessment {
    snapshot: Option<ProfileSnapshot>,
    fingerprint: Option<ProfileFingerprint>,
    target: Option<GuardTarget>,
    notes: Vec<String>,
}

impl ProfileAssessment {
    /// Create a new profile assessment.
    pub fn new(snapshot: Option<ProfileSnapshot>, notes: Vec<String>) -> Self {
        Self {
            snapshot,
            fingerprint: None,
            target: None,
            notes,
        }
    }

    /// Borrow the underlying profile snapshot when available.
    pub fn snapshot(&self) -> Option<&ProfileSnapshot> {
        self.snapshot.as_ref()
    }

    /// Borrow the runtime target associated with this profile, when known.
    pub fn target(&self) -> Option<&GuardTarget> {
        self.target.as_ref()
    }

    /// Borrow the advisory fingerprint that reinforced this assessment, when any.
    pub fn fingerprint(&self) -> Option<&ProfileFingerprint> {
        self.fingerprint.as_ref()
    }

    /// Resolve the owning branch for the current target when graph context exists.
    pub fn target_branch_id(&self, graph: &ExecutionGraph) -> Option<ExecutionBranchId> {
        match self.target.as_ref() {
            Some(GuardTarget::Branch(branch_id)) => Some(*branch_id),
            Some(GuardTarget::Node(node_id)) => graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .map(|node| node.branch_id),
            _ => None,
        }
    }

    /// Borrow operator-facing notes captured during profile assessment.
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// Attach a concrete runtime target to this assessment.
    pub fn with_target(mut self, target: GuardTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Attach a current advisory fingerprint to this assessment.
    pub fn with_fingerprint(mut self, fingerprint: ProfileFingerprint) -> Self {
        if let Some(snapshot) = self.snapshot.as_mut() {
            snapshot.fingerprint_ref = Some(ProfileFingerprintStore::reference(&fingerprint));
        }
        self.fingerprint = Some(fingerprint);
        self
    }

    /// Build a supervisory profile assessment from stream or runtime signals.
    pub fn from_supervisory_signals(
        target: &GuardTarget,
        semantic_signals: Vec<SemanticSignal>,
        notes: Vec<String>,
    ) -> Self {
        Self::new(
            Some(ProfileSnapshot {
                profile_id: ProfileSnapshotId::new(),
                target: profile_target(target),
                health_state: health_state(&semantic_signals),
                latency_window: None,
                error_window: None,
                semantic_signals,
                fingerprint_ref: None,
                updated_at: Utc::now(),
            }),
            notes,
        )
        .with_target(target.clone())
    }
}

fn profile_target(target: &GuardTarget) -> ProfileTarget {
    match target {
        GuardTarget::Node(_) => ProfileTarget::Agent,
        GuardTarget::Branch(_) => ProfileTarget::Branch,
        GuardTarget::Graph(_) => ProfileTarget::Topology,
        GuardTarget::Provider(_) => ProfileTarget::Provider,
    }
}

fn health_state(semantic_signals: &[SemanticSignal]) -> HealthState {
    if semantic_signals.is_empty() {
        return HealthState::Healthy;
    }

    if semantic_signals.iter().any(|signal| signal.severity >= 90) {
        HealthState::Unhealthy
    } else {
        HealthState::Degraded
    }
}
