//! Quarantine inspection primitives for cross-boundary payloads.
//!
//! This module layers a transport-agnostic quarantine decision on top of the
//! existing [`StateValidator`](crate::state_validator::StateValidator)
//! pipeline so callers can distinguish between payloads that may be forwarded,
//! sanitized, rejected, or fully quarantined for audit analysis.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state_validator::{StateValidator, TaintLabel, ValidationError};

/// Decision emitted by quarantine inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum QuarantineAction {
    /// Forward the payload unchanged.
    Pass,
    /// Forward a sanitized payload.
    Sanitize,
    /// Block the payload and return an error to the caller.
    Reject,
    /// Isolate the payload for investigation and block it.
    Quarantine,
}

impl QuarantineAction {
    /// Return the canonical audit string for this action.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Sanitize => "Sanitize",
            Self::Reject => "Reject",
            Self::Quarantine => "Quarantine",
        }
    }
}

/// Direction of a shared-state transfer inspected by a quarantine actor.
#[cfg(feature = "audit")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedStateAccess {
    /// Agent writes data into shared state.
    Write,
    /// Agent reads data from shared state.
    Read,
}

#[cfg(feature = "audit")]
impl SharedStateAccess {
    /// Return the canonical string for this shared-state access direction.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Read => "read",
        }
    }
}

/// Result of inspecting a payload through the quarantine pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct QuarantineInspection {
    /// Decision emitted for the payload.
    pub action: QuarantineAction,
    /// Final taint label associated with the payload.
    pub taint_label: TaintLabel,
    /// Payload safe to forward when inspection allows it.
    pub forwarded_payload: Option<Value>,
    /// Schema version used for successful validation, when available.
    pub schema_version: Option<String>,
    /// Human-readable explanation for non-pass outcomes.
    pub reason: Option<String>,
    /// Malicious pattern captured during quarantine, when applicable.
    pub detected_pattern: Option<String>,
    /// Whether the payload should remain under heightened monitoring.
    pub monitored: bool,
}

/// Payload that passed quarantine inspection and may be forwarded.
#[cfg(feature = "audit")]
#[derive(Debug, Clone, PartialEq)]
pub struct QuarantineTransfer {
    /// Quarantine action that produced this forwardable payload.
    pub action: QuarantineAction,
    /// Final taint label associated with the payload.
    pub taint_label: TaintLabel,
    /// Payload to forward across the protected boundary.
    pub payload: Value,
    /// Schema version used for validation.
    pub schema_version: Option<String>,
    /// Whether downstream systems should keep monitoring the transfer.
    pub monitored: bool,
}

/// Audit metadata describing the boundary protected by quarantine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineAuditContext {
    /// Logical boundary class, for example `cross_boundary` or `shared_state`.
    pub boundary: String,
    /// Source side of the transfer.
    pub source: String,
    /// Target side of the transfer.
    pub target: String,
    /// Resource crossing the boundary, such as a NATS subject or state key.
    pub resource: String,
    /// Validator schema reference used to inspect the payload.
    pub state_type: String,
}

impl QuarantineAuditContext {
    /// Construct audit context for a quarantine-protected transfer.
    #[must_use]
    pub fn new(
        boundary: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        resource: impl Into<String>,
        state_type: impl Into<String>,
    ) -> Self {
        Self {
            boundary: boundary.into(),
            source: source.into(),
            target: target.into(),
            resource: resource.into(),
            state_type: state_type.into(),
        }
    }
}

/// Inspect a payload with the configured validator and map its taint outcome to
/// a quarantine action.
#[must_use]
pub fn inspect_quarantine_payload(
    validator: &dyn StateValidator,
    state_type: &str,
    payload: &Value,
) -> QuarantineInspection {
    match validator.validate(state_type, payload) {
        Ok(validated) => {
            let (action, monitored) = match validated.taint_label {
                TaintLabel::Clean => (QuarantineAction::Pass, false),
                TaintLabel::Sanitized => (QuarantineAction::Sanitize, true),
                TaintLabel::Suspicious => (QuarantineAction::Pass, true),
                TaintLabel::Rejected => (QuarantineAction::Reject, false),
            };

            QuarantineInspection {
                action,
                taint_label: validated.taint_label,
                forwarded_payload: matches!(
                    action,
                    QuarantineAction::Pass | QuarantineAction::Sanitize
                )
                .then_some(validated.data),
                schema_version: Some(validated.schema_version),
                reason: None,
                detected_pattern: None,
                monitored,
            }
        }
        Err(error) => inspection_from_error(error),
    }
}

fn inspection_from_error(error: ValidationError) -> QuarantineInspection {
    match error {
        ValidationError::MaliciousPattern { pattern, path } => QuarantineInspection {
            action: QuarantineAction::Quarantine,
            taint_label: TaintLabel::Rejected,
            forwarded_payload: None,
            schema_version: None,
            reason: Some(format!(
                "malicious pattern '{pattern}' detected at '{path}'"
            )),
            detected_pattern: Some(pattern),
            monitored: true,
        },
        other => QuarantineInspection {
            action: QuarantineAction::Reject,
            taint_label: other.taint_label(),
            forwarded_payload: None,
            schema_version: None,
            reason: Some(other.to_string()),
            detected_pattern: None,
            monitored: false,
        },
    }
}

#[cfg(feature = "audit")]
use crate::audit::{AuditEventType, AuditLogger, AuditOutcome, SecurityAuditEvent};

