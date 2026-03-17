//! Runtime-backed HTTP task execution for one real local workflow path.

use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;
use std::thread;

use async_nats::jetstream::stream::RetentionPolicy;
use async_trait::async_trait;
use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::roles::executor::{ExecutorAgent, ExecutorMessage, ExecutorState};
use mister_smith_agents::roles::planner::{
    normalize_planner_output, PlannerAgent, PlannerMessage, PlannerState,
};
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{Orchestrator, TopologyCompiler, TopologySignals};
use mister_smith_core::{Actor, AgentId, BranchState, GraphState, NodeState, TaskId};
use mister_smith_events::{AutonomyStatusView, EventBus};
use mister_smith_http::server::{
    TaskExecutionService, TaskStatusView, TaskSubmissionRequest, TaskSubmissionResponse,
};
use mister_smith_llm::{
    CircuitBreakerConfig, ModelRouter, OpenAiChatGptProvider, ProviderConfig, ProviderKind,
    RoutingPolicy,
};
use mister_smith_nats::{JetStreamConfig, JetStreamManager, NatsTransport};
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_persistence::postgres::queries::{self, TaskRecord};
use mister_smith_persistence::repository::task::TaskRepository;
use mister_smith_persistence::{PostgresConnection, Repository};
use mister_smith_transport::{MessageEnvelope, MessagePriority};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::task::JoinSet;
use tracing::{error, info};
use uuid::Uuid;

use crate::auth;

const PROVIDER_KIND: ProviderKind = ProviderKind::OpenAiChatGpt;
pub(crate) const PROVIDER_KIND_NAME: &str = "openai_chatgpt";
pub(crate) const MODEL_ID: &str = "gpt-5.4";
const WORKFLOW_STREAM: &str = "mister_smith_workflows";
const WORKFLOW_SUBJECT_PATTERN: &str = "workflow.>";

struct PreparedDecisionExecution {
    tasks: Vec<PreparedTaskExecution>,
}

struct PreparedTaskExecution {
    task: TaskAssignment,
    worker_id: AgentId,
    execution_input: Value,
}

struct CompletedTaskExecution {
    task_id: TaskId,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    result: Value,
}

struct FailedTaskExecution {
    completed_steps: Vec<CompletedTaskExecution>,
    task_id: TaskId,
    worker_id: AgentId,
    branch_id: Value,
    message: String,
}

#[derive(Clone)]
pub(crate) struct RuntimeTaskService {
    pool: PgPool,
    repository: Arc<TaskRepository>,
    jetstream: Arc<JetStreamManager>,
    router: Arc<ModelRouter>,
    orchestrator: Arc<Orchestrator>,
    default_coordinator_id: AgentId,
    worker_ids: Vec<AgentId>,
}

impl RuntimeTaskService {
    pub(crate) async fn bootstrap(
        event_bus: Arc<EventBus>,
        nats_transport: Option<Arc<NatsTransport>>,
    ) -> Result<Arc<Self>, String> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL must be set for runtime task execution".to_string())?;
        let postgres = PostgresConnection::connect(&database_url)
            .await
            .map_err(|error| format!("PostgreSQL connection failed: {error}"))?;
        let pool = postgres.pool().clone();
        let migrations = MigrationRunner::new(pool.clone());
        migrations
            .run()
            .await
            .map_err(|error| format!("PostgreSQL migration failed: {error}"))?;
        let verified = migrations
            .verify()
            .await
            .map_err(|error| format!("PostgreSQL migration verification failed: {error}"))?;
        if !verified {
            return Err("PostgreSQL migrations did not verify cleanly".to_string());
        }

        let auth_status = auth::openai_chatgpt_status()
            .await
            .map_err(|error| format!("ChatGPT auth check failed: {error}"))?;
        if !auth_status.is_chatgpt_session() {
            return Err(auth::render_openai_chatgpt_status(&auth_status));
        }

        let nats_transport = nats_transport
            .ok_or_else(|| "NATS transport must be configured for runtime execution".to_string())?;
        let nats_client = nats_transport
            .inner_client()
            .await
            .map_err(|error| format!("NATS client unavailable: {error}"))?;
        let jetstream = Arc::new(JetStreamManager::new(
            nats_client,
            JetStreamConfig::default(),
        ));
        jetstream
            .create_stream(
                WORKFLOW_STREAM,
                vec![WORKFLOW_SUBJECT_PATTERN.to_string()],
                RetentionPolicy::Limits,
            )
            .await
            .map_err(|error| format!("JetStream stream initialization failed: {error}"))?;

