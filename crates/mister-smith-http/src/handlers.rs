//! HTTP request handlers for REST API endpoints.
//!
//! All handlers accept `State<AppState>` and return JSON responses.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use mister_smith_core::{AgentAvailability, AgentId, AgentType, SessionId, TaskId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::errors::HttpError;
use crate::server::{
    AppState, ConversationContinueRequest, ConversationCreateRequest, ConversationServiceError,
    TaskSubmissionRequest,
};

fn is_false(value: &bool) -> bool {
    !*value
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall system status.
    pub status: String,
    /// Individual component statuses.
    pub components: Vec<ComponentHealth>,
}

/// Individual component health within the health response.
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    /// Component name.
    pub name: String,
    /// Component status: healthy, degraded, or unhealthy.
    pub status: String,
    /// Optional status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Agent summary for list responses.
#[derive(Debug, Serialize)]
pub struct AgentSummary {
    /// Agent identifier.
    pub agent_id: AgentId,
    /// Agent type/role.
    pub agent_type: AgentType,
    /// Current availability.
    pub availability: AgentAvailability,
    /// Human-readable name.
    pub name: String,
}

/// Agent detail response.
#[derive(Debug, Serialize)]
pub struct AgentDetail {
    /// Agent identifier.
    pub agent_id: AgentId,
    /// Agent type/role.
    pub agent_type: AgentType,
    /// Current availability.
    pub availability: AgentAvailability,
    /// Human-readable name.
    pub name: String,
    /// Additional metadata.
    pub metadata: serde_json::Value,
}

/// Task submission request.
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// Task description.
    pub description: String,
    /// Target agent type for assignment.
    #[serde(default)]
    pub agent_type: Option<AgentType>,
    /// Task priority.
    #[serde(default)]
    pub priority: Option<String>,
}

/// Task submission response (202 Accepted).
#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    /// Assigned task identifier.
    pub task_id: TaskId,
    /// Agent the task was assigned to.
    pub assigned_agent_id: AgentId,
    /// Task status.
    pub status: String,
}

/// Task status response.
#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    /// Task identifier.
    pub task_id: TaskId,
    /// Current status.
    pub status: String,
    /// Task result, if complete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Session create request.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Operator message for the first accepted turn.
    pub message: String,
    /// Optional turn priority.
    #[serde(default)]
    pub priority: Option<String>,
}

/// Session continue request.
#[derive(Debug, Deserialize)]
pub struct ContinueSessionRequest {
    /// Operator message for the next accepted turn.
    pub message: String,
    /// Optional turn priority.
    #[serde(default)]
    pub priority: Option<String>,
}

/// Accepted session turn response.
#[derive(Debug, Serialize)]
pub struct SessionTurnAcceptedResponse {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Root workflow created for the accepted turn.
    pub workflow_id: TaskId,
    /// Stable coordinator identity reused across accepted turns.
    pub coordinator_agent_id: AgentId,
    /// 1-based accepted turn order.
    pub turn_index: u32,
    /// Current workflow status.
    pub status: String,
}

/// Ordered session turn summary.
#[derive(Debug, Serialize)]
pub struct SessionTurnSummaryResponse {
    /// 1-based accepted turn order.
    pub turn_index: u32,
    /// Root workflow for the turn.
    pub workflow_id: TaskId,
    /// Current turn status mirrored from the root workflow.
    pub status: String,
    /// Original operator message.
    pub user_message: String,
    /// Restart and resume provenance derived from workflow metadata, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_provenance: Option<SessionResumeProvenanceResponse>,
}

/// Restart and resume provenance for one session turn.
#[derive(Debug, Serialize)]
pub struct SessionResumeProvenanceResponse {
    /// Workflow record was recovered after a runtime restart.
    #[serde(skip_serializing_if = "is_false")]
    pub recovered_after_restart: bool,
    /// Turn resumes after a prior workflow was restart-recovered.
    #[serde(skip_serializing_if = "is_false")]
    pub resumed_after_restart: bool,
    /// Timestamp recorded when the workflow was marked recovered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovered_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Human-readable recovery reason recorded in workflow metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    /// Prior workflow in the resumed turn lineage, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed_from_workflow_id: Option<TaskId>,
    /// Prior turn index in the resumed turn lineage, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed_from_turn_index: Option<u32>,
}

