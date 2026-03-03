# Process Management Implementation Guide

## Overview

This document provides a technical guide for implementing process management capabilities in the Mister Smith AI Agent Framework.
It outlines the key components, implementation patterns, and operational procedures for distributed agent process management.

## Completed Deliverables

### 1. Process Management Specifications (NEW)

**File**: `/ms-framework-docs/operations/process-management-specifications.md`

**Comprehensive coverage including**:

#### 1.1 Systemd Integration

- **Complete service unit files** for orchestrator, worker, messaging, and health check services
- **Target configuration** with proper dependencies and ordering
- **Environment file management** with tier-specific configurations
- **Drop-in configurations** for environment-specific overrides
- **Resource management** through systemd slices and CGroups v2

#### 1.2 Process Supervision Patterns

- **Hierarchical supervision architecture** with fault-tolerant supervision trees
- **Restart policy implementation** with exponential backoff, linear backoff, and adaptive strategies
- **Failure detection and recovery** with configurable thresholds and escalation policies
- **Integration with systemd** for native process supervision

#### 1.3 Lifecycle Management

- **Graceful shutdown implementation** with signal handling and timeout management
- **Process startup and initialization** with health check integration
- **State management** during lifecycle transitions
- **Recovery procedures** for various failure scenarios

#### 1.4 Resource Management

- **CGroups v2 integration** with complete setup scripts
- **Dynamic resource allocation** with monitoring and enforcement
- **Resource limits** by service type and environment tier
- **Performance monitoring** with real-time resource usage tracking

#### 1.5 Health Monitoring System

- **Multi-level health checks** (HTTP, TCP, process, database, custom scripts)
- **Health status aggregation** with component-level and system-level monitoring
- **Alerting and notification** with configurable thresholds
- **Integration with systemd watchdog** for automatic restart on health failures

### 2. Enhanced Observability Framework

**File**: `/ms-framework-docs/operations/observability-monitoring-framework.md`

**Completed network monitoring implementation**:

- **Network statistics collection** from `/proc/net/dev` and `/proc/net/tcp`
- **Real-time connection monitoring** for active TCP connections
- **Bytes and packet counting** for sent/received traffic
- **Integration with process supervision** for network health checks

## Core Implementation Features

### Systemd Service Management

- **Native systemd integration** with proper unit files
- **Service dependencies** and ordering
- **Environment-specific configurations**
- **Resource limits** through systemd and CGroups
- **Security hardening** with privilege restrictions
- **Watchdog integration** for automatic restart

### Process Supervision

- **Hierarchical supervision trees** with configurable strategies
- **Restart policies** with backoff algorithms
- **Failure detection** using phi accrual failure detector
- **Circuit breaker patterns** to prevent cascading failures
- **Hub-and-spoke routing** for centralized task distribution

### Resource Management

- **CGroups v2 setup** with automated scripts
- **Memory, CPU, and I/O limits** per service type
- **Dynamic resource monitoring** with real-time tracking
- **Alerting on resource thresholds** with escalation policies
- **Process-level resource enforcement**

### Health Monitoring

- **HTTP health endpoints** for all services
- **TCP connection monitoring** for network services
- **Process existence checks** via system calls
- **Database connection health** with timeout handling
- **Custom script execution** for specialized checks
- **Aggregated health status** with component roll-up

### Deployment Automation

- **Complete setup script** for system deployment
- **User and directory creation** with proper permissions
- **Service installation** and configuration
- **Environment tier support** (tier_1, tier_2, tier_3)
- **Health check validation** during deployment

## Implementation Architecture

### Service Architecture

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

### Resource Hierarchy

```text
mister-smith.slice (8GB memory, 800% CPU)
├── orchestrator.slice (4GB memory, 400% CPU)
├── workers.slice (3GB memory, 300% CPU) 
└── messaging.slice (1GB memory, 100% CPU)
```

### Supervision Tree

```text
RootSupervisor
├── OrchestrationSupervisor (OneForAll strategy)
├── WorkerPoolSupervisor (OneForOne strategy)
└── MessagingSupervisor (RestForOne strategy)
```

## Key Implementation Files

