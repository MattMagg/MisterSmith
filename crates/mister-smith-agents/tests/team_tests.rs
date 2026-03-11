use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::{TaskState, TeamPattern};
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskDecomposer, TaskScheduler,
};
use mister_smith_agents::team::Team;
use mister_smith_agents::{AgentSystemError, BranchCheckpoint, TopologyCompiler, TopologySignals};
use mister_smith_core::{AgentId, BranchState, MemorySnapshotId, TaskId};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_team_assembly_supervisor_worker() {
    let coord_id = AgentId::new();
    let worker_ids = vec![AgentId::new(), AgentId::new(), AgentId::new()];
    let task_id = TaskId::new();

    let team = Team::new(
        coord_id,
        TeamPattern::SupervisorWorker,
        task_id,
        worker_ids.clone(),
    );

    assert!(team.is_active());
    assert_eq!(team.member_count(), 3);
    assert_eq!(team.coordinator_id, coord_id);
    assert_eq!(team.task_id, task_id);
    assert_eq!(team.pattern, TeamPattern::SupervisorWorker);
}

#[tokio::test]
async fn test_team_with_supervisor() {
    let coord_id = AgentId::new();
    let supervisor_id = AgentId::new();
    let worker_ids = vec![AgentId::new(), AgentId::new()];
    let task_id = TaskId::new();

    let team = Team::new(coord_id, TeamPattern::SupervisorWorker, task_id, worker_ids)
        .with_supervisor(supervisor_id);

    assert_eq!(team.supervisor_id, Some(supervisor_id));
}

#[tokio::test]
async fn test_team_disband_on_completion() {
    let mut team = Team::new(
        AgentId::new(),
        TeamPattern::Pipeline,
        TaskId::new(),
        vec![AgentId::new()],
    );

    assert!(team.is_active());
    team.disband();
    assert!(!team.is_active());
    assert!(team.disbanded_at.is_some());
}

#[tokio::test]
async fn test_task_decomposition_and_assignment() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );

    let task = TaskAssignment::new("analysis", serde_json::json!({"data": "test-data"}));
    let task_id = task.task_id;

    // Decompose
    let subtask_ids = orchestrator.decompose(&task).await.unwrap();
    assert_eq!(subtask_ids.len(), 1);

    // Assign to a worker
    let worker_id = AgentId::new();
    scheduler.assign(&subtask_ids[0], worker_id).unwrap();

    let assigned = scheduler.get(&subtask_ids[0]).unwrap();
    assert_eq!(assigned.state, TaskState::Assigned);
    assert_eq!(assigned.assigned_to, Some(worker_id));
    assert_eq!(assigned.parent_task_id, Some(task_id));
}

#[tokio::test]
async fn test_result_aggregation_from_multiple_workers() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(SplitDecomposer(3)),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );

    let task = TaskAssignment::new("multi-step", serde_json::json!({"items": 3}));
    let task_id = task.task_id;

    // Decompose into 3 subtasks
    let subtask_ids = orchestrator.decompose(&task).await.unwrap();
    assert_eq!(subtask_ids.len(), 3);

    // Simulate workers completing all subtasks
    for (i, sub_id) in subtask_ids.iter().enumerate() {
        let worker = AgentId::new();
        scheduler.assign(sub_id, worker).unwrap();
        scheduler.start(sub_id).unwrap();
        scheduler
            .complete(sub_id, serde_json::json!({"part": i, "done": true}))
            .unwrap();
    }

    // Verify all complete
    assert!(orchestrator.all_subtasks_completed(&task_id));

    // Aggregate
    let result = orchestrator.aggregate(&task_id).await.unwrap();
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn test_worker_failure_and_reassignment() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(SplitDecomposer(2)),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );

    let task = TaskAssignment::new("failover-test", serde_json::json!({}));
    let task_id = task.task_id;

    let subtask_ids = orchestrator.decompose(&task).await.unwrap();
    assert_eq!(subtask_ids.len(), 2);

    let worker_a = AgentId::new();
    let worker_b = AgentId::new();

    // Assign and start both
    scheduler.assign(&subtask_ids[0], worker_a).unwrap();
    scheduler.start(&subtask_ids[0]).unwrap();
    scheduler.assign(&subtask_ids[1], worker_b).unwrap();
    scheduler.start(&subtask_ids[1]).unwrap();

    // Worker A fails
    scheduler.fail(&subtask_ids[0], "worker crashed").unwrap();

    // Check failed subtasks
    let failed = orchestrator.failed_subtasks(&task_id);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].task_id, subtask_ids[0]);

    // Reassign to worker B
    orchestrator
        .reassign_subtask(&subtask_ids[0], worker_b)
        .unwrap();

    let reassigned = scheduler.get(&subtask_ids[0]).unwrap();
    assert_eq!(reassigned.state, TaskState::Assigned);
    assert_eq!(reassigned.assigned_to, Some(worker_b));

    // Complete both
    scheduler.start(&subtask_ids[0]).unwrap();
    scheduler
        .complete(&subtask_ids[0], serde_json::json!({"retried": true}))
        .unwrap();
    scheduler
        .complete(&subtask_ids[1], serde_json::json!({"ok": true}))
        .unwrap();

    assert!(orchestrator.all_subtasks_completed(&task_id));
    let result = orchestrator.aggregate(&task_id).await.unwrap();
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_deadline_timeout() {
    let scheduler = Arc::new(TaskScheduler::new());

    // Create a task with an already-passed deadline
    let task = TaskAssignment::new("deadline-test", serde_json::json!({}))
        .with_deadline(chrono::Utc::now() - chrono::Duration::seconds(10));
    let task_id = task.task_id;

    scheduler.submit(task);
    let agent = AgentId::new();
    scheduler.assign(&task_id, agent).unwrap();
    scheduler.start(&task_id).unwrap();

    // Manually trigger timeout (simulates deadline monitor)
    scheduler.timeout(&task_id).unwrap();

    let timed_out = scheduler.get(&task_id).unwrap();
    assert_eq!(timed_out.state, TaskState::TimedOut);
}

