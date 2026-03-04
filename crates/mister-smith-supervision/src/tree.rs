//! Supervision tree: hierarchical node management.
//!
//! `SupervisionTree` manages the hierarchy of supervisor nodes,
//! handles failure routing, and provides tree status queries.

use std::collections::HashMap;

use mister_smith_core::{AgentId, RestartScope, SupervisionError, SupervisionStrategy};

use crate::strategy::{
    apply_restart_policy, check_restart_budget, should_restart, SupervisionDecision,
    TerminationType,
};
use crate::supervisor::SupervisorNode;

/// The supervision tree — manages the hierarchy of supervisor nodes.
#[derive(Debug)]
pub struct SupervisionTree {
    /// All supervisor nodes, keyed by their ID.
    nodes: HashMap<AgentId, SupervisorNode>,
    /// Mapping from child actor ID to their supervisor's ID.
    child_to_supervisor: HashMap<AgentId, AgentId>,
}

impl SupervisionTree {
    /// Create a new empty supervision tree.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            child_to_supervisor: HashMap::new(),
        }
    }

    /// Add a root-level supervisor to the tree.
    pub fn add_supervisor(
        &mut self,
        id: AgentId,
        strategy: SupervisionStrategy,
    ) -> &SupervisorNode {
        let node = SupervisorNode::new(id, strategy);
        self.nodes.insert(id, node);
        self.nodes.get(&id).unwrap()
    }

    /// Add a supervisor as a child of another supervisor.
    pub fn add_supervisor_under(
        &mut self,
        id: AgentId,
        parent_id: AgentId,
        strategy: SupervisionStrategy,
    ) -> Result<(), SupervisionError> {
        // Verify parent exists
        let parent = self.nodes.get_mut(&parent_id).ok_or_else(|| {
            SupervisionError::TreeCorrupted(format!("Parent supervisor {parent_id} not found"))
        })?;
        parent.add_child(id, RestartScope::Permanent);

        let node = SupervisorNode::with_parent(id, parent_id, strategy);
        self.nodes.insert(id, node);
        self.child_to_supervisor.insert(id, parent_id);
        Ok(())
    }

    /// Add a child actor under a supervisor.
    pub fn add_child(
        &mut self,
        supervisor_id: AgentId,
        child_id: AgentId,
        restart_scope: RestartScope,
    ) -> Result<(), SupervisionError> {
        let node = self.nodes.get_mut(&supervisor_id).ok_or_else(|| {
            SupervisionError::TreeCorrupted(format!("Supervisor {supervisor_id} not found"))
        })?;
        node.add_child(child_id, restart_scope);
        self.child_to_supervisor.insert(child_id, supervisor_id);
        Ok(())
    }

    /// Remove a child from its supervisor.
    pub fn remove_child(&mut self, child_id: &AgentId) -> Option<AgentId> {
        if let Some(supervisor_id) = self.child_to_supervisor.remove(child_id) {
            if let Some(node) = self.nodes.get_mut(&supervisor_id) {
                node.remove_child(child_id);
            }
            Some(supervisor_id)
        } else {
            None
        }
    }

    /// Find the supervisor for a given child actor.
    pub fn find_supervisor(&self, child_id: &AgentId) -> Option<AgentId> {
        self.child_to_supervisor.get(child_id).copied()
    }

    /// Handle a child failure: look up the supervisor, apply strategy,
    /// check scope and budget, return a decision.
    pub fn handle_failure(
        &mut self,
        child_id: AgentId,
        termination_type: TerminationType,
    ) -> Result<SupervisionDecision, SupervisionError> {
        let supervisor_id = self
            .child_to_supervisor
            .get(&child_id)
            .copied()
            .ok_or_else(|| {
                SupervisionError::TreeCorrupted(format!(
                    "No supervisor found for child {child_id}"
                ))
            })?;

        let node = self.nodes.get_mut(&supervisor_id).ok_or_else(|| {
            SupervisionError::TreeCorrupted(format!("Supervisor {supervisor_id} not found"))
        })?;

        // Check restart scope
        let child = node.find_child(&child_id);
        let scope = child
            .map(|c| c.restart_scope)
            .unwrap_or(RestartScope::Permanent);

        if !should_restart(scope, termination_type) {
            return Ok(SupervisionDecision::Stop(child_id));
        }

        // Check restart budget
        if !check_restart_budget(node) {
            return Ok(SupervisionDecision::Escalate);
        }

        // Record restart and compute affected children
        node.record_restart();
        let affected = apply_restart_policy(node, child_id);

        // Increment restart count for affected children
        for id in &affected {
            if let Some(child) = node.find_child_mut(id) {
                child.restart_count += 1;
            }
        }

        Ok(SupervisionDecision::Restart(affected))
    }

    /// Get a reference to a supervisor node.
    pub fn get_node(&self, id: &AgentId) -> Option<&SupervisorNode> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a supervisor node.
    pub fn get_node_mut(&mut self, id: &AgentId) -> Option<&mut SupervisorNode> {
        self.nodes.get_mut(id)
    }

    /// Returns the total number of nodes (supervisors + children).
    pub fn total_nodes(&self) -> usize {
        let mut all_ids: std::collections::HashSet<AgentId> = self.nodes.keys().copied().collect();
        for child_id in self.child_to_supervisor.keys() {
            all_ids.insert(*child_id);
        }
        all_ids.len()
    }

    /// Compute the maximum depth of the tree.
    pub fn tree_depth(&self) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }

        let mut max_depth = 0;
        for node in self.nodes.values() {
            if node.parent_id.is_none() {
                let depth = self.compute_depth(&node.id);
                max_depth = max_depth.max(depth);
            }
        }
        max_depth
    }

    fn compute_depth(&self, supervisor_id: &AgentId) -> usize {
        let node = match self.nodes.get(supervisor_id) {
            Some(n) => n,
            None => return 1,
        };

        let mut max_child_depth = 0;
        for child in &node.children {
            if self.nodes.contains_key(&child.actor_id) {
                // Child is also a supervisor
                let child_depth = self.compute_depth(&child.actor_id);
                max_child_depth = max_child_depth.max(child_depth);
            } else {
                // Leaf node
                max_child_depth = max_child_depth.max(1);
            }
        }

        1 + max_child_depth
    }

    /// Get all actor IDs in reverse start order (leaves first, root last).
    /// Used for graceful shutdown.
    pub fn shutdown_order(&self) -> Vec<AgentId> {
        let mut result = Vec::new();

        // Find root supervisors (no parent)
        let roots: Vec<AgentId> = self
            .nodes
            .values()
            .filter(|n| n.parent_id.is_none())
            .map(|n| n.id)
            .collect();

        for root_id in roots {
            self.collect_shutdown_order(&root_id, &mut result);
        }

        result
    }

    fn collect_shutdown_order(&self, supervisor_id: &AgentId, result: &mut Vec<AgentId>) {
        if let Some(node) = self.nodes.get(supervisor_id) {
            // Recurse into children first (leaves before parents)
            for child in &node.children {
                if self.nodes.contains_key(&child.actor_id) {
                    // Child is a supervisor — recurse
                    self.collect_shutdown_order(&child.actor_id, result);
                } else {
                    // Leaf actor
                    result.push(child.actor_id);
                }
            }
            // Then add the supervisor itself
            result.push(*supervisor_id);
        }
    }

    /// Returns the number of supervisor nodes.
    pub fn supervisor_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a status summary of the tree.
    pub fn query_status(&self) -> TreeStatus {
        TreeStatus {
            total_nodes: self.total_nodes(),
            supervisor_count: self.supervisor_count(),
            tree_depth: self.tree_depth(),
            total_restarts: self
                .nodes
                .values()
                .map(|n| n.restart_history.len())
                .sum::<usize>() as u64,
        }
    }
}

