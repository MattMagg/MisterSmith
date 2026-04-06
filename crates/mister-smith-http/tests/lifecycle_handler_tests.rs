use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use mister_smith_core::{
    AgentId, DurableWorkflowLifecycleState, DurableWorkflowLifecycleVerb, LifecycleDecisionOutcome,
    SessionId, SessionStatus, TaskId,
};
use mister_smith_http::handlers::{
    apply_task_lifecycle, get_session, update_session_controls, SessionControlResponse,
    SessionInspectResponse, SessionTurnSummaryResponse, TaskLifecycleRequest,
    UpdateSessionControlRequest,
};
use mister_smith_http::server::{
    AppState, ConversationContinueRequest, ConversationCreateRequest, ConversationEndView,
    ConversationResumeProvenanceView, ConversationServiceError,
    ConversationSessionControlUpdateRequest, ConversationSessionControlView,
    ConversationSessionService, ConversationSessionSummaryView, ConversationSessionView,
    ConversationSupportNoticeView, ConversationTurnAccepted, ConversationTurnSummaryView,
    SessionListRequest, TaskExecutionService, TaskLifecycleView, TaskListRequest, TaskStatusView,
    TaskSubmissionRequest, TaskSubmissionResponse, TaskSummaryView,
};

#[derive(Clone)]
struct LifecycleOnlyTaskService {
    decision: TaskLifecycleView,
}

#[async_trait::async_trait]
impl TaskExecutionService for LifecycleOnlyTaskService {
    async fn submit_task(
        &self,
        _request: TaskSubmissionRequest,
    ) -> Result<TaskSubmissionResponse, String> {
        Err("submit_task not used in lifecycle handler test".to_string())
    }

    async fn get_task(&self, _task_id: TaskId) -> Result<Option<TaskStatusView>, String> {
        Ok(None)
    }

    async fn apply_task_lifecycle(
        &self,
        _task_id: TaskId,
        _verb: DurableWorkflowLifecycleVerb,
        _reason: Option<String>,
    ) -> Result<Option<TaskLifecycleView>, String> {
        Ok(Some(self.decision.clone()))
    }

    async fn list_tasks(&self, _request: TaskListRequest) -> Result<Vec<TaskSummaryView>, String> {
        Ok(vec![])
    }
}

#[derive(Clone)]
struct FixedConversationService {
    view: ConversationSessionView,
}

#[async_trait::async_trait]
impl ConversationSessionService for FixedConversationService {
    async fn create_session(
        &self,
        _request: ConversationCreateRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
        Err(ConversationServiceError::Internal(
            "create_session not used in lifecycle handler test".to_string(),
        ))
    }

