//! Execution graph types and validation for Phase 10 orchestration.

use std::collections::{HashMap, HashSet};

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mister_smith_core::{
    AgentId, AgentType, BranchRecoveryStrategy, BranchState, CheckpointId, CheckpointPolicy,
    ContextBudget, DelegationScope, DependencyType, ExecutionBranchId, ExecutionGraphId,
    ExecutionNodeId, GraphState, NodeState, TaskId, TopologyError, TopologyPlan,
};

/// Checkpoint lineage entry for a graph branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchCheckpoint {
    /// Stable checkpoint identifier.
    pub checkpoint_id: CheckpointId,
    /// Branch that owns this checkpoint.
    pub branch_id: ExecutionBranchId,
}

/// Checkpointable unit of work within an execution graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBranch {
    /// Stable branch identifier.
    pub branch_id: ExecutionBranchId,
    /// Graph that owns this branch.
    pub graph_id: ExecutionGraphId,
    /// Nodes assigned to this branch.
    pub node_ids: Vec<ExecutionNodeId>,
    /// Current branch lifecycle state.
    pub state: BranchState,
    /// Checkpoint policy for this branch.
    pub checkpoint_policy: CheckpointPolicy,
    /// Agents currently assigned to the branch.
    pub assigned_agents: Vec<AgentId>,
    /// Recovery strategy for this branch.
    pub recovery_strategy: BranchRecoveryStrategy,
}

impl ExecutionBranch {
    /// Create a new execution branch with default lifecycle settings.
    pub fn new(
        graph_id: ExecutionGraphId,
        branch_id: ExecutionBranchId,
        node_ids: Vec<ExecutionNodeId>,
    ) -> Self {
        Self {
            branch_id,
            graph_id,
            node_ids,
            state: BranchState::Pending,
            checkpoint_policy: CheckpointPolicy::OnNodeCompletion,
            assigned_agents: Vec::new(),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }
    }
}

/// Executable unit inside an execution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionNode {
    /// Stable execution node identifier.
    pub node_id: ExecutionNodeId,
    /// Planner-visible key for this step.
    pub step_key: String,
    /// Requested execution role.
    pub role: AgentType,
    /// Branch the node belongs to.
    pub branch_id: ExecutionBranchId,
    /// Direct upstream dependencies.
    pub dependencies: Vec<ExecutionNodeId>,
    /// Current node state.
    pub state: NodeState,
    /// Context budget available to this node.
    pub budget: ContextBudget,
    /// Required delegation scope, when privileged.
    pub delegation_requirement: Option<DelegationScope>,
    /// Planner action or task label.
    pub action: String,
    /// Human-readable step description.
    pub description: String,
    /// Additional planner metadata carried forward for dispatch.
    pub metadata: Value,
}

/// Directed dependency between execution nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEdge {
    /// Upstream node identifier.
    pub from: ExecutionNodeId,
    /// Downstream node identifier.
    pub to: ExecutionNodeId,
    /// Semantics of the dependency.
    pub edge_type: DependencyType,
}

impl ExecutionEdge {
    /// Construct a completion dependency edge.
    pub fn completion(from: ExecutionNodeId, to: ExecutionNodeId) -> Self {
        Self {
            from,
            to,
            edge_type: DependencyType::Completion,
        }
    }
}

/// Canonical graph representation used by the Phase 10 execution control plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionGraph {
    /// Stable graph identifier.
    pub graph_id: ExecutionGraphId,
    /// Workflow or task that owns this graph.
    pub workflow_id: TaskId,
    /// Checkpointable execution branches.
    pub branches: Vec<ExecutionBranch>,
    /// Executable nodes in the graph.
    pub nodes: Vec<ExecutionNode>,
    /// Directed dependencies between nodes.
    pub edges: Vec<ExecutionEdge>,
    /// Selected execution topology and rationale.
    pub topology_plan: TopologyPlan,
    /// Current graph lifecycle state.
    pub state: GraphState,
    /// Optional checkpoint lineage for branch-local recovery.
    pub checkpoint_lineage: Vec<BranchCheckpoint>,
}

impl ExecutionGraph {
    /// Construct a new execution graph with pending lifecycle state.
    pub fn new(
        workflow_id: TaskId,
        branches: Vec<ExecutionBranch>,
        nodes: Vec<ExecutionNode>,
        edges: Vec<ExecutionEdge>,
        topology_plan: TopologyPlan,
    ) -> Self {
        Self {
            graph_id: ExecutionGraphId::from_uuid(*workflow_id.as_ref()),
            workflow_id,
            branches,
            nodes,
            edges,
            topology_plan,
            state: GraphState::Pending,
            checkpoint_lineage: Vec::new(),
        }
    }

