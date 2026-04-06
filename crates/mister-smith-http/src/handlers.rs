//! HTTP request handlers for REST API endpoints.
//!
//! All handlers accept `State<AppState>` and return JSON responses.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use mister_smith_core::{
    AgentAvailability, AgentId, AgentType, DurableWorkflowLifecycleState,
    DurableWorkflowLifecycleVerb, ExternalDelegationEnvelope, LifecycleDecisionOutcome, SessionId,
    TaskId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::errors::HttpError;
use crate::server::{
    AppState, ConversationContinueRequest, ConversationCreateRequest, ConversationServiceError,
    ConversationSessionControlUpdateRequest, SessionListRequest, TaskListRequest,
    TaskSubmissionRequest,
};

fn is_false(value: &bool) -> bool {
    !*value
}

const DEFAULT_COLLECTION_LIMIT: usize = 50;
const MAX_COLLECTION_LIMIT: usize = 200;

/// Optional external delegation envelope attached by transport auth middleware.
pub struct ExternalDelegationBoundary(pub Option<ExternalDelegationEnvelope>);

impl<S> axum::extract::FromRequestParts<S> for ExternalDelegationBoundary
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<ExternalDelegationEnvelope>()
                .cloned(),
        ))
    }
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
    /// Raw persisted lifecycle status from the registry.
    pub status: String,
    /// Latest recorded heartbeat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
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
    /// Raw persisted lifecycle status from the registry.
    pub status: String,
    /// Latest recorded heartbeat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata.
    pub metadata: serde_json::Value,
}

