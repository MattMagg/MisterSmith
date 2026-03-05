//! TLS 1.3 and mTLS certificate management.
//!
//! Provides [`CertificateManager`] for loading, building, reloading, and
//! health-checking TLS certificates. Backed by [`rustls`] with the `aws_lc_rs`
//! crypto provider and [`ArcSwap`] for zero-downtime certificate hot-reload.
//!
//! # Submodules
//!
//! - [`config_builder`] — rustls `ServerConfig` / `ClientConfig` construction
//! - [`dev_certs`] — self-signed certificate generation for development

pub mod config_builder;
pub mod dev_certs;

pub use dev_certs::DevCertificates;

use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use mister_smith_core::SecurityError;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use tracing::{debug, info};

use crate::config::TlsConfig;

/// Severity of a certificate health warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    /// Certificate has more than 30 days until expiry.
    Info,
    /// Certificate expires within 7-30 days.
    Warning,
    /// Certificate expires within 7 days.
    Critical,
    /// Certificate has already expired.
    Expired,
}

/// Warning about a certificate's expiration status.
#[derive(Debug, Clone)]
pub struct CertificateWarning {
    /// Subject (CN) of the certificate.
    pub subject: String,
    /// Days remaining until expiry (negative if already expired).
    pub days_until_expiry: i64,
    /// Severity classification.
    pub severity: WarningSeverity,
}

/// Manages TLS certificates with hot-reload support.
///
/// The server configuration is held behind an [`ArcSwap`] so that
/// [`reload`](CertificateManager::reload) can atomically swap in new
/// certificates without interrupting existing connections.
///
/// Thread-safe: `Send + Sync`.
pub struct CertificateManager {
    /// Atomically-swappable server configuration for hot reload.
    server_config: ArcSwap<rustls::ServerConfig>,
    /// TLS configuration (paths, mTLS flag, etc.).
    config: TlsConfig,
}

impl CertificateManager {
    /// Create a new `CertificateManager` from the given TLS configuration.
    ///
    /// Loads certificates from the paths specified in `config`, builds an
    /// initial [`rustls::ServerConfig`], and stores it for serving.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::CertificateLoadFailed`] if certificate or key
    /// files cannot be read or parsed, or [`SecurityError::TlsConfigFailed`]
    /// if the rustls configuration cannot be built.
    pub fn new(config: &TlsConfig) -> Result<Self, SecurityError> {
        let server_cfg = Self::build_server_config_from_tls(config)?;

        info!("CertificateManager initialized (mTLS={})", config.mtls_enabled);

        Ok(Self {
            server_config: ArcSwap::from(server_cfg),
            config: config.clone(),
        })
    }

    /// Return the current server configuration.
    ///
    /// This is a cheap `Arc` clone from the underlying [`ArcSwap`]. New
    /// connections should call this each time to pick up any reloaded
    /// certificates.
    pub fn server_config(&self) -> Arc<rustls::ServerConfig> {
        self.server_config.load_full()
    }

    /// Build a [`rustls::ClientConfig`] for outgoing TLS connections.
    ///
    /// Loads the CA certificate for server verification. If mTLS is enabled,
    /// the client certificate and key are included for mutual authentication.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::CertificateLoadFailed`] if certificate files
    /// cannot be read, or [`SecurityError::TlsConfigFailed`] if the client
    /// configuration fails to build.
    pub fn client_config(&self) -> Result<Arc<rustls::ClientConfig>, SecurityError> {
        let ca_certs = self.load_ca_certs()?;

        let client_cert = if self.config.mtls_enabled {
            let certs = self.load_server_certs()?;
            let key = self.load_server_key()?;
            Some((certs, key))
        } else {
            None
        };

        config_builder::build_client_config(ca_certs, client_cert)
    }

    /// Generate self-signed development certificates.
    ///
    /// Delegates to [`dev_certs::generate_dev_certificates`]. This is a static
    /// method since it does not require an existing `CertificateManager`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::CertificateGenerationFailed`] on failure.
    pub fn generate_dev_certificates(
        output_dir: &Path,
        server_sans: &[String],
    ) -> Result<DevCertificates, SecurityError> {
        dev_certs::generate_dev_certificates(output_dir, server_sans)
    }