### Systemd Service Files

- `mister-smith-orchestrator.service` - Orchestrator service configuration
- `mister-smith-worker@.service` - Worker service template
- `mister-smith-messaging.service` - NATS messaging service
- `mister-smith-health-check.service` - Health monitoring service
- `mister-smith.target` - Service group target
- `mister-smith.slice` - Resource slice configuration

### Configuration Files

- `orchestrator.env` - Orchestrator environment variables
- `worker.env` - Worker environment variables
- `health-checks.yaml` - Health check configuration
- `monitoring.yaml` - Monitoring integration settings

### Scripts and Tools

- `setup-mister-smith-production.sh` - Complete deployment script
- `setup-mister-smith-cgroups.sh` - CGroups configuration script
- `system-health-check.sh` - System health validation script

## Security Implementations

### Process Isolation

- **Dedicated system users** for each service type
- **File system restrictions** using systemd security features
- **Network isolation** with private networking where appropriate
- **Capability restrictions** removing unnecessary privileges

### Resource Protection

- **Memory limits** preventing OOM scenarios
- **CPU quotas** preventing resource starvation
- **I/O weight management** for fair resource sharing
- **Process count limits** preventing fork bombs

### Secret Management

- **Environment file protection** with restricted permissions
- **Secret file references** avoiding plain text secrets
- **Key rotation support** through configuration reloading

## Monitoring Integration

### System Metrics

- Process CPU and memory usage
- Network I/O statistics
- Disk I/O and filesystem usage
- Service availability and health

### Application Metrics

- Task completion rates and latencies
- Message queue depths and processing times
- Agent lifecycle events and state transitions
- Error rates and failure patterns

### Alerting Rules

- Service down alerts
- Resource usage threshold alerts
- Health check failure alerts
- Performance degradation alerts

## Testing and Validation

### Health Check Coverage

- **HTTP endpoint checks** for API availability
- **TCP connection checks** for network services
- **Process existence verification** for all services
- **Database connectivity testing** for data services
- **System resource validation** for operational health

### Failure Scenario Testing

- **Service crash recovery** with automatic restart
- **Resource exhaustion handling** with graceful degradation
- **Network partition recovery** with reconnection logic
- **Database failure handling** with circuit breaker protection

## Environment Tier Support

### Tier 1 (Experimental)

- Minimal resource allocation
- Basic health monitoring
- Development-friendly logging
- Relaxed restart policies

### Tier 2 (Validation)

- Standard resource allocation
- Comprehensive health monitoring
- Structured logging
- Balanced restart policies

### Tier 3 (Operational)

- Maximum resource allocation
- Production health monitoring
- Minimal logging for performance
- Conservative restart policies

## Implementation Architecture

### Core Components

- **Complete Implementation**: All code examples are functional Rust implementations
- **No Placeholders**: All specification sections are complete
- **Integration Coverage**: Full integration with existing framework components
- **Security Hardening**: Comprehensive security measures implemented
- **Operational Procedures**: Complete deployment and operational procedures
- **Monitoring Integration**: Full observability and alerting coverage
- **Error Handling**: Comprehensive error scenarios and recovery procedures

## Implementation Steps

1. **Deploy Configuration Files**: Install systemd service files and configurations
2. **Set Up CGroups**: Run CGroups setup script for resource management
3. **Install Monitoring**: Deploy health check services and monitoring integration
4. **Test Deployment**: Run complete deployment script in test environment
5. **Validate Operations**: Execute operational procedures and verify functionality

## Integration Points

- **[Configuration Management](configuration-management.md)**: Configuration patterns and environment management
- **[Deployment Architecture](deployment-architecture-specifications.md)**: Infrastructure deployment patterns
- **[Observability Framework](observability-monitoring-framework.md)**: Monitoring and alerting integration
- **[Build Specifications](build-specifications.md)**: Build and packaging coordination

## Summary

The process management specifications provide comprehensive coverage of systemd integration, process supervision, lifecycle management,
resource management, and health monitoring for distributed agent operations. The implementation follows established patterns for
service management, resource control, fault tolerance, and operational monitoring across all environment tiers.