        let provider_config = ProviderConfig {
            provider_kind: PROVIDER_KIND,
            model_id: MODEL_ID.to_string(),
            timeout_ms: 120_000,
            ..ProviderConfig::default()
        };
        let provider = Arc::new(
            OpenAiChatGptProvider::new(provider_config.clone())
                .map_err(|error| format!("provider initialization failed: {error}"))?,
        );
        let router = Arc::new(ModelRouter::new(RoutingPolicy::RoundRobin));
        router
            .add_provider(provider_config, provider, CircuitBreakerConfig::default())
            .await;

        let scheduler = Arc::new(TaskScheduler::new());
        let orchestrator = Arc::new(
            Orchestrator::new(
                Arc::new(IdentityDecomposer),
                Arc::new(ArrayAggregator),
                scheduler,
            )
            .with_event_bus(event_bus),
        );

        info!(
            provider_kind = PROVIDER_KIND_NAME,
            model_id = MODEL_ID,
            stream = WORKFLOW_STREAM,
            "Runtime task execution service ready"
        );

        Ok(Arc::new(Self {
            pool: pool.clone(),
            repository: Arc::new(TaskRepository::new(pool)),
            jetstream,
            router,
            orchestrator,
            default_coordinator_id: AgentId::new(),
            worker_ids: vec![AgentId::new(), AgentId::new()],
        }))
    }

    pub(crate) fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub(crate) async fn autonomy_status(&self, workflow_id: TaskId) -> Option<AutonomyStatusView> {
        let mut view = self.orchestrator.autonomy_status(&workflow_id)?;
        if let Ok(Some(record)) = queries::find_task(&self.pool, *workflow_id.as_ref()).await {
            crate::autonomy::enrich_session_linkage(&mut view, &record.metadata);
        }
        Some(view)
    }

    pub(crate) fn autonomy_workflows(&self) -> Vec<TaskId> {
        self.orchestrator.autonomy_workflow_ids()
    }

    async fn run_workflow(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        let coordinator_id = coordinator_id_for_request(&request, self.default_coordinator_id);
        self.orchestrator
            .register_workflow_coordinator(&workflow_id, coordinator_id);
        let mut metadata = initial_metadata(&request, coordinator_id, &self.worker_ids, "running");
        self.update_root_record(
            workflow_id,
            "running",
            metadata.clone(),
            None,
            Some(Utc::now()),
            None,
        )
        .await?;
        self.publish_event(
            coordinator_id,
            "workflow.started",
            "workflow.started",
            workflow_id,
            json!({
                "workflow_id": workflow_id,
                "provider_kind": PROVIDER_KIND_NAME,
                "model_id": MODEL_ID,
                "description": request.description,
            }),
        )
        .await?;

        let planning_context = json!({
            "submission_path": if request.conversation.is_some() { "session" } else { "http" },
            "provider_kind": PROVIDER_KIND_NAME,
            "model_id": MODEL_ID,
            "conversation": request.conversation.as_ref().map(|conversation| {
                json!({
                    "session_id": conversation.session_id,
                    "turn_index": conversation.turn_index,
                    "coordinator_agent_id": conversation.coordinator_agent_id,
                    "retained_context": conversation.retained_context,
                })
            }).unwrap_or(Value::Null),
            "execution_contract": {
                "require_real_multi_agent_workflow": true,
                "require_parallel_workers": 2,
                "require_join_step": true,
                "return_json_only": true,
                "schema": {
                    "goal": "string",
                    "topology_hint": "hybrid",
                    "steps": [
                        {
                            "id": "string",
                            "step": 1,
                            "action": "string",
                            "description": "string",
                            "role": "worker",
                            "branch": "branch-a",
                            "depends_on": []
                        }
                    ]
                }
            }
        });

        let mut planner = PlannerAgent::with_router(coordinator_id, self.router.clone());
        let mut planner_state = PlannerState::default();
        let planner_output = planner
            .handle_message(
                PlannerMessage::PlanGoal {
                    goal: request.description.clone(),
                    context: planning_context.clone(),
                    managed_context: None,
                },
                &mut planner_state,
            )
            .await
            .map_err(|error| format!("planner execution failed: {error}"))?;
        let execution_plan = normalize_runtime_plan(
            &request.description,
            &planning_context,
            planner_output.clone(),
        );
        put_metadata(&mut metadata, "planner_output", planner_output.clone());
        put_metadata(&mut metadata, "execution_plan", execution_plan.clone());
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        self.publish_event(
            coordinator_id,
            "workflow.planned",
            "workflow.planned",
            workflow_id,
            json!({
                "workflow_id": workflow_id,
                "provider_kind": PROVIDER_KIND_NAME,
                "model_id": MODEL_ID,
                "planner_output": planner_output,
                "execution_plan": execution_plan,
            }),
        )
        .await?;

        let compiler = TopologyCompiler;
        let mut graph = compiler
            .compile(
                workflow_id,
                metadata.get("execution_plan").unwrap(),
                &TopologySignals::default(),
            )
            .map_err(|error| format!("execution graph compile failed: {error}"))?;
        graph.state = GraphState::Running;
        self.orchestrator.register_execution_graph(graph.clone());

        for node in &graph.nodes {
            self.orchestrator
                .scheduler()
                .submit(task_assignment_for_node(
                    workflow_id,
                    node,
                    &request.description,
                ));
        }

        let mut step_results = BTreeMap::new();
        let mut routing_history = Vec::new();

        while !self.orchestrator.all_subtasks_completed(&workflow_id) {
            let decisions = self
                .orchestrator
                .route_ready_branches(&workflow_id, &self.worker_ids)
                .map_err(|error| format!("branch routing failed: {error}"))?;
            if decisions.is_empty() {
                return Err(format!(
                    "no routable branches remained for workflow {workflow_id}"
                ));
            }

            let adaptive_team_plan = self.orchestrator.adaptive_team_plan(&workflow_id);
            if let Some(team_plan) = adaptive_team_plan.as_ref() {
                put_metadata(
                    &mut metadata,
                    "active_worker_ids",
                    json!(team_plan.worker_ids),
                );
                put_metadata(
                    &mut metadata,
                    "team_sizing",
                    json!(team_plan.sizing_decision),
                );
            }

            let decision_payload = decisions
                .iter()
                .map(|decision| {
                    json!({
                        "branch_id": decision.branch_id,
                        "selected_agent": decision.selected_agent,
                        "task_ids": decision.task_ids,
                        "rationale": decision.rationale,
                    })
                })
                .collect::<Vec<_>>();
            routing_history.extend(decision_payload.clone());
            put_metadata(&mut metadata, "routing_history", json!(routing_history));
            self.update_task_metadata(workflow_id, metadata.clone())
                .await?;
            self.publish_event(
                coordinator_id,
                "workflow.routed",
                "workflow.routed",
                workflow_id,
                json!({
                    "workflow_id": workflow_id,
                    "routing": decision_payload,
                    "active_worker_ids": adaptive_team_plan
                        .as_ref()
                        .map(|team_plan| json!(team_plan.worker_ids))
                        .unwrap_or(Value::Null),
                    "team_sizing": adaptive_team_plan
                        .as_ref()
                        .map(|team_plan| json!(team_plan.sizing_decision))
                        .unwrap_or(Value::Null),
                }),
            )
            .await?;

            let mut prepared_decisions = Vec::new();
            for decision in decisions {
                let worker_id = decision.selected_agent.ok_or_else(|| {
                    format!("branch {} has no selected worker", decision.branch_id)
                })?;
                let mut prepared_tasks = Vec::new();
                for task_id in decision.task_ids {
                    self.orchestrator
                        .scheduler()
                        .start(&task_id)
                        .map_err(|error| {
                            format!("failed to start scheduled task {task_id}: {error}")
                        })?;
                    self.transition_graph(workflow_id, task_id, NodeState::Running);

                    let task = self.orchestrator.scheduler().get(&task_id).ok_or_else(|| {
                        format!("scheduled task {task_id} disappeared before execution")
                    })?;
                    let execution_input = execution_input_for_task(
                        &graph,
                        &task,
                        &request.description,
                        worker_id,
                        &step_results,
                    );
                    prepared_tasks.push(PreparedTaskExecution {
                        task,
                        worker_id,
                        execution_input,
                    });
                }

                if !prepared_tasks.is_empty() {
                    prepared_decisions.push(PreparedDecisionExecution {
                        tasks: prepared_tasks,
                    });
                }
            }

            let mut join_set = JoinSet::new();
            for prepared in prepared_decisions {
                let router = self.router.clone();
                join_set.spawn(execute_prepared_decision(router, workflow_id, prepared));
            }

            let mut failures = Vec::new();
            while let Some(joined) = join_set.join_next().await {
                match joined {
                    Ok(Ok(completed_steps)) => {
                        for completed_step in completed_steps {
                            self.record_completed_step(
                                workflow_id,
                                coordinator_id,
                                &mut metadata,
                                &mut step_results,
                                completed_step,
                            )
                            .await?;
                        }
                    }
                    Ok(Err(failed_step)) => {
                        let FailedTaskExecution {
                            completed_steps,
                            task_id,
                            worker_id,
                            branch_id,
                            message,
                        } = failed_step;
                        for completed_step in completed_steps {
                            self.record_completed_step(
                                workflow_id,
                                coordinator_id,
                                &mut metadata,
                                &mut step_results,
                                completed_step,
                            )
                            .await?;
                        }
                        self.record_failed_step(
                            workflow_id,
                            coordinator_id,
                            &mut metadata,
                            FailedTaskExecution {
                                completed_steps: Vec::new(),
                                task_id,
                                worker_id,
                                branch_id,
                                message: message.clone(),
                            },
                        )
                        .await?;
                        failures.push(message);
                    }
                    Err(error) => {
                        failures.push(format!(
                            "parallel branch execution join failed for workflow {workflow_id}: {error}"
                        ));
                    }
                }
            }

            if let Some(failure) = failures.into_iter().next() {
                return Err(failure);
            }

            graph = self
                .orchestrator
                .execution_graph(&workflow_id)
                .ok_or_else(|| format!("execution graph {workflow_id} missing after routing"))?;
        }

        let aggregated_result = self
            .orchestrator
            .aggregate(&workflow_id)
            .await
            .map_err(|error| format!("result aggregation failed: {error}"))?;
        let final_result = json!({
            "workflow_id": workflow_id,
            "provider_kind": PROVIDER_KIND_NAME,
            "model_id": MODEL_ID,
            "description": request.description,
            "planner_output": metadata.get("planner_output").cloned().unwrap_or(Value::Null),
            "execution_plan": metadata.get("execution_plan").cloned().unwrap_or(Value::Null),
            "step_results": step_results.values().cloned().collect::<Vec<_>>(),
            "aggregated_result": aggregated_result,
        });

        put_metadata(&mut metadata, "final_result", final_result.clone());
        self.transition_workflow_complete(workflow_id);
        self.update_root_record(
            workflow_id,
            "completed",
            metadata.clone(),
            Some(final_result.clone()),
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .await?;
        self.publish_event(
            coordinator_id,
            "workflow.completed",
            "workflow.completed",
            workflow_id,
            final_result,
        )
        .await?;

        info!(
            workflow_id = %workflow_id,
            provider_kind = PROVIDER_KIND_NAME,
            model_id = MODEL_ID,
            "Workflow completed"
        );
        Ok(())
    }

    async fn save_root_record(
        &self,
        request: &TaskSubmissionRequest,
        workflow_id: TaskId,
        coordinator_id: AgentId,
    ) -> Result<(), String> {
        let record = TaskRecord {
            task_id: *workflow_id.as_ref(),
            task_type: "workflow".to_string(),
            agent_id: None,
            payload: json!({
                "description": request.description,
                "agent_type": request.agent_type,
                "priority": request.priority,
                "provider_kind": PROVIDER_KIND_NAME,
                "model_id": MODEL_ID,
            }),
            result: None,
            metadata: initial_metadata(request, coordinator_id, &self.worker_ids, "queued"),
            status: "queued".to_string(),
            priority: priority_rank(request.priority.as_deref()),
            correlation_id: Some(*workflow_id.as_ref()),
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            expires_at: None,
        };
        self.repository
            .save(&record)
            .await
            .map_err(|error| format!("failed to persist root workflow record: {error}"))?;
        Ok(())
    }

    pub(crate) async fn prepare_workflow(
        &self,
        workflow_id: TaskId,
        request: &TaskSubmissionRequest,
    ) -> Result<(), String> {
        if request.description.trim().is_empty() {
            return Err("task description must not be empty".to_string());
        }

        let coordinator_id = coordinator_id_for_request(request, self.default_coordinator_id);
        self.save_root_record(request, workflow_id, coordinator_id)
            .await
    }

    pub(crate) fn launch_workflow(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        self.spawn_workflow_runner(workflow_id, request)
    }

    pub(crate) async fn delete_workflow_record(&self, workflow_id: TaskId) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM tasks.records
            WHERE task_id = $1
            "#,
        )
        .bind(*workflow_id.as_ref())
        .execute(&self.pool)
        .await
        .map_err(|error| format!("failed to delete workflow record {workflow_id}: {error}"))?;
        Ok(())
    }

    async fn find_task_record(&self, task_id: TaskId) -> Result<Option<TaskRecord>, String> {
        self.repository
            .find(task_id.as_ref())
            .await
            .map_err(|error| format!("failed to load task record {task_id}: {error}"))
    }

    async fn update_task_metadata(
        &self,
        workflow_id: TaskId,
        metadata: Value,
    ) -> Result<(), String> {
        queries::update_task_metadata(&self.pool, *workflow_id.as_ref(), metadata)
            .await
            .map_err(|error| {
                format!("failed to update workflow metadata {workflow_id}: {error}")
            })?;
        Ok(())
    }

    async fn update_root_record(
        &self,
        workflow_id: TaskId,
        status: &str,
        metadata: Value,
        result: Option<Value>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), String> {
        sqlx::query_as::<_, TaskRecord>(
            r#"
            UPDATE tasks.records
            SET status = $2::task_status_type,
                metadata = $3,
                result = $4,
                started_at = COALESCE(started_at, $5),
                completed_at = $6
            WHERE task_id = $1
            RETURNING
                task_id, task_type, agent_id, payload, result,
                metadata, status::TEXT AS status, priority, correlation_id,
                parent_task_id, created_at, started_at,
                completed_at, expires_at
            "#,
        )
        .bind(*workflow_id.as_ref())
        .bind(status)
        .bind(metadata)
        .bind(result)
        .bind(started_at)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| format!("failed to update workflow record {workflow_id}: {error}"))?;
        Ok(())
    }

    async fn fail_workflow(&self, workflow_id: TaskId, message: String) {
        let metadata = match self.find_task_record(workflow_id).await {
            Ok(Some(record)) => {
                let mut metadata = record.metadata;
                put_metadata(&mut metadata, "failure", json!({ "message": message }));
                metadata
            }
            _ => json!({ "failure": { "message": message } }),
        };
        let coordinator_id =
            coordinator_id_from_metadata(&metadata).unwrap_or(self.default_coordinator_id);
        let _ = self
            .update_root_record(
                workflow_id,
                "failed",
                metadata.clone(),
                Some(json!({
                    "workflow_id": workflow_id,
                    "provider_kind": PROVIDER_KIND_NAME,
                    "model_id": MODEL_ID,
                    "error": message,
                })),
                Some(Utc::now()),
                Some(Utc::now()),
            )
            .await;
        let _ = self
            .publish_event(
                coordinator_id,
                "workflow.failed",
                "workflow.failed",
                workflow_id,
                json!({
                    "workflow_id": workflow_id,
                    "provider_kind": PROVIDER_KIND_NAME,
                    "model_id": MODEL_ID,
                    "error": message,
                }),
            )
            .await;
    }

    async fn publish_event(
        &self,
        source_agent_id: AgentId,
        subject: &str,
        message_type: &str,
        workflow_id: TaskId,
        payload: Value,
    ) -> Result<(), String> {
        let envelope = MessageEnvelope::builder(message_type)
            .correlation_id(*workflow_id.as_ref())
            .source_agent_id(*source_agent_id.as_ref())
            .priority(MessagePriority::Normal)
            .payload_json(&payload)
            .map_err(|error| format!("failed to encode workflow event payload: {error}"))?
            .build()
            .map_err(|error| format!("failed to build workflow event envelope: {error}"))?;

        self.jetstream
            .publish_and_ack(subject, envelope)
            .await
            .map_err(|error| format!("failed to publish JetStream event on {subject}: {error}"))
    }

    async fn record_completed_step(
        &self,
        workflow_id: TaskId,
        coordinator_id: AgentId,
        metadata: &mut Value,
        step_results: &mut BTreeMap<String, Value>,
        completed_step: CompletedTaskExecution,
    ) -> Result<(), String> {
        self.orchestrator
            .scheduler()
            .complete(&completed_step.task_id, completed_step.result.clone())
            .map_err(|error| {
                format!(
                    "failed to complete scheduled task {}: {error}",
                    completed_step.task_id
                )
            })?;
        self.transition_graph(workflow_id, completed_step.task_id, NodeState::Completed);

        let step_result = json!({
            "task_id": completed_step.task_id,
            "worker_id": completed_step.worker_id,
            "branch_id": completed_step.branch_id,
            "action": completed_step.action,
            "result": completed_step.result,
        });
        step_results.insert(completed_step.task_id.to_string(), step_result.clone());
        put_metadata(
            metadata,
            "step_results",
            json!(step_results.values().cloned().collect::<Vec<_>>()),
        );
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        self.publish_event(
            coordinator_id,
            "workflow.step.completed",
            "workflow.step.completed",
            workflow_id,
            step_result,
        )
        .await
    }

    async fn record_failed_step(
        &self,
        workflow_id: TaskId,
        coordinator_id: AgentId,
        metadata: &mut Value,
        failed_step: FailedTaskExecution,
    ) -> Result<(), String> {
        self.orchestrator
            .scheduler()
            .fail(&failed_step.task_id, failed_step.message.clone())
            .map_err(|error| {
                format!(
                    "failed to mark scheduled task {} as failed: {error}",
                    failed_step.task_id
                )
            })?;
        self.transition_graph(workflow_id, failed_step.task_id, NodeState::Failed);
        put_metadata(
            metadata,
            "last_step_failure",
            json!(failed_step.message.clone()),
        );
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        self.publish_event(
            coordinator_id,
            "workflow.step.failed",
            "workflow.step.failed",
            workflow_id,
            json!({
                "task_id": failed_step.task_id,
                "worker_id": failed_step.worker_id,
                "branch_id": failed_step.branch_id,
                "error": failed_step.message,
            }),
        )
        .await
    }

    fn transition_graph(&self, workflow_id: TaskId, task_id: TaskId, node_state: NodeState) {
        let Some(mut graph) = self.orchestrator.execution_graph(&workflow_id) else {
            return;
        };

        let node_id = mister_smith_core::ExecutionNodeId::from_uuid(*task_id.as_ref());
        let branch_id = graph
            .nodes
            .iter_mut()
            .find(|node| node.node_id == node_id)
            .map(|node| {
                node.state = node_state;
                node.branch_id
            });

        let Some(branch_id) = branch_id else {
            return;
        };

        if let Some(branch) = graph
            .branches
            .iter_mut()
            .find(|branch| branch.branch_id == branch_id)
        {
            let branch_nodes = graph
                .nodes
                .iter()
                .filter(|node| node.branch_id == branch_id)
                .collect::<Vec<_>>();
            branch.state = if branch_nodes
                .iter()
                .any(|node| node.state == NodeState::Failed)
            {
                BranchState::Failed
            } else if branch_nodes
                .iter()
                .all(|node| node.state == NodeState::Completed)
            {
                BranchState::Completed
            } else {
                BranchState::Running
            };
        }

        graph.state = if graph
            .nodes
            .iter()
            .any(|node| node.state == NodeState::Failed)
        {
            GraphState::Failed
        } else if graph
            .nodes
            .iter()
            .all(|node| node.state == NodeState::Completed)
        {
            GraphState::Completed
        } else {
            GraphState::Running
        };

        self.orchestrator.register_execution_graph(graph);
    }

    fn transition_workflow_complete(&self, workflow_id: TaskId) {
        let Some(mut graph) = self.orchestrator.execution_graph(&workflow_id) else {
            return;
        };

        for node in &mut graph.nodes {
            node.state = NodeState::Completed;
        }
        for branch in &mut graph.branches {
            branch.state = BranchState::Completed;
        }
        graph.state = GraphState::Completed;
        self.orchestrator.register_execution_graph(graph);
    }

    fn spawn_workflow_runner(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        let service = self.clone();
        let thread_name = format!("mister-smith-workflow-{workflow_id}");

        thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let message =
                            format!("failed to build dedicated workflow runtime: {error}");
                        error!(workflow_id = %workflow_id, error = %message, "Workflow run failed");
                        if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            runtime.block_on(service.fail_workflow(workflow_id, message));
                        }
                        return;
                    }
                };

                runtime.block_on(async move {
                    if let Err(error) = service.run_workflow(workflow_id, request).await {
                        error!(workflow_id = %workflow_id, error = %error, "Workflow run failed");
                        service.fail_workflow(workflow_id, error).await;
                    }
                });
            })
            .map(|_| ())
            .map_err(|error| format!("failed to spawn workflow runner thread: {error}"))
    }
}

