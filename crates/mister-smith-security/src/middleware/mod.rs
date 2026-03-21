//! Security middleware: composition root, rate limiting, and transport-layer enforcement.
//!
//! The [`SecurityLayer`] composes all security subsystems into a single
//! shareable context used by HTTP, gRPC, and NATS middleware.

pub mod rate_limiter;

#[cfg(feature = "jwt")]
pub mod axum_mw;
#[cfg(all(feature = "jwt", feature = "rbac"))]
pub mod nats_mw;
#[cfg(feature = "jwt")]
pub mod tonic_mw;

use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "audit")]
use crate::audit::AuditLogger;
use crate::config::KeySource;
#[cfg(feature = "jwt")]
use crate::delegation::DelegationService;
#[cfg(feature = "jwt")]
use crate::jwt::{JwtManager, DEFAULT_MAX_DELEGATION_CHAIN_DEPTH};
#[cfg(feature = "rbac")]
use crate::rbac::PolicyEngine;
#[cfg(feature = "tls")]
use crate::tls::CertificateManager;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use mister_smith_config::SecurityConfig as RuntimeSecurityConfig;

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
    /// Delegation validation service for transport-bound capability checks.
    #[cfg(feature = "jwt")]
    pub delegation_service: Option<Arc<DelegationService>>,
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
        #[cfg(feature = "jwt")]
        let jwt_config = if value.enabled && value.auth.enabled {
            let algorithm = canonical_jwt_algorithm(&value.auth.algorithm);
            let key_source = match (
                value.auth.private_key_path.as_deref(),
                value.auth.public_key_path.as_deref(),
                value.auth.hmac_secret.as_ref(),
            ) {
                (Some(private_pem), Some(public_pem), _)
                    if algorithm.starts_with("RS") || algorithm.starts_with("PS") =>
                {
                    KeySource::RsaPem {
                        private_pem: private_pem.into(),
                        public_pem: public_pem.into(),
                    }
                }
                (Some(private_pem), Some(public_pem), _) if algorithm.starts_with("ES") => {
                    KeySource::EcPem {
                        private_pem: private_pem.into(),
                        public_pem: public_pem.into(),
                    }
                }
                (Some(private_pem), Some(public_pem), _) if algorithm.starts_with("ED") => {
                    KeySource::EdPem {
                        private_pem: private_pem.into(),
                        public_pem: public_pem.into(),
                    }
                }
                (_, _, Some(secret)) => KeySource::Hmac {
                    secret: decode_hmac_secret(secret),
                },
                (Some(_), Some(_), None) => {
                    tracing::warn!(
                        algorithm = %value.auth.algorithm,
                        "Ignoring JWT PEM key paths because the algorithm requires an HMAC secret"
                    );
                    config::JwtConfig::default().key_source
                }
                _ => config::JwtConfig::default().key_source,
            };

            Some(config::JwtConfig {
                algorithm,
                access_token_ttl: Duration::from_secs(value.auth.access_token_ttl_secs),
                refresh_token_ttl: Duration::from_secs(value.auth.refresh_token_ttl_secs),
                issuer: value.auth.issuer.clone(),
                audience: value.auth.audience.clone(),
                delegation_chain_max_depth: DEFAULT_MAX_DELEGATION_CHAIN_DEPTH,
                key_source,
            })
        } else {
            None
        };

        Self {
            enabled: value.enabled,
            auth_enabled: value.auth.enabled,
            authz_enabled: value.authz.enabled,
            audit_enabled: value.audit.enabled,
            tls_enabled: value.tls.enabled,
            #[cfg(feature = "jwt")]
            jwt_config,
            #[cfg(feature = "rbac")]
            rbac_config: Some(config::RbacConfig {
                default_role: value.authz.default_role.clone(),
            }),
            #[cfg(feature = "audit")]
            audit_config: Some(config::AuditConfig {
                enabled: value.audit.enabled,
                max_events: value.audit.max_events,
                auth_failure_alert_threshold: value.audit.auth_failure_alert_threshold,
            }),
            #[cfg(feature = "tls")]
            tls_config: Some(config::TlsConfig {
                enabled: value.tls.enabled,
                cert_path: value.tls.cert_path.clone().map(Into::into),
                key_path: value.tls.key_path.clone().map(Into::into),
                ca_path: value.tls.ca_path.clone().map(Into::into),
                mtls_enabled: value.tls.mtls_enabled,
                generate_self_signed: value.tls.generate_self_signed,
                reload_interval: value.tls.reload_interval_secs.map(Duration::from_secs),
                expiry_warning_days: value.tls.expiry_warning_days,
            }),
            ..Default::default()
        }
    }
}

