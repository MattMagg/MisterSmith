use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{ExecutionGraph, TopologyCompiler, TopologySignals};
use mister_smith_core::{AgentId, BranchState, NodeState, TaskId};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BenchmarkStrategy {
    Adaptive,
    SequentialBaseline,
}

impl BenchmarkStrategy {
    const fn label(self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::SequentialBaseline => "sequential_baseline",
        }
    }

    const fn worker_count(self) -> usize {
        match self {
            Self::Adaptive => 3,
            Self::SequentialBaseline => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HarnessResult {
    workload_class: &'static str,
    strategy: &'static str,
    ready_branch_count: usize,
    desired_workers: usize,
    selected_workers: usize,
    dispatch_rounds: usize,
}

fn wide_fanout_plan() -> serde_json::Value {
    json!({
        "goal": "wide-fanout",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "root"
            },
            {
                "id": "alpha",
                "step": 2,
                "action": "alpha",
                "description": "alpha",
                "depends_on": ["root"],
                "branch": "alpha"
            },
            {
                "id": "beta",
                "step": 3,
                "action": "beta",
                "description": "beta",
                "depends_on": ["root"],
                "branch": "beta"
            },
            {
                "id": "gamma",
                "step": 4,
                "action": "gamma",
                "description": "gamma",
                "depends_on": ["root"],
                "branch": "gamma"
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
                "action": "step-1",
                "description": "step-1"
            },
            {
                "id": "step-2",
                "step": 2,
                "action": "step-2",
                "description": "step-2",
                "depends_on": ["step-1"]
            },
            {
                "id": "step-3",
                "step": 3,
                "action": "step-3",
                "description": "step-3",
                "depends_on": ["step-2"]
            }
        ]
    })
}

fn step_node_id(graph: &ExecutionGraph, step_key: &str) -> mister_smith_core::ExecutionNodeId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.node_id)
        .expect("expected node to exist")
}

fn step_branch_id(graph: &ExecutionGraph, step_key: &str) -> mister_smith_core::ExecutionBranchId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.branch_id)
        .expect("expected branch to exist")
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
        input: json!({ "source": "team-sizing-benchmark" }),
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

fn fixture_for(
    plan: serde_json::Value,
    completed_steps: &[&str],
) -> (Arc<TaskScheduler>, Orchestrator, ExecutionGraph, TaskId) {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(TaskId::new(), &plan, &TopologySignals::default())
        .expect("fixture graph should compile");
    let workflow_id = graph.workflow_id;
    let completed_step_keys = completed_steps.iter().copied().collect::<Vec<_>>();

    for completed_step in completed_steps {
        let node_id = step_node_id(&graph, completed_step);
        graph
            .nodes
            .iter_mut()
            .find(|node| node.node_id == node_id)
            .expect("completed node should exist")
            .state = NodeState::Completed;
    }

    let branch_ids = graph
        .branches
        .iter()
        .map(|branch| branch.branch_id)
        .collect::<Vec<_>>();
    for branch_id in branch_ids {
        let all_nodes_completed = graph
            .branch(&branch_id)
            .expect("branch should exist")
            .node_ids
            .iter()
            .all(|node_id| {
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *node_id)
                    .map(|node| completed_step_keys.contains(&node.step_key.as_str()))
                    .unwrap_or(false)
            });
        if all_nodes_completed {
            graph
                .branch_mut(&branch_id)
                .expect("completed branch should exist")
                .state = BranchState::Completed;
        }
    }

    if completed_steps.contains(&"root") {
        let root_branch = step_branch_id(&graph, "root");
        graph
            .branch_mut(&root_branch)
            .expect("root branch should exist")
            .state = BranchState::Completed;
    }

    orchestrator.register_execution_graph(graph.clone());

    for node in &graph.nodes {
        let state = if completed_steps.contains(&node.step_key.as_str()) {
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

    (scheduler, orchestrator, graph, workflow_id)
}

fn ready_branch_count(graph: &ExecutionGraph) -> usize {
    graph
        .branches
        .iter()
        .filter(|branch| branch.state != BranchState::Completed)
        .filter(|branch| {
            branch.node_ids.iter().any(|node_id| {
                let node = graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *node_id)
                    .expect("branch node should exist");
                node.state == NodeState::Pending
                    && node.dependencies.iter().all(|dependency_id| {
                        graph
                            .nodes
                            .iter()
                            .find(|candidate| candidate.node_id == *dependency_id)
                            .map(|dependency| dependency.state == NodeState::Completed)
                            .unwrap_or(false)
                    })
            })
        })
        .count()
}

