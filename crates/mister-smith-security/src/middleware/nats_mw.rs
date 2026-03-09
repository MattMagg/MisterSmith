//! NATS application-level RBAC enforcement.
//!
//! Wraps a [`Transport`] to check RBAC permissions before publish/subscribe
//! operations at the application layer.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use mister_smith_core::SecurityError;
use mister_smith_transport::{MessageEnvelope, Subscription, Transport, TransportError};
use uuid::Uuid;

#[cfg(feature = "audit")]
use crate::audit::{AuditEventType, AuditLogger, AuditOutcome, SecurityAuditEvent};
use crate::jwt::AgentClaims;
use crate::message_signer::MessageSigner;
use crate::rbac::PolicyEngine;

/// Transport wrapper that enforces RBAC permissions before delegating
/// to the underlying transport.
///
/// Checks `publish:{subject}:*` before publish/request operations and
/// `subscribe:{subject}:*` before subscribe operations.
pub struct SecureTransport<T: Transport> {
    inner: T,
    policy_engine: Option<Arc<PolicyEngine>>,
    agent_claims: AgentClaims,
    message_signer: Option<Arc<dyn MessageSigner>>,
    #[cfg(feature = "audit")]
    audit_logger: Option<Arc<AuditLogger>>,
}

impl<T: Transport> SecureTransport<T> {
    /// Create a new `SecureTransport` wrapping the given transport.
    pub fn new(inner: T, policy_engine: Option<Arc<PolicyEngine>>, claims: AgentClaims) -> Self {
        Self {
            inner,
            policy_engine,
            agent_claims: claims,
            message_signer: None,
            #[cfg(feature = "audit")]
            audit_logger: None,
        }
    }

    /// Enable envelope signing and verification for this transport wrapper.
    pub fn with_message_signer(mut self, signer: Arc<dyn MessageSigner>) -> Self {
        self.message_signer = Some(signer);
        self
    }

    /// Attach an audit logger used for rejected inbound messages.
    #[cfg(feature = "audit")]
    pub fn with_audit_logger(mut self, audit_logger: Arc<AuditLogger>) -> Self {
        self.audit_logger = Some(audit_logger);
        self
    }

    /// Check a permission, converting denial to a TransportError.
    fn check_permission(&self, action: &str, subject: &str) -> Result<(), TransportError> {
        let Some(policy_engine) = self.policy_engine.as_ref() else {
            return Ok(());
        };

        if policy_engine.check_permission(&self.agent_claims, action, subject) {
            Ok(())
        } else {
            Err(TransportError::ConnectionFailed(format!(
                "{}",
                SecurityError::InsufficientPermissions(format!("{action} on subject '{subject}'"))
            )))
        }
    }

    fn prepare_outbound_envelope(
        &self,
        mut envelope: MessageEnvelope,
    ) -> Result<MessageEnvelope, TransportError> {
        let Some(message_signer) = self.message_signer.as_ref() else {
            return Ok(envelope);
        };

        if envelope.nonce.is_none() {
            envelope.nonce = Some(message_signer.generate_nonce());
        }

        envelope.signature = Some(message_signer.sign(&envelope)?);
        Ok(envelope)
    }

    fn validate_inbound_envelope(
        &self,
        subject: &str,
        envelope: &MessageEnvelope,
    ) -> Result<(), TransportError> {
        let Some(message_signer) = self.message_signer.as_ref() else {
            return Ok(());
        };

        if let Err(error) = message_signer.validate_envelope(envelope) {
            record_validation_failure(
                #[cfg(feature = "audit")]
                self.audit_logger.as_ref(),
                &self.agent_claims.agent_id,
                subject,
                envelope,
                &error,
            );
            return Err(error.into());
        }

        Ok(())
    }

    /// Wrap a raw subscription stream with inbound envelope validation.
    ///
    /// Messages that fail signature or nonce checks are silently dropped
    /// (and optionally audit-logged).  This helper is shared by both
    /// [`subscribe`](Transport::subscribe) and
    /// [`queue_subscribe`](Transport::queue_subscribe).
    fn wrap_inbound_subscription(&self, inner: Subscription, subject: &str) -> Subscription {
        let message_signer = self.message_signer.clone();
        #[cfg(feature = "audit")]
        let audit_logger = self.audit_logger.clone();
        let agent_id = self.agent_claims.agent_id.clone();
        let subject_name = subject.to_string();

        let mut stream = inner.into_stream();
        let filtered = async_stream::stream! {
            while let Some(message) = stream.next().await {
                if let Some(message_signer) = message_signer.as_ref() {
                    if let Err(error) = message_signer.validate_envelope(&message.envelope) {
                        record_validation_failure(
                            #[cfg(feature = "audit")]
                            audit_logger.as_ref(),
                            &agent_id,
                            &subject_name,
                            &message.envelope,
                            &error,
                        );
                        continue;
                    }
                }

                yield message;
            }
        };

        Subscription::new(filtered)
    }
}

#[async_trait]
impl<T: Transport> Transport for SecureTransport<T> {
    async fn publish(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        self.check_permission("publish", subject)?;
        let envelope = self.prepare_outbound_envelope(envelope)?;
        self.inner.publish(subject, envelope).await
    }

    async fn subscribe(&self, subject: &str) -> Result<Subscription, TransportError> {
        self.check_permission("subscribe", subject)?;
        let inner = self.inner.subscribe(subject).await?;
        Ok(self.wrap_inbound_subscription(inner, subject))
    }

    async fn queue_subscribe(
        &self,
        subject: &str,
        queue: &str,
    ) -> Result<Subscription, TransportError> {
        self.check_permission("subscribe", subject)?;
        let inner = self.inner.queue_subscribe(subject, queue).await?;
        Ok(self.wrap_inbound_subscription(inner, subject))
    }

    async fn request(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
        timeout: Duration,
    ) -> Result<MessageEnvelope, TransportError> {
        self.check_permission("publish", subject)?;
        let mut envelope = envelope;
        if self.message_signer.is_some() && envelope.correlation_id.is_none() {
            envelope.correlation_id = Some(Uuid::new_v4());
        }

        let envelope = self.prepare_outbound_envelope(envelope)?;
        let response = self.inner.request(subject, envelope, timeout).await?;
        self.validate_inbound_envelope(subject, &response)?;
        Ok(response)
    }
}

#[cfg(feature = "audit")]
fn record_validation_failure(
    audit_logger: Option<&Arc<AuditLogger>>,
    agent_id: &str,
    subject: &str,
    envelope: &MessageEnvelope,
    error: &SecurityError,
) {
    let Some(audit_logger) = audit_logger else {
        return;
    };

    let mut details = HashMap::new();
    details.insert("subject".to_string(), subject.to_string());
    details.insert("message_id".to_string(), envelope.message_id.to_string());
    details.insert("message_type".to_string(), envelope.message_type.clone());
    details.insert("reason".to_string(), error.to_string());

    audit_logger.record(SecurityAuditEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        event_type: AuditEventType::SuspiciousActivity,
        principal: Some(agent_id.to_string()),
        resource: Some(subject.to_string()),
        action: Some("message_validation".to_string()),
        outcome: AuditOutcome::Blocked,
        details,
        source_ip: None,
        previous_hash: None,
    });
}

#[cfg(not(feature = "audit"))]
fn record_validation_failure(
    agent_id: &str,
    subject: &str,
    envelope: &MessageEnvelope,
    error: &SecurityError,
) {
    let _ = (agent_id, subject, envelope, error);
}
