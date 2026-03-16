//! Durable multi-turn conversation service and CLI helpers.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mister_smith_config::FrameworkConfig;
use mister_smith_core::{AgentId, SessionId, SessionStatus, TaskId};
use mister_smith_http::server::{
    ConversationContinueRequest, ConversationCreateRequest, ConversationEndView,
    ConversationServiceError, ConversationSessionService, ConversationSessionView,
    ConversationTurnAccepted, ConversationTurnContext, ConversationTurnSummaryView,
    TaskSubmissionRequest,
};
use mister_smith_persistence::postgres::queries::{self, TaskRecord};
use mister_smith_persistence::{SessionRecord, SessionRepository, SessionTurnRecord};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::execution::{RuntimeTaskService, MODEL_ID, PROVIDER_KIND_NAME};

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
            let task = self.load_task(workflow_id).await?;
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
        };

        self.runtime_task_service
            .prepare_workflow(workflow_id, &task_request)
            .await
            .map_err(ConversationServiceError::Internal)?;

        let session = SessionRecord {
            session_id: *session_id.as_ref(),
            coordinator_agent_id: *coordinator_agent_id.as_ref(),
            status: session_status_text(SessionStatus::Active).to_string(),
            provider_kind: PROVIDER_KIND_NAME.to_string(),
            model_id: MODEL_ID.to_string(),
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
        Ok(Some(build_session_view(session, turns)))
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
}

fn build_session_view(
    session: SessionRecord,
    turns: Vec<SessionTurnRecord>,
) -> ConversationSessionView {
    ConversationSessionView {
        session_id: SessionId::from_uuid(session.session_id),
        status: parse_session_status(&session.status),
        coordinator_agent_id: AgentId::from_uuid(session.coordinator_agent_id),
        provider_kind: session.provider_kind,
        model_id: session.model_id,
        active_workflow_id: session.active_workflow_id.map(TaskId::from_uuid),
        last_completed_workflow_id: session.last_completed_workflow_id.map(TaskId::from_uuid),
        turn_count: session.turn_count.max(0) as u32,
        turns: turns
            .into_iter()
            .map(|turn| ConversationTurnSummaryView {
                turn_index: turn.turn_index.max(0) as u32,
                workflow_id: TaskId::from_uuid(turn.workflow_id),
                status: turn.status,
                user_message: turn.user_message,
            })
            .collect(),
        ended_at: session.ended_at,
    }
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
        "completed" | "failed" | "cancelled"
    )
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

    let assistant_result = result_summary.unwrap_or_else(|| json!({ "status": turn.status }));
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
    pub turns: Vec<ConversationCliTurnSummary>,
    pub ended_at: Option<String>,
}

/// Client-facing turn summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliTurnSummary {
    pub turn_index: u32,
    pub workflow_id: String,
    pub status: String,
    pub user_message: String,
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
            format!(
                "  {} {} {} {}",
                turn.turn_index, turn.workflow_id, turn.status, turn.user_message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "session_id: {}\nstatus: {}\ncoordinator_agent_id: {}\nprovider_kind: {}\nmodel_id: {}\nactive_workflow_id: {}\nlast_completed_workflow_id: {}\nturn_count: {}\nended_at: {}\nturns:\n{}",
        view.session_id,
        view.status,
        view.coordinator_agent_id,
        view.provider_kind,
        view.model_id,
        view.active_workflow_id.as_deref().unwrap_or("none"),
        view.last_completed_workflow_id.as_deref().unwrap_or("none"),
        view.turn_count,
        view.ended_at.as_deref().unwrap_or("none"),
        if turns.is_empty() { "none".to_string() } else { turns }
    )
}

pub(crate) fn render_end_view(view: &ConversationCliEndView) -> String {
    format!(
        "session_id: {}\nstatus: {}\nended_at: {}",
        view.session_id, view.status, view.ended_at
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
}
