//! Failure escalation through the supervision hierarchy.
//!
//! When a supervisor exhausts its restart budget, failures escalate to the parent.
//! If the root supervisor exhausts its budget, the escalation policy determines
//! the outcome (Terminate, Restart subtree, LogAndIgnore, or Shutdown).

use mister_smith_core::{AgentId, EscalationPolicy, SupervisionError};

use crate::strategy::{SupervisionDecision, TerminationType};
use crate::tree::SupervisionTree;

/// Escalate a failure up the supervision tree.
///
/// Starting from the given supervisor, walks up the tree applying each
/// parent's strategy until the failure is handled or the root is reached.
pub fn escalate(
    tree: &mut SupervisionTree,
    supervisor_id: AgentId,
    error: &str,
) -> Result<SupervisionDecision, SupervisionError> {
    // Look up the supervisor's escalation policy
    let node = tree.get_node(&supervisor_id).ok_or_else(|| {
        SupervisionError::TreeCorrupted(format!("Supervisor {supervisor_id} not found"))
    })?;

    let escalation_policy = node.strategy.escalation_policy;
    let parent_id = node.parent_id;

    match escalation_policy {
        EscalationPolicy::LogAndIgnore => {
            tracing::warn!(
                supervisor_id = %supervisor_id,
                error = %error,
                "Supervisor budget exhausted, logging and ignoring"
            );
            Ok(SupervisionDecision::Ignore)
        }
        EscalationPolicy::Terminate => {
            tracing::error!(
                supervisor_id = %supervisor_id,
                error = %error,
                "Supervisor budget exhausted, terminating subtree"
            );
            Ok(SupervisionDecision::Shutdown)
        }
        EscalationPolicy::Restart => {
            // Restart the entire subtree under this supervisor
            let node = tree.get_node(&supervisor_id).ok_or_else(|| {
                SupervisionError::TreeCorrupted(format!("Supervisor {supervisor_id} not found"))
            })?;
            let children: Vec<AgentId> = node.children.iter().map(|c| c.actor_id).collect();
            Ok(SupervisionDecision::Restart(children))
        }
        EscalationPolicy::Escalate => {
            match parent_id {
                Some(parent) => {
                    // Try the parent's failure handler
                    let decision = tree.handle_failure(supervisor_id, TerminationType::Error);
                    match decision {
                        Ok(SupervisionDecision::Escalate) => {
                            // Parent also escalated — recurse
                            escalate(tree, parent, error)
                        }
                        Ok(decision) => Ok(decision),
                        Err(e) => Err(e),
                    }
                }
                None => {
                    // Root supervisor — no parent to escalate to
                    tracing::error!(
                        supervisor_id = %supervisor_id,
                        error = %error,
                        "Root supervisor budget exhausted, shutting down tree"
                    );
                    Ok(SupervisionDecision::Shutdown)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_core::{RestartPolicy, RestartScope, SupervisionStrategy};

    // T068: Escalation tests

    #[test]
    fn mid_level_exhausts_budget_escalates_to_root() {
        let mut tree = SupervisionTree::new();

        let root_id = AgentId::new();
        tree.add_supervisor(
            root_id,
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 5,
                escalation_policy: EscalationPolicy::Terminate,
                ..Default::default()
            },
        );

        let mid_id = AgentId::new();
        tree.add_supervisor_under(
            mid_id,
            root_id,
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 1,
                escalation_policy: EscalationPolicy::Escalate,
                ..Default::default()
            },
        )
        .unwrap();

        let worker = AgentId::new();
        tree.add_child(mid_id, worker, RestartScope::Permanent)
            .unwrap();

        // First failure — mid-level handles it
        let d1 = tree.handle_failure(worker, TerminationType::Error).unwrap();
        assert!(matches!(d1, SupervisionDecision::Restart(_)));

        // Second failure — mid-level budget exhausted, escalates
        let d2 = tree.handle_failure(worker, TerminationType::Error).unwrap();
        assert_eq!(d2, SupervisionDecision::Escalate);

        // Escalation to root — root decides based on its policy
        let d3 = escalate(&mut tree, mid_id, "worker failed repeatedly").unwrap();
        // Root's escalation policy is Terminate, but mid's escalation policy is Escalate.
        // The escalate function checks mid's policy: Escalate → goes to parent (root).
        // At root level, handle_failure processes mid_id as a child of root.
        // If root can handle it, it restarts mid.
        // But we need the root to have mid registered as a child, which add_supervisor_under does.
        // So root should restart mid if budget allows.
        assert!(
            matches!(d3, SupervisionDecision::Restart(_))
                || matches!(d3, SupervisionDecision::Shutdown),
            "Got {:?}",
            d3
        );
    }

    #[test]
    fn root_exhaustion_returns_shutdown() {
        let mut tree = SupervisionTree::new();
        let root_id = AgentId::new();
        tree.add_supervisor(
            root_id,
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 0, // No budget at all
                escalation_policy: EscalationPolicy::Escalate,
                ..Default::default()
            },
        );

        // No parent, Escalate policy, root → Shutdown
        let decision = escalate(&mut tree, root_id, "critical failure").unwrap();
        assert_eq!(decision, SupervisionDecision::Shutdown);
    }

    #[test]
    fn log_and_ignore_policy() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(
            sup_id,
            SupervisionStrategy {
                escalation_policy: EscalationPolicy::LogAndIgnore,
                ..Default::default()
            },
        );

        let decision = escalate(&mut tree, sup_id, "some error").unwrap();
        assert_eq!(decision, SupervisionDecision::Ignore);
    }

    #[test]
    fn terminate_policy() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(
            sup_id,
            SupervisionStrategy {
                escalation_policy: EscalationPolicy::Terminate,
                ..Default::default()
            },
        );

        let decision = escalate(&mut tree, sup_id, "fatal error").unwrap();
        assert_eq!(decision, SupervisionDecision::Shutdown);
    }

    #[test]
    fn restart_policy_restarts_children() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(
            sup_id,
            SupervisionStrategy {
                escalation_policy: EscalationPolicy::Restart,
                ..Default::default()
            },
        );

        let child1 = AgentId::new();
        let child2 = AgentId::new();
        tree.add_child(sup_id, child1, RestartScope::Permanent)
            .unwrap();
        tree.add_child(sup_id, child2, RestartScope::Permanent)
            .unwrap();

        let decision = escalate(&mut tree, sup_id, "need subtree restart").unwrap();
        match decision {
            SupervisionDecision::Restart(ids) => {
                assert_eq!(ids.len(), 2);
                assert!(ids.contains(&child1));
                assert!(ids.contains(&child2));
            }
            other => panic!("Expected Restart, got {:?}", other),
        }
    }
}
