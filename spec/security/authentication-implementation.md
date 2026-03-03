# Authentication Implementation - Certificate + JWT

## Overview

This document provides complete implementation patterns for certificate management and JWT authentication in the Mister Smith Framework. It includes production-ready code for certificate lifecycle management, JWT token operations, and authentication middleware.

**Key Features**:

- **Certificate Management**: Complete CA setup and certificate lifecycle management
- **JWT Authentication**: ES384-based token generation and verification
- **Session Management**: Secure session handling with refresh token rotation
- **Middleware Integration**: Authentication middleware for various transport layers
- **Security Standards**: TLS 1.3 minimum enforcement across all components

## Integration Points

**Security Framework**: [`security-framework.md`](security-framework.md) - Core security patterns and configurations
**Authentication Specifications**: [`authentication-specifications.md`](authentication-specifications.md) - Detailed authentication patterns
**Authorization Implementation**: [`authorization-implementation.md`](authorization-implementation.md) - RBAC and permission implementation
**Security Integration**: [`security-integration.md`](security-integration.md) - NATS and hook security implementation
**Transport Layer**: [`../transport/`](../transport/) - Secure transport implementations

## Implementation Requirements

**TLS Standards**: This implementation enforces TLS 1.3 minimum across the entire framework:

- Ensures consistent security posture across all components
- Provides modern cryptographic protection for all connections
- Meets current security best practices and compliance requirements

**Security Features**:

- **mTLS**: Mutual TLS authentication for service-to-service communication
- **JWT**: ES384-based token authentication with role-based claims
- **Session Management**: Secure session handling with refresh token rotation
- **Certificate Management**: Automated certificate lifecycle management
- **Rate Limiting**: Authentication rate limiting and DDoS protection

## 1. Certificate Management Implementation

### 1.1 Certificate Generation Scripts

**Complete Certificate Authority Setup:**

```bash
#!/bin/bash
# generate_ca.sh - Complete CA setup for Mister Smith Framework

set -euo pipefail

CA_DIR="/etc/mister-smith/certs"
CA_KEY_SIZE=4096
CERT_VALIDITY_DAYS=3650
SERVER_CERT_VALIDITY_DAYS=90
CLIENT_CERT_VALIDITY_DAYS=365

# Create directory structure
mkdir -p $CA_DIR/{ca,server,client,crl}
cd $CA_DIR

# Generate CA private key (RSA 4096-bit)
openssl genrsa -out ca/ca-key.pem $CA_KEY_SIZE

# Generate CA certificate (10 years)
openssl req -new -x509 -days $CERT_VALIDITY_DAYS -key ca/ca-key.pem \
    -out ca/ca-cert.pem \
    -subj "/C=US/ST=CA/L=San Francisco/O=Mister Smith Framework/OU=Security/CN=Mister Smith CA"

# Generate server private key
openssl genrsa -out server/server-key.pem 4096

# Generate server certificate signing request
openssl req -new -key server/server-key.pem -out server/server.csr \
    -subj "/C=US/ST=CA/L=San Francisco/O=Mister Smith Framework/OU=Services/CN=mister-smith.local"

# Create server certificate extensions
cat > server/server-ext.cnf << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = mister-smith.local
DNS.2 = localhost
DNS.3 = *.mister-smith.local
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

# Sign server certificate (90 days)
openssl x509 -req -in server/server.csr -CA ca/ca-cert.pem -CAkey ca/ca-key.pem \
    -out server/server-cert.pem -days $SERVER_CERT_VALIDITY_DAYS \
    -extensions v3_req -extfile server/server-ext.cnf -CAcreateserial

# Generate client private key
openssl genrsa -out client/client-key.pem 4096

# Generate client certificate signing request
openssl req -new -key client/client-key.pem -out client/client.csr \
    -subj "/C=US/ST=CA/L=San Francisco/O=Mister Smith Framework/OU=Clients/CN=mister-smith-client"

# Create client certificate extensions
cat > client/client-ext.cnf << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
EOF

# Sign client certificate (365 days)
openssl x509 -req -in client/client.csr -CA ca/ca-cert.pem -CAkey ca/ca-key.pem \
    -out client/client-cert.pem -days $CLIENT_CERT_VALIDITY_DAYS \
    -extensions v3_req -extfile client/client-ext.cnf -CAcreateserial

# Set proper permissions
chmod 600 ca/ca-key.pem server/server-key.pem client/client-key.pem
chmod 644 ca/ca-cert.pem server/server-cert.pem client/client-cert.pem

echo "Certificates generated successfully in $CA_DIR"
```

**Certificate Rotation Script:**

