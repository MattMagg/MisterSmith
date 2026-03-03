# Authentication Implementation Specifications

## Overview

This document provides comprehensive authentication implementation specifications for the Mister Smith AI Agent Framework,
building on the security framework foundation and integrating with transport layer protocols.

**Integration Points**:

- **Security Framework**: [`security-framework.md`](security-framework.md) - Core security patterns and configurations
- **Authentication Implementation**: [`authentication-implementation.md`](authentication-implementation.md) - Certificate + JWT implementation
- **Authorization Specifications**: [`authorization-specifications.md`](authorization-specifications.md) - RBAC and permission patterns
- **Security Integration**: [`security-integration.md`](security-integration.md) - NATS and hook security patterns
- **Transport Layer**: [`../transport/`](../transport/) - Modular transport specifications
  - [Transport Core](../transport/transport-core.md) - Core patterns and security foundations
  - [NATS Transport](../transport/nats-transport.md) - Messaging security
  - [gRPC Transport](../transport/grpc-transport.md) - RPC security with TLS/mTLS
  - [HTTP Transport](../transport/http-transport.md) - API security and authentication
- **Deployment Architecture**: [`../operations/deployment-architecture-specifications.md`](../operations/deployment-architecture-specifications.md)

## 1. JWT Token Structure and Claims

### 1.1 Token Architecture

```rust
use serde::{Deserialize, Serialize};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentClaims {
    // Standard JWT Claims (RFC 7519)
    pub iss: String,              // Issuer
    pub sub: String,              // Subject (agent_id)
    pub aud: Vec<String>,         // Audience
    pub exp: i64,                 // Expiration time
    pub nbf: i64,                 // Not before
    pub iat: i64,                 // Issued at
    pub jti: String,              // JWT ID
    
    // Custom Agent Claims
    pub agent_id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub permissions: Vec<Permission>,
    pub session_id: String,
    pub tenant_id: Option<String>,
    pub delegation_chain: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentType {
    Planner,
    Executor,
    Critic,
    Router,
    Memory,
    External,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<String>,
    pub constraints: Option<serde_json::Value>,
}
```

### 1.2 Token Generation

```rust
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    issuer: String,
    default_ttl: Duration,
}

impl JwtManager {
    pub fn new(config: &JwtConfig) -> Result<Self, AuthError> {
        let encoding_key = match &config.algorithm {
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                EncodingKey::from_rsa_pem(config.private_key.as_bytes())?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                EncodingKey::from_ec_pem(config.private_key.as_bytes())?
            }
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                EncodingKey::from_secret(config.secret.as_ref())
            }
            _ => return Err(AuthError::UnsupportedAlgorithm),
        };
        
        let decoding_key = match &config.algorithm {
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                DecodingKey::from_rsa_pem(config.public_key.as_bytes())?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                DecodingKey::from_ec_pem(config.public_key.as_bytes())?
            }
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(config.secret.as_ref())
            }
            _ => return Err(AuthError::UnsupportedAlgorithm),
        };
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_audience(&config.audience);
        validation.set_issuer(&[&config.issuer]);
        validation.leeway = config.leeway_seconds;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            issuer: config.issuer.clone(),
            default_ttl: Duration::seconds(config.default_ttl_seconds),
        })
    }
    
    pub fn generate_token(&self, agent: &Agent) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + self.default_ttl;
        
        let claims = AgentClaims {
            iss: self.issuer.clone(),
            sub: agent.id.clone(),
            aud: vec!["mister-smith".to_string()],
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            agent_id: agent.id.clone(),
            agent_type: agent.agent_type.clone(),
            capabilities: agent.capabilities.clone(),
            permissions: agent.permissions.clone(),
            session_id: agent.session_id.clone(),
            tenant_id: agent.tenant_id.clone(),
            delegation_chain: agent.delegation_chain.clone(),
        };
        
        let header = Header::new(Algorithm::RS256);
        jsonwebtoken::encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }
}
```

### 1.3 Token Validation

```rust
impl JwtManager {
    pub fn validate_token(&self, token: &str) -> Result<AgentClaims, AuthError> {
        let token_data = jsonwebtoken::decode::<AgentClaims>(
            token,
            &self.decoding_key,
            &self.validation,
        )?;
        
        // Additional validation
        self.validate_permissions(&token_data.claims)?;
        self.check_revocation(&token_data.claims.jti)?;
        self.validate_delegation_chain(&token_data.claims.delegation_chain)?;
        
        Ok(token_data.claims)
    }
    
    fn validate_permissions(&self, claims: &AgentClaims) -> Result<(), AuthError> {
        // Validate that permissions are consistent with agent type
        match claims.agent_type {
            AgentType::Executor => {
                if claims.permissions.iter().any(|p| p.resource == "system") {
                    return Err(AuthError::InsufficientPermissions);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 2. API Key Management Procedures

### 2.1 API Key Structure

```rust
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String,
    pub prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub agent_id: String,
    pub permissions: Vec<Permission>,
    pub rate_limit: RateLimit,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub rotation_version: u32,
    pub enabled: bool,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
    pub quota_reset: QuotaResetPolicy,
}