/// Separate boundary inspector used by sandboxed agent crossings.
#[cfg(feature = "audit")]
#[derive(Clone)]
pub struct QuarantineActor {
    validator: Arc<dyn StateValidator>,
    audit_logger: Arc<AuditLogger>,
}

#[cfg(feature = "audit")]
impl QuarantineActor {
    /// Create a new quarantine actor.
    #[must_use]
    pub fn new(validator: Arc<dyn StateValidator>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            validator,
            audit_logger,
        }
    }

    /// Inspect a persistent/ephemeral crossing and return the forwardable
    /// payload when the transfer is allowed.
    pub fn inspect_cross_boundary_transfer(
        &self,
        principal: Option<&str>,
        source_account: &str,
        target_account: &str,
        subject: &str,
        state_type: &str,
        payload: &Value,
    ) -> Result<QuarantineTransfer, crate::SecurityError> {
        let context = QuarantineAuditContext::new(
            "cross_boundary",
            source_account,
            target_account,
            subject,
            state_type,
        );
        self.inspect(principal, &context, payload)
    }

    /// Inspect an agent/shared-state transfer and return the forwardable
    /// payload when the transfer is allowed.
    pub fn inspect_shared_state_access(
        &self,
        principal: Option<&str>,
        access: SharedStateAccess,
        state_type: &str,
        state_key: &str,
        payload: &Value,
    ) -> Result<QuarantineTransfer, crate::SecurityError> {
        let (source, target) = match access {
            SharedStateAccess::Write => ("agent", "shared_state"),
            SharedStateAccess::Read => ("shared_state", "agent"),
        };
        let resource = format!("{}:{state_key}", access.as_str());
        let context =
            QuarantineAuditContext::new("shared_state", source, target, resource, state_type);
        self.inspect(principal, &context, payload)
    }

    fn inspect(
        &self,
        principal: Option<&str>,
        context: &QuarantineAuditContext,
        payload: &Value,
    ) -> Result<QuarantineTransfer, crate::SecurityError> {
        let inspection = inspect_quarantine_payload(&*self.validator, &context.state_type, payload);
        record_quarantine_audit_event(&self.audit_logger, principal, context, &inspection);

        match inspection.action {
            QuarantineAction::Pass | QuarantineAction::Sanitize => Ok(QuarantineTransfer {
                action: inspection.action,
                taint_label: inspection.taint_label,
                payload: inspection
                    .forwarded_payload
                    .expect("forwarded payload must exist for pass/sanitize decisions"),
                schema_version: inspection.schema_version,
                monitored: inspection.monitored,
            }),
            QuarantineAction::Reject | QuarantineAction::Quarantine => {
                let reason = inspection
                    .reason
                    .unwrap_or_else(|| "payload failed quarantine inspection".to_string());
                Err(crate::SecurityError::AuthorizationDenied(reason))
            }
        }
    }
}

/// Record a quarantine decision to the audit log.
#[cfg(feature = "audit")]
pub fn record_quarantine_audit_event(
    audit_logger: &AuditLogger,
    principal: Option<&str>,
    context: &QuarantineAuditContext,
    inspection: &QuarantineInspection,
) {
    let mut details = std::collections::HashMap::new();
    details.insert("boundary".to_string(), context.boundary.clone());
    details.insert("source".to_string(), context.source.clone());
    details.insert("target".to_string(), context.target.clone());
    details.insert("resource".to_string(), context.resource.clone());
    details.insert("state_type".to_string(), context.state_type.clone());
    details.insert(
        "decision".to_string(),
        inspection.action.as_str().to_string(),
    );
    details.insert(
        "taint_label".to_string(),
        taint_label_name(inspection.taint_label).to_string(),
    );
    details.insert("monitored".to_string(), inspection.monitored.to_string());

    if let Some(schema_version) = &inspection.schema_version {
        details.insert("schema_version".to_string(), schema_version.clone());
    }
    if let Some(reason) = &inspection.reason {
        details.insert("reason".to_string(), reason.clone());
    }
    if let Some(pattern) = &inspection.detected_pattern {
        details.insert("pattern".to_string(), pattern.clone());
    }

    let (event_type, outcome, action) = match inspection.action {
        QuarantineAction::Pass if inspection.monitored => (
            AuditEventType::SuspiciousActivity,
            AuditOutcome::Warning,
            "quarantine_pass_monitored",
        ),
        QuarantineAction::Pass => (
            AuditEventType::DataValidation,
            AuditOutcome::Success,
            "quarantine_pass",
        ),
        QuarantineAction::Sanitize => (
            AuditEventType::DataValidation,
            AuditOutcome::Warning,
            "quarantine_sanitize",
        ),
        QuarantineAction::Reject => (
            AuditEventType::SuspiciousActivity,
            AuditOutcome::Blocked,
            "quarantine_reject",
        ),
        QuarantineAction::Quarantine => (
            AuditEventType::SuspiciousActivity,
            AuditOutcome::Blocked,
            "quarantine_isolate",
        ),
    };

    audit_logger.record(SecurityAuditEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        event_type,
        principal: principal.map(ToOwned::to_owned),
        resource: Some(context.resource.clone()),
        action: Some(action.to_string()),
        outcome,
        details,
        delegation: None,
        source_ip: None,
        previous_hash: None,
    });
}

fn taint_label_name(taint_label: TaintLabel) -> &'static str {
    match taint_label {
        TaintLabel::Clean => "Clean",
        TaintLabel::Sanitized => "Sanitized",
        TaintLabel::Suspicious => "Suspicious",
        TaintLabel::Rejected => "Rejected",
    }
}
