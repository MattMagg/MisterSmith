# Process Management Specifications

## Process Supervision and Lifecycle Management Framework

### Overview

This document provides comprehensive process management specifications for the Mister Smith AI Agent Framework. It defines systemd integration,
process supervision patterns, lifecycle management strategies, resource management policies, and health monitoring systems for distributed agent operations.

**Related Documentation**:

- [Configuration Management](configuration-management.md) - Configuration patterns and deployment strategies
- [Deployment Architecture](deployment-architecture-specifications.md) - Infrastructure and deployment patterns
- [Observability Framework](observability-monitoring-framework.md) - Monitoring and observability specifications
- [Build Specifications](build-specifications.md) - Build and packaging specifications

### Key Features

- **Systemd Integration**: Native service management with proper dependencies and resource limits
- **Hierarchical Supervision Trees**: Configurable restart policies and fault tolerance
- **CGroups v2 Integration**: Dynamic resource management and enforcement
- **Multi-Level Health Monitoring**: HTTP, TCP, process, database, and custom health checks
- **Lifecycle Management**: Graceful startup/shutdown with state management
- **Security Hardening**: Comprehensive security features through systemd
- **Service Discovery**: NATS-based communication with service registry patterns

---

## 1. Executive Summary

### 1.1 Process Management Philosophy

- **Systemd-First Approach**: Native integration with systemd for service management and supervision
- **Multi-Tier Process Architecture**: Support for tier_1 (experimental), tier_2 (validation), and tier_3 (operational) process patterns
- **Fault-Tolerant Supervision**: Hierarchical supervision trees with configurable restart policies
- **Resource-Aware Management**: Dynamic resource allocation and enforcement using CGroups v2
- **Observable Lifecycle**: Complete lifecycle event tracking and monitoring with structured logging
- **Security-First Design**: Process isolation, privilege separation, and secure configuration management
- **Agent-Optimized Operations**: Structured process management patterns designed for multi-agent orchestration

### 1.2 Process Management Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                   PROCESS MANAGEMENT LAYERS                │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Health Monitoring & Alerting (Systemd + Custom)  │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Resource Management (CGroups + Limits)            │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Process Supervision (Systemd + Supervision Trees) │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Service Lifecycle (Systemd Units + Dependencies) │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Systemd Integration

### 2.1 Service Unit Specifications

#### 2.1.1 Base Orchestrator Service

```ini
# /etc/systemd/system/mister-smith-orchestrator.service
[Unit]
Description=Mister Smith AI Agent Framework - Orchestrator
Documentation=https://github.com/mister-smith/framework/docs
Wants=network-online.target
After=network-online.target
After=nats.service
After=postgresql.service
After=redis.service
Requires=mister-smith-messaging.service
PartOf=mister-smith.target

# Health check dependencies
Requisite=mister-smith-health-check.service
Before=mister-smith-worker@.service

[Service]
Type=notify
ExecStart=/usr/local/bin/mister-smith-orchestrator
ExecReload=/bin/kill -s HUP $MAINPID
ExecStop=/bin/kill -s TERM $MAINPID
TimeoutStartSec=60
TimeoutStopSec=30
RestartSec=5
Restart=always
RestartPreventExitStatus=100

# Process management
User=mister-smith
Group=mister-smith
UMask=0022
WorkingDirectory=/app
Environment=AGENT_TYPE=orchestrator
Environment=ENVIRONMENT_TIER=tier_3
EnvironmentFile=-/etc/mister-smith/orchestrator.env

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
LimitCORE=0

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/app/data /app/logs /tmp
PrivateTmp=true
PrivateDevices=true
ProtectControlGroups=true
ProtectKernelModules=true
ProtectKernelTunables=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# Memory management
MemoryAccounting=true
MemoryMax=2G
MemorySwapMax=0

# CPU management
CPUAccounting=true
CPUQuota=200%
CPUShares=1024

# I/O management
IOAccounting=true
IOWeight=100

# Watchdog configuration
WatchdogSec=30
NotifyAccess=main

[Install]
WantedBy=mister-smith.target
Alias=orchestrator.service
```

#### 2.1.2 Worker Service Template

```ini
# /etc/systemd/system/mister-smith-worker@.service
[Unit]
Description=Mister Smith AI Agent Framework - Worker %i
Documentation=https://github.com/mister-smith/framework/docs
Wants=network-online.target
After=network-online.target
After=mister-smith-orchestrator.service
Requires=mister-smith-messaging.service
PartOf=mister-smith.target

# Worker-specific dependencies
BindsTo=mister-smith-orchestrator.service
After=mister-smith-orchestrator.service

[Service]
Type=notify
ExecStart=/usr/local/bin/mister-smith-worker --worker-id=%i
ExecReload=/bin/kill -s HUP $MAINPID
ExecStop=/bin/kill -s TERM $MAINPID
TimeoutStartSec=45
TimeoutStopSec=20
RestartSec=5
Restart=always
RestartPreventExitStatus=100

# Process management
User=mister-smith
Group=mister-smith
UMask=0022
WorkingDirectory=/app
Environment=AGENT_TYPE=worker
Environment=WORKER_ID=%i
Environment=ENVIRONMENT_TIER=tier_3
EnvironmentFile=-/etc/mister-smith/worker.env

# Resource limits (per worker)
LimitNOFILE=16384
LimitNPROC=1024
LimitCORE=0

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/app/workspace /app/logs /tmp
PrivateTmp=true
PrivateDevices=true
ProtectControlGroups=true
ProtectKernelModules=true
ProtectKernelTunables=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# Memory management (per worker)
MemoryAccounting=true
MemoryMax=1G
MemorySwapMax=0

# CPU management (per worker)
CPUAccounting=true
CPUQuota=100%
CPUShares=512

# I/O management
IOAccounting=true
IOWeight=50

# Watchdog configuration
WatchdogSec=20
NotifyAccess=main

[Install]
WantedBy=mister-smith.target
```

#### 2.1.3 Messaging Service

```ini
# /etc/systemd/system/mister-smith-messaging.service
[Unit]
Description=Mister Smith AI Agent Framework - Messaging (NATS)
Documentation=https://github.com/mister-smith/framework/docs
Wants=network-online.target
After=network-online.target
PartOf=mister-smith.target

[Service]
Type=notify
ExecStart=/usr/local/bin/nats-server -c /etc/mister-smith/nats.conf
ExecReload=/bin/kill -s HUP $MAINPID
ExecStop=/bin/kill -s TERM $MAINPID
TimeoutStartSec=30
TimeoutStopSec=15
RestartSec=3
Restart=always

# Process management
User=nats
Group=nats
UMask=0022
WorkingDirectory=/var/lib/nats
Environment=NATS_CONFIG=/etc/mister-smith/nats.conf
EnvironmentFile=-/etc/mister-smith/messaging.env

# Resource limits
LimitNOFILE=65536
LimitNPROC=2048
LimitCORE=0

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/nats /var/log/nats
PrivateTmp=true
PrivateDevices=true
ProtectControlGroups=true
ProtectKernelModules=true
ProtectKernelTunables=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# Memory management
MemoryAccounting=true
MemoryMax=1G
MemorySwapMax=0

# CPU management
CPUAccounting=true
CPUQuota=150%
CPUShares=768

# Network configuration
PrivateNetwork=false
IPAccounting=true

# Watchdog configuration
WatchdogSec=25
NotifyAccess=main

[Install]
WantedBy=mister-smith.target
```

#### 2.1.4 Health Check Service

```ini
# /etc/systemd/system/mister-smith-health-check.service
[Unit]
Description=Mister Smith AI Agent Framework - Health Check Monitor
Documentation=https://github.com/mister-smith/framework/docs
Wants=network-online.target
After=network-online.target
PartOf=mister-smith.target

[Service]
Type=simple
ExecStart=/usr/local/bin/mister-smith-health-monitor
Restart=always
RestartSec=5
TimeoutStartSec=15
TimeoutStopSec=10

# Process management
User=mister-smith
Group=mister-smith
UMask=0022
WorkingDirectory=/app
Environment=HEALTH_CHECK_INTERVAL=15
Environment=HEALTH_CHECK_TIMEOUT=5
EnvironmentFile=-/etc/mister-smith/health.env

# Resource limits
LimitNOFILE=1024
LimitNPROC=256
LimitCORE=0

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/app/logs /tmp
PrivateTmp=true
PrivateDevices=true
ProtectControlGroups=true
ProtectKernelModules=true
ProtectKernelTunables=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# Memory management
MemoryAccounting=true
MemoryMax=256M
MemorySwapMax=0

# CPU management
CPUAccounting=true
CPUQuota=25%
CPUShares=256

[Install]
WantedBy=mister-smith.target
```

### 2.2 Target Configuration

#### 2.2.1 System Target

```ini
# /etc/systemd/system/mister-smith.target
[Unit]
Description=Mister Smith AI Agent Framework
Documentation=https://github.com/mister-smith/framework/docs
Wants=multi-user.target
After=multi-user.target
AllowIsolate=true

[Install]
WantedBy=multi-user.target
Alias=agent-framework.target
```

#### 2.2.2 Service Dependencies

```ini
# /etc/systemd/system/mister-smith-dependencies.target
[Unit]
Description=Mister Smith AI Agent Framework - External Dependencies
Documentation=https://github.com/mister-smith/framework/docs
Wants=postgresql.service redis.service
After=postgresql.service redis.service
PartOf=mister-smith.target

[Install]
WantedBy=mister-smith.target
```

### 2.3 Systemd Configuration Management

#### 2.3.1 Environment Files

```bash
# /etc/mister-smith/orchestrator.env
MISTER_SMITH_ENVIRONMENT_TIER=tier_3
MISTER_SMITH_LOG_LEVEL=info
MISTER_SMITH_CONFIG_PATH=/etc/mister-smith
MISTER_SMITH_AGENT_TYPE=orchestrator
MISTER_SMITH_AGENT_TIER=premium
MISTER_SMITH_AGENT_ID=orchestrator-primary

# NATS Configuration
MISTER_SMITH_NATS_URL=nats://localhost:4222
MISTER_SMITH_NATS_CLUSTER_ID=mister-smith-cluster
MISTER_SMITH_NATS_CLIENT_ID=orchestrator-primary
MISTER_SMITH_NATS_TLS_ENABLED=true

# Database Configuration
MISTER_SMITH_DB_ENABLED=true
MISTER_SMITH_DB_URL=postgresql://mister_smith:${DB_PASSWORD}@localhost:5432/mister_smith
MISTER_SMITH_DB_MAX_CONNECTIONS=20

# Resource Limits
MISTER_SMITH_RUNTIME_MAX_BLOCKING_THREADS=512
MISTER_SMITH_AGENT_MAX_CONCURRENT_TASKS=1000

# Security
MISTER_SMITH_SECURITY_ENABLED=true
MISTER_SMITH_AUTH_JWT_SECRET_FILE=/etc/mister-smith/secrets/jwt.key
```

```bash
# /etc/mister-smith/worker.env
MISTER_SMITH_ENVIRONMENT_TIER=tier_3
MISTER_SMITH_LOG_LEVEL=info
MISTER_SMITH_CONFIG_PATH=/etc/mister-smith
MISTER_SMITH_AGENT_TYPE=worker
MISTER_SMITH_AGENT_TIER=standard

# NATS Configuration
MISTER_SMITH_NATS_URL=nats://localhost:4222
MISTER_SMITH_NATS_CLUSTER_ID=mister-smith-cluster
MISTER_SMITH_NATS_TLS_ENABLED=true

# Worker Configuration
MISTER_SMITH_WORKER_CONCURRENCY=4
MISTER_SMITH_WORKER_TIMEOUT_SECONDS=300
MISTER_SMITH_WORKER_WORKSPACE_DIR=/app/workspace

# Claude CLI Integration
MISTER_SMITH_CLAUDE_PARALLEL_DEFAULT=4
MISTER_SMITH_CLAUDE_PARALLEL_MAX_AGENTS=50
MISTER_SMITH_CLAUDE_PARALLEL_CPU_BUDGET=4.0
MISTER_SMITH_CLAUDE_PARALLEL_MEMORY_BUDGET=4Gi
```

