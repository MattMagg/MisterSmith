//! Deterministic topology compilation for Phase 10 execution graphs.

use std::collections::{HashMap, HashSet};

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use mister_smith_core::{
    AgentType, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope, CheckpointPolicy,
    ContextBudget, CoordinationPolicy, DelegationScope, ExecutionBranchId, ExecutionNodeId,
    HealthState, NodeState, TaskId, TopologyError, TopologyKind, TopologyPlan, TopologyRationale,
};

use crate::execution_graph::{ExecutionBranch, ExecutionEdge, ExecutionGraph, ExecutionNode};

/// Operational hints used when selecting a topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologySignals {
    /// Current health posture of the execution environment.
    pub health_state: Option<HealthState>,
    /// Coarse context or budget pressure from 0 to 100.
    pub budget_pressure: Option<u8>,
    /// Whether streaming handoff is preferred for strict chains.
    pub prefer_streaming: bool,
    /// Maximum parallel width allowed by policy.
    pub max_parallelism: Option<usize>,
    /// Whether the system should stay in a conservative execution posture.
    pub conservative_mode: bool,
}

impl Default for TopologySignals {
    fn default() -> Self {
        Self {
            health_state: Some(HealthState::Healthy),
            budget_pressure: None,
            prefer_streaming: false,
            max_parallelism: None,
            conservative_mode: false,
        }
    }
}

/// Deterministic execution-graph compiler and topology selector.
#[derive(Debug, Clone, Default)]
pub struct TopologyCompiler;

#[derive(Debug, Clone)]
struct PlannerStepSpec {
    key: String,
    numeric_step: Option<u64>,
    node_id: ExecutionNodeId,
    action: String,
    description: String,
    role: AgentType,
    branch_label: Option<String>,
    dependency_keys: Vec<String>,
    budget: ContextBudget,
    delegation_requirement: Option<DelegationScope>,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct GraphAnalysis {
    root_count: usize,
    max_parallel_width: usize,
    max_depth: usize,
    has_join: bool,
    has_fanout: bool,
    is_chain: bool,
}

impl TopologyCompiler {
    /// Compile planner output into a validated execution graph with an attached topology plan.
    pub fn compile(
        &self,
        workflow_id: TaskId,
        planner_output: &Value,
        signals: &TopologySignals,
    ) -> Result<ExecutionGraph, TopologyError> {
        let hint = parse_topology_hint(planner_output.get("topology_hint"))?;
        let steps = parse_steps(planner_output)?;
        let (branches, nodes, edges) = build_graph_components(workflow_id, steps)?;

        let placeholder_plan = TopologyPlan {
            topology_kind: TopologyKind::Sequential,
            parallelism_width: 1,
            rationale: TopologyRationale {
                dependency_shape: "unclassified".to_string(),
                operational_signals: vec!["health:unknown".to_string()],
                selected_for: "placeholder validation plan".to_string(),
                fallback_reason: None,
            },
            coordination_policy: CoordinationPolicy::StrictSequence,
            fallback_topology: None,
        };

        let mut graph = ExecutionGraph::new(workflow_id, branches, nodes, edges, placeholder_plan);
        graph.validate()?;
        graph.topology_plan = self.select_topology_with_hint(&graph, signals, hint)?;
        graph.validate()?;

        Ok(graph)
    }

    /// Validate an existing execution graph.
    pub fn validate(&self, graph: &ExecutionGraph) -> Result<(), TopologyError> {
        graph.validate()
    }

    /// Select a deterministic topology for an existing validated execution graph.
    pub fn select_topology(
        &self,
        graph: &ExecutionGraph,
        signals: &TopologySignals,
    ) -> Result<TopologyPlan, TopologyError> {
        self.select_topology_with_hint(graph, signals, None)
    }

