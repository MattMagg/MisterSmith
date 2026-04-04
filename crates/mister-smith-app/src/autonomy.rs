//! Operator-facing autonomy status helpers.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use mister_smith_config::FrameworkConfig;
use mister_smith_core::{
    AgentId, CapabilityId, CoordinationPolicy, CoordinatorRuntimeProofView, DelegationScope,
    DurableWorkflowLifecycleState, ExecutionGraphId, GraphState, OperatorResultPreview,
    OrchestrationQualityView, ProofOutcomeClassification, RepairDirectiveAction,
    ResultProvenanceSummary, RevocationState, SessionId, SessionRetainedResultView,
    StepEvaluationRecord, StepPolicySummaryView, SupervisionEvidenceView,
    SupervisionTargetKind, TaskId, TaskResultView, TaskShapeClassification, TaskShapeKind,
    TopologyKind, TopologyRationale, UnifiedResultEnvelope, VerifierVerdict,
};
use mister_smith_events::autonomy::merge_operator_result_preview;
use mister_smith_events::autonomy::AttestationSource;
use mister_smith_events::{
    AutonomyStatusView, EventBus, ExternalCapabilityDecisionOutcome,
    ExternalCapabilityDecisionSummary, ExternalCapabilityDecisionSurface, ResumeProvenanceSummary,
    StepRoutingDecisionSummary,
};
use mister_smith_persistence::postgres::queries;
use mister_smith_persistence::{lifecycle_decision_history, workflow_history};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::PgPool;
use uuid::Uuid;

const RESULT_PREVIEW_MAX_CHARS: usize = 160;
const ACCEPTED_TASK_INGRESS_METADATA_KEY: &str = "accepted_task_ingress";
const TASK_INGRESS_REQUEST_SURFACE: &str = "POST /api/v1/tasks";

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

