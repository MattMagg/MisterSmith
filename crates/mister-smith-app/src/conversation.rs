//! Durable multi-turn conversation service and CLI helpers.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mister_smith_config::FrameworkConfig;
use mister_smith_core::{
    AgentId, DurableWorkflowLifecycleState, SessionId, SessionRetainedResultView, SessionStatus,
    TaskId,
};
use mister_smith_http::server::{
    ConversationContinueRequest, ConversationCreateRequest, ConversationEndView,
    ConversationResumeProvenanceView, ConversationServiceError, ConversationSessionService,
    ConversationSessionSummaryView, ConversationSessionView, ConversationTurnAccepted,
    ConversationTurnContext, ConversationTurnSummaryView, SessionListRequest,
    TaskSubmissionRequest,
};
use mister_smith_persistence::postgres::queries::{self, TaskRecord};
use mister_smith_persistence::{SessionRecord, SessionRepository, SessionTurnRecord};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::autonomy::{
    durable_lifecycle_state_from_metadata, lifecycle_state_from_status,
    resume_provenance_from_metadata,
};
use crate::execution::RuntimeTaskService;

/// Runtime-backed durable conversation service.
#[derive(Clone)]
pub(crate) struct ConversationRuntimeService {
    runtime_task_service: Arc<RuntimeTaskService>,
    session_repository: Arc<SessionRepository>,
}

impl ConversationRuntimeService {
    pub(crate) fn new(runtime_task_service: Arc<RuntimeTaskService>) -> Arc<Self> {
        let session_repository = Arc::new(SessionRepository::new(runtime_task_service.pool()));
        Arc::new(Self {
            runtime_task_service,
            session_repository,
        })
    }

    async fn load_session(
        &self,
        session_id: SessionId,
    ) -> Result<SessionRecord, ConversationServiceError> {
        self.session_repository
            .find_session(session_id)
            .await
            .map_err(persistence_error)?
            .ok_or(ConversationServiceError::NotFound { session_id })
    }

    async fn load_task(&self, workflow_id: TaskId) -> Result<TaskRecord, ConversationServiceError> {
        queries::find_task(&self.runtime_task_service.pool(), *workflow_id.as_ref())
            .await
            .map_err(persistence_error)?
            .ok_or_else(|| {
                ConversationServiceError::Internal(format!(
                    "workflow {workflow_id} disappeared before session sync"
                ))
            })
    }

    async fn sync_session(
        &self,
        session_id: SessionId,
    ) -> Result<(SessionRecord, Vec<SessionTurnRecord>), ConversationServiceError> {
        let mut session = self.load_session(session_id).await?;

        if let Some(active_workflow_id) = session.active_workflow_id {
            let workflow_id = TaskId::from_uuid(active_workflow_id);
            let mut task = self.load_task(workflow_id).await?;
            if !is_terminal_status(&task.status) {
                if let Some(recovered) = self
                    .runtime_task_service
                    .recover_orphaned_workflow(workflow_id)
                    .await
                    .map_err(ConversationServiceError::Internal)?
                {
                    task = recovered;
                }
            }
            if is_terminal_status(&task.status) {
                let mut turn = self
                    .session_repository
                    .find_turn_by_workflow(workflow_id)
                    .await
                    .map_err(persistence_error)?
                    .ok_or_else(|| {
                        ConversationServiceError::Internal(format!(
                            "session turn for workflow {workflow_id} disappeared before sync"
                        ))
                    })?;

                let task_result = task.result.clone();
                turn.status = task.status.clone();
                turn.result_summary = task_result.clone();
                turn.completed_at = task.completed_at.or_else(|| Some(Utc::now()));
                self.session_repository
                    .update_turn(&turn)
                    .await
                    .map_err(persistence_error)?;

                session.active_workflow_id = None;
                session.last_completed_workflow_id = Some(active_workflow_id);
                session.retained_context = retained_context_after_turn(
                    &session.retained_context,
                    &turn,
                    workflow_id,
                    task_result,
                );
                session.updated_at = Utc::now();
                session = self
                    .session_repository
                    .update_session(&session)
                    .await
                    .map_err(persistence_error)?;
            }
        }

        let turns = self
            .session_repository
            .list_turns(session_id)
            .await
            .map_err(persistence_error)?;

        Ok((session, turns))
    }

    async fn compensate_new_session(&self, session_id: SessionId, workflow_id: TaskId) {
        let _ = self.session_repository.delete_session(session_id).await;
        let _ = self
            .runtime_task_service
            .delete_workflow_record(workflow_id)
            .await;
    }

    async fn compensate_new_turn(&self, prior_session: &SessionRecord, workflow_id: TaskId) {
        let _ = self
            .session_repository
            .delete_turn_by_workflow(workflow_id)
            .await;
        let _ = self.session_repository.update_session(prior_session).await;
        let _ = self
            .runtime_task_service
            .delete_workflow_record(workflow_id)
            .await;
    }
}

#[async_trait]
impl ConversationSessionService for ConversationRuntimeService {
    async fn create_session(
        &self,
        request: ConversationCreateRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
        validate_message(&request.message)?;

        let session_id = SessionId::new();
        let coordinator_agent_id = AgentId::new();
        let workflow_id = TaskId::new();
        let retained_context = empty_retained_context();
        let now = Utc::now();
        let task_request = TaskSubmissionRequest {
            description: request.message.clone(),
            agent_type: None,
            priority: request.priority.clone(),
            conversation: Some(ConversationTurnContext {
                session_id,
                turn_index: 1,
                coordinator_agent_id,
                retained_context: retained_context.clone(),
            }),
            delegation: request.delegation.clone(),
        };

        self.runtime_task_service
            .prepare_workflow(workflow_id, &task_request)
            .await
            .map_err(ConversationServiceError::Internal)?;

        let session = SessionRecord {
            session_id: *session_id.as_ref(),
            coordinator_agent_id: *coordinator_agent_id.as_ref(),
            status: session_status_text(SessionStatus::Active).to_string(),
            provider_kind: self.runtime_task_service.provider_kind_name().to_string(),
            model_id: self.runtime_task_service.model_id().to_string(),
            active_workflow_id: Some(*workflow_id.as_ref()),
            last_completed_workflow_id: None,
            turn_count: 1,
            retained_context,
            created_at: now,
            updated_at: now,
            ended_at: None,
        };
        if let Err(error) = self.session_repository.create_session(&session).await {
            self.compensate_new_session(session_id, workflow_id).await;
            return Err(persistence_error(error));
        }

        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: *session_id.as_ref(),
            turn_index: 1,
            workflow_id: *workflow_id.as_ref(),
            user_message: request.message,
            result_summary: None,
            status: "queued".to_string(),
            created_at: now,
            completed_at: None,
        };
        if let Err(error) = self.session_repository.create_turn(&turn).await {
            self.compensate_new_session(session_id, workflow_id).await;
            return Err(persistence_error(error));
        }

