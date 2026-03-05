# Contract: CertificateManager

**Module**: `mister_smith_security::tls`

## Public API

### CertificateManager

```rust
pub struct CertificateManager { /* private */ }

impl CertificateManager {
    /// Create a new CertificateManager from TLS configuration.
    pub fn new(config: &TlsConfig) -> Result<Self, SecurityError>;

    /// Build a rustls ServerConfig for accepting TLS connections.
    /// Enforces TLS 1.3 minimum. Configures mTLS if enabled.
    pub fn server_config(&self) -> Result<Arc<rustls::ServerConfig>, SecurityError>;

    /// Build a rustls ClientConfig for outgoing TLS connections.
    /// Configures client certificate if mTLS is enabled.
    pub fn client_config(&self) -> Result<Arc<rustls::ClientConfig>, SecurityError>;

    /// Generate self-signed CA + server + client certificates for dev/test.
    /// Returns paths to the generated certificate files.
    pub fn generate_dev_certificates(
        output_dir: &Path,
        server_sans: &[String],
    ) -> Result<DevCertificates, SecurityError>;

    /// Check certificate health (expiration, validity).
    /// Returns warnings for certificates nearing expiration.
    pub fn check_health(&self) -> Vec<CertificateWarning>;

    /// Reload certificates from disk. New connections use the new certs;
    /// existing connections are unaffected.
    pub fn reload(&self) -> Result<(), SecurityError>;
}
```

### DevCertificates

```rust
pub struct DevCertificates {
    pub ca_cert_path: PathBuf,
    pub ca_key_path: PathBuf,
    pub server_cert_path: PathBuf,
    pub server_key_path: PathBuf,
    pub client_cert_path: PathBuf,
    pub client_key_path: PathBuf,
}
```

### CertificateWarning

```rust
pub struct CertificateWarning {
    pub subject: String,
    pub days_until_expiry: i64,
    pub severity: WarningSeverity,
}

pub enum WarningSeverity {
    Info,      // > 30 days
    Warning,   // 7-30 days
    Critical,  // < 7 days
    Expired,   // already expired
}
```

### Error Cases

| Method | Error Condition | SecurityError Variant |
|--------|----------------|----------------------|
| `new` | Certificate file not found | `CertificateLoadFailed(String)` |
| `new` | Invalid PEM format | `CertificateLoadFailed(String)` |
| `server_config` | Certificate/key mismatch | `TlsConfigFailed(String)` |
| `client_config` | CA cert invalid | `TlsConfigFailed(String)` |
| `generate_dev_certificates` | File write failure | `CertificateGenerationFailed(String)` |
| `reload` | New cert invalid | `CertificateLoadFailed(String)` |

### Thread Safety

`CertificateManager` is `Send + Sync`. Certificate data wrapped in `ArcSwap` for zero-downtime reload.

### Test Contract

```rust
#[test] fn load_valid_certificates();
#[test] fn tls13_minimum_enforced();
#[test] fn mtls_rejects_unauthenticated_client();
#[test] fn mtls_accepts_valid_client_cert();
#[test] fn generate_dev_certificates_creates_valid_chain();
#[test] fn certificate_expiry_warning();
#[test] fn certificate_reload_succeeds();
#[test] fn server_config_produced_correctly();
#[test] fn client_config_with_mtls();
```