/// Root workflow collection query parameters.
#[derive(Debug, Deserialize, Default)]
pub struct TaskListQuery {
    /// Optional task status filter.
    #[serde(default)]
    pub status: Option<String>,
    /// Max rows to return.
    #[serde(default)]
    pub limit: Option<usize>,
    /// Rows to skip from the head of the result set.
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Root workflow summary for list responses.
#[derive(Debug, Serialize)]
pub struct TaskSummaryResponse {
    /// Stable workflow identifier.
    pub task_id: TaskId,
    /// Persisted workflow status.
    pub status: String,
    /// Durable lifecycle meaning projected for operator-facing views.
    pub lifecycle_state: DurableWorkflowLifecycleState,
    /// Persisted numeric priority.
    pub priority: i32,
    /// Operator-visible workflow description.
    pub description: String,
    /// Workflow creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Workflow start timestamp when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Workflow completion timestamp when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Linked retained session when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    /// Accepted turn index when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<u32>,
    /// Shared proof-outcome classification when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_outcome: Option<mister_smith_core::ProofOutcomeClassification>,
    /// Compact operator-facing result preview when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_preview: Option<mister_smith_core::OperatorResultPreview>,
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
    /// Durable lifecycle meaning projected for operator-facing views.
    pub lifecycle_state: DurableWorkflowLifecycleState,
    /// Task result, if complete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Task lifecycle command request.
#[derive(Debug, Deserialize)]
pub struct TaskLifecycleRequest {
    /// Durable lifecycle verb to apply.
    pub verb: DurableWorkflowLifecycleVerb,
    /// Optional operator-visible reason.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Task lifecycle command response.
#[derive(Debug, Serialize)]
pub struct TaskLifecycleResponse {
    /// Task identifier.
    pub task_id: TaskId,
    /// Current persisted task status after the command.
    pub status: String,
    /// Durable lifecycle meaning projected for operator-facing views.
    pub lifecycle_state: DurableWorkflowLifecycleState,
    /// Durable accepted outcome of the lifecycle command.
    pub outcome: LifecycleDecisionOutcome,
    /// Optional operator-facing note for no-op or deferred handling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
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
    /// Durable lifecycle meaning projected for operator-facing views.
    pub lifecycle_state: DurableWorkflowLifecycleState,
    /// Original operator message.
    pub user_message: String,
    /// Retained session-facing result projection for the turn, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_result: Option<mister_smith_core::SessionRetainedResultView>,
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

/// Focused current-turn state returned by the session inspect view.
#[derive(Debug, Serialize)]
pub struct SessionCurrentTurnStateResponse {
    /// Root workflow in focus for the current turn state.
    pub workflow_id: TaskId,
    /// 1-based accepted turn order.
    pub turn_index: u32,
    /// User-visible current turn state.
    pub turn_status: String,
    /// Durable lifecycle meaning projected for operator-facing views.
    pub lifecycle_state: DurableWorkflowLifecycleState,
    /// Compact preview of the most recent result when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_preview: Option<String>,
    /// Explicit proof-boundary wording when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_boundary_note: Option<String>,
    /// Whether this state is backed by live runtime or retained projections.
    pub state_source: String,
    /// Next honest action from the same session loop.
    pub next_action_hint: String,
}

/// Session inspect response.
#[derive(Debug, Serialize)]
pub struct SessionInspectResponse {
    /// Compact user-facing title for the retained session.
    pub title: String,
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Session lifecycle state.
    pub status: mister_smith_core::SessionStatus,
    /// Current loop posture for the active session.
    pub loop_state: String,
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
    /// Most recent retained session-facing result projection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_assistant_result: Option<mister_smith_core::SessionRetainedResultView>,
    /// Focused current-turn projection for the live loop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_turn_state: Option<SessionCurrentTurnStateResponse>,
    /// Ordered turn summaries.
    pub turns: Vec<SessionTurnSummaryResponse>,
    /// Durable control state currently attached to the session shell.
    pub control_state: SessionControlResponse,
    /// Inline warnings and degraded-state notes for the session shell.
    pub support_notices: Vec<SupportNoticeResponse>,
    /// Next honest action from the same session identity.
    pub next_action_hint: String,
    /// Logical close time when ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session collection query parameters.
#[derive(Debug, Deserialize, Default)]
pub struct SessionListQuery {
    /// Optional session lifecycle filter.
    #[serde(default)]
    pub status: Option<String>,
    /// Max rows to return.
    #[serde(default)]
    pub limit: Option<usize>,
    /// Rows to skip from the head of the result set.
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Session summary for collection responses.
#[derive(Debug, Serialize)]
pub struct SessionSummaryResponse {
    /// Compact user-facing title for the retained session.
    pub title: String,
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Session lifecycle state.
    pub status: mister_smith_core::SessionStatus,
    /// Stable coordinator identity.
    pub coordinator_agent_id: AgentId,
    /// Provider currently attributed to the session.
    pub provider_kind: String,
    /// Model currently attributed to the session.
    pub model_id: String,
    /// Active workflow when the session is busy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_workflow_id: Option<TaskId>,
    /// Most recent terminal workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_completed_workflow_id: Option<TaskId>,
    /// Number of accepted turns.
    pub turn_count: u32,
    /// Most recent update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Logical close time when ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Compact preview of the most recent retained assistant result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_preview: Option<String>,
}

/// One support warning or degraded-state note shown by the CLI shell.
#[derive(Debug, Serialize)]
pub struct SupportNoticeResponse {
    /// Stable machine-readable notice kind.
    pub notice_kind: String,
    /// Relative severity shown in the shell.
    pub severity: String,
    /// User-facing summary rendered inline.
    pub summary: String,
    /// Related support surface, when one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_surface: Option<String>,
    /// Whether the notice currently blocks another live turn.
    #[serde(skip_serializing_if = "is_false")]
    pub blocks_live_turn: bool,
    /// Next honest action while the notice remains active.
    pub allowed_next_action: String,
}

/// Durable control state exposed to the CLI session shell.
#[derive(Debug, Serialize)]
pub struct SessionControlResponse {
    /// Preferred provider kind recorded for later turns, when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_provider_kind: Option<String>,
    /// Preferred model recorded for later turns, when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_model_id: Option<String>,
    /// Current permission posture selected in the shell.
    pub permission_mode: String,
    /// Config posture shown by the shell.
    pub config_posture: String,
    /// Session status rendering mode selected in the shell.
    pub status_view: String,
    /// MCP posture selected in the shell.
    pub mcp_posture: String,
}

/// Session control update request.
#[derive(Debug, Deserialize)]
pub struct UpdateSessionControlRequest {
    /// Preferred provider kind recorded for later turns, when set.
    #[serde(default)]
    pub selected_provider_kind: Option<String>,
    /// Preferred model recorded for later turns, when set.
    #[serde(default)]
    pub selected_model_id: Option<String>,
    /// Permission posture selected in the shell, when set.
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Config posture selected in the shell, when set.
    #[serde(default)]
    pub config_posture: Option<String>,
    /// Session status rendering mode selected in the shell, when set.
    #[serde(default)]
    pub status_view: Option<String>,
    /// MCP posture selected in the shell, when set.
    #[serde(default)]
    pub mcp_posture: Option<String>,
    /// Clear selected_provider_kind override to revert to inherit.
    #[serde(default)]
    pub clear_selected_provider_kind: bool,
    /// Clear selected_model_id override to revert to inherit.
    #[serde(default)]
    pub clear_selected_model_id: bool,
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
    State(state): State<AppState>,
    Query(query): Query<AgentListQuery>,
) -> Result<Json<Vec<AgentSummary>>, HttpError> {
    let agent_service = state.agent_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("agent inspection service unavailable".to_string())
    })?;
    let agents = agent_service
        .list_agents()
        .await
        .map_err(HttpError::InternalError)?;
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
        .map(|agent| AgentSummary {
            agent_id: agent.agent_id,
            agent_type: agent.agent_type,
            availability: agent.availability,
            name: agent.name,
            status: agent.status,
            last_heartbeat: agent.last_heartbeat,
        })
        .collect();

