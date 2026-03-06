//! Strategy execution: restart policies, scope filtering, budget checking, backoff.
//!
//! Applies `RestartPolicy`, `RestartScope`, and `BackoffStrategy` from core.

use std::time::{Duration, Instant};

use mister_smith_core::{AgentId, BackoffStrategy, RestartPolicy, RestartScope};

use crate::supervisor::SupervisorNode;

/// The decision made by the supervision strategy after a child failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisionDecision {
    /// Restart the specified actors.
    Restart(Vec<AgentId>),
    /// Escalate the failure to the parent supervisor.
    Escalate,
    /// Stop the failed actor (do not restart).
    Stop(AgentId),
    /// Ignore the failure.
    Ignore,
    /// Shut down the entire supervision tree.
    Shutdown,
}

/// Why an actor terminated, for scope filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationType {
    /// Normal termination (requested shutdown, mailbox closed).
    Normal,
    /// Abnormal termination (error, panic).
    Error,
}

/// Determine which children to restart based on the restart policy.
///
/// - `OneForOne`: Only the failed child.
/// - `OneForAll`: All children.
/// - `RestForOne`: The failed child and all children started after it.
pub fn apply_restart_policy(node: &SupervisorNode, failed_child_id: AgentId) -> Vec<AgentId> {
    match node.strategy.restart_policy {
        RestartPolicy::OneForOne => {
            vec![failed_child_id]
        }
        RestartPolicy::OneForAll => node.children.iter().map(|c| c.actor_id).collect(),
        RestartPolicy::RestForOne => {
            // Find the failed child's start_order
            if let Some(failed) = node.find_child(&failed_child_id) {
                let failed_order = failed.start_order;
                node.children
                    .iter()
                    .filter(|c| c.start_order >= failed_order)
                    .map(|c| c.actor_id)
                    .collect()
            } else {
                vec![failed_child_id]
            }
        }
    }
}

/// Check whether a child should be restarted based on its restart scope
/// and the type of termination.
///
/// - `Permanent`: Always restart.
/// - `Transient`: Restart only on error, not on normal termination.
/// - `Temporary`: Never restart.
pub fn should_restart(scope: RestartScope, termination_type: TerminationType) -> bool {
    match scope {
        RestartScope::Permanent => true,
        RestartScope::Transient => termination_type == TerminationType::Error,
        RestartScope::Temporary => false,
    }
}

/// Check whether the supervisor's restart budget allows another restart.
///
/// Prunes expired entries from the restart history (outside the failure window),
/// then checks if the count exceeds `max_failures`.
///
/// Returns `true` if the restart is allowed, `false` if the budget is exhausted.
pub fn check_restart_budget(node: &mut SupervisorNode) -> bool {
    let window = node.strategy.failure_window;
    let now = Instant::now();

    // Prune expired entries
    while let Some(front) = node.restart_history.front() {
        if now.duration_since(*front) > window {
            node.restart_history.pop_front();
        } else {
            break;
        }
    }

    // Check budget
    node.restart_history.len() < node.strategy.max_failures as usize
}