#[derive(Debug, Clone)]
struct StepEvaluationCandidate {
    record: StepEvaluationRecord,
    plan_index: usize,
    repair_attempt_count: u32,
    verdict_rank: u8,
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
/// it with persisted workflow metadata continuity when available.
pub async fn status_from_bus_with_metadata_continuity(
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
        enrich_lifecycle_state(&mut view, &record.metadata);
        enrich_accepted_task_ingress_continuity(&mut view, &record.metadata);
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
    let lifecycle_summary = view
        .lifecycle_state
        .unwrap_or_else(|| {
            mister_smith_events::autonomy::lifecycle_state_for_graph_state(view.graph.state)
        })
        .as_str();
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
    let step_policy_summary = view
        .step_policy
        .as_ref()
        .or_else(|| {
            view.result_preview
                .as_ref()
                .and_then(|preview| preview.step_policy.as_ref())
        })
        .map(render_step_policy_summary)
        .unwrap_or_else(|| "none".to_string());
    let supervision_summary = view
        .supervision_evidence
        .as_ref()
        .map(render_supervision_evidence)
        .unwrap_or_else(|| "none".to_string());
    let runtime_truth_summary = view
        .result_preview
        .as_ref()
        .and_then(|preview| preview.runtime_truth.as_ref())
        .and_then(|summary| serde_json::to_value(summary).ok())
        .map(|value| render_runtime_truth_summary(&value))
        .unwrap_or_else(|| "none".to_string());
    let coordinator_runtime_summary = view
        .coordinator_runtime_proof
        .as_ref()
        .or_else(|| {
            view.result_preview
                .as_ref()
                .and_then(|preview| preview.coordinator_runtime_proof.as_ref())
        })
        .map(render_coordinator_runtime_proof)
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
        "workflow: {}\ngraph: {} {:?}\nsession: {}\nresume provenance: {}\nlifecycle: {}\ntopology: {:?} width={} shape={} structure={} dependency={} rationale={} signals={}\nfallback: {}\nteam sizing: {}\nbranches:\n{}\ncheckpoints:\n{}\nrouting:\n{}\nstep routing:\n{}\nstep policy: {}\nresult preview: {}\nruntime truth: {}\ncoordinator runtime: {}\nsupervision: {}\ninterventions:\n{}\ndelegation:\n{}\ndelegation alerts:\n{}\nexternal capability decisions:\n{}\nconservative: {}",
        view.graph.workflow_id,
        view.graph.graph_id,
        view.graph.state,
        session_summary,
        resume_summary,
        lifecycle_summary,
        view.topology.topology_kind,
        view.topology.parallelism_width,
        task_shape,
        structural_signals,
        dependency_shape,
        topology_reason,
        topology_signals,
        fallback_reason,
        team_sizing_summary,
        if branch_summary.is_empty() {
            "none".to_string()
        } else {
            branch_summary
        },
        if checkpoint_summary.is_empty() {
            "none".to_string()
        } else {
            checkpoint_summary
        },
        if routing_summary.is_empty() {
            "none".to_string()
        } else {
            routing_summary
        },
        if step_routing_summary.is_empty() {
            "none".to_string()
        } else {
            step_routing_summary
        },
        step_policy_summary,
        result_preview_summary,
        runtime_truth_summary,
        coordinator_runtime_summary,
        supervision_summary,
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

pub(crate) fn durable_lifecycle_state_from_metadata(
    metadata: &Value,
) -> Option<DurableWorkflowLifecycleState> {
    lifecycle_decision_history(metadata)
        .ok()
        .and_then(|history| history.last().map(|decision| decision.resulting_state))
        .or_else(|| {
            workflow_history(metadata)
                .ok()
                .and_then(|history| history.iter().rev().find_map(|event| event.lifecycle_state))
        })
}

pub(crate) fn lifecycle_state_from_status(status: &str) -> DurableWorkflowLifecycleState {
    DurableWorkflowLifecycleState::from_task_status(status)
}

pub(crate) fn enrich_lifecycle_state(view: &mut AutonomyStatusView, metadata: &Value) {
    view.lifecycle_state = Some(
        durable_lifecycle_state_from_metadata(metadata)
            .unwrap_or_else(|| DurableWorkflowLifecycleState::from_graph_state(view.graph.state)),
    );
}

pub(crate) fn enrich_accepted_task_ingress_continuity(
    view: &mut AutonomyStatusView,
    metadata: &Value,
) {
    if view.external_capability_decisions.iter().any(|decision| {
        decision.boundary_surface == Some(ExternalCapabilityDecisionSurface::TaskIngress)
    }) {
        return;
    }

    let Some(accepted_ingress) = metadata
        .get(ACCEPTED_TASK_INGRESS_METADATA_KEY)
        .and_then(Value::as_object)
    else {
        return;
    };

    let Some(request_surface) = accepted_ingress
        .get("request_surface")
        .and_then(Value::as_str)
    else {
        return;
    };
    if request_surface != TASK_INGRESS_REQUEST_SURFACE {
        return;
    }

    let source_metadata_key = accepted_ingress
        .get("source_metadata_key")
        .and_then(Value::as_str)
        .unwrap_or("external_delegation");
    let capability_descriptor_id = accepted_ingress
        .get("capability_descriptor_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let action_descriptor_id = accepted_ingress
        .get("action_descriptor_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let action_id = accepted_ingress
        .get("action_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let action_title = accepted_ingress
        .get("action_title")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let scope = accepted_ingress
        .get("scope")
        .and_then(parse_enum_value::<DelegationScope>);
    let required_scope = accepted_ingress
        .get("required_scope")
        .and_then(parse_enum_value::<DelegationScope>);
    let policy_action = accepted_ingress
        .get("policy_action")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let policy_resource = accepted_ingress
        .get("policy_resource")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let policy_scope = accepted_ingress
        .get("policy_scope")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let policy_resource_id = accepted_ingress
        .get("policy_resource_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let revocation_state_raw = accepted_ingress
        .get("revocation_state")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let revocation_state = revocation_state_raw
        .as_ref()
        .and_then(|raw| serde_json::from_value::<RevocationState>(Value::String(raw.clone())).ok());
    let chain_depth = accepted_ingress
        .get("chain_depth")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();

    let mut rationale = vec![
        format!("accepted delegated task ingress metadata was present at {request_surface}"),
        format!(
            "continuity projected from workflow metadata {ACCEPTED_TASK_INGRESS_METADATA_KEY} sourced from {source_metadata_key}"
        ),
        "trust state derived from persisted metadata, not re-verified at inspection time".to_string(),
    ];
    if let Some(descriptor_id) = capability_descriptor_id.as_deref() {
        rationale.push(format!(
            "capability descriptor at accepted task ingress was '{descriptor_id}'"
        ));
    }
    if let Some(descriptor_id) = action_descriptor_id.as_deref() {
        rationale.push(format!(
            "accepted task ingress requested descriptor '{descriptor_id}'"
        ));
    }
    if let Some(required_scope) = required_scope {
        rationale.push(format!(
            "accepted task ingress required scope {:?} while the capability carried {:?}",
            required_scope, scope
        ));
    }
    if let Some(revocation_state) = revocation_state {
        rationale.push(format!(
            "revocation_state was {:?} in persisted ingress metadata",
            revocation_state
        ));
    } else if revocation_state_raw.is_some() {
        rationale.push(
            "revocation_state not present or invalid in persisted ingress metadata".to_string(),
        );
    } else {
        rationale.push(
            "revocation_state not present or invalid in persisted ingress metadata".to_string(),
        );
    }
    if let (Some(action), Some(policy_scope), Some(resource)) = (
        policy_action.as_deref(),
        policy_scope.as_deref(),
        policy_resource.as_deref(),
    ) {
        rationale.push(format!(
            "policy continuity preserved as {action}/{policy_scope}/{resource}"
        ));
    }

    let outcome = match revocation_state {
        Some(RevocationState::Active) => ExternalCapabilityDecisionOutcome::Allowed,
        Some(RevocationState::Revoked) | Some(RevocationState::Expired) => {
            ExternalCapabilityDecisionOutcome::Rejected
        }
        None => ExternalCapabilityDecisionOutcome::Rejected,
    };

    if matches!(revocation_state, Some(RevocationState::Revoked)) {
        rationale.push("revocation_state was revoked at ingress time".to_string());
    } else if matches!(revocation_state, Some(RevocationState::Expired)) {
        rationale.push("revocation_state was expired at ingress time".to_string());
    }

    view.external_capability_decisions
        .push(ExternalCapabilityDecisionSummary {
            boundary_surface: Some(ExternalCapabilityDecisionSurface::TaskIngress),
            branch_id: None,
            capability_id: accepted_ingress
                .get("capability_id")
                .and_then(parse_enum_value::<CapabilityId>),
            capability_descriptor_id,
            action_descriptor_id,
            action_id,
            action_title,
            scope,
            required_scope,
            policy_action,
            policy_resource,
            policy_scope,
            policy_resource_id,
            revocation_state,
            attestation_source: Some(AttestationSource::MetadataContinuity),
            chain_depth,
            outcome,
            observed_at: None,
            rationale,
        });
    sort_external_capability_decisions(&mut view.external_capability_decisions);
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

fn sort_external_capability_decisions(decisions: &mut [ExternalCapabilityDecisionSummary]) {
    decisions.sort_by(|left, right| {
        left.observed_at.cmp(&right.observed_at).then_with(|| {
            external_capability_decision_sort_key(left)
                .cmp(&external_capability_decision_sort_key(right))
        })
    });
}

fn external_capability_decision_sort_key(decision: &ExternalCapabilityDecisionSummary) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        decision
            .boundary_surface
            .map(|surface| format!("{surface:?}"))
            .unwrap_or_else(|| "none".to_string()),
        decision
            .branch_id
            .map(|branch_id| branch_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        decision
            .capability_id
            .map(|capability_id| capability_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        decision.action_id.as_deref().unwrap_or("none"),
        decision.action_descriptor_id.as_deref().unwrap_or("none"),
        match decision.outcome {
            ExternalCapabilityDecisionOutcome::Allowed => "allowed",
            ExternalCapabilityDecisionOutcome::Rejected => "rejected",
        }
    )
}

fn parse_enum_value<T>(value: &Value) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value.clone()).ok()
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
        coordinator_runtime_proof: None,
    }
}

pub(crate) fn build_task_result_view(
    status: &str,
    canonical_result: UnifiedResultEnvelope,
    supervision_evidence: Option<SupervisionEvidenceView>,
    step_policy: Option<StepPolicySummaryView>,
) -> TaskResultView {
    let orchestration_quality =
        orchestration_quality_projection(&canonical_result).map(|projection| projection.view);
    let runtime_truth = synthesized_runtime_truth_value(&canonical_result)
        .and_then(|value| serde_json::from_value(value).ok());
    TaskResultView {
        workflow_id: canonical_result.workflow_id,
        status: status.to_string(),
        proof_outcome: canonical_result.proof_outcome,
        orchestration_quality,
        runtime_truth,
        supervision_evidence,
        step_policy,
        coordinator_runtime_proof: canonical_result.coordinator_runtime_proof.clone(),
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
    let existing_preview = view.result_preview.clone();
    let workflow_id = view.graph.workflow_id;
    let task_result_step_policy = task_result.and_then(task_result_step_policy);
    let payload_preview = task_result
        .and_then(|value| {
            canonical_result_from_value(value, Some(status_hint))
                .filter(|result| result.workflow_id == workflow_id)
                .map(|result| {
                    let mut preview = operator_result_preview(&result, "task.result", view);
                    preview.step_policy = task_result_step_policy.clone();
                    preview
                })
        })
        .or_else(|| {
            metadata.get("final_result").and_then(|value| {
                canonical_result_from_value(value, Some(status_hint))
                    .filter(|result| result.workflow_id == workflow_id)
                    .map(|result| operator_result_preview(&result, "metadata.final_result", view))
            })
        });

    if let Some(mut preview) = payload_preview {
        if let Some(existing_preview) = existing_preview.as_ref() {
            preview = merge_operator_result_preview(&preview, existing_preview);
        }
        if view.step_policy.is_none() {
            view.step_policy = preview.step_policy.clone();
        }
        view.result_preview = Some(preview);
    }
}

pub(crate) fn synthesize_failed_before_graph_status(
    workflow_id: TaskId,
    metadata: &Value,
    task_result: Option<&Value>,
) -> Option<AutonomyStatusView> {
    let canonical_result = task_result
        .and_then(|value| canonical_result_from_value(value, Some("failed")))
        .filter(|result| {
            result.workflow_id == workflow_id
                && result.proof_outcome == ProofOutcomeClassification::FailedBeforeGraph
        })
        .or_else(|| {
            metadata
                .get("final_result")
                .and_then(|value| canonical_result_from_value(value, Some("failed")))
                .filter(|result| {
                    result.workflow_id == workflow_id
                        && result.proof_outcome == ProofOutcomeClassification::FailedBeforeGraph
                })
        })?;

    let plan_summary = failed_before_graph_plan_summary(&canonical_result);
    let graph_id = ExecutionGraphId::from_uuid(*workflow_id.as_ref());

    Some(AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
        lifecycle_state: Some(DurableWorkflowLifecycleState::Failed),
        graph: mister_smith_events::ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Failed,
            branch_count: plan_summary.branch_count,
            node_count: plan_summary.node_count,
            active_topology: Some(plan_summary.topology_kind),
        },
        topology: mister_smith_events::TopologyPlanSummary {
            graph_id,
            topology_kind: plan_summary.topology_kind,
            parallelism_width: plan_summary.parallelism_width,
            task_shape: plan_summary.task_shape,
            coordination_policy: plan_summary.coordination_policy,
            rationale: plan_summary.rationale,
            fallback_topology: Some(TopologyKind::Sequential),
        },
        team_sizing: None,
        branches: vec![],
        checkpoint_lineage: vec![],
        memory_pressure: vec![],
        routing_history: vec![],
        step_routing_history: vec![],
        result_preview: None,
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        delegation_records: vec![],
        subordinate_inbox: vec![],
        subagent_states: vec![],
        delegated_work_evidence: vec![],
        coordinator_decisions: vec![],
        coordinator_runtime_proof: None,
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        runtime_truth: None,
        step_policy: None,
        conservative_reasons: vec![
            "workflow failed before graph publication".to_string(),
            "autonomy status reconstructed from persisted canonical result".to_string(),
        ],
    })
}

pub(crate) fn retained_result_view(
    task_result: &Value,
    turn_index: u32,
    status: &str,
) -> Option<SessionRetainedResultView> {
    build_session_retained_result(task_result, turn_index, status)
}

pub(crate) fn retained_assistant_result(
    task_result: &Value,
    turn_index: u32,
    status: &str,
) -> Option<Value> {
    retained_result_view(task_result, turn_index, status)
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
    let runtime_truth = task_result
        .get("runtime_truth")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok());

    Some(SessionRetainedResultView {
        workflow_id: canonical_result.workflow_id,
        turn_index,
        status: status.to_string(),
        assistant_result,
        preview,
        runtime_truth,
        provenance: result_provenance(&canonical_result, None, None),
    })
}

fn operator_result_preview(
    canonical_result: &UnifiedResultEnvelope,
    payload_location: &str,
    view: &AutonomyStatusView,
) -> OperatorResultPreview {
    OperatorResultPreview {
        workflow_id: canonical_result.workflow_id,
        proof_outcome: canonical_result.proof_outcome,
        preview_text: preview_text(&canonical_result.aggregated_result),
        payload_location: payload_location.to_string(),
        orchestration_quality: orchestration_quality_projection(canonical_result)
            .map(|projection| projection.view),
        runtime_truth: synthesized_runtime_truth_value(canonical_result)
            .and_then(|value| serde_json::from_value(value).ok()),
        step_policy: None,
        coordinator_runtime_proof: canonical_result.coordinator_runtime_proof.clone(),
        provenance_lines: result_preview_provenance(canonical_result, payload_location, view),
    }
}

fn task_result_step_policy(task_result: &Value) -> Option<StepPolicySummaryView> {
    task_result
        .get("step_policy")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

fn synthesized_runtime_truth_value(canonical_result: &UnifiedResultEnvelope) -> Option<Value> {
    if canonical_result.proof_outcome == ProofOutcomeClassification::FailedBeforeGraph {
        return None;
    }

    Some(serde_json::json!({
        "evidence_class": "placeholder_or_simulated_step_completion",
        "proof_boundary": {
            "graph_execution": "workflow graph executed successfully",
            "semantic_completion": "semantic completion not yet proven",
            "grounded_tool_execution": "grounded tool execution: none/minimal",
            "task_proof": "result is orchestration proof, not substantive task proof",
        },
        "run_trace": {
            "trace_root_id": canonical_result.workflow_id,
            "workflow_id": canonical_result.workflow_id,
            "graph_id": Value::Null,
            "branch_id": Value::Null,
            "node_id": Value::Null,
            "relationships": ["graph", "tool_boundary"],
        },
        "grounded_evidence": [],
    }))
}

struct FailedBeforeGraphPlanSummary {
    branch_count: usize,
    node_count: usize,
    topology_kind: TopologyKind,
    parallelism_width: usize,
    coordination_policy: CoordinationPolicy,
    task_shape: TaskShapeClassification,
    rationale: TopologyRationale,
}

fn failed_before_graph_plan_summary(
    canonical_result: &UnifiedResultEnvelope,
) -> FailedBeforeGraphPlanSummary {
    let steps = canonical_result
        .execution_plan
        .get("steps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let node_count = steps.len();

    let mut step_ids = Vec::new();
    let mut dependencies_by_step = BTreeMap::<String, Vec<String>>::new();
    let mut dependency_fanout = BTreeMap::<String, usize>::new();
    let mut root_count = 0usize;
    let mut has_join = false;

    for (index, raw_step) in steps.iter().enumerate() {
        let Some(step) = raw_step.as_object() else {
            continue;
        };
        let step_id = step
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("step-{}", index + 1));
        let dependencies = step
            .get("depends_on")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if dependencies.is_empty() {
            root_count += 1;
        }
        if dependencies.len() > 1 {
            has_join = true;
        }
        for dependency in &dependencies {
            *dependency_fanout.entry(dependency.clone()).or_insert(0) += 1;
        }

        step_ids.push(step_id.clone());
        dependencies_by_step.insert(step_id, dependencies);
    }

    let max_depth = max_plan_depth(&step_ids, &dependencies_by_step);
    let max_frontier_width = max_plan_frontier_width(&step_ids, &dependencies_by_step);
    let has_fanout = dependency_fanout.values().any(|count| *count > 1);
    let branch_count = if root_count > 0 {
        root_count
    } else if node_count > 0 {
        1
    } else {
        0
    };
    let parallelism_width = if node_count > 0 {
        max_frontier_width.max(1)
    } else {
        0
    };
    let topology_kind = if parallelism_width <= 1 {
        TopologyKind::Sequential
    } else if has_join {
        TopologyKind::Hybrid
    } else {
        TopologyKind::Parallel
    };
    let coordination_policy = if parallelism_width <= 1 {
        CoordinationPolicy::StrictSequence
    } else {
        CoordinationPolicy::Barrier
    };
    let task_shape_kind = if node_count <= 1 || (parallelism_width <= 1 && !has_join) {
        TaskShapeKind::StrictChain
    } else if has_join {
        TaskShapeKind::FanoutJoin
    } else if has_fanout && max_depth > 2 {
        TaskShapeKind::HierarchicalFanout
    } else if parallelism_width > 1 {
        TaskShapeKind::ParallelFanout
    } else {
        TaskShapeKind::MixedGraph
    };
    let task_shape = TaskShapeClassification {
        kind: task_shape_kind,
        root_count,
        max_parallel_width: parallelism_width,
        max_depth,
        has_join,
        has_fanout,
        structural_signals: vec![
            format!("stored_plan_nodes:{node_count}"),
            format!("stored_plan_roots:{root_count}"),
            format!("stored_plan_frontier_width:{max_frontier_width}"),
            format!("stored_plan_depth:{max_depth}"),
        ],
    };
    let rationale = TopologyRationale {
        dependency_shape: format!("stored execution plan reconstructed with {node_count} node(s)"),
        operational_signals: vec![
            "workflow failed before graph publication".to_string(),
            format!("proof_outcome={}", canonical_result.proof_outcome.as_str()),
            format!(
                "recorded_step_results={}",
                canonical_result.step_results.len()
            ),
        ],
        selected_for: "preserve bounded operator-visible parity from persisted failure metadata"
            .to_string(),
        fallback_reason: Some(
            "graph compilation failed before a live autonomy snapshot could be published"
                .to_string(),
        ),
    };

    FailedBeforeGraphPlanSummary {
        branch_count,
        node_count,
        topology_kind,
        parallelism_width,
        coordination_policy,
        task_shape,
        rationale,
    }
}

fn max_plan_depth(
    step_ids: &[String],
    dependencies_by_step: &BTreeMap<String, Vec<String>>,
) -> usize {
    let mut memo = BTreeMap::<String, usize>::new();
    let mut visiting = BTreeSet::<String>::new();

    step_ids
        .iter()
        .map(|step_id| plan_step_depth(step_id, dependencies_by_step, &mut memo, &mut visiting))
        .max()
        .unwrap_or(0)
}

fn max_plan_frontier_width(
    step_ids: &[String],
    dependencies_by_step: &BTreeMap<String, Vec<String>>,
) -> usize {
    let mut memo = BTreeMap::<String, usize>::new();
    let mut visiting = BTreeSet::<String>::new();
    let mut frontier_counts = BTreeMap::<usize, usize>::new();

    for step_id in step_ids {
        let depth = plan_step_depth(step_id, dependencies_by_step, &mut memo, &mut visiting);
        *frontier_counts.entry(depth).or_insert(0) += 1;
    }

    frontier_counts.values().copied().max().unwrap_or(0)
}

fn plan_step_depth(
    step_id: &str,
    dependencies_by_step: &BTreeMap<String, Vec<String>>,
    memo: &mut BTreeMap<String, usize>,
    visiting: &mut BTreeSet<String>,
) -> usize {
    if let Some(depth) = memo.get(step_id) {
        return *depth;
    }
    if !visiting.insert(step_id.to_string()) {
        return 1;
    }

    let depth = dependencies_by_step
        .get(step_id)
        .map(|dependencies| {
            if dependencies.is_empty() {
                1
            } else {
                1 + dependencies
                    .iter()
                    .map(|dependency| {
                        plan_step_depth(dependency, dependencies_by_step, memo, visiting)
                    })
                    .max()
                    .unwrap_or(0)
            }
        })
        .unwrap_or(1);

    visiting.remove(step_id);
    memo.insert(step_id.to_string(), depth);
    depth
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

    if let Some(proof) = canonical_result.coordinator_runtime_proof.as_ref() {
        let session_id = proof
            .delegation_records
            .iter()
            .find_map(|record| record.session_id)
            .map(|value| value.to_string());
        let delegation_ids = proof
            .delegation_records
            .iter()
            .map(|record| record.delegation_id.clone())
            .collect::<Vec<_>>();
        let delegated_child_ids = proof
            .delegation_records
            .iter()
            .map(|record| record.subagent_id.to_string())
            .collect::<Vec<_>>();
        let decision_ids = proof
            .coordinator_decisions
            .iter()
            .map(|decision| decision.decision_id.clone())
            .collect::<Vec<_>>();
        let evidence_refs = proof
            .delegated_work_evidence
            .iter()
            .flat_map(|evidence| evidence.artifact_refs.iter().cloned())
            .collect::<Vec<_>>();

        payload.insert(
            "coordinator_runtime_follow_up".to_string(),
            serde_json::json!({
                "session_id": session_id,
                "coordinator_agent_id": proof.coordinator_agent_id,
                "proof_boundary": proof.proof_boundary,
                "session_follow_up_note": proof.session_follow_up_note,
                "delegation_ids": delegation_ids,
                "delegated_child_ids": delegated_child_ids,
                "decision_ids": decision_ids,
                "evidence_refs": evidence_refs,
            }),
        );
    }

    Value::Object(payload)
}

fn result_provenance(
    canonical_result: &UnifiedResultEnvelope,
    graph_state: Option<String>,
    graph_id: Option<String>,
) -> ResultProvenanceSummary {
    let mut source_fields = vec![
        "metadata.final_result".to_string(),
        "metadata.aggregated_result".to_string(),
    ];
    if canonical_result.coordinator_runtime_proof.is_some() {
        source_fields.push("metadata.final_result.coordinator_runtime_proof".to_string());
    }

    ResultProvenanceSummary {
        runtime_execution_mode: canonical_result.runtime_execution_mode.clone(),
        graph_state,
        graph_id,
        source_fields,
    }
}

fn result_preview_provenance(
    canonical_result: &UnifiedResultEnvelope,
    payload_location: &str,
    view: &AutonomyStatusView,
) -> Vec<String> {
    let outcome_line = match canonical_result.proof_outcome {
        ProofOutcomeClassification::GraphFormedAndCompleted => {
            "graph formed and completed before final result publication"
        }
        ProofOutcomeClassification::CollapsedToSequential => "planner emitted one sequential step",
        ProofOutcomeClassification::FailedBeforeGraph => {
            "workflow failed before usable graph formation"
        }
    };

    let mut lines = vec![
        outcome_line.to_string(),
        format!(
            "provider={} model={}",
            canonical_result.provider_kind, canonical_result.model_id
        ),
    ];
    if let Some(runtime_line) =
        runtime_execution_mode_provenance(&canonical_result.runtime_execution_mode)
    {
        lines.push(runtime_line);
    }
    lines.push(format!(
        "graph state {:?} with topology {:?} ({} branch(es), {} node(s))",
        view.graph.state,
        view.topology.topology_kind,
        view.graph.branch_count,
        view.graph.node_count
    ));
    if !view.routing_history.is_empty() {
        lines.push(format!(
            "routing history retained {} decision(s)",
            view.routing_history.len()
        ));
    }
    if let Some(step_line) = latest_step_routing_provenance(view) {
        lines.push(step_line);
    }
    if let Some(projection) = orchestration_quality_projection(canonical_result) {
        lines.push(render_orchestration_quality_line(&projection.view));
        if matches!(
            projection.source,
            OrchestrationQualityProjectionSource::PlannerRepairStep
        ) {
            lines.push(format!(
                "orchestration quality inferred from planner repair step '{}' without verifier_policy",
                projection.view.step_id
            ));
        }
    }
    lines.extend([
        "canonical result stored in metadata.final_result".to_string(),
        "aggregated payload nested under metadata.aggregated_result".to_string(),
        format!("full payload remains recoverable from {payload_location}"),
        "session assistant_result derives from the canonical result object".to_string(),
    ]);
    if let Some(proof) = canonical_result.coordinator_runtime_proof.as_ref() {
        lines.push(format!(
            "packet 026 proof boundary: {}",
            proof.proof_boundary
        ));
        lines.push(format!(
            "packet 026 follow-up stays bounded to IDs and evidence refs: {}",
            proof.session_follow_up_note
        ));
    }
    lines
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrchestrationQualityProjectionSource {
    StepEvaluation,
    PlannerRepairStep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrchestrationQualityProjection {
    view: OrchestrationQualityView,
    source: OrchestrationQualityProjectionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannerRepairCandidate {
    plan_index: usize,
    step_id: String,
    repair_action: RepairDirectiveAction,
    clarification_attempt_count: u32,
    last_stable_step_id: Option<String>,
}

fn orchestration_quality_projection(
    canonical_result: &UnifiedResultEnvelope,
) -> Option<OrchestrationQualityProjection> {
    let plan_index_by_step = execution_plan_step_indices(&canonical_result.execution_plan);
    let candidates = step_evaluation_candidates(canonical_result, &plan_index_by_step);
    if let Some(terminal) = select_terminal_step_evaluation(&candidates) {
        let recovered_repair_action = terminal
            .record
            .repair_directive
            .as_ref()
            .map(|directive| directive.action)
            .or_else(|| recover_prior_repair_action(&candidates, &terminal.record));
        let clarification_attempt_count = terminal
            .record
            .clarification_request
            .as_ref()
            .map(|request| request.attempt_count)
            .unwrap_or(0);
        let checkpoint_ref = terminal
            .record
            .failure_context_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.checkpoint_ref.clone())
            .or_else(|| terminal.record.checkpoint_ref.clone());
        let last_stable_step_id = terminal
            .record
            .failure_context_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.last_stable_step_id.clone());
        let failure_context_ref = failure_context_ref(&terminal.record);

        return Some(OrchestrationQualityProjection {
            view: OrchestrationQualityView {
                step_id: terminal.record.step_id.clone(),
                verdict: terminal.record.verdict,
                repair_action: recovered_repair_action,
                clarification_attempt_count,
                checkpoint_ref,
                last_stable_step_id,
                failure_context_ref,
                outcome_summary: orchestration_outcome_summary(
                    terminal.record.verdict,
                    recovered_repair_action,
                ),
            },
            source: OrchestrationQualityProjectionSource::StepEvaluation,
        });
    }

    let planner_repair_candidates =
        planner_repair_candidates(canonical_result, &plan_index_by_step);
    let inferred = select_terminal_planner_repair_candidate(&planner_repair_candidates)?;
    let verdict = if canonical_result.proof_outcome == ProofOutcomeClassification::FailedBeforeGraph
    {
        VerifierVerdict::Rejected
    } else {
        VerifierVerdict::Accepted
    };

    Some(OrchestrationQualityProjection {
        view: OrchestrationQualityView {
            step_id: inferred.step_id.clone(),
            verdict,
            repair_action: Some(inferred.repair_action),
            clarification_attempt_count: inferred.clarification_attempt_count,
            checkpoint_ref: inferred
                .last_stable_step_id
                .as_ref()
                .map(|step_id| format!("planner-step:{step_id}")),
            last_stable_step_id: inferred.last_stable_step_id.clone(),
            failure_context_ref: Some(format!(
                "planner:{}/{}",
                inferred.step_id,
                inferred.repair_action.as_str()
            )),
            outcome_summary: orchestration_outcome_summary(verdict, Some(inferred.repair_action)),
        },
        source: OrchestrationQualityProjectionSource::PlannerRepairStep,
    })
}

fn execution_plan_step_indices(execution_plan: &Value) -> BTreeMap<String, usize> {
    execution_plan
        .get("steps")
        .and_then(Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .enumerate()
                .filter_map(|(index, step)| {
                    step.get("id")
                        .and_then(Value::as_str)
                        .map(|step_id| (step_id.to_string(), index + 1))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn step_evaluation_candidates(
    canonical_result: &UnifiedResultEnvelope,
    plan_index_by_step: &BTreeMap<String, usize>,
) -> Vec<StepEvaluationCandidate> {
    canonical_result
        .step_results
        .iter()
        .filter_map(|step_result| {
            let evaluation = step_result
                .get("step_evaluation")
                .cloned()
                .and_then(|value| serde_json::from_value::<StepEvaluationRecord>(value).ok())?;
            let repair_attempt_count = evaluation
                .failure_context_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.attempt_count)
                .or_else(|| {
                    evaluation
                        .clarification_request
                        .as_ref()
                        .map(|request| request.attempt_count)
                })
                .unwrap_or(0);

            Some(StepEvaluationCandidate {
                plan_index: plan_index_by_step
                    .get(&evaluation.step_id)
                    .copied()
                    .unwrap_or(0),
                repair_attempt_count,
                verdict_rank: if evaluation.verdict == VerifierVerdict::Accepted {
                    1
                } else {
                    0
                },
                record: evaluation,
            })
        })
        .collect()
}

fn select_terminal_step_evaluation(
    candidates: &[StepEvaluationCandidate],
) -> Option<&StepEvaluationCandidate> {
    candidates.iter().max_by_key(|candidate| {
        (
            candidate.plan_index,
            candidate.repair_attempt_count,
            candidate.verdict_rank,
        )
    })
}

fn recover_prior_repair_action(
    candidates: &[StepEvaluationCandidate],
    terminal_record: &StepEvaluationRecord,
) -> Option<RepairDirectiveAction> {
    let terminal_failure_context = failure_context_ref(terminal_record);

    let by_failure_context = terminal_failure_context
        .as_deref()
        .and_then(|terminal_failure_ref| {
            candidates
                .iter()
                .filter(|candidate| {
                    failure_context_ref(&candidate.record).as_deref() == Some(terminal_failure_ref)
                })
                .filter_map(|candidate| {
                    candidate
                        .record
                        .repair_directive
                        .as_ref()
                        .map(|directive| (candidate.repair_attempt_count, directive.action))
                })
                .max_by_key(|(attempt_count, _)| *attempt_count)
                .map(|(_, action)| action)
        });

    by_failure_context.or_else(|| {
        candidates
            .iter()
            .filter(|candidate| candidate.record.step_id == terminal_record.step_id)
            .filter_map(|candidate| {
                candidate
                    .record
                    .repair_directive
                    .as_ref()
                    .map(|directive| (candidate.repair_attempt_count, directive.action))
            })
            .max_by_key(|(attempt_count, _)| *attempt_count)
            .map(|(_, action)| action)
    })
}

fn planner_repair_candidates(
    canonical_result: &UnifiedResultEnvelope,
    plan_index_by_step: &BTreeMap<String, usize>,
) -> Vec<PlannerRepairCandidate> {
    canonical_result
        .step_results
        .iter()
        .filter_map(|step_result| {
            let task = step_result
                .get("result")
                .and_then(|result| result.get("task"))
                .and_then(Value::as_object)?;
            let step_id = task
                .get("step_id")
                .and_then(Value::as_str)
                .map(str::to_string)?;
            let action = task
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let description = task
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let repair_action = infer_planner_repair_action(action, description)?;
            let last_stable_step_id = task
                .get("dependencies")
                .and_then(Value::as_array)
                .and_then(|dependencies| dependencies.last())
                .and_then(Value::as_str)
                .map(str::to_string);

            Some(PlannerRepairCandidate {
                plan_index: plan_index_by_step.get(&step_id).copied().unwrap_or(0),
                step_id,
                repair_action,
                clarification_attempt_count: if repair_action
                    == RepairDirectiveAction::ClarifyHandoff
                {
                    1
                } else {
                    0
                },
                last_stable_step_id,
            })
        })
        .collect()
}

fn select_terminal_planner_repair_candidate(
    candidates: &[PlannerRepairCandidate],
) -> Option<&PlannerRepairCandidate> {
    candidates
        .iter()
        .max_by_key(|candidate| candidate.plan_index)
}

fn infer_planner_repair_action(action: &str, description: &str) -> Option<RepairDirectiveAction> {
    let normalized_action = action.trim().to_ascii_lowercase();
    let normalized_description = description.trim().to_ascii_lowercase();
    let action_is_final_output = [
        "deliver",
        "write",
        "return",
        "summarize",
        "combine",
        "merge",
        "answer",
    ]
    .iter()
    .any(|token| normalized_action.contains(token));

    if action_is_final_output {
        return None;
    }

    if normalized_action.contains("clarify")
        || normalized_action.contains("missing_context")
        || normalized_action.contains("check_context")
        || normalized_action.contains("resolve_missing_context")
        || normalized_description.contains("request clarification")
        || normalized_description.contains("missing required context")
        || normalized_description.contains("if context is missing")
        || normalized_description.contains("missing context")
        || normalized_description.contains("missing-context")
        || normalized_description.contains("missing_context")
    {
        return Some(RepairDirectiveAction::ClarifyHandoff);
    }
    if normalized_action.contains("replan")
        || normalized_description.contains("replan from checkpoint")
    {
        return Some(RepairDirectiveAction::ReplanFromCheckpoint);
    }
    if normalized_action.contains("retry")
        || normalized_action.contains("repair")
        || normalized_action.contains("resolve")
        || normalized_description.contains("retry")
        || normalized_description.contains("bounded local repair")
    {
        return Some(RepairDirectiveAction::RetryStep);
    }

    None
}

fn failure_context_ref(record: &StepEvaluationRecord) -> Option<String> {
    record
        .failure_context_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.failure_context_ref.clone())
        .or_else(|| {
            record
                .repair_directive
                .as_ref()
                .map(|directive| directive.failure_context_ref.clone())
        })
}

fn orchestration_outcome_summary(
    verdict: VerifierVerdict,
    repair_action: Option<RepairDirectiveAction>,
) -> String {
    match (verdict, repair_action) {
        (VerifierVerdict::Accepted, Some(action)) => {
            format!("accepted_after_{}", action.as_str())
        }
        (VerifierVerdict::Accepted, None) => "accepted_without_repair".to_string(),
        (VerifierVerdict::Rejected, Some(action)) => {
            format!("rejected_with_{}", action.as_str())
        }
        (VerifierVerdict::Rejected, None) => "rejected_without_repair".to_string(),
    }
}

fn render_orchestration_quality_line(summary: &OrchestrationQualityView) -> String {
    let repair_action = summary
        .repair_action
        .map(RepairDirectiveAction::as_str)
        .unwrap_or("none");
    let checkpoint_ref = summary.checkpoint_ref.as_deref().unwrap_or("none");
    let last_stable_step = summary.last_stable_step_id.as_deref().unwrap_or("none");
    let failure_context = summary.failure_context_ref.as_deref().unwrap_or("none");

    format!(
        "orchestration quality step={} verdict={} repair={} clarification_attempts={} checkpoint={} last_stable_step={} failure_context={} outcome={}",
        summary.step_id,
        summary.verdict.as_str(),
        repair_action,
        summary.clarification_attempt_count,
        checkpoint_ref,
        last_stable_step,
        failure_context,
        summary.outcome_summary
    )
}

fn runtime_execution_mode_provenance(runtime_execution_mode: &Value) -> Option<String> {
    let object = runtime_execution_mode.as_object()?;
    let mut parts = Vec::new();

    if let Some(boundary) = object.get("execution_boundary").and_then(Value::as_str) {
        parts.push(format!("boundary={boundary}"));
    }
    if let Some(runner) = object.get("workflow_runner").and_then(Value::as_str) {
        parts.push(format!("runner={runner}"));
    }
    if let Some(planner) = object.get("planner_lifecycle").and_then(Value::as_str) {
        parts.push(format!("planner={planner}"));
    }
    if let Some(executor) = object.get("executor_lifecycle").and_then(Value::as_str) {
        parts.push(format!("executor={executor}"));
    }
    if let Some(tool_name) = object.get("tool_name").and_then(Value::as_str) {
        parts.push(format!("tool={tool_name}"));
    }
    if let Some(routing_policy) = object.get("routing_policy").and_then(Value::as_str) {
        parts.push(format!("routing_policy={routing_policy}"));
    }
    if let Some(count) = object
        .get("registered_provider_count")
        .and_then(Value::as_u64)
    {
        parts.push(format!("registered_providers={count}"));
    }
    if let Some(budget_root) = object.get("budget_root").and_then(Value::as_str) {
        parts.push(format!("budget_root={budget_root}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("runtime execution mode {}", parts.join(" ")))
    }
}

fn latest_step_routing_provenance(view: &AutonomyStatusView) -> Option<String> {
    let latest = view.step_routing_history.last()?;
    let checkpoints = if latest.triggered_checkpoints.is_empty() {
        "none".to_string()
    } else {
        latest.triggered_checkpoints.join(" | ")
    };

    Some(format!(
        "latest step routing tier={} action={} checkpoints={}",
        latest.tier, latest.action, checkpoints
    ))
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
        coordinator_runtime_proof: object
            .get("coordinator_runtime_proof")
            .cloned()
            .and_then(|raw| serde_json::from_value(raw).ok()),
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
        .and_then(transcript_assistant_result_payload)
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

fn transcript_assistant_result_payload(entry: &Value) -> Option<&Value> {
    let projection = entry.get("assistant_result")?;
    projection.get("assistant_result").or(Some(projection))
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
    let mut rendered = format!(
        "proof={} location={} preview={}",
        summary.proof_outcome.as_str(),
        summary.payload_location,
        preview_text,
    );
    if summary.provenance_lines.is_empty() {
        rendered.push_str("\n  provenance: none");
    } else {
        rendered.push_str("\n  provenance:");
        for line in &summary.provenance_lines {
            rendered.push_str("\n  - ");
            rendered.push_str(line);
        }
    }
    if let Some(proof) = summary.coordinator_runtime_proof.as_ref() {
        rendered.push_str("\n  coordinator_runtime:");
        rendered.push_str("\n  - ");
        rendered.push_str(&render_coordinator_runtime_proof(proof).replace('\n', "\n  "));
    }
    if let Some(orchestration_quality) = summary.orchestration_quality.as_ref() {
        let repair_action = orchestration_quality
            .repair_action
            .map(RepairDirectiveAction::as_str)
            .unwrap_or("none");
        rendered.push_str("\n  orchestration_quality:");
        rendered.push_str("\n  - step=");
        rendered.push_str(&orchestration_quality.step_id);
        rendered.push_str(" verdict=");
        rendered.push_str(orchestration_quality.verdict.as_str());
        rendered.push_str(" repair=");
        rendered.push_str(repair_action);
        rendered.push_str(" clarification_attempts=");
        rendered.push_str(
            &orchestration_quality
                .clarification_attempt_count
                .to_string(),
        );
        rendered.push_str(" checkpoint=");
        rendered.push_str(
            orchestration_quality
                .checkpoint_ref
                .as_deref()
                .unwrap_or("none"),
        );
        rendered.push_str(" last_stable_step=");
        rendered.push_str(
            orchestration_quality
                .last_stable_step_id
                .as_deref()
                .unwrap_or("none"),
        );
        rendered.push_str(" failure_context=");
        rendered.push_str(
            orchestration_quality
                .failure_context_ref
                .as_deref()
                .unwrap_or("none"),
        );
        rendered.push_str(" outcome=");
        rendered.push_str(&orchestration_quality.outcome_summary);
    }
    if let Some(step_policy) = summary.step_policy.as_ref() {
        rendered.push_str("\n  step_policy:");
        rendered.push_str("\n  - ");
        rendered.push_str(&render_step_policy_summary(step_policy));
    }
    rendered
}

fn render_coordinator_runtime_proof(proof: &CoordinatorRuntimeProofView) -> String {
    let delegation_summary = if proof.delegation_records.is_empty() {
        "none".to_string()
    } else {
        proof
            .delegation_records
            .iter()
            .map(|record| {
                format!(
                    "{} label={} role={} child={} status={} scope={}",
                    record.delegation_id,
                    record.delegated_job_label,
                    record.child_role,
                    record.subagent_id,
                    record.status,
                    record.delegated_scope_ref,
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let inbox_summary = if proof.subordinate_inbox.is_empty() {
        "none".to_string()
    } else {
        proof
            .subordinate_inbox
            .iter()
            .map(|record| {
                format!(
                    "{}#{} kind={} payload={}",
                    record.delegation_id,
                    record.event_sequence,
                    record.event_kind,
                    record.event_payload_ref,
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let state_summary = if proof.subagent_states.is_empty() {
        "none".to_string()
    } else {
        proof
            .subagent_states
            .iter()
            .map(|record| {
                format!(
                    "{} {} -> {} reason={}",
                    record.subagent_id,
                    record.previous_state.as_deref().unwrap_or("none"),
                    record.current_state,
                    record.state_reason,
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let evidence_summary = if proof.delegated_work_evidence.is_empty() {
        "none".to_string()
    } else {
        proof
            .delegated_work_evidence
            .iter()
            .map(|record| {
                format!(
                    "{} kind={} refs={} summary={}",
                    record.delegation_id,
                    record.evidence_kind,
                    if record.artifact_refs.is_empty() {
                        "none".to_string()
                    } else {
                        record.artifact_refs.join(",")
                    },
                    record.evidence_summary,
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let decision_summary = if proof.coordinator_decisions.is_empty() {
        "none".to_string()
    } else {
        proof
            .coordinator_decisions
            .iter()
            .map(|record| {
                format!(
                    "{} kind={} outcome={} reason={}",
                    record.decision_id,
                    record.decision_kind,
                    record.decision_outcome,
                    record.decision_reason,
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };

    format!(
        "proof_boundary={}\n  follow_up={}\n  delegations={}\n  inbox={}\n  states={}\n  evidence={}\n  decisions={}",
        proof.proof_boundary,
        proof.session_follow_up_note,
        delegation_summary,
        inbox_summary,
        state_summary,
        evidence_summary,
        decision_summary,
    )
}

fn render_step_policy_summary(summary: &StepPolicySummaryView) -> String {
    let reason_codes = if summary.difficulty_assessment.reason_codes.is_empty() {
        "none".to_string()
    } else {
        summary.difficulty_assessment.reason_codes.join("|")
    };
    let budget_summary = summary
        .budget_pressure
        .as_ref()
        .map(|pressure| {
            format!(
                "{}/{}/{}/{} note={}",
                step_budget_pressure_level_label(pressure.pressure_level),
                pressure.pressure_source,
                pressure.policy_hint,
                pressure.budget_root.as_deref().unwrap_or("none"),
                pressure.note.as_deref().unwrap_or("none")
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let input_refs = [
        summary
            .input_refs
            .latest_step_evaluation
            .as_deref()
            .map(|value| format!("evaluation={value}")),
        summary
            .input_refs
            .latest_step_routing
            .as_deref()
            .map(|value| format!("routing={value}")),
        summary
            .input_refs
            .supervision_evidence
            .as_deref()
            .map(|value| format!("supervision={value}")),
        summary
            .input_refs
            .runtime_truth
            .as_deref()
            .map(|value| format!("runtime_truth={value}")),
        summary
            .input_refs
            .boundary_evidence
            .as_deref()
            .map(|value| format!("boundary={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let input_refs = if input_refs.is_empty() {
        "none".to_string()
    } else {
        input_refs.join(" | ")
    };

    format!(
        "step={} difficulty={} confidence={} reasons={} action={} action_reason={} budget={} difficulty_ref={} budget_ref={} repair_lineage={} operator_attention={} proof_owner={} task_proof={} note={} inputs={}",
        summary.difficulty_assessment.step_id,
        step_difficulty_bucket_label(summary.difficulty_assessment.difficulty_bucket),
        step_policy_confidence_label(summary.difficulty_assessment.confidence_label),
        reason_codes,
        step_policy_action_label(summary.policy_decision.chosen_action),
        summary.policy_decision.action_reason,
        budget_summary,
        summary
            .policy_decision
            .difficulty_ref
            .as_deref()
            .unwrap_or("none"),
        summary
            .policy_decision
            .budget_ref
            .as_deref()
            .unwrap_or("none"),
        summary
            .policy_decision
            .repair_lineage_ref
            .as_deref()
            .unwrap_or("none"),
        summary.policy_decision.requires_operator_attention,
        summary.proof_boundary_ref.owner_packet,
        summary.proof_boundary_ref.task_proof,
        summary.display_note,
        input_refs
    )
}

fn step_difficulty_bucket_label(bucket: mister_smith_core::StepDifficultyBucket) -> &'static str {
    match bucket {
        mister_smith_core::StepDifficultyBucket::Low => "low",
        mister_smith_core::StepDifficultyBucket::Moderate => "moderate",
        mister_smith_core::StepDifficultyBucket::High => "high",
        mister_smith_core::StepDifficultyBucket::Critical => "critical",
    }
}

fn step_policy_confidence_label(
    label: mister_smith_core::StepPolicyConfidenceLabel,
) -> &'static str {
    match label {
        mister_smith_core::StepPolicyConfidenceLabel::LowConfidence => "low_confidence",
        mister_smith_core::StepPolicyConfidenceLabel::ModerateConfidence => "moderate_confidence",
        mister_smith_core::StepPolicyConfidenceLabel::Deterministic => "deterministic",
    }
}

fn step_policy_action_label(action: mister_smith_core::StepPolicyAction) -> &'static str {
    match action {
        mister_smith_core::StepPolicyAction::Keep => "keep",
        mister_smith_core::StepPolicyAction::Retry => "retry",
        mister_smith_core::StepPolicyAction::Clarify => "clarify",
        mister_smith_core::StepPolicyAction::Downgrade => "downgrade",
        mister_smith_core::StepPolicyAction::Escalate => "escalate",
    }
}

fn step_budget_pressure_level_label(
    level: mister_smith_core::StepBudgetPressureLevel,
) -> &'static str {
    match level {
        mister_smith_core::StepBudgetPressureLevel::None => "none",
        mister_smith_core::StepBudgetPressureLevel::Watch => "watch",
        mister_smith_core::StepBudgetPressureLevel::Softcap => "softcap",
        mister_smith_core::StepBudgetPressureLevel::HardStop => "hard_stop",
    }
}

fn render_supervision_evidence(summary: &SupervisionEvidenceView) -> String {
    let target_kind = match summary.target_scope.kind {
        SupervisionTargetKind::Provider => "provider",
        SupervisionTargetKind::Graph => "graph",
        SupervisionTargetKind::Branch => "branch",
        SupervisionTargetKind::Node => "node",
    };
    let fingerprint = summary
        .fingerprint_ref
        .as_ref()
        .map(|reference| {
            format!(
                "{} confidence={:.2}",
                reference.fingerprint_key, reference.confidence
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let repair_lineage = summary
        .repair_lineage_ref
        .as_ref()
        .map(|lineage| {
            format!(
                "{} checkpoint={}",
                lineage.source,
                lineage.checkpoint_ref.as_deref().unwrap_or("none")
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let mut rendered = format!(
        "target={} provider={} graph={} branch={} node={} basis={} fingerprint={} repair_lineage={} proof_boundary={}",
        target_kind,
        summary.target_scope.provider.as_deref().unwrap_or("none"),
        summary
            .target_scope
            .graph_id
            .map(|graph_id| graph_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        summary
            .target_scope
            .branch_id
            .map(|branch_id| branch_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        summary
            .target_scope
            .node_id
            .map(|node_id| node_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        summary.decision_basis.as_deref().unwrap_or("none"),
        fingerprint,
        repair_lineage,
        summary.proof_boundary.as_deref().unwrap_or("none"),
    );

    if let Some(profile) = summary.profile_snapshot.as_ref() {
        let signals = if profile.semantic_signals.is_empty() {
            "none".to_string()
        } else {
            profile
                .semantic_signals
                .iter()
                .map(|signal| {
                    let kind = match signal.signal_kind {
                        mister_smith_core::SemanticSignalKind::Stalled => "stalled",
                        mister_smith_core::SemanticSignalKind::Repetitive => "repetitive",
                        mister_smith_core::SemanticSignalKind::LowConfidence => "low_confidence",
                        mister_smith_core::SemanticSignalKind::MissingContext => "missing_context",
                        mister_smith_core::SemanticSignalKind::PolicyConflict => "policy_conflict",
                    };
                    format!("{kind}:{}:{}", signal.severity, signal.detail)
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };
        rendered.push_str("\n  profile:");
        rendered.push_str("\n  - health=");
        rendered.push_str(&format!("{:?}", profile.health_state));
        rendered.push_str(" signals=");
        rendered.push_str(&signals);
    }

    if let Some(decision) = summary.guard_decision.as_ref() {
        let notes = if decision.evidence.notes.is_empty() {
            "none".to_string()
        } else {
            decision.evidence.notes.join(" | ")
        };
        rendered.push_str("\n  decision:");
        rendered.push_str("\n  - failure=");
        rendered.push_str(&format!("{:?}", decision.failure_class));
        rendered.push_str(" intervention=");
        rendered.push_str(&format!("{:?}", decision.intervention));
        rendered.push_str(" basis=");
        rendered.push_str(decision.evidence.decision_basis.as_str());
        rendered.push_str(" notes=");
        rendered.push_str(&notes);
    }

    if let Some(record) = summary.intervention_record.as_ref() {
        rendered.push_str("\n  intervention:");
        rendered.push_str("\n  - rationale=");
        rendered.push_str(&record.rationale);
    }

    rendered
}

pub(crate) fn render_runtime_truth_summary(summary: &Value) -> String {
    let relationships = summary
        .get("run_trace")
        .and_then(|value| value.get("relationships"))
        .and_then(Value::as_array)
        .filter(|relationships| !relationships.is_empty())
        .map(|relationships| {
            relationships
                .iter()
                .map(|kind| {
                    kind.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| kind.to_string().trim_matches('"').to_string())
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_else(|| "none".to_string());
    let grounded_evidence = summary
        .get("grounded_evidence")
        .and_then(Value::as_array)
        .filter(|references| !references.is_empty())
        .map(|references| {
            references
                .iter()
                .map(|reference| {
                    format!(
                        "{}:{}",
                        reference
                            .get("source")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        reference
                            .get("reference")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_else(|| "none".to_string());

    format!(
        "class={} trace_root={} graph={} branch={} node={} relationships={} graph_execution={} semantic_completion={} grounded_tool_execution={} task_proof={} grounded_evidence={}",
        summary
            .get("evidence_class")
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("run_trace")
            .and_then(|value| value.get("trace_root_id"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("run_trace")
            .and_then(|value| value.get("graph_id"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("run_trace")
            .and_then(|value| value.get("branch_id"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("run_trace")
            .and_then(|value| value.get("node_id"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        relationships,
        summary
            .get("proof_boundary")
            .and_then(|value| value.get("graph_execution"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("proof_boundary")
            .and_then(|value| value.get("semantic_completion"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("proof_boundary")
            .and_then(|value| value.get("grounded_tool_execution"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        summary
            .get("proof_boundary")
            .and_then(|value| value.get("task_proof"))
            .and_then(Value::as_str)
            .unwrap_or("none"),
        grounded_evidence,
    )
}

fn render_external_capability_decision(summary: &ExternalCapabilityDecisionSummary) -> String {
    let outcome = match summary.outcome {
        ExternalCapabilityDecisionOutcome::Allowed => "allowed",
        ExternalCapabilityDecisionOutcome::Rejected => "rejected",
    };
    let boundary_surface = summary
        .boundary_surface
        .map(|surface| match surface {
            ExternalCapabilityDecisionSurface::ToolBus => "tool_bus",
            ExternalCapabilityDecisionSurface::TaskIngress => "task_ingress",
        })
        .unwrap_or("unknown");
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
    let attestation_source = summary
        .attestation_source
        .map(|source| match source {
            AttestationSource::RuntimeVerified => "runtime_verified",
            AttestationSource::MetadataContinuity => "metadata_continuity",
        })
        .unwrap_or("none");
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
        "{} surface={} branch={} observed_at={} outcome={} capability_descriptor={} action_descriptor={} action_id={} title={} scope={} required_scope={} source={} state={} depth={} policy={} resource_id={} rationale={}",
        capability_id,
        boundary_surface,
        branch_id,
        observed_at,
        outcome,
        capability_descriptor,
        action_descriptor,
        action_id,
        action_title,
        scope,
        required_scope,
        attestation_source,
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