#[cfg(feature = "jwt")]
fn canonical_jwt_algorithm(algorithm: &str) -> String {
    match algorithm.to_ascii_uppercase().as_str() {
        "EDDSA" => "EdDSA".to_string(),
        other => other.to_string(),
    }
}

#[cfg(feature = "jwt")]
fn decode_hmac_secret(secret: &str) -> Vec<u8> {
    STANDARD.decode(secret).unwrap_or_else(|error| {
        tracing::warn!(
            error = %error,
            "Failed to decode base64 JWT HMAC secret; falling back to raw secret bytes"
        );
        secret.as_bytes().to_vec()
    })
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

        #[cfg(feature = "jwt")]
        let delegation_service = if config.enabled && config.auth_enabled {
            let max_depth = config
                .jwt_config
                .as_ref()
                .map(|cfg| cfg.delegation_chain_max_depth)
                .unwrap_or(DEFAULT_MAX_DELEGATION_CHAIN_DEPTH);
            Some(Arc::new(
                DelegationService::new_with_delegation_chain_max_depth(max_depth),
            ))
        } else {
            None
        };

        #[cfg(feature = "rbac")]
        let policy = if config.enabled && config.authz_enabled {
            config
                .rbac_config
                .as_ref()
                .map(|cfg| Arc::new(PolicyEngine::new(cfg)))
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
            #[cfg(feature = "jwt")]
            delegation_service,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_security_config_conversion_populates_jwt_config() {
        let mut runtime = RuntimeSecurityConfig::default();
        runtime.enabled = true;
        runtime.auth.enabled = true;
        runtime.auth.algorithm = "HS256".to_string();
        runtime.auth.access_token_ttl_secs = 60;
        runtime.auth.refresh_token_ttl_secs = 3600;
        runtime.auth.issuer = Some("mister-smith-tests".to_string());
        runtime.auth.audience = vec!["ops".to_string(), "proof".to_string()];
        runtime.auth.hmac_secret = Some("c2hhcmVkLXNlY3JldA==".to_string());
        runtime.authz.default_role = Some("observer".to_string());

        let converted = SecurityLayerConfig::from(&runtime);
        let jwt = converted
            .jwt_config
            .as_ref()
            .expect("runtime auth config should produce jwt config");

        assert_eq!(jwt.algorithm, "HS256");
        assert_eq!(jwt.access_token_ttl, Duration::from_secs(60));
        assert_eq!(jwt.refresh_token_ttl, Duration::from_secs(3600));
        assert_eq!(jwt.issuer.as_deref(), Some("mister-smith-tests"));
        assert_eq!(jwt.audience, vec!["ops".to_string(), "proof".to_string()]);
        match &jwt.key_source {
            KeySource::Hmac { secret } => assert_eq!(secret, b"shared-secret"),
            other => panic!("expected HMAC key source, got {other:?}"),
        }
    }

    #[test]
    fn runtime_security_config_conversion_maps_ps_algorithms_to_rsa_pem() {
        let mut runtime = RuntimeSecurityConfig::default();
        runtime.enabled = true;
        runtime.auth.enabled = true;
        runtime.auth.algorithm = "PS256".to_string();
        runtime.auth.private_key_path = Some("/tmp/private.pem".to_string());
        runtime.auth.public_key_path = Some("/tmp/public.pem".to_string());

        let converted = SecurityLayerConfig::from(&runtime);
        let jwt = converted
            .jwt_config
            .as_ref()
            .expect("runtime auth config should produce jwt config");

        match &jwt.key_source {
            KeySource::RsaPem {
                private_pem,
                public_pem,
            } => {
                assert_eq!(private_pem, &std::path::PathBuf::from("/tmp/private.pem"));
                assert_eq!(public_pem, &std::path::PathBuf::from("/tmp/public.pem"));
            }
            other => panic!("expected RSA PEM key source for PS256, got {other:?}"),
        }
    }

    #[test]
    fn runtime_security_config_conversion_normalizes_jwt_algorithm() {
        let mut runtime = RuntimeSecurityConfig::default();
        runtime.enabled = true;
        runtime.auth.enabled = true;
        runtime.auth.algorithm = "hs256".to_string();
        runtime.auth.hmac_secret = Some("c2hhcmVkLXNlY3JldA==".to_string());

        let converted = SecurityLayerConfig::from(&runtime);
        let jwt = converted
            .jwt_config
            .as_ref()
            .expect("runtime auth config should produce jwt config");

        assert_eq!(jwt.algorithm, "HS256");
    }
}
