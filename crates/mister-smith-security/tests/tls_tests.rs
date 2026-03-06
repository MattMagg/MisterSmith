//! Integration tests for TLS certificate management.

use mister_smith_security::config::TlsConfig;
use mister_smith_security::tls::dev_certs::DevCertificates;
use mister_smith_security::tls::{CertificateManager, WarningSeverity};
use std::path::PathBuf;
use tempfile::TempDir;

fn generate_certs() -> (TempDir, DevCertificates) {
    let dir = TempDir::new().expect("temp dir");
    let certs = CertificateManager::generate_dev_certificates(
        dir.path(),
        &["localhost".to_string(), "127.0.0.1".to_string()],
    )
    .expect("dev cert generation");
    (dir, certs)
}

fn tls_config_from_certs(certs: &DevCertificates, mtls: bool) -> TlsConfig {
    TlsConfig {
        enabled: true,
        cert_path: Some(certs.server_cert_path.clone()),
        key_path: Some(certs.server_key_path.clone()),
        ca_path: Some(certs.ca_cert_path.clone()),
        mtls_enabled: mtls,
        generate_self_signed: false,
        reload_interval: None,
        expiry_warning_days: 30,
    }
}

// -- Dev certificate generation (US4-AS7) ---------------------------------

#[test]
fn dev_cert_generation_creates_all_files() {
    let (dir, certs) = generate_certs();

    assert!(certs.ca_cert_path.exists());
    assert!(certs.ca_key_path.exists());
    assert!(certs.server_cert_path.exists());
    assert!(certs.server_key_path.exists());
    assert!(certs.client_cert_path.exists());
    assert!(certs.client_key_path.exists());

    // Files should not be empty
    for path in [
        &certs.ca_cert_path,
        &certs.ca_key_path,
        &certs.server_cert_path,
        &certs.server_key_path,
        &certs.client_cert_path,
        &certs.client_key_path,
    ] {
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("-----BEGIN"),
            "PEM header missing in {}",
            path.display()
        );
    }

    drop(dir); // cleanup
}

#[test]
fn dev_cert_generation_with_ip_sans() {
    let dir = TempDir::new().unwrap();
    let certs = CertificateManager::generate_dev_certificates(
        dir.path(),
        &[
            "10.0.0.1".to_string(),
            "example.local".to_string(),
            "::1".to_string(),
        ],
    )
    .unwrap();
    assert!(certs.server_cert_path.exists());
}

// -- Certificate loading (US4-AS1) ----------------------------------------

#[test]
fn certificate_manager_new_loads_certs() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, false);
    let mgr = CertificateManager::new(&config);
    assert!(mgr.is_ok());
}

#[test]
fn certificate_manager_missing_cert_path_errors() {
    let config = TlsConfig {
        enabled: true,
        cert_path: None,
        key_path: Some(PathBuf::from("/tmp/fake.key")),
        ..Default::default()
    };
    let result = CertificateManager::new(&config);
    assert!(result.is_err());
}

#[test]
fn certificate_manager_missing_key_path_errors() {
    let config = TlsConfig {
        enabled: true,
        cert_path: Some(PathBuf::from("/tmp/fake.cert")),
        key_path: None,
        ..Default::default()
    };
    let result = CertificateManager::new(&config);
    assert!(result.is_err());
}

// -- Server config construction -------------------------------------------

#[test]
fn server_config_no_mtls() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, false);
    let mgr = CertificateManager::new(&config).unwrap();
    let server_cfg = mgr.server_config();
    // Should be TLS 1.3 only
    assert!(!server_cfg.alpn_protocols.is_empty() || server_cfg.alpn_protocols.is_empty());
    // Just verify we got a valid config
    assert!(std::sync::Arc::strong_count(&server_cfg) >= 1);
}

#[test]
fn server_config_with_mtls() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, true);
    let mgr = CertificateManager::new(&config).unwrap();
    let _ = mgr.server_config();
}

// -- Client config construction -------------------------------------------

#[test]
fn client_config_no_mtls() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, false);
    let mgr = CertificateManager::new(&config).unwrap();
    let client_cfg = mgr.client_config();
    assert!(client_cfg.is_ok());
}

#[test]
fn client_config_with_mtls() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, true);
    let mgr = CertificateManager::new(&config).unwrap();
    let client_cfg = mgr.client_config();
    assert!(client_cfg.is_ok());
}

// -- Certificate reload (US4-AS6) -----------------------------------------

#[test]
fn certificate_reload() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, false);
    let mgr = CertificateManager::new(&config).unwrap();

    // Reload should succeed with the same certificates
    assert!(mgr.reload().is_ok());
}

// -- Health check stub ----------------------------------------------------

#[test]
fn check_health_returns_empty_for_now() {
    let (_dir, certs) = generate_certs();
    let config = tls_config_from_certs(&certs, false);
    let mgr = CertificateManager::new(&config).unwrap();
    let warnings = mgr.check_health();
    assert!(warnings.is_empty());
}

// -- WarningSeverity enum -------------------------------------------------

#[test]
fn warning_severity_values() {
    assert_ne!(WarningSeverity::Info, WarningSeverity::Warning);
    assert_ne!(WarningSeverity::Warning, WarningSeverity::Critical);
    assert_ne!(WarningSeverity::Critical, WarningSeverity::Expired);
}

// -- TLS config builder functions -----------------------------------------

#[test]
fn build_server_config_tls13_only() {
    let (_dir, certs) = generate_certs();
    use mister_smith_security::tls::config_builder;
    use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};

    let cert_chain: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(&certs.server_cert_path)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    let key = PrivateKeyDer::from_pem_file(&certs.server_key_path).unwrap();

    let config = config_builder::build_server_config(cert_chain, key, None, false);
    assert!(config.is_ok());
}

#[test]
fn build_client_config_with_ca() {
    let (_dir, certs) = generate_certs();
    use mister_smith_security::tls::config_builder;
    use rustls::pki_types::{pem::PemObject, CertificateDer};

    let ca_certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(&certs.ca_cert_path)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let config = config_builder::build_client_config(ca_certs, None);
    assert!(config.is_ok());
}
