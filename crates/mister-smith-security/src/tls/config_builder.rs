//! Builders for rustls `ServerConfig` and `ClientConfig`.
//!
//! Constructs TLS 1.3-only configurations using the `aws_lc_rs` crypto provider.
//! Supports mutual TLS (mTLS) with [`WebPkiClientVerifier`] on the server side
//! and optional client certificate presentation on the client side.

use std::sync::Arc;

use mister_smith_core::SecurityError;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::RootCertStore;

/// Build a rustls [`ServerConfig`](rustls::ServerConfig) enforcing TLS 1.3.
///
/// # Arguments
///
/// * `certs` - Server certificate chain (leaf first, then intermediates).
/// * `key` - Server private key.
/// * `ca_certs` - CA certificates for client verification (required when `mtls` is true).
/// * `mtls` - Whether to require and verify client certificates.
///
/// # Errors
///
/// Returns [`SecurityError::TlsConfigFailed`] if the provider, protocol version,
/// certificate, or verifier configuration fails.
pub fn build_server_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    ca_certs: Option<Vec<CertificateDer<'static>>>,
    mtls: bool,
) -> Result<Arc<rustls::ServerConfig>, SecurityError> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());

    let builder = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| SecurityError::TlsConfigFailed(format!("protocol version: {e}")))?;

    let config = if mtls {
        let ca_certs = ca_certs.ok_or_else(|| {
            SecurityError::TlsConfigFailed(
                "mTLS enabled but no CA certificates provided".to_string(),
            )
        })?;

        let mut root_store = RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(cert).map_err(|e| {
                SecurityError::TlsConfigFailed(format!("failed to add CA cert to root store: {e}"))
            })?;
        }

        let verifier = WebPkiClientVerifier::builder_with_provider(root_store.into(), provider)
            .build()
            .map_err(|e| {
                SecurityError::TlsConfigFailed(format!("client verifier build failed: {e}"))
            })?;

        builder
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, key)
            .map_err(|e| {
                SecurityError::TlsConfigFailed(format!("server cert/key config failed: {e}"))
            })?
    } else {
        builder
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| {
                SecurityError::TlsConfigFailed(format!("server cert/key config failed: {e}"))
            })?
    };

    Ok(Arc::new(config))
}

/// Build a rustls [`ClientConfig`](rustls::ClientConfig) enforcing TLS 1.3.
///
/// # Arguments
///
/// * `ca_certs` - CA certificates to trust for server verification.
/// * `client_cert` - Optional client certificate and key for mTLS.
///
/// # Errors
///
/// Returns [`SecurityError::TlsConfigFailed`] if the provider, root store,
/// or client certificate configuration fails.
pub fn build_client_config(
    ca_certs: Vec<CertificateDer<'static>>,
    client_cert: Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>,
) -> Result<Arc<rustls::ClientConfig>, SecurityError> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());

    let mut root_store = RootCertStore::empty();
    for cert in ca_certs {
        root_store.add(cert).map_err(|e| {
            SecurityError::TlsConfigFailed(format!("failed to add CA cert to root store: {e}"))
        })?;
    }

    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| SecurityError::TlsConfigFailed(format!("protocol version: {e}")))?
        .with_root_certificates(root_store);

    let config = if let Some((certs, key)) = client_cert {
        builder.with_client_auth_cert(certs, key).map_err(|e| {
            SecurityError::TlsConfigFailed(format!("client cert/key config failed: {e}"))
        })?
    } else {
        builder.with_no_client_auth()
    };

    Ok(Arc::new(config))
}
