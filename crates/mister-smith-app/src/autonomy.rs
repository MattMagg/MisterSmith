//! Operator-facing autonomy status helpers.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use mister_smith_config::FrameworkConfig;
use mister_smith_core::{
    AgentId, OperatorResultPreview, ProofOutcomeClassification, ResultProvenanceSummary, SessionId,
    SessionRetainedResultView, TaskId, TaskResultView, UnifiedResultEnvelope,
};
use mister_smith_events::{
    AutonomyStatusView, EventBus, ExternalCapabilityDecisionOutcome,
    ExternalCapabilityDecisionSummary, ResumeProvenanceSummary, StepRoutingDecisionSummary,
};
use mister_smith_persistence::postgres::queries;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::PgPool;
use uuid::Uuid;

const RESULT_PREVIEW_MAX_CHARS: usize = 160;

/// Serializable list of workflow IDs with autonomy status projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutonomyWorkflowList {
    /// Workflow IDs that currently have an assembled autonomy view.
    pub workflows: Vec<String>,
}

/// Error returned when the autonomy status cannot be inspected.
#[derive(Debug)]
pub enum AutonomyStatusError {
    /// The provided workflow ID was not a valid UUID.
    InvalidWorkflowId(String),
    /// No autonomy status exists for the requested workflow.
    NotFound(TaskId),
    /// The HTTP request to the local runtime failed.
    Http(reqwest::Error),
    /// The local runtime returned an unexpected HTTP status.
    HttpStatus(StatusCode, String),
}

impl fmt::Display for AutonomyStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AutonomyStatusError::InvalidWorkflowId(raw) => {
                write!(f, "invalid workflow id '{raw}'")
            }
            AutonomyStatusError::NotFound(workflow_id) => {
                write!(f, "no autonomy status found for workflow {workflow_id}")
            }
            AutonomyStatusError::Http(error) => write!(f, "{error}"),
            AutonomyStatusError::HttpStatus(status, body) => {
                write!(f, "runtime returned {}: {}", status.as_u16(), body)
            }
        }
    }
}

impl Error for AutonomyStatusError {}