    fn select_topology_with_hint(
        &self,
        graph: &ExecutionGraph,
        signals: &TopologySignals,
        hint: Option<TopologyKind>,
    ) -> Result<TopologyPlan, TopologyError> {
        graph.validate()?;

        let analysis = analyze_graph(graph)?;
        let dependency_shape = describe_dependency_shape(&analysis);
        let operational_signals = describe_operational_signals(signals);

        if let Some(hint) = hint {
            if hint_is_compatible(hint, &analysis, graph.nodes.len()) {
                return Ok(build_plan(
                    hint,
                    &analysis,
                    signals,
                    &dependency_shape,
                    &operational_signals,
                    &format!(
                        "selected {} topology from compatible planner policy hint",
                        topology_name(hint)
                    ),
                ));
            }
        }

        let selected = if signals.conservative_mode
            || matches!(
                signals.health_state,
                Some(HealthState::Unhealthy) | Some(HealthState::Unknown)
            )
            || signals.budget_pressure.unwrap_or(0) >= 90
        {
            if analysis.is_chain && signals.prefer_streaming {
                TopologyKind::Pipeline
            } else {
                TopologyKind::Sequential
            }
        } else if analysis.is_chain {
            if signals.prefer_streaming && graph.nodes.len() > 1 {
                TopologyKind::Pipeline
            } else {
                TopologyKind::Sequential
            }
        } else if analysis.has_join && analysis.max_parallel_width > 1 {
            TopologyKind::Hybrid
        } else if analysis.has_fanout && analysis.max_depth >= 3 && analysis.root_count == 1 {
            TopologyKind::Hierarchical
        } else if analysis.max_parallel_width > 1 {
            TopologyKind::Parallel
        } else {
            TopologyKind::Sequential
        };

        Ok(build_plan(
            selected,
            &analysis,
            signals,
            &dependency_shape,
            &operational_signals,
            &format!(
                "selected {} topology from dependency analysis",
                topology_name(selected)
            ),
        ))
    }
}

type GraphComponents = (Vec<ExecutionBranch>, Vec<ExecutionNode>, Vec<ExecutionEdge>);

fn build_graph_components(
    workflow_id: TaskId,
    steps: Vec<PlannerStepSpec>,
) -> Result<GraphComponents, TopologyError> {
    let graph_id = mister_smith_core::ExecutionGraphId::from_uuid(*workflow_id.as_ref());

    let any_branch_labels = steps.iter().any(|step| step.branch_label.is_some());
    let default_branch_id = ExecutionBranchId::new();
    let mut branch_ids = HashMap::new();
    if any_branch_labels {
        for label in steps.iter().filter_map(|step| step.branch_label.clone()) {
            branch_ids
                .entry(label)
                .or_insert_with(ExecutionBranchId::new);
        }
    }

    let mut reference_lookup = HashMap::new();
    for step in &steps {
        reference_lookup.insert(step.key.clone(), step.node_id);
        if let Some(number) = step.numeric_step {
            reference_lookup.insert(number.to_string(), step.node_id);
        }
    }

    let mut nodes = Vec::with_capacity(steps.len());
    let mut edges = Vec::new();
    let mut branch_members: HashMap<ExecutionBranchId, Vec<ExecutionNodeId>> = HashMap::new();

    for step in steps {
        let branch_id = step
            .branch_label
            .as_ref()
            .and_then(|label| branch_ids.get(label))
            .copied()
            .unwrap_or(default_branch_id);
        let mut dependencies = Vec::with_capacity(step.dependency_keys.len());

        for dependency_key in &step.dependency_keys {
            let dependency = reference_lookup
                .get(dependency_key)
                .copied()
                .ok_or_else(|| TopologyError::MissingDependency {
                    node_id: Some(step.node_id),
                    dependency: dependency_id_for_error(dependency_key),
                })?;
            dependencies.push(dependency);
            edges.push(ExecutionEdge::completion(dependency, step.node_id));
        }

        branch_members
            .entry(branch_id)
            .or_default()
            .push(step.node_id);
        nodes.push(ExecutionNode {
            node_id: step.node_id,
            step_key: step.key,
            role: step.role,
            branch_id,
            dependencies,
            state: NodeState::Pending,
            budget: step.budget,
            delegation_requirement: step.delegation_requirement,
            action: step.action,
            description: step.description,
            metadata: step.metadata,
        });
    }

    if !any_branch_labels {
        branch_members.entry(default_branch_id).or_default();
    }

    let branches = branch_members
        .into_iter()
        .map(|(branch_id, node_ids)| ExecutionBranch {
            branch_id,
            graph_id,
            node_ids,
            state: BranchState::Pending,
            checkpoint_policy: CheckpointPolicy::OnNodeCompletion,
            assigned_agents: Vec::new(),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        })
        .collect();

    Ok((branches, nodes, edges))
}

fn parse_steps(planner_output: &Value) -> Result<Vec<PlannerStepSpec>, TopologyError> {
    let steps = planner_output
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TopologyError::Invalid("planner output must include a steps array".into())
        })?;
    if steps.is_empty() {
        return Err(TopologyError::Invalid(
            "planner output must include at least one step".into(),
        ));
    }

    let mut specs = Vec::with_capacity(steps.len());
    let mut seen_keys = HashSet::new();
    let mut seen_numeric_steps = HashSet::new();
    for (index, raw_step) in steps.iter().enumerate() {
        let object = raw_step.as_object().ok_or_else(|| {
            TopologyError::Invalid(format!("planner step {} must be an object", index + 1))
        })?;

        let numeric_step = object.get("step").and_then(Value::as_u64);
        if let Some(step_number) = numeric_step {
            if !seen_numeric_steps.insert(step_number) {
                return Err(TopologyError::Invalid(format!(
                    "planner output contains duplicate numeric step reference '{}'",
                    step_number
                )));
            }
        }
        let key = object
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| numeric_step.map(|step| step.to_string()))
            .unwrap_or_else(|| format!("step-{}", index + 1));
        if !seen_keys.insert(key.clone()) {
            return Err(TopologyError::Invalid(format!(
                "planner output contains duplicate step key '{}'",
                key
            )));
        }

        let node_id = if let Ok(uuid) = Uuid::parse_str(&key) {
            ExecutionNodeId::from_uuid(uuid)
        } else {
            ExecutionNodeId::new()
        };
        let action = object
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("execute")
            .to_string();
        let description = object
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or(action.as_str())
            .to_string();
        let role = parse_agent_type(object.get("role"))?;
        let branch_label = object
            .get("branch")
            .and_then(Value::as_str)
            .map(str::to_string);
        let dependency_keys = object
            .get("depends_on")
            .and_then(Value::as_array)
            .map(|deps| {
                deps.iter()
                    .filter_map(|value| {
                        value
                            .as_str()
                            .map(str::to_string)
                            .or_else(|| value.as_u64().map(|num| num.to_string()))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let budget = parse_budget(object.get("budget"))?;
        let delegation_requirement = parse_delegation_scope(object.get("delegation_requirement"))?;
        let metadata = object
            .get("input")
            .cloned()
            .unwrap_or_else(|| json!({ "planner_index": index }));

        specs.push(PlannerStepSpec {
            key,
            numeric_step,
            node_id,
            action,
            description,
            role,
            branch_label,
            dependency_keys,
            budget,
            delegation_requirement,
            metadata,
        });
    }

    Ok(specs)
}

fn parse_budget(raw_budget: Option<&Value>) -> Result<ContextBudget, TopologyError> {
    let max_units =
        match raw_budget {
            Some(Value::Number(number)) => number.as_u64().ok_or_else(|| {
                TopologyError::Invalid("budget value must be a positive integer".into())
            })?,
            Some(Value::Object(object)) => object
                .get("max_units")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    TopologyError::Invalid("budget object must include numeric max_units".into())
                })?,
            Some(_) => {
                return Err(TopologyError::Invalid(
                    "budget must be an integer or object".into(),
                ))
            }
            None => 2048,
        };

    Ok(ContextBudget {
        budget_id: mister_smith_core::ContextBudgetId::new(),
        scope: BudgetScope::Node,
        max_units,
        reserved_units: 0,
        policy: BudgetPolicy::Summarize,
    })
}