pub struct ApiKeyManager {
    repository: Arc<dyn ApiKeyRepository>,
    hasher: Arc<dyn PasswordHasher>,
    entropy_source: SystemRandom,
}

impl ApiKeyManager {
    pub async fn generate_api_key(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyResponse, AuthError> {
        // Generate 32 bytes of entropy
        let mut key_bytes = [0u8; 32];
        self.entropy_source
            .fill(&mut key_bytes)
            .map_err(|_| AuthError::EntropyGeneration)?;
        
        // Create readable key format: prefix_base64key
        let prefix = format!("sk_{}", &request.agent_id[..8]);
        let key = general_purpose::URL_SAFE_NO_PAD.encode(&key_bytes);
        let full_key = format!("{}_{}", prefix, key);
        
        // Hash the key for storage
        let key_hash = self.hasher.hash(&full_key)?;
        
        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            key_hash,
            prefix: prefix.clone(),
            name: request.name,
            description: request.description,
            agent_id: request.agent_id,
            permissions: request.permissions,
            rate_limit: request.rate_limit.unwrap_or_default(),
            created_at: Utc::now(),
            expires_at: request.expires_at,
            last_used_at: None,
            rotation_version: 1,
            enabled: true,
            tags: request.tags,
        };
        
        self.repository.save(&api_key).await?;
        
        Ok(ApiKeyResponse {
            id: api_key.id,
            key: full_key, // Only returned once
            prefix,
            created_at: api_key.created_at,
        })
    }
}
```

### 2.2 Key Rotation Procedures

```rust
impl ApiKeyManager {
    pub async fn rotate_api_key(
        &self,
        key_id: &str,
        grace_period: Duration,
    ) -> Result<RotateKeyResponse, AuthError> {
        let existing_key = self.repository.get(key_id).await?;
        
        // Generate new key
        let new_key_response = self.generate_api_key(CreateApiKeyRequest {
            agent_id: existing_key.agent_id.clone(),
            name: format!("{} (rotated)", existing_key.name),
            permissions: existing_key.permissions.clone(),
            rate_limit: Some(existing_key.rate_limit.clone()),
            ..Default::default()
        }).await?;
        
        // Schedule old key for deletion after grace period
        self.repository
            .schedule_deletion(key_id, Utc::now() + grace_period)
            .await?;
        
        // Notify dependent systems
        self.notify_key_rotation(&existing_key, &new_key_response).await?;
        
        Ok(RotateKeyResponse {
            old_key_id: key_id.to_string(),
            new_key_id: new_key_response.id,
            new_key: new_key_response.key,
            grace_period_ends: Utc::now() + grace_period,
        })
    }
}
```

## 3. Certificate Authority Setup

### 3.1 Root CA Configuration

```rust
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair};
use x509_parser::prelude::*;

pub struct CertificateAuthority {
    root_cert: Certificate,
    intermediate_cert: Certificate,
    crl_endpoint: String,
    ocsp_endpoint: String,
}

impl CertificateAuthority {
    pub fn initialize(config: &CAConfig) -> Result<Self, CAError> {
        // Generate Root CA
        let mut root_params = CertificateParams::default();
        root_params.not_before = time::OffsetDateTime::now_utc();
        root_params.not_after = root_params.not_before + time::Duration::days(3650); // 10 years
        
        let mut root_dn = DistinguishedName::new();
        root_dn.push(DnType::CountryName, "US");
        root_dn.push(DnType::OrganizationName, "Mister Smith Framework");
        root_dn.push(DnType::CommonName, "Mister Smith Root CA");
        root_params.distinguished_name = root_dn;
        
        root_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        root_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];
        
        let root_key = KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384)?;
        let root_cert = root_params.self_signed(&root_key)?;
        
        // Generate Intermediate CA
        let mut int_params = CertificateParams::default();
        int_params.not_before = time::OffsetDateTime::now_utc();
        int_params.not_after = int_params.not_before + time::Duration::days(1825); // 5 years
        
        let mut int_dn = DistinguishedName::new();
        int_dn.push(DnType::CountryName, "US");
        int_dn.push(DnType::OrganizationName, "Mister Smith Framework");
        int_dn.push(DnType::CommonName, "Mister Smith Intermediate CA");
        int_params.distinguished_name = int_dn;
        
        int_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Constrained(0));
        int_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];
        
        let int_key = KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384)?;
        let int_cert = int_params.signed_by(&int_key, &root_cert, &root_key)?;
        
        Ok(Self {
            root_cert,
            intermediate_cert: Certificate::from_params(int_params)?,
            crl_endpoint: config.crl_endpoint.clone(),
            ocsp_endpoint: config.ocsp_endpoint.clone(),
        })
    }
}
```

### 3.2 Certificate Lifecycle Management

```rust
pub struct CertificateManager {
    ca: Arc<CertificateAuthority>,
    repository: Arc<dyn CertificateRepository>,
    revocation_list: Arc<RwLock<CertificateRevocationList>>,
}