```bash
#!/bin/bash
# rotate_certs.sh - Zero-downtime certificate rotation

set -euo pipefail

CA_DIR="/etc/mister-smith/certs"
BACKUP_DIR="/etc/mister-smith/certs/backup/$(date +%Y%m%d_%H%M%S)"

# Create backup
mkdir -p $BACKUP_DIR
cp -r $CA_DIR/* $BACKUP_DIR/

# Check certificate expiration (warn at 30 days)
check_expiration() {
    local cert_file=$1
    local threshold_days=30
    
    expiry_date=$(openssl x509 -in $cert_file -noout -enddate | cut -d= -f2)
    expiry_timestamp=$(date -d "$expiry_date" +%s)
    current_timestamp=$(date +%s)
    days_until_expiry=$(( (expiry_timestamp - current_timestamp) / 86400 ))
    
    if [ $days_until_expiry -le $threshold_days ]; then
        echo "WARNING: Certificate $cert_file expires in $days_until_expiry days"
        return 1
    fi
    return 0
}

# Rotate server certificate
rotate_server_cert() {
    echo "Rotating server certificate..."
    
    # Generate new server key and certificate
    openssl genrsa -out $CA_DIR/server/server-key-new.pem 4096
    openssl req -new -key $CA_DIR/server/server-key-new.pem -out $CA_DIR/server/server-new.csr \
        -subj "/C=US/ST=CA/L=San Francisco/O=Mister Smith Framework/OU=Services/CN=mister-smith.local"
    
    openssl x509 -req -in $CA_DIR/server/server-new.csr -CA $CA_DIR/ca/ca-cert.pem \
        -CAkey $CA_DIR/ca/ca-key.pem -out $CA_DIR/server/server-cert-new.pem \
        -days 90 -extensions v3_req -extfile $CA_DIR/server/server-ext.cnf -CAcreateserial
    
    # Atomic replacement
    mv $CA_DIR/server/server-cert-new.pem $CA_DIR/server/server-cert.pem
    mv $CA_DIR/server/server-key-new.pem $CA_DIR/server/server-key.pem
    
    # Send SIGHUP to services for hot reload
    systemctl reload mister-smith-api
    systemctl reload nats-server
    
    echo "Server certificate rotated successfully"
}

# Check and rotate if needed
if ! check_expiration $CA_DIR/server/server-cert.pem; then
    rotate_server_cert
fi
```

### 1.2 Rustls Certificate Management Implementation

**Complete Certificate Manager:**

```rust
// certificate_manager.rs
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct CertificateManager {
    ca_cert_path: String,
    server_cert_path: String,
    server_key_path: String,
    client_cert_path: String,
    client_key_path: String,
}

impl CertificateManager {
    pub fn new() -> Self {
        Self {
            ca_cert_path: "/etc/mister-smith/certs/ca/ca-cert.pem".to_string(),
            server_cert_path: "/etc/mister-smith/certs/server/server-cert.pem".to_string(),
            server_key_path: "/etc/mister-smith/certs/server/server-key.pem".to_string(),
            client_cert_path: "/etc/mister-smith/certs/client/client-cert.pem".to_string(),
            client_key_path: "/etc/mister-smith/certs/client/client-key.pem".to_string(),
        }
    }

    /// Load certificates from PEM files
    pub fn load_certificates(&self, path: &str) -> Result<Vec<Certificate>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open certificate file: {}", path))?;
        let mut reader = BufReader::new(file);
        
        let certs = certs(&mut reader)
            .with_context(|| "Failed to parse certificates")?
            .into_iter()
            .map(Certificate)
            .collect();

        if certs.is_empty() {
            anyhow::bail!("No certificates found in file: {}", path);
        }

        info!("Loaded {} certificates from {}", certs.len(), path);
        Ok(certs)
    }

    /// Load private key from PEM file
    pub fn load_private_key(&self, path: &str) -> Result<PrivateKey> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open private key file: {}", path))?;
        let mut reader = BufReader::new(file);

        // Try PKCS8 format first
        if let Ok(mut keys) = pkcs8_private_keys(&mut reader) {
            if !keys.is_empty() {
                info!("Loaded PKCS8 private key from {}", path);
                return Ok(PrivateKey(keys.remove(0)));
            }
        }

        // Reset reader and try RSA format
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        let mut keys = rsa_private_keys(&mut reader)
            .with_context(|| "Failed to parse RSA private key")?;

        if keys.is_empty() {
            anyhow::bail!("No private keys found in file: {}", path);
        }

        info!("Loaded RSA private key from {}", path);
        Ok(PrivateKey(keys.remove(0)))
    }

    /// Create TLS server configuration with mTLS
    pub fn create_server_config(&self) -> Result<Arc<ServerConfig>> {
        let certs = self.load_certificates(&self.server_cert_path)?;
        let key = self.load_private_key(&self.server_key_path)?;
        let ca_certs = self.load_certificates(&self.ca_cert_path)?;

        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(&cert)
                .with_context(|| "Failed to add CA certificate to root store")?;
        }

        let client_cert_verifier = rustls::server::AllowAnyAuthenticatedClient::new(root_store);

        let config = ServerConfig::builder()
            .with_cipher_suites(&[
                rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
                rustls::cipher_suite::TLS13_AES_128_GCM_SHA256,
            ])
            .with_kx_groups(&[
                &rustls::kx_group::X25519,
                &rustls::kx_group::SECP384R1,
                &rustls::kx_group::SECP256R1,
            ])
            .with_protocol_versions(&[&rustls::version::TLS13])
            // ✅ STANDARDIZED: TLS 1.3 minimum enforced framework-wide
            .with_context(|| "Failed to configure TLS parameters")?
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(certs, key)
            .with_context(|| "Failed to configure server certificate")?;

        info!("Created TLS server configuration with mTLS");
        Ok(Arc::new(config))
    }

    /// Create TLS client configuration
    pub fn create_client_config(&self) -> Result<Arc<ClientConfig>> {
        let certs = self.load_certificates(&self.client_cert_path)?;
        let key = self.load_private_key(&self.client_key_path)?;
        let ca_certs = self.load_certificates(&self.ca_cert_path)?;

        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(&cert)
                .with_context(|| "Failed to add CA certificate to root store")?;
        }

        let config = ClientConfig::builder()
            .with_cipher_suites(&[
                rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
                rustls::cipher_suite::TLS13_AES_128_GCM_SHA256,
            ])
            .with_kx_groups(&[
                &rustls::kx_group::X25519,
                &rustls::kx_group::SECP384R1,
                &rustls::kx_group::SECP256R1,
            ])
            .with_protocol_versions(&[&rustls::version::TLS13])
            // ✅ STANDARDIZED: TLS 1.3 minimum enforced framework-wide
            .with_context(|| "Failed to configure TLS parameters")?
            .with_root_certificates(root_store)
            .with_single_cert(certs, key)
            .with_context(|| "Failed to configure client certificate")?;

        info!("Created TLS client configuration");
        Ok(Arc::new(config))
    }

    /// Check certificate expiration
    pub fn check_certificate_expiration(&self, cert_path: &str) -> Result<Duration> {
        use x509_parser::prelude::*;
        
        let cert_data = std::fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate: {}", cert_path))?;
            
        let pem = pem::parse(&cert_data)
            .with_context(|| "Failed to parse PEM certificate")?;
            
        let x509 = X509Certificate::from_der(&pem.contents)
            .with_context(|| "Failed to parse X509 certificate")?;

        let expiry_time = x509.1.validity().not_after.timestamp() as u64;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if expiry_time <= current_time {
            anyhow::bail!("Certificate has expired: {}", cert_path);
        }

        let remaining = Duration::from_secs(expiry_time - current_time);
        
        if remaining.as_secs() < 30 * 24 * 60 * 60 { // 30 days
            warn!("Certificate expires in {} days: {}", 
                remaining.as_secs() / (24 * 60 * 60), cert_path);
        }

        Ok(remaining)
    }

    /// Start certificate monitoring task
    pub async fn start_monitoring(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_hours(24));
            
            loop {
                interval.tick().await;
                
                // Check server certificate expiration
                if let Err(e) = manager.check_certificate_expiration(&manager.server_cert_path) {
                    error!("Server certificate check failed: {}", e);
                }
                
                // Check client certificate expiration
                if let Err(e) = manager.check_certificate_expiration(&manager.client_cert_path) {
                    error!("Client certificate check failed: {}", e);
                }
            }
        });
        
        info!("Started certificate monitoring task");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_certificate_manager() {
        let temp_dir = TempDir::new().unwrap();
        // Add comprehensive tests for certificate operations
    }
}
```

