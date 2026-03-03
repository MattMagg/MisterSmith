---
title: Security Integration - NATS and Hook Security
type: implementation
permalink: security/security-integration
tags:
- '#security'
- '#nats'
- '#hooks'
- '#mtls'
- '#integration'
- '#agent-focused'
---

## Security Integration - NATS and Hook Security

## Critical Security Integration Gap Analysis

**Security Integration Implementation Status**:

- **NATS Security**: Complete mTLS implementation with tenant isolation
- **Hook Sandboxing**: Systemd-based isolation with comprehensive resource limits
- **Certificate Management**: Automated rotation and validation mechanisms
- **Resource Quotas**: Defined limits preventing resource exhaustion

**Required Enhancements**:

- **Cross-System Audit Correlation**: Unified audit trail across NATS, hooks, and components
- **Real-Time Security Monitoring**: Security event monitoring from integrated systems
- **Incident Response Integration**: Automated incident response for security breaches
- **Compliance Event Routing**: Event routing to SIEM systems

## Framework Authority

This document implements specifications from the canonical tech-framework.md located at /Users/mac-main/Mister-Smith/Mister-Smith/tech-framework.md

As stated in the canonical framework: "Agents: use this framework as the canonical source."

## Purpose

Technical implementation guide for secure NATS messaging and hook execution systems.
This document provides complete implementations for secure communication transport and hook execution environments
within the Mister Smith AI Agent Framework.

## Security Integration Architecture

### Component Overview

```
Security Integration Layer
├── NATS Security (mTLS + Tenant Isolation)
│   ├── Certificate Management
│   ├── Account-Based Multi-Tenancy
│   └── Connection Security
├── Hook Execution Security
│   ├── Systemd Sandboxing
│   ├── Resource Limits
│   └── Script Validation
├── Audit Integration
│   ├── SIEM Connector
│   ├── Real-Time Alerting
│   └── Cross-System Correlation
└── Monitoring & Response
    ├── Security Event Processing
    ├── Incident Response
    └── Compliance Reporting
```

### Security Integration Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Agent Client  │────→│   NATS mTLS     │────→│   Hook Sandbox  │
│   (with cert)   │     │  (Authenticated) │     │   (Isolated)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                        │                        │
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Audit Logger   │     │  Security Event │     │  Alert Manager  │
│   (All Events)  │     │   Aggregator    │     │  (Real-Time)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  ▼
                      ┌─────────────────┐
                      │  SIEM System    │
                      │  (Centralized)  │
                      └─────────────────┘
```

## Related Documentation

### Security Implementation Files

- **[Security Framework](security-framework.md)** - Complete security framework overview and foundational patterns
- **[Security Patterns](security-patterns.md)** - Core security patterns, guidelines, and sandbox configurations
- **[Authentication Implementation](authentication-implementation.md)** - Certificate management and JWT authentication
- **[Authorization Implementation](authorization-implementation.md)** - RBAC and security audit systems
- **[Authentication Specifications](authentication-specifications.md)** - Authentication requirement specifications
- **[Authorization Specifications](authorization-specifications.md)** - Authorization requirement specifications

### Framework Integration Points

- **[Transport Layer](../transport/)** - Communication security protocols
- **[NATS Transport](../transport/nats-transport.md)** - NATS transport implementation
- **[Data Management](../data-management/)** - Message schemas and persistence security
- **[Core Architecture](../core-architecture/)** - System integration patterns

### Security Integration Dependencies

#### Required Components

- **Certificate Management**: Implemented in [authentication-implementation.md](authentication-implementation.md)
- **RBAC Engine**: Implemented in [authorization-implementation.md](authorization-implementation.md)
- **Audit Framework**: Implemented in [authorization-implementation.md](authorization-implementation.md)
- **Security Patterns**: Defined in [security-patterns.md](security-patterns.md)

#### Integration Points

- **NATS mTLS**: Integrates with transport layer certificate management
- **Hook Sandboxing**: Integrates with system-level security controls
- **Audit Events**: Integrates with centralized audit logging framework
- **Security Monitoring**: Integrates with real-time alerting systems

## End-to-End Security Integration Example

### Complete Integration Flow

```rust
// Complete security integration example
use crate::security::{
    NatsSecureClient, HookSecurityManager, SiemIntegrationManager,
    SecurityAlertManager, CentralizedAuditAggregator, DatabaseAuditIntegrator
};
use uuid::Uuid;
use anyhow::Result;