impl IntoResponse for AutonomyStatusError {
    fn into_response(self) -> Response {
        let status = match self {
            AutonomyStatusError::InvalidWorkflowId(_) => StatusCode::BAD_REQUEST,
            AutonomyStatusError::NotFound(_) => StatusCode::NOT_FOUND,
            AutonomyStatusError::Http(_) | AutonomyStatusError::HttpStatus(_, _) => {
                StatusCode::BAD_GATEWAY
            }
        };

        let body = Json(AutonomyErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AutonomyErrorBody {
    error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResumeProvenanceDetails {
    pub recovered_after_restart: bool,
    pub resumed_after_restart: bool,
    pub recovered_at: Option<DateTime<Utc>>,
    pub recovery_reason: Option<String>,
    pub resumed_from_workflow_id: Option<TaskId>,
    pub resumed_from_turn_index: Option<u32>,
}

/// Derive the local autonomy inspection base URL from framework config.
pub fn default_base_url(config: &FrameworkConfig) -> String {
    let port = config.transport.http_port.unwrap_or(8080);
    format!("http://127.0.0.1:{port}")
}

/// Parse a workflow ID from CLI or HTTP input.
pub fn parse_workflow_id(raw: &str) -> Result<TaskId, AutonomyStatusError> {
    Uuid::parse_str(raw)
        .map(TaskId::from_uuid)
        .map_err(|_| AutonomyStatusError::InvalidWorkflowId(raw.to_string()))
}

/// Fetch autonomy status from a running local runtime over HTTP.
pub async fn fetch_status(
    base_url: &str,
    workflow_id: TaskId,
) -> Result<AutonomyStatusView, AutonomyStatusError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/autonomy/status/{}",
        base_url.trim_end_matches('/'),
        workflow_id
    );
    let response = client
        .get(url)
        .send()
        .await
        .map_err(AutonomyStatusError::Http)?;

    if response.status().is_success() {
        return response
            .json::<AutonomyStatusView>()
            .await
            .map_err(AutonomyStatusError::Http);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(AutonomyStatusError::HttpStatus(status, body))
}

/// Fetch the list of workflow IDs with an autonomy projection.
pub async fn fetch_workflows(base_url: &str) -> Result<AutonomyWorkflowList, AutonomyStatusError> {
    let client = Client::new();
    let url = format!(
        "{}/api/v1/autonomy/workflows",
        base_url.trim_end_matches('/')
    );
    let response = client
        .get(url)
        .send()
        .await
        .map_err(AutonomyStatusError::Http)?;

    if response.status().is_success() {
        return response
            .json::<AutonomyWorkflowList>()
            .await
            .map_err(AutonomyStatusError::Http);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(AutonomyStatusError::HttpStatus(status, body))
}

/// Resolve autonomy status directly from the in-process event bus.
pub async fn status_from_bus(
    event_bus: Arc<EventBus>,
    workflow_id: &str,
) -> Result<AutonomyStatusView, AutonomyStatusError> {
    let workflow_id = parse_workflow_id(workflow_id)?;
    event_bus
        .autonomy_status(&workflow_id)
        .await
        .ok_or(AutonomyStatusError::NotFound(workflow_id))
}

/// Resolve autonomy status directly from the in-process event bus and enrich
/// it with persisted session linkage when available.
pub async fn status_from_bus_with_session_linkage(
    event_bus: Arc<EventBus>,
    pool: PgPool,
    workflow_id: &str,
) -> Result<AutonomyStatusView, AutonomyStatusError> {
    let mut view = status_from_bus(event_bus, workflow_id).await?;
    if let Some(record) = queries::find_task(&pool, *view.graph.workflow_id.as_ref())
        .await
        .map_err(|error| {
            AutonomyStatusError::HttpStatus(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load workflow metadata: {error}"),
            )
        })?
    {
        enrich_session_linkage(&mut view, &record.metadata);
        enrich_step_routing_history(&mut view, &record.metadata);
        enrich_result_preview(&mut view, &record.metadata, record.result.as_ref());
    }
    Ok(view)
}

/// Resolve workflow IDs that currently have an autonomy status projection.
pub async fn workflows_from_bus(event_bus: Arc<EventBus>) -> AutonomyWorkflowList {
    let workflows = event_bus
        .autonomy_workflows()
        .await
        .into_iter()
        .map(|workflow_id| workflow_id.to_string())
        .collect();
    AutonomyWorkflowList { workflows }
}

/// Render the typed autonomy view for human operators.
pub fn render_status(view: &AutonomyStatusView) -> String {
    let session_summary = match (view.session_id, view.turn_index, view.coordinator_agent_id) {
        (Some(session_id), Some(turn_index), Some(coordinator_agent_id)) => format!(
            "{} turn={} coordinator={}",
            session_id, turn_index, coordinator_agent_id
        ),
        _ => "none".to_string(),
    };
    let resume_summary = view
        .resume_provenance
        .as_ref()
        .map(render_resume_provenance)
        .unwrap_or_else(|| "none".to_string());
    let topology_reason = &view.topology.rationale.selected_for;
    let task_shape = view.topology.task_shape.kind.as_str();
    let structural_signals = if view.topology.task_shape.structural_signals.is_empty() {
        "none".to_string()
    } else {
        view.topology.task_shape.structural_signals.join(" | ")
    };
    let dependency_shape = &view.topology.rationale.dependency_shape;
    let topology_signals = if view.topology.rationale.operational_signals.is_empty() {
        "none".to_string()
    } else {
        view.topology.rationale.operational_signals.join(" | ")
    };
    let fallback_reason = view
        .topology
        .rationale
        .fallback_reason
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let team_sizing_summary = view
        .team_sizing
        .as_ref()
        .map(|decision| {
            let budget_pressure = decision
                .budget_pressure
                .map(|pressure| pressure.to_string())
                .unwrap_or_else(|| "none".to_string());
            let cap_reason = decision
                .cap_reason
                .clone()
                .unwrap_or_else(|| "none".to_string());
            let rationale = if decision.rationale_lines.is_empty() {
                "none".to_string()
            } else {
                decision.rationale_lines.join(" | ")
            };
            format!(
                "phase={} desired={} selected={} frontier={} depth={} conservative={} budget={} cap={} rationale={}",
                decision.decision_phase,
                decision.desired_workers,
                decision.selected_workers,
                decision.branch_frontier_width,
                decision.dependency_depth,
                decision.conservative_mode,
                budget_pressure,
                cap_reason,
                rationale
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let branch_summary = view
        .branches
        .iter()
        .map(|branch| {
            format!(
                "{} {:?} checkpoint={} recovery={:?}",
                branch.branch_id,
                branch.state,
                branch
                    .checkpoint_id
                    .map(|checkpoint_id| checkpoint_id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                branch.recovery_strategy
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let checkpoint_summary = view
        .checkpoint_lineage
        .iter()
        .map(|checkpoint| {
            format!(
                "{} {} completed={} pending={}",
                checkpoint.branch_id,
                checkpoint.checkpoint_id,
                checkpoint.completed_nodes.len(),
                checkpoint.pending_nodes.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let routing_summary = view
        .routing_history
        .iter()
        .map(|routing| {
            format!(
                "{} {:?} budget={} depth={} rationale={}",
                routing.branch_id,
                routing.health_state,
                routing.budget_pressure,
                routing.dependency_depth,
                routing.rationale.join(" | ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let step_routing_summary = view
        .step_routing_history
        .iter()
        .map(|routing| {
            let previous_action = routing
                .previous_action
                .clone()
                .unwrap_or_else(|| "none".to_string());
            let confidence = routing
                .confidence_score
                .map(|score| format!("{score:.2}"))
                .unwrap_or_else(|| "none".to_string());
            let preferred_tier = routing
                .preferred_tier_after
                .clone()
                .unwrap_or_else(|| "none".to_string());
            let estimated_cost = routing
                .estimated_cost_tokens
                .map(|tokens| tokens.to_string())
                .unwrap_or_else(|| "none".to_string());
            let checkpoints = if routing.triggered_checkpoints.is_empty() {
                "none".to_string()
            } else {
                routing.triggered_checkpoints.join(" | ")
            };
            let delta = if routing.change_rationale.is_empty() {
                "none".to_string()
            } else {
                routing.change_rationale.join(" | ")
            };
            format!(
                "{}#{} kind={} action={} previous={} changed={} tier={} preferred={} cost={} confidence={} checkpoints={} reason={} delta={}",
                routing.step_id,
                routing
                    .step_index
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                routing.step_kind.as_deref().unwrap_or("none"),
                routing.action,
                previous_action,
                routing.action_changed,
                routing.tier,
                preferred_tier,
                estimated_cost,
                confidence,
                checkpoints,
                routing.reason,
                delta
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let result_preview_summary = view
        .result_preview
        .as_ref()
        .map(render_result_preview)
        .unwrap_or_else(|| "none".to_string());
    let intervention_summary = view
        .interventions
        .iter()
        .map(|record| record.rationale.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let delegation_summary = view
        .delegation_capabilities
        .iter()
        .map(|capability| {
            let lineage = capability
                .provenance
                .links
                .iter()
                .map(|link| {
                    format!(
                        "{:?}->{}/{}",
                        link.issuer, link.recipient, link.capability_id
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            format!(
                "{} {:?} state={:?} depth={} expires={} lineage={}",
                capability.capability_id,
                capability.scope,
                capability.revocation_state,
                capability.chain_depth(),
                capability.expires_at,
                lineage
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let delegation_alerts = view
        .delegation_alerts
        .iter()
        .map(|alert| {
            let reason = alert
                .rejection_reason
                .clone()
                .unwrap_or_else(|| "none".to_string());
            format!(
                "{} depth={} reason={}",
                alert.message, alert.chain_depth, reason
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let external_capability_decisions = view
        .external_capability_decisions
        .iter()
        .map(render_external_capability_decision)
        .collect::<Vec<_>>()
        .join("\n");
    let conservative_summary = if view.conservative_reasons.is_empty() {
        "none".to_string()
    } else {
        view.conservative_reasons.join(" | ")
    };

    format!(
        "workflow: {}\ngraph: {} {:?}\nsession: {}\nresume provenance: {}\ntopology: {:?} width={} shape={} structure={} dependency={} rationale={} signals={}\nfallback: {}\nteam sizing: {}\nbranches:\n{}\ncheckpoints:\n{}\nrouting:\n{}\nstep routing:\n{}\nresult preview: {}\ninterventions:\n{}\ndelegation:\n{}\ndelegation alerts:\n{}\nexternal capability decisions:\n{}\nconservative: {}",
        view.graph.workflow_id,
        view.graph.graph_id,
        view.graph.state,
        session_summary,
        resume_summary,
        view.topology.topology_kind,
        view.topology.parallelism_width,
        task_shape,
        structural_signals,
        dependency_shape,
        topology_reason,
        topology_signals,
        fallback_reason,
        team_sizing_summary,
        if branch_summary.is_empty() { "none".to_string() } else { branch_summary },
        if checkpoint_summary.is_empty() { "none".to_string() } else { checkpoint_summary },
        if routing_summary.is_empty() { "none".to_string() } else { routing_summary },
        if step_routing_summary.is_empty() { "none".to_string() } else { step_routing_summary },
        result_preview_summary,
        if intervention_summary.is_empty() {
            "none".to_string()
        } else {
            intervention_summary
        },
        if delegation_summary.is_empty() {
            "none".to_string()
        } else {
            delegation_summary
        },
        if delegation_alerts.is_empty() {
            "none".to_string()
        } else {
            delegation_alerts
        },
        if external_capability_decisions.is_empty() {
            "none".to_string()
        } else {
            external_capability_decisions
        },
        conservative_summary
    )
}

pub(crate) fn enrich_session_linkage(view: &mut AutonomyStatusView, metadata: &serde_json::Value) {
    if view.session_id.is_none() {
        view.session_id = metadata
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .map(SessionId::from_uuid);
    }
    if view.turn_index.is_none() {
        view.turn_index = metadata
            .get("turn_index")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok());
    }
    if view.coordinator_agent_id.is_none() {
        view.coordinator_agent_id = metadata
            .get("coordinator_agent_id")
            .and_then(serde_json::Value::as_str)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .map(AgentId::from_uuid);
    }
    if view.resume_provenance.is_none() {
        view.resume_provenance =
            resume_provenance_from_metadata(metadata).map(|details| ResumeProvenanceSummary {
                recovered_after_restart: details.recovered_after_restart,
                resumed_after_restart: details.resumed_after_restart,
                recovered_at: details.recovered_at,
                recovery_reason: details.recovery_reason,
                resumed_from_workflow_id: details.resumed_from_workflow_id,
                resumed_from_turn_index: details.resumed_from_turn_index,
            });
    }
}

pub(crate) fn enrich_step_routing_history(view: &mut AutonomyStatusView, metadata: &Value) {
    if !view.step_routing_history.is_empty() {
        return;
    }
    let Some(raw) = metadata.get("step_routing_history").cloned() else {
        return;
    };
    if let Ok(history) = serde_json::from_value::<Vec<StepRoutingDecisionSummary>>(raw) {
        view.step_routing_history = history;
    }
}

pub(crate) struct CanonicalResultEnvelopeInput<'a> {
    pub(crate) workflow_id: TaskId,
    pub(crate) provider_kind: &'a str,
    pub(crate) model_id: &'a str,
    pub(crate) description: &'a str,
    pub(crate) runtime_execution_mode: Value,
    pub(crate) planner_output: Value,
    pub(crate) execution_plan: Value,
    pub(crate) step_results: Vec<Value>,
    pub(crate) aggregated_result: Value,
    pub(crate) status: &'a str,
}

pub(crate) fn build_canonical_result_envelope(
    input: CanonicalResultEnvelopeInput<'_>,
) -> UnifiedResultEnvelope {
    let CanonicalResultEnvelopeInput {
        workflow_id,
        provider_kind,
        model_id,
        description,
        runtime_execution_mode,
        planner_output,
        execution_plan,
        step_results,
        aggregated_result,
        status,
    } = input;
    let proof_outcome =
        classify_proof_outcome(status, Some(&execution_plan), Some(step_results.as_slice()));
    UnifiedResultEnvelope {
        workflow_id,
        provider_kind: provider_kind.to_string(),
        model_id: model_id.to_string(),
        description: description.to_string(),
        runtime_execution_mode,
        planner_output,
        execution_plan,
        step_results,
        aggregated_result,
        proof_outcome,
    }
}

pub(crate) fn build_task_result_view(
    status: &str,
    canonical_result: UnifiedResultEnvelope,
) -> TaskResultView {
    TaskResultView {
        workflow_id: canonical_result.workflow_id,
        status: status.to_string(),
        proof_outcome: canonical_result.proof_outcome,
        result: canonical_result,
    }
}

pub(crate) fn classify_proof_outcome(
    status: &str,
    execution_plan: Option<&Value>,
    step_results: Option<&[Value]>,
) -> ProofOutcomeClassification {
    if !status.eq_ignore_ascii_case("completed") {
        return ProofOutcomeClassification::FailedBeforeGraph;
    }

    let collapsed_from_plan = execution_plan
        .and_then(|plan| plan.get("steps"))
        .and_then(Value::as_array)
        .map(|steps| steps.len() <= 1);
    let collapsed_from_steps = step_results
        .filter(|results| !results.is_empty())
        .map(|results| results.len() <= 1);

    if collapsed_from_plan
        .or(collapsed_from_steps)
        .unwrap_or(false)
    {
        ProofOutcomeClassification::CollapsedToSequential
    } else {
        ProofOutcomeClassification::GraphFormedAndCompleted
    }
}

pub(crate) fn enrich_result_preview(
    view: &mut AutonomyStatusView,
    metadata: &Value,
    task_result: Option<&Value>,
) {
    let status_hint = status_hint_from_graph_state(&view.graph.state);
    let payload_preview = task_result
        .and_then(|value| {
            canonical_result_from_value(value, Some(status_hint))
                .map(|result| operator_result_preview(&result, "task.result"))
        })
        .or_else(|| {
            metadata.get("final_result").and_then(|value| {
                canonical_result_from_value(value, Some(status_hint))
                    .map(|result| operator_result_preview(&result, "metadata.final_result"))
            })
        });

    if let Some(preview) = payload_preview {
        view.result_preview = Some(preview);
    }
}

pub(crate) fn retained_assistant_result(
    task_result: &Value,
    turn_index: u32,
    status: &str,
) -> Option<Value> {
    build_session_retained_result(task_result, turn_index, status)
        .map(|projection| projection.assistant_result)
}

fn build_session_retained_result(
    task_result: &Value,
    turn_index: u32,
    status: &str,
) -> Option<SessionRetainedResultView> {
    let canonical_result = canonical_result_from_value(task_result, Some(status))?;
    let preview = preview_text(&canonical_result.aggregated_result);
    let assistant_result = assistant_result_payload(&canonical_result, preview.clone());

    Some(SessionRetainedResultView {
        workflow_id: canonical_result.workflow_id,
        turn_index,
        status: status.to_string(),
        assistant_result,
        preview,
        provenance: result_provenance(&canonical_result, None, None),
    })
}

fn operator_result_preview(
    canonical_result: &UnifiedResultEnvelope,
    payload_location: &str,
) -> OperatorResultPreview {
    OperatorResultPreview {
        workflow_id: canonical_result.workflow_id,
        proof_outcome: canonical_result.proof_outcome,
        preview_text: preview_text(&canonical_result.aggregated_result),
        payload_location: payload_location.to_string(),
        provenance_lines: result_preview_provenance(canonical_result.proof_outcome),
    }
}

fn assistant_result_payload(
    canonical_result: &UnifiedResultEnvelope,
    preview: Option<String>,
) -> Value {
    let mut payload = Map::new();
    if let Some(preview_text) = preview {
        payload.insert("preview".to_string(), Value::String(preview_text));
    }
    payload.insert(
        "aggregated_result".to_string(),
        canonical_result.aggregated_result.clone(),
    );
    payload.insert(
        "proof_outcome".to_string(),
        Value::String(canonical_result.proof_outcome.as_str().to_string()),
    );

    if let Some(recovered_after_restart) = canonical_result
        .aggregated_result
        .get("recovered_after_restart")
        .and_then(Value::as_bool)
    {
        payload.insert(
            "recovered_after_restart".to_string(),
            Value::Bool(recovered_after_restart),
        );
    }

    Value::Object(payload)
}

fn result_provenance(
    canonical_result: &UnifiedResultEnvelope,
    graph_state: Option<String>,
    graph_id: Option<String>,
) -> ResultProvenanceSummary {
    ResultProvenanceSummary {
        runtime_execution_mode: canonical_result.runtime_execution_mode.clone(),
        graph_state,
        graph_id,
        source_fields: vec![
            "metadata.final_result".to_string(),
            "metadata.aggregated_result".to_string(),
        ],
    }
}

fn result_preview_provenance(proof_outcome: ProofOutcomeClassification) -> Vec<String> {
    let outcome_line = match proof_outcome {
        ProofOutcomeClassification::GraphFormedAndCompleted => {
            "graph formed and completed before final result publication"
        }
        ProofOutcomeClassification::CollapsedToSequential => "planner emitted one sequential step",
        ProofOutcomeClassification::FailedBeforeGraph => {
            "workflow failed before usable graph formation"
        }
    };

    vec![
        outcome_line.to_string(),
        "canonical result stored in metadata.final_result".to_string(),
        "aggregated payload nested under metadata.aggregated_result".to_string(),
        "session assistant_result derives from the canonical result object".to_string(),
    ]
}

fn canonical_result_from_value(
    value: &Value,
    status_hint: Option<&str>,
) -> Option<UnifiedResultEnvelope> {
    let object = value.as_object()?;

    if let Some(inner_result) = object.get("result") {
        let nested_status_hint = object.get("status").and_then(Value::as_str).or(status_hint);
        return canonical_result_from_value(inner_result, nested_status_hint);
    }

    if let Ok(canonical_result) = serde_json::from_value::<UnifiedResultEnvelope>(value.clone()) {
        return Some(canonical_result);
    }

    let workflow_id = object
        .get("workflow_id")
        .and_then(Value::as_str)
        .and_then(parse_task_id)?;
    let provider_kind = object
        .get("provider_kind")
        .and_then(Value::as_str)?
        .to_string();
    let model_id = object.get("model_id").and_then(Value::as_str)?.to_string();
    let description = object
        .get("description")
        .and_then(Value::as_str)?
        .to_string();
    let execution_plan = object.get("execution_plan").cloned().unwrap_or(Value::Null);
    let step_results = object
        .get("step_results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let proof_outcome = object
        .get("proof_outcome")
        .cloned()
        .and_then(|raw| serde_json::from_value::<ProofOutcomeClassification>(raw).ok())
        .unwrap_or_else(|| {
            classify_proof_outcome(
                status_hint.unwrap_or("completed"),
                Some(&execution_plan),
                Some(step_results.as_slice()),
            )
        });

    Some(UnifiedResultEnvelope {
        workflow_id,
        provider_kind,
        model_id,
        description,
        runtime_execution_mode: object
            .get("runtime_execution_mode")
            .cloned()
            .unwrap_or(Value::Null),
        planner_output: object.get("planner_output").cloned().unwrap_or(Value::Null),
        execution_plan,
        step_results,
        aggregated_result: object
            .get("aggregated_result")
            .cloned()
            .unwrap_or(Value::Null),
        proof_outcome,
    })
}

fn preview_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(raw) => Some(compact_preview(raw)),
        Value::Bool(raw) => Some(raw.to_string()),
        Value::Number(raw) => Some(raw.to_string()),
        Value::Array(entries) => {
            if entries.is_empty() {
                None
            } else {
                serde_json::to_string(entries)
                    .ok()
                    .map(|raw| compact_preview(&raw))
            }
        }
        Value::Object(object) => {
            for field in [
                "preview", "summary", "message", "error", "content", "result",
            ] {
                if let Some(raw) = object.get(field).and_then(Value::as_str) {
                    return Some(compact_preview(raw));
                }
            }

            serde_json::to_string(object)
                .ok()
                .map(|raw| compact_preview(&raw))
        }
    }
}

fn compact_preview(raw: &str) -> String {
    let compact = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= RESULT_PREVIEW_MAX_CHARS {
        compact
    } else {
        let mut truncated = compact
            .chars()
            .take(RESULT_PREVIEW_MAX_CHARS.saturating_sub(3))
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

fn status_hint_from_graph_state(state: &mister_smith_core::GraphState) -> &'static str {
    match state {
        mister_smith_core::GraphState::Completed => "completed",
        _ => "failed",
    }
}

pub(crate) fn resume_provenance_from_metadata(metadata: &Value) -> Option<ResumeProvenanceDetails> {
    let restart_recovery = metadata.get("restart_recovery").and_then(Value::as_object);
    let recovered_after_restart = restart_recovery.is_some();
    let recovered_at = restart_recovery
        .and_then(|payload| payload.get("recovered_at"))
        .and_then(parse_datetime);
    let recovery_reason = restart_recovery
        .and_then(|payload| payload.get("reason"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let retained_context = metadata.get("retained_context").and_then(Value::as_object);
    let transcript_entry = retained_context
        .and_then(|payload| payload.get("transcript_summary"))
        .and_then(Value::as_array)
        .and_then(|entries| entries.last());
    let resumed_from_workflow_id = retained_context
        .and_then(|payload| payload.get("latest_workflow_id"))
        .and_then(Value::as_str)
        .and_then(parse_task_id);
    let resumed_from_turn_index = transcript_entry
        .and_then(|entry| entry.get("turn_index"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| {
            resumed_from_workflow_id.as_ref().and_then(|_| {
                metadata
                    .get("turn_index")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .filter(|value| *value > 1)
                    .map(|value| value - 1)
            })
        });
    let resumed_after_restart = transcript_entry
        .and_then(|entry| entry.get("assistant_result"))
        .and_then(|result| result.get("recovered_after_restart"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !recovered_after_restart
        && !resumed_after_restart
        && recovered_at.is_none()
        && recovery_reason.is_none()
        && resumed_from_workflow_id.is_none()
        && resumed_from_turn_index.is_none()
    {
        return None;
    }

    Some(ResumeProvenanceDetails {
        recovered_after_restart,
        resumed_after_restart,
        recovered_at,
        recovery_reason,
        resumed_from_workflow_id,
        resumed_from_turn_index,
    })
}

fn render_resume_provenance(summary: &ResumeProvenanceSummary) -> String {
    let mut parts = Vec::new();

    if summary.recovered_after_restart {
        parts.push("recovered_after_restart=true".to_string());
    }
    if summary.resumed_after_restart {
        parts.push("resumed_after_restart=true".to_string());
    }
    if let Some(recovered_at) = summary.recovered_at {
        parts.push(format!("recovered_at={recovered_at}"));
    }
    if let Some(reason) = summary.recovery_reason.as_ref() {
        parts.push(format!("reason={reason}"));
    }
    if let Some(turn_index) = summary.resumed_from_turn_index {
        parts.push(format!("resumed_from_turn={turn_index}"));
    }
    if let Some(workflow_id) = summary.resumed_from_workflow_id {
        parts.push(format!("resumed_from_workflow={workflow_id}"));
    }

    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" ")
    }
}

fn render_result_preview(summary: &OperatorResultPreview) -> String {
    let preview_text = summary
        .preview_text
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let provenance = if summary.provenance_lines.is_empty() {
        "none".to_string()
    } else {
        summary.provenance_lines.join(" | ")
    };

    format!(
        "proof={} location={} preview={} provenance={}",
        summary.proof_outcome.as_str(),
        summary.payload_location,
        preview_text,
        provenance
    )
}

fn render_external_capability_decision(summary: &ExternalCapabilityDecisionSummary) -> String {
    let outcome = match summary.outcome {
        ExternalCapabilityDecisionOutcome::Allowed => "allowed",
        ExternalCapabilityDecisionOutcome::Rejected => "rejected",
    };
    let branch_id = summary
        .branch_id
        .map(|branch_id| branch_id.to_string())
        .unwrap_or_else(|| "none".to_string());
    let capability_id = summary
        .capability_id
        .map(|capability_id| capability_id.to_string())
        .unwrap_or_else(|| "none".to_string());
    let capability_descriptor = summary
        .capability_descriptor_id
        .as_deref()
        .unwrap_or("none");
    let action_descriptor = summary.action_descriptor_id.as_deref().unwrap_or("none");
    let action_id = summary.action_id.as_deref().unwrap_or("none");
    let action_title = summary.action_title.as_deref().unwrap_or("none");
    let required_scope = summary
        .required_scope
        .map(|scope| format!("{scope:?}"))
        .unwrap_or_else(|| "none".to_string());
    let scope = summary
        .scope
        .map(|scope| format!("{scope:?}"))
        .unwrap_or_else(|| "none".to_string());
    let revocation_state = summary
        .revocation_state
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "none".to_string());
    let policy = match (
        summary.policy_action.as_deref(),
        summary.policy_scope.as_deref(),
        summary.policy_resource.as_deref(),
    ) {
        (Some(action), Some(scope), Some(resource)) => format!("{action}/{scope}/{resource}"),
        _ => "none".to_string(),
    };
    let resource_id = summary.policy_resource_id.as_deref().unwrap_or("none");
    let observed_at = summary
        .observed_at
        .map(|timestamp| timestamp.to_rfc3339())
        .unwrap_or_else(|| "none".to_string());
    let rationale = if summary.rationale.is_empty() {
        "none".to_string()
    } else {
        summary.rationale.join(" | ")
    };

    format!(
        "{} branch={} observed_at={} outcome={} capability_descriptor={} action_descriptor={} action_id={} title={} scope={} required_scope={} state={} depth={} policy={} resource_id={} rationale={}",
        capability_id,
        branch_id,
        observed_at,
        outcome,
        capability_descriptor,
        action_descriptor,
        action_id,
        action_title,
        scope,
        required_scope,
        revocation_state,
        summary.chain_depth,
        policy,
        resource_id,
        rationale
    )
}

fn parse_datetime(value: &Value) -> Option<DateTime<Utc>> {
    value
        .as_str()
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|datetime| datetime.with_timezone(&Utc))
}

fn parse_task_id(raw: &str) -> Option<TaskId> {
    Uuid::parse_str(raw).ok().map(TaskId::from_uuid)
}