fn parse_topology_hint(raw_hint: Option<&Value>) -> Result<Option<TopologyKind>, TopologyError> {
    raw_hint
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| TopologyError::Invalid("topology_hint must be a string".into()))
                .and_then(parse_topology_kind)
        })
        .transpose()
}

fn parse_topology_kind(value: &str) -> Result<TopologyKind, TopologyError> {
    match value.to_ascii_lowercase().as_str() {
        "sequential" => Ok(TopologyKind::Sequential),
        "parallel" => Ok(TopologyKind::Parallel),
        "pipeline" => Ok(TopologyKind::Pipeline),
        "hierarchical" => Ok(TopologyKind::Hierarchical),
        "hybrid" => Ok(TopologyKind::Hybrid),
        other => Err(TopologyError::Unsupported(format!(
            "unsupported topology hint '{other}'"
        ))),
    }
}

fn parse_agent_type(raw_role: Option<&Value>) -> Result<AgentType, TopologyError> {
    let Some(raw_role) = raw_role else {
        return Ok(AgentType::Executor);
    };
    let role = raw_role
        .as_str()
        .ok_or_else(|| TopologyError::Invalid("role must be a string".into()))?;
    match role.to_ascii_lowercase().as_str() {
        "supervisor" => Ok(AgentType::Supervisor),
        "worker" => Ok(AgentType::Worker),
        "coordinator" => Ok(AgentType::Coordinator),
        "monitor" => Ok(AgentType::Monitor),
        "planner" => Ok(AgentType::Planner),
        "executor" => Ok(AgentType::Executor),
        "critic" => Ok(AgentType::Critic),
        "router" => Ok(AgentType::Router),
        "memory" => Ok(AgentType::Memory),
        other => Err(TopologyError::Unsupported(format!(
            "unsupported planner role '{other}'"
        ))),
    }
}