    /// Check the health of loaded certificates.
    ///
    /// Returns a list of [`CertificateWarning`]s for certificates nearing
    /// or past their expiration date.
    ///
    /// **Note:** This is currently a stub that returns an empty `Vec`. Full
    /// X.509 expiry parsing will be implemented when the `x509-parser` crate
    /// is added as a dependency.
    pub fn check_health(&self) -> Vec<CertificateWarning> {
        // Stub: parsing X.509 expiry from DER requires x509-parser.
        // The warning system will be fully wired in a follow-up.
        Vec::new()
    }

    /// Reload certificates from disk and atomically swap the server config.
    ///
    /// Existing connections continue using their current config. New
    /// connections accepted after `reload` completes will use the new
    /// certificates.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::CertificateLoadFailed`] or
    /// [`SecurityError::TlsConfigFailed`] if the new certificates are invalid.
    /// On error the previous configuration remains active.
    pub fn reload(&self) -> Result<(), SecurityError> {
        let new_config = Self::build_server_config_from_tls(&self.config)?;
        self.server_config.store(new_config);
        info!("TLS certificates reloaded successfully");
        Ok(())
    }

    // ── Private helpers ─────────────────────────────────────────────────

    /// Build a server config from TlsConfig by loading certs from disk.
    fn build_server_config_from_tls(
        config: &TlsConfig,
    ) -> Result<Arc<rustls::ServerConfig>, SecurityError> {
        let cert_path = config.cert_path.as_ref().ok_or_else(|| {
            SecurityError::CertificateLoadFailed("cert_path not configured".to_string())
        })?;
        let key_path = config.key_path.as_ref().ok_or_else(|| {
            SecurityError::CertificateLoadFailed("key_path not configured".to_string())
        })?;

        let certs = load_certs(cert_path)?;
        let key = load_key(key_path)?;

        debug!(
            cert_path = %cert_path.display(),
            key_path = %key_path.display(),
            "loaded server certificate and key"
        );

        let ca_certs = if config.mtls_enabled {
            let ca_path = config.ca_path.as_ref().ok_or_else(|| {
                SecurityError::CertificateLoadFailed(
                    "mTLS enabled but ca_path not configured".to_string(),
                )
            })?;
            Some(load_certs(ca_path)?)
        } else {
            None
        };

        config_builder::build_server_config(certs, key, ca_certs, config.mtls_enabled)
    }

    /// Load CA certificates from the configured path.
    fn load_ca_certs(&self) -> Result<Vec<CertificateDer<'static>>, SecurityError> {
        let ca_path = self.config.ca_path.as_ref().ok_or_else(|| {
            SecurityError::CertificateLoadFailed("ca_path not configured".to_string())
        })?;
        load_certs(ca_path)
    }

    /// Load server/client certificates from the configured cert_path.
    fn load_server_certs(&self) -> Result<Vec<CertificateDer<'static>>, SecurityError> {
        let cert_path = self.config.cert_path.as_ref().ok_or_else(|| {
            SecurityError::CertificateLoadFailed("cert_path not configured".to_string())
        })?;
        load_certs(cert_path)
    }

    /// Load the server/client private key from the configured key_path.
    fn load_server_key(&self) -> Result<PrivateKeyDer<'static>, SecurityError> {
        let key_path = self.config.key_path.as_ref().ok_or_else(|| {
            SecurityError::CertificateLoadFailed("key_path not configured".to_string())
        })?;
        load_key(key_path)
    }
}

// ── Free-standing PEM loaders ───────────────────────────────────────────────

/// Load PEM-encoded certificates from a file.
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, SecurityError> {
    CertificateDer::pem_file_iter(path)
        .map_err(|e| SecurityError::CertificateLoadFailed(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SecurityError::CertificateLoadFailed(e.to_string()))
}

/// Load a PEM-encoded private key from a file.
fn load_key(path: &Path) -> Result<PrivateKeyDer<'static>, SecurityError> {
    PrivateKeyDer::from_pem_file(path)
        .map_err(|e| SecurityError::CertificateLoadFailed(e.to_string()))
}
