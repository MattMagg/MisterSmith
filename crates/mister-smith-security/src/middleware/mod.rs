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
use mister_smith_config::SecurityConfig as RuntimeSecurityConfig;
#[cfg(feature = "tls")]
use crate::tls::CertificateManager;

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
    pub jwt: Option<Arc<JwtManager>>,
    /// RBAC policy engine.
    #[cfg(feature = "rbac")]
    pub policy: Option<Arc<PolicyEngine>>,
    /// Audit logger.
    #[cfg(feature = "audit")]
    pub audit: Option<Arc<AuditLogger>>,
    /// TLS certificate manager.
    #[cfg(feature = "tls")]
    pub tls: Option<Arc<CertificateManager>>,
    /// Request rate limiter.
    pub rate_limiter: Arc<RateLimiter>,
    /// Whether security is enabled (master switch).
    enabled: bool,
}

/// Runtime middleware construction settings, typically derived from
/// `mister_smith_config::SecurityConfig`.
#[derive(Debug, Clone, Default)]
pub struct SecurityLayerConfig {
    /// Master security switch.
    pub enabled: bool,
    /// Authentication subsystem enabled flag.
    pub auth_enabled: bool,
    /// Authorization subsystem enabled flag.
    pub authz_enabled: bool,
    /// Audit subsystem enabled flag.
    pub audit_enabled: bool,
    /// TLS subsystem enabled flag.
    pub tls_enabled: bool,
    /// JWT subsystem configuration.
    #[cfg(feature = "jwt")]
    pub jwt_config: Option<config::JwtConfig>,
    /// RBAC subsystem configuration.
    #[cfg(feature = "rbac")]
    pub rbac_config: Option<config::RbacConfig>,
    /// Audit subsystem configuration.
    #[cfg(feature = "audit")]
    pub audit_config: Option<config::AuditConfig>,
    /// TLS subsystem configuration.
    #[cfg(feature = "tls")]
    pub tls_config: Option<config::TlsConfig>,
}

impl From<&RuntimeSecurityConfig> for SecurityLayerConfig {
    fn from(value: &RuntimeSecurityConfig) -> Self {
        Self {
            enabled: value.enabled,
            auth_enabled: value.auth.enabled,
            authz_enabled: value.authz.enabled,
            audit_enabled: value.audit.enabled,
            tls_enabled: value.tls.enabled,
            ..Default::default()
        }
    }
}

impl SecurityLayer {
    /// Create a new `SecurityLayer` from parsed configuration.
    ///
    /// # Errors
    ///
    /// Returns `SecurityError` if JWT key loading fails.
    pub fn new(config: SecurityLayerConfig) -> Result<Self, SecurityError> {
        #[cfg(feature = "jwt")]
        let jwt = if config.enabled && config.auth_enabled {
            config
                .jwt_config
                .as_ref()
                .map(JwtManager::new)
                .transpose()?
                .map(Arc::new)
        } else {
            None
        };

        #[cfg(feature = "rbac")]
        let policy = if config.enabled && config.authz_enabled {
            config.rbac_config.as_ref().map(|cfg| Arc::new(PolicyEngine::new(cfg)))
        } else {
            None
        };

        #[cfg(feature = "audit")]
        let audit = if config.enabled && config.audit_enabled {
            config
                .audit_config
                .as_ref()
                .map(|cfg| Arc::new(AuditLogger::new(cfg)))
        } else {
            None
        };

        #[cfg(feature = "tls")]
        let tls = if config.enabled && config.tls_enabled {
            config
                .tls_config
                .as_ref()
                .map(CertificateManager::new)
                .transpose()?
                .map(Arc::new)
        } else {
            None
        };

        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));

        Ok(Self {
            #[cfg(feature = "jwt")]
            jwt,
            #[cfg(feature = "rbac")]
            policy,
            #[cfg(feature = "audit")]
            audit,
            #[cfg(feature = "tls")]
            tls,
            rate_limiter,
            enabled: config.enabled,
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