/// Compute the backoff delay for the given attempt number.
///
/// - `Fixed(d)`: Always returns `d`.
/// - `Exponential { initial, max, multiplier }`: Returns `min(initial * multiplier^attempt, max)`.
/// - `Linear { initial, increment }`: Returns `initial + increment * attempt`.
pub fn compute_backoff(strategy: &BackoffStrategy, attempt: u32) -> Duration {
    match strategy {
        BackoffStrategy::Fixed(d) => *d,
        BackoffStrategy::Exponential {
            initial,
            max,
            multiplier,
        } => {
            let delay_nanos = initial.as_nanos() as f64 * multiplier.powi(attempt as i32);
            let delay = Duration::from_nanos(delay_nanos.min(u64::MAX as f64) as u64);
            if delay > *max {
                *max
            } else {
                delay
            }
        }
        BackoffStrategy::Linear { initial, increment } => *initial + *increment * attempt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::SupervisorNode;
    use mister_smith_core::SupervisionStrategy;

    fn make_node(policy: RestartPolicy, max_failures: u32) -> SupervisorNode {
        let id = AgentId::new();
        SupervisorNode::new(
            id,
            SupervisionStrategy {
                restart_policy: policy,
                max_failures,
                ..Default::default()
            },
        )
    }

    // T049: Strategy executor tests

    #[test]
    fn one_for_one_returns_only_failed() {
        let mut node = make_node(RestartPolicy::OneForOne, 3);
        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        node.add_child(a, RestartScope::Permanent);
        node.add_child(b, RestartScope::Permanent);
        node.add_child(c, RestartScope::Permanent);

        let result = apply_restart_policy(&node, b);
        assert_eq!(result, vec![b]);
    }

    #[test]
    fn one_for_all_returns_all_children() {
        let mut node = make_node(RestartPolicy::OneForAll, 3);
        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        node.add_child(a, RestartScope::Permanent);
        node.add_child(b, RestartScope::Permanent);
        node.add_child(c, RestartScope::Permanent);

        let result = apply_restart_policy(&node, b);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&a));
        assert!(result.contains(&b));
        assert!(result.contains(&c));
    }

    #[test]
    fn rest_for_one_returns_failed_and_younger() {
        let mut node = make_node(RestartPolicy::RestForOne, 3);
        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        node.add_child(a, RestartScope::Permanent);
        node.add_child(b, RestartScope::Permanent);
        node.add_child(c, RestartScope::Permanent);

        let result = apply_restart_policy(&node, b);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&b));
        assert!(result.contains(&c));
        assert!(!result.contains(&a));
    }

    // T050: RestartScope tests

    #[test]
    fn permanent_always_restarts() {
        assert!(should_restart(
            RestartScope::Permanent,
            TerminationType::Normal
        ));
        assert!(should_restart(
            RestartScope::Permanent,
            TerminationType::Error
        ));
    }

    #[test]
    fn transient_restarts_on_error_only() {
        assert!(!should_restart(
            RestartScope::Transient,
            TerminationType::Normal
        ));
        assert!(should_restart(
            RestartScope::Transient,
            TerminationType::Error
        ));
    }

    #[test]
    fn temporary_never_restarts() {
        assert!(!should_restart(
            RestartScope::Temporary,
            TerminationType::Normal
        ));
        assert!(!should_restart(
            RestartScope::Temporary,
            TerminationType::Error
        ));
    }

    // T051: Restart budget tests

    #[test]
    fn budget_allows_within_limit() {
        let mut node = make_node(RestartPolicy::OneForOne, 3);

        // 3 failures within window should be OK (budget is max_failures, meaning < max_failures allowed)
        node.record_restart();
        node.record_restart();
        assert!(check_restart_budget(&mut node)); // 2 < 3

        node.record_restart();
        assert!(!check_restart_budget(&mut node)); // 3 >= 3, budget exhausted
    }

    #[test]
    fn budget_prunes_expired_entries() {
        let mut node = make_node(RestartPolicy::OneForOne, 3);
        // Simulate old entries by manually inserting past timestamps
        let old = Instant::now() - Duration::from_secs(120); // well outside 60s window
        node.restart_history.push_back(old);
        node.restart_history.push_back(old);
        node.restart_history.push_back(old);

        // Should prune all expired and allow restart
        assert!(check_restart_budget(&mut node));
        assert!(node.restart_history.is_empty());
    }

    #[test]
    fn budget_exhaustion_triggers_correctly() {
        let mut node = make_node(RestartPolicy::OneForOne, 3);

        // 3 recent restarts
        for _ in 0..3 {
            node.record_restart();
        }

        // 4th should be denied
        assert!(!check_restart_budget(&mut node));
    }

    // T052: Backoff computation tests

    #[test]
    fn exponential_backoff() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(30),
            multiplier: 2.0,
        };

        let d0 = compute_backoff(&strategy, 0);
        let d1 = compute_backoff(&strategy, 1);
        let d2 = compute_backoff(&strategy, 2);

        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn exponential_backoff_respects_max() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(5),
            multiplier: 10.0,
        };

        let d3 = compute_backoff(&strategy, 3); // 1000s, capped at 5s
        assert_eq!(d3, Duration::from_secs(5));
    }

    #[test]
    fn fixed_backoff() {
        let strategy = BackoffStrategy::Fixed(Duration::from_millis(500));
        assert_eq!(compute_backoff(&strategy, 0), Duration::from_millis(500));
        assert_eq!(compute_backoff(&strategy, 5), Duration::from_millis(500));
        assert_eq!(compute_backoff(&strategy, 100), Duration::from_millis(500));
    }

    #[test]
    fn linear_backoff() {
        let strategy = BackoffStrategy::Linear {
            initial: Duration::from_millis(100),
            increment: Duration::from_millis(50),
        };

        assert_eq!(compute_backoff(&strategy, 0), Duration::from_millis(100));
        assert_eq!(compute_backoff(&strategy, 1), Duration::from_millis(150));
        assert_eq!(compute_backoff(&strategy, 2), Duration::from_millis(200));
        assert_eq!(compute_backoff(&strategy, 4), Duration::from_millis(300));
    }
}
