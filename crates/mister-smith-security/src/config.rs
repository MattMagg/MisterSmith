//! Security configuration types.
//!
//! These types provide rich, validated configuration for each security
//! subsystem. They are constructed from the serde-parsed
//! [`mister_smith_config::SecurityConfig`] at startup.

use std::path::PathBuf;
use std::time::Duration;
use ring::rand::SecureRandom;

/// JWT subsystem configuration.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Signing algorithm (RS256, ES256, HS256, etc.).
    pub algorithm: String,
    /// Access token lifetime.
    pub access_token_ttl: Duration,
    /// Refresh token lifetime.
    pub refresh_token_ttl: Duration,
    /// Token issuer claim.
    pub issuer: Option<String>,
    /// Required audience claims.
    pub audience: Vec<String>,
    /// Maximum allowed delegation-chain depth.
    pub delegation_chain_max_depth: usize,
    /// Key source for signing and verification.
    pub key_source: KeySource,
}

impl Default for JwtConfig {
    fn default() -> Self {
        let mut secret = vec![0u8; 32];
        let rng = ring::rand::SystemRandom::new();
        rng.fill(&mut secret)
            .expect("failed to generate secure random secret for default JwtConfig");

        Self {
            algorithm: "HS256".to_string(),
            access_token_ttl: Duration::from_secs(900),
            refresh_token_ttl: Duration::from_secs(86400),
            issuer: None,
            audience: Vec::new(),
            delegation_chain_max_depth: 5,
            key_source: KeySource::Hmac { secret },
        }
    }
}

/// Source of cryptographic keys for JWT signing/verification.
#[derive(Debug, Clone)]
pub enum KeySource {
    /// Symmetric HMAC secret.
    Hmac {
        /// The raw secret bytes.
        secret: Vec<u8>,
    },
    /// RSA key pair from PEM files.
    RsaPem {
        /// Path to the private key PEM.
        private_pem: PathBuf,
        /// Path to the public key PEM.
        public_pem: PathBuf,
    },
    /// ECDSA key pair from PEM files.
    EcPem {
        /// Path to the private key PEM.
        private_pem: PathBuf,
        /// Path to the public key PEM.
        public_pem: PathBuf,
    },
    /// EdDSA key pair from PEM files.
    EdPem {
        /// Path to the private key PEM.
        private_pem: PathBuf,
        /// Path to the public key PEM.
        public_pem: PathBuf,
    },
}

/// RBAC subsystem configuration.
#[derive(Debug, Clone, Default)]
pub struct RbacConfig {
    /// Default role for unauthenticated requests.
    pub default_role: Option<String>,
}

/// TLS subsystem configuration.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Whether TLS is enabled.
    pub enabled: bool,
    /// Path to the server certificate PEM file.
    pub cert_path: Option<PathBuf>,
    /// Path to the server private key PEM file.
    pub key_path: Option<PathBuf>,
    /// Path to the CA certificate PEM for client verification.
    pub ca_path: Option<PathBuf>,
    /// Whether mutual TLS is required.
    pub mtls_enabled: bool,
    /// Auto-generate self-signed certificates.
    pub generate_self_signed: bool,
    /// Certificate reload check interval.
    pub reload_interval: Option<Duration>,
    /// Days before expiry to emit warnings.
    pub expiry_warning_days: u32,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_path: None,
            mtls_enabled: false,
            generate_self_signed: false,
            reload_interval: None,
            expiry_warning_days: 30,
        }
    }
}

/// Audit logging configuration.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Whether audit logging is enabled.
    pub enabled: bool,
    /// Maximum number of events to retain in memory.
    pub max_events: usize,
    /// Auth failure threshold per source per minute before alert.
    pub auth_failure_alert_threshold: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: 10_000,
            auth_failure_alert_threshold: 5,
        }
    }
}