fn parse_delegation_scope(
    raw_scope: Option<&Value>,
) -> Result<Option<DelegationScope>, TopologyError> {
    let Some(raw_scope) = raw_scope else {
        return Ok(None);
    };
    let scope = raw_scope
        .as_str()
        .ok_or_else(|| TopologyError::Invalid("delegation_requirement must be a string".into()))?;
    let scope = match scope.to_ascii_lowercase().as_str() {
        "executeworkflow" | "execute_workflow" => DelegationScope::ExecuteWorkflow,
        "managebranch" | "manage_branch" => DelegationScope::ManageBranch,
        "refreshcontext" | "refresh_context" => DelegationScope::RefreshContext,
        "applyintervention" | "apply_intervention" => DelegationScope::ApplyIntervention,
        "accessmemory" | "access_memory" => DelegationScope::AccessMemory,
        "invoketool" | "invoke_tool" => DelegationScope::InvokeTool,
        other => {
            return Err(TopologyError::Unsupported(format!(
                "unsupported delegation scope '{other}'"
            )))
        }
    };
    Ok(Some(scope))
}

fn analyze_graph(graph: &ExecutionGraph) -> Result<GraphAnalysis, TopologyError> {
    let mut petgraph = DiGraph::<ExecutionNodeId, ()>::new();
    let mut indices = HashMap::new();
    let mut indegree = HashMap::new();
    let mut outdegree = HashMap::new();

    for node in &graph.nodes {
        indices.insert(node.node_id, petgraph.add_node(node.node_id));
        indegree.insert(node.node_id, 0usize);
        outdegree.insert(node.node_id, 0usize);
    }
    for edge in &graph.edges {
        petgraph.add_edge(indices[&edge.from], indices[&edge.to], ());
        *indegree.entry(edge.to).or_default() += 1;
        *outdegree.entry(edge.from).or_default() += 1;
    }

    let ordered = toposort(&petgraph, None).map_err(|cycle| TopologyError::CycleDetected {
        graph_id: Some(graph.graph_id),
        message: format!("cycle detected around node {}", petgraph[cycle.node_id()]),
    })?;

    let mut depths = HashMap::new();
    let mut max_depth = 0usize;
    for index in ordered {
        let node_id = petgraph[index];
        let depth = graph
            .nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .map(|node| {
                node.dependencies
                    .iter()
                    .map(|dep| depths.get(dep).copied().unwrap_or(0) + 1)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        depths.insert(node_id, depth);
        max_depth = max_depth.max(depth);
    }

    let mut widths = HashMap::new();
    for depth in depths.values() {
        *widths.entry(*depth).or_insert(0usize) += 1;
    }
    let max_parallel_width = widths.values().copied().max().unwrap_or(1);

    Ok(GraphAnalysis {
        root_count: indegree.values().filter(|count| **count == 0).count(),
        max_parallel_width,
        max_depth,
        has_join: indegree.values().any(|count| *count > 1),
        has_fanout: outdegree.values().any(|count| *count > 1),
        is_chain: graph.nodes.iter().all(|node| node.dependencies.len() <= 1)
            && indegree.values().filter(|count| **count == 0).count() <= 1
            && outdegree.values().all(|count| *count <= 1),
    })
}

fn describe_dependency_shape(analysis: &GraphAnalysis) -> String {
    if analysis.is_chain {
        "strict-chain".to_string()
    } else if analysis.has_join {
        "fanout-join".to_string()
    } else if analysis.has_fanout && analysis.max_depth >= 3 {
        "hierarchical-fanout".to_string()
    } else if analysis.max_parallel_width > 1 {
        "parallel-fanout".to_string()
    } else {
        "mixed-graph".to_string()
    }
}

fn describe_operational_signals(signals: &TopologySignals) -> Vec<String> {
    let mut descriptions = Vec::new();
    descriptions.push(format!(
        "health:{}",
        match signals.health_state.unwrap_or(HealthState::Unknown) {
            HealthState::Healthy => "healthy",
            HealthState::Degraded => "degraded",
            HealthState::Unhealthy => "unhealthy",
            HealthState::Unknown => "unknown",
        }
    ));
    if let Some(pressure) = signals.budget_pressure {
        descriptions.push(format!("budget_pressure:{pressure}"));
    } else {
        descriptions.push("budget_pressure:normal".to_string());
    }
    if signals.prefer_streaming {
        descriptions.push("streaming:preferred".to_string());
    }
    if let Some(limit) = signals.max_parallelism {
        descriptions.push(format!("parallelism_cap:{limit}"));
    }
    if signals.conservative_mode {
        descriptions.push("mode:conservative".to_string());
    }
    descriptions
}

fn build_plan(
    topology_kind: TopologyKind,
    analysis: &GraphAnalysis,
    signals: &TopologySignals,
    dependency_shape: &str,
    operational_signals: &[String],
    selected_for: &str,
) -> TopologyPlan {
    let mut parallelism_width = match topology_kind {
        TopologyKind::Sequential => 1,
        _ => analysis.max_parallel_width.max(1),
    };
    if let Some(limit) = signals.max_parallelism {
        parallelism_width = parallelism_width.min(limit.max(1));
    }
    let coordination_policy = match topology_kind {
        TopologyKind::Sequential => CoordinationPolicy::StrictSequence,
        TopologyKind::Parallel => CoordinationPolicy::Barrier,
        TopologyKind::Pipeline => CoordinationPolicy::Streaming,
        TopologyKind::Hierarchical => CoordinationPolicy::HierarchicalReduce,
        TopologyKind::Hybrid => CoordinationPolicy::Mixed,
    };
    let fallback_topology = match topology_kind {
        TopologyKind::Sequential => None,
        _ => Some(TopologyKind::Sequential),
    };
    let fallback_reason = fallback_topology.map(|_| {
        "fallback to sequential preserves dependency order under degraded signals".to_string()
    });

    TopologyPlan {
        topology_kind,
        parallelism_width,
        rationale: TopologyRationale {
            dependency_shape: dependency_shape.to_string(),
            operational_signals: operational_signals.to_vec(),
            selected_for: selected_for.to_string(),
            fallback_reason,
        },
        coordination_policy,
        fallback_topology,
    }
}

fn hint_is_compatible(hint: TopologyKind, analysis: &GraphAnalysis, node_count: usize) -> bool {
    match hint {
        TopologyKind::Sequential => analysis.is_chain,
        TopologyKind::Parallel => analysis.max_parallel_width > 1 && !analysis.has_join,
        TopologyKind::Pipeline => analysis.is_chain && node_count > 1,
        TopologyKind::Hierarchical => analysis.has_fanout,
        TopologyKind::Hybrid => analysis.has_join && analysis.max_parallel_width > 1,
    }
}

fn dependency_id_for_error(dependency_key: &str) -> ExecutionNodeId {
    Uuid::parse_str(dependency_key)
        .map(ExecutionNodeId::from_uuid)
        .unwrap_or_default()
}

fn topology_name(topology_kind: TopologyKind) -> &'static str {
    match topology_kind {
        TopologyKind::Sequential => "sequential",
        TopologyKind::Parallel => "parallel",
        TopologyKind::Pipeline => "pipeline",
        TopologyKind::Hierarchical => "hierarchical",
        TopologyKind::Hybrid => "hybrid",
    }
}
