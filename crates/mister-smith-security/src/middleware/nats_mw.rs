//! NATS application-level RBAC enforcement.
//!
//! Wraps a [`Transport`] to check RBAC permissions before publish/subscribe
//! operations at the application layer.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mister_smith_core::SecurityError;
use mister_smith_transport::{MessageEnvelope, Subscription, Transport, TransportError};

use crate::jwt::AgentClaims;
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
}

impl<T: Transport> SecureTransport<T> {
    /// Create a new `SecureTransport` wrapping the given transport.
    pub fn new(inner: T, policy_engine: Option<Arc<PolicyEngine>>, claims: AgentClaims) -> Self {
        Self {
            inner,
            policy_engine,
            agent_claims: claims,
        }
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
}

#[async_trait]
impl<T: Transport> Transport for SecureTransport<T> {
    async fn publish(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        self.check_permission("publish", subject)?;
        self.inner.publish(subject, envelope).await
    }

    async fn subscribe(&self, subject: &str) -> Result<Subscription, TransportError> {
        self.check_permission("subscribe", subject)?;
        self.inner.subscribe(subject).await
    }

    async fn queue_subscribe(
        &self,
        subject: &str,
        queue: &str,
    ) -> Result<Subscription, TransportError> {
        self.check_permission("subscribe", subject)?;
        self.inner.queue_subscribe(subject, queue).await
    }

    async fn request(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
        timeout: Duration,
    ) -> Result<MessageEnvelope, TransportError> {
        self.check_permission("publish", subject)?;
        self.inner.request(subject, envelope, timeout).await
    }
}