#### 2.3.2 Systemd Drop-ins for Environment-Specific Configuration

```ini
# /etc/systemd/system/mister-smith-orchestrator.service.d/tier-3.conf
[Unit]
Description=Mister Smith AI Agent Framework - Orchestrator (Tier 3)

[Service]
Environment=ENVIRONMENT_TIER=tier_3
Environment=LOG_LEVEL=warn
MemoryMax=4G
CPUQuota=400%
```

```ini
# /etc/systemd/system/mister-smith-worker@.service.d/tier-3.conf
[Unit]
Description=Mister Smith AI Agent Framework - Worker %i (Tier 3)

[Service]
Environment=ENVIRONMENT_TIER=tier_3
Environment=LOG_LEVEL=warn
MemoryMax=2G
CPUQuota=200%
```

---

## 3. Process Supervision Patterns

### 3.1 Hierarchical Supervision Architecture

#### 3.1.1 Supervision Tree Implementation

```rust
// src/supervision/mod.rs - Process supervision implementation
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::process::{Child, Command};
use tokio::time::{interval, sleep};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSupervisionConfig {
    pub restart_policy: RestartPolicy,
    pub max_restart_attempts: u32,
    pub restart_window: Duration,
    pub health_check_interval: Duration,
    pub failure_escalation: EscalationPolicy,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    Always,
    OnFailure,
    UnlessStopped,
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationPolicy {
    RestartProcess,
    RestartService,
    NotifyOperator,
    FailoverToSecondary,
    GracefulShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_file_descriptors: Option<u64>,
    pub max_processes: Option<u64>,
    pub io_weight: Option<u16>,
}

#[derive(Debug)]
pub struct ProcessSupervisor {
    process_id: String,
    config: ProcessSupervisionConfig,
    process_handle: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
    last_restart: Arc<Mutex<Option<Instant>>>,
    health_monitor: Arc<ProcessHealthMonitor>,
    systemd_integration: SystemdIntegration,
}

impl ProcessSupervisor {
    pub fn new(
        process_id: String,
        config: ProcessSupervisionConfig,
        systemd_integration: SystemdIntegration,
    ) -> Self {
        let health_monitor = Arc::new(ProcessHealthMonitor::new(
            process_id.clone(),
            config.health_check_interval,
        ));

        Self {
            process_id,
            config,
            process_handle: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
            last_restart: Arc::new(Mutex::new(None)),
            health_monitor,
            systemd_integration,
        }
    }

    pub async fn start_supervision(&self) -> Result<(), SupervisionError> {
        info!("Starting process supervision for {}", self.process_id);

        // Register with systemd if available
        self.systemd_integration.register_service(&self.process_id).await?;

        // Start initial process
        self.start_process().await?;

        // Start health monitoring
        self.start_health_monitoring().await?;

        // Start restart monitoring
        self.start_restart_monitoring().await?;

        Ok(())
    }

    async fn start_process(&self) -> Result<(), SupervisionError> {
        let mut process_handle = self.process_handle.lock().await;

        if process_handle.is_some() {
            return Err(SupervisionError::ProcessAlreadyRunning);
        }

        // Apply resource limits using systemd or cgroups
        let mut command = self.build_process_command().await?;
        self.apply_resource_limits(&mut command).await?;

        let child = command.spawn()?;
        let pid = child.id().ok_or(SupervisionError::ProcessStartFailed)?;

        info!("Started process {} with PID {}", self.process_id, pid);

        // Notify systemd about the process
        self.systemd_integration.notify_process_started(pid).await?;

        *process_handle = Some(child);
        Ok(())
    }

    async fn build_process_command(&self) -> Result<Command, SupervisionError> {
        let mut command = match self.process_id.as_str() {
            id if id.starts_with("orchestrator") => {
                let mut cmd = Command::new("/usr/local/bin/mister-smith-orchestrator");
                cmd.env("AGENT_TYPE", "orchestrator");
                cmd.env("AGENT_ID", id);
                cmd
            }
            id if id.starts_with("worker") => {
                let mut cmd = Command::new("/usr/local/bin/mister-smith-worker");
                cmd.env("AGENT_TYPE", "worker");
                cmd.env("AGENT_ID", id);
                if let Some(worker_id) = id.strip_prefix("worker-") {
                    cmd.arg("--worker-id").arg(worker_id);
                }
                cmd
            }
            id if id.starts_with("messaging") => {
                let mut cmd = Command::new("/usr/local/bin/nats-server");
                cmd.arg("-c").arg("/etc/mister-smith/nats.conf");
                cmd.env("NATS_CONFIG", "/etc/mister-smith/nats.conf");
                cmd
            }
            _ => return Err(SupervisionError::UnknownProcessType),
        };

        // Common environment setup
        command.env("ENVIRONMENT_TIER", "tier_3");
        command.env("LOG_LEVEL", "info");
        command.env("CONFIG_PATH", "/etc/mister-smith");

        Ok(command)
    }

    async fn apply_resource_limits(&self, command: &mut Command) -> Result<(), SupervisionError> {
        // Set working directory
        command.current_dir("/app");

        // Environment-based resource hints (actual limits set by systemd)
        if let Some(max_memory) = self.config.resource_limits.max_memory_mb {
            command.env("MEMORY_LIMIT_MB", max_memory.to_string());
        }

        if let Some(max_cpu) = self.config.resource_limits.max_cpu_percent {
            command.env("CPU_LIMIT_PERCENT", max_cpu.to_string());
        }

        Ok(())
    }

    async fn start_health_monitoring(&self) -> Result<(), SupervisionError> {
        let health_monitor = self.health_monitor.clone();
        let process_handle = self.process_handle.clone();
        let supervisor_weak = Arc::downgrade(&Arc::new(self.clone()));

        tokio::spawn(async move {
            let mut health_interval = interval(health_monitor.check_interval);

            loop {
                health_interval.tick().await;

                if let Some(supervisor) = supervisor_weak.upgrade() {
                    let health_status = health_monitor.check_health().await;
                    
                    match health_status {
                        HealthStatus::Healthy => {
                            // Update systemd with health status
                            if let Err(e) = supervisor.systemd_integration.send_heartbeat().await {
                                warn!("Failed to send systemd heartbeat: {}", e);
                            }
                        }
                        HealthStatus::Unhealthy(reason) => {
                            warn!("Process {} unhealthy: {}", supervisor.process_id, reason);
                            
                            if let Err(e) = supervisor.handle_process_failure(reason).await {
                                error!("Failed to handle process failure: {}", e);
                            }
                        }
                        HealthStatus::Dead => {
                            error!("Process {} detected as dead", supervisor.process_id);
                            
                            if let Err(e) = supervisor.handle_process_death().await {
                                error!("Failed to handle process death: {}", e);
                            }
                        }
                    }
                } else {
                    break; // Supervisor has been dropped
                }
            }
        });

        Ok(())
    }

    async fn handle_process_failure(&self, reason: String) -> Result<(), SupervisionError> {
        warn!("Handling process failure for {}: {}", self.process_id, reason);

        match self.config.restart_policy {
            RestartPolicy::Never => {
                info!("Process {} configured to never restart", self.process_id);
                return Ok(());
            }
            RestartPolicy::Always => {
                self.restart_process().await?;
            }
            RestartPolicy::OnFailure => {
                // Check if this was actually a failure vs graceful shutdown
                if self.was_graceful_shutdown().await {
                    return Ok(());
                }
                self.restart_process().await?;
            }
            RestartPolicy::UnlessStopped => {
                if self.was_manually_stopped().await {
                    return Ok(());
                }
                self.restart_process().await?;
            }
            RestartPolicy::ExponentialBackoff { initial_delay, max_delay, multiplier } => {
                self.restart_with_backoff(initial_delay, max_delay, multiplier).await?;
            }
        }

        Ok(())
    }

    async fn restart_process(&self) -> Result<(), SupervisionError> {
        let mut restart_count = self.restart_count.lock().await;
        let mut last_restart = self.last_restart.lock().await;

        // Check restart limits
        if *restart_count >= self.config.max_restart_attempts {
            error!("Maximum restart attempts ({}) exceeded for {}", 
                   self.config.max_restart_attempts, self.process_id);
            return self.escalate_failure().await;
        }

        // Check restart window
        if let Some(last) = *last_restart {
            if last.elapsed() < self.config.restart_window {
                warn!("Process {} restarting too frequently", self.process_id);
                return self.escalate_failure().await;
            } else {
                // Reset restart count if outside window
                *restart_count = 0;
            }
        }

        info!("Restarting process {} (attempt {})", self.process_id, *restart_count + 1);

        // Stop current process
        self.stop_process().await?;

        // Wait a bit before restart
        sleep(Duration::from_secs(1)).await;

        // Start new process
        self.start_process().await?;

        *restart_count += 1;
        *last_restart = Some(Instant::now());

        // Notify systemd of restart
        self.systemd_integration.notify_process_restarted().await?;

        Ok(())
    }

    async fn escalate_failure(&self) -> Result<(), SupervisionError> {
        error!("Escalating failure for process {}", self.process_id);

        match self.config.failure_escalation {
            EscalationPolicy::RestartProcess => {
                // Force restart regardless of limits
                self.force_restart().await?;
            }
            EscalationPolicy::RestartService => {
                // Restart entire systemd service
                self.systemd_integration.restart_service(&self.process_id).await?;
            }
            EscalationPolicy::NotifyOperator => {
                // Send alert to monitoring system
                self.send_operator_alert().await?;
            }
            EscalationPolicy::FailoverToSecondary => {
                // Switch to secondary instance
                self.initiate_failover().await?;
            }
            EscalationPolicy::GracefulShutdown => {
                // Shutdown gracefully
                self.graceful_shutdown().await?;
            }
        }

        Ok(())
    }

    async fn stop_process(&self) -> Result<(), SupervisionError> {
        let mut process_handle = self.process_handle.lock().await;

        if let Some(mut child) = process_handle.take() {
            let pid = child.id().unwrap_or(0);
            info!("Stopping process {} (PID: {})", self.process_id, pid);

            // Send SIGTERM first
            if let Err(e) = child.kill().await {
                warn!("Failed to send SIGTERM to process {}: {}", self.process_id, e);
            }

            // Wait for graceful shutdown
            match tokio::time::timeout(Duration::from_secs(30), child.wait()).await {
                Ok(Ok(exit_status)) => {
                    info!("Process {} exited with status: {}", self.process_id, exit_status);
                }
                Ok(Err(e)) => {
                    error!("Error waiting for process {} to exit: {}", self.process_id, e);
                }
                Err(_) => {
                    warn!("Process {} did not exit gracefully, sending SIGKILL", self.process_id);
                    // SIGKILL would need to be sent via system command
                    // as tokio doesn't provide direct access
                }
            }

            // Notify systemd about process stop
            self.systemd_integration.notify_process_stopped().await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Dead,
}

#[derive(Debug)]
pub struct ProcessHealthMonitor {
    process_id: String,
    check_interval: Duration,
    last_health_check: Arc<Mutex<Option<Instant>>>,
}

impl ProcessHealthMonitor {
    pub fn new(process_id: String, check_interval: Duration) -> Self {
        Self {
            process_id,
            check_interval,
            last_health_check: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn check_health(&self) -> HealthStatus {
        let mut last_check = self.last_health_check.lock().await;
        let now = Instant::now();

        // Basic process existence check
        if !self.is_process_running().await {
            return HealthStatus::Dead;
        }

        // HTTP health check for agents with health endpoints
        if let Some(health_url) = self.get_health_endpoint() {
            match self.http_health_check(&health_url).await {
                Ok(healthy) => {
                    if healthy {
                        *last_check = Some(now);
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Unhealthy("HTTP health check failed".to_string())
                    }
                }
                Err(e) => HealthStatus::Unhealthy(format!("Health check error: {}", e)),
            }
        } else {
            // For processes without health endpoints, check resource usage
            match self.resource_health_check().await {
                Ok(healthy) => {
                    if healthy {
                        *last_check = Some(now);
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Unhealthy("Resource usage unhealthy".to_string())
                    }
                }
                Err(e) => HealthStatus::Unhealthy(format!("Resource check error: {}", e)),
            }
        }
    }

    async fn is_process_running(&self) -> bool {
        // Check if process is still running via systemd or process table
        match tokio::process::Command::new("systemctl")
            .args(&["is-active", &format!("mister-smith-{}.service", self.process_id)])
            .output()
            .await
        {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout);
                status.trim() == "active"
            }
            Err(_) => false,
        }
    }

    fn get_health_endpoint(&self) -> Option<String> {
        match self.process_id.as_str() {
            id if id.starts_with("orchestrator") => Some("http://localhost:8080/health".to_string()),
            id if id.starts_with("worker") => {
                // Workers typically run on 8081+ ports
                Some("http://localhost:8081/health".to_string())
            }
            _ => None,
        }
    }

    async fn http_health_check(&self, url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let response = client.get(url).send().await?;
        Ok(response.status().is_success())
    }

    async fn resource_health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Check memory and CPU usage via systemd
        let output = tokio::process::Command::new("systemctl")
            .args(&["show", &format!("mister-smith-{}.service", self.process_id), 
                   "--property=MemoryCurrent,CPUUsageNSec"])
            .output()
            .await?;

        let status_output = String::from_utf8_lossy(&output.stdout);
        
        // Parse memory and CPU usage
        // This is a simplified check - in practice you'd parse the actual values
        // and compare against configured thresholds
        Ok(!status_output.contains("MemoryCurrent=18446744073709551615")) // Max value indicates error
    }
}

#[derive(Debug)]
pub struct SystemdIntegration {
    service_name: String,
}

impl SystemdIntegration {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    pub async fn register_service(&self, process_id: &str) -> Result<(), SupervisionError> {
        // Notify systemd that we're ready (if running under systemd)
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("--ready")
                .output()
                .await;
        }
        Ok(())
    }

    pub async fn notify_process_started(&self, pid: u32) -> Result<(), SupervisionError> {
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg(&format!("MAINPID={}", pid))
                .output()
                .await;
        }
        Ok(())
    }

    pub async fn send_heartbeat(&self) -> Result<(), SupervisionError> {
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("WATCHDOG=1")
                .output()
                .await;
        }
        Ok(())
    }

    pub async fn restart_service(&self, process_id: &str) -> Result<(), SupervisionError> {
        let service_name = format!("mister-smith-{}.service", process_id);
        
        let output = tokio::process::Command::new("systemctl")
            .args(&["restart", &service_name])
            .output()
            .await
            .map_err(|e| SupervisionError::SystemdError(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SupervisionError::SystemdError(error.to_string()));
        }

        Ok(())
    }

    pub async fn notify_process_stopped(&self) -> Result<(), SupervisionError> {
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("STOPPING=1")
                .output()
                .await;
        }
        Ok(())
    }

    pub async fn notify_process_restarted(&self) -> Result<(), SupervisionError> {
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("RELOADING=1")
                .output()
                .await;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    #[error("Process already running")]
    ProcessAlreadyRunning,
    #[error("Process start failed")]
    ProcessStartFailed,
    #[error("Unknown process type")]
    UnknownProcessType,
    #[error("Systemd error: {0}")]
    SystemdError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

#### 3.1.2 Restart Policy Implementation

```rust
// src/supervision/restart_policies.rs
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RestartPolicyManager {
    policies: HashMap<String, RestartPolicyConfig>,
    restart_history: Arc<Mutex<HashMap<String, VecDeque<RestartEvent>>>>,
}