#[tokio::test]
async fn test_orchestrator_execute_assigns_subtasks() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(SplitDecomposer(3)),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );

    let task = TaskAssignment::new("execute-test", serde_json::json!({"data": 42}));
    let workers = vec![AgentId::new(), AgentId::new()];

    let result = orchestrator.execute(&task, &workers).await.unwrap();

    // Subtasks are assigned but not yet complete
    assert_eq!(result["status"], "assigned");
    assert_eq!(result["subtask_count"], 3);
}

#[tokio::test]
async fn test_team_member_management() {
    let mut team = Team::new(
        AgentId::new(),
        TeamPattern::Consensus,
        TaskId::new(),
        vec![],
    );

    let member1 = AgentId::new();
    let member2 = AgentId::new();

    team.add_member(member1);
    team.add_member(member2);
    assert_eq!(team.member_count(), 2);

    // Remove one
    team.remove_member(&member1);
    assert_eq!(team.member_count(), 1);
    assert!(team.members.contains(&member2));
    assert!(!team.members.contains(&member1));
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
        input: json!({ "source": "team-routing-test" }),
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

#[tokio::test]
async fn branch_routing_assigns_only_checkpoint_recovery_scope() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &json!({
                "goal": "checkpoint-routing",
                "steps": [
                    {
                        "id": "root",
                        "step": 1,
                        "action": "root",
                        "description": "root"
                    },
                    {
                        "id": "branch-a-1",
                        "step": 2,
                        "action": "branch-a-1",
                        "description": "branch-a-1",
                        "depends_on": ["root"],
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-a-2",
                        "step": 3,
                        "action": "branch-a-2",
                        "description": "branch-a-2",
                        "depends_on": ["branch-a-1"],
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 4,
                        "action": "branch-b",
                        "description": "branch-b",
                        "depends_on": ["root"],
                        "branch": "branch-b"
                    }
                ]
            }),
            &TopologySignals::default(),
        )
        .expect("checkpoint routing graph should compile");
    let workflow_id = graph.workflow_id;
    let root = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "root")
        .map(|node| node.node_id)
        .expect("root should exist");
    let a1 = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a-1")
        .map(|node| node.node_id)
        .expect("branch-a-1 should exist");
    let a2 = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a-2")
        .map(|node| node.node_id)
        .expect("branch-a-2 should exist");
    let b = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-b")
        .map(|node| node.node_id)
        .expect("branch-b should exist");
    let root_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "root")
        .map(|node| node.branch_id)
        .expect("root branch should exist");
    let a_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a-1")
        .map(|node| node.branch_id)
        .expect("branch-a should exist");
    let b_branch = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-b")
        .map(|node| node.branch_id)
        .expect("branch-b should exist");

    graph.branch_mut(&root_branch).unwrap().state = BranchState::Completed;
    graph.branch_mut(&a_branch).unwrap().state = BranchState::Checkpointed;
    graph.branch_mut(&b_branch).unwrap().state = BranchState::Completed;
    graph.checkpoint_lineage.push(BranchCheckpoint::new(
        a_branch,
        vec![a1],
        vec![a2],
        MemorySnapshotId::new(),
    ));
    orchestrator.register_execution_graph(graph);

    submit_task(
        &scheduler,
        TaskId::from_uuid(*root.as_ref()),
        "root",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*a1.as_ref()),
        "branch-a-1",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*a2.as_ref()),
        "branch-a-2",
        workflow_id,
        TaskState::Failed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*b.as_ref()),
        "branch-b",
        workflow_id,
        TaskState::Completed,
    );

    let worker = AgentId::new();
    let decisions = orchestrator
        .route_ready_branches(&workflow_id, &[worker])
        .expect("checkpoint recovery should route pending recovery scope");

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].task_ids, vec![TaskId::from_uuid(*a2.as_ref())]);
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*a1.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*b.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*a2.as_ref()))
            .unwrap()
            .assigned_to,
        Some(worker)
    );
}

// --- Test helpers ---

/// Decomposes a task into N identical subtasks.
struct SplitDecomposer(usize);

#[async_trait::async_trait]
impl TaskDecomposer for SplitDecomposer {
    async fn decompose(
        &self,
        task: &TaskAssignment,
    ) -> Result<Vec<TaskAssignment>, AgentSystemError> {
        let mut subtasks = Vec::with_capacity(self.0);
        for i in 0..self.0 {
            let sub =
                TaskAssignment::new(format!("{}-part-{}", task.task_type, i), task.input.clone());
            subtasks.push(sub);
        }
        Ok(subtasks)
    }
}
