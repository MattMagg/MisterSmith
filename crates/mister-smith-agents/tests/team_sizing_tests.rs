use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{TopologyCompiler, TopologySignals};
use mister_smith_core::{
    AgentId, BudgetPolicy, BudgetScope, ContextBudget, ContextBudgetId, NodeState, TaskId,
};
use serde_json::json;
use uuid::Uuid;

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

fn step_node_id(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionNodeId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.node_id)
        .expect("expected node to exist")
}

fn step_branch_id(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionBranchId {
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
        input: json!({ "source": "team-sizing-test" }),
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
) -> (Arc<TaskScheduler>, Orchestrator, TaskId) {
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

    for completed_step in completed_steps {
        let node_id = step_node_id(&graph, completed_step);
        graph
            .nodes
            .iter_mut()
            .find(|node| node.node_id == node_id)
            .expect("completed node should exist")
            .state = NodeState::Completed;
    }
    let completed_step_keys = completed_steps.iter().copied().collect::<Vec<_>>();
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
                .state = mister_smith_core::BranchState::Completed;
        }
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

    (scheduler, orchestrator, workflow_id)
}

#[test]
fn adaptive_team_sizing_varies_across_representative_shapes() {
    let workers = [AgentId::new(), AgentId::new(), AgentId::new()];

    let (_, wide_orchestrator, wide_workflow_id) = fixture_for(wide_fanout_plan(), &["root"]);
    let wide_decisions = wide_orchestrator
        .route_ready_branches(&wide_workflow_id, &workers)
        .expect("wide frontier should route");
    let wide_team_plan = wide_orchestrator
        .adaptive_team_plan(&wide_workflow_id)
        .expect("wide frontier should materialize a team plan");

    assert_eq!(wide_decisions.len(), 3);
    assert_eq!(wide_team_plan.sizing_decision.desired_workers, 3);
    assert_eq!(wide_team_plan.sizing_decision.selected_workers, 3);
    assert_eq!(wide_team_plan.worker_ids.len(), 3);

    let (_, chain_orchestrator, chain_workflow_id) = fixture_for(strict_chain_plan(), &["step-1"]);
    let chain_decisions = chain_orchestrator
        .route_ready_branches(&chain_workflow_id, &workers)
        .expect("chain frontier should route");
    let chain_team_plan = chain_orchestrator
        .adaptive_team_plan(&chain_workflow_id)
        .expect("chain frontier should materialize a team plan");

    assert_eq!(chain_decisions.len(), 1);
    assert_eq!(chain_team_plan.sizing_decision.desired_workers, 1);
    assert_eq!(chain_team_plan.sizing_decision.selected_workers, 1);
    assert_eq!(chain_team_plan.worker_ids.len(), 1);
}

#[test]
fn degraded_pressure_caps_parallel_frontier_to_single_worker() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &wide_fanout_plan(),
            &TopologySignals::default(),
        )
        .expect("wide graph should compile");
    let workflow_id = graph.workflow_id;
    let root_branch = step_branch_id(&graph, "root");

    graph
        .nodes
        .iter_mut()
        .find(|node| node.step_key == "root")
        .expect("root node should exist")
        .state = NodeState::Completed;
    graph.branch_mut(&root_branch).unwrap().state = mister_smith_core::BranchState::Completed;
    for step_key in ["alpha", "beta", "gamma"] {
        graph
            .nodes
            .iter_mut()
            .find(|node| node.step_key == step_key)
            .expect("fanout node should exist")
            .budget = ContextBudget {
            budget_id: ContextBudgetId::new(),
            scope: BudgetScope::Branch,
            max_units: 10,
            reserved_units: 10,
            policy: BudgetPolicy::Summarize,
        };
    }
    orchestrator.register_execution_graph(graph.clone());

    for node in &graph.nodes {
        let state = if node.step_key == "root" {
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

    let workers = [AgentId::new(), AgentId::new(), AgentId::new()];
    let decisions = orchestrator
        .route_ready_branches(&workflow_id, &workers)
        .expect("capped frontier should still route");
    let team_plan = orchestrator
        .adaptive_team_plan(&workflow_id)
        .expect("capped frontier should materialize a team plan");

    assert_eq!(decisions.len(), 1);
    assert_eq!(team_plan.sizing_decision.desired_workers, 3);
    assert_eq!(team_plan.sizing_decision.selected_workers, 1);
    assert_eq!(team_plan.worker_ids.len(), 1);
    assert!(team_plan.sizing_decision.cap_reason.is_some());
    assert_eq!(team_plan.sizing_decision.budget_pressure, Some(100));
    assert_eq!(team_plan.sizing_decision.conservative_mode, true);
}