#[derive(Debug, Clone)]
pub struct RestartPolicyConfig {
    pub policy_type: RestartPolicyType,
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone)]
pub enum RestartPolicyType {
    Immediate,
    FixedDelay(Duration),
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        jitter: bool,
    },
    LinearBackoff {
        initial_delay: Duration,
        increment: Duration,
        max_delay: Duration,
    },
    AdaptiveBackoff {
        base_delay: Duration,
        success_threshold: u32,
        failure_threshold: u32,
    },
}

#[derive(Debug, Clone)]
pub struct EscalationRule {
    pub condition: EscalationCondition,
    pub action: EscalationAction,
    pub delay: Duration,
}

#[derive(Debug, Clone)]
pub enum EscalationCondition {
    ConsecutiveFailures(u32),
    FailuresInWindow { count: u32, window: Duration },
    TotalFailures(u32),
    ContinuousFailures(Duration),
}

#[derive(Debug, Clone)]
pub enum EscalationAction {
    NotifyOperator,
    RestartService,
    FailoverToBackup,
    GracefulShutdown,
    ForceRestart,
    ScaleHorizontally,
}

#[derive(Debug, Clone)]
pub struct RestartEvent {
    pub timestamp: Instant,
    pub reason: String,
    pub success: bool,
    pub delay: Duration,
}

impl RestartPolicyManager {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            restart_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_policy(&mut self, process_id: String, config: RestartPolicyConfig) {
        self.policies.insert(process_id, config);
    }

    pub async fn should_restart(&self, process_id: &str, failure_reason: &str) -> RestartDecision {
        let config = match self.policies.get(process_id) {
            Some(config) => config,
            None => return RestartDecision::NoPolicy,
        };

        let mut history = self.restart_history.lock().await;
        let process_history = history.entry(process_id.to_string()).or_insert_with(VecDeque::new);

        // Clean old entries outside restart window
        let cutoff = Instant::now() - config.restart_window;
        while let Some(front) = process_history.front() {
            if front.timestamp < cutoff {
                process_history.pop_front();
            } else {
                break;
            }
        }

        // Check if max restarts exceeded
        if process_history.len() >= config.max_restarts as usize {
            return self.evaluate_escalation(process_id, config, process_history).await;
        }

        // Calculate restart delay
        let delay = self.calculate_restart_delay(config, process_history);

        RestartDecision::Restart { delay }
    }

    pub async fn record_restart(&self, process_id: &str, reason: String, success: bool, delay: Duration) {
        let mut history = self.restart_history.lock().await;
        let process_history = history.entry(process_id.to_string()).or_insert_with(VecDeque::new);

        let event = RestartEvent {
            timestamp: Instant::now(),
            reason,
            success,
            delay,
        };

        process_history.push_back(event);

        // Keep only recent history
        if process_history.len() > 100 {
            process_history.pop_front();
        }
    }

    fn calculate_restart_delay(&self, config: &RestartPolicyConfig, history: &VecDeque<RestartEvent>) -> Duration {
        match &config.policy_type {
            RestartPolicyType::Immediate => Duration::from_secs(0),
            
            RestartPolicyType::FixedDelay(delay) => *delay,
            
            RestartPolicyType::ExponentialBackoff { initial_delay, max_delay, multiplier, jitter } => {
                let attempt = history.len() as u32;
                let mut delay = initial_delay.as_secs_f64() * multiplier.powi(attempt as i32);
                
                if delay > max_delay.as_secs_f64() {
                    delay = max_delay.as_secs_f64();
                }

                if *jitter {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let jitter_factor = rng.gen_range(0.5..1.5);
                    delay *= jitter_factor;
                }

                Duration::from_secs_f64(delay)
            }
            
            RestartPolicyType::LinearBackoff { initial_delay, increment, max_delay } => {
                let attempt = history.len() as u32;
                let delay = initial_delay.as_secs() + (increment.as_secs() * attempt as u64);
                let delay = std::cmp::min(delay, max_delay.as_secs());
                Duration::from_secs(delay)
            }
            
            RestartPolicyType::AdaptiveBackoff { base_delay, success_threshold, failure_threshold } => {
                let recent_failures = history.iter()
                    .rev()
                    .take(*failure_threshold as usize)
                    .filter(|e| !e.success)
                    .count();

                let recent_successes = history.iter()
                    .rev()
                    .take(*success_threshold as usize)
                    .filter(|e| e.success)
                    .count();

                if recent_successes >= *success_threshold as usize {
                    // Decrease delay for successful restarts
                    Duration::from_secs(base_delay.as_secs() / 2)
                } else if recent_failures >= *failure_threshold as usize {
                    // Increase delay for repeated failures
                    Duration::from_secs(base_delay.as_secs() * 2)
                } else {
                    *base_delay
                }
            }
        }
    }

    async fn evaluate_escalation(
        &self,
        process_id: &str,
        config: &RestartPolicyConfig,
        history: &VecDeque<RestartEvent>,
    ) -> RestartDecision {
        for rule in &config.escalation_rules {
            if self.matches_escalation_condition(&rule.condition, history) {
                return RestartDecision::Escalate {
                    action: rule.action.clone(),
                    delay: rule.delay,
                };
            }
        }

        RestartDecision::MaxRestartsExceeded
    }

    fn matches_escalation_condition(&self, condition: &EscalationCondition, history: &VecDeque<RestartEvent>) -> bool {
        match condition {
            EscalationCondition::ConsecutiveFailures(count) => {
                history.iter()
                    .rev()
                    .take(*count as usize)
                    .all(|e| !e.success)
                    && history.len() >= *count as usize
            }
            
            EscalationCondition::FailuresInWindow { count, window } => {
                let cutoff = Instant::now() - *window;
                let failures = history.iter()
                    .filter(|e| e.timestamp >= cutoff && !e.success)
                    .count();
                failures >= *count as usize
            }
            
            EscalationCondition::TotalFailures(count) => {
                history.iter().filter(|e| !e.success).count() >= *count as usize
            }
            
            EscalationCondition::ContinuousFailures(duration) => {
                let cutoff = Instant::now() - *duration;
                history.iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .all(|e| !e.success)
            }
        }
    }
}

#[derive(Debug)]
pub enum RestartDecision {
    Restart { delay: Duration },
    Escalate { action: EscalationAction, delay: Duration },
    MaxRestartsExceeded,
    NoPolicy,
}

// Configuration examples for different service types
impl RestartPolicyManager {
    pub fn create_orchestrator_policy() -> RestartPolicyConfig {
        RestartPolicyConfig {
            policy_type: RestartPolicyType::ExponentialBackoff {
                initial_delay: Duration::from_secs(5),
                max_delay: Duration::from_secs(300),
                multiplier: 2.0,
                jitter: true,
            },
            max_restarts: 5,
            restart_window: Duration::from_secs(3600), // 1 hour
            escalation_rules: vec![
                EscalationRule {
                    condition: EscalationCondition::ConsecutiveFailures(3),
                    action: EscalationAction::NotifyOperator,
                    delay: Duration::from_secs(0),
                },
                EscalationRule {
                    condition: EscalationCondition::ConsecutiveFailures(5),
                    action: EscalationAction::FailoverToBackup,
                    delay: Duration::from_secs(30),
                },
            ],
        }
    }

