//! Typed Guard / Advisor policy for predictive supervision.

use mister_smith_core::{
    FailureClass, GuardDecision, GuardDecisionId, GuardError, GuardEvidence, GuardTarget,
    HealthState, InterventionType, SupervisionDecisionBasis,
};

use crate::execution_graph::BranchCheckpoint;
use crate::profile::ProfileAssessment;

/// Guard policy thresholds used to classify supervisory interventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardPolicy {
    /// Semantic signal severity that triggers branch isolation instead of refresh.
    pub semantic_isolation_threshold: u8,
}

impl Default for GuardPolicy {
    fn default() -> Self {
        Self {
            semantic_isolation_threshold: 90,
        }
    }
}

/// Input surface for evaluating a branch-, node-, graph-, or provider-local Guard decision.
#[derive(Debug, Clone, PartialEq)]
pub struct GuardContext {
    target: GuardTarget,
    profile: Option<ProfileAssessment>,
    checkpoints: Vec<BranchCheckpoint>,
    control_plane_fresh: bool,
    memory_metadata_available: bool,
}

impl GuardContext {
    /// Create a new Guard context for the given target.
    pub fn new(target: GuardTarget) -> Self {
        Self {
            target,
            profile: None,
            checkpoints: Vec::new(),
            control_plane_fresh: true,
            memory_metadata_available: true,
        }
    }

    /// Attach an assessed profile snapshot to the context.
    pub fn with_profile(mut self, profile: ProfileAssessment) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Attach checkpoint lineage that can inform branch-local recovery.
    pub fn with_checkpoints(mut self, checkpoints: Vec<BranchCheckpoint>) -> Self {
        self.checkpoints = checkpoints;
        self
    }

    /// Mark whether the control plane view is fresh enough to widen autonomy.
    pub fn with_control_plane_fresh(mut self, fresh: bool) -> Self {
        self.control_plane_fresh = fresh;
        self
    }

    /// Mark whether managed-memory metadata is available.
    pub fn with_memory_metadata_available(mut self, available: bool) -> Self {
        self.memory_metadata_available = available;
        self
    }

    /// Borrow the target scope.
    pub fn target(&self) -> &GuardTarget {
        &self.target
    }

    /// Borrow the assessed profile, when available.
    pub fn profile(&self) -> Option<&ProfileAssessment> {
        self.profile.as_ref()
    }
}

/// Predictive supervision classifier that maps evidence to typed interventions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guard {
    policy: GuardPolicy,
}

impl Guard {
    /// Create a new Guard instance.
    pub fn new(policy: GuardPolicy) -> Self {
        Self { policy }
    }

    /// Evaluate the current context and produce a typed Guard decision.
    pub fn evaluate(&self, context: &GuardContext) -> Result<GuardDecision, GuardError> {
        let mut notes = Vec::new();
        if !context.control_plane_fresh {
            notes.push("conservative fallback: control-plane state unavailable".to_string());
        }
        if !context.memory_metadata_available {
            notes.push("conservative fallback: memory metadata unavailable".to_string());
        }

        let assessment = context.profile();
        let profile = assessment.and_then(ProfileAssessment::snapshot);
        if profile.is_none() {
            notes.push("conservative fallback: profile data unavailable".to_string());
        }

        if !notes.is_empty() {
            let signal_descriptions = profile
                .map(|snapshot| {
                    snapshot
                        .semantic_signals
                        .iter()
                        .map(|signal| signal.detail.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            return Ok(GuardDecision {
                decision_id: GuardDecisionId::new(),
                failure_class: FailureClass::Structural,
                intervention: InterventionType::Escalation,
                evidence: GuardEvidence {
                    profile_id: profile.map(|snapshot| snapshot.profile_id),
                    decision_basis: SupervisionDecisionBasis::ConservativeFallback,
                    signal_descriptions,
                    checkpoint_ids: context
                        .checkpoints
                        .iter()
                        .map(|checkpoint| checkpoint.checkpoint_id)
                        .collect(),
                    notes,
                },
                target_scope: context.target.clone(),
                operator_visibility: true,
            });
        }

        let profile = profile.expect("checked above");
        let mut signal_descriptions = profile
            .semantic_signals
            .iter()
            .map(|signal| signal.detail.clone())
            .collect::<Vec<_>>();
        if signal_descriptions.is_empty() {
            signal_descriptions.push(format!("health state: {:?}", profile.health_state));
        }
        let mut evidence_notes = assessment
            .map(|profile| profile.notes().to_vec())
            .unwrap_or_default();

        let (failure_class, intervention) = if profile.semantic_signals.iter().any(|signal| {
            signal.signal_kind == mister_smith_core::SemanticSignalKind::PolicyConflict
        }) {
            evidence_notes.push("policy conflict requires escalation".to_string());
            (FailureClass::Structural, InterventionType::Escalation)
        } else if profile
            .semantic_signals
            .iter()
            .any(|signal| signal.signal_kind == mister_smith_core::SemanticSignalKind::Stalled)
        {
            evidence_notes
                .push("stream degradation is recoverable without global restart".to_string());
            (FailureClass::Streaming, InterventionType::Retry)
        } else if let Some(signal) = profile.semantic_signals.iter().find(|signal| {
            matches!(
                signal.signal_kind,
                mister_smith_core::SemanticSignalKind::Repetitive
                    | mister_smith_core::SemanticSignalKind::LowConfidence
            )
        }) {
            if signal.severity >= self.policy.semantic_isolation_threshold
                || profile.health_state == HealthState::Unhealthy
            {
                evidence_notes
                    .push("semantic degradation exceeded isolation threshold".to_string());
                (FailureClass::Semantic, InterventionType::BranchIsolation)
            } else {
                (FailureClass::Semantic, InterventionType::ContextRefresh)
            }
        } else if profile.semantic_signals.iter().any(|signal| {
            signal.signal_kind == mister_smith_core::SemanticSignalKind::MissingContext
        }) {
            (FailureClass::Semantic, InterventionType::ContextRefresh)
        } else if matches!(
            profile.health_state,
            HealthState::Degraded | HealthState::Unhealthy
        ) {
            evidence_notes
                .push("health degradation suggests retry before graph-wide restart".to_string());
            (FailureClass::Transient, InterventionType::Retry)
        } else {
            return Err(GuardError::InsufficientEvidence(
                "no supervisory evidence available for a non-trivial intervention".to_string(),
            ));
        };

        Ok(GuardDecision {
            decision_id: GuardDecisionId::new(),
            failure_class,
            intervention,
            evidence: GuardEvidence {
                profile_id: Some(profile.profile_id),
                decision_basis: SupervisionDecisionBasis::LiveSignalsOnly,
                signal_descriptions,
                checkpoint_ids: context
                    .checkpoints
                    .iter()
                    .map(|checkpoint| checkpoint.checkpoint_id)
                    .collect(),
                notes: evidence_notes,
            },
            target_scope: context.target.clone(),
            operator_visibility: true,
        })
    }
}
