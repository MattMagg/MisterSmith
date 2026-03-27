//! Supervision model types: restart policies, escalation, and backoff strategies.
//!
//! Implements an OTP-inspired supervision model for managing agent lifecycles.

use crate::enums::RepairDirectiveAction;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Strategy for restarting children when one fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Restart only the failed child.
    OneForOne,
    /// Restart all children when one fails.
    OneForAll,
    /// Restart the failed child and all children started after it.
    RestForOne,
}

/// Restart scope controlling whether a child is restarted after failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartScope {
    /// Always restart on any exit.
    Permanent,
    /// Restart only on abnormal exit.
    Transient,
    /// Never restart.
    Temporary,
}

/// Policy for handling failures that exceed restart limits.
///
/// This is the canonical definition reconciling definitions across
/// `agent-lifecycle.md` and `async-patterns.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationPolicy {
    /// Terminate the supervisor and all children.
    Terminate,
    /// Restart the entire supervision subtree.
    Restart,
    /// Escalate the failure to the parent supervisor.
    Escalate,
    /// Log the failure and continue operation.
    LogAndIgnore,
}

/// Backoff strategy for restart delays.
///
/// Cannot derive `Copy` because struct variants contain `Duration` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between restarts.
    Fixed(Duration),
    /// Exponential backoff with configurable multiplier and ceiling.
    Exponential {
        /// Initial delay before first retry.
        initial: Duration,
        /// Maximum delay ceiling.
        max: Duration,
        /// Multiplier applied per retry.
        multiplier: f64,
    },
    /// Linear backoff with fixed increment per retry.
    Linear {
        /// Initial delay before first retry.
        initial: Duration,
        /// Amount added to delay per retry.
        increment: Duration,
    },
}

/// Bounded repair action emitted after verifier rejection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairDirective {
    /// Repair action selected for the rejected step.
    pub action: RepairDirectiveAction,
    /// Runtime surface that issued the directive.
    pub issued_by: String,
    /// Stable reference to preserved rejection diagnostics.
    pub failure_context_ref: String,
    /// Remaining local retry budget for the directive.
    pub retry_budget_remaining: u32,
}

/// Complete supervision configuration for a supervisor node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionStrategy {
    /// How to restart children on failure.
    pub restart_policy: RestartPolicy,
    /// Maximum number of failures within the window before escalation.
    pub max_failures: u32,
    /// Time window for counting failures.
    pub failure_window: Duration,
    /// What to do when max_failures is exceeded.
    pub escalation_policy: EscalationPolicy,
    /// Delay strategy between restart attempts.
    pub backoff_strategy: BackoffStrategy,
}

impl Default for SupervisionStrategy {
    fn default() -> Self {
        Self {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 3,
            failure_window: Duration::from_secs(60),
            escalation_policy: EscalationPolicy::Escalate,
            backoff_strategy: BackoffStrategy::Exponential {
                initial: Duration::from_millis(100),
                max: Duration::from_secs(30),
                multiplier: 2.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy() {
        let strategy = SupervisionStrategy::default();
        assert_eq!(strategy.restart_policy, RestartPolicy::OneForOne);
        assert_eq!(strategy.max_failures, 3);
        assert_eq!(strategy.failure_window, Duration::from_secs(60));
        assert_eq!(strategy.escalation_policy, EscalationPolicy::Escalate);
    }

    #[test]
    fn serde_roundtrip() {
        let strategy = SupervisionStrategy::default();
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: SupervisionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.restart_policy, strategy.restart_policy);
        assert_eq!(deserialized.max_failures, strategy.max_failures);
        assert_eq!(deserialized.escalation_policy, strategy.escalation_policy);
    }

    #[test]
    fn all_restart_policies() {
        let policies = [
            RestartPolicy::OneForOne,
            RestartPolicy::OneForAll,
            RestartPolicy::RestForOne,
        ];
        for policy in &policies {
            let json = serde_json::to_string(policy).unwrap();
            let deserialized: RestartPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(*policy, deserialized);
        }
    }

    #[test]
    fn all_escalation_policies() {
        let policies = [
            EscalationPolicy::Terminate,
            EscalationPolicy::Restart,
            EscalationPolicy::Escalate,
            EscalationPolicy::LogAndIgnore,
        ];
        for policy in &policies {
            let json = serde_json::to_string(policy).unwrap();
            let deserialized: EscalationPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(*policy, deserialized);
        }
    }

    #[test]
    fn repair_directive_serde_roundtrip() {
        let directive = RepairDirective {
            action: RepairDirectiveAction::RetryStep,
            issued_by: "verifier.runtime".to_string(),
            failure_context_ref: "step-1/missing-context".to_string(),
            retry_budget_remaining: 2,
        };

        let json = serde_json::to_string(&directive).unwrap();
        let deserialized: RepairDirective = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, directive);
    }
}