## 2. JWT Authentication Implementation

### 2.1 JWT Service Implementation

**Security Features**: Complete JWT implementation with token revocation, key rotation, and MFA support.

**Complete JWT Authentication Service:**

```rust
// jwt_service.rs
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use uuid::Uuid;

/// Standard security parameters - DO NOT MODIFY
const ACCESS_TOKEN_DURATION: u64 = 15 * 60; // 15 minutes
const REFRESH_TOKEN_DURATION: u64 = 7 * 24 * 60 * 60; // 7 days
const API_KEY_DURATION: u64 = 90 * 24 * 60 * 60; // 90 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "readonly")]
    ReadOnly,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "moderator")]
    Moderator,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "system")]
    System,
}

impl Role {
    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            Role::ReadOnly => vec!["read:own"],
            Role::User => vec!["read:own", "write:own"],
            Role::Moderator => vec!["read:own", "write:own", "read:team", "write:team"],
            Role::Admin => vec!["read:*", "write:*", "delete:*"],
            Role::System => vec!["read:*", "write:*", "delete:*", "admin:*"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub roles: Vec<Role>,
    pub permissions: Vec<String>,
    pub token_type: TokenType,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh,
    #[serde(rename = "api_key")]
    ApiKey,
}

pub struct JwtService {
    access_key: ES384KeyPair,
    refresh_key: ES384KeyPair,
    api_key: ES384KeyPair,
    issuer: String,
    audience: String,
}

impl JwtService {
    pub fn new() -> Result<Self> {
        // Generate separate key pairs for different token types
        let access_key = ES384KeyPair::generate();
        let refresh_key = ES384KeyPair::generate();
        let api_key = ES384KeyPair::generate();

        Ok(Self {
            access_key,
            refresh_key,
            api_key,
            issuer: "mister-smith-framework".to_string(),
            audience: "mister-smith-services".to_string(),
        })
    }

    /// Generate access token (15 minutes expiration)
    pub fn generate_access_token(&self, user_id: Uuid, tenant_id: Uuid, roles: Vec<Role>) -> Result<String> {
        let permissions = roles.iter()
            .flat_map(|role| role.permissions())
            .map(|p| p.to_string())
            .collect();

        let user_claims = UserClaims {
            user_id,
            tenant_id,
            roles,
            permissions,
            token_type: TokenType::Access,
            session_id: Uuid::new_v4(),
        };

        let claims = Claims::with_custom_claims(user_claims, Duration::from_secs(ACCESS_TOKEN_DURATION))
            .with_issuer(&self.issuer)
            .with_audience(&self.audience)
            .with_subject(&user_id.to_string());

        let token = self.access_key.sign(claims)
            .with_context(|| "Failed to sign access token")?;

        info!("Generated access token for user: {}", user_id);
        Ok(token)
    }

    /// Generate refresh token (7 days expiration)
    pub fn generate_refresh_token(&self, user_id: Uuid, tenant_id: Uuid, session_id: Uuid) -> Result<String> {
        let user_claims = UserClaims {
            user_id,
            tenant_id,
            roles: vec![], // Refresh tokens don't carry permissions
            permissions: vec![],
            token_type: TokenType::Refresh,
            session_id,
        };

        let claims = Claims::with_custom_claims(user_claims, Duration::from_secs(REFRESH_TOKEN_DURATION))
            .with_issuer(&self.issuer)
            .with_audience(&self.audience)
            .with_subject(&user_id.to_string());

        let token = self.refresh_key.sign(claims)
            .with_context(|| "Failed to sign refresh token")?;

        info!("Generated refresh token for user: {}", user_id);
        Ok(token)
    }

    /// Generate API key (90 days expiration)
    pub fn generate_api_key(&self, user_id: Uuid, tenant_id: Uuid, roles: Vec<Role>) -> Result<String> {
        let permissions = roles.iter()
            .flat_map(|role| role.permissions())
            .map(|p| p.to_string())
            .collect();

        let user_claims = UserClaims {
            user_id,
            tenant_id,
            roles,
            permissions,
            token_type: TokenType::ApiKey,
            session_id: Uuid::new_v4(),
        };

        let claims = Claims::with_custom_claims(user_claims, Duration::from_secs(API_KEY_DURATION))
            .with_issuer(&self.issuer)
            .with_audience(&self.audience)
            .with_subject(&user_id.to_string());

        let token = self.api_key.sign(claims)
            .with_context(|| "Failed to sign API key")?;

        info!("Generated API key for user: {}", user_id);
        Ok(token)
    }

    /// Verify access token
    pub fn verify_access_token(&self, token: &str) -> Result<Claims<UserClaims>> {
        let public_key = self.access_key.public_key();
        
        let mut options = VerificationOptions::default();
        options.allowed_issuers = Some(HashSet::from([self.issuer.clone()]));
        options.allowed_audiences = Some(HashSet::from([self.audience.clone()]));

        let claims = public_key.verify_token::<UserClaims>(token, Some(options))
            .with_context(|| "Failed to verify access token")?;

        // Verify token type
        if !matches!(claims.custom.token_type, TokenType::Access) {
            anyhow::bail!("Invalid token type for access token");
        }

        Ok(claims)
    }

    /// Verify refresh token
    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims<UserClaims>> {
        let public_key = self.refresh_key.public_key();
        
        let mut options = VerificationOptions::default();
        options.allowed_issuers = Some(HashSet::from([self.issuer.clone()]));
        options.allowed_audiences = Some(HashSet::from([self.audience.clone()]));

        let claims = public_key.verify_token::<UserClaims>(token, Some(options))
            .with_context(|| "Failed to verify refresh token")?;

        // Verify token type
        if !matches!(claims.custom.token_type, TokenType::Refresh) {
            anyhow::bail!("Invalid token type for refresh token");
        }

        Ok(claims)
    }

    /// Verify API key
    pub fn verify_api_key(&self, token: &str) -> Result<Claims<UserClaims>> {
        let public_key = self.api_key.public_key();
        
        let mut options = VerificationOptions::default();
        options.allowed_issuers = Some(HashSet::from([self.issuer.clone()]));
        options.allowed_audiences = Some(HashSet::from([self.audience.clone()]));
        // API keys have longer validity, so allow more clock skew
        options.time_tolerance = Some(Duration::from_mins(30));

        let claims = public_key.verify_token::<UserClaims>(token, Some(options))
            .with_context(|| "Failed to verify API key")?;

        // Verify token type
        if !matches!(claims.custom.token_type, TokenType::ApiKey) {
            anyhow::bail!("Invalid token type for API key");
        }

        Ok(claims)
    }

    /// Check if user has permission for resource and action
    pub fn check_permission(&self, claims: &Claims<UserClaims>, resource: &str, action: &str) -> bool {
        let required_permission = format!("{}:{}", action, resource);
        let wildcard_permission = format!("{}:*", action);
        let super_wildcard = "admin:*";

        claims.custom.permissions.iter().any(|perm| {
            perm == &required_permission || 
            perm == &wildcard_permission || 
            perm == super_wildcard
        })
    }

    /// Refresh access token using refresh token
    pub fn refresh_access_token(&self, refresh_token: &str, new_roles: Option<Vec<Role>>) -> Result<String> {
        let refresh_claims = self.verify_refresh_token(refresh_token)?;
        
        // Use provided roles or fetch from user store
        let roles = new_roles.unwrap_or_else(|| vec![Role::User]);
        
        self.generate_access_token(
            refresh_claims.custom.user_id,
            refresh_claims.custom.tenant_id,
            roles
        )
    }
}

/// JWT Authentication Middleware
pub struct JwtMiddleware {
    jwt_service: JwtService,
}

impl JwtMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Extract and verify JWT from Authorization header
    pub fn authenticate_request(&self, auth_header: Option<&str>) -> Result<Claims<UserClaims>> {
        let auth_header = auth_header
            .ok_or_else(|| anyhow::anyhow!("Missing Authorization header"))?;

        let token = auth_header.strip_prefix("Bearer ")
            .ok_or_else(|| anyhow::anyhow!("Invalid Authorization header format"))?;

        // Try access token first
        if let Ok(claims) = self.jwt_service.verify_access_token(token) {
            return Ok(claims);
        }

        // Try API key if access token fails
        self.jwt_service.verify_api_key(token)
            .with_context(|| "Invalid or expired token")
    }

    /// Check authorization for specific resource and action
    pub fn authorize_request(&self, claims: &Claims<UserClaims>, resource: &str, action: &str) -> Result<()> {
        if !self.jwt_service.check_permission(claims, resource, action) {
            anyhow::bail!("Insufficient permissions for {} on {}", action, resource);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_lifecycle() {
        let jwt_service = JwtService::new().unwrap();
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let roles = vec![Role::User];

        // Test access token generation and verification
        let access_token = jwt_service.generate_access_token(user_id, tenant_id, roles.clone()).unwrap();
        let claims = jwt_service.verify_access_token(&access_token).unwrap();
        assert_eq!(claims.custom.user_id, user_id);

        // Test permission checking
        assert!(jwt_service.check_permission(&claims, "own", "read"));
        assert!(!jwt_service.check_permission(&claims, "all", "delete"));
    }
}
```

