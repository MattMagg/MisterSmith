//! Durable multi-turn conversation service and CLI helpers.

use std::collections::HashMap;
use std::env;
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
    ConversationContinueRequest, ConversationCreateRequest, ConversationCurrentTurnStateView,
    ConversationEndView, ConversationLoopState, ConversationResumeProvenanceView,
    ConversationServiceError, ConversationSessionControlUpdateRequest,
    ConversationSessionControlView, ConversationSessionService, ConversationSessionSummaryView,
    ConversationSessionView, ConversationSupportNoticeView, ConversationTurnAccepted,
    ConversationTurnContext, ConversationTurnStateSource, ConversationTurnSummaryView,
    SessionListRequest, TaskSubmissionRequest,
};
use mister_smith_persistence::postgres::pool::PostgresConnection;
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

    async fn update_session_control_state(
        &self,
        session_id: SessionId,
        request: ConversationSessionControlUpdateRequest,
    ) -> Result<ConversationSessionControlView, ConversationServiceError> {
        let (mut session, _) = self.sync_session(session_id).await?;
        if matches!(parse_session_status(&session.status), SessionStatus::Ended) {
            return Err(ConversationServiceError::SessionEnded { session_id });
        }

        let mut control_state =
            session_control_state_from_context(&session.retained_context, &session);
        validate_and_apply_control_updates(&mut control_state, &request)?;

        session.retained_context =
            upsert_session_control_state(&session.retained_context, &control_state);
        session.updated_at = Utc::now();
        self.session_repository
            .update_session(&session)
            .await
            .map_err(persistence_error)?;

        Ok(control_state)
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
    let control_state = session_control_state_from_context(&session.retained_context, &session);
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

    let support_notices =
        session_support_notices(&session, &control_state, last_assistant_result.as_ref());
    let state_source = if session.active_workflow_id.is_none() {
        ConversationTurnStateSource::RetainedSession
    } else {
        ConversationTurnStateSource::LiveRuntime
    };
    let mut current_turn_state =
        current_turn_state_for_loop(&session, &turn_summaries, state_source);
    let loop_state = loop_state_for_session(&session, current_turn_state.as_ref());
    let next_action_hint = next_action_for_loop_state(loop_state).to_string();

    // Override next_action_hint when the session/loop has ended
    if loop_state == ConversationLoopState::Ended {
        if let Some(ref mut turn_state) = current_turn_state {
            turn_state.next_action_hint =
                next_action_for_loop_state(ConversationLoopState::Ended).to_string();
        }
    }

    Ok(ConversationSessionView {
        title: session_title(&session.retained_context, &session),
        session_id: SessionId::from_uuid(session.session_id),
        status: parse_session_status(&session.status),
        loop_state,
        coordinator_agent_id: AgentId::from_uuid(session.coordinator_agent_id),
        provider_kind: session.provider_kind,
        model_id: session.model_id,
        active_workflow_id: session.active_workflow_id.map(TaskId::from_uuid),
        last_completed_workflow_id: session.last_completed_workflow_id.map(TaskId::from_uuid),
        turn_count: session.turn_count.max(0) as u32,
        last_assistant_result,
        current_turn_state,
        turns: turn_summaries,
        control_state,
        support_notices,
        next_action_hint,
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

const DEFAULT_PERMISSION_MODE: &str = "default";
const DEFAULT_CONFIG_POSTURE: &str = "inline";
const DEFAULT_STATUS_VIEW: &str = "summary";
const DEFAULT_MCP_POSTURE: &str = "support_only";

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

fn last_user_message_from_context(retained_context: &Value) -> Option<String> {
    retained_context
        .get("last_user_message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn compact_title(value: &str) -> String {
    let title = value.trim().replace('\n', " ");
    let mut chars = title.chars();
    let compact: String = chars.by_ref().take(72).collect();
    if chars.next().is_some() {
        format!("{compact}...")
    } else {
        compact
    }
}

fn session_title(retained_context: &Value, session: &SessionRecord) -> String {
    if let Some(user_message) = last_user_message_from_context(retained_context) {
        return compact_title(&user_message);
    }
    if let Some(preview) = last_preview_from_context(retained_context) {
        return compact_title(&preview);
    }

    format!("session {}", &session.session_id.to_string()[..8])
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn session_control_state_from_context(
    retained_context: &Value,
    session: &SessionRecord,
) -> ConversationSessionControlView {
    let stored = retained_context.get("session_control_state");

    ConversationSessionControlView {
        session_id: SessionId::from_uuid(session.session_id),
        selected_provider_kind: normalized_optional_string(
            stored
                .and_then(|value| value.get("selected_provider_kind"))
                .and_then(Value::as_str),
        ),
        selected_model_id: normalized_optional_string(
            stored
                .and_then(|value| value.get("selected_model_id"))
                .and_then(Value::as_str),
        ),
        permission_mode: normalized_optional_string(
            stored
                .and_then(|value| value.get("permission_mode"))
                .and_then(Value::as_str),
        )
        .unwrap_or_else(|| DEFAULT_PERMISSION_MODE.to_string()),
        config_posture: normalized_optional_string(
            stored
                .and_then(|value| value.get("config_posture"))
                .and_then(Value::as_str),
        )
        .unwrap_or_else(|| DEFAULT_CONFIG_POSTURE.to_string()),
        status_view: normalized_optional_string(
            stored
                .and_then(|value| value.get("status_view"))
                .and_then(Value::as_str),
        )
        .unwrap_or_else(|| DEFAULT_STATUS_VIEW.to_string()),
        mcp_posture: normalized_optional_string(
            stored
                .and_then(|value| value.get("mcp_posture"))
                .and_then(Value::as_str),
        )
        .unwrap_or_else(|| DEFAULT_MCP_POSTURE.to_string()),
    }
}

fn upsert_session_control_state(
    retained_context: &Value,
    control_state: &ConversationSessionControlView,
) -> Value {
    let mut updated = retained_context.clone();
    if updated.is_null() || !updated.is_object() {
        updated = empty_retained_context();
    }

    if let Some(object) = updated.as_object_mut() {
        object.insert(
            "session_control_state".to_string(),
            json!({
                "selected_provider_kind": control_state.selected_provider_kind,
                "selected_model_id": control_state.selected_model_id,
                "permission_mode": control_state.permission_mode,
                "config_posture": control_state.config_posture,
                "status_view": control_state.status_view,
                "mcp_posture": control_state.mcp_posture,
            }),
        );
    }

    updated
}

fn session_support_notices(
    session: &SessionRecord,
    control_state: &ConversationSessionControlView,
    last_assistant_result: Option<&SessionRetainedResultView>,
) -> Vec<ConversationSupportNoticeView> {
    let mut notices = Vec::new();
    let session_ended = matches!(parse_session_status(&session.status), SessionStatus::Ended);

    if session.active_workflow_id.is_some() {
        notices.push(ConversationSupportNoticeView {
            notice_kind: "busy".to_string(),
            severity: "info".to_string(),
            summary:
                "Another live turn is already active in this session. Stay here to watch it or wait before sending the next follow-up."
                    .to_string(),
            support_surface: Some("status".to_string()),
            blocks_live_turn: true,
            allowed_next_action: "wait for the current turn to finish or inspect the current state"
                .to_string(),
        });
    }

    if session_ended {
        notices.push(ConversationSupportNoticeView {
            notice_kind: "ended".to_string(),
            severity: "warning".to_string(),
            summary:
                "This session is ended. Retained context is still visible, but it cannot accept another live turn."
                    .to_string(),
            support_surface: Some("resume".to_string()),
            blocks_live_turn: true,
            allowed_next_action:
                "start a new session or resume a different retained session".to_string(),
        });
    }

    if last_assistant_result.is_some() {
        let (summary, support_surface, blocks_live_turn, allowed_next_action) = if session_ended {
            (
                "This loop shows bounded retained previews with explicit proof limits from an ended session. It is not a fresh live-proof claim."
                    .to_string(),
                Some("resume".to_string()),
                true,
                "read the retained result, or start a new session or resume a different retained session"
                    .to_string(),
            )
        } else {
            (
                "This loop shows bounded retained previews with explicit proof limits. It is not a fresh live-proof claim."
                    .to_string(),
                Some("status".to_string()),
                false,
                "read the retained result or send a follow-up turn when the loop is ready"
                    .to_string(),
            )
        };
        notices.push(ConversationSupportNoticeView {
            notice_kind: "proof_limited".to_string(),
            severity: "info".to_string(),
            summary,
            support_surface,
            blocks_live_turn,
            allowed_next_action,
        });
    }

    if control_state.selected_model_id.is_some()
        || control_state.selected_provider_kind.is_some()
        || control_state.permission_mode != DEFAULT_PERMISSION_MODE
        || control_state.mcp_posture != DEFAULT_MCP_POSTURE
    {
        let (summary, support_surface, blocks_live_turn, allowed_next_action) = if session_ended {
            (
                "Stored shell control preferences are shown for this ended session, but it cannot accept new live turns or control edits."
                    .to_string(),
                Some("resume".to_string()),
                true,
                "start a new session or resume a different retained session".to_string(),
            )
        } else {
            (
                "Shell control changes are stored with this session, but runtime execution still follows the active runtime path."
                    .to_string(),
                Some("config".to_string()),
                false,
                "keep working in this session or adjust the support posture".to_string(),
            )
        };
        notices.push(ConversationSupportNoticeView {
            notice_kind: "session_shell_preferences".to_string(),
            severity: "warning".to_string(),
            summary,
            support_surface,
            blocks_live_turn,
            allowed_next_action,
        });
    }

    notices
}

fn normalized_turn_status(status: &str) -> &str {
    match status.trim().to_ascii_lowercase().as_str() {
        "queued" | "pending" => "accepted",
        "running" | "active" => "running",
        "completed" => "completed",
        "failed" | "cancelled" | "terminated" => "failed",
        _ => "running",
    }
}

fn retained_result_preview(result: &SessionRetainedResultView) -> Option<String> {
    result.preview.clone().or_else(|| {
        result
            .assistant_result
            .get("preview")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn retained_result_proof_boundary(result: &SessionRetainedResultView) -> Option<String> {
    result
        .runtime_truth
        .as_ref()
        .map(|runtime_truth| runtime_truth.proof_boundary.task_proof.clone())
        .or_else(|| {
            result
                .assistant_result
                .get("runtime_truth")
                .and_then(|runtime_truth| runtime_truth.get("proof_boundary"))
                .and_then(|proof_boundary| proof_boundary.get("task_proof"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn next_action_for_turn_status(turn_status: &str) -> &'static str {
    match turn_status {
        "accepted" => "stay in this session while the accepted turn starts",
        "running" => "stay in this session while live work continues",
        "completed" => "send a follow-up turn or adjust the session controls",
        "failed" => "send a clarifying follow-up or start a new session",
        "blocked" => "address the blocking notice, then retry from this session",
        _ => "stay in this session and inspect the current state",
    }
}

fn current_turn_state_for_loop(
    session: &SessionRecord,
    turns: &[ConversationTurnSummaryView],
    state_source: ConversationTurnStateSource,
) -> Option<ConversationCurrentTurnStateView> {
    let focused_turn = if let Some(active_workflow_id) = session.active_workflow_id {
        turns
            .iter()
            .find(|turn| turn.workflow_id == TaskId::from_uuid(active_workflow_id))
    } else {
        turns.last()
    }?;

    let turn_status = normalized_turn_status(&focused_turn.status).to_string();

    let (result_preview, proof_boundary_note) = focused_turn
        .assistant_result
        .as_ref()
        .map(|result| {
            (
                retained_result_preview(result),
                retained_result_proof_boundary(result),
            )
        })
        .unwrap_or((None, None));

    Some(ConversationCurrentTurnStateView {
        workflow_id: focused_turn.workflow_id,
        turn_index: focused_turn.turn_index,
        turn_status: turn_status.clone(),
        lifecycle_state: focused_turn.lifecycle_state,
        result_preview,
        proof_boundary_note,
        state_source,
        next_action_hint: next_action_for_turn_status(&turn_status).to_string(),
    })
}

fn loop_state_for_session(
    session: &SessionRecord,
    current_turn_state: Option<&ConversationCurrentTurnStateView>,
) -> ConversationLoopState {
    if matches!(parse_session_status(&session.status), SessionStatus::Ended) {
        return ConversationLoopState::Ended;
    }

    if session.active_workflow_id.is_some() {
        return match current_turn_state.map(|turn| turn.turn_status.as_str()) {
            Some("accepted") => ConversationLoopState::TurnPending,
            _ => ConversationLoopState::TurnRunning,
        };
    }

    ConversationLoopState::Ready
}

fn next_action_for_loop_state(loop_state: ConversationLoopState) -> &'static str {
    match loop_state {
        ConversationLoopState::Ready => {
            "send a follow-up turn or adjust the session controls from this loop"
        }
        ConversationLoopState::TurnPending => {
            "wait for the accepted turn to start or inspect the current session state"
        }
        ConversationLoopState::TurnRunning => {
            "stay in this session while the active turn runs, then follow up here"
        }
        ConversationLoopState::Blocked => {
            "address the blocking notice, then retry from the same session"
        }
        ConversationLoopState::Degraded => {
            "resume later or use the support surfaces until the runtime becomes available again"
        }
        ConversationLoopState::Ended => {
            "start a new session or resume a different retained session"
        }
    }
}

fn validate_permission_mode(value: &str) -> Result<(), ConversationServiceError> {
    if matches!(value, "default" | "review" | "full") {
        return Ok(());
    }

    Err(ConversationServiceError::BadRequest(format!(
        "invalid permission mode '{value}'; expected default, review, or full"
    )))
}

fn validate_config_posture(value: &str) -> Result<(), ConversationServiceError> {
    if matches!(value, "inline" | "support") {
        return Ok(());
    }

    Err(ConversationServiceError::BadRequest(format!(
        "invalid config posture '{value}'; expected inline or support"
    )))
}

fn validate_status_view(value: &str) -> Result<(), ConversationServiceError> {
    if matches!(value, "summary" | "detail") {
        return Ok(());
    }

    Err(ConversationServiceError::BadRequest(format!(
        "invalid status view '{value}'; expected summary or detail"
    )))
}

fn validate_mcp_posture(value: &str) -> Result<(), ConversationServiceError> {
    if matches!(value, "connected" | "support_only" | "detached") {
        return Ok(());
    }

    Err(ConversationServiceError::BadRequest(format!(
        "invalid mcp posture '{value}'; expected connected, support_only, or detached"
    )))
}

fn validate_and_apply_control_updates(
    control_state: &mut ConversationSessionControlView,
    request: &ConversationSessionControlUpdateRequest,
) -> Result<(), ConversationServiceError> {
    if request.clear_selected_provider_kind {
        control_state.selected_provider_kind = None;
    } else if let Some(value) =
        normalized_optional_string(request.selected_provider_kind.as_deref())
    {
        control_state.selected_provider_kind = Some(value);
    }
    if request.clear_selected_model_id {
        control_state.selected_model_id = None;
    } else if let Some(value) = normalized_optional_string(request.selected_model_id.as_deref()) {
        control_state.selected_model_id = Some(value);
    }
    if let Some(value) = normalized_optional_string(request.permission_mode.as_deref()) {
        validate_permission_mode(&value)?;
        control_state.permission_mode = value;
    }
    if let Some(value) = normalized_optional_string(request.config_posture.as_deref()) {
        validate_config_posture(&value)?;
        control_state.config_posture = value;
    }
    if let Some(value) = normalized_optional_string(request.status_view.as_deref()) {
        validate_status_view(&value)?;
        control_state.status_view = value;
    }
    if let Some(value) = normalized_optional_string(request.mcp_posture.as_deref()) {
        validate_mcp_posture(&value)?;
        control_state.mcp_posture = value;
    }
    Ok(())
}

fn build_session_summary_view(session: &SessionRecord) -> ConversationSessionSummaryView {
    ConversationSessionSummaryView {
        title: session_title(&session.retained_context, session),
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
    pub title: String,
    pub session_id: String,
    pub status: String,
    #[serde(default = "default_cli_loop_state")]
    pub loop_state: String,
    pub coordinator_agent_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub active_workflow_id: Option<String>,
    pub last_completed_workflow_id: Option<String>,
    pub turn_count: u32,
    #[serde(default)]
    pub last_assistant_result: Option<SessionRetainedResultView>,
    #[serde(default)]
    pub current_turn_state: Option<ConversationCliCurrentTurnStateView>,
    #[serde(default)]
    pub turns: Vec<ConversationCliTurnSummary>,
    #[serde(default)]
    pub control_state: ConversationCliSessionControlState,
    #[serde(default)]
    pub support_notices: Vec<ConversationCliSupportNoticeView>,
    #[serde(default = "default_cli_next_action")]
    pub next_action_hint: String,
    pub ended_at: Option<String>,
}

fn default_cli_loop_state() -> String {
    ConversationLoopState::Ready.as_str().to_string()
}

fn default_cli_next_action() -> String {
    next_action_for_loop_state(ConversationLoopState::Ready).to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliCurrentTurnStateView {
    pub workflow_id: String,
    pub turn_index: u32,
    pub turn_status: String,
    pub lifecycle_state: DurableWorkflowLifecycleState,
    #[serde(default)]
    pub result_preview: Option<String>,
    #[serde(default)]
    pub proof_boundary_note: Option<String>,
    pub state_source: String,
    pub next_action_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliSessionControlState {
    pub selected_provider_kind: Option<String>,
    pub selected_model_id: Option<String>,
    pub permission_mode: String,
    pub config_posture: String,
    pub status_view: String,
    pub mcp_posture: String,
}

impl Default for ConversationCliSessionControlState {
    fn default() -> Self {
        Self {
            selected_provider_kind: None,
            selected_model_id: None,
            permission_mode: DEFAULT_PERMISSION_MODE.to_string(),
            config_posture: DEFAULT_CONFIG_POSTURE.to_string(),
            status_view: DEFAULT_STATUS_VIEW.to_string(),
            mcp_posture: DEFAULT_MCP_POSTURE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliSupportNoticeView {
    pub notice_kind: String,
    pub severity: String,
    pub summary: String,
    #[serde(default)]
    pub support_surface: Option<String>,
    #[serde(default)]
    pub blocks_live_turn: bool,
    #[serde(default)]
    pub allowed_next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationCliSessionSummaryView {
    pub title: String,
    pub session_id: String,
    pub status: String,
    pub coordinator_agent_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub active_workflow_id: Option<String>,
    pub last_completed_workflow_id: Option<String>,
    pub turn_count: u32,
    pub updated_at: String,
    pub ended_at: Option<String>,
    pub last_preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConversationCliStartupHomeView {
    pub recent_sessions: Vec<ConversationCliSessionSummaryView>,
    pub resume_last_session_id: Option<String>,
    pub startup_warnings: Vec<ConversationCliSupportNoticeView>,
    pub provider_kind: String,
    pub model_id: String,
    pub config_action: String,
    pub session_source: String,
    pub runtime_available: bool,
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
    StorageUnavailable(String),
    Http(reqwest::Error),
    HttpStatus(StatusCode, String),
}

impl fmt::Display for ConversationClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationClientError::InvalidSessionId(raw) => {
                write!(f, "invalid session id '{raw}'")
            }
            ConversationClientError::StorageUnavailable(message) => write!(f, "{message}"),
            ConversationClientError::Http(error) => write!(f, "{error}"),
            ConversationClientError::HttpStatus(status, body) => {
                write!(f, "runtime returned {}: {}", status.as_u16(), body)
            }
        }
    }
}

impl Error for ConversationClientError {}

pub(crate) fn should_fallback_to_direct_session_store(error: &ConversationClientError) -> bool {
    match error {
        ConversationClientError::Http(error) => error.is_connect() || error.is_timeout(),
        ConversationClientError::InvalidSessionId(_)
        | ConversationClientError::StorageUnavailable(_)
        | ConversationClientError::HttpStatus(_, _) => false,
    }
}

pub(crate) fn parse_session_id(raw: &str) -> Result<SessionId, ConversationClientError> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| ConversationClientError::InvalidSessionId(raw.to_string()))
}

pub(crate) fn default_base_url(config: &FrameworkConfig) -> String {
    let port = config.transport.http_port.unwrap_or(8080);
    format!("http://127.0.0.1:{port}")
}

#[derive(Debug, Deserialize)]
struct ConversationHealthResponse {
    status: String,
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

pub(crate) async fn list_sessions_http(
    base_url: &str,
    limit: usize,
) -> Result<Vec<ConversationCliSessionSummaryView>, ConversationClientError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/sessions?limit={limit}",
        base_url.trim_end_matches('/')
    );
    let response = client
        .get(url)
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

pub(crate) async fn update_session_control_http(
    base_url: &str,
    session_id: SessionId,
    request: ConversationSessionControlUpdateRequest,
) -> Result<ConversationCliSessionControlState, ConversationClientError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/sessions/{}/controls",
        base_url.trim_end_matches('/'),
        session_id
    );
    let response = client
        .post(url)
        .json(&request)
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

async fn runtime_available_http(base_url: &str) -> bool {
    let client = Client::new();
    let url = format!("{}/api/v1/health", base_url.trim_end_matches('/'));
    match client.get(url).send().await {
        Ok(response) if response.status().is_success() => response
            .json::<ConversationHealthResponse>()
            .await
            .map(|body| body.status.eq_ignore_ascii_case("healthy"))
            .unwrap_or(false),
        _ => false,
    }
}

fn session_store_url() -> Result<String, ConversationClientError> {
    for name in ["MISTER_SMITH_DATABASE_URL", "DATABASE_URL"] {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
    }

    Err(ConversationClientError::StorageUnavailable(
        "MISTER_SMITH_DATABASE_URL or DATABASE_URL must be set, so the CLI can read retained sessions directly."
            .to_string(),
    ))
}

async fn connect_session_store() -> Result<PostgresConnection, ConversationClientError> {
    let url = session_store_url()?;
    PostgresConnection::connect(&url).await.map_err(|error| {
        ConversationClientError::StorageUnavailable(format!(
            "failed to connect to the retained session store: {error}"
        ))
    })
}

fn cli_control_state_from_view(
    view: ConversationSessionControlView,
) -> ConversationCliSessionControlState {
    ConversationCliSessionControlState {
        selected_provider_kind: view.selected_provider_kind,
        selected_model_id: view.selected_model_id,
        permission_mode: view.permission_mode,
        config_posture: view.config_posture,
        status_view: view.status_view,
        mcp_posture: view.mcp_posture,
    }
}

fn cli_notice_from_view(view: ConversationSupportNoticeView) -> ConversationCliSupportNoticeView {
    ConversationCliSupportNoticeView {
        notice_kind: view.notice_kind,
        severity: view.severity,
        summary: view.summary,
        support_surface: view.support_surface,
        blocks_live_turn: view.blocks_live_turn,
        allowed_next_action: view.allowed_next_action,
    }
}

fn cli_summary_from_view(
    view: ConversationSessionSummaryView,
) -> ConversationCliSessionSummaryView {
    ConversationCliSessionSummaryView {
        title: view.title,
        session_id: view.session_id.to_string(),
        status: session_status_text(view.status).to_string(),
        coordinator_agent_id: view.coordinator_agent_id.to_string(),
        provider_kind: view.provider_kind,
        model_id: view.model_id,
        active_workflow_id: view.active_workflow_id.map(|value| value.to_string()),
        last_completed_workflow_id: view
            .last_completed_workflow_id
            .map(|value| value.to_string()),
        turn_count: view.turn_count,
        updated_at: view.updated_at.to_rfc3339(),
        ended_at: view.ended_at.map(|value| value.to_rfc3339()),
        last_preview: view.last_preview,
    }
}

fn cli_session_from_view(view: ConversationSessionView) -> ConversationCliSessionView {
    ConversationCliSessionView {
        title: view.title,
        session_id: view.session_id.to_string(),
        status: session_status_text(view.status).to_string(),
        loop_state: view.loop_state.as_str().to_string(),
        coordinator_agent_id: view.coordinator_agent_id.to_string(),
        provider_kind: view.provider_kind,
        model_id: view.model_id,
        active_workflow_id: view.active_workflow_id.map(|value| value.to_string()),
        last_completed_workflow_id: view
            .last_completed_workflow_id
            .map(|value| value.to_string()),
        turn_count: view.turn_count,
        last_assistant_result: view.last_assistant_result,
        current_turn_state: view.current_turn_state.map(|turn| {
            ConversationCliCurrentTurnStateView {
                workflow_id: turn.workflow_id.to_string(),
                turn_index: turn.turn_index,
                turn_status: turn.turn_status,
                lifecycle_state: turn.lifecycle_state,
                result_preview: turn.result_preview,
                proof_boundary_note: turn.proof_boundary_note,
                state_source: turn.state_source.as_str().to_string(),
                next_action_hint: turn.next_action_hint,
            }
        }),
        turns: view
            .turns
            .into_iter()
            .map(|turn| ConversationCliTurnSummary {
                turn_index: turn.turn_index,
                workflow_id: turn.workflow_id.to_string(),
                status: turn.status,
                lifecycle_state: turn.lifecycle_state,
                user_message: turn.user_message,
                assistant_result: turn.assistant_result,
                resume_provenance: turn.resume_provenance.map(|provenance| {
                    ConversationCliResumeProvenanceView {
                        recovered_after_restart: provenance.recovered_after_restart,
                        resumed_after_restart: provenance.resumed_after_restart,
                        recovered_at: provenance.recovered_at.map(|value| value.to_rfc3339()),
                        recovery_reason: provenance.recovery_reason,
                        resumed_from_workflow_id: provenance
                            .resumed_from_workflow_id
                            .map(|value| value.to_string()),
                        resumed_from_turn_index: provenance.resumed_from_turn_index,
                    }
                }),
            })
            .collect(),
        control_state: cli_control_state_from_view(view.control_state),
        support_notices: view
            .support_notices
            .into_iter()
            .map(cli_notice_from_view)
            .collect(),
        next_action_hint: view.next_action_hint,
        ended_at: view.ended_at.map(|value| value.to_rfc3339()),
    }
}

pub(crate) async fn list_sessions_direct(
    limit: usize,
) -> Result<Vec<ConversationCliSessionSummaryView>, ConversationClientError> {
    let connection = connect_session_store().await?;
    let repository = SessionRepository::new(connection.pool().clone());
    let rows = repository
        .list_sessions(None, i64::try_from(limit).unwrap_or(i64::MAX), 0)
        .await
        .map_err(|error| {
            ConversationClientError::StorageUnavailable(format!(
                "failed to read retained sessions directly: {error}"
            ))
        })?;

    Ok(rows
        .into_iter()
        .map(|row| cli_summary_from_view(build_session_summary_view(&row)))
        .collect())
}

pub(crate) async fn inspect_session_direct(
    session_id: SessionId,
) -> Result<ConversationCliSessionView, ConversationClientError> {
    let connection = connect_session_store().await?;
    let repository = SessionRepository::new(connection.pool().clone());
    let session = repository
        .find_session(session_id)
        .await
        .map_err(|error| {
            ConversationClientError::StorageUnavailable(format!(
                "failed to load retained session {session_id}: {error}"
            ))
        })?
        .ok_or_else(|| {
            ConversationClientError::StorageUnavailable(format!(
                "retained session {session_id} was not found in the direct session store"
            ))
        })?;
    let turns = repository.list_turns(session_id).await.map_err(|error| {
        ConversationClientError::StorageUnavailable(format!(
            "failed to load retained turns for session {session_id}: {error}"
        ))
    })?;
    let view = build_session_view(session, turns, connection.pool())
        .await
        .map_err(|error| {
            ConversationClientError::StorageUnavailable(format!(
                "failed to build the retained session view for {session_id}: {error}"
            ))
        })?;
    Ok(cli_session_from_view(view))
}

pub(crate) async fn update_session_control_direct(
    session_id: SessionId,
    request: ConversationSessionControlUpdateRequest,
) -> Result<ConversationCliSessionControlState, ConversationClientError> {
    let connection = connect_session_store().await?;
    let repository = SessionRepository::new(connection.pool().clone());
    let mut session = repository
        .find_session(session_id)
        .await
        .map_err(|error| {
            ConversationClientError::StorageUnavailable(format!(
                "failed to load retained session {session_id}: {error}"
            ))
        })?
        .ok_or_else(|| {
            ConversationClientError::StorageUnavailable(format!(
                "retained session {session_id} was not found in the direct session store"
            ))
        })?;

    if matches!(parse_session_status(&session.status), SessionStatus::Ended) {
        return Err(ConversationClientError::StorageUnavailable(format!(
            "session {session_id} has ended"
        )));
    }

    let mut control_state = session_control_state_from_context(&session.retained_context, &session);
    validate_and_apply_control_updates(&mut control_state, &request)
        .map_err(|error| ConversationClientError::StorageUnavailable(error.to_string()))?;

    session.retained_context = upsert_session_control_state(
        &session.retained_context,
        &ConversationSessionControlView {
            session_id: control_state.session_id,
            selected_provider_kind: control_state.selected_provider_kind.clone(),
            selected_model_id: control_state.selected_model_id.clone(),
            permission_mode: control_state.permission_mode.clone(),
            config_posture: control_state.config_posture.clone(),
            status_view: control_state.status_view.clone(),
            mcp_posture: control_state.mcp_posture.clone(),
        },
    );
    session.updated_at = Utc::now();
    repository.update_session(&session).await.map_err(|error| {
        ConversationClientError::StorageUnavailable(format!(
            "failed to store session shell controls for {session_id}: {error}"
        ))
    })?;

    Ok(cli_control_state_from_view(control_state))
}

pub(crate) async fn resolve_last_session_id(
    base_url: &str,
    _config: &FrameworkConfig,
) -> Result<Option<SessionId>, ConversationClientError> {
    match list_sessions_http(base_url, 1).await {
        Ok(rows) => Ok(rows
            .first()
            .and_then(|row| parse_session_id(&row.session_id).ok())),
        Err(error) if should_fallback_to_direct_session_store(&error) => {
            Ok(list_sessions_direct(1)
                .await
                .ok()
                .and_then(|rows| rows.first().cloned())
                .and_then(|row| parse_session_id(&row.session_id).ok()))
        }
        Err(error) => Err(error),
    }
}

pub(crate) async fn inspect_session_for_cli(
    base_url: &str,
    _config: &FrameworkConfig,
    session_id: SessionId,
) -> Result<ConversationCliSessionView, ConversationClientError> {
    match inspect_session_http(base_url, session_id).await {
        Ok(view) => Ok(view),
        Err(err) if should_fallback_to_direct_session_store(&err) => {
            tracing::warn!("inspect_session_for_cli: HTTP inspect failed: {}", err);
            let mut view = inspect_session_direct(session_id).await?;

            // Only mark as degraded for transport/connection errors
            let should_mark_degraded = match &err {
                ConversationClientError::Http(_) => true,
                ConversationClientError::HttpStatus(status, _) => {
                    // 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout
                    matches!(status.as_u16(), 502..=504)
                }
                _ => false,
            };

            if should_mark_degraded {
                mark_view_as_durable_storage(&mut view);
                view.support_notices.insert(
                    0,
                    ConversationCliSupportNoticeView {
                        notice_kind: "degraded".to_string(),
                        severity: "warning".to_string(),
                        summary:
                            "Runtime is unavailable. This session is shown from durable storage only, so live work cannot continue yet."
                                .to_string(),
                        support_surface: Some("run".to_string()),
                        blocks_live_turn: true,
                        allowed_next_action:
                            next_action_for_loop_state(ConversationLoopState::Degraded).to_string(),
                    },
                );
            } else {
                // For other errors, add a neutral notice without marking as degraded
                view.support_notices.insert(
                    0,
                    ConversationCliSupportNoticeView {
                        notice_kind: "inspect_error".to_string(),
                        severity: "info".to_string(),
                        summary: format!("Session inspection encountered an issue: {}", err),
                        support_surface: None,
                        blocks_live_turn: false,
                        allowed_next_action: view.next_action_hint.clone(),
                    },
                );
            }
            Ok(view)
        }
        Err(err) => Err(err),
    }
}

fn mark_view_as_durable_storage(view: &mut ConversationCliSessionView) {
    view.loop_state = ConversationLoopState::Degraded.as_str().to_string();
    view.next_action_hint = next_action_for_loop_state(ConversationLoopState::Degraded).to_string();
    if let Some(current_turn_state) = view.current_turn_state.as_mut() {
        current_turn_state.state_source = ConversationTurnStateSource::DurableStorage
            .as_str()
            .to_string();
        current_turn_state.next_action_hint =
            next_action_for_loop_state(ConversationLoopState::Degraded).to_string();
    }
}

pub(crate) fn apply_blocked_follow_up_notice(
    view: &mut ConversationCliSessionView,
    error_summary: &str,
) {
    let normalized = error_summary.trim().to_ascii_lowercase();
    let (notice_kind, allowed_next_action) = if normalized.contains("busy") {
        (
            "busy",
            "wait for the current turn to finish or inspect the current state",
        )
    } else if normalized.contains("ended") {
        (
            "ended",
            "start a new session or resume a different retained session",
        )
    } else if normalized.contains("runtime returned 503")
        || normalized.contains("unavailable")
        || normalized.contains("connection refused")
    {
        (
            "degraded",
            "resume later or use the support surfaces until the runtime becomes available again",
        )
    } else {
        (
            "blocked",
            "address the blocking condition, then retry from this session",
        )
    };

    view.loop_state = ConversationLoopState::Blocked.as_str().to_string();
    view.next_action_hint = allowed_next_action.to_string();
    view.support_notices.insert(
        0,
        ConversationCliSupportNoticeView {
            notice_kind: notice_kind.to_string(),
            severity: "warning".to_string(),
            summary: format!("The latest follow-up did not start: {error_summary}"),
            support_surface: Some("status".to_string()),
            blocks_live_turn: true,
            allowed_next_action: allowed_next_action.to_string(),
        },
    );
    if let Some(current_turn_state) = view.current_turn_state.as_mut() {
        current_turn_state.next_action_hint = allowed_next_action.to_string();
    }
}

pub(crate) fn is_session_state_error(error: &ConversationClientError) -> bool {
    match error {
        ConversationClientError::HttpStatus(status, body) => {
            if status.as_u16() == 409 {
                // Conflict errors are session-state related (busy/ended)
                return true;
            }
            let normalized = body.trim().to_ascii_lowercase();
            normalized.contains("busy") || normalized.contains("ended")
        }
        _ => {
            let error_text = error.to_string().to_ascii_lowercase();
            error_text.contains("busy") || error_text.contains("ended")
        }
    }
}

pub(crate) async fn update_session_control_for_cli(
    base_url: &str,
    _config: &FrameworkConfig,
    session_id: SessionId,
    request: ConversationSessionControlUpdateRequest,
) -> Result<ConversationCliSessionControlState, ConversationClientError> {
    match update_session_control_http(base_url, session_id, request.clone()).await {
        Ok(view) => Ok(view),
        Err(e) if should_fallback_to_direct_session_store(&e) => {
            tracing::error!("update_session_control_for_cli: HTTP update failed: {}", e);
            update_session_control_direct(session_id, request).await
        }
        Err(e) => Err(e),
    }
}

pub(crate) async fn build_startup_home(
    base_url: &str,
    config: &FrameworkConfig,
    config_action: String,
    limit: usize,
) -> ConversationCliStartupHomeView {
    let runtime_available = runtime_available_http(base_url).await;
    let (recent_sessions, session_source, mut startup_warnings, discovery_success) =
        match list_sessions_http(base_url, limit).await {
            Ok(rows) => (rows, "runtime_api".to_string(), Vec::new(), true),
            Err(error) if should_fallback_to_direct_session_store(&error) => match list_sessions_direct(limit).await {
                Ok(rows) => (
                    rows,
                    "durable_store".to_string(),
                    vec![ConversationCliSupportNoticeView {
                        notice_kind: "runtime_unavailable".to_string(),
                        severity: "warning".to_string(),
                        summary:
                            "Runtime is unavailable. Recent sessions are shown from durable storage only."
                                .to_string(),
                        support_surface: Some("run".to_string()),
                        blocks_live_turn: true,
                        allowed_next_action:
                            "start the runtime before sending or continuing live work".to_string(),
                    }],
                    true,
                ),
                Err(_) => (
                    Vec::new(),
                    "unavailable".to_string(),
                    vec![ConversationCliSupportNoticeView {
                        notice_kind: "runtime_unavailable".to_string(),
                        severity: "warning".to_string(),
                        summary:
                            "Runtime is unavailable and the retained session store could not be read."
                                .to_string(),
                        support_surface: Some("run".to_string()),
                        blocks_live_turn: true,
                        allowed_next_action:
                            "restore runtime or storage access before continuing".to_string(),
                    }],
                    false,
                ),
            },
            Err(error) => (
                Vec::new(),
                "runtime_api_error".to_string(),
                vec![ConversationCliSupportNoticeView {
                    notice_kind: "session_discovery_failed".to_string(),
                    severity: "warning".to_string(),
                    summary: format!("Recent session discovery failed: {error}"),
                    support_surface: Some("run".to_string()),
                    blocks_live_turn: false,
                    allowed_next_action:
                        "inspect the runtime status or retry recent session discovery"
                            .to_string(),
                }],
                false,
            ),
        };

    if !runtime_available && startup_warnings.is_empty() {
        startup_warnings.push(ConversationCliSupportNoticeView {
            notice_kind: "runtime_unavailable".to_string(),
            severity: "warning".to_string(),
            summary: "Runtime is unavailable. Start and continue actions will stay blocked until it recovers.".to_string(),
            support_surface: Some("run".to_string()),
            blocks_live_turn: true,
            allowed_next_action:
                "start the runtime before sending or continuing live work".to_string(),
        });
    }

    if discovery_success && recent_sessions.is_empty() {
        startup_warnings.push(ConversationCliSupportNoticeView {
            notice_kind: "no_recent_sessions".to_string(),
            severity: "info".to_string(),
            summary: "No retained sessions were found yet. Start a new session to begin."
                .to_string(),
            support_surface: None,
            blocks_live_turn: false,
            allowed_next_action: "start a new session from this shell".to_string(),
        });
    }

    ConversationCliStartupHomeView {
        resume_last_session_id: recent_sessions.first().map(|row| row.session_id.clone()),
        recent_sessions,
        startup_warnings,
        provider_kind: config.llm.provider_kind.as_str().to_string(),
        model_id: config.llm.model_id.clone(),
        config_action,
        session_source,
        runtime_available,
    }
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
                "  - turn {} [{}] workflow={} lifecycle={}\n    you: {}\n    assistant: {}\n    resume: {}",
                turn.turn_index,
                turn.status,
                turn.workflow_id,
                turn.lifecycle_state.as_str(),
                turn.user_message,
                assistant_result,
                resume_provenance
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let current_turn_state = render_current_turn_state(view.current_turn_state.as_ref());
    let support_notices = render_support_notices(&view.support_notices);
    let control_state =
        render_control_state(&view.control_state, &view.provider_kind, &view.model_id);

    format!(
        "session: {}\nsession_id: {}\nstatus: {}\nloop_state: {}\nnext_action: {}\ncoordinator_agent_id: {}\nprovider_kind: {}\nmodel_id: {}\nactive_workflow_id: {}\nlast_completed_workflow_id: {}\nturn_count: {}\ncurrent_turn:\n{}\ncontrols:\n{}\nsupport_notices:\n{}\nended_at: {}\nconversation:\n{}",
        view.title,
        view.session_id,
        view.status,
        view.loop_state,
        view.next_action_hint,
        view.coordinator_agent_id,
        view.provider_kind,
        view.model_id,
        view.active_workflow_id.as_deref().unwrap_or("none"),
        view.last_completed_workflow_id.as_deref().unwrap_or("none"),
        view.turn_count,
        current_turn_state,
        control_state,
        support_notices,
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

pub(crate) fn render_session_list(sessions: &[ConversationCliSessionSummaryView]) -> String {
    if sessions.is_empty() {
        return "recent_sessions:\nnone".to_string();
    }

    let rows = sessions
        .iter()
        .enumerate()
        .map(|(index, session)| {
            format!(
                "  {}. {} [{}] status={} model={} updated={} preview={}",
                index + 1,
                session.title,
                session.session_id,
                session.status,
                session.model_id,
                session.updated_at,
                session.last_preview.as_deref().unwrap_or("none")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("recent_sessions:\n{rows}")
}

pub(crate) fn render_startup_home(view: &ConversationCliStartupHomeView) -> String {
    format!(
        "Mister Smith CLI shell\nprovider: {}\nmodel: {}\nruntime_available: {}\nsession_source: {}\nresume_last: {}\nconfig: {}\nstartup_warnings:\n{}\n{}\nactions:\n  new <message>\n  resume last\n  open <session_id>\n  sessions\n  config\n  help\n  quit",
        view.provider_kind,
        view.model_id,
        if view.runtime_available { "yes" } else { "no" },
        view.session_source,
        view.resume_last_session_id.as_deref().unwrap_or("none"),
        view.config_action,
        render_support_notices(&view.startup_warnings),
        render_session_list(&view.recent_sessions),
    )
}

pub(crate) fn render_support_notices(notices: &[ConversationCliSupportNoticeView]) -> String {
    if notices.is_empty() {
        return "none".to_string();
    }

    notices
        .iter()
        .map(|notice| {
            format!(
                "  - [{}] {}{}{}{}",
                notice.severity,
                notice.summary,
                notice
                    .support_surface
                    .as_ref()
                    .map(|surface| format!(" (support: {surface})"))
                    .unwrap_or_default(),
                if notice.blocks_live_turn {
                    " (blocks live turn)"
                } else {
                    ""
                },
                if notice.allowed_next_action.is_empty() {
                    String::new()
                } else {
                    format!(" next: {}", notice.allowed_next_action)
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_current_turn_state(view: Option<&ConversationCliCurrentTurnStateView>) -> String {
    let Some(view) = view else {
        return "  none".to_string();
    };

    let proof_boundary = view.proof_boundary_note.as_deref().unwrap_or("none");
    let preview = view.result_preview.as_deref().unwrap_or("none");

    format!(
        "  turn_index: {}\n  workflow_id: {}\n  turn_status: {}\n  lifecycle_state: {}\n  state_source: {}\n  result_preview: {}\n  proof_boundary: {}\n  next_action: {}",
        view.turn_index,
        view.workflow_id,
        view.turn_status,
        view.lifecycle_state.as_str(),
        view.state_source,
        preview,
        proof_boundary,
        view.next_action_hint
    )
}

fn render_control_state(
    control_state: &ConversationCliSessionControlState,
    provider_kind: &str,
    model_id: &str,
) -> String {
    format!(
        "  runtime_provider: {}\n  runtime_model: {}\n  selected_provider: {}\n  selected_model: {}\n  permission_mode: {}\n  config_posture: {}\n  status_view: {}\n  mcp_posture: {}",
        provider_kind,
        model_id,
        control_state
            .selected_provider_kind
            .as_deref()
            .unwrap_or("inherit"),
        control_state
            .selected_model_id
            .as_deref()
            .unwrap_or("inherit"),
        control_state.permission_mode,
        control_state.config_posture,
        control_state.status_view,
        control_state.mcp_posture
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
    fn fallback_predicate_rejects_http_status_errors() {
        let error = ConversationClientError::HttpStatus(
            StatusCode::UNAUTHORIZED,
            "permission denied".to_string(),
        );

        assert!(!should_fallback_to_direct_session_store(&error));
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
            title: "resume interrupted workflow".to_string(),
            session_id: "11111111-1111-1111-1111-111111111111".to_string(),
            status: "active".to_string(),
            loop_state: "ready".to_string(),
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
            current_turn_state: Some(ConversationCliCurrentTurnStateView {
                workflow_id: "55555555-5555-5555-5555-555555555555".to_string(),
                turn_index: 2,
                turn_status: "completed".to_string(),
                lifecycle_state: DurableWorkflowLifecycleState::Completed,
                result_preview: Some("bounded answer preview".to_string()),
                proof_boundary_note: Some(
                    "result is orchestration proof, not substantive task proof".to_string(),
                ),
                state_source: "retained_session".to_string(),
                next_action_hint: "send a follow-up turn or adjust the session controls"
                    .to_string(),
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
            control_state: ConversationCliSessionControlState {
                selected_provider_kind: Some("openai_chatgpt".to_string()),
                selected_model_id: Some("gpt-5.4".to_string()),
                permission_mode: "review".to_string(),
                config_posture: "inline".to_string(),
                status_view: "summary".to_string(),
                mcp_posture: "connected".to_string(),
            },
            support_notices: vec![ConversationCliSupportNoticeView {
                notice_kind: "session_shell_preferences".to_string(),
                severity: "warning".to_string(),
                summary: "Shell control changes are stored with this session, but runtime execution still follows the active runtime path.".to_string(),
                support_surface: Some("config".to_string()),
                blocks_live_turn: false,
                allowed_next_action:
                    "keep working in this session or adjust the support posture".to_string(),
            }],
            next_action_hint: "send a follow-up turn or adjust the session controls".to_string(),
            ended_at: None,
        };

        let rendered = render_session(&view);

        assert!(rendered.contains("loop_state: ready"));
        assert!(rendered.contains("turn_status: completed"));
        assert!(rendered
            .contains("proof_boundary: result is orchestration proof, not substantive task proof"));
        assert!(rendered.contains("resume: recovered_after_restart=true"));
        assert!(
            rendered.contains("reason=workflow interrupted by runtime restart before session sync")
        );
        assert!(rendered.contains("resume: resumed_after_restart=true resumed_from_turn=1"));
        assert!(rendered.contains("resumed_from_workflow=44444444-4444-4444-4444-444444444444"));
        assert!(rendered.contains(
            "assistant: workflow=55555555-5555-5555-5555-555555555555 status=completed proof=collapsed_to_sequential preview=bounded answer preview runtime_truth=placeholder_or_simulated_step_completion:result is orchestration proof, not substantive task proof"
        ));
        assert!(rendered.contains("sources=metadata.final_result|metadata.aggregated_result"));
    }
}
