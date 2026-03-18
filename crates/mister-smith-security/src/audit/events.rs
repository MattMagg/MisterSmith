//! Security audit event types.
//!
//! Defines the [`SecurityAuditEvent`] structure and supporting enums that form
//! the data model for the tamper-evident audit log. Events are hash-chained:
//! each event stores the SHA-256 digest of the preceding entry, enabling
//! integrity verification of the full sequence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use chrono::{DateTime, Utc};

use mister_smith_core::{AuthorityPrincipal, CapabilityId, DelegationScope, RevocationState};

/// A single security audit event.
///
/// Events capture who (`principal`) did what (`action`) to which resource
/// (`resource`), the outcome, and an optional free-form `details` map.
///
/// Hash chaining is maintained by [`super::AuditLogger`]: the `previous_hash`
/// field is set automatically when the event is recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    /// Unique identifier for this event (UUID v4).
    pub event_id: String,
    /// When the event occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Category of security event.
    pub event_type: AuditEventType,
    /// The authenticated identity that triggered the event, if known.
    pub principal: Option<String>,
    /// The target resource of the action, if applicable.
    pub resource: Option<String>,
    /// Human-readable description of the action performed.
    pub action: Option<String>,
    /// Whether the action succeeded, failed, was blocked, etc.
    pub outcome: AuditOutcome,
    /// Arbitrary key-value metadata attached to the event.
    pub details: HashMap<String, String>,
    /// Delegation-specific provenance and rejection context, when applicable.
    pub delegation: Option<DelegationAuditContext>,
    /// Originating IP address, if available.
    pub source_ip: Option<String>,
    /// SHA-256 hex digest of the previous event's serialized form.
    ///
    /// `None` for the first event in the chain.
    pub previous_hash: Option<String>,
}

/// Category of security-relevant event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Login, logout, token exchange, credential validation.
    Authentication,
    /// Permission checks — role/policy evaluation results.
    Authorization,
    /// Token generation, refresh, revocation, expiration.
    TokenLifecycle,
    /// Certificate issuance, renewal, revocation, expiry warnings.
    CertificateEvent,
    /// Anomalous patterns detected (brute force, unusual access, etc.).
    SuspiciousActivity,
    /// State validation or sanitization at a persistence boundary.
    DataValidation,
    /// Access to system-level resources or admin endpoints.
    SystemAccess,
    /// Changes to security configuration (roles, policies, keys).
    ConfigurationChange,
    /// Delegation capability issuance, validation, rejection, or revocation.
    Delegation,
}

/// Outcome of a security action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    /// The action completed successfully.
    Success,
    /// The action failed (bad credentials, insufficient permissions, etc.).
    Failure,
    /// The action was proactively blocked by a security policy.
    Blocked,
    /// The action succeeded but triggered a security warning.
    Warning,
}

/// Typed delegation audit details for privileged capability decisions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationAuditContext {
    /// Capability involved in the decision, when available.
    pub capability_id: Option<CapabilityId>,
    /// Capability descriptor involved in the decision, when available.
    pub descriptor_id: Option<String>,
    /// Parent capability in the delegation chain, when available.
    pub parent_capability: Option<CapabilityId>,
    /// Authority that issued the capability.
    pub issuer: Option<AuthorityPrincipal>,
    /// Recipient agent identifier.
    pub recipient: Option<String>,
    /// Scope granted or requested by the capability.
    pub scope: Option<DelegationScope>,
    /// Effective revocation state observed during the decision.
    pub revocation_state: Option<RevocationState>,
    /// Effective expiry time observed during the decision.
    pub expires_at: Option<DateTime<Utc>>,
    /// Human-readable rejection reason for invalid chains or denials.
    pub rejection_reason: Option<String>,
}