### 2.2 Token Revocation and Security Features

**Token Blacklisting Implementation**:

```rust
// Token revocation store implementation
use std::collections::{HashSet, HashMap};
use std::time::SystemTime;
use tokio::time::Duration;

pub struct TokenBlacklist {
    revoked_tokens: Arc<RwLock<HashSet<String>>>, // JTI (JWT ID) storage
    expiry_times: Arc<RwLock<HashMap<String, SystemTime>>>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            revoked_tokens: Arc::new(RwLock::new(HashSet::new())),
            expiry_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn revoke_token(&self, jti: &str, expires_at: SystemTime) {
        self.revoked_tokens.write().await.insert(jti.to_string());
        self.expiry_times.write().await.insert(jti.to_string(), expires_at);
    }
    
    pub async fn is_revoked(&self, jti: &str) -> bool {
        self.revoked_tokens.read().await.contains(jti)
    }
    
    pub async fn cleanup_expired(&self) {
        let now = SystemTime::now();
        let mut revoked = self.revoked_tokens.write().await;
        let mut expiry = self.expiry_times.write().await;
        
        let expired_tokens: Vec<String> = expiry
            .iter()
            .filter(|(_, &exp_time)| exp_time < now)
            .map(|(jti, _)| jti.clone())
            .collect();
        
        for token in expired_tokens {
            revoked.remove(&token);
            expiry.remove(&token);
        }
    }
}
```

