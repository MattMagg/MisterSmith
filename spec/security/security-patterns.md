---
title: Security Patterns
type: note
permalink: ms-framework/security/security-patterns
tags:
- '#security'
- '#patterns'
- '#foundational'
- '#agent-focused'
---

## Security Patterns

## Overview

This document provides essential security patterns, guidelines, and configurations for agent implementation. It covers authentication, authorization, TLS configuration, secrets management, and secure communication patterns.

**Related Documentation:**

- [Security Framework](./security-framework.md) - Comprehensive security implementations and detailed code examples
- [JWT Implementation](./security-framework.md#2-jwt-authentication-implementation) - Complete JWT authentication service
- [Certificate Management](./security-framework.md#1-certificate-management-implementation) - Production-ready certificate handling
- [RBAC Implementation](./security-framework.md#3-authorization-implementation) - Role-based access control patterns

## Framework Authority

This document implements specifications from the canonical tech-framework.md located at /Users/mac-main/Mister-Smith/Mister-Smith/tech-framework.md

As stated in the canonical framework: "Agents: use this framework as the canonical source."

## Purpose

Essential security patterns, guidelines, and configuration templates for agent implementation. This document provides foundational security building blocks including authentication patterns, authorization flows, TLS setup, and secure communication guidelines.

## Core Security Components

### 1. Basic Authentication Pattern

**See also:** [JWT Authentication Implementation](./security-framework.md#2-jwt-authentication-implementation) for complete production-ready JWT service.

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

// Enhanced Pattern with Refresh Token Support (Recommended)
function generate_token_pair(user_id):
    access_token = generate_auth_token(user_id, 900)  // 15 minutes
    refresh_token = generate_refresh_token(user_id, 86400)  // 24 hours
    return {access_token, refresh_token}

function refresh_access_token(refresh_token):
    if verify_refresh_token(refresh_token):
        return generate_auth_token(refresh_token.user_id, 900)
    return error(401, "Invalid refresh token")
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

### 2. Simple Authorization Pattern

**See also:** [RBAC Authorization Implementation](./security-framework.md#3-authorization-implementation) for complete role-based access control engine.

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

// Enhanced Context-Aware Authorization Pattern (Recommended)
function check_permission_with_context(user_id, resource, action, context):
    // Basic RBAC check
    if not check_permission(user_id, resource, action):
        return deny()
    
    // Context-aware attributes (ABAC)
    attributes = {
        time: context.request_time,
        location: context.source_ip,
        device: context.device_fingerprint,
        tenant: context.tenant_id
    }
    
    // Apply attribute-based rules
    return evaluate_abac_rules(user_id, resource, action, attributes)

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
  
  compliance_frameworks:
    gdpr:
      enabled: true
      consent_tracking: true
      data_retention_limits: true
      right_to_erasure: true
    soc2:
      enabled: true
      access_reviews: quarterly
      change_management: automated
    iso27001:
      enabled: true
      incident_response: true
      risk_management: true
    pci_dss:
      enabled: false
    hipaa:
      enabled: false
    sox:
      enabled: false
  
  roles:
    - name: reader
      permissions: [read]
    - name: writer  
      permissions: [read, write]
    - name: admin
      permissions: [read, write, delete, admin]
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

**See also:** [Certificate Management](./security-framework.md#1-certificate-management-implementation) for production certificate handling patterns.

**Security Note**: Environment variables visible in process lists pose credential compromise risk in production environments

**Pseudocode Pattern:**

```rust
// Environment-Based Secrets (Use only for development)
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

// Enterprise Secrets Management Pattern (Recommended for Production)
function load_secrets_from_vault(vault_config):
    vault_client = initialize_vault_client(vault_config)
    
    secrets = {}
    for secret_path in required_secret_paths:
        secret_data = vault_client.read_secret(secret_path)
        if not secret_data:
            error("Failed to retrieve secret: " + secret_path)
        secrets[extract_key(secret_path)] = secret_data.value
    
    return secrets

// Secret Rotation Pattern
function rotate_secret(secret_name):
    old_secret = get_current_secret(secret_name)
    new_secret = generate_new_secret()
    
    // Update downstream services with new secret
    update_secret_references(secret_name, new_secret)
    
    // Invalidate old secret after grace period
    schedule_secret_invalidation(old_secret, grace_period=300)
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

// Enhanced Security Headers (Recommended)
function enhanced_security_headers_middleware(request, response, next):
    // Basic security headers
    response.headers.add("X-Content-Type-Options", "nosniff")
    response.headers.add("X-Frame-Options", "DENY")
    response.headers.add("X-XSS-Protection", "1; mode=block")
    response.headers.add("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
    
    // Content Security Policy
    csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    response.headers.add("Content-Security-Policy", csp)
    
    // Additional security headers
    response.headers.add("Referrer-Policy", "strict-origin-when-cross-origin")
    response.headers.add("Permissions-Policy", "geolocation=(), microphone=(), camera=()")
    
    return next(request, response)

// CSRF Protection Pattern
function csrf_protection_middleware(request, response, next):
    if request.method in ["POST", "PUT", "DELETE", "PATCH"]:
        csrf_token = extract_csrf_token(request)
        if not verify_csrf_token(csrf_token, request.session):
            return error(403, "CSRF token validation failed")
    
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

**See also:** [Security Audit Implementation](./security-framework.md#4-security-audit-implementation) for comprehensive audit service with integrity protection.

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

// Enhanced Audit Logging with Integrity Protection (Recommended)
function log_security_event_with_integrity(event_type, details):
    event = {
        id: generate_uuid(),
        timestamp: current_timestamp(),
        event_type: event_type,
        details: details,
        source_ip: get_request_ip(),
        user_id: get_current_user_id(),
        session_id: get_session_id(),
        device_fingerprint: get_device_fingerprint()
    }
    
    // Add integrity protection
    event.hash = compute_event_hash(event)
    event.signature = sign_event(event, audit_signing_key)
    
    // Send to SIEM and local storage
    send_to_siem(event)
    append_to_audit_log(event)

// Real-time Security Monitoring
function monitor_security_events():
    while true:
        events = get_recent_security_events(last_check_time)
        
        for event in events:
            severity = assess_event_severity(event)
            if severity >= HIGH_SEVERITY:
                trigger_security_alert(event)
                escalate_to_soc(event)

// Common Security Events to Log
security_events = [
    "authentication_success",
    "authentication_failure", 
    "authorization_denied",
    "invalid_token",
    "rate_limit_exceeded",
    "suspicious_activity",
    "privilege_escalation_attempt",
    "data_access_violation",
    "system_configuration_change"
]
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

### Foundation Security (Basic Implementation)

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

### Advanced Security Enhancements

- [ ] Implement incident response framework with classification and escalation
- [ ] Integrate enterprise secrets management (Vault/AWS Secrets Manager)
- [ ] Implement encryption at rest for databases and file systems
- [ ] Deploy real-time security monitoring and SIEM integration
- [ ] Add Content Security Policy (CSP) and CSRF protection
- [ ] Implement behavioral analytics and anomaly detection
- [ ] Add multi-factor authentication (MFA) support
- [ ] Implement compliance framework alignment (GDPR/HIPAA/SOX)

### Advanced Security Controls

- [ ] Implement SQL injection prevention with parameterized queries
- [ ] Add distributed denial of service (DDoS) protection
- [ ] Implement supply chain security with dependency verification
- [ ] Add runtime application self-protection (RASP) capabilities
- [ ] Implement user and entity behavior analytics (UEBA)
- [ ] Add security metrics and KPIs monitoring
- [ ] Implement automated threat hunting capabilities
- [ ] Add log integrity protection with tamper detection

### Security Validation & Testing

- [ ] Conduct regular penetration testing
- [ ] Perform security code reviews for all components
- [ ] Validate security configurations in isolated environments
- [ ] Test incident response procedures regularly
- [ ] Audit access controls and permissions quarterly
- [ ] Verify backup and disaster recovery procedures

## Security Architecture Examples

### Complete Agent Security Implementation

**Agent Initialization with Full Security Stack:**

```rust
// Complete security setup for MisterSmith agent
use ms_framework::security::{
    JwtService, CertificateManager, AuthorizationMiddleware,
    AuditService, SecureNatsClient
};

pub struct SecureAgent {
    jwt_service: JwtService,
    cert_manager: CertificateManager,
    authz_middleware: AuthorizationMiddleware,
    audit_service: AuditService,
    nats_client: SecureNatsClient,
}

impl SecureAgent {
    pub async fn new(config: SecurityConfig) -> Result<Self> {
        // Initialize certificate management
        let cert_manager = CertificateManager::new();
        cert_manager.start_monitoring().await;
        
        // Initialize JWT authentication
        let jwt_service = JwtService::new()?;
        
        // Initialize authorization engine
        let authz_middleware = AuthorizationMiddleware::new();
        
        // Initialize audit logging
        let mut audit_service = AuditService::new();
        
        // Initialize secure NATS connection
        let tls_config = cert_manager.create_client_config()?;
        let nats_client = SecureNatsClient::connect(
            &config.nats_url,
            tls_config,
            &config.tenant_id
        ).await?;
        
        Ok(Self {
            jwt_service,
            cert_manager,
            authz_middleware,
            audit_service,
            nats_client,
        })
    }
    
    pub async fn handle_request(&mut self, request: Request) -> Result<Response> {
        // Step 1: Authenticate request
        let claims = self.jwt_service.authenticate_request(
            request.headers.get("Authorization")
        )?;
        
        // Step 2: Log authentication success
        self.audit_service.log_authentication(
            Some(claims.custom.user_id),
            Some(claims.custom.tenant_id),
            request.client_ip(),
            request.user_agent(),
            true,
            None
        );
        
        // Step 3: Extract resource and action
        let resource = Resource::from_request(&request)?;
        let action = Action::from_request(&request)?;
        
        // Step 4: Authorize request
        self.authz_middleware.authorize(&claims.custom, &resource, &action)
            .map_err(|e| {
                // Log authorization failure
                self.audit_service.log_authorization(
                    claims.custom.user_id,
                    claims.custom.tenant_id,
                    claims.custom.session_id,
                    resource.id.clone(),
                    resource.resource_type.to_string(),
                    action.to_string(),
                    false,
                    Some(e.to_string())
                );
                e
            })?;
        
        // Step 5: Process request with security context
        let response = self.process_authorized_request(request, claims).await?;
        
        // Step 6: Add security headers
        let secure_response = self.add_security_headers(response);
        
        Ok(secure_response)
    }
}
```

### Multi-Tenant NATS Security Architecture

**Complete tenant isolation pattern:**

```rust
// Tenant-isolated NATS client with comprehensive security
pub struct TenantSecureNatsClient {
    client: async_nats::Client,
    tenant_id: String,
    audit_service: AuditService,
}

impl TenantSecureNatsClient {
    pub async fn new(tenant_id: String, cert_manager: &CertificateManager) -> Result<Self> {
        // Create tenant-specific TLS configuration
        let tls_config = cert_manager.create_client_config()?;
        
        // Connect with tenant-specific credentials
        let client = async_nats::ConnectOptions::new()
            .tls_client_config(tls_config)
            .name(&format!("tenant_{}_agent", tenant_id))
            .connect("nats://localhost:4222")
            .await?;
        
        let audit_service = AuditService::new();
        
        Ok(Self {
            client,
            tenant_id,
            audit_service,
        })
    }
    
    pub async fn publish_secure(&mut self, subject: &str, payload: &[u8]) -> Result<()> {
        // Validate subject follows tenant isolation pattern
        let tenant_subject = format!("tenant.{}.{}", self.tenant_id, subject);
        
        // Log message publication
        self.audit_service.log_message_event(
            &self.tenant_id,
            &tenant_subject,
            "publish",
            payload.len()
        );
        
        // Publish with rate limiting
        self.client.publish(tenant_subject, payload.into()).await?;
        
        Ok(())
    }
    
    pub async fn subscribe_secure(&self, subject: &str) -> Result<async_nats::Subscriber> {
        // Enforce tenant isolation in subscription
        let tenant_subject = format!("tenant.{}.{}", self.tenant_id, subject);
        
        // Create subscriber with resource limits
        let subscriber = self.client
            .subscribe(tenant_subject)
            .await?
            .with_capacity(1000); // Bounded buffer
        
        Ok(subscriber)
    }
}
```

### Zero-Trust Security Flow

**End-to-end security validation:**

```rust
// Zero-trust security validation flow
pub struct ZeroTrustValidator {
    jwt_service: JwtService,
    rbac_engine: RbacEngine,
    audit_service: AuditService,
    rate_limiter: RateLimiter,
}

impl ZeroTrustValidator {
    pub async fn validate_request(&mut self, request: &Request) -> Result<SecurityContext> {
        // 1. Rate limiting (first line of defense)
        let client_id = self.extract_client_id(request)?;
        self.rate_limiter.check_limit(&client_id)?;
        
        // 2. TLS verification (transport security)
        self.verify_tls_connection(request)?;
        
        // 3. Authentication (identity verification)
        let claims = self.jwt_service.authenticate_request(
            request.headers.get("Authorization")
        )?;
        
        // 4. Authorization (permission validation)
        let resource = Resource::from_request(request)?;
        let action = Action::from_request(request)?;
        
        let authorized = self.rbac_engine.check_permission(
            &claims.custom,
            &resource,
            &action
        )?;
        
        if !authorized {
            self.audit_service.log_authorization_denied(
                claims.custom.user_id,
                resource.id,
                action.to_string(),
                "Insufficient permissions".to_string()
            );
            return Err(SecurityError::Unauthorized);
        }
        
        // 5. Context validation (additional security checks)
        self.validate_request_context(request, &claims)?;
        
        // 6. Log successful security validation
        self.audit_service.log_successful_validation(
            claims.custom.user_id,
            resource.id,
            action.to_string()
        );
        
        Ok(SecurityContext {
            user_claims: claims.custom,
            validated_resource: resource,
            permitted_action: action,
            validation_timestamp: Utc::now(),
        })
    }
    
    fn validate_request_context(&self, request: &Request, claims: &Claims<UserClaims>) -> Result<()> {
        // IP address validation
        if let Some(allowed_ips) = &claims.custom.allowed_ips {
            let client_ip = request.client_ip();
            if !allowed_ips.contains(&client_ip) {
                return Err(SecurityError::UnauthorizedSource);
            }
        }
        
        // Time-based access control
        if claims.custom.require_business_hours {
            let current_hour = Utc::now().hour();
            if current_hour < 9 || current_hour > 17 {
                return Err(SecurityError::OutsideBusinessHours);
            }
        }
        
        // Device fingerprint validation
        if let Some(device_id) = request.headers.get("X-Device-ID") {
            if !self.validate_device_fingerprint(device_id, claims.custom.user_id)? {
                return Err(SecurityError::UnknownDevice);
            }
        }
        
        Ok(())
    }
}
```

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

## Hook Execution Sandbox Pattern

### Non-Root User Execution Pattern

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

### Hook Script Validation Pattern

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

## Security Enhancement Areas

### Incident Response Framework

- Implement incident classification and severity assessment system
- Create response playbooks for common security scenarios
- Establish escalation procedures and communication templates
- Integrate with security operations center (SOC)

### Enterprise Secrets Management

- Integrate HashiCorp Vault or AWS Secrets Manager
- Implement automatic secret rotation mechanisms
- Remove secrets from environment variables and process lists
- Add key management lifecycle procedures

### Encryption at Rest

- Implement database encryption guidelines
- Add file system encryption patterns
- Provide key management lifecycle procedures

### Real-Time Security Monitoring

- Implement SIEM integration patterns
- Add real-time alerting for critical security events
- Create security operations dashboard
- Implement behavioral analytics and anomaly detection

### Advanced Web Application Security

- Content Security Policy (CSP) implementation
- Cross-Site Request Forgery (CSRF) protection
- Advanced XSS prevention beyond basic headers
- SQL injection prevention patterns

### Advanced Authentication

- Implement multi-factor authentication (MFA)
- Add risk-based authentication
- Implement adaptive authentication based on context

## Attack Vector Coverage Analysis

### Well-Protected Attack Vectors

- **Man-in-the-Middle Attacks**: mTLS implementation provides strong protection
- **Privilege Escalation**: Sandbox execution and RBAC limit attack surface
- **Resource Exhaustion**: Rate limiting and NATS quotas prevent DoS
- **Network Eavesdropping**: TLS encryption secures communications
- **Tenant Isolation Breaches**: NATS account separation ensures true isolation

### Areas Requiring Additional Security Patterns

- **SQL Injection**: Database security patterns and parameterized query guidelines
- **Cross-Site Scripting (XSS)**: Content Security Policy (CSP) implementation
- **Cross-Site Request Forgery (CSRF)**: CSRF token validation patterns
- **Distributed Denial of Service (DDoS)**: Distributed rate limiting mechanisms
- **Supply Chain Attacks**: Dependency verification and integrity checking
- **Insider Threats**: Enhanced audit capabilities and behavioral monitoring
- **Zero-Day Exploits**: Runtime application self-protection (RASP) patterns

## Security Pattern Implementations

### Incident Response Framework

```yaml
incident_response:
  classification:
    - severity_levels: [low, medium, high, critical]
    - impact_assessment: [data_breach, service_disruption, compliance_violation]
  escalation:
    - tier1: security_team
    - tier2: incident_commander
    - tier3: executive_leadership
  playbooks:
    - data_breach_response
    - malware_infection
    - unauthorized_access
    - ddos_attack
```

### Enterprise Secrets Management Integration

```yaml
secrets_management:
  provider: vault  # HashiCorp Vault, AWS Secrets Manager, Azure Key Vault
  rotation:
    enabled: true
    schedule: weekly
    grace_period: 300  # seconds
  audit:
    log_access: true
    track_rotation: true
```

### Encryption at Rest Patterns

```yaml
encryption_at_rest:
  database:
    enabled: true
    algorithm: AES-256-GCM
    key_management: external_kms
  filesystem:
    enabled: true
    mount_encryption: true
```

### Real-Time Security Monitoring

```yaml
security_monitoring:
  siem_integration:
    enabled: true
    endpoint: https://siem.company.com/api/events
  real_time_alerting:
    critical_events: immediate
    high_events: within_5_minutes
  behavioral_analytics:
    baseline_period: 30_days
    anomaly_threshold: 2_standard_deviations
```

### Enhanced Web Application Security

```yaml
web_security:
  content_security_policy:
    default_src: "'self'"
    script_src: "'self' 'unsafe-inline'"
    style_src: "'self' 'unsafe-inline'"
  csrf_protection:
    enabled: true
    token_rotation: per_request
  xss_protection:
    content_type_options: nosniff
    frame_options: DENY
```

## Production Readiness Assessment

### Security Maturity Scoring (6.8/10)

**Authentication/Authorization Completeness**: 6/10 points

- JWT authentication implemented but lacks enterprise features
- RBAC present but missing ABAC and context-aware controls
- No multi-factor authentication or advanced session management

**Transport Security Implementation**: 8/10 points  

- Excellent mTLS patterns with proper certificate management
- Strong TLS configuration with hostname verification
- Minor gaps in TLS 1.3 enforcement and perfect forward secrecy

**Threat Protection Coverage**: 3/5 points

- Good coverage of infrastructure threats
- Missing web application security patterns
- Limited advanced persistent threat protection

### Security Pattern Assessment by Category

| Pattern Category | Score | Status | Key Strengths | Critical Gaps |
|-----------------|-------|---------|---------------|---------------|
| NATS Security | 9/10 | ✅ Production-Ready | Complete mTLS, tenant isolation, resource quotas | Minor: TLS 1.3 enforcement |
| Hook Sandboxing | 8/10 | ✅ Production-Ready | Non-root execution, resource limits, script validation | Enhanced network isolation |
| TLS Configuration | 8/10 | ✅ Production-Ready | Comprehensive mTLS setup, proper validation | Perfect forward secrecy |
| Authentication | 6/10 | ⚠️ Needs Enhancement | JWT implementation, token validation | MFA, refresh tokens |
| Authorization | 6/10 | ⚠️ Needs Enhancement | RBAC patterns, role management | ABAC, context-awareness |
| Framework Integration | 7/10 | ⚠️ Needs Enhancement | Modular design, clear guidelines | Security governance |
| Secrets Management | 5/10 | ⚠️ Critical Gaps | Basic environment/file patterns | Enterprise integration |
| Security Middleware | 5/10 | ⚠️ Critical Gaps | Essential headers, rate limiting | CSP, CSRF protection |
| Audit Logging | 5/10 | ⚠️ Critical Gaps | Structured events, metadata | Integrity protection, SIEM |
| Incident Response | 3/10 | ❌ Insufficient | Basic event logging | Complete framework missing |
| Security Monitoring | 4/10 | ❌ Insufficient | Event capture, audit trail | Real-time alerting, SIEM |
| Threat Detection | 3/10 | ❌ Insufficient | Basic patterns | Behavioral analytics |

### Deployment Recommendations by Environment

**Development Environment** (Current patterns sufficient)

- Basic authentication and authorization patterns
- Environment-based secrets management
- Standard TLS configuration
- Basic audit logging

**Staging Environment** (Requires enhancements)

- Enhanced authentication with refresh tokens
- File-based secrets management with proper permissions
- Real-time security monitoring setup
- Incident response testing procedures

**Production Environment** (Critical gaps must be addressed)

- Enterprise secrets management integration (Vault/AWS)
- Comprehensive incident response framework
- Real-time security monitoring and SIEM integration
- Encryption at rest implementation
- Behavioral analytics and anomaly detection
- Advanced web application security (CSP, CSRF)

### Risk Assessment Summary

**Current Risk Level**: **MEDIUM-HIGH** for production deployments

**Acceptable for**:

- Development and testing environments
- Basic production deployments with low security requirements
- Internal systems with limited external exposure

**Not suitable for**:

- High-security production environments
- Regulated industries (healthcare, finance, government)
- Systems handling sensitive customer data
- Internet-facing applications with high threat exposure

**Key Recommendation**: Implement critical priority enhancements before deploying to high-security production environments.

## Incident Response Patterns

### Automated Incident Detection and Response

**CRITICAL GAP ADDRESSED: Missing incident response framework (Agent 18 Compliance Audit Finding)**

**Incident Response Pattern:**

```rust
// Incident Detection Pattern
function detect_security_incident(event):
    incident_indicators = analyze_event_for_threats(event)
    
    if incident_indicators.severity >= CRITICAL:
        return create_incident(
            type=map_to_incident_type(incident_indicators),
            severity=incident_indicators.severity,
            trigger_event=event,
            automated_response=true
        )
    
    return null

// Incident Response Automation
function handle_security_incident(incident):
    // Create incident record
    incident_id = create_incident_record(incident)
    
    // Execute immediate response
    playbook = get_response_playbook(incident.type)
    execute_automated_response(playbook, incident)
    
    // Collect evidence
    evidence = collect_incident_evidence(incident)
    
    // Notify security team
    notify_incident_team(incident, evidence)
    
    // Track response progress
    monitor_incident_resolution(incident_id)

// Evidence Collection Pattern
function collect_incident_evidence(incident):
    evidence_chain = create_evidence_chain(incident.id)
    
    // Collect system logs
    system_logs = collect_system_logs(incident.time_range)
    evidence_chain.add_evidence(system_logs, SYSTEM_LOGS)
    
    // Collect audit trails
    audit_trails = collect_audit_trails(incident.affected_users)
    evidence_chain.add_evidence(audit_trails, AUDIT_TRAILS)
    
    // Collect network traces
    network_traces = collect_network_traces(incident.source_ip)
    evidence_chain.add_evidence(network_traces, NETWORK_TRACES)
    
    // Sign and seal evidence
    evidence_chain.seal_with_digital_signature()
    
    return evidence_chain
```

**Configuration Pattern:**

```yaml
incident_response:
  detection:
    real_time_monitoring: true
    threshold_rules:
      - pattern: "failed_login_attempts > 5"
        severity: medium
        type: unauthorized_access
      - pattern: "privilege_escalation_detected"
        severity: critical
        type: privilege_escalation
      - pattern: "data_export_volume > 100MB"
        severity: high
        type: data_exfiltration
  
  response:
    automated_containment: true
    evidence_collection: true
    notification_channels:
      - email: security-team@company.com
      - slack: "#security-incidents"
      - pagerduty: security_oncall
  
  playbooks:
    unauthorized_access:
      - lock_user_account
      - review_session_tokens
      - analyze_source_ip
      - check_for_lateral_movement
    data_exfiltration:
      - quarantine_affected_data
      - review_access_logs
      - identify_data_scope
      - notify_data_protection_officer
    privilege_escalation:
      - revoke_elevated_permissions
      - audit_permission_changes
      - review_admin_actions
      - check_system_integrity
```

### Compliance Incident Reporting

**CRITICAL GAP ADDRESSED: Missing regulatory-specific incident handling**

**Compliance Reporting Pattern:**

```rust
// Compliance Incident Assessment
function assess_compliance_impact(incident):
    impact_assessment = ComplianceImpactAssessment()
    
    // Check GDPR requirements
    if incident.involves_personal_data:
        impact_assessment.gdpr_breach = true
        impact_assessment.notification_deadline = 72_hours
        impact_assessment.supervisory_authority = get_gdpr_authority(incident.location)
    
    // Check HIPAA requirements
    if incident.involves_health_data:
        impact_assessment.hipaa_breach = true
        impact_assessment.notification_deadline = 60_days
        impact_assessment.requires_hhs_notification = true
    
    // Check SOX requirements
    if incident.affects_financial_reporting:
        impact_assessment.sox_impact = true
        impact_assessment.requires_executive_notification = true
    
    return impact_assessment

// Automated Compliance Reporting
function generate_compliance_reports(incident, assessment):
    reports = {}
    
    if assessment.gdpr_breach:
        reports["gdpr"] = generate_gdpr_breach_report(incident)
        schedule_supervisory_authority_notification(reports["gdpr"])
    
    if assessment.hipaa_breach:
        reports["hipaa"] = generate_hipaa_breach_report(incident)
        schedule_hhs_notification(reports["hipaa"])
    
    if assessment.sox_impact:
        reports["sox"] = generate_sox_incident_report(incident)
        notify_financial_controls_team(reports["sox"])
    
    return reports
```

## Forensic Investigation Patterns

### Digital Evidence Collection and Chain of Custody

**CRITICAL GAP ADDRESSED: Missing evidence chain of custody protocols (Agent 18 Compliance Audit Finding)**

**Evidence Collection Pattern:**

```rust
// Evidence Chain of Custody
function create_evidence_chain(incident_id, investigator_id):
    chain = EvidenceChain()
    chain.id = generate_unique_id()
    chain.incident_id = incident_id
    chain.created_by = investigator_id
    chain.created_at = current_timestamp()
    chain.custody_log = []
    
    // Initial custody entry
    initial_entry = CustodyEntry(
        action=CREATED,
        actor=investigator_id,
        timestamp=current_timestamp(),
        notes="Evidence chain created for incident investigation"
    )
    chain.add_custody_entry(initial_entry)
    
    return chain

// Secure Evidence Collection
function collect_evidence_securely(source, evidence_type, chain):
    // Create evidence hash before collection
    pre_collection_hash = calculate_integrity_hash(source)
    
    // Collect evidence with metadata
    evidence = Evidence(
        id=generate_unique_id(),
        type=evidence_type,
        source=source,
        collected_at=current_timestamp(),
        collected_by=get_current_investigator(),
        integrity_hash=pre_collection_hash,
        metadata=extract_evidence_metadata(source)
    )
    
    // Add to evidence chain
    chain.add_evidence(evidence)
    
    // Create custody entry
    custody_entry = CustodyEntry(
        action=COLLECTED,
        actor=get_current_investigator(),
        timestamp=current_timestamp(),
        notes=f"Collected {evidence_type} from {source}",
        evidence_id=evidence.id
    )
    chain.add_custody_entry(custody_entry)
    
    // Verify evidence integrity
    post_collection_hash = calculate_integrity_hash(evidence.data)
    if pre_collection_hash != post_collection_hash:
        raise EvidenceIntegrityError("Evidence integrity compromised during collection")
    
    return evidence

// Evidence Transfer Protocol
function transfer_evidence_custody(evidence_chain, from_investigator, to_investigator, reason):
    transfer_entry = CustodyEntry(
        action=TRANSFERRED,
        actor=from_investigator,
        timestamp=current_timestamp(),
        notes=f"Evidence custody transferred to {to_investigator}. Reason: {reason}",
        transfer_recipient=to_investigator
    )
    
    // Digital signature for custody transfer
    transfer_signature = digital_sign(transfer_entry, from_investigator.private_key)
    transfer_entry.digital_signature = transfer_signature
    
    evidence_chain.add_custody_entry(transfer_entry)
    evidence_chain.current_custodian = to_investigator
    
    return evidence_chain
```

**Configuration Pattern:**

```yaml
forensic_investigation:
  evidence_collection:
    automatic_hash_verification: true
    digital_signatures: required
    custody_tracking: mandatory
    
  evidence_types:
    - system_logs
    - audit_trails
    - network_captures
    - database_queries
    - user_activity_logs
    - configuration_snapshots
    
  custody_requirements:
    chain_of_custody: mandatory
    digital_signatures: required
    integrity_verification: automatic
    access_logging: complete
    
  storage:
    encryption: AES-256-GCM
    retention_period: 7_years
    access_control: strict
    backup_redundancy: 3_copies
```

### Advanced Forensic Analysis Patterns

**User Activity Reconstruction Pattern:**

```rust
// Comprehensive User Activity Analysis
function reconstruct_user_activity(user_id, time_range, include_related_entities):
    reconstruction = UserActivityReconstruction()
    
    // Collect base user events
    user_events = collect_user_events(user_id, time_range)
    reconstruction.add_events(user_events)
    
    if include_related_entities:
        // Cross-reference with system events
        related_events = find_related_system_events(user_events)
        reconstruction.add_related_events(related_events)
        
        // Analyze data access patterns
        data_access = analyze_data_access_patterns(user_id, time_range)
        reconstruction.add_data_access_analysis(data_access)
        
        // Check for privilege escalation
        privilege_changes = analyze_privilege_changes(user_id, time_range)
        reconstruction.add_privilege_analysis(privilege_changes)
    
    // Build behavioral baseline
    behavioral_profile = build_behavioral_profile(user_id, time_range)
    reconstruction.set_behavioral_baseline(behavioral_profile)
    
    // Identify anomalies
    anomalies = detect_behavioral_anomalies(reconstruction)
    reconstruction.add_anomaly_analysis(anomalies)
    
    return reconstruction

// Cross-System Event Correlation
function correlate_events_across_systems(correlation_id, time_window):
    correlated_events = CrossSystemEventCorrelation()
    
    // Collect from multiple sources
    sources = ["nats_audit", "http_logs", "database_logs", "file_access", "network_logs"]
    
    for source in sources:
        events = collect_events_from_source(source, correlation_id, time_window)
        correlated_events.add_source_events(source, events)
    
    // Build correlation chains
    correlation_chains = build_correlation_chains(correlated_events.all_events())
    correlated_events.set_correlation_chains(correlation_chains)
    
    // Timeline reconstruction
    timeline = build_comprehensive_timeline(correlated_events)
    correlated_events.set_timeline(timeline)
    
    return correlated_events
```

## Compliance Validation Patterns

### Automated Compliance Checking

**CRITICAL GAP ADDRESSED: Missing automated compliance scanning (Agent 18 Compliance Audit Finding)**

**Compliance Validation Pattern:**

```rust
// Automated Compliance Assessment
function validate_compliance_posture(framework_type):
    assessment = ComplianceAssessment(framework_type)
    
    switch framework_type:
        case GDPR:
            assessment = validate_gdpr_compliance()
        case SOC2:
            assessment = validate_soc2_compliance()
        case ISO27001:
            assessment = validate_iso27001_compliance()
        case PCI_DSS:
            assessment = validate_pci_dss_compliance()
        case HIPAA:
            assessment = validate_hipaa_compliance()
        case SOX:
            assessment = validate_sox_compliance()
    
    // Generate compliance report
    report = generate_compliance_report(assessment)
    
    // Schedule remediation for gaps
    if assessment.has_critical_gaps():
        schedule_remediation_activities(assessment.critical_gaps)
    
    return assessment

// GDPR Compliance Validation
function validate_gdpr_compliance():
    assessment = GDPRAssessment()
    
    // Data minimization check
    assessment.data_minimization = check_data_minimization_compliance()
    
    // Consent management validation
    assessment.consent_management = validate_consent_tracking()
    
    // Right to access implementation
    assessment.right_to_access = validate_data_access_apis()
    
    // Right to erasure implementation  
    assessment.right_to_erasure = validate_data_erasure_procedures()
    
    // Breach notification procedures
    assessment.breach_notification = validate_breach_notification_system()
    
    return assessment
```

**Key Recommendation**: Implement critical priority enhancements before deploying to high-security production environments.

## Related Documents

### Security Implementation Files

- **[Authentication Implementation](authentication-implementation.md)** - Certificate management and JWT authentication implementation
- **[Authorization Implementation](authorization-implementation.md)** - RBAC and security audit implementation
- **[Security Integration](security-integration.md)** - NATS and hook security implementation
- **[Security Framework](security-framework.md)** - Complete security implementation details and concrete code
- **[Authentication Specifications](authentication-specifications.md)** - Detailed authentication requirements
- **[Authorization Specifications](authorization-specifications.md)** - Detailed authorization requirements

### Framework Integration Points

- **[Core Architecture](../core-architecture/)** - System architecture and design patterns
- **[Transport Layer](../transport/)** - Communication security specifications
- **[NATS Transport](../transport/nats-transport.md)** - NATS messaging security implementation
- **[Data Management](../data-management/)** - Message schemas and persistence security

---

*Security Patterns - Extracted from Framework Modularization Operation Phase 1, Group 1C*
*Agent 19 - Core patterns, guidelines, templates, and sandbox extraction*
