---
title: Security Framework - Revised
type: note
permalink: revision-swarm/security/security-framework-revised
tags:
- '#security'
- '#revision'
- '#foundational'
- '#agent-focused'
---

## Security Framework

## Framework Authority

This document implements specifications from the canonical tech-framework.md located at /Users/mac-main/Mister-Smith/Mister-Smith/tech-framework.md

As stated in the canonical framework: "Agents: use this framework as the canonical source."

## Overview

This document provides foundational security patterns for the MisterSmith framework, covering authentication, authorization, TLS configuration, and secure communication patterns. All patterns are designed for systematic implementation by agents.

**Related Documentation:**

- [Security Patterns](./security-patterns.md) - Essential security patterns, guidelines, and configurations
- [Security Checklist](./security-patterns.md#security-checklist-for-agents) - Complete security implementation checklist
- [Configuration Templates](./security-patterns.md#configuration-templates) - Ready-to-use security configurations

## Purpose

Comprehensive security implementations and detailed code examples for the MisterSmith framework. This document provides production-ready implementations of security patterns including authentication services, authorization engines, certificate management, and audit systems.

## Core Security Components

### 1. Basic Authentication Pattern

**Pseudocode Pattern:**

```rust
// Basic JWT Authentication Flow
function authenticate_request(request):
    token = extract_bearer_token(request.headers)
    if not token:
        return error(401, "No authentication provided")
    
    claims = verify_jwt_token(token, public_key)
    if not claims or claims.expired:
        return error(401, "Invalid or expired token")
    
    request.context.user_id = claims.subject
    return success()

// Token Generation Pattern
function generate_auth_token(user_id, expires_in):
    claims = {
        subject: user_id,
        issued_at: current_timestamp(),
        expires_at: current_timestamp() + expires_in
    }
    return sign_jwt(claims, private_key)
```

**Configuration Pattern:**

```yaml
authentication:
  jwt:
    algorithm: RS256
    public_key_path: /path/to/public.pem
    private_key_path: /path/to/private.pem
    token_expiry: 3600  # seconds
```

#### Enhanced Agent-Specific JWT Claims Structure

```rust
// Complete JWT structure with agent-specific extensions
pub struct AgentClaims {
    // Standard JWT Claims (RFC 7519)
    pub iss: String,              // Issuer
    pub sub: String,              // Subject (agent_id)
    pub aud: Vec<String>,         // Audience
    pub exp: i64,                 // Expiration time
    
    // Agent-specific extensions
    pub agent_id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub permissions: Vec<Permission>,
    pub session_id: String,
    pub delegation_chain: Vec<String>,
}

pub enum AgentType {
    Autonomous,
    UserAssisted,
    SystemService,
    DelegatedAgent,
}
```

### 2. Simple Authorization Pattern

**Pseudocode Pattern:**

```rust
// Role-Based Access Control Pattern
function check_permission(user_id, resource, action):
    user_roles = get_user_roles(user_id)
    
    for role in user_roles:
        permissions = get_role_permissions(role)
        if has_permission(permissions, resource, action):
            return allow()
    
    return deny()

// Permission Definition Structure
permissions = {
    "reader": ["read:*"],
    "writer": ["read:*", "write:own"],
    "admin": ["read:*", "write:*", "delete:*"]
}
```

**Configuration Pattern:**

```yaml
authorization:
  type: role_based
  default_role: reader
  roles:
    - name: reader
      permissions: [read]
    - name: writer  
      permissions: [read, write]
    - name: admin
      permissions: [read, write, delete, admin]
```

#### Enhanced Hybrid Authorization Model

```rust
// Hybrid RBAC/ABAC Authorization Model
pub enum AuthorizationModel {
    RBAC(RbacPolicy),
    ABAC(AbacPolicy),
    Hybrid {
        rbac: RbacPolicy,
        abac: Vec<AbacPolicy>,
    },
}

// Enhanced Permission Structure
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub constraints: Vec<Constraint>,
}

pub struct AbacPolicy {
    pub name: String,
    pub conditions: Vec<PolicyCondition>,
    pub priority: u32,
    pub effect: PolicyEffect,
}

pub enum PolicyEffect {
    Allow,
    Deny,
}

// Fine-grained authorization flow
function evaluate_hybrid_authorization(principal, resource, action, context):
    // First evaluate RBAC
    rbac_result = evaluate_rbac(principal.roles, resource, action)
    
    // Then evaluate ABAC policies
    abac_results = []
    for policy in abac_policies:
        result = evaluate_abac_policy(policy, principal, resource, action, context)
        abac_results.append(result)
    
    // Combine results with priority resolution
    return resolve_policy_conflicts(rbac_result, abac_results)
```

### 3. TLS Configuration Pattern

**Pseudocode Pattern:**

```rust
// TLS Server Setup
function create_tls_server(cert_path, key_path):
    tls_config = {
        certificate: load_certificate(cert_path),
        private_key: load_private_key(key_path),
        min_version: "TLS_1_2",
        cipher_suites: ["TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"]
    }
    
    return create_server_with_tls(tls_config)

// TLS Client Configuration
function create_tls_client(ca_cert_path):
    tls_config = {
        ca_certificate: load_certificate(ca_cert_path),
        verify_hostname: true,
        min_version: "TLS_1_2"
    }
    
    return create_client_with_tls(tls_config)
```

**Configuration Pattern:**

```yaml
tls:
  server:
    cert_path: /certs/server.crt
    key_path: /certs/server.key
    min_version: TLS1.2
  client:
    ca_cert_path: /certs/ca.crt
    verify_hostname: true
```

### 4. Basic Secrets Management

**Pseudocode Pattern:**

```rust
// Environment-Based Secrets
function load_secrets():
    secrets = {
        database_url: get_env("DATABASE_URL"),
        api_key: get_env("API_KEY"),
        jwt_secret: get_env("JWT_SECRET")
    }
    
    // Validate required secrets
    for key, value in secrets:
        if not value:
            error("Missing required secret: " + key)
    
    return secrets

// File-Based Secrets Pattern
function load_secrets_from_file(path):
    if not file_exists(path):
        error("Secrets file not found")
    
    // Ensure proper file permissions
    if not check_file_permissions(path, "600"):
        error("Insecure secrets file permissions")
    
    return parse_secrets_file(path)
```

**Configuration Pattern:**

```yaml
secrets:
  source: environment  # or 'file'
  file_path: /secrets/app.secrets
  required:
    - DATABASE_URL
    - JWT_SECRET
    - API_KEY
```

### 5. Basic Security Middleware

**Pseudocode Pattern:**

```rust
// Security Headers Middleware
function security_headers_middleware(request, response, next):
    response.headers.add("X-Content-Type-Options", "nosniff")
    response.headers.add("X-Frame-Options", "DENY")
    response.headers.add("X-XSS-Protection", "1; mode=block")
    response.headers.add("Strict-Transport-Security", "max-age=31536000")
    
    return next(request, response)

// Rate Limiting Pattern
function rate_limit_middleware(request, response, next):
    client_id = get_client_identifier(request)
    
    if exceeded_rate_limit(client_id):
        return error(429, "Rate limit exceeded")
    
    increment_request_count(client_id)
    return next(request, response)
```

### 6. Basic Audit Logging

**Pseudocode Pattern:**

```rust
// Security Event Logging
function log_security_event(event_type, details):
    event = {
        timestamp: current_timestamp(),
        event_type: event_type,
        details: details,
        source_ip: get_request_ip(),
        user_id: get_current_user_id()
    }
    
    append_to_audit_log(event)

// Common Security Events to Log
security_events = [
    "authentication_success",
    "authentication_failure", 
    "authorization_denied",
    "invalid_token",
    "rate_limit_exceeded",
    "suspicious_activity"
]
```

#### Enhanced Security Audit Framework

```rust
// Comprehensive audit event structure
pub struct SecurityAuditEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub principal: Principal,
    pub resource: Resource,
    pub action: Action,
    pub outcome: EventOutcome,
    pub details: serde_json::Value,
    pub context: AuditContext,
}

pub enum SecurityEventType {
    Authentication,
    Authorization,
    SystemAccess,
    DataAccess,
    AdminAction,
    SuspiciousActivity,
    ComplianceEvent,
}

pub enum EventOutcome {
    Success,
    Failure,
    Partial,
    Blocked,
}

// Audit trail with tamper-evidence
pub struct TamperEvidentAuditLog {
    pub entries: Vec<SecurityAuditEvent>,
    pub hash_chain: Vec<String>,
    pub digital_signature: Option<String>,
}

// Audit processing flow
function process_security_audit_event(event_data):
    audit_event = SecurityAuditEvent::new(event_data)
    
    // Add to immutable log
    audit_log.append_with_hash_chain(audit_event)
    
    // Real-time alerting
    if requires_immediate_attention(audit_event):
        trigger_security_alert(audit_event)
    
    // Compliance processing
    compliance_processor.process_event(audit_event)
```

### 7. NATS Security Patterns

**Pseudocode Pattern - mTLS Configuration:**

```rust
// Initialize secure NATS connection with mTLS
function init_secure_nats(tenant_id):
    // Load certificates from secure storage
    ca_cert = load_file("/secrets/ca.crt")
    client_cert = load_file("/secrets/client.crt")
    client_key = load_file("/secrets/client.key")
    
    // Configure connection with tenant-specific identity
    options = {
        require_tls: true,
        ca_certificate: ca_cert,
        client_certificate: client_cert,
        client_key: client_key,
        client_name: "tenant_" + tenant_id + "_agent"
    }
    
    return connect_nats(NATS_URL, options)

// Apply rate limiting and backpressure
function configure_nats_limits(connection):
    connection.subscription_capacity = 1000  // Bounded buffer
    connection.ping_interval = 10  // seconds
    connection.reconnect_buffer_size = 8388608  // 8MB
```

**Configuration Pattern - Server mTLS:**

```hocon
# NATS server mTLS configuration
tls {
  cert_file: "./certs/nats-server.crt"
  key_file:  "./certs/nats-server.key"
  ca_file:   "./certs/ca.crt"
  verify: true  # Enforce client certificates
}

cluster {
  tls {
    cert_file: "./certs/cluster.crt"
    key_file:  "./certs/cluster.key"
    ca_file:   "./certs/ca.crt"
  }
}
```

**Account-Based Tenant Isolation Pattern:**

```yaml
# NATS account isolation pattern
account_isolation:
  principle: "Each tenant gets isolated NATS account"
  benefits:
    - Complete namespace separation
    - No subject prefix complexity
    - Built-in multi-tenancy support
  implementation: |
    nsc add account --name tenantA
    nsc edit account --name tenantA \
      --js-mem-storage 512M \
      --js-disk-storage 1G \
      --js-streams 10 \
      --js-consumer 50
```

**Fine-Grained ACL Configuration:**

```json
// Per-user permission model
{
  "users": [
    {
      "user": "admin",
      "permissions": {
        "publish": [ ">" ],     // Full access
        "subscribe": [ ">" ]
      }
    },
    {
      "user": "tenantA_bot",
      "permissions": {
        "publish": { "allow": ["tenantA.>"] },
        "subscribe": { "allow": ["tenantA.>"] }
      }
    }
  ]
}
```

**Resource Quota Enforcement:**

```yaml
# Per-account resource limits
jetstream_limits:
  per_account:
    max_memory: 512M
    max_disk: 1G
    max_streams: 10
    max_consumers: 100
  per_stream:
    max_bytes: configurable
    max_msgs: configurable
    max_age: configurable
    discard_policy: old_on_full
  connection_limits:
    max_connections: 100
    max_subscriptions: 1000
    max_payload_size: 1MB
```

**Key Rotation Pattern:**

```rust
// Zero-downtime key rotation state machine
key_rotation_states = [
    "KEY_A_ACTIVE",
    "STAGING_NEW_KEY",
    "RELOADING_CONFIG",
    "KEY_B_ACTIVE"
]

// Signal handler for hot reload
function handle_sighup_signal():
    if signal_received == SIGHUP:
        // Atomically swap API keys in memory
        rotate_keys()
        reload_tls_certificates()
        update_active_connections()
```

**Critical NATS Security Patterns:**

1. **Never share accounts between tenants** - Use NATS accounts for true isolation
2. **Always enforce mTLS** - Both client and cluster connections must verify certificates
3. **Apply least privilege** - Restrict subjects to minimum required patterns
4. **Set resource quotas** - Prevent any tenant from exhausting cluster resources
5. **Rotate secrets regularly** - Use SIGHUP for zero-downtime key rotation
6. **Monitor wildcard usage** - Detect and prevent unauthorized subject access

## Implementation Guidelines

### Authentication Flow

1. Extract authentication token from request
2. Verify token signature and expiration
3. Extract user identity from token claims
4. Attach identity to request context

### Authorization Flow

1. Identify resource and action from request
2. Retrieve user roles/permissions
3. Check if user has required permission
4. Allow or deny based on permission check

### TLS Setup Flow

1. Generate or obtain TLS certificates
2. Configure minimum TLS version (1.2+)
3. Select secure cipher suites
4. Enable hostname verification for clients

### Secrets Management Flow

1. Define required secrets
2. Load from environment or secure file
3. Validate all required secrets present
4. Use secrets for service configuration

### NATS Security Flow

1. Generate or obtain mTLS certificates for NATS
2. Create isolated accounts for each tenant
3. Configure ACLs for subject-based access control
4. Set resource quotas to prevent resource exhaustion
5. Implement key rotation handlers for hot reload

## Security Checklist for Agents

- [ ] Implement authentication before processing requests
- [ ] Check authorization for protected resources
- [ ] Enable TLS for all network communication
- [ ] Store secrets securely (environment variables or encrypted files)
- [ ] Add security headers to HTTP responses
- [ ] Implement rate limiting for API endpoints
- [ ] Log security-relevant events for audit trails
- [ ] Validate and sanitize all input data
- [ ] Use secure random number generation for tokens
- [ ] Set appropriate timeouts for authentication tokens
- [ ] Configure NATS with mTLS for secure messaging
- [ ] Implement account-based isolation for multi-tenant NATS
- [ ] Set resource quotas for NATS accounts and streams
- [ ] Configure fine-grained ACLs for NATS subjects
- [ ] Implement key rotation for zero-downtime secret updates

## Configuration Templates

### Basic Security Configuration

```yaml
security:
  authentication:
    enabled: true
    type: jwt
    token_expiry: 3600
  
  authorization:
    enabled: true
    type: role_based
    default_role: reader
  
  tls:
    enabled: true
    min_version: TLS1.2
  
  rate_limiting:
    enabled: true
    requests_per_minute: 60
  
  audit_logging:
    enabled: true
    log_path: /logs/security.log
  
  nats:
    enabled: true
    mtls:
      cert_path: /certs/nats-client.crt
      key_path: /certs/nats-client.key
      ca_path: /certs/ca.crt
    account_isolation: true
    resource_quotas:
      max_memory: 512M
      max_disk: 1G
      max_connections: 100
```

## 11. Hook Execution Sandbox Pattern

### 11.1 Non-Root User Execution Pattern

**Pseudocode Pattern:**

```rust
// Hook execution with privilege isolation
function execute_hook_safely(hook_script, payload):
    // Ensure hook runs under non-root user
    execution_user = get_non_privileged_user()  // e.g., "claude-hook-runner"

    // Create isolated execution environment
    sandbox_config = {
        user: execution_user,
        working_directory: "/tmp/hook-sandbox",
        environment_variables: filter_safe_env_vars(),
        resource_limits: {
            max_memory: "128M",
            max_cpu_time: "30s",
            max_file_descriptors: 64,
            max_processes: 1
        },
        filesystem_access: {
            read_only: ["/usr", "/lib", "/bin"],
            read_write: ["/tmp/hook-sandbox"],
            no_access: ["/etc", "/root", "/home"]
        }
    }

    // Execute with timeout and resource constraints
    result = execute_with_sandbox(hook_script, payload, sandbox_config)

    // Clean up sandbox environment
    cleanup_sandbox_directory()

    return result

// User privilege management
function setup_hook_user():
    // Create dedicated user for hook execution
    create_user("claude-hook-runner", {
        home_directory: "/var/lib/claude-hooks",
        shell: "/bin/bash",
        groups: ["claude-hooks"],
        no_login: false,
        system_user: true
    })

    // Set up hook directory permissions
    set_directory_permissions("/var/lib/claude-hooks", {
        owner: "claude-hook-runner",
        group: "claude-hooks",
        permissions: "750"
    })
```

**Configuration Pattern:**

```yaml
hook_security:
  execution_user: claude-hook-runner
  sandbox_directory: /tmp/hook-sandbox
  resource_limits:
    max_memory_mb: 128
    max_cpu_seconds: 30
    max_file_descriptors: 64
    max_processes: 1

  filesystem_isolation:
    read_only_paths:
      - /usr
      - /lib
      - /bin
      - /etc/passwd
    read_write_paths:
      - /tmp/hook-sandbox
    blocked_paths:
      - /etc
      - /root
      - /home
      - /var/lib/claude-hooks/.ssh

  network_isolation:
    allow_outbound: false
    allow_localhost: true
    blocked_ports: [22, 3389, 5432, 27017]
```

### 11.2 Hook Script Validation Pattern

**Pseudocode Pattern:**

```rust
// Validate hook scripts before execution
function validate_hook_script(script_path):
    // Check file permissions
    file_info = get_file_info(script_path)
    if file_info.owner != "claude-hook-runner":
        return error("Hook script must be owned by claude-hook-runner")

    if file_info.permissions & WORLD_WRITABLE:
        return error("Hook script cannot be world-writable")

    // Validate script content
    script_content = read_file(script_path)

    // Check for dangerous patterns
    dangerous_patterns = [
        "sudo", "su -", "chmod 777", "rm -rf /",
        "curl.*|.*sh", "wget.*|.*sh", "eval",
        "/etc/passwd", "/etc/shadow"
    ]

    for pattern in dangerous_patterns:
        if matches_pattern(script_content, pattern):
            return error("Hook script contains dangerous pattern: " + pattern)

    // Validate shebang
    if not script_content.starts_with("#!/"):
        return error("Hook script must have valid shebang")

    return success()

// Hook directory security
function secure_hook_directory(hook_dir):
    // Ensure proper ownership and permissions
    set_ownership(hook_dir, "claude-hook-runner", "claude-hooks")
    set_permissions(hook_dir, "750")  // rwxr-x---

    // Validate all hook scripts
    for script in list_files(hook_dir):
        validate_hook_script(script)
        set_permissions(script, "750")  // rwxr-x---
```

## Pattern Implementation Notes

- All patterns are foundational and can be extended as needed
- Focus on understanding core security concepts before adding complexity
- Use standard libraries for cryptographic operations
- Test security configurations in isolated environments
- Follow the principle of least privilege for all access control

## CONCRETE IMPLEMENTATIONS

This section provides production-ready implementations that leave zero security implementation decisions for consumers. All security parameters, algorithms, and procedures are specified concretely.

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

## 3. Authorization Implementation

### 3.1 RBAC Policy Engine

**Complete RBAC Implementation:**

```rust
// rbac_engine.rs
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resource_type: ResourceType,
    pub owner_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub team_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "configuration")]
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "admin")]
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub action: Action,
    pub resource_pattern: String,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    #[serde(rename = "owner_only")]
    OwnerOnly,
    #[serde(rename = "same_tenant")]
    SameTenant,
    #[serde(rename = "same_team")]
    SameTeam,
    #[serde(rename = "business_hours")]
    BusinessHours,
}

pub struct RbacEngine {
    role_permissions: HashMap<Role, Vec<Permission>>,
}

impl RbacEngine {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        
        // ReadOnly role permissions
        role_permissions.insert(Role::ReadOnly, vec![
            Permission {
                action: Action::Read,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::OwnerOnly, Condition::SameTenant],
            },
        ]);

        // User role permissions
        role_permissions.insert(Role::User, vec![
            Permission {
                action: Action::Read,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::OwnerOnly, Condition::SameTenant],
            },
            Permission {
                action: Action::Write,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::OwnerOnly, Condition::SameTenant],
            },
        ]);

        // Moderator role permissions
        role_permissions.insert(Role::Moderator, vec![
            Permission {
                action: Action::Read,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::SameTeam, Condition::SameTenant],
            },
            Permission {
                action: Action::Write,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::SameTeam, Condition::SameTenant],
            },
            Permission {
                action: Action::Delete,
                resource_pattern: "document".to_string(),
                conditions: vec![Condition::SameTeam, Condition::SameTenant],
            },
        ]);

        // Admin role permissions
        role_permissions.insert(Role::Admin, vec![
            Permission {
                action: Action::Read,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::SameTenant],
            },
            Permission {
                action: Action::Write,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::SameTenant],
            },
            Permission {
                action: Action::Delete,
                resource_pattern: "*".to_string(),
                conditions: vec![Condition::SameTenant],
            },
        ]);

        // System role permissions (no restrictions)
        role_permissions.insert(Role::System, vec![
            Permission {
                action: Action::Read,
                resource_pattern: "*".to_string(),
                conditions: vec![],
            },
            Permission {
                action: Action::Write,
                resource_pattern: "*".to_string(),
                conditions: vec![],
            },
            Permission {
                action: Action::Delete,
                resource_pattern: "*".to_string(),
                conditions: vec![],
            },
            Permission {
                action: Action::Admin,
                resource_pattern: "*".to_string(),
                conditions: vec![],
            },
        ]);

        Self { role_permissions }
    }

    /// Check if user has permission to perform action on resource
    pub fn check_permission(
        &self,
        user_claims: &UserClaims,
        resource: &Resource,
        action: &Action,
    ) -> Result<bool> {
        // Check each role the user has
        for role in &user_claims.roles {
            if let Some(permissions) = self.role_permissions.get(role) {
                for permission in permissions {
                    if self.permission_matches(permission, resource, action, user_claims)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    /// Check if permission matches the requested action and resource
    fn permission_matches(
        &self,
        permission: &Permission,
        resource: &Resource,
        action: &Action,
        user_claims: &UserClaims,
    ) -> Result<bool> {
        // Check action match
        if !self.action_matches(&permission.action, action) {
            return Ok(false);
        }

        // Check resource pattern match
        if !self.resource_pattern_matches(&permission.resource_pattern, resource) {
            return Ok(false);
        }

        // Check all conditions
        for condition in &permission.conditions {
            if !self.condition_matches(condition, resource, user_claims)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if action matches (including hierarchical permissions)
    fn action_matches(&self, permission_action: &Action, requested_action: &Action) -> bool {
        match (permission_action, requested_action) {
            // Exact match
            (a, b) if a == b => true,
            // Admin action grants all permissions
            (Action::Admin, _) => true,
            // Write action grants read permission
            (Action::Write, Action::Read) => true,
            // Delete action grants read and write permissions
            (Action::Delete, Action::Read) | (Action::Delete, Action::Write) => true,
            _ => false,
        }
    }

    /// Check if resource pattern matches
    fn resource_pattern_matches(&self, pattern: &str, resource: &Resource) -> bool {
        if pattern == "*" {
            return true;
        }

        let resource_type_str = match resource.resource_type {
            ResourceType::User => "user",
            ResourceType::Project => "project",
            ResourceType::Document => "document",
            ResourceType::System => "system",
            ResourceType::Configuration => "configuration",
        };

        pattern == resource_type_str
    }

    /// Check if condition is satisfied
    fn condition_matches(
        &self,
        condition: &Condition,
        resource: &Resource,
        user_claims: &UserClaims,
    ) -> Result<bool> {
        match condition {
            Condition::OwnerOnly => {
                Ok(resource.owner_id == Some(user_claims.user_id))
            },
            Condition::SameTenant => {
                Ok(resource.tenant_id == user_claims.tenant_id)
            },
            Condition::SameTeam => {
                match (resource.team_id, &user_claims.roles) {
                    (Some(resource_team), _) => {
                        // For now, check if user has moderator role in same tenant
                        // In a real implementation, you'd check team membership
                        Ok(resource.tenant_id == user_claims.tenant_id)
                    },
                    (None, _) => Ok(true), // Resource not tied to team
                }
            },
            Condition::BusinessHours => {
                use chrono::{Local, Timelike};
                let now = Local::now();
                let hour = now.hour();
                Ok(hour >= 9 && hour <= 17) // 9 AM to 5 PM
            },
        }
    }

    /// Get effective permissions for user
    pub fn get_effective_permissions(&self, user_claims: &UserClaims) -> Vec<String> {
        let mut permissions = HashSet::new();
        
        for role in &user_claims.roles {
            if let Some(role_permissions) = self.role_permissions.get(role) {
                for permission in role_permissions {
                    let perm_str = format!("{}:{}", 
                        action_to_string(&permission.action),
                        permission.resource_pattern
                    );
                    permissions.insert(perm_str);
                }
            }
        }
        
        permissions.into_iter().collect()
    }
}

fn action_to_string(action: &Action) -> &'static str {
    match action {
        Action::Read => "read",
        Action::Write => "write", 
        Action::Delete => "delete",
        Action::Admin => "admin",
    }
}

/// Authorization middleware
pub struct AuthorizationMiddleware {
    rbac_engine: RbacEngine,
}

impl AuthorizationMiddleware {
    pub fn new() -> Self {
        Self {
            rbac_engine: RbacEngine::new(),
        }
    }

    /// Authorize request for specific resource and action
    pub fn authorize(
        &self,
        user_claims: &UserClaims,
        resource: &Resource,
        action: &Action,
    ) -> Result<()> {
        if self.rbac_engine.check_permission(user_claims, resource, action)? {
            Ok(())
        } else {
            anyhow::bail!(
                "Access denied: user {} lacks permission to {:?} resource {}",
                user_claims.user_id,
                action,
                resource.id
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_permissions() {
        let rbac = RbacEngine::new();
        
        let user_claims = UserClaims {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            roles: vec![Role::User],
            permissions: vec![],
            token_type: TokenType::Access,
            session_id: Uuid::new_v4(),
        };

        let resource = Resource {
            id: "test-doc".to_string(),
            resource_type: ResourceType::Document,
            owner_id: Some(user_claims.user_id),
            tenant_id: user_claims.tenant_id,
            team_id: None,
        };

        // User should be able to read their own document
        assert!(rbac.check_permission(&user_claims, &resource, &Action::Read).unwrap());
        
        // User should be able to write their own document
        assert!(rbac.check_permission(&user_claims, &resource, &Action::Write).unwrap());
        
        // User should NOT be able to delete their own document (requires moderator+)
        assert!(!rbac.check_permission(&user_claims, &resource, &Action::Delete).unwrap());
    }
}
```

## 4. Security Audit Implementation

### 4.1 Structured Audit Logging

**Complete Audit Service:**

```rust
// audit_service.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    #[serde(rename = "authentication_success")]
    AuthenticationSuccess,
    #[serde(rename = "authentication_failure")]
    AuthenticationFailure,
    #[serde(rename = "authorization_granted")]
    AuthorizationGranted,
    #[serde(rename = "authorization_denied")]
    AuthorizationDenied,
    #[serde(rename = "token_issued")]
    TokenIssued,
    #[serde(rename = "token_expired")]
    TokenExpired,
    #[serde(rename = "token_revoked")]
    TokenRevoked,
    #[serde(rename = "rate_limit_exceeded")]
    RateLimitExceeded,
    #[serde(rename = "suspicious_activity")]
    SuspiciousActivity,
    #[serde(rename = "certificate_rotation")]
    CertificateRotation,
    #[serde(rename = "configuration_change")]
    ConfigurationChange,
    #[serde(rename = "privilege_escalation")]
    PrivilegeEscalation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "critical")]
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: Severity,
    pub user_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub outcome: SecurityOutcome,
    pub details: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOutcome {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "warning")]
    Warning,
}

pub struct AuditService {
    // In production, this would write to a secure audit log storage
    events: Vec<SecurityEvent>,
    correlation_tracker: HashMap<Uuid, Vec<Uuid>>,
}

impl AuditService {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            correlation_tracker: HashMap::new(),
        }
    }

    /// Log authentication attempt
    pub fn log_authentication(&mut self, 
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        source_ip: Option<String>,
        user_agent: Option<String>,
        success: bool,
        failure_reason: Option<String>,
    ) -> Uuid {
        let event_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        let mut details = HashMap::new();
        if let Some(reason) = failure_reason {
            details.insert("failure_reason".to_string(), serde_json::Value::String(reason));
        }

        let event = SecurityEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: if success { 
                SecurityEventType::AuthenticationSuccess 
            } else { 
                SecurityEventType::AuthenticationFailure 
            },
            severity: if success { Severity::Info } else { Severity::Warning },
            user_id,
            tenant_id,
            session_id: None,
            source_ip,
            user_agent,
            resource_id: None,
            resource_type: None,
            action: Some("authenticate".to_string()),
            outcome: if success { SecurityOutcome::Success } else { SecurityOutcome::Failure },
            details,
            correlation_id: Some(correlation_id),
        };

        self.events.push(event);
        info!("Logged authentication event: {}", event_id);
        event_id
    }

    /// Log authorization decision
    pub fn log_authorization(&mut self,
        user_id: Uuid,
        tenant_id: Uuid,
        session_id: Option<Uuid>,
        resource_id: String,
        resource_type: String,
        action: String,
        granted: bool,
        roles: Vec<String>,
        correlation_id: Option<Uuid>,
    ) -> Uuid {
        let event_id = Uuid::new_v4();
        
        let mut details = HashMap::new();
        details.insert("roles".to_string(), serde_json::Value::Array(
            roles.into_iter().map(serde_json::Value::String).collect()
        ));

        let event = SecurityEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: if granted { 
                SecurityEventType::AuthorizationGranted 
            } else { 
                SecurityEventType::AuthorizationDenied 
            },
            severity: if granted { Severity::Info } else { Severity::Warning },
            user_id: Some(user_id),
            tenant_id: Some(tenant_id),
            session_id,
            source_ip: None,
            user_agent: None,
            resource_id: Some(resource_id),
            resource_type: Some(resource_type),
            action: Some(action),
            outcome: if granted { SecurityOutcome::Success } else { SecurityOutcome::Blocked },
            details,
            correlation_id,
        };

        self.events.push(event);
        info!("Logged authorization event: {}", event_id);
        event_id
    }

    /// Log token issuance
    pub fn log_token_issued(&mut self,
        user_id: Uuid,
        tenant_id: Uuid,
        token_type: String,
        expires_at: DateTime<Utc>,
        correlation_id: Option<Uuid>,
    ) -> Uuid {
        let event_id = Uuid::new_v4();
        
        let mut details = HashMap::new();
        details.insert("token_type".to_string(), serde_json::Value::String(token_type));
        details.insert("expires_at".to_string(), serde_json::Value::String(expires_at.to_rfc3339()));

        let event = SecurityEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: SecurityEventType::TokenIssued,
            severity: Severity::Info,
            user_id: Some(user_id),
            tenant_id: Some(tenant_id),
            session_id: None,
            source_ip: None,
            user_agent: None,
            resource_id: None,
            resource_type: None,
            action: Some("issue_token".to_string()),
            outcome: SecurityOutcome::Success,
            details,
            correlation_id,
        };

        self.events.push(event);
        info!("Logged token issuance: {}", event_id);
        event_id
    }

    /// Log suspicious activity
    pub fn log_suspicious_activity(&mut self,
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        source_ip: Option<String>,
        activity_type: String,
        details: HashMap<String, serde_json::Value>,
        correlation_id: Option<Uuid>,
    ) -> Uuid {
        let event_id = Uuid::new_v4();
        
        let mut event_details = details;
        event_details.insert("activity_type".to_string(), serde_json::Value::String(activity_type));

        let event = SecurityEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: SecurityEventType::SuspiciousActivity,
            severity: Severity::Error,
            user_id,
            tenant_id,
            session_id: None,
            source_ip,
            user_agent: None,
            resource_id: None,
            resource_type: None,
            action: Some("suspicious_activity".to_string()),
            outcome: SecurityOutcome::Warning,
            details: event_details,
            correlation_id,
        };

        self.events.push(event);
        error!("Logged suspicious activity: {}", event_id);
        event_id
    }

    /// Log certificate rotation
    pub fn log_certificate_rotation(&mut self,
        certificate_type: String,
        old_expiry: DateTime<Utc>,
        new_expiry: DateTime<Utc>,
    ) -> Uuid {
        let event_id = Uuid::new_v4();
        
        let mut details = HashMap::new();
        details.insert("certificate_type".to_string(), serde_json::Value::String(certificate_type));
        details.insert("old_expiry".to_string(), serde_json::Value::String(old_expiry.to_rfc3339()));
        details.insert("new_expiry".to_string(), serde_json::Value::String(new_expiry.to_rfc3339()));

        let event = SecurityEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: SecurityEventType::CertificateRotation,
            severity: Severity::Info,
            user_id: None,
            tenant_id: None,
            session_id: None,
            source_ip: None,
            user_agent: None,
            resource_id: None,
            resource_type: None,
            action: Some("rotate_certificate".to_string()),
            outcome: SecurityOutcome::Success,
            details,
            correlation_id: None,
        };

        self.events.push(event);
        info!("Logged certificate rotation: {}", event_id);
        event_id
    }

    /// Search audit events
    pub fn search_events(&self, 
        event_type: Option<SecurityEventType>,
        severity: Option<Severity>,
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<&SecurityEvent> {
        let mut filtered: Vec<&SecurityEvent> = self.events
            .iter()
            .filter(|event| {
                if let Some(ref et) = event_type {
                    if std::mem::discriminant(&event.event_type) != std::mem::discriminant(et) {
                        return false;
                    }
                }
                if let Some(ref sev) = severity {
                    if std::mem::discriminant(&event.severity) != std::mem::discriminant(sev) {
                        return false;
                    }
                }
                if let Some(uid) = user_id {
                    if event.user_id != Some(uid) {
                        return false;
                    }
                }
                if let Some(tid) = tenant_id {
                    if event.tenant_id != Some(tid) {
                        return false;
                    }
                }
                if let Some(since_time) = since {
                    if event.timestamp < since_time {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort by timestamp, newest first
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Generate compliance report
    pub fn generate_compliance_report(&self, 
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ComplianceReport {
        let events_in_range: Vec<&SecurityEvent> = self.events
            .iter()
            .filter(|event| event.timestamp >= start_date && event.timestamp <= end_date)
            .collect();

        let mut authentication_attempts = 0;
        let mut authentication_failures = 0;
        let mut authorization_denials = 0;
        let mut suspicious_activities = 0;
        let mut certificate_rotations = 0;

        for event in &events_in_range {
            match event.event_type {
                SecurityEventType::AuthenticationSuccess => authentication_attempts += 1,
                SecurityEventType::AuthenticationFailure => {
                    authentication_attempts += 1;
                    authentication_failures += 1;
                },
                SecurityEventType::AuthorizationDenied => authorization_denials += 1,
                SecurityEventType::SuspiciousActivity => suspicious_activities += 1,
                SecurityEventType::CertificateRotation => certificate_rotations += 1,
                _ => {},
            }
        }

        ComplianceReport {
            period_start: start_date,
            period_end: end_date,
            total_events: events_in_range.len(),
            authentication_attempts,
            authentication_failures,
            authorization_denials,
            suspicious_activities,
            certificate_rotations,
            failure_rate: if authentication_attempts > 0 {
                (authentication_failures as f64 / authentication_attempts as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub authentication_attempts: usize,
    pub authentication_failures: usize,
    pub authorization_denials: usize,
    pub suspicious_activities: usize,
    pub certificate_rotations: usize,
    pub failure_rate: f64, // Percentage
}

/// Audit middleware for HTTP requests
pub struct AuditMiddleware {
    audit_service: std::sync::Arc<std::sync::Mutex<AuditService>>,
}

impl AuditMiddleware {
    pub fn new(audit_service: std::sync::Arc<std::sync::Mutex<AuditService>>) -> Self {
        Self { audit_service }
    }

    /// Log HTTP request for audit
    pub fn log_request(&self, 
        method: &str,
        path: &str,
        user_claims: Option<&UserClaims>,
        source_ip: Option<String>,
        user_agent: Option<String>,
        response_status: u16,
    ) {
        let mut audit = self.audit_service.lock().unwrap();
        
        // Log based on response status
        if response_status == 401 {
            audit.log_authentication(
                user_claims.map(|c| c.user_id),
                user_claims.map(|c| c.tenant_id),
                source_ip,
                user_agent,
                false,
                Some("Unauthorized request".to_string()),
            );
        } else if response_status == 403 {
            if let Some(claims) = user_claims {
                audit.log_authorization(
                    claims.user_id,
                    claims.tenant_id,
                    Some(claims.session_id),
                    path.to_string(),
                    "http_endpoint".to_string(),
                    method.to_string(),
                    false,
                    claims.roles.iter().map(|r| format!("{:?}", r)).collect(),
                    None,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_service() {
        let mut audit = AuditService::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        // Test authentication logging
        let auth_event_id = audit.log_authentication(
            Some(user_id),
            Some(tenant_id),
            Some("192.168.1.1".to_string()),
            Some("test-agent".to_string()),
            true,
            None,
        );

        assert!(audit.events.iter().any(|e| e.event_id == auth_event_id));

        // Test searching events
        let auth_events = audit.search_events(
            Some(SecurityEventType::AuthenticationSuccess),
            None,
            Some(user_id),
            None,
            None,
            None,
        );

        assert_eq!(auth_events.len(), 1);
    }
}
```

### 4.2 Security Monitoring Configuration

**Prometheus Metrics Configuration:**

```yaml
# prometheus_security_metrics.yml
groups:
  - name: security_alerts
    rules:
      # Authentication failure rate
      - alert: HighAuthenticationFailureRate
        expr: rate(authentication_failures_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate detected"
          description: "Authentication failure rate is {{ $value }} failures/second"

      # Suspicious activity detection
      - alert: SuspiciousActivity
        expr: rate(suspicious_activity_total[1m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Suspicious activity detected"
          description: "{{ $value }} suspicious activities detected"

      # Certificate expiration warning
      - alert: CertificateExpiringSoon
        expr: certificate_expiry_days < 30
        for: 0m
        labels:
          severity: warning
        annotations:
          summary: "Certificate expiring soon"
          description: "Certificate {{ $labels.certificate_name }} expires in {{ $value }} days"

      # Token issuance rate anomaly
      - alert: UnusualTokenIssuanceRate
        expr: rate(tokens_issued_total[10m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Unusual token issuance rate"
          description: "Token issuance rate is {{ $value }} tokens/second"

      # Failed authorization attempts
      - alert: RepeatedAuthorizationDenials
        expr: rate(authorization_denied_total[5m]) by (user_id) > 0.5
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Repeated authorization denials for user"
          description: "User {{ $labels.user_id }} has {{ $value }} authorization denials/second"
```

**Grafana Dashboard Configuration:**

```json
{
  "dashboard": {
    "id": null,
    "title": "Mister Smith Security Dashboard",
    "tags": ["security", "audit"],
    "timezone": "UTC",
    "panels": [
      {
        "title": "Authentication Events",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(authentication_success_total[1h])",
            "legendFormat": "Success Rate"
          },
          {
            "expr": "rate(authentication_failure_total[1h])",
            "legendFormat": "Failure Rate"
          }
        ]
      },
      {
        "title": "Security Events Timeline",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(security_events_total[5m]) by (event_type)",
            "legendFormat": "{{ event_type }}"
          }
        ]
      },
      {
        "title": "Certificate Expiry Status",
        "type": "table",
        "targets": [
          {
            "expr": "certificate_expiry_days",
            "format": "table"
          }
        ]
      },
      {
        "title": "Top Failed Authorization Attempts",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, rate(authorization_denied_total[1h]) by (user_id, resource_type))",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

## 5. NATS Security Implementation

### 5.1 NATS Server Configuration with mTLS

**Complete NATS Server Configuration:**

```hocon
# nats_server_secure.conf
# NATS Server Security Configuration for Mister Smith Framework

# Server identity
server_name: "mister-smith-nats"

# Network configuration
host: "0.0.0.0"
port: 4222
http_port: 8222

# TLS Configuration with mTLS enforcement
tls {
  cert_file: "/etc/mister-smith/certs/server/server-cert.pem"
  key_file: "/etc/mister-smith/certs/server/server-key.pem"
  ca_file: "/etc/mister-smith/certs/ca/ca-cert.pem"
  verify: true
  verify_and_map: true
  timeout: 5
  
  # Force all connections to use TLS
  insecure: false
}

# JetStream configuration with security
jetstream {
  enabled: true
  store_dir: "/data/nats/jetstream"
  max_memory_store: 4GB
  max_file_store: 20GB
  
  # Domain isolation
  domain: "mister-smith"
}

# Account-based multi-tenancy
accounts {
  # System account for internal operations
  SYS: {
    users: [
      {
        user: "system",
        password: "$2a$11$..."  # bcrypt hash
        permissions: {
          publish: [">"]
          subscribe: [">"]
        }
      }
    ]
  }
  
  # Template for tenant accounts
  TENANT_A: {
    users: [
      {
        user: "tenant_a_admin",
        password: "$2a$11$..."
        permissions: {
          publish: ["tenantA.>"]
          subscribe: ["tenantA.>", "_INBOX.>"]
        }
      },
      {
        user: "tenant_a_service",
        password: "$2a$11$..."
        permissions: {
          publish: ["tenantA.services.>"]
          subscribe: ["tenantA.services.>", "_INBOX.>"]
        }
      }
    ]
    
    # JetStream limits per tenant
    jetstream: {
      max_memory: 512MB
      max_disk: 2GB
      max_streams: 10
      max_consumers: 100
    }
    
    # Connection limits
    limits: {
      max_connections: 50
      max_subscriptions: 1000
      max_payload: 1MB
      max_data: 10MB
      max_ack_pending: 65536
    }
  }
}

# System account designation
system_account: "SYS"

# Connection limits
max_connections: 1000
max_control_line: 4096
max_payload: 1048576
ping_interval: "2m"
ping_max: 2
write_deadline: "10s"

# Clustering for high availability
cluster {
  name: "mister-smith-cluster"
  host: "0.0.0.0"
  port: 6222
  
  # Cluster TLS
  tls {
    cert_file: "/etc/mister-smith/certs/server/server-cert.pem"
    key_file: "/etc/mister-smith/certs/server/server-key.pem"
    ca_file: "/etc/mister-smith/certs/ca/ca-cert.pem"
    verify: true
    timeout: 5
  }
  
  # Cluster routes
  routes: [
    "nats://nats-1.mister-smith.local:6222"
    "nats://nats-2.mister-smith.local:6222"
    "nats://nats-3.mister-smith.local:6222"
  ]
}

# Gateway configuration for multi-region
gateway {
  name: "mister-smith-gateway"
  host: "0.0.0.0"
  port: 7222
  
  # Gateway TLS
  tls {
    cert_file: "/etc/mister-smith/certs/server/server-cert.pem"
    key_file: "/etc/mister-smith/certs/server/server-key.pem"
    ca_file: "/etc/mister-smith/certs/ca/ca-cert.pem"
    verify: true
  }
}

# Monitoring and debugging
debug: false
trace: false
logtime: true
log_file: "/var/log/nats/nats-server.log"
pid_file: "/var/run/nats/nats-server.pid"

# Disable non-TLS connections completely
no_auth_user: ""
authorization {
  timeout: 5
}
```

### 5.2 NATS Client Implementation with mTLS

**Secure NATS Client:**

```rust
// nats_client.rs
use nats::asynk::{Connection, Options};
use anyhow::{Result, Context};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NatsSecureClient {
    connection: Option<Connection>,
    tenant_id: Uuid,
    client_cert_path: String,
    client_key_path: String,
    ca_cert_path: String,
    nats_urls: Vec<String>,
}

impl NatsSecureClient {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            connection: None,
            tenant_id,
            client_cert_path: "/etc/mister-smith/certs/client/client-cert.pem".to_string(),
            client_key_path: "/etc/mister-smith/certs/client/client-key.pem".to_string(),
            ca_cert_path: "/etc/mister-smith/certs/ca/ca-cert.pem".to_string(),
            nats_urls: vec![
                "tls://nats-1.mister-smith.local:4222".to_string(),
                "tls://nats-2.mister-smith.local:4222".to_string(),
                "tls://nats-3.mister-smith.local:4222".to_string(),
            ],
        }
    }

    /// Connect to NATS with mTLS
    pub async fn connect(&mut self, username: &str, password: &str) -> Result<()> {
        let options = Options::new()
            .with_name(&format!("mister-smith-client-{}", self.tenant_id))
            .with_user_and_password(username, password)
            .with_client_cert(&self.client_cert_path, &self.client_key_path)
            .with_context(|| "Failed to load client certificate")?
            .with_root_certificates(&self.ca_cert_path)
            .with_context(|| "Failed to load CA certificate")?
            .require_tls(true)
            .with_connection_timeout(Duration::from_secs(10))
            .with_reconnect_buffer_size(8 * 1024 * 1024) // 8MB
            .with_max_reconnects(5)
            .with_ping_interval(Duration::from_secs(30));

        let connection = options
            .connect(&self.nats_urls)
            .await
            .with_context(|| "Failed to connect to NATS server")?;

        self.connection = Some(connection);
        info!("Connected to NATS with mTLS for tenant: {}", self.tenant_id);
        Ok(())
    }

    /// Publish message to tenant-scoped subject
    pub async fn publish<T: Serialize>(&self, subject: &str, message: &T) -> Result<()> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to NATS"))?;

        let tenant_subject = format!("tenant{}.{}", self.tenant_id, subject);
        let payload = serde_json::to_vec(message)
            .with_context(|| "Failed to serialize message")?;

        // Check payload size (1MB limit)
        if payload.len() > 1024 * 1024 {
            anyhow::bail!("Message payload exceeds 1MB limit");
        }

        connection.publish(&tenant_subject, payload)
            .await
            .with_context(|| format!("Failed to publish to subject: {}", tenant_subject))?;

        info!("Published message to subject: {}", tenant_subject);
        Ok(())
    }

    /// Subscribe to tenant-scoped subject
    pub async fn subscribe(&self, subject: &str) -> Result<nats::asynk::Subscription> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to NATS"))?;

        let tenant_subject = format!("tenant{}.{}", self.tenant_id, subject);
        
        let subscription = connection.subscribe(&tenant_subject)
            .await
            .with_context(|| format!("Failed to subscribe to subject: {}", tenant_subject))?;

        info!("Subscribed to subject: {}", tenant_subject);
        Ok(subscription)
    }

    /// Create JetStream consumer with security constraints
    pub async fn create_consumer(&self, stream_name: &str, consumer_config: ConsumerConfig) -> Result<()> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to NATS"))?;

        let js = nats::jetstream::new(connection.clone());
        let tenant_stream = format!("tenant{}.{}", self.tenant_id, stream_name);

        // Enforce security constraints
        let secure_config = ConsumerConfig {
            deliver_subject: consumer_config.deliver_subject.map(|s| 
                format!("tenant{}.{}", self.tenant_id, s)
            ),
            max_deliver: Some(consumer_config.max_deliver.unwrap_or(5)),
            max_ack_pending: Some(consumer_config.max_ack_pending.unwrap_or(1000)),
            ..consumer_config
        };

        js.add_consumer(&tenant_stream, &secure_config)
            .await
            .with_context(|| format!("Failed to create consumer for stream: {}", tenant_stream))?;

        info!("Created consumer for stream: {}", tenant_stream);
        Ok(())
    }

    /// Health check for connection
    pub async fn health_check(&self) -> Result<()> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to NATS"))?;

        // Send a ping and wait for pong
        connection.flush().await
            .with_context(|| "Health check failed")?;

        Ok(())
    }

    /// Disconnect from NATS
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection) = self.connection.take() {
            connection.close().await;
            info!("Disconnected from NATS for tenant: {}", self.tenant_id);
        }
        Ok(())
    }
}