    Ok(Json(filtered))
}

/// `GET /api/v1/agents/{agent_id}` — Single agent detail.
pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetail>, HttpError> {
    let uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid agent ID: {agent_id}")))?;
    let agent_service = state.agent_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("agent inspection service unavailable".to_string())
    })?;
    let found = agent_service
        .get_agent(AgentId::from_uuid(uuid))
        .await
        .map_err(HttpError::InternalError)?;

    found
        .map(|agent| {
            Json(AgentDetail {
                agent_id: agent.agent_id,
                agent_type: agent.agent_type,
                availability: agent.availability,
                name: agent.name,
                status: agent.status,
                last_heartbeat: agent.last_heartbeat,
                metadata: agent.metadata,
            })
        })
        .ok_or_else(|| HttpError::NotFound(format!("Agent {agent_id} not found")))
}

/// `GET /api/v1/tasks` — List root workflow runs.
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskListQuery>,
) -> Result<Json<Vec<TaskSummaryResponse>>, HttpError> {
    let task_service = state
        .task_service
        .as_ref()
        .ok_or_else(|| HttpError::InternalError("runtime task service unavailable".to_string()))?;
    let rows = task_service
        .list_tasks(TaskListRequest {
            status: query.status,
            limit: normalize_collection_limit(query.limit),
            offset: query.offset.unwrap_or(0),
        })
        .await
        .map_err(HttpError::InternalError)?;

    Ok(Json(
        rows.into_iter()
            .map(|row| TaskSummaryResponse {
                task_id: row.task_id,
                status: row.status,
                lifecycle_state: row.lifecycle_state,
                priority: row.priority,
                description: row.description,
                created_at: row.created_at,
                started_at: row.started_at,
                completed_at: row.completed_at,
                session_id: row.session_id,
                turn_index: row.turn_index,
                proof_outcome: row.proof_outcome,
                result_preview: row.result_preview,
            })
            .collect(),
    ))
}

/// `POST /api/v1/tasks` — Submit a task, returns 202 Accepted.
pub async fn create_task(
    State(state): State<AppState>,
    ExternalDelegationBoundary(delegation): ExternalDelegationBoundary,
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
            delegation,
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
    ExternalDelegationBoundary(delegation): ExternalDelegationBoundary,
    Json(request): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionTurnAcceptedResponse>), HttpError> {
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let accepted = conversation_service
        .create_session(ConversationCreateRequest {
            message: request.message,
            priority: request.priority,
            delegation,
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
    ExternalDelegationBoundary(delegation): ExternalDelegationBoundary,
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
            delegation,
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
        title: view.title,
        session_id: view.session_id,
        status: view.status,
        loop_state: view.loop_state.as_str().to_string(),
        coordinator_agent_id: view.coordinator_agent_id,
        provider_kind: view.provider_kind,
        model_id: view.model_id,
        active_workflow_id: view.active_workflow_id,
        last_completed_workflow_id: view.last_completed_workflow_id,
        turn_count: view.turn_count,
        last_assistant_result: view.last_assistant_result,
        current_turn_state: view
            .current_turn_state
            .map(|turn| SessionCurrentTurnStateResponse {
                workflow_id: turn.workflow_id,
                turn_index: turn.turn_index,
                turn_status: turn.turn_status,
                lifecycle_state: turn.lifecycle_state,
                result_preview: turn.result_preview,
                proof_boundary_note: turn.proof_boundary_note,
                state_source: turn.state_source.as_str().to_string(),
                next_action_hint: turn.next_action_hint,
            }),
        turns: view
            .turns
            .into_iter()
            .map(|turn| SessionTurnSummaryResponse {
                turn_index: turn.turn_index,
                workflow_id: turn.workflow_id,
                status: turn.status,
                lifecycle_state: turn.lifecycle_state,
                user_message: turn.user_message,
                assistant_result: turn.assistant_result,
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
        control_state: SessionControlResponse {
            selected_provider_kind: view.control_state.selected_provider_kind,
            selected_model_id: view.control_state.selected_model_id,
            permission_mode: view.control_state.permission_mode,
            config_posture: view.control_state.config_posture,
            status_view: view.control_state.status_view,
            mcp_posture: view.control_state.mcp_posture,
        },
        support_notices: view
            .support_notices
            .into_iter()
            .map(|notice| SupportNoticeResponse {
                notice_kind: notice.notice_kind,
                severity: notice.severity,
                summary: notice.summary,
                support_surface: notice.support_surface,
                blocks_live_turn: notice.blocks_live_turn,
                allowed_next_action: notice.allowed_next_action,
            })
            .collect(),
        next_action_hint: view.next_action_hint,
        ended_at: view.ended_at,
    }))
}

/// `GET /api/v1/sessions` — List durable conversation sessions.
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<Vec<SessionSummaryResponse>>, HttpError> {
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let rows = conversation_service
        .list_sessions(SessionListRequest {
            status: query.status,
            limit: normalize_collection_limit(query.limit),
            offset: query.offset.unwrap_or(0),
        })
        .await
        .map_err(map_conversation_error)?;

    Ok(Json(
        rows.into_iter()
            .map(|row| SessionSummaryResponse {
                title: row.title,
                session_id: row.session_id,
                status: row.status,
                coordinator_agent_id: row.coordinator_agent_id,
                provider_kind: row.provider_kind,
                model_id: row.model_id,
                active_workflow_id: row.active_workflow_id,
                last_completed_workflow_id: row.last_completed_workflow_id,
                turn_count: row.turn_count,
                updated_at: row.updated_at,
                ended_at: row.ended_at,
                last_preview: row.last_preview,
            })
            .collect(),
    ))
}

/// `POST /api/v1/sessions/{session_id}/controls` — Update one session shell control state.
pub async fn update_session_controls(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateSessionControlRequest>,
) -> Result<Json<SessionControlResponse>, HttpError> {
    let session_id = parse_session_path(&session_id)?;
    let conversation_service = state.conversation_service.as_ref().ok_or_else(|| {
        HttpError::InternalError("runtime conversation service unavailable".to_string())
    })?;

    let view = conversation_service
        .update_session_control_state(
            session_id,
            ConversationSessionControlUpdateRequest {
                selected_provider_kind: request.selected_provider_kind,
                selected_model_id: request.selected_model_id,
                permission_mode: request.permission_mode,
                config_posture: request.config_posture,
                status_view: request.status_view,
                mcp_posture: request.mcp_posture,
                clear_selected_provider_kind: request.clear_selected_provider_kind,
                clear_selected_model_id: request.clear_selected_model_id,
            },
        )
        .await
        .map_err(map_conversation_error)?;

    Ok(Json(SessionControlResponse {
        selected_provider_kind: view.selected_provider_kind,
        selected_model_id: view.selected_model_id,
        permission_mode: view.permission_mode,
        config_posture: view.config_posture,
        status_view: view.status_view,
        mcp_posture: view.mcp_posture,
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
        lifecycle_state: status.lifecycle_state,
        result: status.result,
    }))
}

/// `POST /api/v1/tasks/{task_id}/lifecycle` — Apply one durable lifecycle verb.
pub async fn apply_task_lifecycle(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(request): Json<TaskLifecycleRequest>,
) -> Result<Json<TaskLifecycleResponse>, HttpError> {
    let uuid = Uuid::parse_str(&task_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid task ID: {task_id}")))?;

    let task_service = state
        .task_service
        .as_ref()
        .ok_or_else(|| HttpError::InternalError("runtime task service unavailable".to_string()))?;

    let task_id = TaskId::from_uuid(uuid);
    let decision = task_service
        .apply_task_lifecycle(task_id, request.verb, request.reason)
        .await
        .map_err(HttpError::InternalError)?
        .ok_or_else(|| HttpError::NotFound(format!("Task {task_id} not found")))?;

    Ok(Json(TaskLifecycleResponse {
        task_id: decision.task_id,
        status: decision.status,
        lifecycle_state: decision.lifecycle_state,
        outcome: decision.outcome,
        note: decision.note,
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

fn parse_session_path(raw: &str) -> Result<SessionId, HttpError> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| HttpError::BadRequest(format!("Invalid session ID: {raw}")))
}

fn normalize_collection_limit(requested: Option<usize>) -> usize {
    requested
        .unwrap_or(DEFAULT_COLLECTION_LIMIT)
        .clamp(1, MAX_COLLECTION_LIMIT)
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
    use std::sync::Arc;

    use crate::server::{
        AgentInspectionDetailView, AgentInspectionService, AgentInspectionSummaryView, AppState,
        ConversationContinueRequest, ConversationEndView, ConversationResumeProvenanceView,
        ConversationServiceError, ConversationSessionControlUpdateRequest,
        ConversationSessionControlView, ConversationSessionService, ConversationSessionSummaryView,
        ConversationSessionView, ConversationSupportNoticeView, ConversationTurnAccepted,
        ConversationTurnSummaryView, NatsHealthCheck, SessionListRequest, TaskExecutionService,
        TaskListRequest, TaskStatusView, TaskSubmissionResponse, TaskSummaryView,
    };
    use mister_smith_core::{
        AuthorityPrincipal, CapabilityActionKind, DelegatedAction, DelegatedActionPolicy,
        DelegationScope, DurableWorkflowLifecycleState, DurableWorkflowLifecycleVerb,
        ExternalDelegationEnvelope, SessionRetainedResultView,
    };

    #[derive(Clone)]
    struct FixedConversationService {
        view: ConversationSessionView,
        summaries: Vec<ConversationSessionSummaryView>,
    }

    #[async_trait::async_trait]
    impl ConversationSessionService for FixedConversationService {
        async fn create_session(
            &self,
            _request: ConversationCreateRequest,
        ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
            Err(ConversationServiceError::Internal(
                "create_session not used in handler test".to_string(),
            ))
        }

        async fn continue_session(
            &self,
            _request: ConversationContinueRequest,
        ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
            Err(ConversationServiceError::Internal(
                "continue_session not used in handler test".to_string(),
            ))
        }

        async fn get_session(
            &self,
            session_id: SessionId,
        ) -> Result<Option<ConversationSessionView>, ConversationServiceError> {
            if self.view.session_id == session_id {
                Ok(Some(self.view.clone()))
            } else {
                Ok(None)
            }
        }

        async fn end_session(
            &self,
            _session_id: SessionId,
        ) -> Result<ConversationEndView, ConversationServiceError> {
            Err(ConversationServiceError::Internal(
                "end_session not used in handler test".to_string(),
            ))
        }

        async fn update_session_control_state(
            &self,
            _session_id: SessionId,
            _request: ConversationSessionControlUpdateRequest,
        ) -> Result<ConversationSessionControlView, ConversationServiceError> {
            Ok(self.view.control_state.clone())
        }

        async fn list_sessions(
            &self,
            _request: SessionListRequest,
        ) -> Result<Vec<ConversationSessionSummaryView>, ConversationServiceError> {
            Ok(self.summaries.clone())
        }
    }

    #[derive(Clone)]
    struct FixedAgentService {
        summaries: Vec<AgentInspectionSummaryView>,
        details: std::collections::HashMap<AgentId, AgentInspectionDetailView>,
    }

    impl Default for FixedAgentService {
        fn default() -> Self {
            let worker_id = AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                    .expect("fixed worker id should parse"),
            );
            let supervisor_id = AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                    .expect("fixed supervisor id should parse"),
            );
            let worker = AgentInspectionDetailView {
                agent_id: worker_id,
                agent_type: AgentType::Worker,
                availability: AgentAvailability::Idle,
                name: "worker-1".to_string(),
                status: "idle".to_string(),
                last_heartbeat: None,
                metadata: serde_json::json!({ "tasks_completed": 3 }),
            };
            let supervisor = AgentInspectionDetailView {
                agent_id: supervisor_id,
                agent_type: AgentType::Supervisor,
                availability: AgentAvailability::Busy,
                name: "supervisor-1".to_string(),
                status: "active".to_string(),
                last_heartbeat: None,
                metadata: serde_json::json!({ "tasks_completed": 7 }),
            };
            Self {
                summaries: vec![
                    AgentInspectionSummaryView {
                        agent_id: worker_id,
                        agent_type: AgentType::Worker,
                        availability: AgentAvailability::Idle,
                        name: "worker-1".to_string(),
                        status: "idle".to_string(),
                        last_heartbeat: None,
                    },
                    AgentInspectionSummaryView {
                        agent_id: supervisor_id,
                        agent_type: AgentType::Supervisor,
                        availability: AgentAvailability::Busy,
                        name: "supervisor-1".to_string(),
                        status: "active".to_string(),
                        last_heartbeat: None,
                    },
                ],
                details: [(worker_id, worker), (supervisor_id, supervisor)]
                    .into_iter()
                    .collect(),
            }
        }
    }

    #[async_trait::async_trait]
    impl AgentInspectionService for FixedAgentService {
        async fn list_agents(&self) -> Result<Vec<AgentInspectionSummaryView>, String> {
            Ok(self.summaries.clone())
        }

        async fn get_agent(
            &self,
            agent_id: AgentId,
        ) -> Result<Option<AgentInspectionDetailView>, String> {
            Ok(self.details.get(&agent_id).cloned())
        }
    }

    #[derive(Clone)]
    struct RecordingTaskService {
        last_request: Arc<tokio::sync::Mutex<Option<TaskSubmissionRequest>>>,
        response: TaskSubmissionResponse,
        list_rows: Vec<TaskSummaryView>,
    }

    impl Default for RecordingTaskService {
        fn default() -> Self {
            Self {
                last_request: Arc::new(tokio::sync::Mutex::new(None)),
                response: TaskSubmissionResponse {
                    task_id: TaskId::from_uuid(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000010")
                            .expect("fixed task id should parse"),
                    ),
                    assigned_agent_id: AgentId::from_uuid(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000011")
                            .expect("fixed agent id should parse"),
                    ),
                    status: "queued".to_string(),
                },
                list_rows: vec![TaskSummaryView {
                    task_id: TaskId::from_uuid(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000010")
                            .expect("fixed task id should parse"),
                    ),
                    status: "completed".to_string(),
                    lifecycle_state: DurableWorkflowLifecycleState::Completed,
                    priority: 2,
                    description: "Operator task".to_string(),
                    created_at: chrono::Utc::now(),
                    started_at: None,
                    completed_at: None,
                    session_id: None,
                    turn_index: None,
                    proof_outcome: Some(
                        mister_smith_core::ProofOutcomeClassification::CollapsedToSequential,
                    ),
                    result_preview: None,
                }],
            }
        }
    }

    impl RecordingTaskService {
        async fn last_request(&self) -> TaskSubmissionRequest {
            self.last_request
                .lock()
                .await
                .clone()
                .expect("task request should be recorded")
        }
    }

    #[async_trait::async_trait]
    impl TaskExecutionService for RecordingTaskService {
        async fn submit_task(
            &self,
            request: TaskSubmissionRequest,
        ) -> Result<TaskSubmissionResponse, String> {
            *self.last_request.lock().await = Some(request);
            Ok(self.response.clone())
        }

        async fn get_task(&self, _task_id: TaskId) -> Result<Option<TaskStatusView>, String> {
            Ok(None)
        }

        async fn apply_task_lifecycle(
            &self,
            _task_id: TaskId,
            _verb: DurableWorkflowLifecycleVerb,
            _reason: Option<String>,
        ) -> Result<Option<crate::server::TaskLifecycleView>, String> {
            Ok(None)
        }

        async fn list_tasks(
            &self,
            _request: TaskListRequest,
        ) -> Result<Vec<TaskSummaryView>, String> {
            Ok(self.list_rows.clone())
        }
    }

    fn test_state() -> AppState {
        AppState::new()
    }

    fn sample_external_delegation() -> ExternalDelegationEnvelope {
        let service = mister_smith_security::DelegationService::new();
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let (capability, provenance) = service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some("tool:http.submit".to_string()),
                std::time::Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        ExternalDelegationEnvelope::new(capability, provenance).with_action(DelegatedAction {
            descriptor_id: "tool:http.submit".to_string(),
            action_id: "tool:http.submit#execute".to_string(),
            title: "execute http.submit".to_string(),
            description: "execute access for http.submit".to_string(),
            kind: CapabilityActionKind::Execute,
            policy: DelegatedActionPolicy {
                action: "execute".to_string(),
                resource: "http".to_string(),
                scope: "api".to_string(),
                resource_id: Some("tasks.create".to_string()),
            },
            required_scope: Some(DelegationScope::InvokeTool),
            revocation_key: "tool:http.submit#execute".to_string(),
        })
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
        let state = test_state().with_agent_service(Arc::new(FixedAgentService::default()));
        let query = Query(AgentListQuery::default());
        let Json(agents) = list_agents(State(state), query)
            .await
            .expect("agent list should succeed");
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn list_agents_filter_by_type() {
        let state = test_state().with_agent_service(Arc::new(FixedAgentService::default()));
        let query = Query(AgentListQuery {
            agent_type: Some("Worker".to_string()),
            availability: None,
        });
        let Json(agents) = list_agents(State(state), query)
            .await
            .expect("filtered agent list should succeed");
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "worker-1");
    }

    #[tokio::test]
    async fn get_agent_found() {
        let state = test_state().with_agent_service(Arc::new(FixedAgentService::default()));
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
        let state = test_state().with_agent_service(Arc::new(FixedAgentService::default()));
        let result = get_agent(
            State(state),
            Path("00000000-0000-0000-0000-000000000099".to_string()),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_agent_invalid_id() {
        let state = test_state().with_agent_service(Arc::new(FixedAgentService::default()));
        let result = get_agent(State(state), Path("not-a-uuid".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_task_returns_202_fields() {
        let service = RecordingTaskService::default();
        let state = test_state().with_task_service(Arc::new(service));
        let request = CreateTaskRequest {
            description: "Test task".to_string(),
            agent_type: None,
            priority: None,
        };
        let (status, Json(response)) = create_task(
            State(state),
            ExternalDelegationBoundary(None),
            Json(request),
        )
        .await
        .expect("task creation should succeed");

        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(
            response.task_id,
            TaskId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000010")
                    .expect("fixed task id should parse"),
            )
        );
        assert_eq!(
            response.assigned_agent_id,
            AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000011")
                    .expect("fixed agent id should parse"),
            )
        );
        assert_eq!(response.status, "queued");
    }

    #[tokio::test]
    async fn create_task_forwards_external_delegation_to_runtime_service() {
        let service = RecordingTaskService::default();
        let state = test_state().with_task_service(Arc::new(service.clone()));
        let delegation = sample_external_delegation();

        let request = CreateTaskRequest {
            description: "Delegated task".to_string(),
            agent_type: None,
            priority: Some("high".to_string()),
        };

        let (status, Json(response)) = create_task(
            State(state),
            ExternalDelegationBoundary(Some(delegation.clone())),
            Json(request),
        )
        .await
        .expect("task creation should succeed");

        assert_eq!(status, StatusCode::ACCEPTED);

        let recorded = service.last_request().await;
        assert_eq!(recorded.description, "Delegated task");
        assert_eq!(recorded.priority.as_deref(), Some("high"));
        assert!(recorded.conversation.is_none());
        assert_eq!(recorded.delegation, Some(delegation));
        assert_eq!(response.status, "queued");
    }

    #[tokio::test]
    async fn get_task_valid_id() {
        let state = test_state();
        let task_id = Uuid::new_v4().to_string();
        let result = get_task(State(state), Path(task_id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_tasks_returns_runtime_rows() {
        let state = test_state().with_task_service(Arc::new(RecordingTaskService::default()));
        let Json(tasks) = list_tasks(
            State(state),
            Query(TaskListQuery {
                status: None,
                limit: None,
                offset: None,
            }),
        )
        .await
        .expect("task list should succeed");

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "Operator task");
    }

    #[tokio::test]
    async fn get_config_returns_version() {
        let state = test_state();
        let Json(config) = get_config(State(state)).await;
        assert_eq!(config.version, "0.1.0");
    }

    #[tokio::test]
    async fn get_session_surfaces_turn_level_restart_resume_provenance() {
        let session_id = SessionId::new();
        let resumed_from_workflow_id = TaskId::new();
        let active_workflow_id = TaskId::new();
        let retained_result = SessionRetainedResultView {
            workflow_id: resumed_from_workflow_id,
            turn_index: 1,
            status: "failed".to_string(),
            assistant_result: serde_json::json!({
                "preview": "workflow interrupted by runtime restart before session sync",
                "aggregated_result": {
                    "error": "workflow interrupted by runtime restart before session sync",
                    "recovered_after_restart": true
                },
                "proof_outcome": "failed_before_graph",
                "recovered_after_restart": true
            }),
            preview: Some(
                "workflow interrupted by runtime restart before session sync".to_string(),
            ),
            runtime_truth: None,
            provenance: mister_smith_core::ResultProvenanceSummary {
                runtime_execution_mode: serde_json::json!({
                    "execution_boundary": "tool_bus"
                }),
                graph_state: None,
                graph_id: None,
                source_fields: vec![
                    "metadata.final_result".to_string(),
                    "metadata.aggregated_result".to_string(),
                ],
            },
        };
        let state = test_state().with_conversation_service(Arc::new(FixedConversationService {
            view: ConversationSessionView {
                title: "resume interrupted workflow".to_string(),
                session_id,
                status: mister_smith_core::SessionStatus::Active,
                loop_state: crate::server::ConversationLoopState::TurnPending,
                coordinator_agent_id: AgentId::new(),
                provider_kind: "openai_chatgpt".to_string(),
                model_id: "gpt-5.4".to_string(),
                active_workflow_id: Some(active_workflow_id),
                last_completed_workflow_id: Some(resumed_from_workflow_id),
                turn_count: 2,
                last_assistant_result: Some(retained_result.clone()),
                current_turn_state: None,
                turns: vec![
                    ConversationTurnSummaryView {
                        turn_index: 1,
                        workflow_id: resumed_from_workflow_id,
                        status: "failed".to_string(),
                        lifecycle_state: DurableWorkflowLifecycleState::Failed,
                        user_message: "turn one".to_string(),
                        assistant_result: Some(retained_result),
                        resume_provenance: Some(ConversationResumeProvenanceView {
                            recovered_after_restart: true,
                            resumed_after_restart: false,
                            recovered_at: Some(chrono::Utc::now()),
                            recovery_reason: Some(
                                "workflow interrupted by runtime restart before session sync"
                                    .to_string(),
                            ),
                            resumed_from_workflow_id: None,
                            resumed_from_turn_index: None,
                        }),
                    },
                    ConversationTurnSummaryView {
                        turn_index: 2,
                        workflow_id: active_workflow_id,
                        status: "queued".to_string(),
                        lifecycle_state: DurableWorkflowLifecycleState::Active,
                        user_message: "turn two".to_string(),
                        assistant_result: None,
                        resume_provenance: Some(ConversationResumeProvenanceView {
                            recovered_after_restart: false,
                            resumed_after_restart: true,
                            recovered_at: None,
                            recovery_reason: None,
                            resumed_from_workflow_id: Some(resumed_from_workflow_id),
                            resumed_from_turn_index: Some(1),
                        }),
                    },
                ],
                control_state: ConversationSessionControlView {
                    session_id,
                    selected_provider_kind: Some("openai_chatgpt".to_string()),
                    selected_model_id: Some("gpt-5.4".to_string()),
                    permission_mode: "review".to_string(),
                    config_posture: "inline".to_string(),
                    status_view: "summary".to_string(),
                    mcp_posture: "connected".to_string(),
                },
                support_notices: vec![ConversationSupportNoticeView {
                    notice_kind: "session_shell_preferences".to_string(),
                    severity: "warning".to_string(),
                    summary: "Shell control changes are stored with this session, but runtime execution still follows the active runtime path.".to_string(),
                    support_surface: Some("config".to_string()),
                    blocks_live_turn: false,
                    allowed_next_action:
                        "keep working in this session or adjust the support posture".to_string(),
                }],
                next_action_hint:
                    "stay in this session while the accepted turn starts".to_string(),
                ended_at: None,
            },
            summaries: vec![ConversationSessionSummaryView {
                title: "resume interrupted workflow".to_string(),
                session_id,
                status: mister_smith_core::SessionStatus::Active,
                coordinator_agent_id: AgentId::new(),
                provider_kind: "openai_chatgpt".to_string(),
                model_id: "gpt-5.4".to_string(),
                active_workflow_id: Some(active_workflow_id),
                last_completed_workflow_id: Some(resumed_from_workflow_id),
                turn_count: 2,
                updated_at: chrono::Utc::now(),
                ended_at: None,
                last_preview: Some(
                    "workflow interrupted by runtime restart before session sync".to_string(),
                ),
            }],
        }));

        let Json(response) = get_session(State(state), Path(session_id.to_string()))
            .await
            .expect("session inspect should succeed");
        let value = serde_json::to_value(response).expect("inspect response should serialize");

        assert_eq!(value["session_id"], session_id.to_string());
        assert_eq!(
            value["last_assistant_result"]["workflow_id"],
            resumed_from_workflow_id.to_string()
        );
        assert_eq!(
            value["turns"][0]["assistant_result"]["assistant_result"]["proof_outcome"],
            "failed_before_graph"
        );
        assert_eq!(
            value["turns"][0]["resume_provenance"]["recovered_after_restart"],
            true
        );
        assert_eq!(
            value["turns"][0]["resume_provenance"]["recovery_reason"],
            "workflow interrupted by runtime restart before session sync"
        );
        assert_eq!(
            value["turns"][1]["resume_provenance"]["resumed_after_restart"],
            true
        );
        assert_eq!(
            value["turns"][1]["resume_provenance"]["resumed_from_turn_index"],
            1
        );
        assert_eq!(
            value["turns"][1]["resume_provenance"]["resumed_from_workflow_id"],
            resumed_from_workflow_id.to_string()
        );

        // Assert new response fields
        assert_eq!(value["loop_state"], "TurnPending");
        assert!(value["current_turn_state"].is_null());
        assert_eq!(
            value["next_action_hint"],
            "wait for the accepted turn to start or inspect the current session state"
        );
        assert_eq!(value["support_notices"][0]["blocks_live_turn"], false);
        assert_eq!(
            value["support_notices"][0]["allowed_next_action"],
            "keep working in this session or adjust the support posture"
        );
    }

    #[tokio::test]
    async fn list_sessions_returns_runtime_rows() {
        let session_id = SessionId::new();
        let state = test_state().with_conversation_service(Arc::new(FixedConversationService {
            view: ConversationSessionView {
                title: "first retained session".to_string(),
                session_id,
                status: mister_smith_core::SessionStatus::Active,
                loop_state: crate::server::ConversationLoopState::Ready,
                coordinator_agent_id: AgentId::new(),
                provider_kind: "openai_chatgpt".to_string(),
                model_id: "gpt-5.4".to_string(),
                active_workflow_id: None,
                last_completed_workflow_id: None,
                turn_count: 1,
                last_assistant_result: None,
                current_turn_state: None,
                turns: vec![],
                control_state: ConversationSessionControlView {
                    session_id,
                    selected_provider_kind: None,
                    selected_model_id: None,
                    permission_mode: "default".to_string(),
                    config_posture: "inline".to_string(),
                    status_view: "summary".to_string(),
                    mcp_posture: "support_only".to_string(),
                },
                support_notices: vec![],
                next_action_hint:
                    "send a follow-up turn or adjust the session controls from this loop"
                        .to_string(),
                ended_at: None,
            },
            summaries: vec![ConversationSessionSummaryView {
                title: "first retained session".to_string(),
                session_id,
                status: mister_smith_core::SessionStatus::Active,
                coordinator_agent_id: AgentId::new(),
                provider_kind: "openai_chatgpt".to_string(),
                model_id: "gpt-5.4".to_string(),
                active_workflow_id: None,
                last_completed_workflow_id: None,
                turn_count: 1,
                updated_at: chrono::Utc::now(),
                ended_at: None,
                last_preview: Some("READY".to_string()),
            }],
        }));

        let Json(sessions) = list_sessions(
            State(state),
            Query(SessionListQuery {
                status: None,
                limit: None,
                offset: None,
            }),
        )
        .await
        .expect("session list should succeed");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].last_preview.as_deref(), Some("READY"));

        // Now test the full inspect response to verify all new fields
        let Json(inspect_response) = get_session(State(state), Path(session_id.to_string()))
            .await
            .expect("session inspect should succeed");
        let inspect_value = serde_json::to_value(inspect_response).expect("inspect response should serialize");

        assert_eq!(inspect_value["loop_state"], "Ready");
        assert!(inspect_value["current_turn_state"].is_null());
        assert_eq!(
            inspect_value["next_action_hint"],
            "send a follow-up turn or adjust the session controls from this loop"
        );
        assert_eq!(inspect_value["support_notices"].as_array().unwrap().len(), 0);
    }
}