    pub fn create_worker_policy() -> RestartPolicyConfig {
        RestartPolicyConfig {
            policy_type: RestartPolicyType::LinearBackoff {
                initial_delay: Duration::from_secs(2),
                increment: Duration::from_secs(3),
                max_delay: Duration::from_secs(60),
            },
            max_restarts: 10,
            restart_window: Duration::from_secs(1800), // 30 minutes
            escalation_rules: vec![
                EscalationRule {
                    condition: EscalationCondition::FailuresInWindow {
                        count: 5,
                        window: Duration::from_secs(300), // 5 minutes
                    },
                    action: EscalationAction::ScaleHorizontally,
                    delay: Duration::from_secs(10),
                },
            ],
        }
    }

    pub fn create_messaging_policy() -> RestartPolicyConfig {
        RestartPolicyConfig {
            policy_type: RestartPolicyType::FixedDelay(Duration::from_secs(10)),
            max_restarts: 3,
            restart_window: Duration::from_secs(900), // 15 minutes
            escalation_rules: vec![
                EscalationRule {
                    condition: EscalationCondition::TotalFailures(3),
                    action: EscalationAction::GracefulShutdown,
                    delay: Duration::from_secs(60),
                },
            ],
        }
    }
}
```

### 3.2 Process Lifecycle Management

#### 3.2.1 Graceful Shutdown Implementation

```rust
// src/lifecycle/graceful_shutdown.rs
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot, broadcast};
use tokio::signal;
use tokio::time::{timeout, sleep};