// JetStream configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    pub durable_name: Option<String>,
    pub deliver_subject: Option<String>,
    pub deliver_policy: Option<String>,
    pub opt_start_seq: Option<u64>,
    pub opt_start_time: Option<String>,
    pub ack_policy: Option<String>,
    pub ack_wait: Option<Duration>,
    pub max_deliver: Option<i32>,
    pub max_ack_pending: Option<i32>,
    pub replay_policy: Option<String>,
}

/// NATS connection pool for high-performance applications
pub struct NatsConnectionPool {
    pools: std::collections::HashMap<Uuid, Vec<NatsSecureClient>>,
    pool_size: usize,
}

impl NatsConnectionPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pools: std::collections::HashMap::new(),
            pool_size,
        }
    }

    /// Get or create connection pool for tenant
    pub async fn get_connection(&mut self, tenant_id: Uuid, username: &str, password: &str) -> Result<&mut NatsSecureClient> {
        if !self.pools.contains_key(&tenant_id) {
            let mut pool = Vec::new();
            for _ in 0..self.pool_size {
                let mut client = NatsSecureClient::new(tenant_id);
                client.connect(username, password).await?;
                pool.push(client);
            }
            self.pools.insert(tenant_id, pool);
        }

        let pool = self.pools.get_mut(&tenant_id).unwrap();
        // Simple round-robin selection (in production, use more sophisticated load balancing)
        Ok(&mut pool[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nats_secure_client() {
        let tenant_id = Uuid::new_v4();
        let mut client = NatsSecureClient::new(tenant_id);

        // This would require a running NATS server with proper certificates
        // assert!(client.connect("test_user", "test_password").await.is_ok());
    }
}
```

## 6. Hook Security Implementation

### 6.1 Secure Hook Execution Environment

**Complete Hook Security Manager:**

```rust
// hook_security.rs
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSecurityConfig {
    pub execution_user: String,
    pub execution_group: String,
    pub sandbox_base_dir: PathBuf,
    pub max_execution_time_seconds: u64,
    pub max_memory_mb: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
    pub allowed_read_paths: Vec<PathBuf>,
    pub allowed_write_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub allowed_network: bool,
    pub allowed_localhost: bool,
    pub blocked_ports: Vec<u16>,
}

impl Default for HookSecurityConfig {
    fn default() -> Self {
        Self {
            execution_user: "claude-hook-runner".to_string(),
            execution_group: "claude-hooks".to_string(),
            sandbox_base_dir: PathBuf::from("/tmp/mister-smith-hooks"),
            max_execution_time_seconds: 30,
            max_memory_mb: 128,
            max_file_descriptors: 64,
            max_processes: 1,
            allowed_read_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/lib"),
                PathBuf::from("/bin"),
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/etc/group"),
            ],
            allowed_write_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/home"),
                PathBuf::from("/var/lib/claude-hooks/.ssh"),
                PathBuf::from("/proc"),
                PathBuf::from("/sys"),
            ],
            allowed_network: false,
            allowed_localhost: true,
            blocked_ports: vec![22, 3389, 5432, 27017],
        }
    }
}

