//! NATS health check integration with the monitoring subsystem.

use async_trait::async_trait;
use mister_smith_monitoring::{ComponentId, HealthCheck, Status};

use crate::client::NatsTransport;

/// Health check for the NATS transport connection.
pub struct NatsHealthCheck {
    transport: NatsTransport,
    component_id: ComponentId,
}

impl NatsHealthCheck {
    /// Create a new health check for a NATS transport.
    pub fn new(transport: NatsTransport) -> Self {
        Self {
            transport,
            component_id: ComponentId::new("nats-transport"),
        }
    }
}

#[async_trait]
impl HealthCheck for NatsHealthCheck {
    async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
        let state = self.transport.connection_state().await;
        let status = match state {
            async_nats::connection::State::Connected => Status::Healthy,
            async_nats::connection::State::Pending => Status::Degraded,
            async_nats::connection::State::Disconnected => Status::Unhealthy,
        };
        Ok(status)
    }

    fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }
}