    async fn continue_session(
        &self,
        _request: ConversationContinueRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError> {
        Err(ConversationServiceError::Internal(
            "continue_session not used in lifecycle handler test".to_string(),
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
            "end_session not used in lifecycle handler test".to_string(),
        ))
    }

    async fn update_session_control_state(
        &self,
        session_id: SessionId,
        request: ConversationSessionControlUpdateRequest,
    ) -> Result<ConversationSessionControlView, ConversationServiceError> {
        fn normalize(value: Option<String>) -> Option<String> {
            value
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        }

        Ok(ConversationSessionControlView {
            session_id,
            selected_provider_kind: normalize(request.selected_provider_kind)
                .or_else(|| self.view.control_state.selected_provider_kind.clone()),
            selected_model_id: normalize(request.selected_model_id)
                .or_else(|| self.view.control_state.selected_model_id.clone()),
            permission_mode: normalize(request.permission_mode)
                .unwrap_or_else(|| self.view.control_state.permission_mode.clone()),
            config_posture: normalize(request.config_posture)
                .unwrap_or_else(|| self.view.control_state.config_posture.clone()),
            status_view: normalize(request.status_view)
                .unwrap_or_else(|| self.view.control_state.status_view.clone()),
            mcp_posture: normalize(request.mcp_posture)
                .unwrap_or_else(|| self.view.control_state.mcp_posture.clone()),
        })
    }

    async fn list_sessions(
        &self,
        _request: SessionListRequest,
    ) -> Result<Vec<ConversationSessionSummaryView>, ConversationServiceError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn task_lifecycle_handler_returns_durable_projection() {
    let task_id = TaskId::new();
    let state = AppState::new().with_task_service(Arc::new(LifecycleOnlyTaskService {
        decision: TaskLifecycleView {
            task_id,
            status: "failed".to_string(),
            lifecycle_state: DurableWorkflowLifecycleState::Terminated,
            outcome: LifecycleDecisionOutcome::Applied,
            note: Some("termination is persisted separately from raw task status".to_string()),
        },
    }));

    let Json(response) = apply_task_lifecycle(
        State(state),
        Path(task_id.to_string()),
        Json(TaskLifecycleRequest {
            verb: DurableWorkflowLifecycleVerb::Terminate,
            reason: Some("operator ended the run".to_string()),
        }),
    )
    .await
    .expect("lifecycle handler should succeed");

    assert_eq!(response.task_id, task_id);
    assert_eq!(response.status, "failed");
    assert_eq!(
        response.lifecycle_state,
        DurableWorkflowLifecycleState::Terminated
    );
    assert_eq!(response.outcome, LifecycleDecisionOutcome::Applied);
    assert!(response
        .note
        .as_deref()
        .unwrap_or_default()
        .contains("persisted separately"));
}

#[tokio::test]
async fn session_handler_includes_turn_lifecycle_projection() {
    let session_id = SessionId::new();
    let workflow_id = TaskId::new();
    let state = AppState::new().with_conversation_service(Arc::new(FixedConversationService {
        view: ConversationSessionView {
            title: "stop now".to_string(),
            session_id,
            status: SessionStatus::Active,
            coordinator_agent_id: AgentId::new(),
            provider_kind: "openai_chatgpt".to_string(),
            model_id: "gpt-5.4".to_string(),
            active_workflow_id: Some(workflow_id),
            last_completed_workflow_id: None,
            turn_count: 1,
            last_assistant_result: None,
            turns: vec![ConversationTurnSummaryView {
                turn_index: 1,
                workflow_id,
                status: "failed".to_string(),
                lifecycle_state: DurableWorkflowLifecycleState::Terminated,
                user_message: "stop now".to_string(),
                assistant_result: None,
                resume_provenance: Some(ConversationResumeProvenanceView {
                    recovered_after_restart: false,
                    resumed_after_restart: false,
                    recovered_at: None,
                    recovery_reason: None,
                    resumed_from_workflow_id: None,
                    resumed_from_turn_index: None,
                }),
            }],
            control_state: ConversationSessionControlView {
                session_id,
                selected_provider_kind: None,
                selected_model_id: None,
                permission_mode: "default".to_string(),
                config_posture: "inline".to_string(),
                status_view: "summary".to_string(),
                mcp_posture: "support_only".to_string(),
            },
            support_notices: vec![ConversationSupportNoticeView {
                notice_kind: "session_busy".to_string(),
                severity: "info".to_string(),
                summary: "This session already has a live workflow. New turns will wait until it finishes.".to_string(),
                support_surface: Some("status".to_string()),
            }],
            ended_at: None,
        },
    }));

    let Json(response): Json<SessionInspectResponse> =
        get_session(State(state), Path(session_id.to_string()))
            .await
            .expect("session inspect should succeed");

    assert_eq!(response.session_id, session_id);
    assert_eq!(response.turns.len(), 1);
    let turn: &SessionTurnSummaryResponse = &response.turns[0];
    assert_eq!(turn.workflow_id, workflow_id);
    assert_eq!(turn.status, "failed");
    assert_eq!(
        turn.lifecycle_state,
        DurableWorkflowLifecycleState::Terminated
    );
}

#[tokio::test]
async fn session_control_handler_returns_updated_control_projection() {
    let session_id = SessionId::new();
    let state = AppState::new().with_conversation_service(Arc::new(FixedConversationService {
        view: ConversationSessionView {
            title: "resume packet review".to_string(),
            session_id,
            status: SessionStatus::Active,
            coordinator_agent_id: AgentId::new(),
            provider_kind: "openai_chatgpt".to_string(),
            model_id: "gpt-5.4".to_string(),
            active_workflow_id: None,
            last_completed_workflow_id: None,
            turn_count: 1,
            last_assistant_result: None,
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
            ended_at: None,
        },
    }));

    let Json(response): Json<SessionControlResponse> = update_session_controls(
        State(state),
        Path(session_id.to_string()),
        Json(UpdateSessionControlRequest {
            selected_provider_kind: Some("openai_chatgpt".to_string()),
            selected_model_id: Some("gpt-5.4-mini".to_string()),
            permission_mode: Some("review".to_string()),
            config_posture: Some("support".to_string()),
            status_view: Some("detail".to_string()),
            mcp_posture: Some("connected".to_string()),
        }),
    )
    .await
    .expect("session control handler should succeed");

    assert_eq!(
        response.selected_provider_kind.as_deref(),
        Some("openai_chatgpt")
    );
    assert_eq!(response.selected_model_id.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(response.permission_mode, "review");
    assert_eq!(response.config_posture, "support");
    assert_eq!(response.status_view, "detail");
    assert_eq!(response.mcp_posture, "connected");
}