impl CertificateManager {
    pub async fn issue_agent_certificate(
        &self,
        request: CertificateRequest,
    ) -> Result<IssuedCertificate, CAError> {
        // Validate request
        self.validate_certificate_request(&request)?;
        
        // Generate certificate
        let mut params = CertificateParams::default();
        params.not_before = time::OffsetDateTime::now_utc();
        params.not_after = params.not_before + time::Duration::days(90); // 90 days
        
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &request.agent_id);
        dn.push(DnType::OrganizationName, "Mister Smith Agent");
        params.distinguished_name = dn;
        
        // Add Subject Alternative Names
        params.subject_alt_names = vec![
            rcgen::SanType::DnsName(format!("{}.agents.local", request.agent_id)),
            rcgen::SanType::Custom(
                2, 5, 29, 17, // Extended Key Usage OID
                request.agent_id.as_bytes().to_vec(),
            ),
        ];
        
        params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ClientAuth,
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        ];
        
        let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        let cert = params.signed_by(
            &key_pair,
            &self.ca.intermediate_cert,
            &self.ca.intermediate_key,
        )?;
        
        // Store certificate
        let issued_cert = IssuedCertificate {
            serial_number: cert.get_params().serial_number.clone(),
            certificate: cert.serialize_pem()?,
            private_key: key_pair.serialize_pem(),
            agent_id: request.agent_id,
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(90),
            fingerprint: self.calculate_fingerprint(&cert)?,
        };
        
        self.repository.save(&issued_cert).await?;
        
        Ok(issued_cert)
    }
    
    pub async fn revoke_certificate(
        &self,
        serial_number: &str,
        reason: RevocationReason,
    ) -> Result<(), CAError> {
        let mut crl = self.revocation_list.write().await;
        crl.add_revocation(serial_number, reason, Utc::now())?;
        
        // Publish updated CRL
        self.publish_crl(&*crl).await?;
        
        // Update OCSP responder
        self.update_ocsp_status(serial_number, CertificateStatus::Revoked).await?;
        
        Ok(())
    }
}
```

## 4. mTLS Configuration Details

### 4.1 mTLS Server Configuration

```rust
use rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;

pub struct MtlsServerConfig {
    tls_config: Arc<ServerConfig>,
    client_auth_required: bool,
    allowed_client_cas: Vec<Certificate>,
    sni_resolver: Option<Arc<dyn SniResolver>>,
}

impl MtlsServerConfig {
    pub fn new(config: &MtlsConfig) -> Result<Self, TlsError> {
        let certs = load_certs(&config.cert_path)?;
        let key = load_private_key(&config.key_path)?;
        
        let client_cert_verifier = Arc::new(
            CustomClientCertVerifier::new(
                config.ca_certs.clone(),
                config.verify_depth,
                config.crl_endpoint.clone(),
            )?
        );
        
        let mut tls_config = ServerConfig::builder()
            .with_cipher_suites(&[
                rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS13_AES_128_GCM_SHA256,
                rustls::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
            ])
            .with_kx_groups(&[&rustls::kx_group::X25519])
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(certs, key)?;
        
        // Configure ALPN
        tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        
        // Configure session resumption
        tls_config.session_storage = ServerSessionMemoryCache::new(1024);
        tls_config.ticketer = rustls::Ticketer::new()?;
        
        Ok(Self {
            tls_config: Arc::new(tls_config),
            client_auth_required: config.require_client_cert,
            allowed_client_cas: config.ca_certs,
            sni_resolver: config.sni_resolver,
        })
    }
}

struct CustomClientCertVerifier {
    ca_certs: Vec<Certificate>,
    verify_depth: u32,
    crl_checker: Arc<CrlChecker>,
    ocsp_checker: Arc<OcspChecker>,
}

impl rustls::server::ClientCertVerifier for CustomClientCertVerifier {
    fn client_auth_root_subjects(&self) -> Option<rustls::DistinguishedNames> {
        Some(self.ca_certs.iter()
            .map(|cert| cert.0.clone())
            .collect())
    }
    
    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        now: std::time::SystemTime,
    ) -> Result<rustls::server::ClientCertVerified, rustls::Error> {
        // Verify certificate chain
        let chain = self.build_chain(end_entity, intermediates)?;
        self.verify_chain(&chain, now)?;
        
        // Check revocation status
        if self.crl_checker.is_revoked(end_entity)? {
            return Err(rustls::Error::General("Certificate revoked".into()));
        }
        
        // OCSP check (non-blocking)
        let ocsp_checker = self.ocsp_checker.clone();
        let cert_clone = end_entity.clone();
        tokio::spawn(async move {
            if let Err(e) = ocsp_checker.check_status(&cert_clone).await {
                log::warn!("OCSP check failed: {}", e);
            }
        });
        
        // Extract and validate agent ID from certificate
        let agent_id = extract_agent_id(end_entity)?;
        validate_agent_permissions(&agent_id)?;
        
        Ok(rustls::server::ClientCertVerified::assertion())
    }
}
```

### 4.2 mTLS Client Configuration

```rust
pub struct MtlsClientConfig {
    tls_config: Arc<ClientConfig>,
    client_cert: Certificate,
    client_key: PrivateKey,
}