        if let Err(error) = self
            .runtime_task_service
            .launch_workflow(workflow_id, task_request)
        {
            self.compensate_new_session(session_id, workflow_id).await;
            return Err(ConversationServiceError::Internal(error));
        }

        Ok(ConversationTurnAccepted {
            session_id,
            workflow_id,
            coordinator_agent_id,
            turn_index: 1,
            status: "queued".to_string(),
        })
    }

    async fn continue_session(
        &self,
        request: ConversationContinueRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
        validate_message(&request.message)?;

        let (session, _) = self.sync_session(request.session_id).await?;
        let session_status = parse_session_status(&session.status);
        if matches!(session_status, SessionStatus::Ended) {
            return Err(ConversationServiceError::SessionEnded {
                session_id: request.session_id,
            });
        }
        if let Some(active_workflow_id) = session.active_workflow_id {
            return Err(ConversationServiceError::SessionBusy {
                session_id: request.session_id,
                active_workflow_id: TaskId::from_uuid(active_workflow_id),
            });
        }

        let workflow_id = TaskId::new();
        let turn_index = (session.turn_count as u32).saturating_add(1);
        let coordinator_agent_id = AgentId::from_uuid(session.coordinator_agent_id);
        let now = Utc::now();
        let task_request = TaskSubmissionRequest {
            description: request.message.clone(),
            agent_type: None,
            priority: request.priority.clone(),
            conversation: Some(ConversationTurnContext {
                session_id: request.session_id,
                turn_index,
                coordinator_agent_id,
                retained_context: session.retained_context.clone(),
            }),
            delegation: request.delegation.clone(),
        };

        self.runtime_task_service
            .prepare_workflow(workflow_id, &task_request)
            .await
            .map_err(ConversationServiceError::Internal)?;

        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: session.session_id,
            turn_index: turn_index as i32,
            workflow_id: *workflow_id.as_ref(),
            user_message: request.message,
            result_summary: None,
            status: "queued".to_string(),
            created_at: now,
            completed_at: None,
        };
        if let Err(error) = self.session_repository.create_turn(&turn).await {
            let _ = self
                .runtime_task_service
                .delete_workflow_record(workflow_id)
                .await;
            return Err(persistence_error(error));
        }

        let prior_session = session.clone();
        let mut updated_session = session;
        updated_session.active_workflow_id = Some(*workflow_id.as_ref());
        updated_session.turn_count += 1;
        updated_session.updated_at = now;
        if let Err(error) = self
            .session_repository
            .update_session(&updated_session)
            .await
        {
            self.compensate_new_turn(&prior_session, workflow_id).await;
            return Err(persistence_error(error));
        }

        if let Err(error) = self
            .runtime_task_service
            .launch_workflow(workflow_id, task_request)
        {
            self.compensate_new_turn(&prior_session, workflow_id).await;
            return Err(ConversationServiceError::Internal(error));
        }

        Ok(ConversationTurnAccepted {
            session_id: request.session_id,
            workflow_id,
            coordinator_agent_id,
            turn_index,
            status: "queued".to_string(),
        })
    }

    async fn get_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<ConversationSessionView>, ConversationServiceError> {
        if self
            .session_repository
            .find_session(session_id)
            .await
            .map_err(persistence_error)?
            .is_none()
        {
            return Ok(None);
        }

        let (session, turns) = self.sync_session(session_id).await?;
        let pool = self.runtime_task_service.pool();
        let view = build_session_view(session, turns, &pool).await?;
        Ok(Some(view))
    }

    async fn end_session(
        &self,
        session_id: SessionId,
    ) -> Result<ConversationEndView, ConversationServiceError> {
        let (mut session, _) = self.sync_session(session_id).await?;
        if matches!(parse_session_status(&session.status), SessionStatus::Ended) {
            return Err(ConversationServiceError::SessionEnded { session_id });
        }
        if let Some(active_workflow_id) = session.active_workflow_id {
            return Err(ConversationServiceError::SessionBusy {
                session_id,
                active_workflow_id: TaskId::from_uuid(active_workflow_id),
            });
        }

        let ended_at = Utc::now();
        session.status = session_status_text(SessionStatus::Ended).to_string();
        session.ended_at = Some(ended_at);
        session.updated_at = ended_at;
        let session = self
            .session_repository
            .update_session(&session)
            .await
            .map_err(persistence_error)?;

        Ok(ConversationEndView {
            session_id,
            status: parse_session_status(&session.status),
            ended_at: session.ended_at.unwrap_or(ended_at),
        })
    }

    async fn list_sessions(
        &self,
        request: SessionListRequest,
    ) -> Result<Vec<ConversationSessionSummaryView>, ConversationServiceError> {
        let limit = i64::try_from(request.limit).map_err(|_| {
            ConversationServiceError::BadRequest("session list limit is invalid".to_string())
        })?;
        let offset = i64::try_from(request.offset).map_err(|_| {
            ConversationServiceError::BadRequest("session list offset is invalid".to_string())
        })?;
        let rows = self
            .session_repository
            .list_sessions(request.status.as_deref(), limit, offset)
            .await
            .map_err(persistence_error)?;

        let mut summaries = Vec::with_capacity(rows.len());
        for session in rows {
            let session = if session.active_workflow_id.is_some() {
                self.sync_session(SessionId::from_uuid(session.session_id))
                    .await?
                    .0
            } else {
                session
            };
            summaries.push(build_session_summary_view(&session));
        }

        Ok(summaries)
    }
}