#[derive(Debug)]
pub struct GracefulShutdownManager {
    shutdown_tx: broadcast::Sender<ShutdownSignal>,
    active_processes: Arc<Mutex<Vec<ProcessHandle>>>,
    shutdown_timeout: Duration,
    force_shutdown_timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum ShutdownSignal {
    Graceful,
    Immediate,
    Maintenance,
}

#[derive(Debug)]
pub struct ProcessHandle {
    pub process_id: String,
    pub shutdown_tx: oneshot::Sender<ShutdownSignal>,
    pub completion_rx: Option<oneshot::Receiver<ShutdownResult>>,
}

#[derive(Debug)]
pub enum ShutdownResult {
    Success,
    Timeout,
    Error(String),
}

impl GracefulShutdownManager {
    pub fn new(shutdown_timeout: Duration, force_shutdown_timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(32);
        
        Self {
            shutdown_tx,
            active_processes: Arc::new(Mutex::new(Vec::new())),
            shutdown_timeout,
            force_shutdown_timeout,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Set up signal handlers
        let shutdown_tx = self.shutdown_tx.clone();
        let active_processes = self.active_processes.clone();
        let shutdown_timeout = self.shutdown_timeout;
        let force_shutdown_timeout = self.force_shutdown_timeout;

        tokio::spawn(async move {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();
            let mut sigusr1 = signal::unix::signal(signal::unix::SignalKind::user_defined1()).unwrap();

            loop {
                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, initiating graceful shutdown");
                        Self::initiate_shutdown(
                            shutdown_tx.clone(),
                            active_processes.clone(),
                            ShutdownSignal::Graceful,
                            shutdown_timeout,
                            force_shutdown_timeout,
                        ).await;
                        break;
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT, initiating immediate shutdown");
                        Self::initiate_shutdown(
                            shutdown_tx.clone(),
                            active_processes.clone(),
                            ShutdownSignal::Immediate,
                            Duration::from_secs(5),
                            force_shutdown_timeout,
                        ).await;
                        break;
                    }
                    _ = sigusr1.recv() => {
                        info!("Received SIGUSR1, initiating maintenance shutdown");
                        Self::initiate_shutdown(
                            shutdown_tx.clone(),
                            active_processes.clone(),
                            ShutdownSignal::Maintenance,
                            shutdown_timeout,
                            force_shutdown_timeout,
                        ).await;
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn initiate_shutdown(
        shutdown_tx: broadcast::Sender<ShutdownSignal>,
        active_processes: Arc<Mutex<Vec<ProcessHandle>>>,
        signal: ShutdownSignal,
        shutdown_timeout: Duration,
        force_shutdown_timeout: Duration,
    ) {
        // Notify all processes about shutdown
        if let Err(e) = shutdown_tx.send(signal.clone()) {
            error!("Failed to send shutdown signal: {}", e);
        }

        // Wait for processes to shutdown gracefully
        let processes = {
            let mut guard = active_processes.lock().await;
            std::mem::take(&mut *guard)
        };

        let mut shutdown_futures = Vec::new();
        
        for process in processes {
            if let Some(completion_rx) = process.completion_rx {
                shutdown_futures.push(async move {
                    match timeout(shutdown_timeout, completion_rx).await {
                        Ok(Ok(result)) => {
                            info!("Process {} shutdown result: {:?}", process.process_id, result);
                        }
                        Ok(Err(_)) => {
                            warn!("Process {} shutdown channel closed", process.process_id);
                        }
                        Err(_) => {
                            warn!("Process {} shutdown timeout", process.process_id);
                        }
                    }
                });
            }
        }

        // Wait for all processes to complete shutdown
        match timeout(shutdown_timeout, futures::future::join_all(shutdown_futures)).await {
            Ok(_) => {
                info!("All processes shutdown gracefully");
            }
            Err(_) => {
                warn!("Shutdown timeout exceeded, forcing shutdown");
                // Force kill remaining processes
                Self::force_shutdown(force_shutdown_timeout).await;
            }
        }

        // Notify systemd about shutdown completion
        Self::notify_systemd_shutdown().await;
    }

    async fn force_shutdown(timeout: Duration) {
        warn!("Initiating force shutdown with {}s timeout", timeout.as_secs());
        
        // Send SIGKILL to all our child processes
        let output = tokio::process::Command::new("pkill")
            .args(&["-KILL", "-f", "mister-smith"])
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    info!("Force shutdown completed");
                } else {
                    error!("Force shutdown failed: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                error!("Failed to execute force shutdown: {}", e);
            }
        }

        // Give processes time to die
        sleep(Duration::from_secs(2)).await;
    }

    async fn notify_systemd_shutdown() {
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("STOPPING=1")
                .output()
                .await;
            
            sleep(Duration::from_millis(100)).await;
            
            let _ = tokio::process::Command::new("systemd-notify")
                .arg("STATUS=Shutdown complete")
                .output()
                .await;
        }
    }

    pub async fn register_process(&self, process_id: String) -> (broadcast::Receiver<ShutdownSignal>, oneshot::Sender<ShutdownResult>) {
        let shutdown_rx = self.shutdown_tx.subscribe();
        let (completion_tx, completion_rx) = oneshot::channel();
        
        let handle = ProcessHandle {
            process_id,
            shutdown_tx: completion_tx,
            completion_rx: Some(completion_rx),
        };
        
        self.active_processes.lock().await.push(handle);
        
        (shutdown_rx, completion_tx)
    }

    pub fn create_shutdown_handler<F>(
        mut shutdown_rx: broadcast::Receiver<ShutdownSignal>,
        completion_tx: oneshot::Sender<ShutdownResult>,
        cleanup_fn: F,
    ) -> tokio::task::JoinHandle<()>
    where
        F: Fn(ShutdownSignal) -> futures::future::BoxFuture<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        tokio::spawn(async move {
            while let Ok(signal) = shutdown_rx.recv().await {
                info!("Received shutdown signal: {:?}", signal);
                
                let result = match cleanup_fn(signal).await {
                    Ok(()) => ShutdownResult::Success,
                    Err(e) => ShutdownResult::Error(e.to_string()),
                };
                
                let _ = completion_tx.send(result);
                break;
            }
        })
    }
}

// Example usage in agent process
pub async fn setup_agent_shutdown_handler(
    shutdown_manager: &GracefulShutdownManager,
    agent_context: AgentContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_rx, completion_tx) = shutdown_manager
        .register_process(agent_context.agent_id.clone())
        .await;

    let _shutdown_handler = GracefulShutdownManager::create_shutdown_handler(
        shutdown_rx,
        completion_tx,
        move |signal| {
            let agent_context = agent_context.clone();
            Box::pin(async move {
                match signal {
                    ShutdownSignal::Graceful => {
                        info!("Starting graceful shutdown for agent {}", agent_context.agent_id);
                        
                        // Stop accepting new tasks
                        agent_context.task_queue.stop_accepting().await?;
                        
                        // Wait for current tasks to complete
                        agent_context.task_queue.wait_for_completion(Duration::from_secs(30)).await?;
                        
                        // Close connections
                        agent_context.close_connections().await?;
                        
                        info!("Graceful shutdown completed for agent {}", agent_context.agent_id);
                        Ok(())
                    }
                    ShutdownSignal::Immediate => {
                        info!("Starting immediate shutdown for agent {}", agent_context.agent_id);
                        
                        // Cancel all running tasks
                        agent_context.task_queue.cancel_all().await?;
                        
                        // Close connections immediately
                        agent_context.close_connections().await?;
                        
                        info!("Immediate shutdown completed for agent {}", agent_context.agent_id);
                        Ok(())
                    }
                    ShutdownSignal::Maintenance => {
                        info!("Starting maintenance shutdown for agent {}", agent_context.agent_id);
                        
                        // Drain task queue without accepting new tasks
                        agent_context.task_queue.drain().await?;
                        
                        // Save state for restart
                        agent_context.save_state().await?;
                        
                        // Close connections
                        agent_context.close_connections().await?;
                        
                        info!("Maintenance shutdown completed for agent {}", agent_context.agent_id);
                        Ok(())
                    }
                }
            })
        },
    );

    Ok(())
}
```

---

## 4. Resource Management

### 4.1 System Resource Limits

#### 4.1.1 CGroups v2 Integration

```bash
#!/bin/bash
# /usr/local/bin/setup-mister-smith-cgroups.sh

set -euo pipefail

# CGroups v2 setup for Mister Smith framework
CGROUP_ROOT="/sys/fs/cgroup"
MISTER_SMITH_CGROUP="$CGROUP_ROOT/mister-smith.slice"

# Create main cgroup
if [ ! -d "$MISTER_SMITH_CGROUP" ]; then
    mkdir -p "$MISTER_SMITH_CGROUP"
    echo "Created main cgroup: $MISTER_SMITH_CGROUP"
fi

# Enable controllers
echo "+memory +cpu +io +pids" > "$MISTER_SMITH_CGROUP/cgroup.subtree_control"

# Set main limits
echo "8G" > "$MISTER_SMITH_CGROUP/memory.max"
echo "800" > "$MISTER_SMITH_CGROUP/cpu.weight"
echo "1000000" > "$MISTER_SMITH_CGROUP/pids.max"

# Create orchestrator cgroup
ORCHESTRATOR_CGROUP="$MISTER_SMITH_CGROUP/orchestrator.slice"
mkdir -p "$ORCHESTRATOR_CGROUP"
echo "4G" > "$ORCHESTRATOR_CGROUP/memory.max"
echo "400" > "$ORCHESTRATOR_CGROUP/cpu.weight"
echo "500" > "$ORCHESTRATOR_CGROUP/pids.max"

# Create workers cgroup
WORKERS_CGROUP="$MISTER_SMITH_CGROUP/workers.slice"
mkdir -p "$WORKERS_CGROUP"
echo "+memory +cpu +io +pids" > "$WORKERS_CGROUP/cgroup.subtree_control"
echo "3G" > "$WORKERS_CGROUP/memory.max"
echo "300" > "$WORKERS_CGROUP/cpu.weight"
echo "800" > "$WORKERS_CGROUP/pids.max"

# Create messaging cgroup
MESSAGING_CGROUP="$MISTER_SMITH_CGROUP/messaging.slice"
mkdir -p "$MESSAGING_CGROUP"
echo "1G" > "$MESSAGING_CGROUP/memory.max"
echo "100" > "$MESSAGING_CGROUP/cpu.weight"
echo "200" > "$MESSAGING_CGROUP/pids.max"

# Set I/O weights
echo "200" > "$ORCHESTRATOR_CGROUP/io.weight"
echo "100" > "$WORKERS_CGROUP/io.weight"
echo "150" > "$MESSAGING_CGROUP/io.weight"

echo "CGroups setup completed successfully"
```

#### 4.1.2 Systemd Slice Configuration

```ini
# /etc/systemd/system/mister-smith.slice
[Unit]
Description=Mister Smith AI Agent Framework
Documentation=https://github.com/mister-smith/framework/docs

[Slice]
# Memory limits for entire framework
MemoryAccounting=true
MemoryMax=8G
MemorySwapMax=0
MemoryLow=2G

# CPU limits
CPUAccounting=true
CPUWeight=800
CPUQuota=800%

# I/O limits
IOAccounting=true
IOWeight=200

# Process limits
TasksAccounting=true
TasksMax=1000

# Network accounting
IPAccounting=true
```

```ini
# /etc/systemd/system/mister-smith-orchestrator.slice
[Unit]
Description=Mister Smith AI Agent Framework - Orchestrator Slice
Documentation=https://github.com/mister-smith/framework/docs
DefaultDependencies=false
Requires=mister-smith.slice
After=mister-smith.slice

[Slice]
# Memory limits for orchestrator
MemoryAccounting=true
MemoryMax=4G
MemoryHigh=3G
MemoryLow=1G

# CPU limits
CPUAccounting=true
CPUWeight=400
CPUQuota=400%

# I/O limits
IOAccounting=true
IOWeight=200

# Process limits
TasksAccounting=true
TasksMax=500
```

```ini
# /etc/systemd/system/mister-smith-workers.slice
[Unit]
Description=Mister Smith AI Agent Framework - Workers Slice
Documentation=https://github.com/mister-smith/framework/docs
DefaultDependencies=false
Requires=mister-smith.slice
After=mister-smith.slice

[Slice]
# Memory limits for workers
MemoryAccounting=true
MemoryMax=3G
MemoryHigh=2.5G
MemoryLow=512M

# CPU limits
CPUAccounting=true
CPUWeight=300
CPUQuota=300%

# I/O limits
IOAccounting=true
IOWeight=100

# Process limits
TasksAccounting=true
TasksMax=800
```

### 4.2 Dynamic Resource Allocation

#### 4.2.1 Resource Monitor Implementation

```rust
// src/resources/monitor.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt, NetworkExt};

#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub timestamp: Instant,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_available_bytes: u64,
    pub disk_io_read_bytes: u64,
    pub disk_io_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub process_count: usize,
    pub file_descriptor_count: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessResourceUsage {
    pub process_id: String,
    pub pid: u32,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub start_time: Instant,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Unknown,
}

pub struct ResourceMonitor {
    system: Arc<Mutex<System>>,
    process_metrics: Arc<RwLock<HashMap<String, ProcessResourceUsage>>>,
    system_metrics: Arc<RwLock<ResourceMetrics>>,
    monitoring_interval: Duration,
    resource_limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,
    alerting_thresholds: ResourceThresholds,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_file_descriptors: Option<u64>,
    pub max_disk_io_bytes_per_sec: Option<u64>,
    pub max_network_bytes_per_sec: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ResourceThresholds {
    pub memory_warning_percent: f64,
    pub memory_critical_percent: f64,
    pub cpu_warning_percent: f64,
    pub cpu_critical_percent: f64,
    pub disk_space_warning_percent: f64,
    pub disk_space_critical_percent: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            memory_warning_percent: 80.0,
            memory_critical_percent: 95.0,
            cpu_warning_percent: 80.0,
            cpu_critical_percent: 95.0,
            disk_space_warning_percent: 85.0,
            disk_space_critical_percent: 95.0,
        }
    }
}

impl ResourceMonitor {
    pub fn new(monitoring_interval: Duration) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(Mutex::new(system)),
            process_metrics: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
            monitoring_interval,
            resource_limits: Arc::new(RwLock::new(HashMap::new())),
            alerting_thresholds: ResourceThresholds::default(),
        }
    }

    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let system = self.system.clone();
        let process_metrics = self.process_metrics.clone();
        let system_metrics = self.system_metrics.clone();
        let resource_limits = self.resource_limits.clone();
        let thresholds = self.alerting_thresholds.clone();
        let interval_duration = self.monitoring_interval;

        tokio::spawn(async move {
            let mut monitoring_interval = interval(interval_duration);

            loop {
                monitoring_interval.tick().await;

                if let Err(e) = Self::collect_metrics(
                    &system,
                    &process_metrics,
                    &system_metrics,
                    &resource_limits,
                    &thresholds,
                ).await {
                    error!("Failed to collect resource metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn collect_metrics(
        system: &Arc<Mutex<System>>,
        process_metrics: &Arc<RwLock<HashMap<String, ProcessResourceUsage>>>,
        system_metrics: &Arc<RwLock<ResourceMetrics>>,
        resource_limits: &Arc<RwLock<HashMap<String, ResourceLimits>>>,
        thresholds: &ResourceThresholds,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sys = system.lock().await;
        sys.refresh_all();

        // Collect system-wide metrics
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        let used_memory = total_memory - available_memory;
        
        let cpu_usage = sys.processors().iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / sys.processors().len() as f32;

        let (disk_read, disk_write) = sys.disks().iter()
            .fold((0u64, 0u64), |(read, write), disk| {
                // Note: sysinfo doesn't provide real-time I/O, 
                // this would need to be implemented differently
                (read, write)
            });

        let system_metric = ResourceMetrics {
            timestamp: Instant::now(),
            cpu_usage_percent: cpu_usage as f64,
            memory_usage_bytes: used_memory,
            memory_available_bytes: available_memory,
            disk_io_read_bytes: disk_read,
            disk_io_write_bytes: disk_write,
            network_rx_bytes: 0, // Would need to be implemented
            network_tx_bytes: 0, // Would need to be implemented
            process_count: sys.processes().len(),
            file_descriptor_count: 0, // Would need to be implemented
        };

        // Check system-level thresholds
        Self::check_system_thresholds(&system_metric, thresholds).await;

        *system_metrics.write().await = system_metric;

        // Collect per-process metrics for Mister Smith processes
        let mut process_map = process_metrics.write().await;
        process_map.clear();

        for (pid, process) in sys.processes() {
            let cmd_line = process.cmd().join(" ");
            if cmd_line.contains("mister-smith") {
                let process_id = Self::extract_process_id(&cmd_line)
                    .unwrap_or_else(|| format!("unknown-{}", pid));

                let process_usage = ProcessResourceUsage {
                    process_id: process_id.clone(),
                    pid: *pid as u32,
                    cpu_usage_percent: process.cpu_usage() as f64,
                    memory_usage_bytes: process.memory(),
                    virtual_memory_bytes: process.virtual_memory(),
                    disk_read_bytes: process.disk_usage().read_bytes,
                    disk_write_bytes: process.disk_usage().written_bytes,
                    start_time: Instant::now(), // Simplified
                    status: Self::convert_process_status(process.status()),
                };

                // Check process-specific limits
                Self::check_process_limits(&process_usage, resource_limits).await;

                process_map.insert(process_id, process_usage);
            }
        }

        Ok(())
    }

    async fn check_system_thresholds(metrics: &ResourceMetrics, thresholds: &ResourceThresholds) {
        let memory_usage_percent = (metrics.memory_usage_bytes as f64 / 
                                   (metrics.memory_usage_bytes + metrics.memory_available_bytes) as f64) * 100.0;

        if memory_usage_percent > thresholds.memory_critical_percent {
            error!("CRITICAL: System memory usage at {:.1}%", memory_usage_percent);
            Self::emit_alert(AlertLevel::Critical, AlertType::MemoryUsage, memory_usage_percent).await;
        } else if memory_usage_percent > thresholds.memory_warning_percent {
            warn!("WARNING: System memory usage at {:.1}%", memory_usage_percent);
            Self::emit_alert(AlertLevel::Warning, AlertType::MemoryUsage, memory_usage_percent).await;
        }

        if metrics.cpu_usage_percent > thresholds.cpu_critical_percent {
            error!("CRITICAL: System CPU usage at {:.1}%", metrics.cpu_usage_percent);
            Self::emit_alert(AlertLevel::Critical, AlertType::CpuUsage, metrics.cpu_usage_percent).await;
        } else if metrics.cpu_usage_percent > thresholds.cpu_warning_percent {
            warn!("WARNING: System CPU usage at {:.1}%", metrics.cpu_usage_percent);
            Self::emit_alert(AlertLevel::Warning, AlertType::CpuUsage, metrics.cpu_usage_percent).await;
        }
    }

    async fn check_process_limits(
        usage: &ProcessResourceUsage,
        resource_limits: &Arc<RwLock<HashMap<String, ResourceLimits>>>,
    ) {
        let limits_map = resource_limits.read().await;
        if let Some(limits) = limits_map.get(&usage.process_id) {
            // Check memory limit
            if let Some(max_memory) = limits.max_memory_bytes {
                if usage.memory_usage_bytes > max_memory {
                    error!("Process {} exceeds memory limit: {} > {} bytes", 
                           usage.process_id, usage.memory_usage_bytes, max_memory);
                    Self::emit_process_alert(
                        &usage.process_id,
                        AlertLevel::Critical,
                        AlertType::MemoryLimit,
                        usage.memory_usage_bytes as f64,
                    ).await;
                }
            }

            // Check CPU limit
            if let Some(max_cpu) = limits.max_cpu_percent {
                if usage.cpu_usage_percent > max_cpu {
                    warn!("Process {} exceeds CPU limit: {:.1}% > {:.1}%", 
                          usage.process_id, usage.cpu_usage_percent, max_cpu);
                    Self::emit_process_alert(
                        &usage.process_id,
                        AlertLevel::Warning,
                        AlertType::CpuLimit,
                        usage.cpu_usage_percent,
                    ).await;
                }
            }
        }
    }

    fn extract_process_id(cmd_line: &str) -> Option<String> {
        // Extract process ID from command line
        if cmd_line.contains("orchestrator") {
            Some("orchestrator".to_string())
        } else if cmd_line.contains("worker") {
            // Try to extract worker ID
            if let Some(pos) = cmd_line.find("--worker-id") {
                let remaining = &cmd_line[pos + 12..];
                if let Some(id) = remaining.split_whitespace().next() {
                    Some(format!("worker-{}", id))
                } else {
                    Some("worker-unknown".to_string())
                }
            } else {
                Some("worker".to_string())
            }
        } else if cmd_line.contains("nats-server") {
            Some("messaging".to_string())
        } else {
            None
        }
    }

    fn convert_process_status(status: sysinfo::ProcessStatus) -> ProcessStatus {
        match status {
            sysinfo::ProcessStatus::Run => ProcessStatus::Running,
            sysinfo::ProcessStatus::Sleep => ProcessStatus::Sleeping,
            sysinfo::ProcessStatus::Stop => ProcessStatus::Stopped,
            sysinfo::ProcessStatus::Zombie => ProcessStatus::Zombie,
            _ => ProcessStatus::Unknown,
        }
    }

    async fn emit_alert(level: AlertLevel, alert_type: AlertType, value: f64) {
        // Integration with alerting system
        let alert = SystemAlert {
            level,
            alert_type,
            value,
            timestamp: Instant::now(),
            message: format!("System {} alert: {:.1}", alert_type.to_string(), value),
        };

        // This would integrate with your alerting system
        info!("ALERT: {:?}", alert);
    }

    async fn emit_process_alert(process_id: &str, level: AlertLevel, alert_type: AlertType, value: f64) {
        let alert = ProcessAlert {
            process_id: process_id.to_string(),
            level,
            alert_type,
            value,
            timestamp: Instant::now(),
            message: format!("Process {} {} alert: {:.1}", process_id, alert_type.to_string(), value),
        };

        // This would integrate with your alerting system
        info!("PROCESS ALERT: {:?}", alert);
    }

    pub async fn set_process_limits(&self, process_id: String, limits: ResourceLimits) {
        self.resource_limits.write().await.insert(process_id, limits);
    }

    pub async fn get_system_metrics(&self) -> ResourceMetrics {
        self.system_metrics.read().await.clone()
    }

    pub async fn get_process_metrics(&self) -> HashMap<String, ProcessResourceUsage> {
        self.process_metrics.read().await.clone()
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_available_bytes: 0,
            disk_io_read_bytes: 0,
            disk_io_write_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            process_count: 0,
            file_descriptor_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    MemoryUsage,
    CpuUsage,
    DiskSpace,
    MemoryLimit,
    CpuLimit,
    FileDescriptorLimit,
}

impl AlertType {
    fn to_string(&self) -> &'static str {
        match self {
            AlertType::MemoryUsage => "memory_usage",
            AlertType::CpuUsage => "cpu_usage",
            AlertType::DiskSpace => "disk_space",
            AlertType::MemoryLimit => "memory_limit",
            AlertType::CpuLimit => "cpu_limit",
            AlertType::FileDescriptorLimit => "fd_limit",
        }
    }
}

#[derive(Debug)]
pub struct SystemAlert {
    pub level: AlertLevel,
    pub alert_type: AlertType,
    pub value: f64,
    pub timestamp: Instant,
    pub message: String,
}

#[derive(Debug)]
pub struct ProcessAlert {
    pub process_id: String,
    pub level: AlertLevel,
    pub alert_type: AlertType,
    pub value: f64,
    pub timestamp: Instant,
    pub message: String,
}
```

---

## 5. Health Monitoring System

### 5.1 Health Check Implementation

#### 5.1.1 Multi-Level Health Checks

```rust
// src/health/checker.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub enabled_checks: Vec<HealthCheckType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    Http { endpoint: String, expected_status: u16 },
    Tcp { host: String, port: u16 },
    Process { process_name: String },
    DatabaseConnection { connection_string: String },
    MessageQueue { broker_url: String },
    CustomScript { script_path: String, args: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component_id: String,
    pub status: ComponentStatus,
    pub last_check: Instant,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

pub struct HealthCheckManager {
    components: Arc<RwLock<HashMap<String, ComponentHealthChecker>>>,
    aggregated_status: Arc<RwLock<HealthStatus>>,
    http_client: Client,
    notification_manager: Arc<HealthNotificationManager>,
}

impl HealthCheckManager {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            aggregated_status: Arc::new(RwLock::new(HealthStatus::default())),
            http_client,
            notification_manager: Arc::new(HealthNotificationManager::new()),
        }
    }

    pub async fn register_component(
        &self,
        component_id: String,
        config: HealthCheckConfig,
    ) -> Result<(), HealthCheckError> {
        let checker = ComponentHealthChecker::new(
            component_id.clone(),
            config,
            self.http_client.clone(),
        );

        self.components.write().await.insert(component_id, checker);
        Ok(())
    }

    pub async fn start_monitoring(&self) -> Result<(), HealthCheckError> {
        // Start individual component monitoring
        let components = self.components.read().await;
        for (component_id, checker) in components.iter() {
            let checker_clone = checker.clone();
            let notification_manager = self.notification_manager.clone();
            let component_id = component_id.clone();

            tokio::spawn(async move {
                checker_clone.start_monitoring(notification_manager, component_id).await;
            });
        }

        // Start aggregated status monitoring
        self.start_aggregated_monitoring().await;

        Ok(())
    }

    async fn start_aggregated_monitoring(&self) {
        let components = self.components.clone();
        let aggregated_status = self.aggregated_status.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                let component_statuses = {
                    let components_guard = components.read().await;
                    let mut statuses = HashMap::new();
                    
                    for (id, checker) in components_guard.iter() {
                        statuses.insert(id.clone(), checker.get_status().await);
                    }
                    statuses
                };

                let overall_status = Self::calculate_overall_status(&component_statuses);
                *aggregated_status.write().await = overall_status;
            }
        });
    }

    fn calculate_overall_status(component_statuses: &HashMap<String, HealthStatus>) -> HealthStatus {
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unknown_count = 0;

        let mut latest_check = Instant::now();
        let mut total_response_time = 0u64;
        let mut response_time_count = 0;

        for status in component_statuses.values() {
            match status.status {
                ComponentStatus::Healthy => healthy_count += 1,
                ComponentStatus::Degraded => degraded_count += 1,
                ComponentStatus::Unhealthy => unhealthy_count += 1,
                ComponentStatus::Unknown => unknown_count += 1,
            }

            if status.last_check < latest_check {
                latest_check = status.last_check;
            }

            if let Some(response_time) = status.response_time_ms {
                total_response_time += response_time;
                response_time_count += 1;
            }
        }

        let overall_status = if unhealthy_count > 0 {
            ComponentStatus::Unhealthy
        } else if degraded_count > 0 {
            ComponentStatus::Degraded
        } else if healthy_count > 0 {
            ComponentStatus::Healthy
        } else {
            ComponentStatus::Unknown
        };

        let avg_response_time = if response_time_count > 0 {
            Some(total_response_time / response_time_count)
        } else {
            None
        };

        HealthStatus {
            component_id: "system".to_string(),
            status: overall_status,
            last_check: latest_check,
            last_success: None, // Would need to track this separately
            last_failure: None, // Would need to track this separately
            consecutive_failures: 0,
            consecutive_successes: 0,
            response_time_ms: avg_response_time,
            error_message: None,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("healthy_components".to_string(), healthy_count.to_string());
                metadata.insert("degraded_components".to_string(), degraded_count.to_string());
                metadata.insert("unhealthy_components".to_string(), unhealthy_count.to_string());
                metadata.insert("unknown_components".to_string(), unknown_count.to_string());
                metadata
            },
        }
    }

    pub async fn get_overall_status(&self) -> HealthStatus {
        self.aggregated_status.read().await.clone()
    }

    pub async fn get_component_status(&self, component_id: &str) -> Option<HealthStatus> {
        let components = self.components.read().await;
        if let Some(checker) = components.get(component_id) {
            Some(checker.get_status().await)
        } else {
            None
        }
    }

    pub async fn get_all_component_statuses(&self) -> HashMap<String, HealthStatus> {
        let components = self.components.read().await;
        let mut statuses = HashMap::new();
        
        for (id, checker) in components.iter() {
            statuses.insert(id.clone(), checker.get_status().await);
        }
        
        statuses
    }
}