impl MtlsClientConfig {
    pub fn new(config: &ClientMtlsConfig) -> Result<Self, TlsError> {
        let client_cert = load_cert(&config.cert_path)?;
        let client_key = load_private_key(&config.key_path)?;
        let ca_certs = load_ca_certs(&config.ca_path)?;
        
        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(&cert)?;
        }
        
        let tls_config = ClientConfig::builder()
            .with_cipher_suites(&[
                rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS13_AES_128_GCM_SHA256,
            ])
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(root_store)
            .with_single_cert(vec![client_cert.clone()], client_key.clone())?;
        
        Ok(Self {
            tls_config: Arc::new(tls_config),
            client_cert,
            client_key,
        })
    }
    
    pub fn create_connector(&self) -> TlsConnector {
        TlsConnector::from(self.tls_config.clone())
    }
}
```

## 5. Session Management Specifications

### 5.1 Session Token Structure

```rust
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub token_family: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_expires_at: DateTime<Utc>,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub device_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: String,
    pub session_id: String,
    pub token_family: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub replaced_by: Option<String>,
}

pub struct SessionManager {
    redis: ConnectionManager,
    jwt_manager: Arc<JwtManager>,
    config: SessionConfig,
}

impl SessionManager {
    pub async fn create_session(
        &self,
        agent: &Agent,
        context: &SessionContext,
    ) -> Result<SessionTokens, SessionError> {
        // Generate session ID and token family
        let session_id = Uuid::new_v4().to_string();
        let token_family = Uuid::new_v4().to_string();
        
        // Create access token
        let access_token = self.jwt_manager.generate_token(agent)?;
        
        // Create refresh token
        let refresh_token_id = Uuid::new_v4().to_string();
        let refresh_token = self.generate_refresh_token(
            &refresh_token_id,
            &session_id,
            &token_family,
        )?;
        
        // Store session
        let session = Session {
            id: session_id.clone(),
            agent_id: agent.id.clone(),
            token_family: token_family.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.session_ttl),
            refresh_token: refresh_token_id.clone(),
            refresh_expires_at: Utc::now() + Duration::seconds(self.config.refresh_ttl),
            ip_address: context.ip_address,
            user_agent: context.user_agent.clone(),
            device_id: context.device_id.clone(),
            metadata: HashMap::new(),
        };
        
        self.store_session(&session).await?;
        
        Ok(SessionTokens {
            access_token,
            refresh_token,
            session_id,
            expires_in: self.config.access_token_ttl,
        })
    }
}
```

### 5.2 Refresh Token Rotation

```rust
impl SessionManager {
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
    ) -> Result<SessionTokens, SessionError> {
        // Decode and validate refresh token
        let token_data = self.decode_refresh_token(refresh_token)?;
        
        // Check if token has been used (replay attack detection)
        if token_data.used {
            // Invalidate entire token family
            self.invalidate_token_family(&token_data.token_family).await?;
            return Err(SessionError::TokenReplayDetected);
        }
        
        // Get session
        let session = self.get_session(&token_data.session_id).await?;
        
        // Validate session is still active
        if session.expires_at < Utc::now() {
            return Err(SessionError::SessionExpired);
        }
        
        // Generate new tokens
        let agent = self.get_agent(&session.agent_id).await?;
        let new_access_token = self.jwt_manager.generate_token(&agent)?;
        
        let new_refresh_token_id = Uuid::new_v4().to_string();
        let new_refresh_token = self.generate_refresh_token(
            &new_refresh_token_id,
            &session.id,
            &session.token_family,
        )?;
        
        // Mark old refresh token as used and link to new one
        self.mark_refresh_token_used(
            &token_data.id,
            Some(&new_refresh_token_id),
        ).await?;
        
        // Update session
        self.update_session_activity(&session.id).await?;
        
        Ok(SessionTokens {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            session_id: session.id,
            expires_in: self.config.access_token_ttl,
        })
    }
}
```

## 6. Multi-Factor Authentication Patterns

### 6.1 TOTP Implementation

```rust
use totp_lite::{totp, Sha1};

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpConfig {
    pub secret: Vec<u8>,
    pub period: u64,
    pub digits: u32,
    pub algorithm: TotpAlgorithm,
    pub issuer: String,
    pub account_name: String,
}

pub struct MfaManager {
    repository: Arc<dyn MfaRepository>,
    config: MfaConfig,
}