async fn build_session_view(
    session: SessionRecord,
    turns: Vec<SessionTurnRecord>,
    pool: &PgPool,
) -> Result<ConversationSessionView, ConversationServiceError> {
    let task_ids = turns
        .iter()
        .map(|turn| turn.workflow_id)
        .collect::<Vec<_>>();
    let task_metadata_by_workflow = queries::find_tasks_by_ids(pool, &task_ids)
        .await
        .map_err(persistence_error)?
        .into_iter()
        .map(|record| (record.task_id, record.metadata))
        .collect::<HashMap<_, _>>();
    let mut turn_summaries = Vec::with_capacity(turns.len());
    let retained_context = &session.retained_context;
    let mut last_assistant_result = last_retained_result_from_context(retained_context);
    for turn in turns {
        let workflow_id = TaskId::from_uuid(turn.workflow_id);
        let turn_index = turn.turn_index.max(0) as u32;
        let assistant_result = retained_result_for_turn(retained_context, workflow_id, turn_index)
            .or_else(|| {
                turn.result_summary.as_ref().and_then(|task_result| {
                    crate::autonomy::retained_result_view(task_result, turn_index, &turn.status)
                })
            });
        if let Some(retained_result) = assistant_result.clone() {
            let should_replace_last = last_assistant_result
                .as_ref()
                .map(|current| retained_result.turn_index >= current.turn_index)
                .unwrap_or(true);
            if should_replace_last {
                last_assistant_result = Some(retained_result);
            }
        }
        let resume_provenance = task_metadata_by_workflow
            .get(&turn.workflow_id)
            .and_then(resume_provenance_from_metadata)
            .map(|details| ConversationResumeProvenanceView {
                recovered_after_restart: details.recovered_after_restart,
                resumed_after_restart: details.resumed_after_restart,
                recovered_at: details.recovered_at,
                recovery_reason: details.recovery_reason,
                resumed_from_workflow_id: details.resumed_from_workflow_id,
                resumed_from_turn_index: details.resumed_from_turn_index,
            });
        let turn_status = turn.status.clone();
        turn_summaries.push(ConversationTurnSummaryView {
            turn_index: turn.turn_index.max(0) as u32,
            workflow_id,
            status: turn_status.clone(),
            lifecycle_state: lifecycle_state_for_turn(
                task_metadata_by_workflow.get(&turn.workflow_id),
                &turn_status,
            ),
            user_message: turn.user_message,
            assistant_result,
            resume_provenance,
        });
    }

    Ok(ConversationSessionView {
        session_id: SessionId::from_uuid(session.session_id),
        status: parse_session_status(&session.status),
        coordinator_agent_id: AgentId::from_uuid(session.coordinator_agent_id),
        provider_kind: session.provider_kind,
        model_id: session.model_id,
        active_workflow_id: session.active_workflow_id.map(TaskId::from_uuid),
        last_completed_workflow_id: session.last_completed_workflow_id.map(TaskId::from_uuid),
        turn_count: session.turn_count.max(0) as u32,
        last_assistant_result,
        turns: turn_summaries,
        ended_at: session.ended_at,
    })
}

fn validate_message(message: &str) -> Result<(), ConversationServiceError> {
    if message.trim().is_empty() {
        return Err(ConversationServiceError::BadRequest(
            "conversation message must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn persistence_error(error: mister_smith_core::PersistenceError) -> ConversationServiceError {
    ConversationServiceError::Internal(error.to_string())
}

fn is_terminal_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "completed" | "failed" | "cancelled" | "terminated"
    )
}

pub(crate) fn lifecycle_state_for_turn(
    metadata: Option<&Value>,
    status: &str,
) -> DurableWorkflowLifecycleState {
    metadata
        .and_then(durable_lifecycle_state_from_metadata)
        .unwrap_or_else(|| lifecycle_state_from_status(status))
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Ended => "ended",
    }
}

fn parse_session_status(raw: &str) -> SessionStatus {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ended" => SessionStatus::Ended,
        _ => SessionStatus::Active,
    }
}

fn empty_retained_context() -> Value {
    json!({
        "last_user_message": Value::Null,
        "last_assistant_result": Value::Null,
        "transcript_summary": [],
        "latest_workflow_id": Value::Null,
    })
}

fn retained_result_projection_from_value(value: &Value) -> Option<SessionRetainedResultView> {
    serde_json::from_value(value.clone()).ok()
}

fn retained_result_for_turn(
    retained_context: &Value,
    workflow_id: TaskId,
    turn_index: u32,
) -> Option<SessionRetainedResultView> {
    let workflow_id = workflow_id.to_string();
    retained_context
        .get("transcript_summary")
        .and_then(Value::as_array)?
        .iter()
        .rev()
        .find_map(|entry| {
            let entry_turn = entry
                .get("turn_index")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())?;
            let entry_workflow_id = entry.get("workflow_id").and_then(Value::as_str)?;
            if entry_turn != turn_index || entry_workflow_id != workflow_id {
                return None;
            }

            entry
                .get("assistant_result")
                .and_then(retained_result_projection_from_value)
        })
}

fn last_retained_result_from_context(
    retained_context: &Value,
) -> Option<SessionRetainedResultView> {
    retained_context
        .get("last_assistant_result")
        .and_then(retained_result_projection_from_value)
}