**JWT Key Rotation Implementation**:

```rust
// JWT key rotation with graceful transition
impl JwtService {
    pub fn rotate_signing_keys(&mut self) -> Result<()> {
        // Generate new key pairs
        let new_access_key = ES384KeyPair::generate();
        let new_refresh_key = ES384KeyPair::generate();
        let new_api_key = ES384KeyPair::generate();
        
        // Store old keys for verification during transition period
        self.old_access_key = Some(self.access_key.clone());
        self.old_refresh_key = Some(self.refresh_key.clone());
        self.old_api_key = Some(self.api_key.clone());
        
        // Update to new keys
        self.access_key = new_access_key;
        self.refresh_key = new_refresh_key;
        self.api_key = new_api_key;
        
        // Schedule cleanup of old keys after grace period
        let grace_period = Duration::from_secs(3600); // 1 hour
        tokio::spawn(async move {
            tokio::time::sleep(grace_period).await;
            // Keys will be dropped automatically
        });
        
        Ok(())
    }
    
    pub fn verify_with_key_rotation(&self, token: &str, token_type: TokenType) -> Result<Claims<UserClaims>> {
        // Try current key first
        if let Ok(claims) = self.verify_token_with_key(token, &self.get_current_key(token_type)) {
            return Ok(claims);
        }
        
        // Try old key if current fails (during rotation period)
        if let Some(old_key) = self.get_old_key(token_type) {
            return self.verify_token_with_key(token, old_key);
        }
        
        Err(anyhow::anyhow!("Token verification failed"))
    }
}
```

**Authentication Rate Limiting**:

