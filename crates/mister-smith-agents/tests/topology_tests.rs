use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{ProfileAssessment, TopologyCompiler, TopologySignals};
use mister_smith_core::{
    AgentId, BudgetPolicy, BudgetScope, ContextBudget, ContextBudgetId, CoordinationPolicy,
    GuardTarget, HealthState, NodeState, SemanticSignal, SemanticSignalKind, TaskId, TopologyKind,
};
use mister_smith_events::AutonomyEvent;
use serde_json::json;
use uuid::Uuid;

fn independent_branches_plan() -> serde_json::Value {
    json!({
        "goal": "independent-branches",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root-step",
                "description": "Root step"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left-branch",
                "description": "Left branch",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right",
                "step": 3,
                "action": "right-branch",
                "description": "Right branch",
                "depends_on": ["root"],
                "branch": "right"
            }
        ]
    })
}

fn strict_chain_plan() -> serde_json::Value {
    json!({
        "goal": "strict-chain",
        "steps": [
            {
                "id": "step-1",
                "step": 1,
                "action": "first",
                "description": "First"
            },
            {
                "id": "step-2",
                "step": 2,
                "action": "second",
                "description": "Second",
                "depends_on": ["step-1"]
            },
            {
                "id": "step-3",
                "step": 3,
                "action": "third",
                "description": "Third",
                "depends_on": ["step-2"]
            }
        ]
    })
}

fn join_plan() -> serde_json::Value {
    json!({
        "goal": "join-workflow",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "Root"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left",
                "description": "Left",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right",
                "step": 3,
                "action": "right",
                "description": "Right",
                "depends_on": ["root"],
                "branch": "right"
            },
            {
                "id": "join",
                "step": 4,
                "action": "join",
                "description": "Join",
                "depends_on": ["left", "right"]
            }
        ]
    })
}

fn hierarchical_hint_plan() -> serde_json::Value {
    json!({
        "goal": "hierarchical-review",
        "topology_hint": "hierarchical",
        "steps": [
            {
                "id": "coordinate",
                "step": 1,
                "action": "coordinate",
                "description": "Coordinate subtree execution"
            },
            {
                "id": "review-a",
                "step": 2,
                "action": "review-a",
                "description": "Review branch A",
                "depends_on": ["coordinate"],
                "branch": "subtree-a"
            },
            {
                "id": "review-b",
                "step": 3,
                "action": "review-b",
                "description": "Review branch B",
                "depends_on": ["coordinate"],
                "branch": "subtree-b"
            }
        ]
    })
}

fn submit_task(
    scheduler: &TaskScheduler,
    task_id: TaskId,
    task_type: &str,
    parent_task_id: TaskId,
    state: TaskState,
) {
    scheduler.submit(TaskAssignment {
        task_id,
        task_type: task_type.to_string(),
        priority: 128,
        deadline: None,
        input: json!({ "source": "topology-routing-test" }),
        output: None,
        state,
        assigned_to: None,
        parent_task_id: Some(parent_task_id),
        team_id: None,
        message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        assigned_at: None,
        completed_at: (state == TaskState::Completed).then_some(Utc::now()),
        error_message: None,
    });
}

#[test]
fn topology_compiler_selects_parallel_for_independent_branches() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(
            TaskId::new(),
            &independent_branches_plan(),
            &TopologySignals::default(),
        )
        .expect("independent branches should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Parallel);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Barrier
    );
    assert!(!graph.topology_plan.rationale.dependency_shape.is_empty());
    assert!(!graph.topology_plan.rationale.operational_signals.is_empty());
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_selects_sequential_for_strict_chain() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(
            TaskId::new(),
            &strict_chain_plan(),
            &TopologySignals::default(),
        )
        .expect("strict chain should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Sequential);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::StrictSequence
    );
    assert_eq!(graph.topology_plan.fallback_topology, None);
}

