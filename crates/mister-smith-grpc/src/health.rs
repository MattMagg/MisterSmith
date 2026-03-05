//! gRPC health checking via the standard `grpc.health.v1.Health` protocol.
//!
//! Wraps `tonic-health` to provide a health reporter and health server pair
//! that can be composed into the gRPC server. The reporter is returned so
//! callers can update service health status at runtime.

use tonic_health::pb::health_server::HealthServer;
use tonic_health::server::HealthReporter;
use tonic_health::ServingStatus;

/// Service names registered in the health system.
pub mod service_names {
    /// Overall server health (empty string per gRPC health check spec).
    pub const SERVER: &str = "";
    /// Agent service health.
    pub const AGENT_SERVICE: &str = "mister_smith.v1.AgentService";
    /// System service health.
    pub const SYSTEM_SERVICE: &str = "mister_smith.v1.SystemService";
}

/// Create a health reporter and gRPC health server pair.
///
/// The returned `HealthReporter` can be used to update service health at
/// runtime. The `HealthServer` should be added to the tonic `Server` via
/// `add_service`.
///
/// All services are initially set to `Serving`.
pub async fn create_health_service() -> (HealthReporter, HealthServer<impl tonic_health::pb::health_server::Health>) {
    let (reporter, server) = tonic_health::server::health_reporter();

    // Register all known services as serving.
    reporter
        .set_service_status(service_names::AGENT_SERVICE, ServingStatus::Serving)
        .await;
    reporter
        .set_service_status(service_names::SYSTEM_SERVICE, ServingStatus::Serving)
        .await;

    (reporter, server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_service_creates_reporter_and_server() {
        let (reporter, _server) = create_health_service().await;

        // Verify we can update service status without panicking.
        reporter
            .set_service_status(service_names::AGENT_SERVICE, ServingStatus::NotServing)
            .await;
        reporter
            .set_service_status(service_names::AGENT_SERVICE, ServingStatus::Serving)
            .await;
    }

    #[tokio::test]
    async fn service_name_constants() {
        assert_eq!(service_names::SERVER, "");
        assert_eq!(
            service_names::AGENT_SERVICE,
            "mister_smith.v1.AgentService"
        );
        assert_eq!(
            service_names::SYSTEM_SERVICE,
            "mister_smith.v1.SystemService"
        );
    }
}