#[async_trait]
impl TaskExecutionService for RuntimeTaskService {
    async fn submit_task(
        &self,
        request: TaskSubmissionRequest,
    ) -> Result<TaskSubmissionResponse, String> {
        let workflow_id = TaskId::new();
        self.prepare_workflow(workflow_id, &request).await?;
        self.launch_workflow(workflow_id, request.clone())?;

        let coordinator_id = coordinator_id_for_request(&request, self.default_coordinator_id);

        Ok(TaskSubmissionResponse {
            task_id: workflow_id,
            assigned_agent_id: coordinator_id,
            status: "queued".to_string(),
        })
    }

    async fn get_task(&self, task_id: TaskId) -> Result<Option<TaskStatusView>, String> {
        let record = self.find_task_record(task_id).await?;
        Ok(record.map(|record| TaskStatusView {
            task_id: TaskId::from_uuid(record.task_id),
            status: record.status,
            result: record.result,
        }))
    }
}

fn initial_metadata(
    request: &TaskSubmissionRequest,
    coordinator_id: AgentId,
    worker_ids: &[AgentId],
    status: &str,
) -> Value {
    let mut metadata = json!({
        "provider_kind": PROVIDER_KIND_NAME,
        "model_id": MODEL_ID,
        "submission_path": if request.conversation.is_some() { "session" } else { "http" },
        "status": status,
        "description": request.description,
        "requested_agent_type": request.agent_type,
        "requested_priority": request.priority,
        "coordinator_agent_id": coordinator_id,
        "worker_agent_ids": worker_ids,
        "active_worker_ids": [],
        "planner_output": Value::Null,
        "execution_plan": Value::Null,
        "routing_history": [],
        "team_sizing": Value::Null,
        "step_results": [],
    });

    if let Some(conversation) = &request.conversation {
        put_metadata(&mut metadata, "session_id", json!(conversation.session_id));
        put_metadata(&mut metadata, "turn_index", json!(conversation.turn_index));
        put_metadata(
            &mut metadata,
            "retained_context",
            conversation.retained_context.clone(),
        );
    }

    metadata
}

