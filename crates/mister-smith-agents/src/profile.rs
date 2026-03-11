//! Supervisory profile helpers for Guard / Advisor decisions.

use chrono::Utc;
use mister_smith_core::ProfileSnapshot;
use mister_smith_core::{
    GuardTarget, HealthState, ProfileSnapshotId, ProfileTarget, SemanticSignal,
};

/// Profile data plus operator-facing notes gathered before a Guard decision.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileAssessment {
    snapshot: Option<ProfileSnapshot>,
    notes: Vec<String>,
}

impl ProfileAssessment {
    /// Create a new profile assessment.
    pub fn new(snapshot: Option<ProfileSnapshot>, notes: Vec<String>) -> Self {
        Self { snapshot, notes }
    }

    /// Borrow the underlying profile snapshot when available.
    pub fn snapshot(&self) -> Option<&ProfileSnapshot> {
        self.snapshot.as_ref()
    }

    /// Borrow operator-facing notes captured during profile assessment.
    pub fn notes(&self) -> &[String] {
        &self.notes
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
                updated_at: Utc::now(),
            }),
            notes,
        )
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
