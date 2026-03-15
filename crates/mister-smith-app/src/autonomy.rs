//! Operator-facing autonomy status helpers.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mister_smith_config::FrameworkConfig;
use mister_smith_core::TaskId;
use mister_smith_events::{AutonomyStatusView, EventBus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    let topology_reason = &view.topology.rationale.selected_for;
    let fallback_reason = view
        .topology
        .rationale
        .fallback_reason
        .clone()
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
    let conservative_summary = if view.conservative_reasons.is_empty() {
        "none".to_string()
    } else {
        view.conservative_reasons.join(" | ")
    };

    format!(
        "workflow: {}\ngraph: {} {:?}\ntopology: {:?} width={} rationale={}\nfallback: {}\nbranches:\n{}\ncheckpoints:\n{}\nrouting:\n{}\ninterventions:\n{}\ndelegation:\n{}\ndelegation alerts:\n{}\nconservative: {}",
        view.graph.workflow_id,
        view.graph.graph_id,
        view.graph.state,
        view.topology.topology_kind,
        view.topology.parallelism_width,
        topology_reason,
        fallback_reason,
        if branch_summary.is_empty() { "none".to_string() } else { branch_summary },
        if checkpoint_summary.is_empty() { "none".to_string() } else { checkpoint_summary },
        if routing_summary.is_empty() { "none".to_string() } else { routing_summary },
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
        conservative_summary
    )
}