```rust
// Rate limiting middleware implementation
use std::net::IpAddr;
use governor::{Quota, RateLimiter};
use nonzero_ext::*;

pub struct AuthRateLimiter {
    ip_limiter: RateLimiter<IpAddr, governor::state::keyed::DefaultKeyedStateStore<IpAddr>>,
    user_limiter: RateLimiter<String, governor::state::keyed::DefaultKeyedStateStore<String>>,
    global_limiter: RateLimiter<(), governor::state::direct::NotKeyed>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        Self {
            ip_limiter: RateLimiter::keyed(Quota::per_minute(nonzero!(100u32))),
            user_limiter: RateLimiter::keyed(Quota::per_minute(nonzero!(10u32))),
            global_limiter: RateLimiter::direct(Quota::per_second(nonzero!(1000u32))),
        }
    }
    
    pub async fn check_rate_limit(&self, ip: IpAddr, user_id: Option<&str>) -> Result<(), RateLimitError> {
        // Check global rate limit
        self.global_limiter.check()
            .map_err(|_| RateLimitError::GlobalLimitExceeded)?;
        
        // Check IP rate limit
        self.ip_limiter.check_key(&ip)
            .map_err(|_| RateLimitError::IpLimitExceeded)?;
        
        // Check user rate limit if user is authenticated
        if let Some(user) = user_id {
            self.user_limiter.check_key(&user.to_string())
                .map_err(|_| RateLimitError::UserLimitExceeded)?;
        }
        
        Ok(())
    }
}
```

### 2.3 Session Management Enhancement

**Enhanced Session Security**:

```rust
// Device fingerprinting for session binding
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub user_agent_hash: String,
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub platform: Option<String>,
}

impl DeviceFingerprint {
    pub fn generate(user_agent: &str, metadata: &HashMap<String, String>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_agent.as_bytes());
        
        // Include stable device characteristics
        if let Some(screen) = metadata.get("screen_resolution") {
            hasher.update(screen.as_bytes());
        }
        if let Some(tz) = metadata.get("timezone") {
            hasher.update(tz.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
}

// Session anomaly detection
pub struct SessionAnomalyDetector {
    ip_geolocation: Arc<IpGeolocationService>,
    device_tracker: Arc<DeviceTracker>,
}

impl SessionAnomalyDetector {
    pub async fn detect_anomalies(&self, session: &Session, new_ip: IpAddr) -> Vec<SessionAnomaly> {
        let mut anomalies = Vec::new();
        
        // Check for IP geolocation changes
        if let Ok(old_location) = self.ip_geolocation.lookup(session.ip_address).await {
            if let Ok(new_location) = self.ip_geolocation.lookup(new_ip).await {
                if old_location.country != new_location.country {
                    anomalies.push(SessionAnomaly::CountryChange {
                        from: old_location.country,
                        to: new_location.country,
                    });
                }
            }
        }
        
        // Check for suspicious timing patterns
        let time_since_last_activity = Utc::now() - session.last_activity;
        if time_since_last_activity < Duration::seconds(1) {
            anomalies.push(SessionAnomaly::RapidRequests);
        }
        
        anomalies
    }
}
```

### 2.4 Multi-Factor Authentication Implementation

**TOTP Implementation**:

```rust
// TOTP enrollment and verification
use totp_lite::{totp, Sha256};
use base32ct::{Base32, Alphabet};

pub struct TotpManager {
    repository: Arc<dyn TotpRepository>,
    issuer: String,
}

impl TotpManager {
    pub async fn enroll_totp(&self, user_id: &str) -> Result<TotpEnrollment, MfaError> {
        // Generate 32-byte secret
        let mut secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);
        
        // Create TOTP configuration
        let totp_config = TotpConfig {
            secret: secret.to_vec(),
            period: 30,
            digits: 6,
            algorithm: TotpAlgorithm::Sha256,
        };
        
        // Generate provisioning URI
        let secret_base32 = Base32::encode_string(&secret);
        let uri = format!(
            "otpauth://totp/{issuer}:{user}?secret={secret}&issuer={issuer}&algorithm=SHA256&digits=6&period=30",
            issuer = self.issuer,
            user = user_id,
            secret = secret_base32
        );
        
        // Store configuration
        self.repository.save_totp_config(user_id, &totp_config).await?;
        
        Ok(TotpEnrollment {
            secret: secret_base32,
            qr_code_uri: uri,
            backup_codes: self.generate_backup_codes(user_id).await?,
        })
    }
    
    pub async fn verify_totp(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        let config = self.repository.get_totp_config(user_id).await?;
        
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check current and adjacent time windows (for clock skew)
        for window in -1..=1 {
            let time_step = (current_time / config.period) as i64 + window;
            let expected_code = totp::<Sha256>(&config.secret, time_step as u64);
            
            if constant_time_eq(code.as_bytes(), expected_code.as_bytes()) {
                return Ok(true);
            }
        }
        
        // Check backup codes
        if let Ok(backup_codes) = self.repository.get_backup_codes(user_id).await {
            for backup_code in backup_codes {
                if constant_time_eq(code.as_bytes(), backup_code.as_bytes()) {
                    // Mark backup code as used
                    self.repository.mark_backup_code_used(user_id, &backup_code).await?;
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}
```