    /// Validate graph structure, dependencies, branches, and topology metadata.
    pub fn validate(&self) -> Result<(), TopologyError> {
        if self.nodes.is_empty() {
            return Err(TopologyError::Invalid(
                "execution graph must contain at least one node".to_string(),
            ));
        }
        if self.branches.is_empty() {
            return Err(TopologyError::Invalid(
                "execution graph must contain at least one branch".to_string(),
            ));
        }
        if self.topology_plan.parallelism_width == 0 {
            return Err(TopologyError::Invalid(
                "topology parallelism width must be at least 1".to_string(),
            ));
        }
        if self.topology_plan.topology_kind == mister_smith_core::TopologyKind::Hybrid
            && self.topology_plan.coordination_policy
                != mister_smith_core::CoordinationPolicy::Mixed
        {
            return Err(TopologyError::Invalid(
                "hybrid topology must use mixed coordination policy".to_string(),
            ));
        }

        let node_lookup: HashMap<ExecutionNodeId, &ExecutionNode> =
            self.nodes.iter().map(|node| (node.node_id, node)).collect();
        if node_lookup.len() != self.nodes.len() {
            return Err(TopologyError::Invalid(
                "execution graph contains duplicate node identifiers".to_string(),
            ));
        }

        let branch_lookup: HashMap<ExecutionBranchId, &ExecutionBranch> = self
            .branches
            .iter()
            .map(|branch| (branch.branch_id, branch))
            .collect();
        if branch_lookup.len() != self.branches.len() {
            return Err(TopologyError::Invalid(
                "execution graph contains duplicate branch identifiers".to_string(),
            ));
        }

        let mut branch_membership = HashMap::new();
        for branch in &self.branches {
            if branch.node_ids.is_empty() {
                return Err(TopologyError::Invalid(format!(
                    "branch {} must contain at least one node",
                    branch.branch_id
                )));
            }
            for node_id in &branch.node_ids {
                if !node_lookup.contains_key(node_id) {
                    return Err(TopologyError::Invalid(format!(
                        "branch {} references unknown node {}",
                        branch.branch_id, node_id
                    )));
                }
                if let Some(existing_branch_id) =
                    branch_membership.insert(*node_id, branch.branch_id)
                {
                    return Err(TopologyError::Invalid(format!(
                        "node {} is assigned to multiple branches: {} and {}",
                        node_id, existing_branch_id, branch.branch_id
                    )));
                }
            }
        }

        for node in &self.nodes {
            if !branch_lookup.contains_key(&node.branch_id) {
                return Err(TopologyError::Invalid(format!(
                    "node {} references unknown branch {}",
                    node.node_id, node.branch_id
                )));
            }
            if !branch_lookup[&node.branch_id]
                .node_ids
                .contains(&node.node_id)
            {
                return Err(TopologyError::Invalid(format!(
                    "node {} is missing from branch {} membership",
                    node.node_id, node.branch_id
                )));
            }
            if !branch_membership.contains_key(&node.node_id) {
                return Err(TopologyError::Invalid(format!(
                    "node {} is not assigned to any branch",
                    node.node_id
                )));
            }
            for dependency in &node.dependencies {
                if !node_lookup.contains_key(dependency) {
                    return Err(TopologyError::MissingDependency {
                        node_id: Some(node.node_id),
                        dependency: *dependency,
                    });
                }
            }
        }

        let edge_lookup: HashSet<(ExecutionNodeId, ExecutionNodeId)> =
            self.edges.iter().map(|edge| (edge.from, edge.to)).collect();
        for edge in &self.edges {
            if !node_lookup.contains_key(&edge.from) || !node_lookup.contains_key(&edge.to) {
                return Err(TopologyError::Invalid(format!(
                    "edge {} -> {} references unknown node",
                    edge.from, edge.to
                )));
            }
        }
        for node in &self.nodes {
            for dependency in &node.dependencies {
                if !edge_lookup.contains(&(*dependency, node.node_id)) {
                    return Err(TopologyError::Invalid(format!(
                        "node {} dependency {} is missing a matching edge",
                        node.node_id, dependency
                    )));
                }
            }
        }

        let mut graph = DiGraph::<ExecutionNodeId, ()>::new();
        let mut indices = HashMap::new();
        for node in &self.nodes {
            indices.insert(node.node_id, graph.add_node(node.node_id));
        }
        for edge in &self.edges {
            let from = indices[&edge.from];
            let to = indices[&edge.to];
            graph.add_edge(from, to, ());
        }

        if let Err(cycle) = toposort(&graph, None) {
            let node_id = graph[cycle.node_id()];
            return Err(TopologyError::CycleDetected {
                graph_id: Some(self.graph_id),
                message: format!("cycle detected around node {node_id}"),
            });
        }

        Ok(())
    }

    /// Return the nodes that have no upstream dependencies.
    pub fn root_nodes(&self) -> Vec<&ExecutionNode> {
        self.nodes
            .iter()
            .filter(|node| node.dependencies.is_empty())
            .collect()
    }

    /// Borrow a branch by identifier.
    pub fn branch(&self, branch_id: &ExecutionBranchId) -> Option<&ExecutionBranch> {
        self.branches
            .iter()
            .find(|branch| branch.branch_id == *branch_id)
    }

    /// Mutably borrow a branch by identifier.
    pub fn branch_mut(&mut self, branch_id: &ExecutionBranchId) -> Option<&mut ExecutionBranch> {
        self.branches
            .iter_mut()
            .find(|branch| branch.branch_id == *branch_id)
    }
}
