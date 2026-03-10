use std::sync::Arc;

use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{Orchestrator, TopologyCompiler, TopologySignals};
use mister_smith_core::{GraphState, TaskId, TopologyError, TopologyKind};
use serde_json::json;

fn parallel_plan() -> serde_json::Value {
    json!({
        "goal": "parallel-analysis",
        "steps": [
            {
                "id": "collect",
                "step": 1,
                "action": "collect-inputs",
                "description": "Collect workflow inputs"
            },
            {
                "id": "analyze-a",
                "step": 2,
                "action": "analyze-branch-a",
                "description": "Analyze branch A",
                "depends_on": ["collect"],
                "branch": "branch-a"
            },
            {
                "id": "analyze-b",
                "step": 3,
                "action": "analyze-branch-b",
                "description": "Analyze branch B",
                "depends_on": ["collect"],
                "branch": "branch-b"
            }
        ]
    })
}

fn missing_dependency_plan() -> serde_json::Value {
    json!({
        "goal": "broken-workflow",
        "steps": [
            {
                "id": "only-step",
                "step": 1,
                "action": "do-work",
                "description": "Do work",
                "depends_on": ["missing-step"]
            }
        ]
    })
}

fn cyclic_plan() -> serde_json::Value {
    json!({
        "goal": "cyclic-workflow",
        "steps": [
            {
                "id": "first",
                "step": 1,
                "action": "first-step",
                "description": "First step",
                "depends_on": ["second"]
            },
            {
                "id": "second",
                "step": 2,
                "action": "second-step",
                "description": "Second step",
                "depends_on": ["first"]
            }
        ]
    })
}

fn duplicate_numeric_step_plan() -> serde_json::Value {
    json!({
        "goal": "ambiguous-workflow",
        "steps": [
            {
                "id": "collect",
                "step": 1,
                "action": "collect-inputs",
                "description": "Collect workflow inputs"
            },
            {
                "id": "draft",
                "step": 2,
                "action": "draft-output",
                "description": "Draft output",
                "depends_on": ["collect"]
            },
            {
                "id": "review",
                "step": 2,
                "action": "review-output",
                "description": "Review output",
                "depends_on": [2]
            }
        ]
    })
}

#[test]
fn compiler_builds_valid_execution_graph_from_planner_output() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(TaskId::new(), &parallel_plan(), &TopologySignals::default())
        .expect("planner output should compile");

    assert_eq!(graph.workflow_id, graph.workflow_id);
    assert_eq!(graph.state, GraphState::Pending);
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
    assert_eq!(graph.root_nodes().len(), 1);
    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Parallel);
}

#[test]
fn compiler_rejects_missing_dependencies_before_dispatch() {
    let compiler = TopologyCompiler::default();
    let err = compiler
        .compile(
            TaskId::new(),
            &missing_dependency_plan(),
            &TopologySignals::default(),
        )
        .expect_err("missing dependency should fail validation");

    assert!(matches!(err, TopologyError::MissingDependency { .. }));
}

#[test]
fn compiler_rejects_cycles_before_dispatch() {
    let compiler = TopologyCompiler::default();
    let err = compiler
        .compile(TaskId::new(), &cyclic_plan(), &TopologySignals::default())
        .expect_err("cycle should fail validation");

    assert!(matches!(err, TopologyError::CycleDetected { .. }));
}

#[test]
fn compiler_rejects_duplicate_numeric_step_references() {
    let compiler = TopologyCompiler::default();
    let err = compiler
        .compile(
            TaskId::new(),
            &duplicate_numeric_step_plan(),
            &TopologySignals::default(),
        )
        .expect_err("duplicate numeric steps should fail validation");

    assert!(matches!(
        err,
        TopologyError::Invalid(message)
            if message.contains("duplicate numeric step reference")
    ));
}

#[test]
fn execution_graph_validation_rejects_nodes_in_multiple_branches() {
    let compiler = TopologyCompiler::default();
    let mut graph = compiler
        .compile(TaskId::new(), &parallel_plan(), &TopologySignals::default())
        .expect("planner output should compile");
    let shared_node = graph.branches[0].node_ids[0];
    graph.branches[1].node_ids.push(shared_node);

    let err = graph
        .validate()
        .expect_err("shared branch membership should fail validation");

    assert!(matches!(
        err,
        TopologyError::Invalid(message) if message.contains("multiple branches")
    ));
}

#[tokio::test]
async fn orchestrator_records_execution_graph_before_scheduler_submission() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let task = TaskAssignment::new("analysis", json!({"data": "test"}));

    assert!(orchestrator.execution_graph(&task.task_id).is_none());

    let subtask_ids = orchestrator
        .decompose(&task)
        .await
        .expect("decomposition should succeed");
    let graph = orchestrator
        .execution_graph(&task.task_id)
        .expect("compiled graph should be available before dispatch");

    assert_eq!(scheduler.count(), subtask_ids.len());
    assert_eq!(graph.workflow_id, task.task_id);
    assert_eq!(graph.nodes.len(), subtask_ids.len());
    assert_eq!(graph.state, GraphState::Pending);
}