#[derive(Clone)]
pub struct ComponentHealthChecker {
    component_id: String,
    config: HealthCheckConfig,
    status: Arc<RwLock<HealthStatus>>,
    http_client: Client,
}

impl ComponentHealthChecker {
    pub fn new(component_id: String, config: HealthCheckConfig, http_client: Client) -> Self {
        let initial_status = HealthStatus {
            component_id: component_id.clone(),
            status: ComponentStatus::Unknown,
            last_check: Instant::now(),
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            response_time_ms: None,
            error_message: None,
            metadata: HashMap::new(),
        };

        Self {
            component_id,
            config,
            status: Arc::new(RwLock::new(initial_status)),
            http_client,
        }
    }

    pub async fn start_monitoring(
        &self,
        notification_manager: Arc<HealthNotificationManager>,
        component_id: String,
    ) {
        let mut interval = interval(self.config.check_interval);
        let config = self.config.clone();
        let status = self.status.clone();
        let http_client = self.http_client.clone();

        loop {
            interval.tick().await;

            let check_start = Instant::now();
            let mut check_results = Vec::new();

            // Run all enabled health checks
            for check_type in &config.enabled_checks {
                let result = Self::perform_health_check(
                    check_type,
                    &http_client,
                    config.timeout,
                ).await;
                check_results.push(result);
            }

            let check_duration = check_start.elapsed();
            let overall_result = Self::aggregate_check_results(check_results);

            // Update status
            let mut status_guard = status.write().await;
            status_guard.last_check = check_start;
            status_guard.response_time_ms = Some(check_duration.as_millis() as u64);

            match overall_result {
                Ok(metadata) => {
                    status_guard.last_success = Some(check_start);
                    status_guard.consecutive_successes += 1;
                    status_guard.consecutive_failures = 0;
                    status_guard.error_message = None;
                    status_guard.metadata = metadata;

                    // Determine health status based on metadata
                    status_guard.status = if status_guard.consecutive_successes >= config.success_threshold {
                        ComponentStatus::Healthy
                    } else {
                        ComponentStatus::Degraded
                    };
                }
                Err(error) => {
                    status_guard.last_failure = Some(check_start);
                    status_guard.consecutive_failures += 1;
                    status_guard.consecutive_successes = 0;
                    status_guard.error_message = Some(error.to_string());

                    status_guard.status = if status_guard.consecutive_failures >= config.failure_threshold {
                        ComponentStatus::Unhealthy
                    } else {
                        ComponentStatus::Degraded
                    };
                }
            }

            // Send notifications on status changes
            if status_guard.consecutive_failures == config.failure_threshold {
                notification_manager.send_failure_notification(&component_id, &status_guard).await;
            } else if status_guard.consecutive_successes == config.success_threshold && 
                     status_guard.last_failure.is_some() {
                notification_manager.send_recovery_notification(&component_id, &status_guard).await;
            }
        }
    }

    async fn perform_health_check(
        check_type: &HealthCheckType,
        http_client: &Client,
        timeout: Duration,
    ) -> Result<HashMap<String, String>, HealthCheckError> {
        match check_type {
            HealthCheckType::Http { endpoint, expected_status } => {
                let start = Instant::now();
                let response = tokio::time::timeout(
                    timeout,
                    http_client.get(endpoint).send()
                ).await
                .map_err(|_| HealthCheckError::Timeout)?
                .map_err(HealthCheckError::HttpError)?;

                let response_time = start.elapsed().as_millis();
                let status_code = response.status().as_u16();

                if status_code == *expected_status {
                    let mut metadata = HashMap::new();
                    metadata.insert("status_code".to_string(), status_code.to_string());
                    metadata.insert("response_time_ms".to_string(), response_time.to_string());
                    Ok(metadata)
                } else {
                    Err(HealthCheckError::UnexpectedStatus {
                        expected: *expected_status,
                        actual: status_code,
                    })
                }
            }

            HealthCheckType::Tcp { host, port } => {
                let start = Instant::now();
                let socket_addr = format!("{}:{}", host, port);
                
                match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&socket_addr)).await {
                    Ok(Ok(_)) => {
                        let response_time = start.elapsed().as_millis();
                        let mut metadata = HashMap::new();
                        metadata.insert("response_time_ms".to_string(), response_time.to_string());
                        metadata.insert("socket".to_string(), socket_addr);
                        Ok(metadata)
                    }
                    Ok(Err(e)) => Err(HealthCheckError::ConnectionError(e.to_string())),
                    Err(_) => Err(HealthCheckError::Timeout),
                }
            }