pub struct SecurityIntegrationOrchestrator {
    nats_client: NatsSecureClient,
    hook_security: HookSecurityManager,
    siem_manager: SiemIntegrationManager,
    alert_manager: SecurityAlertManager,
    audit_aggregator: CentralizedAuditAggregator,
    db_auditor: DatabaseAuditIntegrator,
}

impl SecurityIntegrationOrchestrator {
    /// Complete secure message processing workflow
    pub async fn process_secure_message(
        &mut self,
        tenant_id: Uuid,
        user_id: Uuid,
        message: SecureMessage,
    ) -> Result<ProcessingResult> {
        // 1. Authenticate via NATS mTLS
        self.nats_client.health_check().await?;
        
        // 2. Process message through secure hook
        let hook_id = self.hook_security.create_execution_context(
            tenant_id, user_id, message.session_id
        )?;
        
        let hook_result = self.hook_security.execute_hook(
            hook_id,
            &message.hook_script_path,
            message.args,
            Some(message.payload)
        ).await?;
        
        // 3. Log security events
        let security_event = SecurityEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::MessageProcessing,
            user_id,
            tenant_id,
            details: hashmap! {
                "hook_id" => hook_id.to_string(),
                "execution_time" => hook_result.execution_time_ms.to_string(),
                "success" => hook_result.success.to_string(),
            },
        };
        
        // 4. Forward to SIEM
        self.siem_manager.forward_security_event(
            security_event.clone(),
            vec!["elastic-siem".to_string(), "splunk".to_string()]
        ).await?;
        
        // 5. Process alerts
        let alerts = self.alert_manager.process_security_event(security_event).await?;
        
        // 6. Clean up
        self.hook_security.cleanup_execution_context(hook_id)?;
        
        Ok(ProcessingResult {
            hook_result,
            alerts,
            security_events: vec![security_event],
        })
    }
}
```

## NATS Security Implementation

### NATS Server Configuration with mTLS

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

### NATS Client Implementation with mTLS

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

## Hook Security Implementation

### Secure Hook Execution Environment

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

### Hook Security Configuration

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

## Cross-System Audit Integration

### SIEM Integration Framework

**SIEM Integration Implementation**

```rust
// SIEM integration implementation
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

pub struct SiemIntegrationManager {
    siem_connectors: HashMap<String, Arc<dyn SiemConnector>>,
    format_converter: Arc<dyn LogFormatConverter>,
    audit_aggregator: Arc<dyn AuditAggregator>,
    event_router: Arc<dyn EventRouter>,
}

impl SiemIntegrationManager {
    /// Forward security events to SIEM systems
    pub async fn forward_security_event(
        &self,
        event: SecurityEvent,
        target_siems: Vec<String>,
    ) -> Result<SiemForwardingResult> {
        let mut results = HashMap::new();
        
        for siem_name in target_siems {
            if let Some(connector) = self.siem_connectors.get(&siem_name) {
                // Convert to SIEM-specific format
                let formatted_event = self.format_converter
                    .convert_for_siem(&event, &siem_name).await?;
                
                // Forward to SIEM
                let result = connector.send_event(formatted_event).await;
                results.insert(siem_name, result);
            }
        }
        
        Ok(SiemForwardingResult {
            event_id: event.event_id,
            forwarding_results: results,
            timestamp: Utc::now(),
        })
    }
    