impl MfaManager {
    pub async fn enable_totp(
        &self,
        agent_id: &str,
    ) -> Result<TotpEnrollment, MfaError> {
        // Generate secret
        let secret = generate_secret(32)?;
        
        let totp_config = TotpConfig {
            secret: secret.clone(),
            period: 30,
            digits: 6,
            algorithm: TotpAlgorithm::Sha256,
            issuer: "Mister Smith".to_string(),
            account_name: agent_id.to_string(),
        };
        
        // Generate provisioning URI for QR code
        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            totp_config.issuer,
            totp_config.account_name,
            base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret),
            totp_config.issuer,
            "SHA256",
            totp_config.digits,
            totp_config.period,
        );
        
        // Generate backup codes
        let backup_codes = self.generate_backup_codes(8)?;
        
        // Store configuration
        self.repository.save_totp_config(agent_id, &totp_config).await?;
        self.repository.save_backup_codes(agent_id, &backup_codes).await?;
        
        Ok(TotpEnrollment {
            uri,
            secret: base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret),
            backup_codes,
        })
    }
    
    pub async fn verify_totp(
        &self,
        agent_id: &str,
        code: &str,
    ) -> Result<bool, MfaError> {
        let config = self.repository.get_totp_config(agent_id).await?;
        
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check current and adjacent windows for clock skew
        for window in -1..=1 {
            let time = current_time / config.period + window as u64;
            let expected = totp(&config.secret, time);
            
            if constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                return Ok(true);
            }
        }
        
        // Check backup codes
        if self.verify_backup_code(agent_id, code).await? {
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

### 6.2 WebAuthn/FIDO2 Support

```rust
use webauthn_rs::prelude::*;

pub struct WebAuthnManager {
    webauthn: Webauthn,
    repository: Arc<dyn WebAuthnRepository>,
}

impl WebAuthnManager {
    pub async fn start_registration(
        &self,
        agent_id: &str,
    ) -> Result<CreationChallengeResponse, WebAuthnError> {
        let user_unique_id = Uuid::parse_str(agent_id)?;
        
        let (ccr, reg_state) = self.webauthn.start_passkey_registration(
            user_unique_id,
            agent_id,
            agent_id,
            None,
        )?;
        
        // Store registration state
        self.repository
            .save_registration_state(agent_id, &reg_state)
            .await?;
        
        Ok(ccr)
    }
    
    pub async fn finish_registration(
        &self,
        agent_id: &str,
        credential: &RegisterPublicKeyCredential,
    ) -> Result<(), WebAuthnError> {
        let reg_state = self.repository
            .get_registration_state(agent_id)
            .await?;
        
        let passkey = self.webauthn
            .finish_passkey_registration(credential, &reg_state)?;
        
        self.repository
            .save_passkey(agent_id, &passkey)
            .await?;
        
        Ok(())
    }
    
    pub async fn start_authentication(
        &self,
        agent_id: &str,
    ) -> Result<RequestChallengeResponse, WebAuthnError> {
        let passkeys = self.repository
            .get_passkeys(agent_id)
            .await?;
        
        let (rcr, auth_state) = self.webauthn
            .start_passkey_authentication(&passkeys)?;
        
        self.repository
            .save_auth_state(agent_id, &auth_state)
            .await?;
        
        Ok(rcr)
    }
}
```

## 7. Identity Provider Integration

### 7.1 OAuth2 Authorization Code Flow

```rust
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;

pub struct OAuth2Manager {
    clients: HashMap<String, BasicClient>,
    repository: Arc<dyn OAuth2Repository>,
}

impl OAuth2Manager {
    pub fn new(config: &OAuth2Config) -> Result<Self, OAuth2Error> {
        let mut clients = HashMap::new();
        
        for provider in &config.providers {
            let client = BasicClient::new(
                ClientId::new(provider.client_id.clone()),
                Some(ClientSecret::new(provider.client_secret.clone())),
                AuthUrl::new(provider.auth_url.clone())?,
                Some(TokenUrl::new(provider.token_url.clone())?),
            )
            .set_redirect_uri(RedirectUrl::new(provider.redirect_uri.clone())?);
            
            clients.insert(provider.name.clone(), client);
        }
        
        Ok(Self {
            clients,
            repository: Arc::new(OAuth2Repository::new(config)?),
        })
    }
    
    pub async fn start_authorization(
        &self,
        provider: &str,
        agent_id: &str,
    ) -> Result<AuthorizationUrl, OAuth2Error> {
        let client = self.clients
            .get(provider)
            .ok_or(OAuth2Error::UnknownProvider)?;
        
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Generate CSRF token
        let csrf_token = CsrfToken::new_random();
        
        // Build authorization URL
        let (auth_url, csrf_token) = client
            .authorize_url(|| csrf_token.clone())
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        
        // Store state for callback
        let auth_state = AuthorizationState {
            provider: provider.to_string(),
            agent_id: agent_id.to_string(),
            csrf_token: csrf_token.secret().clone(),
            pkce_verifier: pkce_verifier.secret().clone(),
            created_at: Utc::now(),
        };
        
        self.repository.save_auth_state(&auth_state).await?;
        
        Ok(AuthorizationUrl {
            url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        })
    }
}
```

### 7.2 OpenID Connect Discovery

```rust
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    IssuerUrl, RedirectUrl, ClientId, ClientSecret,
};

pub struct OidcManager {
    clients: HashMap<String, CoreClient>,
    metadata_cache: Arc<RwLock<HashMap<String, CoreProviderMetadata>>>,
}

impl OidcManager {
    pub async fn discover_provider(
        &self,
        issuer_url: &str,
    ) -> Result<CoreProviderMetadata, OidcError> {
        // Check cache first
        if let Some(metadata) = self.metadata_cache.read().await.get(issuer_url) {
            return Ok(metadata.clone());
        }
        
        // Discover provider metadata
        let issuer = IssuerUrl::new(issuer_url.to_string())?;
        let metadata = CoreProviderMetadata::discover_async(
            issuer,
            async_http_client,
        ).await?;
        
        // Cache metadata
        self.metadata_cache
            .write()
            .await
            .insert(issuer_url.to_string(), metadata.clone());
        
        Ok(metadata)
    }
    
    pub async fn validate_id_token(
        &self,
        id_token: &str,
        provider: &str,
    ) -> Result<IdTokenClaims, OidcError> {
        let client = self.clients
            .get(provider)
            .ok_or(OidcError::UnknownProvider)?;
        
        let id_token = client
            .id_token_verifier()
            .insecure_disable_signature_check()
            .verify_id_token_async(id_token, async_http_client)
            .await?;
        
        // Additional validation
        let claims = id_token.claims();
        
        // Verify audience
        if !claims.audiences().contains(&client.client_id()) {
            return Err(OidcError::InvalidAudience);
        }
        
        // Verify issuer
        if claims.issuer() != &client.issuer() {
            return Err(OidcError::InvalidIssuer);
        }
        
        Ok(IdTokenClaims {
            subject: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()),
            email_verified: claims.email_verified(),
            name: claims.name().map(|n| n.to_string()),
            picture: claims.picture().map(|p| p.to_string()),
            issued_at: claims.issued_at(),
            expiration: claims.expiration(),
        })
    }
}
```

## 8. Authentication Middleware Implementations

### 8.1 Axum Authentication Extractor

```rust
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub struct AuthenticatedAgent {
    pub claims: AgentClaims,
    pub session: Option<Session>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedAgent
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingToken)?;
        
        // Get JWT manager from app state
        let jwt_manager = parts
            .extensions
            .get::<Arc<JwtManager>>()
            .ok_or(AuthError::InternalError)?;
        
        // Validate token
        let claims = jwt_manager.validate_token(bearer.token())?;
        
        // Get session if available
        let session_manager = parts
            .extensions
            .get::<Arc<SessionManager>>()
            .ok_or(AuthError::InternalError)?;
        
        let session = session_manager
            .get_session_by_agent(&claims.agent_id)
            .await
            .ok();
        
        Ok(AuthenticatedAgent { claims, session })
    }
}

// Permission-based extractor
pub struct RequirePermission {
    agent: AuthenticatedAgent,
}

impl RequirePermission {
    pub fn new(resource: &str, action: &str) -> impl Fn(AuthenticatedAgent) -> Result<Self, AuthError> {
        let resource = resource.to_string();
        let action = action.to_string();
        
        move |agent: AuthenticatedAgent| {
            let has_permission = agent.claims.permissions.iter().any(|p| {
                p.resource == resource && p.actions.contains(&action)
            });
            
            if has_permission {
                Ok(RequirePermission { agent })
            } else {
                Err(AuthError::InsufficientPermissions)
            }
        }
    }
}
```

### 8.2 Tonic gRPC Interceptor

```rust
use tonic::{Request, Response, Status, service::Interceptor};

#[derive(Clone)]
pub struct AuthInterceptor {
    jwt_manager: Arc<JwtManager>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = match request.metadata().get("authorization") {
            Some(t) => t,
            None => return Err(Status::unauthenticated("No authorization token")),
        };
        
        let token_str = token
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid token format"))?;
        
        // Remove "Bearer " prefix if present
        let token_str = token_str
            .strip_prefix("Bearer ")
            .unwrap_or(token_str);
        
        // Validate token
        let claims = self.jwt_manager
            .validate_token(token_str)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;
        
        // Add claims to request extensions
        request.extensions_mut().insert(claims);
        
        Ok(request)
    }
}

// Helper to extract claims in service methods
pub fn get_claims<T>(request: &Request<T>) -> Result<&AgentClaims, Status> {
    request
        .extensions()
        .get::<AgentClaims>()
        .ok_or_else(|| Status::internal("Missing authentication claims"))
}
```

### 8.3 NATS Authentication Callback

```rust
use async_nats::{
    auth::{AuthCallback, AuthResponse},
    Client, ConnectOptions,
};

pub struct NatsAuthHandler {
    jwt_manager: Arc<JwtManager>,
    api_key_manager: Arc<ApiKeyManager>,
}

#[async_trait]
impl AuthCallback for NatsAuthHandler {
    async fn authenticate(
        &self,
        nonce: &[u8],
    ) -> Result<AuthResponse, async_nats::Error> {
        // Try JWT authentication first
        if let Ok(token) = std::env::var("NATS_JWT_TOKEN") {
            if let Ok(claims) = self.jwt_manager.validate_token(&token) {
                return Ok(AuthResponse::jwt(token, None));
            }
        }
        
        // Try API key authentication
        if let Ok(api_key) = std::env::var("NATS_API_KEY") {
            if let Ok(key_info) = self.api_key_manager.validate_key(&api_key).await {
                // Generate NATS credentials from API key
                let nats_jwt = self.generate_nats_jwt(&key_info)?;
                return Ok(AuthResponse::jwt(nats_jwt, None));
            }
        }
        
        // Fall back to user/password if configured
        if let (Ok(user), Ok(pass)) = (
            std::env::var("NATS_USER"),
            std::env::var("NATS_PASS"),
        ) {
            return Ok(AuthResponse::user_pass(user, pass));
        }
        
        Err(async_nats::Error::Other("No valid authentication method".into()))
    }
}

pub async fn create_authenticated_nats_client(
    urls: Vec<String>,
    auth_handler: NatsAuthHandler,
) -> Result<Client, async_nats::Error> {
    let options = ConnectOptions::new()
        .auth_callback(Arc::new(auth_handler))
        .require_tls(true)
        .tls_client_auth(
            include_bytes!("../certs/client-cert.pem").to_vec(),
            include_bytes!("../certs/client-key.pem").to_vec(),
        )
        .add_root_certificates(include_bytes!("../certs/ca.pem").to_vec())
        .connection_timeout(Duration::from_secs(10))
        .ping_interval(Duration::from_secs(60))
        .reconnect_buffer_size(8 * 1024 * 1024); // 8MB
    
    let client = options
        .connect(urls[0].as_str())
        .await?;
    
    Ok(client)
}
```

### 8.4 Rate Limiting and DDoS Protection

```rust
use governor::{Quota, RateLimiter, state::keyed::DefaultKeyedStateStore};
use std::net::IpAddr;

pub struct RateLimitMiddleware {
    ip_limiter: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>>>,
    api_key_limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>>>,
    jwt_limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>>>,
}

impl RateLimitMiddleware {
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            ip_limiter: Arc::new(RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(config.ip_requests_per_minute).unwrap())
            )),
            api_key_limiter: Arc::new(RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(config.api_key_requests_per_minute).unwrap())
            )),
            jwt_limiter: Arc::new(RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(config.jwt_requests_per_minute).unwrap())
            )),
        }
    }
    
    pub async fn check_rate_limit(
        &self,
        auth_type: &AuthType,
        key: &str,
        ip: IpAddr,
    ) -> Result<(), RateLimitError> {
        // Always check IP rate limit
        self.ip_limiter
            .check_key(&ip)
            .map_err(|_| RateLimitError::IpRateLimitExceeded)?;
        
        // Check auth-specific rate limit
        match auth_type {
            AuthType::ApiKey => {
                self.api_key_limiter
                    .check_key(&key.to_string())
                    .map_err(|_| RateLimitError::ApiKeyRateLimitExceeded)?;
            }
            AuthType::Jwt => {
                self.jwt_limiter
                    .check_key(&key.to_string())
                    .map_err(|_| RateLimitError::JwtRateLimitExceeded)?;
            }
            _ => {}
        }
        
        Ok(())
    }
}

// DDoS protection with connection limits
pub struct DdosProtection {
    connection_tracker: Arc<DashMap<IpAddr, ConnectionInfo>>,
    config: DdosConfig,
}

impl DdosProtection {
    pub async fn check_connection(
        &self,
        ip: IpAddr,
    ) -> Result<(), DdosError> {
        let mut info = self.connection_tracker
            .entry(ip)
            .or_insert_with(|| ConnectionInfo::new());
        
        // Check connection rate
        let now = Instant::now();
        info.connections.push(now);
        info.connections.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
        
        if info.connections.len() > self.config.max_connections_per_minute {
            return Err(DdosError::ConnectionRateExceeded);
        }
        
        // Check concurrent connections
        if info.active_connections >= self.config.max_concurrent_connections {
            return Err(DdosError::ConcurrentConnectionsExceeded);
        }
        
        info.active_connections += 1;
        
        Ok(())
    }
}
```

## Integration Examples

### Example 1: Complete Authentication Flow

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};