            HealthCheckType::Process { process_name } => {
                let output = tokio::process::Command::new("pgrep")
                    .arg("-f")
                    .arg(process_name)
                    .output()
                    .await
                    .map_err(|e| HealthCheckError::ProcessCheckError(e.to_string()))?;

                if output.status.success() && !output.stdout.is_empty() {
                    let pid_list = String::from_utf8_lossy(&output.stdout);
                    let pid_count = pid_list.lines().count();
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("process_count".to_string(), pid_count.to_string());
                    metadata.insert("pids".to_string(), pid_list.trim().to_string());
                    Ok(metadata)
                } else {
                    Err(HealthCheckError::ProcessNotFound(process_name.clone()))
                }
            }

            HealthCheckType::DatabaseConnection { connection_string } => {
                // This would be implemented based on your database driver
                // For example, with sqlx for PostgreSQL:
                // let pool = sqlx::PgPool::connect(connection_string).await?;
                // let _row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;
                
                // Simplified implementation
                let mut metadata = HashMap::new();
                metadata.insert("check_type".to_string(), "database_connection".to_string());
                Ok(metadata)
            }

            HealthCheckType::MessageQueue { broker_url } => {
                // This would be implemented based on your message broker
                // For NATS, you might check the monitoring endpoint
                let monitoring_url = format!("{}/varz", broker_url.replace("nats://", "http://"));
                
                let start = Instant::now();
                let response = tokio::time::timeout(
                    timeout,
                    http_client.get(&monitoring_url).send()
                ).await
                .map_err(|_| HealthCheckError::Timeout)?
                .map_err(HealthCheckError::HttpError)?;

                let response_time = start.elapsed().as_millis();

                if response.status().is_success() {
                    let mut metadata = HashMap::new();
                    metadata.insert("response_time_ms".to_string(), response_time.to_string());
                    metadata.insert("broker_url".to_string(), broker_url.clone());
                    Ok(metadata)
                } else {
                    Err(HealthCheckError::MessageBrokerError(response.status().to_string()))
                }
            }

            HealthCheckType::CustomScript { script_path, args } => {
                let start = Instant::now();
                let output = tokio::time::timeout(
                    timeout,
                    tokio::process::Command::new(script_path)
                        .args(args)
                        .output()
                ).await
                .map_err(|_| HealthCheckError::Timeout)?
                .map_err(|e| HealthCheckError::ScriptError(e.to_string()))?;

                let response_time = start.elapsed().as_millis();

                if output.status.success() {
                    let mut metadata = HashMap::new();
                    metadata.insert("response_time_ms".to_string(), response_time.to_string());
                    metadata.insert("exit_code".to_string(), "0".to_string());
                    if !output.stdout.is_empty() {
                        metadata.insert("output".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
                    }
                    Ok(metadata)
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    Err(HealthCheckError::ScriptError(format!(
                        "Script failed with exit code {}: {}",
                        output.status.code().unwrap_or(-1),
                        error_output
                    )))
                }
            }
        }
    }

    fn aggregate_check_results(
        results: Vec<Result<HashMap<String, String>, HealthCheckError>>
    ) -> Result<HashMap<String, String>, HealthCheckError> {
        let mut aggregated_metadata = HashMap::new();
        let mut errors = Vec::new();

        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(metadata) => {
                    for (key, value) in metadata {
                        aggregated_metadata.insert(format!("check_{}_{}", i, key), value.clone());
                    }
                }
                Err(e) => {
                    errors.push(format!("check_{}: {}", i, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(aggregated_metadata)
        } else {
            Err(HealthCheckError::MultipleFailures(errors))
        }
    }

    pub async fn get_status(&self) -> HealthStatus {
        self.status.read().await.clone()
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            component_id: "unknown".to_string(),
            status: ComponentStatus::Unknown,
            last_check: Instant::now(),
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            response_time_ms: None,
            error_message: None,
            metadata: HashMap::new(),
        }
    }
}

pub struct HealthNotificationManager {
    // This would integrate with your notification system
}

impl HealthNotificationManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_failure_notification(&self, component_id: &str, status: &HealthStatus) {
        error!("HEALTH ALERT: Component {} is now UNHEALTHY after {} consecutive failures", 
               component_id, status.consecutive_failures);
        
        if let Some(error) = &status.error_message {
            error!("Error details: {}", error);
        }

        // Here you would integrate with your alerting system:
        // - Send to Slack/Teams
        // - Send email
        // - Create PagerDuty incident
        // - Update status page
    }

    pub async fn send_recovery_notification(&self, component_id: &str, status: &HealthStatus) {
        info!("HEALTH RECOVERY: Component {} is now HEALTHY after {} consecutive successes", 
              component_id, status.consecutive_successes);

        // Here you would integrate with your alerting system:
        // - Send recovery notification
        // - Close PagerDuty incident
        // - Update status page
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    #[error("Health check timed out")]
    Timeout,
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Unexpected HTTP status: expected {expected}, got {actual}")]
    UnexpectedStatus { expected: u16, actual: u16 },
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Process check error: {0}")]
    ProcessCheckError(String),
    #[error("Process not found: {0}")]
    ProcessNotFound(String),
    #[error("Message broker error: {0}")]
    MessageBrokerError(String),
    #[error("Script error: {0}")]
    ScriptError(String),
    #[error("Multiple check failures: {0:?}")]
    MultipleFailures(Vec<String>),
}
```

#### 5.1.2 Health Check Service Configuration

```ini
# /etc/systemd/system/mister-smith-health-monitor.service
[Unit]
Description=Mister Smith AI Agent Framework - Health Monitor
Documentation=https://github.com/mister-smith/framework/docs
Wants=network-online.target
After=network-online.target
PartOf=mister-smith.target

[Service]
Type=simple
ExecStart=/usr/local/bin/mister-smith-health-monitor
Restart=always
RestartSec=5
TimeoutStartSec=15
TimeoutStopSec=10

# Process management
User=mister-smith
Group=mister-smith
UMask=0022
WorkingDirectory=/app
Environment=HEALTH_CHECK_INTERVAL=15
Environment=HEALTH_CHECK_TIMEOUT=5
Environment=FAILURE_THRESHOLD=3
Environment=SUCCESS_THRESHOLD=2
EnvironmentFile=-/etc/mister-smith/health.env

# Resource limits
LimitNOFILE=1024
LimitNPROC=256
LimitCORE=0

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/app/logs /tmp
PrivateTmp=true
PrivateDevices=true
ProtectControlGroups=true
ProtectKernelModules=true
ProtectKernelTunables=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# Memory management
MemoryAccounting=true
MemoryMax=256M
MemorySwapMax=0

# CPU management
CPUAccounting=true
CPUQuota=25%
CPUShares=256

[Install]
WantedBy=mister-smith.target
```

```yaml
# /etc/mister-smith/health-checks.yaml
health_checks:
  orchestrator:
    check_interval: 15s
    timeout: 5s
    failure_threshold: 3
    success_threshold: 2
    enabled_checks:
      - type: http
        endpoint: "http://localhost:8080/health"
        expected_status: 200
      - type: http
        endpoint: "http://localhost:8080/ready"
        expected_status: 200
      - type: process
        process_name: "mister-smith-orchestrator"
      - type: tcp
        host: "localhost"
        port: 8080

  worker:
    check_interval: 15s
    timeout: 5s
    failure_threshold: 3
    success_threshold: 2
    enabled_checks:
      - type: http
        endpoint: "http://localhost:8081/health"
        expected_status: 200
      - type: process
        process_name: "mister-smith-worker"

  messaging:
    check_interval: 10s
    timeout: 3s
    failure_threshold: 2
    success_threshold: 2
    enabled_checks:
      - type: tcp
        host: "localhost"
        port: 4222
      - type: http
        endpoint: "http://localhost:8222/varz"
        expected_status: 200
      - type: process
        process_name: "nats-server"

  database:
    check_interval: 30s
    timeout: 10s
    failure_threshold: 2
    success_threshold: 1
    enabled_checks:
      - type: database_connection
        connection_string: "${DATABASE_URL}"
      - type: tcp
        host: "localhost"
        port: 5432

  system:
    check_interval: 60s
    timeout: 15s
    failure_threshold: 1
    success_threshold: 1
    enabled_checks:
      - type: custom_script
        script_path: "/usr/local/bin/system-health-check.sh"
        args: []
```

```bash
#!/bin/bash
# /usr/local/bin/system-health-check.sh

set -euo pipefail

# System health check script for Mister Smith framework

# Check disk space
DISK_USAGE=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "ERROR: Disk usage is ${DISK_USAGE}%" >&2
    exit 1
elif [ "$DISK_USAGE" -gt 80 ]; then
    echo "WARNING: Disk usage is ${DISK_USAGE}%" >&2
fi

# Check memory usage
MEMORY_USAGE=$(free | grep Mem | awk '{printf "%.0f", $3/$2 * 100.0}')
if [ "$MEMORY_USAGE" -gt 95 ]; then
    echo "ERROR: Memory usage is ${MEMORY_USAGE}%" >&2
    exit 1
elif [ "$MEMORY_USAGE" -gt 85 ]; then
    echo "WARNING: Memory usage is ${MEMORY_USAGE}%" >&2
fi

# Check load average
LOAD_AVERAGE=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | sed 's/,//')
CPU_COUNT=$(nproc)
LOAD_PERCENT=$(echo "$LOAD_AVERAGE $CPU_COUNT" | awk '{printf "%.0f", $1/$2 * 100}')

if [ "$LOAD_PERCENT" -gt 200 ]; then
    echo "ERROR: Load average is ${LOAD_PERCENT}% of CPU capacity" >&2
    exit 1
elif [ "$LOAD_PERCENT" -gt 150 ]; then
    echo "WARNING: Load average is ${LOAD_PERCENT}% of CPU capacity" >&2
fi

# Check systemd services
FAILED_SERVICES=$(systemctl --failed --no-legend | grep "mister-smith" | wc -l)
if [ "$FAILED_SERVICES" -gt 0 ]; then
    echo "ERROR: $FAILED_SERVICES Mister Smith services have failed" >&2
    systemctl --failed --no-legend | grep "mister-smith" >&2
    exit 1
fi

echo "System health check passed"
echo "Disk: ${DISK_USAGE}%, Memory: ${MEMORY_USAGE}%, Load: ${LOAD_PERCENT}%"
exit 0
```

---

## 6. Integration Examples

### 6.1 Complete Deployment Setup

```bash
#!/bin/bash
# /usr/local/bin/setup-mister-smith-deployment.sh

set -euo pipefail

echo "Setting up Mister Smith AI Agent Framework deployment..."

# Create system user and group
if ! id mister-smith &>/dev/null; then
    useradd --system --shell /bin/false --home /app --create-home mister-smith
    echo "Created mister-smith system user"
fi

# Create directories
mkdir -p /app/{data,logs,workspace,config}
mkdir -p /etc/mister-smith/{secrets,certs}
mkdir -p /var/log/mister-smith
mkdir -p /var/lib/mister-smith

# Set ownership
chown -R mister-smith:mister-smith /app
chown -R mister-smith:mister-smith /var/log/mister-smith
chown -R mister-smith:mister-smith /var/lib/mister-smith
chown -R root:mister-smith /etc/mister-smith
chmod -R 750 /etc/mister-smith