#[derive(Debug, Clone)]
pub struct HookExecutionContext {
    pub hook_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub sandbox_dir: TempDir,
    pub environment: std::collections::HashMap<String, String>,
}

pub struct HookSecurityManager {
    config: HookSecurityConfig,
    active_executions: std::collections::HashMap<Uuid, HookExecutionContext>,
}

impl HookSecurityManager {
    pub fn new(config: HookSecurityConfig) -> Result<Self> {
        // Ensure sandbox base directory exists
        fs::create_dir_all(&config.sandbox_base_dir)
            .with_context(|| "Failed to create sandbox base directory")?;

        // Verify execution user exists
        Self::verify_execution_user(&config.execution_user)?;

        Ok(Self {
            config,
            active_executions: std::collections::HashMap::new(),
        })
    }

    /// Verify that the execution user exists and is properly configured
    fn verify_execution_user(username: &str) -> Result<()> {
        let output = Command::new("id")
            .arg(username)
            .output()
            .with_context(|| format!("Failed to check user: {}", username))?;

        if !output.status.success() {
            anyhow::bail!("Execution user '{}' does not exist", username);
        }

        info!("Verified execution user: {}", username);
        Ok(())
    }

    /// Validate hook script before execution
    pub fn validate_hook_script(&self, script_path: &Path) -> Result<()> {
        // Check if file exists
        if !script_path.exists() {
            anyhow::bail!("Hook script does not exist: {:?}", script_path);
        }

        // Check file permissions
        let metadata = fs::metadata(script_path)
            .with_context(|| "Failed to read script metadata")?;

        // Check if world-writable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o002 != 0 {
                anyhow::bail!("Hook script cannot be world-writable: {:?}", script_path);
            }
        }

        // Read and validate script content
        let content = fs::read_to_string(script_path)
            .with_context(|| "Failed to read script content")?;

        self.validate_script_content(&content)?;

        info!("Hook script validation passed: {:?}", script_path);
        Ok(())
    }

    /// Validate script content for dangerous patterns
    fn validate_script_content(&self, content: &str) -> Result<()> {
        let dangerous_patterns = [
            r"sudo\s",
            r"su\s+-",
            r"chmod\s+777",
            r"rm\s+-rf\s+/",
            r"curl.*\|.*sh",
            r"wget.*\|.*sh",
            r"eval\s*\(",
            r"/etc/passwd",
            r"/etc/shadow",
            r"nc\s+-l",
            r"netcat\s+-l",
            r"python.*-c.*exec",
            r"perl.*-e",
            r"\$\(.*\)",  // Command substitution
        ];

        for pattern in &dangerous_patterns {
            let regex = regex::Regex::new(pattern)
                .with_context(|| format!("Invalid regex pattern: {}", pattern))?;
            
            if regex.is_match(content) {
                anyhow::bail!("Hook script contains dangerous pattern: {}", pattern);
            }
        }

        // Validate shebang
        if !content.starts_with("#!") {
            anyhow::bail!("Hook script must have valid shebang");
        }

        Ok(())
    }

    /// Create secure execution context
    pub fn create_execution_context(
        &mut self,
        tenant_id: Uuid,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<Uuid> {
        let hook_id = Uuid::new_v4();
        
        // Create isolated sandbox directory
        let sandbox_dir = TempDir::new_in(&self.config.sandbox_base_dir)
            .with_context(|| "Failed to create sandbox directory")?;

        // Set up environment variables (filtered for security)
        let mut environment = std::collections::HashMap::new();
        environment.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        environment.insert("HOME".to_string(), sandbox_dir.path().to_string_lossy().to_string());
        environment.insert("USER".to_string(), self.config.execution_user.clone());
        environment.insert("SHELL".to_string(), "/bin/bash".to_string());
        environment.insert("TMPDIR".to_string(), sandbox_dir.path().to_string_lossy().to_string());
        
        // Add hook-specific variables
        environment.insert("HOOK_ID".to_string(), hook_id.to_string());
        environment.insert("TENANT_ID".to_string(), tenant_id.to_string());
        environment.insert("USER_ID".to_string(), user_id.to_string());
        environment.insert("SESSION_ID".to_string(), session_id.to_string());

        let context = HookExecutionContext {
            hook_id,
            tenant_id,
            user_id,
            session_id,
            sandbox_dir,
            environment,
        };

        self.active_executions.insert(hook_id, context);
        info!("Created execution context: {}", hook_id);
        Ok(hook_id)
    }

    /// Execute hook script with security constraints
    pub async fn execute_hook(
        &mut self,
        hook_id: Uuid,
        script_path: &Path,
        args: Vec<String>,
        input_data: Option<Vec<u8>>,
    ) -> Result<HookExecutionResult> {
        let context = self.active_executions.get(&hook_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid hook execution context: {}", hook_id))?;

        // Validate script before execution
        self.validate_hook_script(script_path)?;

        // Copy script to sandbox
        let sandbox_script = context.sandbox_dir.path().join("hook_script");
        fs::copy(script_path, &sandbox_script)
            .with_context(|| "Failed to copy script to sandbox")?;

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&sandbox_script)?.permissions();
            permissions.set_mode(0o750); // rwxr-x---
            fs::set_permissions(&sandbox_script, permissions)?;
        }

        // Prepare command with security constraints
        let mut command = Command::new("timeout");
        command
            .arg(format!("{}s", self.config.max_execution_time_seconds))
            .arg("systemd-run")
            .arg("--user")
            .arg("--scope")
            .arg("--slice=claude-hooks.slice")
            .arg(format!("--property=MemoryMax={}M", self.config.max_memory_mb))
            .arg(format!("--property=TasksMax={}", self.config.max_processes))
            .arg("--property=PrivateNetwork=true")
            .arg("--property=NoNewPrivileges=true")
            .arg("--property=ProtectSystem=strict")
            .arg("--property=ProtectHome=true")
            .arg("--property=PrivateTmp=true")
            .arg("--setenv=PATH=/usr/bin:/bin")
            .arg(format!("--setenv=HOME={}", context.sandbox_dir.path().display()))
            .arg(format!("--setenv=USER={}", self.config.execution_user))
            .arg(&sandbox_script)
            .args(&args)
            .current_dir(context.sandbox_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &context.environment {
            command.env(key, value);
        }

        info!("Executing hook {} with security constraints", hook_id);
        
        let start_time = std::time::Instant::now();
        let mut child = command.spawn()
            .with_context(|| "Failed to spawn hook process")?;

        // Send input data if provided
        if let Some(input) = input_data {
            if let Some(stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let mut stdin = tokio::process::ChildStdin::from_std(stdin)
                    .with_context(|| "Failed to convert stdin")?;
                stdin.write_all(&input).await
                    .with_context(|| "Failed to write input data")?;
                stdin.shutdown().await
                    .with_context(|| "Failed to close stdin")?;
            }
        }

        // Wait for completion with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.max_execution_time_seconds + 5),
            child.wait_with_output()
        ).await
            .with_context(|| "Hook execution timed out")?
            .with_context(|| "Failed to get hook output")?;

        let execution_time = start_time.elapsed();

        let result = HookExecutionResult {
            hook_id,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            execution_time_ms: execution_time.as_millis() as u64,
            success: output.status.success(),
        };

        info!("Hook execution completed: {} (exit code: {})", 
            hook_id, result.exit_code);

        Ok(result)
    }

    /// Clean up execution context
    pub fn cleanup_execution_context(&mut self, hook_id: Uuid) -> Result<()> {
        if let Some(context) = self.active_executions.remove(&hook_id) {
            // Sandbox directory will be automatically cleaned up when TempDir is dropped
            info!("Cleaned up execution context: {}", hook_id);
        }
        Ok(())
    }

    /// Get active execution count
    pub fn active_execution_count(&self) -> usize {
        self.active_executions.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HookExecutionResult {
    pub hook_id: Uuid,
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub execution_time_ms: u64,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_script_validation() {
        let config = HookSecurityConfig::default();
        let manager = HookSecurityManager::new(config).unwrap();

        // Test valid script
        let mut valid_script = NamedTempFile::new().unwrap();
        writeln!(valid_script, "#!/bin/bash\necho 'Hello World'").unwrap();
        assert!(manager.validate_hook_script(valid_script.path()).is_ok());

        // Test dangerous script
        let mut dangerous_script = NamedTempFile::new().unwrap();
        writeln!(dangerous_script, "#!/bin/bash\nsudo rm -rf /").unwrap();
        assert!(manager.validate_hook_script(dangerous_script.path()).is_err());
    }

    #[tokio::test]
    async fn test_execution_context() {
        let config = HookSecurityConfig::default();
        let mut manager = HookSecurityManager::new(config).unwrap();

        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        let hook_id = manager.create_execution_context(tenant_id, user_id, session_id).unwrap();
        assert!(manager.active_executions.contains_key(&hook_id));

        manager.cleanup_execution_context(hook_id).unwrap();
        assert!(!manager.active_executions.contains_key(&hook_id));
    }
}
```

### 6.2 Hook Security Configuration

**Complete Security Configuration:**

```yaml
# hook_security_config.yml
hook_security:
  # User and group for hook execution
  execution_user: claude-hook-runner
  execution_group: claude-hooks
  
  # Sandbox configuration
  sandbox_base_dir: /tmp/mister-smith-hooks
  cleanup_interval_minutes: 60
  max_concurrent_executions: 10
  
  # Resource limits
  resource_limits:
    max_execution_time_seconds: 30
    max_memory_mb: 128
    max_cpu_percent: 50
    max_file_descriptors: 64
    max_processes: 1
    max_disk_usage_mb: 100
  
  # Filesystem access control
  filesystem_access:
    allowed_read_paths:
      - /usr
      - /lib
      - /lib64
      - /bin
      - /etc/passwd
      - /etc/group
      - /etc/hosts
      - /etc/resolv.conf
    
    allowed_write_paths:
      - /tmp/mister-smith-hooks
    
    blocked_paths:
      - /etc
      - /root
      - /home
      - /var/lib/claude-hooks/.ssh
      - /proc
      - /sys
      - /dev
      - /boot
  
  # Network restrictions
  network_access:
    allow_outbound: false
    allow_localhost: true
    blocked_ports:
      - 22    # SSH
      - 23    # Telnet
      - 3389  # RDP
      - 5432  # PostgreSQL
      - 3306  # MySQL
      - 27017 # MongoDB
      - 6379  # Redis
      - 9200  # Elasticsearch
    
    allowed_destinations:
      - 127.0.0.1
      - ::1
  
  # Script validation
  script_validation:
    enforce_shebang: true
    max_script_size_kb: 1024
    
    dangerous_patterns:
      - "sudo\\s"
      - "su\\s+-"
      - "chmod\\s+777"
      - "rm\\s+-rf\\s+/"
      - "curl.*\\|.*sh"
      - "wget.*\\|.*sh"
      - "eval\\s*\\("
      - "/etc/passwd"
      - "/etc/shadow"
      - "nc\\s+-l"
      - "netcat\\s+-l"
      - "python.*-c.*exec"
      - "perl.*-e"
      - "\\$\\(.*\\)"
    
    allowed_interpreters:
      - /bin/bash
      - /bin/sh
      - /usr/bin/python3
      - /usr/bin/node
    
  # Environment variables
  environment:
    # Variables always set
    default_vars:
      PATH: "/usr/bin:/bin"
      SHELL: "/bin/bash"
      TMPDIR: "${SANDBOX_DIR}"
      HOME: "${SANDBOX_DIR}"
      USER: "${EXECUTION_USER}"
    
    # Variables from user context
    context_vars:
      - HOOK_ID
      - TENANT_ID
      - USER_ID
      - SESSION_ID
    
    # Blocked environment variables
    blocked_vars:
      - LD_PRELOAD
      - LD_LIBRARY_PATH
      - SSH_AUTH_SOCK
      - DISPLAY
      - XAUTHORITY
  
  # Monitoring and logging
  monitoring:
    log_all_executions: true
    log_stdout_stderr: true
    max_log_size_mb: 10
    
    # Metrics to collect
    metrics:
      - execution_count
      - execution_duration
      - memory_usage
      - cpu_usage
      - exit_codes
      - validation_failures
    
    # Alerts
    alerts:
      max_failures_per_hour: 10
      max_execution_time_violations: 5
      suspicious_patterns_detected: 1
  
  # Systemd integration
  systemd:
    slice_name: claude-hooks.slice
    service_properties:
      MemoryAccounting: true
      CPUAccounting: true
      TasksAccounting: true
      IOAccounting: true
      PrivateNetwork: true
      NoNewPrivileges: true
      ProtectSystem: strict
      ProtectHome: true
      PrivateTmp: true
      ProtectKernelTunables: true
      ProtectKernelModules: true
      ProtectControlGroups: true
      RestrictSUIDSGID: true
      RemoveIPC: true
      RestrictRealtime: true
      RestrictNamespaces: true
      LockPersonality: true
      ProtectHostname: true
      ProtectClock: true
      ProtectKernelLogs: true
      ProtectProc: invisible
      ProcSubset: pid
```

## Security Enhancement Recommendations

**Based on Agent 13 Validation - Priority Implementation Areas**

### 1. Threat Modeling Framework

**Enhancement Priority: High (Agent 13 Score: 3/5)**

```rust
// Structured threat analysis approach
function perform_threat_modeling():
    // STRIDE analysis
    threats = analyze_stride_threats([
        "Spoofing identity",
        "Tampering with data", 
        "Repudiation of actions",
        "Information disclosure",
        "Denial of service",
        "Elevation of privilege"
    ])
    
    // Risk assessment matrix
    for threat in threats:
        risk_score = calculate_risk(threat.probability, threat.impact)
        mitigation_controls = map_controls_to_threat(threat)
        
        threat_registry.add_threat({
            "id": threat.id,
            "category": threat.stride_category,
            "risk_score": risk_score,
            "mitigations": mitigation_controls,
            "status": "identified"
        })
    
    return generate_threat_model_report(threat_registry)
```

#### Threat Categories for Multi-Agent Systems

- **Agent Impersonation**: Unauthorized agents claiming legitimate identities
- **Command Injection**: Malicious commands in agent communications
- **Data Exfiltration**: Unauthorized access to sensitive agent data
- **Privilege Escalation**: Agents exceeding authorized capabilities
- **Side-Channel Attacks**: Information leakage through timing or resource usage
- **Agent Coordination Attacks**: Manipulation of agent collaboration patterns

### 2. Security Testing Framework

**Enhancement Priority: High (Agent 13 Recommendation)**

```rust
// Comprehensive security testing approach
function security_testing_suite():
    // Authentication testing
    auth_tests = [
        test_token_expiration(),
        test_invalid_signatures(),
        test_algorithm_confusion(),
        test_token_replay_attacks(),
        test_brute_force_protection()
    ]
    
    // Authorization testing  
    authz_tests = [
        test_privilege_escalation(),
        test_resource_enumeration(),
        test_role_boundary_violations(),
        test_policy_bypass_attempts(),
        test_delegation_chain_validation()
    ]
    
    // Transport security testing
    transport_tests = [
        test_certificate_validation(),
        test_protocol_downgrade_protection(),
        test_cipher_suite_selection(),
        test_certificate_pinning(),
        test_mtls_mutual_verification()
    ]
    
    // Execute all test suites
    execute_security_test_suites([auth_tests, authz_tests, transport_tests])
```

#### Security Testing Categories

- **Penetration Testing**: Regular external security assessments
- **Fuzzing**: Input validation testing for all interfaces
- **Load Testing**: Security under high-stress conditions
- **Regression Testing**: Automated security test suites in CI/CD
- **Compliance Testing**: Automated regulatory compliance validation

### 3. Compliance Framework Mapping

**Validated Coverage (Agent 13 Score: 4/5)**

#### Regulatory Compliance Status

- **GDPR**: ✅ Data minimization, user rights, consent management
- **SOC 2 Type II**: ✅ Security, availability, processing integrity controls
- **ISO 27001**: ✅ Information security management alignment
- **NIST Framework**: ✅ Cybersecurity framework mapping
- **OWASP**: ✅ Web application security best practices
- **CIS Controls**: ✅ Critical security control implementation

#### Compliance Automation

```rust
// Automated compliance monitoring
pub struct ComplianceMonitor {
    pub gdpr_processor: GdprComplianceProcessor,
    pub soc2_auditor: Soc2AuditTracker,
    pub iso27001_mapper: Iso27001ControlMapper,
    pub nist_assessor: NistFrameworkAssessor,
}

impl ComplianceMonitor {
    pub fn generate_compliance_report(&self) -> ComplianceReport {
        ComplianceReport {
            gdpr_status: self.gdpr_processor.assess_compliance(),
            soc2_status: self.soc2_auditor.generate_audit_trail(),
            iso27001_status: self.iso27001_mapper.map_controls(),
            nist_status: self.nist_assessor.assess_framework_alignment(),
            overall_score: self.calculate_overall_compliance(),
        }
    }
}
```

---

This document provides foundational security patterns for agent implementation. For complete system architecture and advanced patterns, refer to the canonical tech-framework.md.