fn run_harness(
    workload_class: &'static str,
    plan: serde_json::Value,
    completed_steps: &[&str],
    strategy: BenchmarkStrategy,
) -> HarnessResult {
    let (_, orchestrator, graph, workflow_id) = fixture_for(plan, completed_steps);
    let workers = (0..strategy.worker_count())
        .map(|_| AgentId::new())
        .collect::<Vec<_>>();
    orchestrator
        .route_ready_branches(&workflow_id, &workers)
        .expect("benchmark workload should route");
    let team_plan = orchestrator
        .adaptive_team_plan(&workflow_id)
        .expect("benchmark workload should materialize a team plan");
    let ready_branch_count = ready_branch_count(&graph);
    let selected_workers = team_plan.sizing_decision.selected_workers.max(1);
    let dispatch_rounds = (ready_branch_count + selected_workers - 1) / selected_workers;

    HarnessResult {
        workload_class,
        strategy: strategy.label(),
        ready_branch_count,
        desired_workers: team_plan.sizing_decision.desired_workers,
        selected_workers,
        dispatch_rounds,
    }
}

#[test]
fn adaptive_team_harness_reports_improvement_and_neutral_results() {
    let wide_adaptive = run_harness(
        "parallel_fanout",
        wide_fanout_plan(),
        &["root"],
        BenchmarkStrategy::Adaptive,
    );
    let wide_sequential = run_harness(
        "parallel_fanout",
        wide_fanout_plan(),
        &["root"],
        BenchmarkStrategy::SequentialBaseline,
    );
    let chain_adaptive = run_harness(
        "strict_chain",
        strict_chain_plan(),
        &["step-1"],
        BenchmarkStrategy::Adaptive,
    );
    let chain_sequential = run_harness(
        "strict_chain",
        strict_chain_plan(),
        &["step-1"],
        BenchmarkStrategy::SequentialBaseline,
    );

    assert_eq!(wide_adaptive.ready_branch_count, 3);
    assert_eq!(wide_adaptive.desired_workers, 3);
    assert_eq!(wide_adaptive.selected_workers, 3);
    assert_eq!(wide_adaptive.dispatch_rounds, 1);

    assert_eq!(wide_sequential.ready_branch_count, 3);
    assert_eq!(wide_sequential.desired_workers, 3);
    assert_eq!(wide_sequential.selected_workers, 1);
    assert_eq!(wide_sequential.dispatch_rounds, 3);

    assert_eq!(chain_adaptive.ready_branch_count, 1);
    assert_eq!(chain_adaptive.desired_workers, 1);
    assert_eq!(chain_adaptive.selected_workers, 1);
    assert_eq!(chain_adaptive.dispatch_rounds, 1);

    assert_eq!(chain_sequential.ready_branch_count, 1);
    assert_eq!(chain_sequential.desired_workers, 1);
    assert_eq!(chain_sequential.selected_workers, 1);
    assert_eq!(chain_sequential.dispatch_rounds, 1);
    assert!(wide_adaptive.dispatch_rounds < wide_sequential.dispatch_rounds);
    assert_eq!(
        chain_adaptive.dispatch_rounds,
        chain_sequential.dispatch_rounds
    );

    println!("workload_class,strategy,ready_branch_count,desired_workers,selected_workers,dispatch_rounds");
    for result in [
        &wide_adaptive,
        &wide_sequential,
        &chain_adaptive,
        &chain_sequential,
    ] {
        println!(
            "{},{},{},{},{},{}",
            result.workload_class,
            result.strategy,
            result.ready_branch_count,
            result.desired_workers,
            result.selected_workers,
            result.dispatch_rounds
        );
    }
}

#[test]
fn adaptive_team_harness_is_repeatable() {
    let first_run = [
        run_harness(
            "parallel_fanout",
            wide_fanout_plan(),
            &["root"],
            BenchmarkStrategy::Adaptive,
        ),
        run_harness(
            "parallel_fanout",
            wide_fanout_plan(),
            &["root"],
            BenchmarkStrategy::SequentialBaseline,
        ),
        run_harness(
            "strict_chain",
            strict_chain_plan(),
            &["step-1"],
            BenchmarkStrategy::Adaptive,
        ),
        run_harness(
            "strict_chain",
            strict_chain_plan(),
            &["step-1"],
            BenchmarkStrategy::SequentialBaseline,
        ),
    ];
    let second_run = [
        run_harness(
            "parallel_fanout",
            wide_fanout_plan(),
            &["root"],
            BenchmarkStrategy::Adaptive,
        ),
        run_harness(
            "parallel_fanout",
            wide_fanout_plan(),
            &["root"],
            BenchmarkStrategy::SequentialBaseline,
        ),
        run_harness(
            "strict_chain",
            strict_chain_plan(),
            &["step-1"],
            BenchmarkStrategy::Adaptive,
        ),
        run_harness(
            "strict_chain",
            strict_chain_plan(),
            &["step-1"],
            BenchmarkStrategy::SequentialBaseline,
        ),
    ];

    assert_eq!(first_run, second_run);
}