#[test]
fn topology_compiler_selects_pipeline_for_streaming_chain() {
    let compiler = TopologyCompiler::default();
    let signals = TopologySignals {
        prefer_streaming: true,
        ..TopologySignals::default()
    };
    let graph = compiler
        .compile(TaskId::new(), &strict_chain_plan(), &signals)
        .expect("streaming chain should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Pipeline);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Streaming
    );
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_selects_hybrid_for_join_graph() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(TaskId::new(), &join_plan(), &TopologySignals::default())
        .expect("join graph should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Hybrid);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Mixed
    );
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_honors_compatible_hierarchical_hint() {
    let compiler = TopologyCompiler::default();
    let signals = TopologySignals {
        health_state: Some(HealthState::Healthy),
        ..TopologySignals::default()
    };
    let graph = compiler
        .compile(TaskId::new(), &hierarchical_hint_plan(), &signals)
        .expect("hierarchical hint should compile");

    assert_eq!(
        graph.topology_plan.topology_kind,
        TopologyKind::Hierarchical
    );
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::HierarchicalReduce
    );
    assert!(graph
        .topology_plan
        .rationale
        .selected_for
        .contains("hierarchical"));
}

#[tokio::test]
async fn orchestrator_routes_ready_branches_with_health_budget_depth_and_profile_rationale() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(TaskId::new(), &join_plan(), &TopologySignals::default())
        .expect("join workflow should compile");
    let workflow_id = graph.workflow_id;
    let root_id = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "root")
        .map(|node| node.node_id)
        .expect("root node should exist");
    let left_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "left")
        .map(|node| node.branch_id)
        .expect("left branch should exist");
    let right_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "right")
        .map(|node| node.branch_id)
        .expect("right branch should exist");

    for node in &mut graph.nodes {
        if node.step_key == "root" {
            node.state = NodeState::Completed;
        }
        if node.step_key == "left" {
            node.budget = ContextBudget {
                budget_id: ContextBudgetId::new(),
                scope: BudgetScope::Branch,
                max_units: 10,
                reserved_units: 1,
                policy: BudgetPolicy::Summarize,
            };
        }
        if node.step_key == "right" {
            node.budget = ContextBudget {
                budget_id: ContextBudgetId::new(),
                scope: BudgetScope::Branch,
                max_units: 10,
                reserved_units: 9,
                policy: BudgetPolicy::Summarize,
            };
        }
    }
    let root_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "root")
        .map(|node| node.branch_id)
        .expect("root branch should exist");
    graph.branch_mut(&root_branch).unwrap().state = mister_smith_core::BranchState::Completed;
    orchestrator.register_execution_graph(graph.clone());

    for node in &graph.nodes {
        let state = if node.node_id == root_id {
            TaskState::Completed
        } else {
            TaskState::Pending
        };
        submit_task(
            &scheduler,
            TaskId::from_uuid(*node.node_id.as_ref()),
            node.step_key.as_str(),
            workflow_id,
            state,
        );
    }

    orchestrator.record_profile_assessment(
        &workflow_id,
        ProfileAssessment::from_supervisory_signals(
            &GuardTarget::Branch(left_branch),
            vec![],
            vec![],
        ),
    );
    orchestrator.record_profile_assessment(
        &workflow_id,
        ProfileAssessment::from_supervisory_signals(
            &GuardTarget::Branch(right_branch),
            vec![SemanticSignal {
                signal_kind: SemanticSignalKind::LowConfidence,
                severity: 10,
                detail: "profile shows degraded reasoning quality".to_string(),
            }],
            vec!["route conservatively".to_string()],
        ),
    );

    let workers = [AgentId::new(), AgentId::new()];
    let decisions = orchestrator
        .route_ready_branches(&workflow_id, &workers)
        .expect("ready branches should route");

    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].branch_id, left_branch);
    assert_eq!(decisions[0].health_state, HealthState::Healthy);
    assert!(decisions[0]
        .rationale
        .iter()
        .any(|line| line.contains("budget pressure")));
    assert!(decisions[0]
        .rationale
        .iter()
        .any(|line| line.contains("dependency depth")));
    assert!(decisions[0]
        .rationale
        .iter()
        .any(|line| line.contains("profile")));
    assert!(orchestrator
        .autonomy_events(&workflow_id)
        .iter()
        .any(|event| matches!(event, AutonomyEvent::RoutingDecisionRecorded(_))));
}
