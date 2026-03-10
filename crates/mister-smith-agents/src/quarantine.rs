//! Quarantine inspection actor for cross-boundary agent transfers.
//!
//! The actor stays outside the protected runtimes and applies the security
//! crate's quarantine pipeline before forwarding payloads across persistent /
//! ephemeral and shared-state boundaries.

use std::sync::Arc;

use serde_json::Value;

use mister_smith_core::SecurityError;
use mister_smith_security::audit::AuditLogger;
use mister_smith_security::{
    inspect_quarantine_payload, record_quarantine_audit_event, QuarantineAction,
    QuarantineAuditContext, StateValidator,
};

/// Direction of a shared-state transfer inspected by a quarantine actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedStateAccess {
    /// Agent writes data into shared state.
    Write,
    /// Agent reads data from shared state.
    Read,
}

impl SharedStateAccess {
    fn as_str(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Read => "read",
        }
    }
}

/// Payload that passed quarantine inspection and may be forwarded.
#[derive(Debug, Clone, PartialEq)]
pub struct QuarantineTransfer {
    /// Quarantine action that produced this forwardable payload.
    pub action: QuarantineAction,
    /// Payload to forward across the protected boundary.
    pub payload: Value,
    /// Whether downstream systems should keep monitoring the transfer.
    pub monitored: bool,
}

/// Separate boundary inspector used by sandboxed agent crossings.
#[derive(Clone)]
pub struct QuarantineActor {
    validator: Arc<dyn StateValidator>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl QuarantineActor {
    /// Create a new quarantine actor.
    #[must_use]
    pub fn new(validator: Arc<dyn StateValidator>, audit_logger: Option<Arc<AuditLogger>>) -> Self {
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
    ) -> Result<QuarantineTransfer, SecurityError> {
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
    ) -> Result<QuarantineTransfer, SecurityError> {
        let (source, target) = match access {
            SharedStateAccess::Write => ("agent", "shared_state"),
            SharedStateAccess::Read => ("shared_state", "agent"),
        };
        let resource = format!("{}:{state_key}", access.as_str());
        let context = QuarantineAuditContext::new("shared_state", source, target, resource, state_type);
        self.inspect(principal, &context, payload)
    }

    fn inspect(
        &self,
        principal: Option<&str>,
        context: &QuarantineAuditContext,
        payload: &Value,
    ) -> Result<QuarantineTransfer, SecurityError> {
        let inspection = inspect_quarantine_payload(&*self.validator, &context.state_type, payload);
        if let Some(audit_logger) = &self.audit_logger {
            record_quarantine_audit_event(audit_logger, principal, context, &inspection);
        }

        match inspection.action {
            QuarantineAction::Pass | QuarantineAction::Sanitize => Ok(QuarantineTransfer {
                action: inspection.action,
                payload: inspection
                    .forwarded_payload
                    .expect("forwarded payload must exist for pass/sanitize decisions"),
                monitored: inspection.monitored,
            }),
            QuarantineAction::Reject | QuarantineAction::Quarantine => {
                let reason = inspection
                    .reason
                    .unwrap_or_else(|| "payload failed quarantine inspection".to_string());
                Err(SecurityError::AuthorizationDenied(reason))
            }
            _ => Err(SecurityError::AuthorizationDenied(
                "payload failed quarantine inspection with an unknown quarantine action".to_string(),
            )),
        }
    }
}