fn last_preview_from_context(retained_context: &Value) -> Option<String> {
    last_retained_result_from_context(retained_context).and_then(|view| {
        view.preview.or_else(|| {
            view.assistant_result
                .get("preview")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
    })
}

fn build_session_summary_view(session: &SessionRecord) -> ConversationSessionSummaryView {
    ConversationSessionSummaryView {
        session_id: SessionId::from_uuid(session.session_id),
        status: parse_session_status(&session.status),
        coordinator_agent_id: AgentId::from_uuid(session.coordinator_agent_id),
        provider_kind: session.provider_kind.clone(),
        model_id: session.model_id.clone(),
        active_workflow_id: session.active_workflow_id.map(TaskId::from_uuid),
        last_completed_workflow_id: session.last_completed_workflow_id.map(TaskId::from_uuid),
        turn_count: session.turn_count.max(0) as u32,
        updated_at: session.updated_at,
        ended_at: session.ended_at,
        last_preview: last_preview_from_context(&session.retained_context),
    }
}

fn retained_context_after_turn(
    current: &Value,
    turn: &SessionTurnRecord,
    workflow_id: TaskId,
    result_summary: Option<Value>,
) -> Value {
    let mut updated = current.clone();
    if updated.is_null() || !updated.is_object() {
        updated = empty_retained_context();
    }

    let latest_workflow_id = updated
        .get("latest_workflow_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if latest_workflow_id == workflow_id.to_string() {
        return updated;
    }

    let assistant_result = result_summary
        .as_ref()
        .and_then(|value| {
            u32::try_from(turn.turn_index).ok().and_then(|turn_index| {
                crate::autonomy::retained_result_view(value, turn_index, &turn.status)
            })
        })
        .and_then(|projection| serde_json::to_value(projection).ok())
        .or_else(|| {
            result_summary.as_ref().and_then(|value| {
                u32::try_from(turn.turn_index).ok().and_then(|turn_index| {
                    crate::autonomy::retained_assistant_result(value, turn_index, &turn.status)
                })
            })
        })
        .unwrap_or_else(|| json!({ "status": turn.status }));
    let summary_entry = json!({
        "turn_index": turn.turn_index,
        "workflow_id": workflow_id,
        "status": turn.status,
        "user_message": turn.user_message,
        "assistant_result": assistant_result,
    });

    if let Some(object) = updated.as_object_mut() {
        object.insert(
            "last_user_message".to_string(),
            json!(turn.user_message.clone()),
        );
        object.insert(
            "last_assistant_result".to_string(),
            summary_entry
                .get("assistant_result")
                .cloned()
                .unwrap_or(Value::Null),
        );
        object.insert(
            "latest_workflow_id".to_string(),
            json!(workflow_id.to_string()),
        );
        let transcript = object
            .entry("transcript_summary".to_string())
            .or_insert_with(|| json!([]));
        if let Some(entries) = transcript.as_array_mut() {
            entries.push(summary_entry);
        }
    }

    updated
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ApiErrorBody {
    error: String,
    #[serde(default)]
    message: Option<String>,
}

/// Client-facing accepted turn view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliTurnAccepted {
    pub session_id: String,
    pub workflow_id: String,
    pub coordinator_agent_id: String,
    pub turn_index: u32,
    pub status: String,
}

/// Client-facing inspect view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliSessionView {
    pub session_id: String,
    pub status: String,
    pub coordinator_agent_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub active_workflow_id: Option<String>,
    pub last_completed_workflow_id: Option<String>,
    pub turn_count: u32,
    #[serde(default)]
    pub last_assistant_result: Option<SessionRetainedResultView>,
    #[serde(default)]
    pub turns: Vec<ConversationCliTurnSummary>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliResumeProvenanceView {
    #[serde(default)]
    pub recovered_after_restart: bool,
    #[serde(default)]
    pub resumed_after_restart: bool,
    pub recovered_at: Option<String>,
    pub recovery_reason: Option<String>,
    pub resumed_from_workflow_id: Option<String>,
    pub resumed_from_turn_index: Option<u32>,
}

/// Client-facing turn summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliTurnSummary {
    pub turn_index: u32,
    pub workflow_id: String,
    pub status: String,
    pub lifecycle_state: DurableWorkflowLifecycleState,
    pub user_message: String,
    #[serde(default)]
    pub assistant_result: Option<SessionRetainedResultView>,
    #[serde(default)]
    pub resume_provenance: Option<ConversationCliResumeProvenanceView>,
}

/// Client-facing end response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliEndView {
    pub session_id: String,
    pub status: String,
    pub ended_at: String,
}

/// Error returned when conversation CLI operations fail.
#[derive(Debug)]
pub(crate) enum ConversationClientError {
    InvalidSessionId(String),
    Http(reqwest::Error),
    HttpStatus(StatusCode, String),
}

impl fmt::Display for ConversationClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationClientError::InvalidSessionId(raw) => {
                write!(f, "invalid session id '{raw}'")
            }
            ConversationClientError::Http(error) => write!(f, "{error}"),
            ConversationClientError::HttpStatus(status, body) => {
                write!(f, "runtime returned {}: {}", status.as_u16(), body)
            }
        }
    }
}

impl Error for ConversationClientError {}

pub(crate) fn parse_session_id(raw: &str) -> Result<SessionId, ConversationClientError> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| ConversationClientError::InvalidSessionId(raw.to_string()))
}

pub(crate) fn default_base_url(config: &FrameworkConfig) -> String {
    let port = config.transport.http_port.unwrap_or(8080);
    format!("http://127.0.0.1:{port}")
}

pub(crate) async fn start_session_http(
    base_url: &str,
    message: &str,
    priority: Option<String>,
) -> Result<ConversationCliTurnAccepted, ConversationClientError> {
    let client = Client::new();
    let url = format!("{}/api/v1/sessions", base_url.trim_end_matches('/'));
    let response = client
        .post(url)
        .json(&json!({
            "message": message,
            "priority": priority,
        }))
        .send()
        .await
        .map_err(ConversationClientError::Http)?;
    decode_json_response(response).await
}

pub(crate) async fn continue_session_http(
    base_url: &str,
    session_id: SessionId,
    message: &str,
    priority: Option<String>,
) -> Result<ConversationCliTurnAccepted, ConversationClientError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/sessions/{}/turns",
        base_url.trim_end_matches('/'),
        session_id
    );
    let response = client
        .post(url)
        .json(&json!({
            "message": message,
            "priority": priority,
        }))
        .send()
        .await
        .map_err(ConversationClientError::Http)?;
    decode_json_response(response).await
}