fn put_metadata(metadata: &mut Value, key: &str, value: Value) {
    if let Some(object) = metadata.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}

fn priority_rank(priority: Option<&str>) -> i32 {
    match priority
        .unwrap_or("normal")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "0" | "urgent" => 0,
        "1" | "high" => 1,
        "2" | "normal" | "medium" => 2,
        "3" | "low" => 3,
        "4" | "background" => 4,
        _ => 2,
    }
}

fn coordinator_id_for_request(request: &TaskSubmissionRequest, fallback: AgentId) -> AgentId {
    request
        .conversation
        .as_ref()
        .map(|conversation| conversation.coordinator_agent_id)
        .unwrap_or(fallback)
}

fn coordinator_id_from_metadata(metadata: &Value) -> Option<AgentId> {
    metadata
        .get("coordinator_agent_id")
        .and_then(Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .map(AgentId::from_uuid)
}

fn normalize_runtime_plan(goal: &str, context: &Value, raw_plan: Value) -> Value {
    let mut plan = normalize_planner_output(goal, context, raw_plan);
    let steps = plan
        .get("steps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut runtime_steps = Vec::new();
    let mut previous_join_id: Option<String> = None;
    let mut root_ids = Vec::new();

    for (index, raw_step) in steps.iter().enumerate() {
        let mut step = raw_step.as_object().cloned().unwrap_or_default();
        let step_id = step
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("step-{}", index + 1));
        step.insert("id".to_string(), json!(step_id.clone()));
        step.entry("step".to_string())
            .or_insert_with(|| json!(index + 1));
        step.entry("action".to_string())
            .or_insert_with(|| json!("execute"));
        step.entry("description".to_string())
            .or_insert_with(|| json!(goal));

        if index == 0 {
            step.insert("role".to_string(), json!("worker"));
            step.insert("branch".to_string(), json!("branch-a"));
            step.insert("depends_on".to_string(), json!([]));
            root_ids.push(step_id.clone());
        } else if index == 1 {
            step.insert("role".to_string(), json!("worker"));
            step.insert("branch".to_string(), json!("branch-b"));
            step.insert("depends_on".to_string(), json!([]));
            root_ids.push(step_id.clone());
        } else {
            step.insert("role".to_string(), json!("coordinator"));
            step.insert("branch".to_string(), json!("join"));
            let deps = if let Some(previous_join_id) = &previous_join_id {
                vec![previous_join_id.clone()]
            } else {
                root_ids.clone()
            };
            step.insert("depends_on".to_string(), json!(deps));
            previous_join_id = Some(step_id.clone());
        }

        runtime_steps.push(Value::Object(step));
    }

    if runtime_steps.len() == 2 {
        runtime_steps.push(json!({
            "id": "join-step",
            "step": 3,
            "action": "synthesize",
            "description": format!(
                "Synthesize both worker branch results into one final answer for: {goal}"
            ),
            "role": "coordinator",
            "branch": "join",
            "depends_on": root_ids,
        }));
    }

    if let Some(object) = plan.as_object_mut() {
        object.insert("goal".to_string(), json!(goal));
        object.insert("context".to_string(), context.clone());
        object.insert("steps".to_string(), Value::Array(runtime_steps.clone()));
        object.insert(
            "topology_hint".to_string(),
            json!(if runtime_steps.len() >= 3 {
                "hybrid"
            } else {
                "sequential"
            }),
        );
        object.insert(
            "runtime_normalized".to_string(),
            json!(runtime_steps.len() >= 2),
        );
    }

    plan
}

fn task_assignment_for_node(
    workflow_id: TaskId,
    node: &mister_smith_agents::ExecutionNode,
    goal: &str,
) -> TaskAssignment {
    TaskAssignment {
        task_id: TaskId::from_uuid(*node.node_id.as_ref()),
        task_type: node.action.clone(),
        priority: 128,
        deadline: None,
        input: json!({
            "workflow_id": workflow_id,
            "goal": goal,
            "step_id": node.step_key,
            "description": node.description,
            "action": node.action,
            "branch_id": node.branch_id,
            "dependencies": node.dependencies,
            "planner_metadata": node.metadata,
        }),
        output: None,
        state: TaskState::Pending,
        assigned_to: None,
        parent_task_id: Some(workflow_id),
        team_id: None,
        message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        assigned_at: None,
        completed_at: None,
        error_message: None,
    }
}

fn execution_input_for_task(
    graph: &mister_smith_agents::ExecutionGraph,
    task: &TaskAssignment,
    goal: &str,
    worker_id: AgentId,
    step_results: &BTreeMap<String, Value>,
) -> Value {
    let node_id = mister_smith_core::ExecutionNodeId::from_uuid(*task.task_id.as_ref());
    let dependency_results = graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .map(|node| {
            node.dependencies
                .iter()
                .filter_map(|dependency| {
                    let key = TaskId::from_uuid(*dependency.as_ref()).to_string();
                    step_results.get(&key).cloned()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "workflow_goal": goal,
        "provider_kind": PROVIDER_KIND_NAME,
        "model_id": MODEL_ID,
        "worker_id": worker_id,
        "task": task.input,
        "dependency_results": dependency_results,
    })
}

async fn execute_prepared_decision(
    router: Arc<ModelRouter>,
    workflow_id: TaskId,
    prepared: PreparedDecisionExecution,
) -> Result<Vec<CompletedTaskExecution>, FailedTaskExecution> {
    let mut completed_steps = Vec::new();

    for prepared_task in prepared.tasks {
        let task_id = prepared_task.task.task_id;
        let worker_id = prepared_task.worker_id;
        let branch_id = prepared_task
            .task
            .input
            .get("branch_id")
            .cloned()
            .unwrap_or(Value::Null);
        let action = prepared_task.task.task_type.clone();

        info!(
            workflow_id = %workflow_id,
            task_id = %task_id,
            worker_id = %worker_id,
            provider_kind = PROVIDER_KIND_NAME,
            model_id = MODEL_ID,
            "Executing workflow step"
        );

        let mut executor = ExecutorAgent::with_router(worker_id, router.clone());
        let mut executor_state = ExecutorState::default();
        let result = match executor
            .handle_message(
                ExecutorMessage::ExecutePlan {
                    plan: prepared_task.execution_input,
                    managed_context: None,
                },
                &mut executor_state,
            )
            .await
        {
            Ok(result) => result,
            Err(error) => {
                return Err(FailedTaskExecution {
                    completed_steps,
                    task_id,
                    worker_id,
                    branch_id: branch_id.clone(),
                    message: format!("worker {worker_id} failed task {task_id}: {error}"),
                });
            }
        };

        completed_steps.push(CompletedTaskExecution {
            task_id,
            worker_id,
            branch_id,
            action,
            result,
        });
    }

    Ok(completed_steps)
}
