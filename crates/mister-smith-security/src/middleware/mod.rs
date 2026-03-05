//! Security middleware: composition root, rate limiting, and transport-layer enforcement.
//!
//! The [`SecurityLayer`] composes all security subsystems into a single
//! shareable context used by HTTP, gRPC, and NATS middleware.

pub mod rate_limiter;

#[cfg(feature = "jwt")]
pub mod axum_mw;
#[cfg(feature = "jwt")]
pub mod tonic_mw;
#[cfg(all(feature = "jwt", feature = "rbac"))]
pub mod nats_mw;

use std::sync::Arc;

#[cfg(feature = "jwt")]
use crate::jwt::JwtManager;
#[cfg(feature = "rbac")]
use crate::rbac::PolicyEngine;
#[cfg(feature = "audit")]
use crate::audit::AuditLogger;

use crate::config;
use mister_smith_core::SecurityError;
use rate_limiter::RateLimiter;

/// Composes all security subsystems into a single shareable context.
///
/// Pass an `Arc<SecurityLayer>` to HTTP middleware, gRPC interceptors, and
/// NATS transport wrappers to enforce authentication and authorization.
pub struct SecurityLayer {
    /// JWT token manager.
    #[cfg(feature = "jwt")]
    pub jwt: Arc<JwtManager>,
    /// RBAC policy engine.
    #[cfg(feature = "rbac")]
    pub policy: Arc<PolicyEngine>,
    /// Audit logger.
    #[cfg(feature = "audit")]
    pub audit: Arc<AuditLogger>,
    /// Request rate limiter.
    pub rate_limiter: Arc<RateLimiter>,
    /// Whether security is enabled (master switch).
    enabled: bool,
}

impl SecurityLayer {
    /// Create a new `SecurityLayer` from parsed configuration.
    ///
    /// # Errors
    ///
    /// Returns `SecurityError` if JWT key loading fails.
    pub fn new(
        security_enabled: bool,
        #[cfg(feature = "jwt")] jwt_config: &config::JwtConfig,
        #[cfg(feature = "rbac")] rbac_config: &config::RbacConfig,
        #[cfg(feature = "audit")] audit_config: &config::AuditConfig,
    ) -> Result<Self, SecurityError> {
        #[cfg(feature = "jwt")]
        let jwt = Arc::new(JwtManager::new(jwt_config)?);

        #[cfg(feature = "rbac")]
        let policy = Arc::new(PolicyEngine::new(rbac_config));

        #[cfg(feature = "audit")]
        let audit = Arc::new(AuditLogger::new(audit_config));

        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));

        Ok(Self {
            #[cfg(feature = "jwt")]
            jwt,
            #[cfg(feature = "rbac")]
            policy,
            #[cfg(feature = "audit")]
            audit,
            rate_limiter,
            enabled: security_enabled,
        })
    }

    /// Check if security enforcement is enabled (master switch).
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl std::fmt::Debug for SecurityLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityLayer")
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}