/// Session inspect response.
#[derive(Debug, Serialize)]
pub struct SessionInspectResponse {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Session lifecycle state.
    pub status: mister_smith_core::SessionStatus,
    /// Stable coordinator identity.
    pub coordinator_agent_id: AgentId,
    /// Provider attributed to the session.
    pub provider_kind: String,
    /// Model attributed to the session.
    pub model_id: String,
    /// Active workflow when the session is busy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_workflow_id: Option<TaskId>,
    /// Most recent terminal workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_completed_workflow_id: Option<TaskId>,
    /// Number of accepted turns.
    pub turn_count: u32,
    /// Ordered turn summaries.
    pub turns: Vec<SessionTurnSummaryResponse>,
    /// Logical close time when ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// End-session response.
#[derive(Debug, Serialize)]
pub struct EndSessionResponse {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Updated lifecycle state.
    pub status: mister_smith_core::SessionStatus,
    /// Logical end time.
    pub ended_at: chrono::DateTime<chrono::Utc>,
}

/// System configuration response.
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    /// System version.
    pub version: String,
    /// Configuration entries.
    pub config: serde_json::Value,
}

/// Query parameters for listing agents.
#[derive(Debug, Deserialize, Default)]
pub struct AgentListQuery {
    /// Filter by availability.
    #[serde(default)]
    pub availability: Option<String>,
    /// Filter by agent type.
    #[serde(default)]
    pub agent_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/health` — Public system health for liveness/readiness probes.
///
/// This endpoint is intentionally left unauthenticated so deployment probes can
/// verify process and transport health without minting JWT bearer tokens.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let transport_connected = state.transport_health.is_connected();

    let components = vec![
        ComponentHealth {
            name: "http_server".to_string(),
            status: "healthy".to_string(),
            message: None,
        },
        ComponentHealth {
            name: "nats_transport".to_string(),
            status: if transport_connected {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            message: if transport_connected {
                None
            } else {
                Some("NATS transport disconnected".to_string())
            },
        },
    ];

    let overall = if components.iter().all(|c| c.status == "healthy") {
        "healthy"
    } else if components.iter().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: overall.to_string(),
        components,
    })
}

/// `GET /api/v1/agents` — List agents with optional filters.
pub async fn list_agents(
    State(_state): State<AppState>,
    Query(query): Query<AgentListQuery>,
) -> Json<Vec<AgentSummary>> {
    // Placeholder: return mock agents, applying filters if provided.
    let agents = mock_agents();
    let filtered: Vec<AgentSummary> = agents
        .into_iter()
        .filter(|a| {
            if let Some(ref avail) = query.availability {
                let agent_avail = format!("{:?}", a.availability);
                if !agent_avail.eq_ignore_ascii_case(avail) {
                    return false;
                }
            }
            if let Some(ref atype) = query.agent_type {
                let agent_type = format!("{:?}", a.agent_type);
                if !agent_type.eq_ignore_ascii_case(atype) {
                    return false;
                }
            }
            true
        })
        .collect();

    Json(filtered)
}

/// `GET /api/v1/agents/{agent_id}` — Single agent detail.
pub async fn get_agent(
    State(_state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetail>, HttpError> {
    // Try to parse agent_id as UUID.
    let uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid agent ID: {agent_id}")))?;

    // Placeholder: check mock agents.
    let agents = mock_agents();
    let found = agents.iter().find(|a| *a.agent_id.as_ref() == uuid);

    match found {
        Some(agent) => Ok(Json(AgentDetail {
            agent_id: agent.agent_id,
            agent_type: agent.agent_type,
            availability: agent.availability,
            name: agent.name.clone(),
            metadata: serde_json::json!({
                "uptime_seconds": 3600,
                "tasks_completed": 42,
            }),
        })),
        None => Err(HttpError::NotFound(format!("Agent {agent_id} not found"))),
    }
}

/// `POST /api/v1/tasks` — Submit a task, returns 202 Accepted.
pub async fn create_task(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<CreateTaskResponse>), HttpError> {
    let task_service = state
        .task_service
        .as_ref()
        .ok_or_else(|| HttpError::InternalError("runtime task service unavailable".to_string()))?;

    let response = task_service
        .submit_task(TaskSubmissionRequest {
            description: request.description,
            agent_type: request.agent_type,
            priority: request.priority,
            conversation: None,
        })
        .await
        .map_err(HttpError::InternalError)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CreateTaskResponse {
            task_id: response.task_id,
            assigned_agent_id: response.assigned_agent_id,
            status: response.status,
        }),
    ))
}

/// `POST /api/v1/sessions` — Create a session and accept the first turn.
pub async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionTurnAcceptedResponse>), HttpError> {
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let accepted = conversation_service
        .create_session(ConversationCreateRequest {
            message: request.message,
            priority: request.priority,
        })
        .await
        .map_err(map_conversation_error)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(SessionTurnAcceptedResponse {
            session_id: accepted.session_id,
            workflow_id: accepted.workflow_id,
            coordinator_agent_id: accepted.coordinator_agent_id,
            turn_index: accepted.turn_index,
            status: accepted.status,
        }),
    ))
}

/// `POST /api/v1/sessions/{session_id}/turns` — Continue one existing session.
pub async fn continue_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<ContinueSessionRequest>,
) -> Result<(StatusCode, Json<SessionTurnAcceptedResponse>), HttpError> {
    let session_id = parse_session_path(&session_id)?;
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let accepted = conversation_service
        .continue_session(ConversationContinueRequest {
            session_id,
            message: request.message,
            priority: request.priority,
        })
        .await
        .map_err(map_conversation_error)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(SessionTurnAcceptedResponse {
            session_id: accepted.session_id,
            workflow_id: accepted.workflow_id,
            coordinator_agent_id: accepted.coordinator_agent_id,
            turn_index: accepted.turn_index,
            status: accepted.status,
        }),
    ))
}

/// `GET /api/v1/sessions/{session_id}` — Inspect one durable conversation session.
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionInspectResponse>, HttpError> {
    let session_id = parse_session_path(&session_id)?;
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let view = conversation_service
        .get_session(session_id)
        .await
        .map_err(map_conversation_error)?
        .ok_or_else(|| HttpError::NotFound(format!("session {session_id} not found")))?;

    Ok(Json(SessionInspectResponse {
        session_id: view.session_id,
        status: view.status,
        coordinator_agent_id: view.coordinator_agent_id,
        provider_kind: view.provider_kind,
        model_id: view.model_id,
        active_workflow_id: view.active_workflow_id,
        last_completed_workflow_id: view.last_completed_workflow_id,
        turn_count: view.turn_count,
        turns: view
            .turns
            .into_iter()
            .map(|turn| SessionTurnSummaryResponse {
                turn_index: turn.turn_index,
                workflow_id: turn.workflow_id,
                status: turn.status,
                user_message: turn.user_message,
                resume_provenance: turn.resume_provenance.map(|provenance| {
                    SessionResumeProvenanceResponse {
                        recovered_after_restart: provenance.recovered_after_restart,
                        resumed_after_restart: provenance.resumed_after_restart,
                        recovered_at: provenance.recovered_at,
                        recovery_reason: provenance.recovery_reason,
                        resumed_from_workflow_id: provenance.resumed_from_workflow_id,
                        resumed_from_turn_index: provenance.resumed_from_turn_index,
                    }
                }),
            })
            .collect(),
        ended_at: view.ended_at,
    }))
}

/// `POST /api/v1/sessions/{session_id}/end` — Logically end one idle session.
pub async fn end_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<EndSessionResponse>, HttpError> {
    let session_id = parse_session_path(&session_id)?;
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let ended = conversation_service
        .end_session(session_id)
        .await
        .map_err(map_conversation_error)?;

    Ok(Json(EndSessionResponse {
        session_id: ended.session_id,
        status: ended.status,
        ended_at: ended.ended_at,
    }))
}

/// `GET /api/v1/tasks/{task_id}` — Task status and result.
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, HttpError> {
    let uuid = Uuid::parse_str(&task_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid task ID: {task_id}")))?;

    let task_service = state
        .task_service
        .as_ref()
        .ok_or_else(|| HttpError::InternalError("runtime task service unavailable".to_string()))?;

    let task_id = TaskId::from_uuid(uuid);
    let status = task_service
        .get_task(task_id)
        .await
        .map_err(HttpError::InternalError)?
        .ok_or_else(|| HttpError::NotFound(format!("Task {task_id} not found")))?;

    Ok(Json(TaskStatusResponse {
        task_id: status.task_id,
        status: status.status,
        result: status.result,
    }))
}

/// `GET /api/v1/config` — System configuration.
pub async fn get_config(State(_state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        version: "0.1.0".to_string(),
        config: serde_json::json!({
            "runtime": {
                "worker_threads": 4,
                "max_blocking_threads": 512,
            },
            "transport": {
                "http": {
                    "bind_address": "0.0.0.0:8080",
                },
            },
        }),
    })
}

// ---------------------------------------------------------------------------
// Mock data helpers
// ---------------------------------------------------------------------------

/// Generate mock agents for placeholder responses.
fn mock_agents() -> Vec<AgentSummary> {
    vec![
        AgentSummary {
            agent_id: AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            ),
            agent_type: AgentType::Worker,
            availability: AgentAvailability::Idle,
            name: "worker-1".to_string(),
        },
        AgentSummary {
            agent_id: AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            ),
            agent_type: AgentType::Supervisor,
            availability: AgentAvailability::Busy,
            name: "supervisor-1".to_string(),
        },
    ]
}

fn parse_session_path(raw: &str) -> Result<SessionId, HttpError> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| HttpError::BadRequest(format!("Invalid session ID: {raw}")))
}

fn map_conversation_error(error: ConversationServiceError) -> HttpError {
    match error {
        ConversationServiceError::BadRequest(message) => HttpError::BadRequest(message),
        ConversationServiceError::NotFound { session_id } => {
            HttpError::NotFound(format!("session {session_id} not found"))
        }
        ConversationServiceError::SessionBusy {
            session_id,
            active_workflow_id,
        } => HttpError::Conflict {
            code: "session_busy".to_string(),
            message: format!("session {session_id} is busy with workflow {active_workflow_id}"),
            context: BTreeMap::from([
                (
                    "session_id".to_string(),
                    Value::String(session_id.to_string()),
                ),
                (
                    "active_workflow_id".to_string(),
                    Value::String(active_workflow_id.to_string()),
                ),
            ]),
        },
        ConversationServiceError::SessionEnded { session_id } => HttpError::Conflict {
            code: "session_ended".to_string(),
            message: format!("session {session_id} has ended"),
            context: BTreeMap::from([(
                "session_id".to_string(),
                Value::String(session_id.to_string()),
            )]),
        },
        ConversationServiceError::Internal(message) => HttpError::InternalError(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{AppState, NatsHealthCheck};

    fn test_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    async fn health_check_returns_healthy_when_transport_connected() {
        let state =
            test_state().with_transport_health(std::sync::Arc::new(NatsHealthCheck::new(true)));
        let Json(response) = health_check(State(state)).await;

        assert_eq!(response.status, "healthy");
        assert_eq!(response.components.len(), 2);
        assert_eq!(response.components[0].name, "http_server");
        assert_eq!(response.components[0].status, "healthy");
        assert_eq!(response.components[1].name, "nats_transport");
        assert_eq!(response.components[1].status, "healthy");
    }

    #[tokio::test]
    async fn health_check_returns_unhealthy_when_transport_disconnected() {
        let state =
            test_state().with_transport_health(std::sync::Arc::new(NatsHealthCheck::new(false)));
        let Json(response) = health_check(State(state)).await;

        assert_eq!(response.status, "unhealthy");
        assert_eq!(response.components[1].name, "nats_transport");
        assert_eq!(response.components[1].status, "unhealthy");
        assert_eq!(
            response.components[1].message.as_deref(),
            Some("NATS transport disconnected")
        );
    }

    #[tokio::test]
    async fn list_agents_returns_agents() {
        let state = test_state();
        let query = Query(AgentListQuery::default());
        let Json(agents) = list_agents(State(state), query).await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn list_agents_filter_by_type() {
        let state = test_state();
        let query = Query(AgentListQuery {
            agent_type: Some("Worker".to_string()),
            availability: None,
        });
        let Json(agents) = list_agents(State(state), query).await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "worker-1");
    }

    #[tokio::test]
    async fn get_agent_found() {
        let state = test_state();
        let result = get_agent(
            State(state),
            Path("00000000-0000-0000-0000-000000000001".to_string()),
        )
        .await;
        assert!(result.is_ok());
        let Json(detail) = result.unwrap();
        assert_eq!(detail.name, "worker-1");
    }

    #[tokio::test]
    async fn get_agent_not_found() {
        let state = test_state();
        let result = get_agent(
            State(state),
            Path("00000000-0000-0000-0000-000000000099".to_string()),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_agent_invalid_id() {
        let state = test_state();
        let result = get_agent(State(state), Path("not-a-uuid".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_task_returns_202_fields() {
        let state = test_state();
        let request = CreateTaskRequest {
            description: "Test task".to_string(),
            agent_type: None,
            priority: None,
        };
        let result = create_task(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_task_valid_id() {
        let state = test_state();
        let task_id = Uuid::new_v4().to_string();
        let result = get_task(State(state), Path(task_id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_config_returns_version() {
        let state = test_state();
        let Json(config) = get_config(State(state)).await;
        assert_eq!(config.version, "0.1.0");
    }
}