#[tokio::main]
async fn main() {
    // Initialize authentication components
    let jwt_manager = Arc::new(JwtManager::new(&jwt_config).unwrap());
    let session_manager = Arc::new(SessionManager::new(&session_config).await.unwrap());
    let mfa_manager = Arc::new(MfaManager::new(&mfa_config).unwrap());
    
    // Build application
    let app = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/mfa/verify", post(verify_mfa))
        .route("/protected", get(protected_endpoint))
        .layer(Extension(jwt_manager))
        .layer(Extension(session_manager))
        .layer(Extension(mfa_manager));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn login(
    State(session_manager): State<Arc<SessionManager>>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Validate credentials
    let agent = authenticate_agent(&credentials).await?;
    
    // Create session
    let tokens = session_manager.create_session(&agent, &context).await?;
    
    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        requires_mfa: agent.mfa_enabled,
    }))
}

async fn protected_endpoint(
    auth: AuthenticatedAgent,
) -> Result<Json<ProtectedResponse>, AuthError> {
    Ok(Json(ProtectedResponse {
        message: format!("Hello, {}", auth.claims.agent_id),
        permissions: auth.claims.permissions,
    }))
}
```

### Example 2: mTLS with NATS

```rust
use async_nats::ConnectOptions;

async fn setup_secure_nats() -> Result<Client, Error> {
    let mtls_config = MtlsClientConfig::new(&client_config)?;
    
    let options = ConnectOptions::new()
        .require_tls(true)
        .tls_client_auth(
            mtls_config.client_cert.0.clone(),
            mtls_config.client_key.0.clone(),
        )
        .add_root_certificates(ca_cert.0.clone())
        .auth_callback(Arc::new(NatsAuthHandler::new(jwt_manager)))
        .connection_timeout(Duration::from_secs(10));
    
    let client = options.connect("nats://secure.nats.local:4222").await?;
    
    Ok(client)
}
```

## Security Considerations

### Token Security

- **Algorithm Selection**: Use RS256 or ES256 for JWT signing (asymmetric algorithms)
- **Key Management**: Implement proper key rotation procedures with overlap periods
- **Storage Security**: Store refresh tokens securely (hashed in database)
- **Revocation**: Implement token revocation mechanisms with blacklisting
- **Token Lifetime**: Use short-lived access tokens (15-30 minutes)
- **Validation**: Implement JTI (JWT ID) tracking for replay protection

### Network Security

- **TLS Version**: Enforce TLS 1.3 minimum for all connections
- **Certificate Management**: Implement certificate pinning for critical services
- **Service Communication**: Use mTLS for service-to-service communication
- **SNI Validation**: Implement proper SNI validation
- **OCSP Stapling**: Enable OCSP stapling for real-time certificate validation

### Implementation Security

- **Timing Attacks**: Use constant-time comparison for token validation
- **Rate Limiting**: Implement rate limiting at multiple layers (IP, user, endpoint)
- **Audit Logging**: Log authentication events for audit with structured format
- **Monitoring**: Monitor for suspicious patterns and implement alerting
- **Account Protection**: Implement account lockout policies with exponential backoff
- **Session Security**: Implement session binding and device fingerprinting

## Performance Optimizations

### Caching Strategy

- **Token Caching**: Cache validated tokens for 30 seconds (balance security/performance)
- **JWKS Caching**: Cache JWKS for 1 hour with background refresh
- **Provider Metadata**: Cache OAuth2 provider metadata for 24 hours
- **Session Storage**: Use Redis for distributed session storage with clustering
- **Certificate Caching**: Cache parsed certificates for 5 minutes

### Connection Pooling

- **IdP Connections**: Maintain persistent connections to IdPs with keep-alive
- **Database Pooling**: Pool database connections for auth queries (min 5, max 20)
- **TLS Session Reuse**: Reuse TLS sessions where possible
- **Connection Limits**: Configure appropriate connection limits per service

### Async Processing

- **Background Tasks**: Process MFA enrollment and key rotation asynchronously
- **Batch Operations**: Batch token validation for high-throughput scenarios
- **Non-blocking I/O**: Use non-blocking I/O for all network operations

## Monitoring and Metrics

### Key Metrics

- **Authentication Metrics**:
  - Success/failure rates by authentication method
  - Token validation latency (p50, p95, p99)
  - MFA adoption rates and success rates
  - Session duration distribution
  - API key usage patterns and quotas

- **Security Metrics**:
  - Failed authentication attempts per IP/user
  - Suspicious login patterns (location, device changes)
  - Token revocation events
  - Certificate expiration timeline
  - Rate limit violations by endpoint

### Alerts

- **Critical Alerts**:
  - High authentication failure rate (>5% in 5 minutes)
  - Certificate expiration warnings (30, 7, 1 days)
  - Token blacklist growth rate
  - Multiple failed MFA attempts

- **Warning Alerts**:
  - Unusual token refresh patterns
  - Rate limit violations
  - New device/location logins
  - Extended session durations

### Dashboard Integration

- **Real-time Monitoring**: Integration with Grafana/Prometheus for real-time auth metrics
- **Security Dashboard**: Dedicated security dashboard for authentication events
- **Audit Trails**: Comprehensive audit logging for compliance and forensics

---

*Authentication Implementation Specifications - Comprehensive Security for Agent Communication*