pub(crate) async fn inspect_session_http(
    base_url: &str,
    session_id: SessionId,
) -> Result<ConversationCliSessionView, ConversationClientError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/sessions/{}",
        base_url.trim_end_matches('/'),
        session_id
    );
    let response = client
        .get(url)
        .send()
        .await
        .map_err(ConversationClientError::Http)?;
    decode_json_response(response).await
}

pub(crate) async fn end_session_http(
    base_url: &str,
    session_id: SessionId,
) -> Result<ConversationCliEndView, ConversationClientError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/sessions/{}/end",
        base_url.trim_end_matches('/'),
        session_id
    );
    let response = client
        .post(url)
        .send()
        .await
        .map_err(ConversationClientError::Http)?;
    decode_json_response(response).await
}

async fn decode_json_response<T>(response: reqwest::Response) -> Result<T, ConversationClientError>
where
    T: serde::de::DeserializeOwned,
{
    if response.status().is_success() {
        return response
            .json::<T>()
            .await
            .map_err(ConversationClientError::Http);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let rendered = serde_json::from_str::<ApiErrorBody>(&body)
        .ok()
        .map(|payload| payload.message.unwrap_or(payload.error))
        .unwrap_or(body);
    Err(ConversationClientError::HttpStatus(status, rendered))
}

pub(crate) fn render_turn_accepted(view: &ConversationCliTurnAccepted) -> String {
    format!(
        "session_id: {}\nworkflow_id: {}\ncoordinator_agent_id: {}\nturn_index: {}\nstatus: {}",
        view.session_id, view.workflow_id, view.coordinator_agent_id, view.turn_index, view.status
    )
}

pub(crate) fn render_session(view: &ConversationCliSessionView) -> String {
    let turns = view
        .turns
        .iter()
        .map(|turn| {
            let assistant_result = turn
                .assistant_result
                .as_ref()
                .map(render_retained_result)
                .unwrap_or_else(|| "none".to_string());
            let resume_provenance = turn
                .resume_provenance
                .as_ref()
                .map(render_resume_provenance)
                .unwrap_or_else(|| "none".to_string());
            format!(
                "  {} {} {} lifecycle={} resume={} result={} {}",
                turn.turn_index,
                turn.workflow_id,
                turn.status,
                turn.lifecycle_state.as_str(),
                resume_provenance,
                assistant_result,
                turn.user_message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let last_assistant_result = view
        .last_assistant_result
        .as_ref()
        .map(render_retained_result)
        .unwrap_or_else(|| "none".to_string());

    format!(
        "session_id: {}\nstatus: {}\ncoordinator_agent_id: {}\nprovider_kind: {}\nmodel_id: {}\nactive_workflow_id: {}\nlast_completed_workflow_id: {}\nlast_assistant_result: {}\nturn_count: {}\nended_at: {}\nturns:\n{}",
        view.session_id,
        view.status,
        view.coordinator_agent_id,
        view.provider_kind,
        view.model_id,
        view.active_workflow_id.as_deref().unwrap_or("none"),
        view.last_completed_workflow_id.as_deref().unwrap_or("none"),
        last_assistant_result,
        view.turn_count,
        view.ended_at.as_deref().unwrap_or("none"),
        if turns.is_empty() {
            "none".to_string()
        } else {
            turns
        }
    )
}

pub(crate) fn render_end_view(view: &ConversationCliEndView) -> String {
    format!(
        "session_id: {}\nstatus: {}\nended_at: {}",
        view.session_id, view.status, view.ended_at
    )
}

fn render_resume_provenance(view: &ConversationCliResumeProvenanceView) -> String {
    let mut parts = Vec::new();

    if view.recovered_after_restart {
        parts.push("recovered_after_restart=true".to_string());
    }
    if view.resumed_after_restart {
        parts.push("resumed_after_restart=true".to_string());
    }
    if let Some(recovered_at) = view.recovered_at.as_ref() {
        parts.push(format!("recovered_at={recovered_at}"));
    }
    if let Some(reason) = view.recovery_reason.as_ref() {
        parts.push(format!("reason={reason}"));
    }
    if let Some(turn_index) = view.resumed_from_turn_index {
        parts.push(format!("resumed_from_turn={turn_index}"));
    }
    if let Some(workflow_id) = view.resumed_from_workflow_id.as_ref() {
        parts.push(format!("resumed_from_workflow={workflow_id}"));
    }

    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" ")
    }
}

fn render_retained_result(view: &SessionRetainedResultView) -> String {
    let proof_outcome = view
        .assistant_result
        .get("proof_outcome")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let preview = view
        .preview
        .clone()
        .or_else(|| {
            view.assistant_result
                .get("preview")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "none".to_string());
    let source_fields = if view.provenance.source_fields.is_empty() {
        "none".to_string()
    } else {
        view.provenance.source_fields.join("|")
    };
    let runtime_truth = view
        .runtime_truth
        .as_ref()
        .map(|summary| {
            let relationships = if summary.run_trace.relationships.is_empty() {
                "none".to_string()
            } else {
                summary
                    .run_trace
                    .relationships
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            format!(
                "{}:{} trace_root={} relationships=[{}]",
                summary.evidence_class.as_str(),
                summary.proof_boundary.task_proof,
                summary.run_trace.trace_root_id,
                relationships
            )
        })
        .unwrap_or_else(|| "none".to_string());

    format!(
        "workflow={} status={} proof={} preview={} runtime_truth={} sources={}",
        view.workflow_id, view.status, proof_outcome, preview, runtime_truth, source_fields
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_id_rejects_invalid_uuid() {
        let error = parse_session_id("not-a-uuid").unwrap_err();
        assert!(matches!(
            error,
            ConversationClientError::InvalidSessionId(_)
        ));
    }

    #[test]
    fn retained_context_after_turn_is_idempotent_per_workflow() {
        let workflow_id = TaskId::new();
        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            turn_index: 1,
            workflow_id: *workflow_id.as_ref(),
            user_message: "Summarize the runtime proof".to_string(),
            result_summary: None,
            status: "completed".to_string(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        let first = retained_context_after_turn(
            &empty_retained_context(),
            &turn,
            workflow_id,
            Some(json!({"summary": "done"})),
        );
        let second = retained_context_after_turn(
            &first,
            &turn,
            workflow_id,
            Some(json!({"summary": "done"})),
        );

        let transcript = second
            .get("transcript_summary")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert_eq!(transcript.len(), 1);
        assert_eq!(second["latest_workflow_id"], workflow_id.to_string());
    }

    #[test]
    fn retained_context_after_turn_projects_canonical_task_result_into_assistant_result() {
        let workflow_id = TaskId::new();
        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            turn_index: 2,
            workflow_id: *workflow_id.as_ref(),
            user_message: "Summarize the runtime proof".to_string(),
            result_summary: None,
            status: "completed".to_string(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        let retained = retained_context_after_turn(
            &empty_retained_context(),
            &turn,
            workflow_id,
            Some(json!({
                "workflow_id": workflow_id,
                "status": "completed",
                "proof_outcome": "graph_formed_and_completed",
                "runtime_truth": {
                    "evidence_class": "placeholder_or_simulated_step_completion",
                    "proof_boundary": {
                        "graph_execution": "workflow graph executed successfully",
                        "semantic_completion": "semantic completion not yet proven",
                        "grounded_tool_execution": "grounded tool execution: none/minimal",
                        "task_proof": "result is orchestration proof, not substantive task proof"
                    },
                    "run_trace": {
                        "trace_root_id": workflow_id.to_string(),
                        "workflow_id": workflow_id.to_string(),
                        "relationships": ["graph", "tool_boundary"]
                    },
                    "grounded_evidence": []
                },
                "result": {
                    "workflow_id": workflow_id,
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "description": "freeze the result contract",
                    "runtime_execution_mode": {
                        "execution_boundary": "tool_bus"
                    },
                    "planner_output": {
                        "steps": 2
                    },
                    "execution_plan": {
                        "steps": [{"id": "step-1"}, {"id": "step-2"}]
                    },
                    "step_results": [],
                    "aggregated_result": {
                        "summary": "bounded answer preview"
                    },
                    "proof_outcome": "graph_formed_and_completed"
                }
            })),
        );

        let assistant_result = retained["transcript_summary"][0]["assistant_result"].clone();
        assert_eq!(
            assistant_result["workflow_id"],
            json!(workflow_id.to_string())
        );
        assert_eq!(assistant_result["turn_index"], json!(2));
        assert_eq!(assistant_result["status"], json!("completed"));
        assert_eq!(assistant_result["preview"], json!("bounded answer preview"));
        assert_eq!(
            assistant_result["assistant_result"]["proof_outcome"],
            json!("graph_formed_and_completed")
        );
        assert_eq!(
            assistant_result["assistant_result"]["aggregated_result"]["summary"],
            json!("bounded answer preview")
        );
        assert_eq!(
            assistant_result["provenance"]["source_fields"],
            json!(["metadata.final_result", "metadata.aggregated_result"])
        );
        assert_eq!(
            assistant_result["runtime_truth"]["proof_boundary"]["task_proof"],
            json!("result is orchestration proof, not substantive task proof")
        );
        assert!(assistant_result["assistant_result"].get("result").is_none());
    }

    #[test]
    fn retained_context_after_turn_preserves_restart_recovery_flag_in_projection() {
        let workflow_id = TaskId::new();
        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            turn_index: 1,
            workflow_id: *workflow_id.as_ref(),
            user_message: "Resume the interrupted workflow".to_string(),
            result_summary: None,
            status: "failed".to_string(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        let retained = retained_context_after_turn(
            &empty_retained_context(),
            &turn,
            workflow_id,
            Some(json!({
                "workflow_id": workflow_id,
                "status": "failed",
                "proof_outcome": "failed_before_graph",
                "result": {
                    "workflow_id": workflow_id,
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "description": "resume the interrupted workflow",
                    "runtime_execution_mode": {
                        "execution_boundary": "tool_bus"
                    },
                    "planner_output": null,
                    "execution_plan": null,
                    "step_results": [],
                    "aggregated_result": {
                        "error": "workflow interrupted by runtime restart before session sync",
                        "recovered_after_restart": true
                    },
                    "proof_outcome": "failed_before_graph"
                }
            })),
        );

        assert_eq!(
            retained["last_assistant_result"]["assistant_result"]["recovered_after_restart"],
            json!(true)
        );
        assert_eq!(
            retained["last_assistant_result"]["assistant_result"]["proof_outcome"],
            json!("failed_before_graph")
        );
    }

    #[test]
    fn retained_context_after_turn_does_not_relabel_autonomy_status_surfaces() {
        let workflow_id = TaskId::new();
        let turn = SessionTurnRecord {
            turn_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            turn_index: 3,
            workflow_id: *workflow_id.as_ref(),
            user_message: "Continue the delegated workflow".to_string(),
            result_summary: None,
            status: "completed".to_string(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        let retained = retained_context_after_turn(
            &empty_retained_context(),
            &turn,
            workflow_id,
            Some(json!({
                "workflow_id": workflow_id,
                "status": "completed",
                "proof_outcome": "graph_formed_and_completed",
                "external_capability_decisions": [
                    {
                        "boundary_surface": "task_ingress",
                        "action_id": "tool:agent.echo#execute"
                    }
                ],
                "result": {
                    "workflow_id": workflow_id,
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "description": "continue the delegated workflow",
                    "runtime_execution_mode": {
                        "execution_boundary": "tool_bus"
                    },
                    "planner_output": {
                        "steps": 1
                    },
                    "execution_plan": {
                        "steps": [{"id": "step-1"}]
                    },
                    "step_results": [],
                    "aggregated_result": {
                        "summary": "bounded answer preview"
                    },
                    "proof_outcome": "graph_formed_and_completed"
                }
            })),
        );

        let assistant_result = retained["last_assistant_result"]["assistant_result"].clone();
        assert!(assistant_result
            .get("external_capability_decisions")
            .is_none());
        assert!(assistant_result.get("boundary_surface").is_none());
        assert_eq!(
            retained["last_assistant_result"]["preview"],
            json!("bounded answer preview")
        );
    }

    #[test]
    fn retained_result_for_turn_uses_stored_projection_with_proof_outcome() {
        let workflow_id = TaskId::new();
        let retained_context = json!({
            "transcript_summary": [
                {
                    "turn_index": 2,
                    "workflow_id": workflow_id,
                    "assistant_result": {
                        "workflow_id": workflow_id,
                        "turn_index": 2,
                        "status": "completed",
                        "assistant_result": {
                            "preview": "bounded answer preview",
                            "aggregated_result": {
                                "summary": "bounded answer preview"
                            },
                            "proof_outcome": "collapsed_to_sequential"
                        },
                        "preview": "bounded answer preview",
                        "provenance": {
                            "runtime_execution_mode": {
                                "execution_boundary": "tool_bus"
                            },
                            "graph_state": "completed",
                            "graph_id": null,
                            "source_fields": [
                                "metadata.final_result",
                                "metadata.aggregated_result"
                            ]
                        }
                    }
                }
            ]
        });

        let retained_result = retained_result_for_turn(&retained_context, workflow_id, 2)
            .expect("stored retained projection should round-trip");

        assert_eq!(retained_result.workflow_id, workflow_id);
        assert_eq!(retained_result.turn_index, 2);
        assert_eq!(
            retained_result.assistant_result["proof_outcome"],
            json!("collapsed_to_sequential")
        );
    }

    #[test]
    fn last_retained_result_from_context_preserves_stored_proof_outcome() {
        let workflow_id = TaskId::new();
        let retained_context = json!({
            "last_assistant_result": {
                "workflow_id": workflow_id,
                "turn_index": 3,
                "status": "failed",
                "assistant_result": {
                    "preview": "planner failed before graph formation",
                    "aggregated_result": {
                        "error": "planner failed before graph formation"
                    },
                    "proof_outcome": "failed_before_graph"
                },
                "preview": "planner failed before graph formation",
                "provenance": {
                    "runtime_execution_mode": {
                        "execution_boundary": "tool_bus"
                    },
                    "graph_state": null,
                    "graph_id": null,
                    "source_fields": [
                        "metadata.final_result",
                        "metadata.aggregated_result"
                    ]
                }
            }
        });

        let retained_result = last_retained_result_from_context(&retained_context)
            .expect("last retained result should round-trip");

        assert_eq!(retained_result.workflow_id, workflow_id);
        assert_eq!(retained_result.turn_index, 3);
        assert_eq!(
            retained_result.assistant_result["proof_outcome"],
            json!("failed_before_graph")
        );
    }

    #[test]
    fn render_turn_accepted_surfaces_all_identifiers() {
        let view = ConversationCliTurnAccepted {
            session_id: "11111111-1111-1111-1111-111111111111".to_string(),
            workflow_id: "22222222-2222-2222-2222-222222222222".to_string(),
            coordinator_agent_id: "33333333-3333-3333-3333-333333333333".to_string(),
            turn_index: 2,
            status: "queued".to_string(),
        };

        let rendered = render_turn_accepted(&view);
        assert!(rendered.contains("session_id: 11111111-1111-1111-1111-111111111111"));
        assert!(rendered.contains("workflow_id: 22222222-2222-2222-2222-222222222222"));
        assert!(rendered.contains("turn_index: 2"));
        assert!(rendered.contains("status: queued"));
    }

    #[test]
    fn resume_provenance_derives_from_existing_turn_metadata() {
        let resumed_from_workflow = "44444444-4444-4444-4444-444444444444".to_string();
        let metadata = json!({
            "turn_index": 2,
            "retained_context": {
                "latest_workflow_id": resumed_from_workflow,
                "transcript_summary": [
                    {
                        "turn_index": 1,
                        "workflow_id": "44444444-4444-4444-4444-444444444444",
                        "assistant_result": {
                            "workflow_id": "44444444-4444-4444-4444-444444444444",
                            "turn_index": 1,
                            "status": "failed",
                            "assistant_result": {
                                "recovered_after_restart": true
                            },
                            "preview": "workflow interrupted",
                            "provenance": {
                                "runtime_execution_mode": {
                                    "execution_boundary": "tool_bus"
                                },
                                "source_fields": [
                                    "metadata.final_result",
                                    "metadata.aggregated_result"
                                ]
                            }
                        }
                    }
                ]
            }
        });

        let provenance = crate::autonomy::resume_provenance_from_metadata(&metadata)
            .expect("resume provenance should derive from retained context");

        assert!(provenance.resumed_after_restart);
        assert_eq!(provenance.resumed_from_turn_index, Some(1));
        assert_eq!(
            provenance
                .resumed_from_workflow_id
                .map(|value| value.to_string()),
            Some("44444444-4444-4444-4444-444444444444".to_string())
        );
    }

    #[test]
    fn render_session_surfaces_restart_resume_provenance() {
        let view = ConversationCliSessionView {
            session_id: "11111111-1111-1111-1111-111111111111".to_string(),
            status: "active".to_string(),
            coordinator_agent_id: "33333333-3333-3333-3333-333333333333".to_string(),
            provider_kind: "openai_chatgpt".to_string(),
            model_id: "gpt-5.4".to_string(),
            active_workflow_id: None,
            last_completed_workflow_id: Some("55555555-5555-5555-5555-555555555555".to_string()),
            turn_count: 2,
            last_assistant_result: Some(SessionRetainedResultView {
                workflow_id: TaskId::from_uuid(
                    Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap(),
                ),
                turn_index: 2,
                status: "completed".to_string(),
                assistant_result: json!({
                    "preview": "bounded answer preview",
                    "aggregated_result": {
                        "summary": "bounded answer preview"
                    },
                    "proof_outcome": "collapsed_to_sequential"
                }),
                preview: Some("bounded answer preview".to_string()),
                runtime_truth: Some(mister_smith_core::RuntimeTruthView {
                    evidence_class:
                        mister_smith_core::ExecutionEvidenceClass::PlaceholderOrSimulatedStepCompletion,
                    proof_boundary: mister_smith_core::packet_023_placeholder_proof_boundary(),
                    run_trace: mister_smith_core::RunTraceSummaryView {
                        trace_root_id: "55555555-5555-5555-5555-555555555555".to_string(),
                        workflow_id: TaskId::from_uuid(
                            Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap(),
                        ),
                        graph_id: None,
                        branch_id: None,
                        node_id: None,
                        relationships: vec![mister_smith_core::RunTraceRelationshipKind::Graph],
                    },
                    grounded_evidence: vec![],
                }),
                provenance: mister_smith_core::ResultProvenanceSummary {
                    runtime_execution_mode: json!({
                        "execution_boundary": "tool_bus"
                    }),
                    graph_state: None,
                    graph_id: None,
                    source_fields: vec![
                        "metadata.final_result".to_string(),
                        "metadata.aggregated_result".to_string(),
                    ],
                },
            }),
            turns: vec![
                ConversationCliTurnSummary {
                    turn_index: 1,
                    workflow_id: "44444444-4444-4444-4444-444444444444".to_string(),
                    status: "failed".to_string(),
                    lifecycle_state: DurableWorkflowLifecycleState::Failed,
                    user_message: "turn one".to_string(),
                    assistant_result: Some(SessionRetainedResultView {
                        workflow_id: TaskId::from_uuid(
                            Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap(),
                        ),
                        turn_index: 1,
                        status: "failed".to_string(),
                        assistant_result: json!({
                            "preview": "workflow interrupted by runtime restart before session sync",
                            "aggregated_result": {
                                "error": "workflow interrupted by runtime restart before session sync",
                                "recovered_after_restart": true
                            },
                            "proof_outcome": "failed_before_graph",
                            "recovered_after_restart": true
                        }),
                        preview: Some(
                            "workflow interrupted by runtime restart before session sync"
                                .to_string(),
                        ),
                        runtime_truth: None,
                        provenance: mister_smith_core::ResultProvenanceSummary {
                            runtime_execution_mode: json!({
                                "execution_boundary": "tool_bus"
                            }),
                            graph_state: None,
                            graph_id: None,
                            source_fields: vec![
                                "metadata.final_result".to_string(),
                                "metadata.aggregated_result".to_string(),
                            ],
                        },
                    }),
                    resume_provenance: Some(ConversationCliResumeProvenanceView {
                        recovered_after_restart: true,
                        resumed_after_restart: false,
                        recovered_at: Some("2026-03-17T21:00:00+00:00".to_string()),
                        recovery_reason: Some(
                            "workflow interrupted by runtime restart before session sync"
                                .to_string(),
                        ),
                        resumed_from_workflow_id: None,
                        resumed_from_turn_index: None,
                    }),
                },
                ConversationCliTurnSummary {
                    turn_index: 2,
                    workflow_id: "55555555-5555-5555-5555-555555555555".to_string(),
                    status: "completed".to_string(),
                    lifecycle_state: DurableWorkflowLifecycleState::Completed,
                    user_message: "turn two".to_string(),
                    assistant_result: Some(SessionRetainedResultView {
                        workflow_id: TaskId::from_uuid(
                            Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap(),
                        ),
                        turn_index: 2,
                        status: "completed".to_string(),
                        assistant_result: json!({
                            "preview": "bounded answer preview",
                            "aggregated_result": {
                                "summary": "bounded answer preview"
                            },
                            "proof_outcome": "collapsed_to_sequential"
                        }),
                        preview: Some("bounded answer preview".to_string()),
                        runtime_truth: Some(mister_smith_core::RuntimeTruthView {
                            evidence_class:
                                mister_smith_core::ExecutionEvidenceClass::PlaceholderOrSimulatedStepCompletion,
                            proof_boundary: mister_smith_core::packet_023_placeholder_proof_boundary(),
                            run_trace: mister_smith_core::RunTraceSummaryView {
                                trace_root_id:
                                    "55555555-5555-5555-5555-555555555555".to_string(),
                                workflow_id: TaskId::from_uuid(
                                    Uuid::parse_str(
                                        "55555555-5555-5555-5555-555555555555",
                                    )
                                    .unwrap(),
                                ),
                                graph_id: None,
                                branch_id: None,
                                node_id: None,
                                relationships: vec![
                                    mister_smith_core::RunTraceRelationshipKind::Graph,
                                    mister_smith_core::RunTraceRelationshipKind::ToolBoundary,
                                ],
                            },
                            grounded_evidence: vec![],
                        }),
                        provenance: mister_smith_core::ResultProvenanceSummary {
                            runtime_execution_mode: json!({
                                "execution_boundary": "tool_bus"
                            }),
                            graph_state: None,
                            graph_id: None,
                            source_fields: vec![
                                "metadata.final_result".to_string(),
                                "metadata.aggregated_result".to_string(),
                            ],
                        },
                    }),
                    resume_provenance: Some(ConversationCliResumeProvenanceView {
                        recovered_after_restart: false,
                        resumed_after_restart: true,
                        recovered_at: None,
                        recovery_reason: None,
                        resumed_from_workflow_id: Some(
                            "44444444-4444-4444-4444-444444444444".to_string(),
                        ),
                        resumed_from_turn_index: Some(1),
                    }),
                },
            ],
            ended_at: None,
        };

        let rendered = render_session(&view);

        assert!(rendered.contains("resume=recovered_after_restart=true"));
        assert!(
            rendered.contains("reason=workflow interrupted by runtime restart before session sync")
        );
        assert!(rendered.contains("resume=resumed_after_restart=true resumed_from_turn=1"));
        assert!(rendered.contains("resumed_from_workflow=44444444-4444-4444-4444-444444444444"));
        assert!(rendered
            .contains("last_assistant_result: workflow=55555555-5555-5555-5555-555555555555"));
        assert!(rendered.contains("result=workflow=55555555-5555-5555-5555-555555555555 status=completed proof=collapsed_to_sequential preview=bounded answer preview runtime_truth=placeholder_or_simulated_step_completion:result is orchestration proof, not substantive task proof"));
        assert!(rendered.contains("sources=metadata.final_result|metadata.aggregated_result"));
    }
}
