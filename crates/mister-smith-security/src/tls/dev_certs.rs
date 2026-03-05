//! Self-signed certificate generation for development and testing.
//!
//! Generates a CA, server, and client certificate chain using [`rcgen`].
//! Certificates are written to disk as PEM files, ready for use with
//! [`CertificateManager`](super::CertificateManager).

use std::path::{Path, PathBuf};

use mister_smith_core::SecurityError;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose,
    IsCa, Issuer, KeyPair, KeyUsagePurpose, SanType,
};

/// Paths to all generated development certificate and key files.
#[derive(Debug, Clone)]
pub struct DevCertificates {
    /// Path to the CA certificate PEM.
    pub ca_cert_path: PathBuf,
    /// Path to the CA private key PEM.
    pub ca_key_path: PathBuf,
    /// Path to the server certificate PEM.
    pub server_cert_path: PathBuf,
    /// Path to the server private key PEM.
    pub server_key_path: PathBuf,
    /// Path to the client certificate PEM.
    pub client_cert_path: PathBuf,
    /// Path to the client private key PEM.
    pub client_key_path: PathBuf,
}

/// Generate a complete CA + server + client certificate chain for development.
///
/// All certificates use ECDSA P-256 keys. The CA is self-signed, and both
/// the server and client certificates are signed by the CA.
///
/// # Arguments
///
/// * `output_dir` - Directory to write PEM files into. Created if it does not exist.
/// * `server_sans` - Subject Alternative Names for the server certificate
///   (e.g., `["localhost", "127.0.0.1"]`).
///
/// # Errors
///
/// Returns [`SecurityError::CertificateGenerationFailed`] if key generation,
/// certificate signing, or file I/O fails.
pub fn generate_dev_certificates(
    output_dir: &Path,
    server_sans: &[String],
) -> Result<DevCertificates, SecurityError> {
    std::fs::create_dir_all(output_dir).map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!(
            "failed to create output directory {}: {e}",
            output_dir.display()
        ))
    })?;

    // ── CA key pair and self-signed certificate ──────────────────────────

    let ca_key = KeyPair::generate().map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("CA key generation failed: {e}"))
    })?;

    let mut ca_params = CertificateParams::default();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let mut ca_dn = DistinguishedName::new();
    ca_dn.push(DnType::CommonName, "Mister Smith Dev CA");
    ca_dn.push(DnType::OrganizationName, "Mister Smith");
    ca_params.distinguished_name = ca_dn;

    // Clone CA params before consuming them — we need a reference for Issuer.
    let ca_params_for_issuer = ca_params.clone();
    let ca_cert = ca_params.self_signed(&ca_key).map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("CA self-sign failed: {e}"))
    })?;

    // Build an Issuer from the saved CA params + key for signing child certificates.
    let issuer = Issuer::from_params(&ca_params_for_issuer, &ca_key);

    // ── Server certificate ──────────────────────────────────────────────

    let server_key = KeyPair::generate().map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("server key generation failed: {e}"))
    })?;

    let mut server_params = CertificateParams::default();

    let mut server_dn = DistinguishedName::new();
    server_dn.push(DnType::CommonName, "Mister Smith Server");
    server_params.distinguished_name = server_dn;

    server_params.subject_alt_names = server_sans
        .iter()
        .map(|san| {
            // Try to parse as IP address first, fall back to DNS name.
            if let Ok(ip) = san.parse::<std::net::IpAddr>() {
                SanType::IpAddress(ip)
            } else {
                SanType::DnsName(san.clone().try_into().unwrap_or_else(|_| {
                    // Fallback: use "localhost" if the SAN is not a valid DNS name.
                    "localhost".to_string().try_into().expect("localhost is valid")
                }))
            }
        })
        .collect();

    server_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let server_cert = server_params.signed_by(&server_key, &issuer).map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("server cert signing failed: {e}"))
    })?;

    // ── Client certificate ──────────────────────────────────────────────

    let client_key = KeyPair::generate().map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("client key generation failed: {e}"))
    })?;

    let mut client_params = CertificateParams::default();

    let mut client_dn = DistinguishedName::new();
    client_dn.push(DnType::CommonName, "Mister Smith Client");
    client_params.distinguished_name = client_dn;

    client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];

    let client_cert = client_params.signed_by(&client_key, &issuer).map_err(|e| {
        SecurityError::CertificateGenerationFailed(format!("client cert signing failed: {e}"))
    })?;

    // ── Write PEM files to disk ─────────────────────────────────────────

    let paths = DevCertificates {
        ca_cert_path: output_dir.join("ca-cert.pem"),
        ca_key_path: output_dir.join("ca-key.pem"),
        server_cert_path: output_dir.join("server-cert.pem"),
        server_key_path: output_dir.join("server-key.pem"),
        client_cert_path: output_dir.join("client-cert.pem"),
        client_key_path: output_dir.join("client-key.pem"),
    };

    let write = |path: &Path, content: &str, label: &str| -> Result<(), SecurityError> {
        std::fs::write(path, content).map_err(|e| {
            SecurityError::CertificateGenerationFailed(format!(
                "failed to write {label} to {}: {e}",
                path.display()
            ))
        })
    };

    write(&paths.ca_cert_path, &ca_cert.pem(), "CA certificate")?;
    write(&paths.ca_key_path, &ca_key.serialize_pem(), "CA key")?;
    write(
        &paths.server_cert_path,
        &server_cert.pem(),
        "server certificate",
    )?;
    write(
        &paths.server_key_path,
        &server_key.serialize_pem(),
        "server key",
    )?;
    write(
        &paths.client_cert_path,
        &client_cert.pem(),
        "client certificate",
    )?;
    write(
        &paths.client_key_path,
        &client_key.serialize_pem(),
        "client key",
    )?;

    Ok(paths)
}
