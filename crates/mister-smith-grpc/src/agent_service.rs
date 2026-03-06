//! Agent service — gRPC service for agent management and task execution.
//!
//! Provides methods matching the `AgentService` proto definition:
//! - `list_agents` — list active agents with optional filters
//! - `get_agent` — retrieve a specific agent by ID
//! - `submit_task` — submit a task for execution
//! - `get_task_result` — retrieve a task's result
//! - `stream_agent_status` — server-streaming status updates
//! - `agent_channel` — bidirectional message envelope streaming

use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

use crate::proto::{
    AgentAvailability, AgentInfo, AgentStatusUpdate, GetAgentRequest, GetTaskResultRequest,
    ListAgentsRequest, ListAgentsResponse, MessageEnvelope, StreamAgentStatusRequest,
    SubmitTaskRequest, SubmitTaskResponse, TaskResult, TaskStatus,
};

/// Type alias for server-streaming response streams.
pub type AgentStatusStream =
    Pin<Box<dyn Stream<Item = Result<AgentStatusUpdate, Status>> + Send + 'static>>;

/// Type alias for bidirectional envelope streams.
pub type EnvelopeStream =
    Pin<Box<dyn Stream<Item = Result<MessageEnvelope, Status>> + Send + 'static>>;

/// Mock agent registry for testing and initial implementation.
///
/// In production this would be backed by a real registry (e.g., NATS KV store
/// or an actor system query). The mock provides deterministic behavior for
/// integration tests.
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<Vec<AgentInfo>>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    /// Create a new registry pre-populated with mock agents.
    #[must_use]
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = prost_types::Timestamp {
            seconds: now.as_secs() as i64,
            nanos: 0,
        };

        let agents = vec![
            AgentInfo {
                agent_id: "agent-001".to_string(),
                agent_type: "worker".to_string(),
                availability: AgentAvailability::Idle as i32,
                active_tasks: 0,
                started_at: Some(timestamp),
                load: 0.0,
            },
            AgentInfo {
                agent_id: "agent-002".to_string(),
                agent_type: "coordinator".to_string(),
                availability: AgentAvailability::Busy as i32,
                active_tasks: 3,
                started_at: Some(timestamp),
                load: 0.65,
            },
            AgentInfo {
                agent_id: "agent-003".to_string(),
                agent_type: "worker".to_string(),
                availability: AgentAvailability::Idle as i32,
                active_tasks: 1,
                started_at: Some(timestamp),
                load: 0.15,
            },
        ];

        Self {
            agents: Arc::new(RwLock::new(agents)),
        }
    }

    /// Look up an agent by ID.
    async fn find_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.iter().find(|a| a.agent_id == agent_id).cloned()
    }

    /// List agents matching optional filters.
    async fn list_agents(
        &self,
        filter_availability: Option<i32>,
        filter_type: Option<&str>,
    ) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .iter()
            .filter(|a| {
                if let Some(avail) = filter_availability {
                    if a.availability != avail {
                        return false;
                    }
                }
                if let Some(agent_type) = filter_type {
                    if a.agent_type != agent_type {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }
}

/// Implementation of the AgentService gRPC server.
///
/// This struct holds the service state and provides async methods corresponding
/// to each RPC in the proto definition. It is designed to be composed into a
/// tonic `Server` via the [`crate::server`] module once service codegen is
/// available.
#[derive(Debug, Clone)]
pub struct AgentServiceImpl {
    registry: AgentRegistry,
}

impl Default for AgentServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentServiceImpl {
    /// Create a new `AgentServiceImpl` with a default mock registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
        }
    }

    /// Create a new `AgentServiceImpl` with the provided registry.
    #[must_use]
    pub fn with_registry(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// List all active agents, optionally filtered by availability or type.
    pub async fn list_agents(
        &self,
        request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            filter_availability = ?req.filter_availability,
            filter_type = ?req.filter_type,
            "listing agents"
        );

        let agents = self
            .registry
            .list_agents(req.filter_availability, req.filter_type.as_deref())
            .await;

        info!(count = agents.len(), "listed agents");
        Ok(Response::new(ListAgentsResponse { agents }))
    }

    /// Get a specific agent's details by ID.
    pub async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<AgentInfo>, Status> {
        let agent_id = request.into_inner().agent_id;
        debug!(agent_id = %agent_id, "getting agent");

        match self.registry.find_agent(&agent_id).await {
            Some(info) => Ok(Response::new(info)),
            None => {
                warn!(agent_id = %agent_id, "agent not found");
                Err(Status::not_found(format!("agent not found: {agent_id}")))
            }
        }
    }

    /// Submit a task for execution by an available agent.
    pub async fn submit_task(
        &self,
        request: Request<SubmitTaskRequest>,
    ) -> Result<Response<SubmitTaskResponse>, Status> {
        let req = request.into_inner();
        let task = req
            .task
            .ok_or_else(|| Status::invalid_argument("task assignment is required"))?;

        let task_id = if task.task_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            task.task_id.clone()
        };

        // Pick the first idle agent, or fall back to the first available.
        let agents = self.registry.list_agents(None, None).await;
        let assigned = task
            .assigned_agent
            .clone()
            .or_else(|| {
                agents
                    .iter()
                    .find(|a| a.availability == AgentAvailability::Idle as i32)
                    .map(|a| a.agent_id.clone())
            })
            .or_else(|| agents.first().map(|a| a.agent_id.clone()))
            .unwrap_or_else(|| "unassigned".to_string());

        info!(task_id = %task_id, assigned_agent = %assigned, "task submitted");

        Ok(Response::new(SubmitTaskResponse {
            task_id,
            assigned_agent_id: assigned,
        }))
    }

    /// Get the result of a previously submitted task.
    pub async fn get_task_result(
        &self,
        request: Request<GetTaskResultRequest>,
    ) -> Result<Response<TaskResult>, Status> {
        let task_id = request.into_inner().task_id;
        debug!(task_id = %task_id, "getting task result");

        // Mock: return a success result for known task IDs, NOT_FOUND otherwise.
        if task_id.is_empty() {
            return Err(Status::invalid_argument("task_id is required"));
        }

        // In a real implementation this would look up a task store. For now,
        // return a synthetic success result for any non-empty task ID.
        let result = TaskResult {
            task_id: task_id.clone(),
            status: TaskStatus::Success as i32,
            result: None,
            error: None,
            duration_ms: 42,
            agent_id: "agent-001".to_string(),
        };

        info!(task_id = %task_id, "returned task result");
        Ok(Response::new(result))
    }

    /// Server-streaming: yield periodic agent status updates.
    pub async fn stream_agent_status(
        &self,
        request: Request<StreamAgentStatusRequest>,
    ) -> Result<Response<AgentStatusStream>, Status> {
        let req = request.into_inner();
        let filter_agent_id = req.agent_id.clone();
        let registry = self.registry.clone();

        info!(filter_agent_id = ?filter_agent_id, "starting agent status stream");

        let stream = async_stream::try_stream! {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            // Emit 5 updates then close the stream (mock behavior).
            for i in 0..5u32 {
                interval.tick().await;

                let agents = registry.list_agents(None, None).await;
                for agent in &agents {
                    if let Some(ref filter) = filter_agent_id {
                        if agent.agent_id != *filter {
                            continue;
                        }
                    }

                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default();

                    yield AgentStatusUpdate {
                        agent_id: agent.agent_id.clone(),
                        availability: agent.availability,
                        timestamp: Some(prost_types::Timestamp {
                            seconds: now.as_secs() as i64,
                            nanos: 0,
                        }),
                        load: Some(agent.load + (i as f64 * 0.01)),
                        active_tasks: Some(agent.active_tasks + i),
                    };
                }
            }
        };

        Ok(Response::new(Box::pin(stream) as AgentStatusStream))
    }

    /// Bidirectional streaming: echo received envelopes back with updated metadata.
    pub async fn agent_channel(
        &self,
        request: Request<tonic::Streaming<MessageEnvelope>>,
    ) -> Result<Response<EnvelopeStream>, Status> {
        let mut inbound = request.into_inner();

        info!("agent channel opened");

        let stream = async_stream::try_stream! {
            while let Some(envelope) = inbound.next().await {
                let mut envelope = envelope?;
                // Echo back with a new message_id and swapped source/target.
                let original_source = envelope.source_agent_id.take();
                let original_target = envelope.target_agent_id.take();
                envelope.message_id = uuid::Uuid::new_v4().to_string();
                envelope.source_agent_id = original_target;
                envelope.target_agent_id = original_source;

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();
                envelope.timestamp = Some(prost_types::Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: 0,
                });

                yield envelope;
            }
        };

        Ok(Response::new(Box::pin(stream) as EnvelopeStream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_agents_returns_all() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .list_agents(Request::new(ListAgentsRequest {
                filter_availability: None,
                filter_type: None,
            }))
            .await
            .unwrap();
        assert_eq!(resp.into_inner().agents.len(), 3);
    }

    #[tokio::test]
    async fn list_agents_filter_by_availability() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .list_agents(Request::new(ListAgentsRequest {
                filter_availability: Some(AgentAvailability::Idle as i32),
                filter_type: None,
            }))
            .await
            .unwrap();
        let agents = resp.into_inner().agents;
        assert_eq!(agents.len(), 2);
        for agent in &agents {
            assert_eq!(agent.availability, AgentAvailability::Idle as i32);
        }
    }

    #[tokio::test]
    async fn list_agents_filter_by_type() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .list_agents(Request::new(ListAgentsRequest {
                filter_availability: None,
                filter_type: Some("coordinator".to_string()),
            }))
            .await
            .unwrap();
        let agents = resp.into_inner().agents;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_type, "coordinator");
    }

    #[tokio::test]
    async fn get_agent_found() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .get_agent(Request::new(GetAgentRequest {
                agent_id: "agent-002".to_string(),
            }))
            .await
            .unwrap();
        let info = resp.into_inner();
        assert_eq!(info.agent_id, "agent-002");
        assert_eq!(info.agent_type, "coordinator");
    }

    #[tokio::test]
    async fn get_agent_not_found() {
        let svc = AgentServiceImpl::new();
        let err = svc
            .get_agent(Request::new(GetAgentRequest {
                agent_id: "nonexistent".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        assert!(err.message().contains("nonexistent"));
    }

    #[tokio::test]
    async fn submit_task_with_assignment() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .submit_task(Request::new(SubmitTaskRequest {
                task: Some(crate::proto::TaskAssignment {
                    task_id: "task-123".to_string(),
                    task_type: "analysis".to_string(),
                    payload: None,
                    priority: crate::proto::MessagePriority::High as i32,
                    deadline: None,
                    assigned_agent: Some("agent-002".to_string()),
                    requester_id: "user-1".to_string(),
                    metadata: Default::default(),
                }),
            }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.task_id, "task-123");
        assert_eq!(inner.assigned_agent_id, "agent-002");
    }

    #[tokio::test]
    async fn submit_task_auto_assign() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .submit_task(Request::new(SubmitTaskRequest {
                task: Some(crate::proto::TaskAssignment {
                    task_id: "".to_string(),
                    task_type: "analysis".to_string(),
                    payload: None,
                    priority: crate::proto::MessagePriority::Normal as i32,
                    deadline: None,
                    assigned_agent: None,
                    requester_id: "user-1".to_string(),
                    metadata: Default::default(),
                }),
            }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        // Auto-generated UUID task ID.
        assert!(!inner.task_id.is_empty());
        // Auto-assigned to first idle agent.
        assert_eq!(inner.assigned_agent_id, "agent-001");
    }

    #[tokio::test]
    async fn submit_task_missing_body() {
        let svc = AgentServiceImpl::new();
        let err = svc
            .submit_task(Request::new(SubmitTaskRequest { task: None }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn get_task_result_found() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .get_task_result(Request::new(GetTaskResultRequest {
                task_id: "task-123".to_string(),
            }))
            .await
            .unwrap();
        let result = resp.into_inner();
        assert_eq!(result.task_id, "task-123");
        assert_eq!(result.status, TaskStatus::Success as i32);
    }

    #[tokio::test]
    async fn get_task_result_empty_id() {
        let svc = AgentServiceImpl::new();
        let err = svc
            .get_task_result(Request::new(GetTaskResultRequest {
                task_id: "".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn stream_agent_status_emits_updates() {
        let svc = AgentServiceImpl::new();
        let resp = svc
            .stream_agent_status(Request::new(StreamAgentStatusRequest {
                agent_id: Some("agent-001".to_string()),
            }))
            .await
            .unwrap();

        let mut stream = resp.into_inner();
        let mut count = 0;
        while let Some(item) = stream.next().await {
            let update = item.unwrap();
            assert_eq!(update.agent_id, "agent-001");
            assert!(update.timestamp.is_some());
            count += 1;
        }
        // Mock emits 5 rounds, filtering to agent-001 should yield 5 updates.
        assert_eq!(count, 5);
    }

    // NOTE: The `agent_channel` bidirectional streaming method requires a
    // `tonic::Streaming<MessageEnvelope>` which is difficult to construct in
    // unit tests without a running gRPC server. Full bidirectional streaming
    // is validated in integration tests with an actual tonic server.
    //
    // The compilation of `AgentServiceImpl::agent_channel` is itself the
    // primary assertion that the method signature is correct.
}