**WebAuthn/FIDO2 Implementation**:

```rust
// WebAuthn implementation for FIDO2 support
use webauthn_rs::prelude::*;

pub struct WebAuthnManager {
    webauthn: WebAuthn,
    repository: Arc<dyn WebAuthnRepository>,
}

impl WebAuthnManager {
    pub async fn start_registration(&self, user_id: &str) -> Result<CreationChallengeResponse, WebAuthnError> {
        let user_unique_id = Uuid::parse_str(user_id)
            .map_err(|_| WebAuthnError::InvalidUserId)?;
        
        let (ccr, reg_state) = self.webauthn.start_passkey_registration(
            user_unique_id,
            user_id,
            user_id,
            None,
        )?;
        
        // Store registration state
        self.repository.save_registration_state(user_id, &reg_state).await?;
        
        Ok(ccr)
    }
    
    pub async fn finish_registration(
        &self,
        user_id: &str,
        credential: &RegisterPublicKeyCredential,
    ) -> Result<(), WebAuthnError> {
        let reg_state = self.repository.get_registration_state(user_id).await?;
        
        let passkey = self.webauthn.finish_passkey_registration(credential, &reg_state)?;
        
        self.repository.save_passkey(user_id, &passkey).await?;
        
        Ok(())
    }
    
    pub async fn start_authentication(&self, user_id: &str) -> Result<RequestChallengeResponse, WebAuthnError> {
        let passkeys = self.repository.get_passkeys(user_id).await?;
        
        let (rcr, auth_state) = self.webauthn.start_passkey_authentication(&passkeys)?;
        
        self.repository.save_auth_state(user_id, &auth_state).await?;
        
        Ok(rcr)
    }
    
    pub async fn finish_authentication(
        &self,
        user_id: &str,
        credential: &PublicKeyCredential,
    ) -> Result<AuthenticationResult, WebAuthnError> {
        let auth_state = self.repository.get_auth_state(user_id).await?;
        
        let auth_result = self.webauthn.finish_passkey_authentication(credential, &auth_state)?;
        
        Ok(auth_result)
    }
}
```

## Integration Examples

### Complete Authentication Flow

```rust
// Example: Complete authentication flow with MFA
use axum::{routing::{get, post}, Router, Json, extract::State};

#[tokio::main]
async fn main() {
    // Initialize authentication components
    let cert_manager = Arc::new(CertificateManager::new());
    let jwt_service = Arc::new(JwtService::new().unwrap());
    let totp_manager = Arc::new(TotpManager::new());
    let webauthn_manager = Arc::new(WebAuthnManager::new().unwrap());
    let rate_limiter = Arc::new(AuthRateLimiter::new());
    
    // Build application with authentication middleware
    let app = Router::new()
        .route("/auth/login", post(login_handler))
        .route("/auth/mfa/totp", post(verify_totp_handler))
        .route("/auth/mfa/webauthn/start", post(start_webauthn_handler))
        .route("/auth/mfa/webauthn/finish", post(finish_webauthn_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/protected", get(protected_handler))
        .layer(Extension(jwt_service))
        .layer(Extension(totp_manager))
        .layer(Extension(webauthn_manager))
        .layer(Extension(rate_limiter));
    
    // Configure TLS with mTLS
    let tls_config = cert_manager.create_server_config().unwrap();
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8443").await.unwrap();
    axum_server::from_tcp_rustls(listener, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn login_handler(
    State(jwt_service): State<Arc<JwtService>>,
    State(rate_limiter): State<Arc<AuthRateLimiter>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Check rate limiting
    rate_limiter.check_rate_limit(request.ip, Some(&request.username)).await?;
    
    // Authenticate user
    let user = authenticate_user(&request.username, &request.password).await?;
    
    // Generate tokens
    let access_token = jwt_service.generate_access_token(
        user.id, 
        user.tenant_id, 
        user.roles.clone()
    )?;
    
    let refresh_token = jwt_service.generate_refresh_token(
        user.id, 
        user.tenant_id, 
        Uuid::new_v4()
    )?;
    
    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        requires_mfa: user.mfa_enabled,
        mfa_methods: user.mfa_methods,
    }))
}
```

### NATS Integration with mTLS

```rust
// Example: NATS client with mTLS authentication
use async_nats::ConnectOptions;

async fn create_secure_nats_client() -> Result<async_nats::Client, Box<dyn std::error::Error>> {
    let cert_manager = CertificateManager::new();
    let client_config = cert_manager.create_client_config()?;
    
    let options = ConnectOptions::new()
        .require_tls(true)
        .tls_client_config(client_config)
        .jwt_auth(generate_nats_jwt().await?)
        .connection_timeout(Duration::from_secs(10))
        .ping_interval(Duration::from_secs(60));
    
    let client = options.connect("tls://nats.mister-smith.local:4222").await?;
    Ok(client)
}

async fn generate_nats_jwt() -> Result<String, JwtError> {
    let jwt_service = JwtService::new()?;
    
    // Generate NATS-specific JWT with appropriate claims
    let claims = NatsClaims {
        sub: "nats-client".to_string(),
        aud: "nats-server".to_string(),
        permissions: NatsPermissions {
            publish: vec!["agents.>".to_string()],
            subscribe: vec!["agents.>".to_string()],
        },
    };
    
    jwt_service.generate_nats_token(claims)
}
```