impl Default for SupervisionTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary status of the supervision tree.
#[derive(Debug, Clone)]
pub struct TreeStatus {
    /// Total number of nodes (supervisors + children).
    pub total_nodes: usize,
    /// Number of supervisor nodes.
    pub supervisor_count: usize,
    /// Maximum depth of the tree.
    pub tree_depth: usize,
    /// Total restarts across all supervisors.
    pub total_restarts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::TerminationType;
    use mister_smith_core::RestartPolicy;

    fn default_strategy(policy: RestartPolicy) -> SupervisionStrategy {
        SupervisionStrategy {
            restart_policy: policy,
            ..Default::default()
        }
    }

    // T053: SupervisionTree management tests

    #[test]
    fn add_supervisor_and_children() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));

        let child1 = AgentId::new();
        let child2 = AgentId::new();
        tree.add_child(sup_id, child1, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, child2, RestartScope::Transient).unwrap();

        assert_eq!(tree.total_nodes(), 3); // 1 supervisor + 2 children
        assert_eq!(tree.find_supervisor(&child1), Some(sup_id));
        assert_eq!(tree.find_supervisor(&child2), Some(sup_id));
    }

    #[test]
    fn handle_failure_one_for_one() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));

        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        tree.add_child(sup_id, a, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, b, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, c, RestartScope::Permanent).unwrap();

        let decision = tree.handle_failure(b, TerminationType::Error).unwrap();
        match decision {
            SupervisionDecision::Restart(ids) => {
                assert_eq!(ids, vec![b]);
            }
            other => panic!("Expected Restart, got {:?}", other),
        }
    }

    #[test]
    fn handle_failure_one_for_all() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForAll));

        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        tree.add_child(sup_id, a, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, b, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, c, RestartScope::Permanent).unwrap();

        let decision = tree.handle_failure(b, TerminationType::Error).unwrap();
        match decision {
            SupervisionDecision::Restart(ids) => {
                assert_eq!(ids.len(), 3);
                assert!(ids.contains(&a));
                assert!(ids.contains(&b));
                assert!(ids.contains(&c));
            }
            other => panic!("Expected Restart, got {:?}", other),
        }
    }

    #[test]
    fn handle_failure_rest_for_one() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::RestForOne));

        let a = AgentId::new();
        let b = AgentId::new();
        let c = AgentId::new();
        tree.add_child(sup_id, a, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, b, RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, c, RestartScope::Permanent).unwrap();

        let decision = tree.handle_failure(b, TerminationType::Error).unwrap();
        match decision {
            SupervisionDecision::Restart(ids) => {
                assert_eq!(ids.len(), 2);
                assert!(ids.contains(&b));
                assert!(ids.contains(&c));
                assert!(!ids.contains(&a));
            }
            other => panic!("Expected Restart, got {:?}", other),
        }
    }

    #[test]
    fn transient_child_not_restarted_on_normal_exit() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));

        let child = AgentId::new();
        tree.add_child(sup_id, child, RestartScope::Transient).unwrap();

        let decision = tree.handle_failure(child, TerminationType::Normal).unwrap();
        assert_eq!(decision, SupervisionDecision::Stop(child));
    }

    #[test]
    fn temporary_child_never_restarted() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));

        let child = AgentId::new();
        tree.add_child(sup_id, child, RestartScope::Temporary).unwrap();

        let decision = tree.handle_failure(child, TerminationType::Error).unwrap();
        assert_eq!(decision, SupervisionDecision::Stop(child));
    }

    #[test]
    fn budget_exhaustion_escalates() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(
            sup_id,
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 2,
                ..Default::default()
            },
        );

        let child = AgentId::new();
        tree.add_child(sup_id, child, RestartScope::Permanent).unwrap();

        // First two failures succeed
        assert!(matches!(
            tree.handle_failure(child, TerminationType::Error).unwrap(),
            SupervisionDecision::Restart(_)
        ));
        assert!(matches!(
            tree.handle_failure(child, TerminationType::Error).unwrap(),
            SupervisionDecision::Restart(_)
        ));

        // Third exceeds budget
        let decision = tree.handle_failure(child, TerminationType::Error).unwrap();
        assert_eq!(decision, SupervisionDecision::Escalate);
    }

    // T069: Tree status tests

    #[test]
    fn tree_depth_single_level() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));
        tree.add_child(sup_id, AgentId::new(), RestartScope::Permanent).unwrap();

        assert_eq!(tree.tree_depth(), 2); // supervisor + 1 level of children
    }

    #[test]
    fn tree_depth_three_levels() {
        let mut tree = SupervisionTree::new();
        let root = AgentId::new();
        let mid = AgentId::new();

        tree.add_supervisor(root, default_strategy(RestartPolicy::OneForOne));
        tree.add_supervisor_under(mid, root, default_strategy(RestartPolicy::OneForOne))
            .unwrap();
        tree.add_child(mid, AgentId::new(), RestartScope::Permanent).unwrap();

        assert_eq!(tree.tree_depth(), 3);
    }

    // T070: Shutdown order tests

    #[test]
    fn shutdown_order_leaves_first() {
        let mut tree = SupervisionTree::new();
        let root = AgentId::new();
        let mid = AgentId::new();
        let leaf1 = AgentId::new();
        let leaf2 = AgentId::new();

        tree.add_supervisor(root, default_strategy(RestartPolicy::OneForOne));
        tree.add_supervisor_under(mid, root, default_strategy(RestartPolicy::OneForOne))
            .unwrap();
        tree.add_child(mid, leaf1, RestartScope::Permanent).unwrap();
        tree.add_child(mid, leaf2, RestartScope::Permanent).unwrap();

        let order = tree.shutdown_order();
        // Leaves first, then mid supervisor, then root
        assert_eq!(order.len(), 4); // leaf1, leaf2, mid, root
        let leaf1_pos = order.iter().position(|id| *id == leaf1).unwrap();
        let leaf2_pos = order.iter().position(|id| *id == leaf2).unwrap();
        let mid_pos = order.iter().position(|id| *id == mid).unwrap();
        let root_pos = order.iter().position(|id| *id == root).unwrap();

        assert!(leaf1_pos < mid_pos);
        assert!(leaf2_pos < mid_pos);
        assert!(mid_pos < root_pos);
    }

    #[test]
    fn query_status() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));
        tree.add_child(sup_id, AgentId::new(), RestartScope::Permanent).unwrap();
        tree.add_child(sup_id, AgentId::new(), RestartScope::Permanent).unwrap();

        let status = tree.query_status();
        assert_eq!(status.total_nodes, 3);
        assert_eq!(status.supervisor_count, 1);
        assert_eq!(status.tree_depth, 2);
        assert_eq!(status.total_restarts, 0);
    }

    #[test]
    fn remove_child_works() {
        let mut tree = SupervisionTree::new();
        let sup_id = AgentId::new();
        tree.add_supervisor(sup_id, default_strategy(RestartPolicy::OneForOne));

        let child = AgentId::new();
        tree.add_child(sup_id, child, RestartScope::Permanent).unwrap();
        assert_eq!(tree.total_nodes(), 2);

        tree.remove_child(&child);
        assert!(tree.find_supervisor(&child).is_none());
    }
}
