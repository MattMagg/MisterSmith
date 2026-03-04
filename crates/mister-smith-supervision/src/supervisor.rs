//! Supervisor node and child entry types.
//!
//! `SupervisorNode` tracks supervised children and restart history.
//! `ChildEntry` represents a single supervised child actor.

use std::collections::VecDeque;
use std::time::Instant;

use mister_smith_core::{AgentId, RestartScope, SupervisionStrategy};

/// A supervised child actor entry.
#[derive(Debug, Clone)]
pub struct ChildEntry {
    /// The child actor's unique ID.
    pub actor_id: AgentId,
    /// Restart scope controlling restart behavior.
    pub restart_scope: RestartScope,
    /// Order in which this child was started (for RestForOne).
    pub start_order: u64,
    /// Number of times this child has been restarted.
    pub restart_count: u32,
}

impl ChildEntry {
    /// Create a new child entry.
    pub fn new(actor_id: AgentId, restart_scope: RestartScope, start_order: u64) -> Self {
        Self {
            actor_id,
            restart_scope,
            start_order,
            restart_count: 0,
        }
    }
}

/// A supervisor node in the supervision tree.
#[derive(Debug)]
pub struct SupervisorNode {
    /// This supervisor's unique ID.
    pub id: AgentId,
    /// Parent supervisor ID (None for root).
    pub parent_id: Option<AgentId>,
    /// Supervised children, ordered by start_order.
    pub children: Vec<ChildEntry>,
    /// The supervision strategy (restart policy, budget, backoff).
    pub strategy: SupervisionStrategy,
    /// History of recent restart timestamps for budget checking.
    pub restart_history: VecDeque<Instant>,
    /// Monotonically increasing child start order counter.
    next_start_order: u64,
}

impl SupervisorNode {
    /// Create a new supervisor node.
    pub fn new(id: AgentId, strategy: SupervisionStrategy) -> Self {
        Self {
            id,
            parent_id: None,
            children: Vec::new(),
            strategy,
            restart_history: VecDeque::new(),
            next_start_order: 0,
        }
    }

    /// Create a new supervisor node with a parent.
    pub fn with_parent(id: AgentId, parent_id: AgentId, strategy: SupervisionStrategy) -> Self {
        Self {
            id,
            parent_id: Some(parent_id),
            children: Vec::new(),
            strategy,
            restart_history: VecDeque::new(),
            next_start_order: 0,
        }
    }

    /// Add a child to this supervisor.
    pub fn add_child(&mut self, actor_id: AgentId, restart_scope: RestartScope) -> &ChildEntry {
        let order = self.next_start_order;
        self.next_start_order += 1;
        self.children
            .push(ChildEntry::new(actor_id, restart_scope, order));
        self.children.last().unwrap()
    }

    /// Remove a child by actor ID.
    pub fn remove_child(&mut self, actor_id: &AgentId) -> Option<ChildEntry> {
        if let Some(pos) = self.children.iter().position(|c| c.actor_id == *actor_id) {
            Some(self.children.remove(pos))
        } else {
            None
        }
    }

    /// Find a child by actor ID.
    pub fn find_child(&self, actor_id: &AgentId) -> Option<&ChildEntry> {
        self.children.iter().find(|c| c.actor_id == *actor_id)
    }

    /// Find a mutable child by actor ID.
    pub fn find_child_mut(&mut self, actor_id: &AgentId) -> Option<&mut ChildEntry> {
        self.children.iter_mut().find(|c| c.actor_id == *actor_id)
    }

    /// Record a restart in the history.
    pub fn record_restart(&mut self) {
        self.restart_history.push_back(Instant::now());
    }
}

/// Internal notification for supervision events.
#[derive(Debug)]
pub struct SupervisionEvent {
    /// The supervisor that generated this event.
    pub supervisor_id: AgentId,
    /// The type of supervision event.
    pub event_type: SupervisionEventType,
}

/// Types of supervision events.
#[derive(Debug)]
pub enum SupervisionEventType {
    /// A child was restarted.
    ChildRestarted {
        /// The restarted child's ID.
        child_id: AgentId,
        /// Number of restarts for this child.
        restart_count: u32,
    },
    /// A failure was escalated to the parent.
    FailureEscalated {
        /// The failed child's ID.
        child_id: AgentId,
        /// Error description.
        error: String,
    },
    /// Restart budget was exhausted.
    BudgetExhausted {
        /// Total restarts in the window.
        restarts_in_window: usize,
        /// Max allowed.
        max_failures: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn child_entry_creation() {
        let id = AgentId::new();
        let child = ChildEntry::new(id, RestartScope::Permanent, 0);
        assert_eq!(child.actor_id, id);
        assert_eq!(child.restart_scope, RestartScope::Permanent);
        assert_eq!(child.start_order, 0);
        assert_eq!(child.restart_count, 0);
    }

    #[test]
    fn supervisor_node_add_remove_children() {
        let sup_id = AgentId::new();
        let mut node = SupervisorNode::new(sup_id, SupervisionStrategy::default());

        let child1 = AgentId::new();
        let child2 = AgentId::new();
        let child3 = AgentId::new();

        node.add_child(child1, RestartScope::Permanent);
        node.add_child(child2, RestartScope::Transient);
        node.add_child(child3, RestartScope::Temporary);

        assert_eq!(node.children.len(), 3);
        assert_eq!(node.children[0].start_order, 0);
        assert_eq!(node.children[1].start_order, 1);
        assert_eq!(node.children[2].start_order, 2);

        // Remove middle child
        let removed = node.remove_child(&child2).unwrap();
        assert_eq!(removed.actor_id, child2);
        assert_eq!(node.children.len(), 2);

        // Remove non-existent
        assert!(node.remove_child(&AgentId::new()).is_none());
    }

    #[test]
    fn supervisor_node_find_child() {
        let sup_id = AgentId::new();
        let mut node = SupervisorNode::new(sup_id, SupervisionStrategy::default());

        let child_id = AgentId::new();
        node.add_child(child_id, RestartScope::Permanent);

        assert!(node.find_child(&child_id).is_some());
        assert!(node.find_child(&AgentId::new()).is_none());
    }

    #[test]
    fn supervisor_node_with_parent() {
        let sup_id = AgentId::new();
        let parent_id = AgentId::new();
        let node =
            SupervisorNode::with_parent(sup_id, parent_id, SupervisionStrategy::default());
        assert_eq!(node.parent_id, Some(parent_id));
    }

    #[test]
    fn restart_history_recording() {
        let sup_id = AgentId::new();
        let mut node = SupervisorNode::new(sup_id, SupervisionStrategy::default());
        assert!(node.restart_history.is_empty());

        node.record_restart();
        node.record_restart();
        assert_eq!(node.restart_history.len(), 2);
    }
}