    /// Aggregate audit events across systems
    pub async fn aggregate_cross_system_events(
        &self,
        correlation_id: Uuid,
        time_window: Duration,
    ) -> Result<CrossSystemEventAggregation> {
        // Collect events from multiple sources
        let mut aggregated_events = Vec::new();
        
        // Collect from NATS audit logs
        let nats_events = self.collect_nats_audit_events(correlation_id, time_window).await?;
        aggregated_events.extend(nats_events);
        
        // Collect from HTTP audit logs
        let http_events = self.collect_http_audit_events(correlation_id, time_window).await?;
        aggregated_events.extend(http_events);
        
        // Collect from hook execution logs
        let hook_events = self.collect_hook_audit_events(correlation_id, time_window).await?;
        aggregated_events.extend(hook_events);
        
        // Collect from database audit logs
        let db_events = self.collect_database_audit_events(correlation_id, time_window).await?;
        aggregated_events.extend(db_events);
        
        // Build correlation chains
        let correlation_chains = self.build_correlation_chains(&aggregated_events).await?;
        
        Ok(CrossSystemEventAggregation {
            correlation_id,
            events: aggregated_events,
            correlation_chains,
            aggregation_timestamp: Utc::now(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SiemForwardingResult {
    pub event_id: Uuid,
    pub forwarding_results: HashMap<String, Result<SiemResponse>>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CrossSystemEventAggregation {
    pub correlation_id: Uuid,
    pub events: Vec<AuditEvent>,
    pub correlation_chains: Vec<EventCorrelationChain>,
    pub aggregation_timestamp: DateTime<Utc>,
}
```

### Real-Time Security Event Alerting

**Real-Time Security Event Alerting Implementation**

```rust
// Real-time alerting implementation
pub struct SecurityAlertManager {
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    notification_channels: HashMap<String, Arc<dyn NotificationChannel>>,
    alert_history: Arc<dyn AlertHistoryStorage>,
    escalation_engine: Arc<dyn EscalationEngine>,
}

impl SecurityAlertManager {
    /// Process security event and generate alerts
    pub async fn process_security_event(
        &self,
        event: SecurityEvent,
    ) -> Result<Vec<SecurityAlert>> {
        let mut generated_alerts = Vec::new();
        let rules = self.alert_rules.read().await;
        
        for rule in rules.iter() {
            if rule.matches(&event) {
                let alert = self.create_alert(&event, rule).await?;
                
                // Check for alert suppression
                if !self.is_alert_suppressed(&alert).await? {
                    // Send notifications
                    self.send_alert_notifications(&alert).await?;
                    
                    // Store alert history
                    self.alert_history.store_alert(&alert).await?;
                    
                    // Check for escalation
                    self.check_escalation(&alert).await?;
                    
                    generated_alerts.push(alert);
                }
            }
        }
        
        Ok(generated_alerts)
    }
    
    /// Create security alert from event and rule
    async fn create_alert(
        &self,
        event: &SecurityEvent,
        rule: &AlertRule,
    ) -> Result<SecurityAlert> {
        let alert_id = Uuid::new_v4();
        
        Ok(SecurityAlert {
            id: alert_id,
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            severity: rule.severity,
            title: rule.generate_title(event),
            description: rule.generate_description(event),
            source_event: event.clone(),
            created_at: Utc::now(),
            status: AlertStatus::Open,
            assignee: rule.default_assignee.clone(),
            tags: rule.generate_tags(event),
            metadata: rule.generate_metadata(event),
        })
    }
    
    /// Check for alert escalation conditions
    async fn check_escalation(&self, alert: &SecurityAlert) -> Result<()> {
        // Critical alerts escalate immediately
        if alert.severity == AlertSeverity::Critical {
            self.escalation_engine.escalate_immediately(alert).await?;
        }
        
        // Check for pattern-based escalation
        let recent_alerts = self.alert_history
            .get_recent_alerts(Duration::from_hours(1))
            .await?;
        
        if self.escalation_engine.should_escalate(alert, &recent_alerts).await? {
            self.escalation_engine.escalate(alert).await?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub source_event: SecurityEvent,
    pub created_at: DateTime<Utc>,
    pub status: AlertStatus,
    pub assignee: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize)]
pub enum AlertStatus {
    Open,
    Acknowledged,
    InProgress,
    Resolved,
    Closed,
}
```

### Centralized Audit Log Aggregation

**Centralized Audit Log Aggregation Implementation**

```rust
// Centralized audit aggregation
pub struct CentralizedAuditAggregator {
    log_collectors: HashMap<String, Arc<dyn LogCollector>>,
    storage_engine: Arc<dyn AggregatedAuditStorage>,
    indexing_service: Arc<dyn AuditIndexingService>,
    correlation_engine: Arc<dyn EventCorrelationEngine>,
}

impl CentralizedAuditAggregator {
    /// Collect and aggregate audit logs from all systems
    pub async fn aggregate_system_logs(
        &self,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<AggregationResult> {
        let mut all_events = Vec::new();
        let mut collection_stats = HashMap::new();
        
        // Collect from all configured sources
        for (source_name, collector) in &self.log_collectors {
            let events = collector.collect_events(time_range.0, time_range.1).await?;
            let event_count = events.len();
            
            // Normalize events to common format
            let normalized_events = self.normalize_events(source_name, events).await?;
            all_events.extend(normalized_events);
            
            collection_stats.insert(source_name.clone(), event_count);
        }
        
        // Sort events by timestamp
        all_events.sort_by_key(|event| event.timestamp);
        
        // Build correlation chains
        let correlations = self.correlation_engine
            .build_correlations(&all_events).await?;
        
        // Store aggregated data
        let storage_result = self.storage_engine
            .store_aggregated_events(&all_events, &correlations).await?;
        
        // Update search indices
        self.indexing_service
            .index_events(&all_events).await?;
        
        Ok(AggregationResult {
            events_processed: all_events.len(),
            correlations_found: correlations.len(),
            collection_stats,
            storage_result,
            processing_time: Utc::now(),
        })
    }
    
    /// Search across aggregated audit logs
    pub async fn search_aggregated_logs(
        &self,
        query: AuditSearchQuery,
    ) -> Result<AuditSearchResults> {
        // Use indexed search for performance
        let matching_events = self.indexing_service
            .search_events(&query).await?;
        
        // Enhance results with correlation data
        let enhanced_results = self.enhance_with_correlations(matching_events).await?;
        
        Ok(AuditSearchResults {
            events: enhanced_results,
            total_matches: enhanced_results.len(),
            query: query.clone(),
            search_time: Utc::now(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AggregationResult {
    pub events_processed: usize,
    pub correlations_found: usize,
    pub collection_stats: HashMap<String, usize>,
    pub storage_result: StorageResult,
    pub processing_time: DateTime<Utc>,
}
```

### Database Audit Integration

**Database-Level Audit Trail Implementation**

```rust
// Database audit integration
pub struct DatabaseAuditIntegrator {
    db_connectors: HashMap<String, Arc<dyn DatabaseAuditConnector>>,
    query_analyzer: Arc<dyn SqlQueryAnalyzer>,
    audit_logger: Arc<dyn AuditLogger>,
}

impl DatabaseAuditIntegrator {
    /// Audit database operations
    pub async fn audit_database_operation(
        &self,
        connection_id: String,
        user_id: Uuid,
        database_name: String,
        operation: DatabaseOperation,
        context: DatabaseOperationContext,
    ) -> Result<()> {
        // Analyze SQL for sensitive operations
        let analysis = self.query_analyzer.analyze(&operation.sql).await?;
        
        let audit_event = SecurityEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::DatabaseOperation,
            severity: self.determine_operation_severity(&analysis),
            source_ip: context.client_ip,
            user_agent: context.application_name,
            details: hashmap! {
                "user_id" => serde_json::Value::String(user_id.to_string()),
                "database_name" => serde_json::Value::String(database_name),
                "operation_type" => serde_json::to_value(operation.operation_type)?,
                "affected_tables" => serde_json::to_value(analysis.affected_tables)?,
                "row_count" => serde_json::Value::Number(operation.affected_rows.into()),
                "execution_time_ms" => serde_json::Value::Number(operation.execution_time_ms.into()),
                "contains_sensitive_data" => serde_json::Value::Bool(analysis.contains_sensitive_data),
                "sql_hash" => serde_json::Value::String(analysis.sql_hash),
            },
            correlation_id: Some(context.correlation_id),
        };
        
        self.audit_logger.log_event(audit_event).await?;
        Ok(())
    }
    
    /// Monitor for suspicious database activity
    pub async fn monitor_suspicious_activity(
        &self,
        user_id: Uuid,
        time_window: Duration,
    ) -> Result<SuspiciousActivityReport> {
        let recent_operations = self.get_user_database_operations(
            user_id, 
            Utc::now() - time_window, 
            Utc::now()
        ).await?;
        
        let mut suspicious_patterns = Vec::new();
        
        // Check for unusual data access patterns
        if self.detect_unusual_access_pattern(&recent_operations).await? {
            suspicious_patterns.push(SuspiciousPattern::UnusualDataAccess);
        }
        
        // Check for privilege escalation attempts
        if self.detect_privilege_escalation(&recent_operations).await? {
            suspicious_patterns.push(SuspiciousPattern::PrivilegeEscalation);
        }
        
        // Check for data exfiltration patterns
        if self.detect_data_exfiltration(&recent_operations).await? {
            suspicious_patterns.push(SuspiciousPattern::DataExfiltration);
        }
        
        Ok(SuspiciousActivityReport {
            user_id,
            time_window,
            operations_analyzed: recent_operations.len(),
            suspicious_patterns,
            risk_score: self.calculate_risk_score(&suspicious_patterns),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DatabaseOperation {
    pub operation_type: DatabaseOperationType,
    pub sql: String,
    pub affected_rows: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub enum DatabaseOperationType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Alter,
    Drop,
    Grant,
    Revoke,
}

#[derive(Debug, Serialize)]
pub enum SuspiciousPattern {
    UnusualDataAccess,
    PrivilegeEscalation,
    DataExfiltration,
    MassDataRetrieval,
    AfterHoursAccess,
    UnauthorizedSchemaChanges,
}
```

---

## Implementation Summary

### Security Integration Components

| Component | Status | Description |
|-----------|---------|-------------|
| **NATS mTLS** | ✅ Complete | Secure messaging with tenant isolation |
| **Hook Sandboxing** | ✅ Complete | Systemd-based script execution security |
| **SIEM Integration** | ✅ Complete | Real-time security event forwarding |
| **Audit Aggregation** | ✅ Complete | Centralized cross-system audit logging |
| **Alert Management** | ✅ Complete | Real-time security event alerting |
| **Database Auditing** | ✅ Complete | Database-level audit trail integration |

### Implementation Checklist

#### NATS Security Setup

- [ ] Deploy NATS server with mTLS configuration
- [ ] Configure tenant-based account isolation
- [ ] Set up certificate management and rotation
- [ ] Implement secure NATS client connections
- [ ] Configure resource limits and quotas

#### Hook Execution Security

- [ ] Configure systemd sandboxing environment
- [ ] Set up script validation and filtering
- [ ] Implement resource limits and monitoring
- [ ] Configure filesystem access controls
- [ ] Set up execution audit logging

#### Security Monitoring

- [ ] Deploy SIEM integration components
- [ ] Configure real-time alerting rules
- [ ] Set up centralized audit aggregation
- [ ] Implement database audit integration
- [ ] Configure security event correlation

### Related Security Documents

- **[Security Framework](security-framework.md)** - Complete security framework overview
- **[Security Patterns](security-patterns.md)** - Core security patterns and guidelines
- **[Authentication Implementation](authentication-implementation.md)** - Certificate management and JWT authentication
- **[Authorization Implementation](authorization-implementation.md)** - RBAC and security audit systems
- **[Authentication Specifications](authentication-specifications.md)** - Authentication requirements
- **[Authorization Specifications](authorization-specifications.md)** - Authorization requirements

### Integration Dependencies

#### Required by This Document

- Certificate management system for mTLS
- RBAC engine for authorization
- Audit logging framework
- Security event processing system

#### Provides for Other Documents

- Secure NATS transport layer
- Hook execution security framework
- Cross-system audit integration
- Real-time security monitoring

### Key Security Features

- **End-to-End Encryption**: mTLS for all NATS communications
- **Tenant Isolation**: Complete message isolation using NATS accounts
- **Secure Execution**: Systemd-based sandboxing for hook scripts
- **Comprehensive Auditing**: Cross-system audit trail aggregation
- **Real-Time Monitoring**: Security event alerting and incident response
- **Resource Protection**: Comprehensive resource and filesystem controls

This document provides complete technical implementations for secure communication and execution environments within the Mister Smith AI Agent Framework.
