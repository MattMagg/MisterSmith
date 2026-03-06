//! gRPC server lifecycle: composition, startup, and graceful shutdown.
//!
//! The [`GrpcServer`] composes a tonic `Server` with the health service and
//! configures max message sizes from [`GrpcTransportConfig`]. It exposes a
//! `serve` method that runs until a shutdown signal is received.

use std::future::Future;

use tonic::transport::Server;
use tonic_health::server::HealthReporter;
use tracing::{error, info};

use crate::config::GrpcTransportConfig;
use crate::errors::TransportError;
use crate::health;

/// A composed gRPC server with health checking.
///
/// Holds the configuration and health reporter handle. Call [`GrpcServer::serve`]
/// to start listening for connections. The server shuts down gracefully when the
/// provided shutdown signal completes.
#[derive(Debug)]
pub struct GrpcServer {
    config: GrpcTransportConfig,
    health_reporter: Option<HealthReporter>,
    /// Optional security layer for JWT authentication.
    #[cfg(feature = "security")]
    #[allow(dead_code)]
    security: Option<std::sync::Arc<mister_smith_security::middleware::SecurityLayer>>,
}

impl GrpcServer {
    /// Create a new `GrpcServer` with the given configuration.
    #[must_use]
    pub fn new(config: GrpcTransportConfig) -> Self {
        Self {
            config,
            health_reporter: None,
            #[cfg(feature = "security")]
            security: None,
        }
    }

    /// Create a new `GrpcServer` with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(GrpcTransportConfig::default())
    }

    /// Set the security layer for gRPC authentication.
    ///
    /// When set, the [`grpc_auth_interceptor`](mister_smith_security::middleware::tonic_mw::grpc_auth_interceptor)
    /// can be applied to service builders to enforce JWT authentication
    /// on incoming requests.
    #[cfg(feature = "security")]
    pub fn with_security(
        mut self,
        security: std::sync::Arc<mister_smith_security::middleware::SecurityLayer>,
    ) -> Self {
        self.security = Some(security);
        self
    }

    /// Returns the security layer, if configured.
    #[cfg(feature = "security")]
    #[must_use]
    pub fn security(
        &self,
    ) -> Option<&std::sync::Arc<mister_smith_security::middleware::SecurityLayer>> {
        self.security.as_ref()
    }

    /// Returns a reference to the health reporter, if the server has been
    /// started. The reporter can be used to update service health status.
    #[must_use]
    pub fn health_reporter(&self) -> Option<&HealthReporter> {
        self.health_reporter.as_ref()
    }

    /// Returns the server configuration.
    #[must_use]
    pub fn config(&self) -> &GrpcTransportConfig {
        &self.config
    }

    /// Start serving gRPC requests.
    ///
    /// This method:
    /// 1. Parses the bind address from config.
    /// 2. Creates the health service with all services marked as `Serving`.
    /// 3. Composes the tonic `Server` with max message size limits.
    /// 4. Adds the health service.
    /// 5. Serves until the `shutdown_signal` future completes.
    ///
    /// # Errors
    ///
    /// Returns `TransportError::ConnectionFailed` if the address cannot be
    /// parsed or the server fails to bind.
    pub async fn serve<F>(&mut self, shutdown_signal: F) -> Result<(), TransportError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let addr = self
            .config
            .socket_addr()
            .map_err(|e| TransportError::ConnectionFailed(format!("invalid bind address: {e}")))?;

        let (reporter, health_service) = health::create_health_service().await;
        self.health_reporter = Some(reporter);

        info!(
            addr = %addr,
            max_message_size = self.config.max_message_size,
            "starting gRPC server"
        );

        let mut builder = Server::builder();

        #[cfg(feature = "security")]
        let builder = if let Some(security) =
            self.security.as_ref().filter(|layer| layer.is_enabled())
        {
            let interceptor = mister_smith_security::middleware::tonic_mw::grpc_auth_interceptor(
                std::sync::Arc::clone(security),
            );
            builder.add_service(tonic::service::interceptor::InterceptedService::new(
                health_service,
                interceptor,
            ))
        } else {
            builder.add_service(health_service)
        };

        #[cfg(not(feature = "security"))]
        let builder = builder.add_service(health_service);

        builder
            .serve_with_shutdown(addr, shutdown_signal)
            .await
            .map_err(|e| {
                error!(error = %e, "gRPC server error");
                TransportError::ConnectionFailed(format!("server error: {e}"))
            })?;

        info!("gRPC server shut down gracefully");
        Ok(())
    }

    /// Start serving with a default SIGINT/SIGTERM shutdown handler.
    ///
    /// This is a convenience wrapper around [`GrpcServer::serve`] that uses
    /// `tokio::signal::ctrl_c()` as the shutdown signal.
    ///
    /// # Errors
    ///
    /// Returns `TransportError` if the server fails to start or encounters
    /// a fatal error during operation.
    pub async fn serve_with_ctrl_c(&mut self) -> Result<(), TransportError> {
        self.serve(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C handler");
            info!("received shutdown signal");
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_default_config() {
        let server = GrpcServer::with_defaults();
        assert_eq!(server.config().bind_address, "0.0.0.0:50051");
        assert_eq!(server.config().max_message_size, 4_194_304);
        assert!(server.health_reporter().is_none());
    }

    #[test]
    fn server_custom_config() {
        let config = GrpcTransportConfig::new("127.0.0.1:9090").with_max_message_size(1024);
        let server = GrpcServer::new(config);
        assert_eq!(server.config().bind_address, "127.0.0.1:9090");
        assert_eq!(server.config().max_message_size, 1024);
    }

    #[tokio::test]
    async fn serve_with_immediate_shutdown() {
        let config = GrpcTransportConfig::new("127.0.0.1:0");
        let mut server = GrpcServer::new(config);

        // Use a channel to trigger immediate shutdown.
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let handle = tokio::spawn(async move {
            server
                .serve(async {
                    let _ = rx.await;
                })
                .await
        });

        // Small delay to let the server bind, then signal shutdown.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = tx.send(());

        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn serve_invalid_address() {
        let config = GrpcTransportConfig::new("not-a-valid-address");
        let mut server = GrpcServer::new(config);

        let result = server.serve(std::future::pending()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TransportError::ConnectionFailed(_)));
    }
}