# Install systemd service files
cp ./systemd/*.service /etc/systemd/system/
cp ./systemd/*.slice /etc/systemd/system/
cp ./systemd/*.target /etc/systemd/system/

# Install configuration files
cp ./config/*.env /etc/mister-smith/
cp ./config/*.yaml /etc/mister-smith/
cp ./config/*.toml /etc/mister-smith/

# Install binaries
cp ./target/release/mister-smith-* /usr/local/bin/
chmod +x /usr/local/bin/mister-smith-*

# Install health check scripts
cp ./scripts/system-health-check.sh /usr/local/bin/
cp ./scripts/setup-cgroups.sh /usr/local/bin/
chmod +x /usr/local/bin/*.sh

# Set up CGroups
/usr/local/bin/setup-mister-smith-cgroups.sh

# Generate systemd drop-ins for environment
if [ "${ENVIRONMENT_TIER:-}" = "tier_3" ]; then
    mkdir -p /etc/systemd/system/mister-smith-orchestrator.service.d/
    mkdir -p /etc/systemd/system/mister-smith-worker@.service.d/
    
    cat > /etc/systemd/system/mister-smith-orchestrator.service.d/tier-3.conf << EOF
[Service]
Environment=ENVIRONMENT_TIER=tier_3
Environment=LOG_LEVEL=warn
MemoryMax=4G
CPUQuota=400%
EOF

    cat > /etc/systemd/system/mister-smith-worker@.service.d/tier-3.conf << EOF
[Service]
Environment=ENVIRONMENT_TIER=tier_3
Environment=LOG_LEVEL=warn
MemoryMax=2G
CPUQuota=200%
EOF
fi

# Reload systemd
systemctl daemon-reload

# Enable services
systemctl enable mister-smith.target
systemctl enable mister-smith-health-monitor.service

# Start dependencies if needed
if systemctl is-active --quiet postgresql; then
    echo "PostgreSQL is already running"
else
    echo "Starting PostgreSQL..."
    systemctl start postgresql
fi

if systemctl is-active --quiet redis; then
    echo "Redis is already running"
else
    echo "Starting Redis..."
    systemctl start redis
fi

# Start Mister Smith services
echo "Starting Mister Smith framework..."
systemctl start mister-smith.target

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 10

# Check service status
echo "Checking service status..."
systemctl status mister-smith.target --no-pager
systemctl status mister-smith-orchestrator.service --no-pager
systemctl status mister-smith-health-monitor.service --no-pager

# Run health checks
echo "Running initial health checks..."
/usr/local/bin/system-health-check.sh

# Check orchestrator health
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "✓ Orchestrator health check passed"
else
    echo "✗ Orchestrator health check failed"
    exit 1
fi

# Start workers
echo "Starting worker agents..."
for i in {1..4}; do
    systemctl start "mister-smith-worker@${i}.service"
    echo "Started worker ${i}"
done

# Final status check
echo "Final deployment status:"
systemctl list-units --state=active | grep mister-smith

echo "✓ Mister Smith AI Agent Framework deployment completed successfully!"
echo ""
echo "Service URLs:"
echo "  - Orchestrator: http://localhost:8080"
echo "  - Health Monitor: http://localhost:8080/health"
echo "  - Metrics: http://localhost:9090/metrics"
echo ""
echo "Log locations:"
echo "  - System logs: journalctl -u mister-smith.target"
echo "  - Application logs: /var/log/mister-smith/"
echo ""
echo "Management commands:"
echo "  - Status: systemctl status mister-smith.target"
echo "  - Stop: systemctl stop mister-smith.target"
echo "  - Restart: systemctl restart mister-smith.target"
```

### 6.2 Monitoring Integration

```yaml
# /etc/mister-smith/monitoring.yaml
monitoring:
  prometheus:
    enabled: true
    bind_address: "0.0.0.0"
    port: 9090
    metrics_path: "/metrics"
    scrape_interval: 15s
    
  systemd_exporter:
    enabled: true
    port: 9558
    unit_whitelist:
      - "mister-smith-*.service"
      - "mister-smith.target"
      
  process_exporter:
    enabled: true
    port: 9256
    config:
      process_names:
        - name: "mister-smith-orchestrator"
          cmdline:
            - "mister-smith-orchestrator"
        - name: "mister-smith-worker"
          cmdline:
            - "mister-smith-worker"
        - name: "nats-server"
          cmdline:
            - "nats-server"
            
  node_exporter:
    enabled: true
    port: 9100
    collectors:
      - systemd
      - processes
      - meminfo
      - diskstats
      - filesystem
      - netstat
      
alerting:
  rules:
    - name: "mister-smith-process-alerts"
      rules:
        - alert: "MisterSmithProcessDown"
          expr: up{job="mister-smith-processes"} == 0
          for: 30s
          labels:
            severity: critical
          annotations:
            summary: "Mister Smith process {{ $labels.instance }} is down"
            
        - alert: "MisterSmithHighMemoryUsage"
          expr: process_resident_memory_bytes{job="mister-smith-processes"} > 2e+9
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "Mister Smith process {{ $labels.instance }} using > 2GB memory"
            
        - alert: "MisterSmithHighCPUUsage"
          expr: rate(process_cpu_seconds_total{job="mister-smith-processes"}[5m]) > 0.8
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "Mister Smith process {{ $labels.instance }} high CPU usage"
```

---

## 7. Best Practices Summary

### 7.1 Process Management Best Practices

1. **Systemd-Native Design**: Use systemd features for service management, dependencies, and resource control
2. **Graceful Shutdowns**: Always implement proper shutdown handlers for data consistency
3. **Resource Limits**: Set appropriate memory, CPU, and I/O limits for each service type
4. **Health Monitoring**: Implement multi-level health checks with proper escalation
5. **Supervision Trees**: Use hierarchical supervision for fault tolerance and recovery

### 7.2 Security Best Practices

1. **Process Isolation**: Run services as dedicated non-root users with minimal privileges
2. **File System Protection**: Use systemd security features to restrict file system access
3. **Network Isolation**: Implement proper network policies and access controls
4. **Secret Management**: Store secrets securely and rotate them regularly
5. **Audit Logging**: Log all process lifecycle events for security monitoring

### 7.3 Operational Best Practices

1. **Environment Separation**: Use different configurations for tier_1, tier_2, and tier_3 environments
2. **Monitoring Integration**: Integrate with existing monitoring and alerting systems (see [Observability Framework](observability-monitoring-framework.md))
3. **Resource Planning**: Plan resource allocation based on workload requirements and agent capabilities
4. **Backup and Recovery**: Implement proper backup strategies for process state and configuration
5. **Documentation**: Maintain up-to-date documentation for operational procedures
6. **Configuration Management**: Use centralized configuration management (see [Configuration Management](configuration-management.md))
7. **Deployment Automation**: Implement automated deployment pipelines (see [Deployment Architecture](deployment-architecture-specifications.md))
8. **Build Integration**: Coordinate with build systems for deployment artifacts (see [Build Specifications](build-specifications.md))

## 8. Enhanced Service Discovery Patterns (Enhancement)

### 8.1 Explicit Service Registry Architecture

```yaml
# service-registry.yaml - Enhanced service discovery patterns
service_registry:
  backends:
    consul:
      enabled: true
      address: "consul.service.consul:8500"
      datacenter: "dc1"
      service_prefix: "mister-smith"
      
    etcd:
      enabled: false
      endpoints:
        - "http://etcd-1:2379"
        - "http://etcd-2:2379"
        - "http://etcd-3:2379"
      prefix: "/mister-smith/services"
      
    kubernetes:
      enabled: true
      namespace: "mister-smith-system"
      service_type: "ClusterIP"
      
  registration:
    auto_register: true
    health_check_interval: 10s
    deregister_critical_after: 60s
    
    metadata:
      version: "${AGENT_VERSION}"
      tier: "${ENVIRONMENT_TIER}"
      capabilities: "${AGENT_CAPABILITIES}"
      
  discovery:
    cache_ttl: 30s
    retry_policy:
      max_attempts: 3
      backoff: exponential
      initial_delay: 1s
```

### 8.2 Service Mesh Integration

```yaml
# istio-service-mesh.yaml - Advanced service discovery
apiVersion: networking.istio.io/v1beta1
kind: ServiceEntry
metadata:
  name: mister-smith-services
spec:
  hosts:
  - orchestrator.mister-smith.local
  - worker.mister-smith.local
  - messaging.mister-smith.local
  ports:
  - number: 8080
    name: http
    protocol: HTTP
  - number: 9090
    name: grpc
    protocol: GRPC
  location: MESH_INTERNAL
  resolution: DNS
---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: mister-smith-load-balancing
spec:
  host: worker.mister-smith.local
  trafficPolicy:
    loadBalancer:
      consistentHash:
        httpHeaderName: "x-task-type"
```

### 8.3 DNS-Based Service Discovery

```ini
# systemd-resolved integration
# /etc/systemd/resolved.conf.d/mister-smith.conf
[Resolve]
DNS=127.0.0.1:8600
Domains=~mister-smith.local
DNSSEC=no
```

```yaml
# coredns-config.yaml
mister-smith.local:53 {
    file /etc/coredns/mister-smith.local.zone
    prometheus :9153
    log
    errors
    cache 30
    reload 10s
}
```

### 8.4 NATS-Enhanced Discovery

```rust
// Enhanced NATS service discovery patterns
pub struct ServiceRegistry {
    nats_client: nats::Connection,
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub async fn register_service(&self, info: ServiceInfo) -> Result<()> {
        // Publish service registration
        let subject = format!("services.{}.register", info.service_type);
        self.nats_client.publish(&subject, &info.to_json()?)?;
        
        // Set up heartbeat
        let heartbeat_subject = format!("services.{}.heartbeat.{}", 
            info.service_type, info.instance_id);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                let _ = self.nats_client.publish(&heartbeat_subject, b"alive");
            }
        });
        
        Ok(())
    }
    
    pub async fn discover_services(&self, service_type: &str) -> Vec<ServiceInfo> {
        let subject = format!("services.{}.info", service_type);
        let responses = self.nats_client.request_multi(&subject, b"", 
            Duration::from_secs(1))?;
        
        responses.into_iter()
            .filter_map(|msg| ServiceInfo::from_json(&msg.data).ok())
            .collect()
    }
}

pub struct ServiceInfo {
    pub service_type: String,
    pub instance_id: String,
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_status: HealthStatus,
    pub last_heartbeat: Instant,
}
```

### 8.5 Load Balancing Strategies

```rust
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin { weights: HashMap<String, u32> },
    ConsistentHashing { replicas: u32 },
    CapabilityBased { required_capabilities: Vec<String> },
}

impl ServiceSelector {
    pub fn select_service(
        &self, 
        services: &[ServiceInfo], 
        strategy: LoadBalancingStrategy,
        context: &RequestContext
    ) -> Option<&ServiceInfo> {
        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.round_robin_select(services)
            },
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections_select(services)
            },
            LoadBalancingStrategy::ConsistentHashing { replicas } => {
                self.consistent_hash_select(services, context, replicas)
            },
            LoadBalancingStrategy::CapabilityBased { required_capabilities } => {
                self.capability_based_select(services, &required_capabilities)
            },
            _ => None,
        }
    }
}
```

---

This comprehensive process management specification provides production-ready patterns and implementations for systemd integration, process supervision,
lifecycle management, resource management, and health monitoring, ensuring robust operation of the Mister Smith AI Agent Framework across all deployment tiers.
The enhanced service discovery patterns provide multiple options for service registration and discovery, supporting various deployment scenarios from
simple NATS-based communication to advanced service mesh integrations.