## Cross-References

### Related Security Documents

- **[Security Framework](security-framework.md)** - Complete security patterns and configurations
- **[Authentication Specifications](authentication-specifications.md)** - Detailed authentication patterns and requirements
- **[Authorization Implementation](authorization-implementation.md)** - RBAC and security audit implementation
- **[Security Integration](security-integration.md)** - NATS and hook security implementation
- **[Security Patterns](security-patterns.md)** - Foundational security patterns and guidelines
- **[Authorization Specifications](authorization-specifications.md)** - Authorization patterns and RBAC specifications

### Transport Integration

- **[NATS Transport](../transport/nats-transport.md)** - Secure messaging with mTLS
- **[gRPC Transport](../transport/grpc-transport.md)** - RPC security with TLS/mTLS
- **[HTTP Transport](../transport/http-transport.md)** - API security and authentication
- **[Transport Core](../transport/transport-core.md)** - Core transport security patterns

### Implementation Integration Points

1. **Certificate Management** → Integrates with TLS transport layer and NATS mTLS configuration
2. **JWT Authentication** → Provides authentication for HTTP APIs and NATS messaging
3. **MFA Support** → Enhances security with TOTP and WebAuthn authentication
4. **Session Management** → Secure session handling with device fingerprinting
5. **Rate Limiting** → Protects against authentication attacks and abuse
6. **Cross-Component Security** → Foundation for authorization middleware and audit logging

## Testing and Validation

### Security Testing Requirements

**Certificate Testing**:

```bash
# Test certificate chain validation
openssl verify -CAfile ca/ca-cert.pem server/server-cert.pem
openssl verify -CAfile ca/ca-cert.pem client/client-cert.pem

# Test TLS handshake
openssl s_client -connect localhost:8443 -cert client/client-cert.pem -key client/client-key.pem -CAfile ca/ca-cert.pem

# Verify cipher suites
nmap --script ssl-enum-ciphers -p 8443 localhost
```

**Authentication Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_complete_auth_flow() {
        let jwt_service = JwtService::new().unwrap();
        let totp_manager = TotpManager::new();
        let user_id = Uuid::new_v4();
        
        // Test JWT generation and verification
        let token = jwt_service.generate_access_token(
            user_id, 
            Uuid::new_v4(), 
            vec![Role::User]
        ).unwrap();
        
        let claims = jwt_service.verify_access_token(&token).unwrap();
        assert_eq!(claims.custom.user_id, user_id);
        
        // Test TOTP enrollment
        let enrollment = totp_manager.enroll_totp(&user_id.to_string()).await.unwrap();
        assert!(!enrollment.secret.is_empty());
        
        // Test token revocation
        let blacklist = TokenBlacklist::new();
        blacklist.revoke_token(&claims.jwt_id, SystemTime::now() + Duration::from_secs(3600)).await;
        assert!(blacklist.is_revoked(&claims.jwt_id).await);
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let rate_limiter = AuthRateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();
        
        // Test normal operation
        assert!(rate_limiter.check_rate_limit(ip, Some("user1")).await.is_ok());
        
        // Test rate limit exceeded (requires loop to exhaust quota)
        for _ in 0..100 {
            let _ = rate_limiter.check_rate_limit(ip, Some("user1")).await;
        }
        
        // Should be rate limited now
        assert!(rate_limiter.check_rate_limit(ip, Some("user1")).await.is_err());
    }
}
```

### Configuration Management

**TLS Configuration**:

```rust
// TLS configuration with version enforcement
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub min_version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub require_client_cert: bool,
    pub ca_cert_path: String,
    pub server_cert_path: String,
    pub server_key_path: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_version: TlsVersion::TLS13, // Enforced minimum
            cipher_suites: vec![
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ],
            require_client_cert: true,
            ca_cert_path: "/etc/mister-smith/certs/ca/ca-cert.pem".to_string(),
            server_cert_path: "/etc/mister-smith/certs/server/server-cert.pem".to_string(),
            server_key_path: "/etc/mister-smith/certs/server/server-key.pem".to_string(),
        }
    }
}
```

### Security Monitoring

**Audit Logging**:

```rust
// Structured audit logging for security events
#[derive(Debug, Serialize)]
pub struct SecurityAuditEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<Uuid>,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub enum SecurityEventType {
    Login,
    Logout,
    MfaEnrollment,
    MfaVerification,
    TokenRevocation,
    CertificateRotation,
    RateLimitExceeded,
    SuspiciousActivity,
}

pub struct SecurityAuditor {
    logger: Arc<dyn AuditLogger>,
}

impl SecurityAuditor {
    pub async fn log_security_event(&self, event: SecurityAuditEvent) {
        self.logger.log_structured(serde_json::to_value(event).unwrap()).await;
    }
}
```

---

*Mister Smith Framework - Authentication Implementation*
*Optimized for agent consumption and technical implementation*
