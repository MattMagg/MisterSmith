---
title: Deployment Architecture Specifications - Revised
type: note
permalink: revision-swarm/operations/deployment-architecture-specifications-revised
---

## Deployment Architecture Specifications - Revised

## Multi-Agent Platform Deployment Patterns

### Deployment Architecture Readiness Assessment

#### Component Architecture Status

| Component | Implementation Status | Technical Requirements |
|-----------|----------------------|----------------------|
| **Container Patterns** | ✅ Specified | Multi-stage builds, runtime optimization |
| **Orchestration Patterns** | ✅ Specified | DAG/FSM models, retry strategies |
| **Scaling Patterns** | ✅ Specified | HPA, cluster autoscaling, resource allocation |
| **Network Architecture** | ✅ Specified | Service mesh, network policies, security |
| **Security Implementation** | ⚠️ Partial | Pod security standards, RBAC, secret management |
| **Deployment Pipelines** | ✅ Specified | GitOps, progressive delivery, automation |
| **Monitoring Integration** | ✅ Specified | Observability stack, metrics, tracing |

#### Critical Implementation Requirements

**Security Hardening Requirements**:

- Pod Security Standards enforcement
- Network Policy manifests
- RBAC policy definitions
- Secret management implementation
- Storage access policies

**Operational Requirements**:

- Health check implementations
- Resource quota definitions
- Backup and recovery procedures
- Monitoring and alerting integration

#### Architecture Dependencies

**External Dependencies**:

- Kubernetes cluster (v1.24+)
- Container registry access
- Service mesh infrastructure
- Monitoring infrastructure

**Internal Dependencies**:

- Agent lifecycle management
- Message routing protocols
- Data persistence layers
- Security policy enforcement

### Architecture Overview

This document specifies deployment architecture patterns for multi-agent platforms, providing technical specifications for containerization strategies, orchestration patterns, and infrastructure designs. The patterns define container architectures, Kubernetes orchestration, service mesh integration, and scaling strategies for agent deployment implementations.

### 1. Container Architecture Patterns

#### 1.1 Base Container Strategy Pattern

```rust
PATTERN MultiStageContainer:
    STAGES:
        build_stage:
            setup_build_environment()
            copy_source_code()
            compile_application()
            run_tests()
        
        runtime_stage:
            create_minimal_image()
            copy_built_artifacts()
            configure_runtime()
            expose_service_ports()
            define_entry_point()
```

#### 1.2 Agent Container Types Pattern

```rust
PATTERN AgentContainerTypes:
    CONTAINER_CATEGORIES:
        orchestrator_container:
            purpose: "MisterSmith coordination and management"
            resource_profile: "moderate"
            exposed_services: ["mister_smith_management_api", "coordination_rpc"]
            agent_types: ["orchestrator_agent"]
        
        worker_container:
            purpose: "MisterSmith task execution"
            resource_profile: "variable"
            exposed_services: ["worker_api", "task_executor"]
            agent_types: ["worker_agent", "domain_agent"]
        
        messaging_container:
            purpose: "MisterSmith inter-agent communication"
            resource_profile: "minimal"
            exposed_services: ["nats_message_bus", "agent_event_stream"]
            agent_types: ["messaging_proxy"]
```

#### 1.3 Sidecar Pattern Implementation

```rust
PATTERN SidecarArchitecture:
    SIDECAR_TYPES:
        mister_smith_coordination_proxy:
            intercept_agent_communication()
            apply_mister_smith_routing_rules()
            enforce_agent_policies()
            manage_agent_lifecycle()
        
        mister_smith_metrics_collector:
            gather_agent_metrics()
            aggregate_agent_performance()
            export_to_prometheus()
            track_agent_health()
        
        mister_smith_security_proxy:
            handle_agent_authentication()
            enforce_agent_authorization()
            audit_agent_access()
            manage_agent_secrets()
```

### 2. Orchestration Patterns

#### 2.1 DAG vs FSM Orchestration Pattern

```rust
PATTERN OrchestrationModelSelection:
    DAG_PATTERN:
        characteristics:
            - "explicit dependency graph"
            - "batch processing oriented"
            - "visual representation"
            - "static or dynamic construction"
        
        use_cases:
            - "data pipelines"
            - "batch workflows"
            - "ETL processes"
            - "parallel task execution"
        
        implementation:
            static_dag:
                define_all_tasks()
                explicit_dependencies()
                compile_time_validation()
            
            dynamic_dag:
                runtime_task_generation()
                inferred_dependencies()
                flexible_execution_paths()
    
    FSM_PATTERN:
        characteristics:
            - "state-based control flow"
            - "complex decision logic"
            - "compensation support"
            - "dynamic transitions"
        
        use_cases:
            - "order processing"
            - "approval workflows"
            - "multi-step transactions"
            - "error recovery flows"
        
        implementation:
            state_machine:
                define_states()
                transition_rules()
                compensation_stack()
                rollback_handlers()
```

#### 2.2 Workflow Retry Pattern

```rust
PATTERN RetryStrategies:
    RETRY_POLICIES:
        exponential_backoff:
            initial_delay: "1s"
            max_delay: "300s"
            backoff_factor: 2
            jitter: "random_0_to_1"
            
        linear_backoff:
            fixed_delay: "30s"
            max_attempts: 3
            
        circuit_breaker:
            failure_threshold: 5
            recovery_timeout: "60s"
            half_open_attempts: 3
    
    RETRY_IMPLEMENTATION:
        task_level_retry:
            configure_per_task_policy()
            track_attempt_count()
            handle_final_failure()
            
        workflow_level_retry:
            checkpoint_state()
            resume_from_failure()
            compensate_completed_tasks()
            
        dead_letter_handling:
            capture_failed_tasks()
            manual_intervention_queue()
            retry_with_modifications()
```

#### 2.3 Namespace Organization Pattern

```rust
PATTERN NamespaceArchitecture:
    NAMESPACE_CATEGORIES:
        mister_smith_system_namespace:
            purpose: "MisterSmith core platform components"
            isolation_level: "strict"
            components: ["orchestrator_agents", "nats_messaging", "configuration_management"]
            namespace_name: "mister-smith-core"
        
        mister_smith_workload_namespace:
            purpose: "MisterSmith agent workloads"
            isolation_level: "moderate"
            components: ["worker_agent_pools", "domain_agent_pools", "task_queues"]
            namespace_name: "mister-smith-workers"
        
        mister_smith_data_namespace:
            purpose: "MisterSmith persistence layer"
            isolation_level: "strict"
            components: ["postgresql", "jetstream_kv", "agent_state_storage"]
            namespace_name: "mister-smith-data"
        
        monitoring_namespace:
            purpose: "observability stack"
            isolation_level: "moderate"
            components: ["metrics", "logs", "traces"]
```

#### 2.4 Deployment Topology Pattern

```rust
PATTERN DeploymentTopology:
    ORCHESTRATOR_DEPLOYMENT:
        replica_strategy: "odd_number_for_consensus"
        distribution: "across_availability_zones"
        update_strategy: "rolling_with_zero_downtime"
        health_checks: ["liveness", "readiness", "startup"]
    
    WORKER_DEPLOYMENT:
        replica_strategy: "dynamic_based_on_load"
        distribution: "maximize_spread"
        update_strategy: "blue_green_or_canary"
        scaling_policy: "horizontal_and_vertical"
```

#### 2.5 Service Discovery Pattern

```rust
PATTERN ServiceDiscovery:
    DISCOVERY_METHODS:
        dns_based:
            register_mister_smith_service_endpoints()
            resolve_agent_by_service_name()
            handle_agent_endpoint_changes()
        
        registry_based:
            register_agent_with_metadata()
            query_agents_by_attributes()
            watch_for_agent_updates()
        
        mesh_based:
            automatic_agent_sidecar_injection()
            intelligent_agent_routing()
            agent_circuit_breaking()
```

### 3. Scaling Architecture Patterns

#### 3.1 Horizontal Scaling Pattern

```rust
PATTERN HorizontalScaling:
    SCALING_TRIGGERS:
        resource_based:
            monitor_cpu_usage()
            monitor_memory_usage()
            monitor_network_throughput()
        
        application_based:
            monitor_queue_depth()
            monitor_response_time()
            monitor_error_rate()
        
        custom_metrics:
            monitor_business_metrics()
            monitor_agent_specific_metrics()
    
    SCALING_BEHAVIOR:
        scale_up_policy:
            stabilization_window
            maximum_scale_rate
            scale_up_threshold
        
        scale_down_policy:
            cooldown_period
            gradual_reduction
            minimum_instances
```

#### 3.2 Cluster Autoscaling Pattern

```rust
PATTERN ClusterAutoscaling:
    NODE_GROUPS:
        control_plane_nodes:
            purpose: "orchestration"
            scaling: "vertical_preferred"
            placement: "dedicated_hosts"
        
        worker_nodes:
            purpose: "agent_workloads"
            scaling: "horizontal_preferred"
            placement: "spot_instances_allowed"
        
        data_nodes:
            purpose: "persistence"
            scaling: "careful_horizontal"
            placement: "storage_optimized"
```

#### 3.3 Resource Allocation Pattern

```rust
PATTERN ResourceAllocation:
    RESOURCE_TIERS:
        minimal_tier:
            cpu_allocation: "fractional"
            memory_allocation: "constrained"
            priority: "best_effort"
        
        standard_tier:
            cpu_allocation: "guaranteed_minimum"
            memory_allocation: "reserved"
            priority: "normal"
        
        premium_tier:
            cpu_allocation: "dedicated"
            memory_allocation: "exclusive"
            priority: "high"
```

#### 3.4 Orchestration Autoscale Pattern

```rust
PATTERN OrchestrationAutoscale:
    SCALING_STRATEGIES:
        queue_based_scaling:
            monitor_queue_depth()
            calculate_processing_rate()
            scale_workers_proportionally()
            consider_message_age()
            
        event_driven_scaling:
            monitor_event_streams()
            predict_load_patterns()
            preemptive_scaling()
            custom_metric_triggers()
            
        schedule_based_scaling:
            define_time_patterns()
            pre_scale_for_known_loads()
            combine_with_reactive_scaling()
            handle_timezone_variations()
    
    ORCHESTRATOR_SCALING:
        task_distribution_model:
            worker_pool_sizing:
                min_workers_per_queue: 2
                max_workers_per_queue: 100
                scale_increment: "10%_or_1_worker"
                
            task_assignment:
                round_robin_distribution()
                load_based_assignment()
                affinity_based_routing()
                
        resource_based_autoscale:
            cpu_threshold: "70%_sustained_60s"
            memory_threshold: "80%_sustained_30s"
            queue_depth_threshold: "100_pending_tasks"
            
        failure_aware_scaling:
            monitor_task_failures()
            detect_poison_messages()
            isolate_failing_workers()
            scale_healthy_workers()
```

### 4. Package Management Patterns

*Cross-references:*

- [Configuration Management](./configuration-management.md)
- [Configuration Deployment Specifications](./configuration-deployment-specifications.md)
- [Implementation Config](../core-architecture/implementation-config.md)

#### 4.1 Helm Chart Structure Pattern

```rust
PATTERN ChartOrganization:
    STRUCTURE:
        chart_metadata:
            define_chart_properties()
            specify_dependencies()
            set_version_constraints()
        
        templates:
            deployment_templates()
            service_templates()
            configuration_templates()
            security_templates()
        
        values:
            default_values()
            environment_overrides()
            secret_references()
```

#### 4.2 Configuration Management Pattern

*Cross-references:*

- [Configuration Management](./configuration-management.md)
- [Configuration Deployment Specifications](./configuration-deployment-specifications.md)

```rust
PATTERN ConfigurationHierarchy:
    LEVELS:
        base_configuration:
            common_settings()
            default_behaviors()

        environment_configuration:
            tier_specific_settings()
            resource_adjustments()

        runtime_configuration:
            dynamic_overrides()
            feature_toggles()
```

#### 4.3 Claude-CLI Configuration Pattern

```rust
PATTERN ClaudeCLIConfiguration:
    PARALLEL_EXECUTION_SETTINGS:
        environment_variables:
            CLAUDE_PARALLEL_DEFAULT: "default_fan_out_per_task_plan"
            # Controls default number of parallel agents spawned
            # when --parallel flag is not explicitly specified

        deployment_configuration:
            parallel_agent_limits:
                min_parallel_agents: 1
                max_parallel_agents: 50
                default_parallel_agents: 4

            resource_allocation:
                per_agent_cpu_limit: "0.5"
                per_agent_memory_limit: "512Mi"
                total_parallel_cpu_budget: "4.0"

        integration_settings:
            output_parsing_enabled: true
            nats_subject_mapping: true
            observability_integration: true
            span_tagging_per_agent: true
```

### 5. Network Architecture Patterns

#### 5.1 Network Segmentation Pattern

```rust
PATTERN NetworkSegmentation:
    SEGMENTS:
        control_plane_network:
            purpose: "management_traffic"
            isolation: "strict"
            encryption: "required"
        
        data_plane_network:
            purpose: "agent_communication"
            isolation: "moderate"
            encryption: "optional"
        
        ingress_network:
            purpose: "external_access"
            isolation: "dmz_pattern"
            encryption: "tls_required"
```

#### 5.1a Network Policy Implementation (Security Enhancement)

```yaml
# network-policies.yaml - Defense-in-depth network security
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: mister-smith-orchestrator-netpol
  namespace: mister-smith-system
spec:
  podSelector:
    matchLabels:
      app: mister-smith-orchestrator
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: mister-smith-workload
    - podSelector:
        matchLabels:
          app: mister-smith-worker
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 9090
  - from:
    - namespaceSelector:
        matchLabels:
          name: mister-smith-monitoring
    ports:
    - protocol: TCP
      port: 8080  # Metrics endpoint
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: mister-smith-system
    ports:
    - protocol: TCP
      port: 4222  # NATS
  - to:
    - namespaceSelector:
        matchLabels:
          name: mister-smith-data
    ports:
    - protocol: TCP
      port: 6379  # Redis
    - protocol: TCP
      port: 5432  # PostgreSQL
  - ports:
    - protocol: TCP
      port: 53   # DNS
    - protocol: UDP
      port: 53
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-default
  namespace: mister-smith-system
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

#### 5.2 Service Mesh Pattern

```rust
PATTERN ServiceMesh:
    CAPABILITIES:
        traffic_management:
            intelligent_routing()
            load_balancing()
            circuit_breaking()
            retry_logic:
                per_service_configuration()
                backoff_algorithms()
                retry_budgets()
                idempotency_headers()
        
        security:
            mutual_tls()
            authorization_policies()
            encryption_in_transit()
        
        observability:
            distributed_tracing()
            metrics_collection()
            traffic_visualization()
    
    ADVANCED_RETRY_CONFIGURATION:
        retry_conditions:
            retriable_status_codes: [502, 503, 504]
            retriable_headers: ["x-retry-after"]
            connect_failure: true
            reset_before_request: true
            
        retry_budgets:
            percentage_allowed: "20%_of_requests"
            min_retry_concurrency: 10
            ttl_seconds: 10
            
        backoff_configuration:
            base_interval: "25ms"
            max_interval: "250ms"
            backoff_algorithm: "exponential_with_jitter"
```

### 6. Deployment Pipeline Patterns

*Cross-references:*

- [Build Specifications](./build-specifications.md)
- [Configuration Deployment Specifications](./configuration-deployment-specifications.md)

#### 6.1 GitOps Pattern

```rust
PATTERN GitOpsDeployment:
    WORKFLOW:
        source_of_truth: "git_repository"
        
        change_process:
            propose_change_via_pr()
            automated_validation()
            approval_workflow()
            automated_deployment()
        
        reconciliation:
            continuous_monitoring()
            drift_detection()
            automatic_correction()
```

#### 6.2 Progressive Delivery Pattern

```rust
PATTERN ProgressiveDelivery:
    STAGES:
        canary_release:
            deploy_to_small_subset()
            monitor_key_metrics()
            automated_analysis()
            promotion_decision()
        
        feature_flags:
            gradual_enablement()
            user_segmentation()
            instant_rollback()
        
        blue_green:
            parallel_environments()
            instant_switchover()
            rollback_capability()
```

### 7. Multi-Environment Strategy Pattern

#### 7.1 Environment Tiers Pattern

```rust
PATTERN EnvironmentTiers:
    TIER_DEFINITIONS:
        tier_1_experimental:
            purpose: "development_testing"
            scale: "minimal"
            data: "synthetic"
            stability: "volatile"
        
        tier_2_validation:
            purpose: "integration_testing"
            scale: "moderate"
            data: "anonymized"
            stability: "stable"
        
        tier_3_operational:
            purpose: "live_operations"
            scale: "full"
            data: "production"
            stability: "highly_stable"
```

#### 7.2 Environment Promotion Pattern

```rust
PATTERN EnvironmentPromotion:
    PROMOTION_FLOW:
        build_artifacts()
        deploy_to_tier_1()
        run_tier_1_validation()
        promote_to_tier_2()
        run_integration_tests()
        gate_checks()
        promote_to_tier_3()
        monitor_deployment()
```

### 8. High Availability Patterns

#### 8.1 Multi-Region Deployment Pattern

```rust
PATTERN MultiRegionDeployment:
    TOPOLOGY:
        active_active:
            all_regions_serve_traffic()
            data_replication_async()
            conflict_resolution()
        
        active_passive:
            primary_region_active()
            standby_regions_ready()
            failover_automation()
```

#### 8.2 Disaster Recovery Pattern

```rust
PATTERN DisasterRecovery:
    COMPONENTS:
        backup_strategy:
            continuous_backups()
            point_in_time_recovery()
            geographic_distribution()
        
        recovery_procedures:
            automated_failover()
            data_restoration()
            service_reconstruction()
            validation_testing()
```

### 9. Security Patterns

#### 9.1 Zero Trust Architecture Pattern

```rust
PATTERN ZeroTrustSecurity:
    PRINCIPLES:
        never_trust_always_verify()
        least_privilege_access()
        assume_breach_mindset()
        continuous_verification()
    
    IMPLEMENTATION:
        identity_verification()
        device_validation()
        application_authentication()
        data_encryption()
        continuous_monitoring()
```

#### 9.2 Secret Management Pattern

```rust
PATTERN SecretManagement:
    SECRET_LIFECYCLE:
        generation: "automated_strong_secrets"
        storage: "encrypted_vault"
        distribution: "secure_injection"
        rotation: "automated_periodic"
        auditing: "comprehensive_logging"
```

### 10. Monitoring Integration Patterns

*Cross-references:*

- [Monitoring and Health](../core-architecture/monitoring-and-health.md)
- [Observability Monitoring Framework](./observability-monitoring-framework.md)

#### 10.1 Observability Stack Pattern

```rust
PATTERN ObservabilityIntegration:
    COMPONENTS:
        metrics_pipeline:
            collection_agents()
            aggregation_layer()
            storage_backend()
            query_interface()
        
        logging_pipeline:
            log_collectors()
            processing_layer()
            indexing_system()
            search_interface()
        
        tracing_pipeline:
            trace_collectors()
            sampling_layer()
            storage_system()
            analysis_tools()
```

### 11. Implementation Guidelines

#### 11.1 Deployment Orchestration Pattern

```rust
PATTERN DeploymentOrchestration:
    PHASES:
        infrastructure_preparation:
            provision_compute_resources()
            configure_networking()
            setup_storage_systems()
        
        platform_deployment:
            deploy_core_services()
            configure_messaging()
            setup_orchestration()
        
        workload_deployment:
            deploy_agent_pools()
            configure_scaling()
            enable_monitoring()
        
        validation:
            health_checks()
            integration_tests()
            performance_validation()
```

### 12. Best Practices Summary

#### 12.1 Container Best Practices

- Use multi-stage builds for optimization
- Implement proper health checks
- Follow security scanning practices
- Maintain minimal base images

#### 12.2 Orchestration Best Practices

- Implement proper resource limits
- Use anti-affinity for high availability
- Enable pod disruption budgets
- Implement graceful shutdowns

#### 12.3 Scaling Best Practices

- Define clear scaling metrics
- Implement gradual scaling policies
- Monitor scaling effectiveness
- Plan for burst capacity

#### 12.4 Security Best Practices

- Implement network policies
- Use service accounts properly
- Enable audit logging
- Regular security updates

---

## CONCRETE DEPLOYMENT TEMPLATES

### 13. Dockerfile Templates

*Cross-references:*

- [Build Specifications](./build-specifications.md)
- [Configuration Management](./configuration-management.md)

#### 13.1 Base Orchestrator Container Template

```dockerfile
# syntax=docker/dockerfile:1
# Multi-stage Dockerfile for Mister Smith Orchestrator Agent

ARG RUST_VERSION="1.75"
ARG ALPINE_VERSION="3.19"

# Build Stage
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS builder
LABEL stage=builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    && rm -rf /var/cache/apk/*

# Create app user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

WORKDIR /app

# Copy dependency manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with optimizations
RUN cargo build --release --locked && \
    strip target/release/mister-smith-orchestrator

# Runtime Stage
FROM alpine:${ALPINE_VERSION}@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    openssl \
    && rm -rf /var/cache/apk/*

# Create app user (matching builder stage)
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Copy binary from builder stage
COPY --from=builder --chown=appuser:appgroup /app/target/release/mister-smith-orchestrator /usr/local/bin/orchestrator

# Create necessary directories
RUN mkdir -p /app/logs /app/config && \
    chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Set working directory
WORKDIR /app

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV AGENT_TYPE=orchestrator
ENV HTTP_PORT=8080
ENV GRPC_PORT=9090

# Expose ports
EXPOSE 8080 9090

# Start the orchestrator
ENTRYPOINT ["/usr/local/bin/orchestrator"]
```

#### 13.2 Worker Container Template

```dockerfile
# syntax=docker/dockerfile:1
# Multi-stage Dockerfile for Mister Smith Worker Agent

ARG RUST_VERSION="1.75"
ARG ALPINE_VERSION="3.19"

# Build Stage  
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS builder
LABEL stage=builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    && rm -rf /var/cache/apk/*

WORKDIR /app

# Copy and build application
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked && \
    strip target/release/mister-smith-worker

# Runtime Stage
FROM alpine:${ALPINE_VERSION}@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    openssl \
    curl \
    && rm -rf /var/cache/apk/*

# Create app user
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Copy binary
COPY --from=builder --chown=appuser:appgroup /app/target/release/mister-smith-worker /usr/local/bin/worker

# Create directories
RUN mkdir -p /app/workspace /app/logs && \
    chown -R appuser:appgroup /app

USER appuser
WORKDIR /app

# Health checks
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Environment configuration
ENV RUST_LOG=info
ENV AGENT_TYPE=worker
ENV HTTP_PORT=8081
ENV WORKER_CONCURRENCY=4

EXPOSE 8081

ENTRYPOINT ["/usr/local/bin/worker"]
```

#### 13.3 Messaging Container Template  

```dockerfile
# syntax=docker/dockerfile:1
# NATS-based Messaging Container for Mister Smith

FROM nats:2.10-alpine AS runtime

# Install additional tools
RUN apk add --no-cache curl

# Create nats user and directories
RUN addgroup -g 1001 -S natsgroup && \
    adduser -u 1001 -S natsuser -G natsgroup && \
    mkdir -p /etc/nats /var/lib/nats && \
    chown -R natsuser:natsgroup /etc/nats /var/lib/nats

# Copy NATS configuration
COPY --chown=natsuser:natsgroup nats.conf /etc/nats/nats.conf

USER natsuser

# Health check for NATS
HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD nats server ping || exit 1

# Environment variables
ENV NATS_CONFIG_FILE=/etc/nats/nats.conf

EXPOSE 4222 8222 6222

ENTRYPOINT ["/nats-server", "-c", "/etc/nats/nats.conf"]
```

### 14. Docker Compose Specifications

#### 14.1 Base Development Stack

```yaml
# docker-compose.yml - Base Mister Smith development stack
version: '3.8'

services:
  nats:
    image: nats:2.10-alpine
    container_name: mister-smith-nats
    ports:
      - "4222:4222"
      - "8222:8222"
      - "6222:6222"
    command: ["-js", "-m", "8222"]
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:8222/healthz"]
      interval: 10s
      timeout: 5s
      retries: 3
    networks:
      - mister-smith-net

  redis:
    image: redis:7-alpine
    container_name: mister-smith-redis
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 3
    networks:
      - mister-smith-net

  postgres:
    image: postgres:16-alpine
    container_name: mister-smith-postgres
    environment:
      POSTGRES_DB: mister_smith
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 3
    networks:
      - mister-smith-net

  orchestrator:
    build:
      context: .
      dockerfile: Dockerfile.orchestrator
      target: runtime
    container_name: mister-smith-orchestrator
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=debug
      - NATS_URL=nats://nats:4222
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/mister_smith
      - AGENT_TYPE=orchestrator
      - AGENT_TIER=standard
    depends_on:
      nats:
        condition: service_healthy
      redis:
        condition: service_healthy
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - mister-smith-net

  worker:
    build:
      context: .
      dockerfile: Dockerfile.worker
      target: runtime
    environment:
      - RUST_LOG=debug
      - NATS_URL=nats://nats:4222
      - REDIS_URL=redis://redis:6379
      - ORCHESTRATOR_URL=http://orchestrator:8080
      - AGENT_TYPE=worker
      - AGENT_TIER=standard
      - WORKER_CONCURRENCY=2
    depends_on:
      orchestrator:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 15s
      timeout: 5s
      retries: 3
    networks:
      - mister-smith-net
    deploy:
      replicas: 2

volumes:
  redis_data:
  postgres_data:

networks:
  mister-smith-net:
    driver: bridge
```

#### 14.2 Development Override Configuration

```yaml
# docker-compose.override.yml - Development-specific overrides
version: '3.8'

services:
  orchestrator:
    volumes:
      - ./src:/app/src:ro
      - ./target/debug:/app/target/debug
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    ports:
      - "5005:5005"  # Debug port

  worker:
    volumes:
      - ./src:/app/src:ro
      - ./workspace:/app/workspace
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
      - WORKER_CONCURRENCY=1
    ports:
      - "5006:5006"  # Debug port
```

#### 14.3 Monitoring Stack Extension

```yaml
# docker-compose.monitoring.yml - Observability stack
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: mister-smith-prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
    networks:
      - mister-smith-net

  grafana:
    image: grafana/grafana:10.2.0
    container_name: mister-smith-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - mister-smith-net

  jaeger:
    image: jaegertracing/all-in-one:1.51
    container_name: mister-smith-jaeger
    ports:
      - "16686:16686"
      - "14268:14268"
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    networks:
      - mister-smith-net

volumes:
  prometheus_data:
  grafana_data:
```

### 15. Kubernetes Manifest Templates

#### 15.1 Namespace Definitions

```yaml
# namespaces.yaml - Namespace organization
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-system
  labels:
    name: mister-smith-system
    tier: system
---
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-workload
  labels:
    name: mister-smith-workload
    tier: workload
---
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-data
  labels:
    name: mister-smith-data
    tier: data
---
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-monitoring
  labels:
    name: mister-smith-monitoring
    tier: monitoring
```

#### 15.2 Orchestrator Deployment Template

```yaml
# orchestrator-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mister-smith-orchestrator
  namespace: mister-smith-system
  labels:
    app: mister-smith-orchestrator
    component: orchestrator
    tier: system
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: mister-smith-orchestrator
  template:
    metadata:
      labels:
        app: mister-smith-orchestrator
        component: orchestrator
        tier: system
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: mister-smith-orchestrator
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        runAsGroup: 1001
        fsGroup: 1001
      containers:
      - name: orchestrator
        image: mister-smith/orchestrator:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 9090
          name: grpc
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: AGENT_TYPE
          value: "orchestrator"
        - name: AGENT_TIER
          value: "standard"
        - name: NATS_URL
          value: "nats://nats:4222"
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /startup
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 30
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: logs
          mountPath: /app/logs
      - name: metrics-sidecar
        image: prom/node-exporter:v1.7.0
        args:
        - '--path.procfs=/host/proc'
        - '--path.sysfs=/host/sys'
        - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
        ports:
        - containerPort: 9100
          name: metrics
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "100m"
        volumeMounts:
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: sys
          mountPath: /host/sys
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: mister-smith-orchestrator-config
      - name: logs
        emptyDir: {}
      - name: proc
        hostPath:
          path: /proc
      - name: sys
        hostPath:
          path: /sys
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - mister-smith-orchestrator
              topologyKey: kubernetes.io/hostname
      tolerations:
      - key: "node.kubernetes.io/not-ready"
        operator: "Exists"
        effect: "NoExecute"
        tolerationSeconds: 300
      - key: "node.kubernetes.io/unreachable"
        operator: "Exists"
        effect: "NoExecute"
        tolerationSeconds: 300
```

#### 15.3 Worker Deployment with HPA

```yaml
# worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mister-smith-worker
  namespace: mister-smith-workload
  labels:
    app: mister-smith-worker
    component: worker
    tier: workload
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 2
  selector:
    matchLabels:
      app: mister-smith-worker
  template:
    metadata:
      labels:
        app: mister-smith-worker
        component: worker
        tier: workload
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8081"
    spec:
      serviceAccountName: mister-smith-worker
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        runAsGroup: 1001
        fsGroup: 1001
      containers:
      - name: worker
        image: mister-smith/worker:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8081
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: AGENT_TYPE
          value: "worker"
        - name: AGENT_TIER
          valueFrom:
            configMapKeyRef:
              name: mister-smith-worker-config
              key: agent.tier
        - name: NATS_URL
          value: "nats://nats.mister-smith-system:4222"
        - name: WORKER_CONCURRENCY
          valueFrom:
            configMapKeyRef:
              name: mister-smith-worker-config
              key: worker.concurrency
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8081
          initialDelaySeconds: 30
          periodSeconds: 15
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8081
          initialDelaySeconds: 15
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /startup
            port: 8081
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 20
        volumeMounts:
        - name: workspace
          mountPath: /app/workspace
        - name: temp
          mountPath: /tmp
      volumes:
      - name: workspace
        emptyDir:
          sizeLimit: 1Gi
      - name: temp
        emptyDir:
          sizeLimit: 512Mi
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mister-smith-worker-hpa
  namespace: mister-smith-workload
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mister-smith-worker
  minReplicas: 2
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: pending_tasks
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 4
        periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      selectPolicy: Min
```

### 16. Environment Variable Naming Conventions

#### 16.1 Core Environment Variables

```bash
# Core Agent Configuration
AGENT_TYPE=orchestrator|worker|messaging
AGENT_TIER=minimal|standard|premium
AGENT_ID=${POD_NAME}  # Auto-populated in Kubernetes

# Claude CLI Integration
CLAUDE_PARALLEL_DEFAULT=4
CLAUDE_PARALLEL_MAX_AGENTS=50
CLAUDE_PARALLEL_CPU_BUDGET="4.0"
CLAUDE_PARALLEL_MEMORY_BUDGET="4Gi"

# Messaging Configuration
NATS_URL=nats://nats.mister-smith-system:4222
NATS_CLUSTER_ID=mister-smith-cluster
NATS_CLIENT_ID=${AGENT_TYPE}-${POD_NAME}
NATS_SUBJECT_PREFIX=mister-smith

# Persistence Configuration
REDIS_URL=redis://redis.mister-smith-data:6379
DATABASE_URL=postgresql://postgres:password@postgres.mister-smith-data:5432/mister_smith

# Observability Configuration
METRICS_ENABLED=true
METRICS_PORT=9090
TRACING_ENABLED=true
JAEGER_ENDPOINT=http://jaeger.mister-smith-monitoring:14268/api/traces

# Logging Configuration
RUST_LOG=info
LOG_FORMAT=json
LOG_LEVEL=info

# Security Configuration
TLS_ENABLED=true
MTLS_ENABLED=true
CERT_PATH=/etc/tls/certs
KEY_PATH=/etc/tls/private

# Resource Management
WORKER_CONCURRENCY=4
WORKER_TIMEOUT_SECONDS=300
ORCHESTRATOR_HEARTBEAT_INTERVAL=30
HEALTH_CHECK_INTERVAL=15

# Development Configuration (override files only)
RUST_BACKTRACE=1
DEBUG_MODE=true
HOT_RELOAD_ENABLED=true
```

### 17. Resource Allocation Specifications

#### 17.1 Resource Tier Definitions

```yaml
# resource-tiers.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mister-smith-resource-tiers
  namespace: mister-smith-system
data:
  minimal.yaml: |
    resources:
      requests:
        memory: "128Mi"
        cpu: "100m"
      limits:
        memory: "256Mi"
        cpu: "250m"
    priorityClass: "best-effort"
    
  standard.yaml: |
    resources:
      requests:
        memory: "512Mi"
        cpu: "500m"
      limits:
        memory: "1Gi"
        cpu: "1000m"
    priorityClass: "normal"
    
  premium.yaml: |
    resources:
      requests:
        memory: "2Gi"
        cpu: "2000m"
      limits:
        memory: "4Gi"
        cpu: "4000m"
    priorityClass: "high"
---
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: best-effort
value: 100
globalDefault: false
description: "Best effort priority class for minimal tier"
---
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: normal
value: 500
globalDefault: true
description: "Normal priority class for standard tier"
---
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: high
value: 1000
globalDefault: false
description: "High priority class for premium tier"
```

#### 17.2 Resource Quotas (Governance Enhancement)

```yaml
# resource-quotas.yaml - Namespace-level resource governance
apiVersion: v1
kind: ResourceQuota
metadata:
  name: mister-smith-system-quota
  namespace: mister-smith-system
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "20"
    limits.memory: 40Gi
    persistentvolumeclaims: "10"
    services: "20"
    pods: "50"
---
apiVersion: v1
kind: ResourceQuota
metadata:
  name: mister-smith-workload-quota
  namespace: mister-smith-workload
spec:
  hard:
    requests.cpu: "100"
    requests.memory: 200Gi
    limits.cpu: "200"
    limits.memory: 400Gi
    persistentvolumeclaims: "50"
    services: "50"
    pods: "500"
---
apiVersion: v1
kind: ResourceQuota
metadata:
  name: mister-smith-data-quota
  namespace: mister-smith-data
spec:
  hard:
    requests.cpu: "20"
    requests.memory: 100Gi
    limits.cpu: "40"
    limits.memory: 200Gi
    persistentvolumeclaims: "100"
    requests.storage: 1Ti
```

#### 17.3 Pod Security Standards (Security Enhancement)

```yaml
# pod-security-standards.yaml - Enhanced pod security policies
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-system
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
---
apiVersion: v1
kind: Namespace
metadata:
  name: mister-smith-workload
  labels:
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
---
# Pod Security Policy (for clusters < 1.25)
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: mister-smith-restricted
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
  - ALL
  volumes:
  - 'configMap'
  - 'emptyDir'
  - 'projected'
  - 'secret'
  - 'downwardAPI'
  - 'persistentVolumeClaim'
  hostNetwork: false
  hostIPC: false
  hostPID: false
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  supplementalGroups:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
  readOnlyRootFilesystem: true

### 18. Health Check Implementation Templates

#### 18.1 Application Health Check Server (Rust)
```rust
// health.rs - Health check implementation
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[derive(Clone)]
pub struct HealthState {
    pub start_time: Instant,
    pub nats_connected: Arc<tokio::sync::RwLock<bool>>,
    pub redis_connected: Arc<tokio::sync::RwLock<bool>>,
    pub db_connected: Arc<tokio::sync::RwLock<bool>>,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            nats_connected: Arc::new(tokio::sync::RwLock::new(false)),
            redis_connected: Arc::new(tokio::sync::RwLock::new(false)),
            db_connected: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }
}

pub fn health_router() -> Router<HealthState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/startup", get(startup_check))
        .route("/metrics", get(metrics_endpoint))
}

async fn health_check(State(state): State<HealthState>) -> Result<Json<Value>, StatusCode> {
    let uptime = state.start_time.elapsed().as_secs();
    
    Ok(Json(json!({
        "status": "healthy",
        "uptime_seconds": uptime,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "agent_type": std::env::var("AGENT_TYPE").unwrap_or_default()
    })))
}

async fn readiness_check(State(state): State<HealthState>) -> Result<Json<Value>, StatusCode> {
    let nats_ok = *state.nats_connected.read().await;
    let redis_ok = *state.redis_connected.read().await;
    let db_ok = *state.db_connected.read().await;
    
    let ready = nats_ok && redis_ok && db_ok;
    
    let response = json!({
        "ready": ready,
        "checks": {
            "nats": nats_ok,
            "redis": redis_ok,
            "database": db_ok
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if ready {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn startup_check(State(state): State<HealthState>) -> Result<Json<Value>, StatusCode> {
    let uptime = state.start_time.elapsed();
    let startup_complete = uptime > Duration::from_secs(10);
    
    let response = json!({
        "started": startup_complete,
        "uptime_seconds": uptime.as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if startup_complete {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn metrics_endpoint() -> Json<Value> {
    // Prometheus metrics would go here
    Json(json!({
        "metrics_endpoint": "/metrics",
        "format": "prometheus"
    }))
}
```

---

### 19. Service Mesh Implementation

#### 19.1 Istio Service Mesh Configuration

```yaml
# istio-config.yaml - Service mesh for Mister Smith
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: mister-smith-control-plane
  namespace: istio-system
spec:
  values:
    global:
      meshID: mister-smith-mesh
      multiCluster:
        clusterName: mister-smith-cluster
      network: mister-smith-network
    pilot:
      traceSampling: 1.0
  components:
    pilot:
      k8s:
        resources:
          requests:
            cpu: 500m
            memory: 2048Mi
          limits:
            cpu: 1000m
            memory: 4096Mi
        hpaSpec:
          maxReplicas: 5
          minReplicas: 2
          metrics:
          - type: Resource
            resource:
              name: cpu
              target:
                type: Utilization
                averageUtilization: 80
    ingressGateways:
    - name: istio-ingressgateway
      enabled: true
      k8s:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 2000m
            memory: 1024Mi
        hpaSpec:
          maxReplicas: 5
          minReplicas: 2
    egressGateways:
    - name: istio-egressgateway
      enabled: true
```

#### 19.2 Gateway and Virtual Service Configuration

```yaml
# gateway.yaml - Ingress gateway configuration
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: mister-smith-gateway
  namespace: mister-smith-system
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 80
      name: http
      protocol: HTTP
    hosts:
    - api.mister-smith.local
    tls:
      httpsRedirect: true
  - port:
      number: 443
      name: https
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: mister-smith-tls
    hosts:
    - api.mister-smith.local
---
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: mister-smith-orchestrator
  namespace: mister-smith-system
spec:
  hosts:
  - api.mister-smith.local
  gateways:
  - mister-smith-gateway
  http:
  - match:
    - uri:
        prefix: /api/v1/
    route:
    - destination:
        host: mister-smith-orchestrator.mister-smith-system.svc.cluster.local
        port:
          number: 8080
    retries:
      attempts: 3
      perTryTimeout: 30s
      retryOn: gateway-error,connect-failure,refused-stream
    timeout: 60s
  - match:
    - uri:
        prefix: /health
    route:
    - destination:
        host: mister-smith-orchestrator.mister-smith-system.svc.cluster.local
        port:
          number: 8080
```

#### 19.3 Traffic Policies and Circuit Breaker

```yaml
# traffic-policy.yaml - Advanced traffic management
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: mister-smith-orchestrator
  namespace: mister-smith-system
spec:
  host: mister-smith-orchestrator.mister-smith-system.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
        connectTimeout: 30s
        tcpKeepalive:
          time: 7200s
          interval: 75s
      http:
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
        maxRetries: 3
        consecutiveGatewayErrors: 5
        interval: 30s
        baseEjectionTime: 30s
    circuitBreaker:
      consecutiveGatewayErrors: 5
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
      minHealthPercent: 50
    loadBalancer:
      simple: LEAST_CONN
    outlierDetection:
      consecutiveGatewayErrors: 5
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
      minHealthPercent: 50
---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: mister-smith-worker
  namespace: mister-smith-workload
spec:
  host: mister-smith-worker.mister-smith-workload.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 50
        connectTimeout: 10s
      http:
        http1MaxPendingRequests: 50
        http2MaxRequests: 100
        maxRequestsPerConnection: 5
        maxRetries: 2
    loadBalancer:
      consistentHash:
        httpHeaderName: "x-agent-affinity"
```

### 20. Blue-Green Deployment Implementation

#### 20.1 Blue-Green Deployment Script

```bash
#!/bin/bash
# blue-green-deploy.sh - Blue-green deployment automation

set -euo pipefail

NAMESPACE=${NAMESPACE:-mister-smith-workload}
APP_NAME=${APP_NAME:-mister-smith-worker}
NEW_VERSION=${NEW_VERSION:-latest}
HEALTH_CHECK_TIMEOUT=${HEALTH_CHECK_TIMEOUT:-300}
ROLLBACK_ON_FAILURE=${ROLLBACK_ON_FAILURE:-true}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        error "kubectl not found"
    fi
    
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        error "Namespace $NAMESPACE not found"
    fi
    
    log "Prerequisites check passed"
}

get_current_color() {
    local selector=$(kubectl get service "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "")
    
    if [[ "$selector" == *"blue"* ]]; then
        echo "blue"
    elif [[ "$selector" == *"green"* ]]; then
        echo "green"
    else
        echo "blue"  # Default to blue if no color found
    fi
}

get_inactive_color() {
    local current_color=$1
    if [[ "$current_color" == "blue" ]]; then
        echo "green"
    else
        echo "blue"
    fi
}

deploy_new_version() {
    local target_color=$1
    local deployment_name="${APP_NAME}-${target_color}"
    
    log "Deploying version $NEW_VERSION to $target_color environment..."
    
    # Update deployment with new image
    kubectl set image deployment/"$deployment_name" \
        worker="mister-smith/worker:$NEW_VERSION" \
        -n "$NAMESPACE"
    
    # Wait for rollout to complete
    kubectl rollout status deployment/"$deployment_name" \
        -n "$NAMESPACE" \
        --timeout=600s
    
    log "Deployment to $target_color completed"
}

health_check() {
    local target_color=$1
    local service_name="${APP_NAME}-${target_color}"
    local port=$(kubectl get service "$service_name" -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].port}')
    
    log "Performing health checks for $target_color environment..."
    
    local end_time=$((SECONDS + HEALTH_CHECK_TIMEOUT))
    
    while [ $SECONDS -lt $end_time ]; do
        if kubectl exec -n "$NAMESPACE" deployment/"${APP_NAME}-${target_color}" -- \
           curl -f -s "http://localhost:8081/health" > /dev/null; then
            log "Health check passed for $target_color"
            return 0
        fi
        
        log "Health check failed, retrying in 10 seconds..."
        sleep 10
    done
    
    error "Health check timed out for $target_color environment"
}

switch_traffic() {
    local target_color=$1
    
    log "Switching traffic to $target_color environment..."
    
    # Update service selector
    kubectl patch service "$APP_NAME" -n "$NAMESPACE" -p \
        "{\"spec\":{\"selector\":{\"version\":\"$target_color\"}}}"
    
    # Verify traffic switch
    sleep 10
    local current_selector=$(kubectl get service "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.spec.selector.version}')
    
    if [[ "$current_selector" == "$target_color" ]]; then
        log "Traffic successfully switched to $target_color"
    else
        error "Failed to switch traffic to $target_color"
    fi
}

cleanup_old_version() {
    local old_color=$1
    
    log "Cleaning up $old_color environment..."
    
    # Scale down old deployment
    kubectl scale deployment "${APP_NAME}-${old_color}" --replicas=0 -n "$NAMESPACE"
    
    log "Scaled down $old_color environment"
}

rollback() {
    local rollback_color=$1
    
    warn "Rolling back to $rollback_color environment..."
    
    switch_traffic "$rollback_color"
    
    # Scale up rollback environment if needed
    kubectl scale deployment "${APP_NAME}-${rollback_color}" --replicas=2 -n "$NAMESPACE"
    
    log "Rollback completed to $rollback_color"
}

main() {
    log "Starting blue-green deployment for $APP_NAME version $NEW_VERSION"
    
    check_prerequisites
    
    local current_color=$(get_current_color)
    local target_color=$(get_inactive_color "$current_color")
    
    log "Current active environment: $current_color"
    log "Target deployment environment: $target_color"
    
    # Deploy new version to inactive environment
    deploy_new_version "$target_color"
    
    # Perform health checks
    if ! health_check "$target_color"; then
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            rollback "$current_color"
        fi
        exit 1
    fi
    
    # Switch traffic to new version
    switch_traffic "$target_color"
    
    # Final health check after traffic switch
    sleep 30
    if ! health_check "$target_color"; then
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            rollback "$current_color"
        fi
        exit 1
    fi
    
    # Cleanup old environment
    cleanup_old_version "$current_color"
    
    log "Blue-green deployment completed successfully!"
    log "Active environment: $target_color"
    log "Version: $NEW_VERSION"
}

# Handle signals for cleanup
trap 'error "Deployment interrupted"' INT TERM

main "$@"
```

#### 20.2 Blue-Green Service Configuration

```yaml
# blue-green-services.yaml - Service definitions for blue-green deployment
apiVersion: v1
kind: Service
metadata:
  name: mister-smith-worker
  namespace: mister-smith-workload
  labels:
    app: mister-smith-worker
    deployment-strategy: blue-green
spec:
  selector:
    app: mister-smith-worker
    version: blue  # This will be updated during deployment
  ports:
  - name: http
    port: 8081
    targetPort: 8081
    protocol: TCP
  type: ClusterIP
---
apiVersion: v1
kind: Service
metadata:
  name: mister-smith-worker-blue
  namespace: mister-smith-workload
  labels:
    app: mister-smith-worker
    version: blue
spec:
  selector:
    app: mister-smith-worker
    version: blue
  ports:
  - name: http
    port: 8081
    targetPort: 8081
    protocol: TCP
  type: ClusterIP
---
apiVersion: v1
kind: Service
metadata:
  name: mister-smith-worker-green
  namespace: mister-smith-workload
  labels:
    app: mister-smith-worker
    version: green
spec:
  selector:
    app: mister-smith-worker
    version: green
  ports:
  - name: http
    port: 8081
    targetPort: 8081
    protocol: TCP
  type: ClusterIP
```

### 21. Automated Deployment Pipeline

*Cross-references:*

- [Build Specifications](./build-specifications.md)
- [Configuration Deployment Specifications](./configuration-deployment-specifications.md)
- [Monitoring and Health](../core-architecture/monitoring-and-health.md)

#### 21.1 GitHub Actions CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml - Automated deployment pipeline
name: Deploy Mister Smith

on:
  push:
    branches: [main, develop]
    tags: ['v*']
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: mister-smith

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    outputs:
      image-tag: ${{ steps.meta.outputs.tags }}
      image-digest: ${{ steps.build.outputs.digest }}
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
      
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
        
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=semver,pattern={{major}}.{{minor}}
          type=sha,prefix={{branch}}-
          
    - name: Build and push orchestrator image
      id: build-orchestrator
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile.orchestrator
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}-orchestrator
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
        
    - name: Build and push worker image
      id: build-worker
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile.worker
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}-worker
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  security-scan:
    runs-on: ubuntu-latest
    needs: build
    permissions:
      security-events: write
    
    steps:
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: ${{ needs.build.outputs.image-tag }}-orchestrator
        format: 'sarif'
        output: 'trivy-results.sarif'
        
    - name: Upload Trivy scan results to GitHub Security tab
      uses: github/codeql-action/upload-sarif@v3
      with:
        sarif_file: 'trivy-results.sarif'

  deploy-dev:
    runs-on: ubuntu-latest
    needs: [build, security-scan]
    if: github.ref == 'refs/heads/develop'
    environment: development
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Configure kubectl
      uses: azure/k8s-set-context@v3
      with:
        method: kubeconfig
        kubeconfig: ${{ secrets.KUBE_CONFIG_DEV }}
        
    - name: Deploy to development
      run: |
        helm upgrade --install mister-smith-dev ./helm/mister-smith \
          --namespace mister-smith-dev \
          --create-namespace \
          --set image.tag=${{ github.sha }} \
          --set environment=development \
          --set replicaCount=1 \
          --set resources.requests.cpu=100m \
          --set resources.requests.memory=256Mi \
          --wait --timeout=600s
          
    - name: Run smoke tests
      run: |
        kubectl wait --for=condition=ready pod -l app=mister-smith-orchestrator \
          -n mister-smith-dev --timeout=300s
        kubectl exec -n mister-smith-dev deployment/mister-smith-orchestrator -- \
          curl -f http://localhost:8080/health

  deploy-staging:
    runs-on: ubuntu-latest
    needs: [build, deploy-dev]
    if: github.ref == 'refs/heads/main'
    environment: staging
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Configure kubectl
      uses: azure/k8s-set-context@v3
      with:
        method: kubeconfig
        kubeconfig: ${{ secrets.KUBE_CONFIG_STAGING }}
        
    - name: Deploy to staging
      run: |
        helm upgrade --install mister-smith-staging ./helm/mister-smith \
          --namespace mister-smith-staging \
          --create-namespace \
          --set image.tag=${{ github.sha }} \
          --set environment=staging \
          --set replicaCount=2 \
          --set autoscaling.enabled=true \
          --set autoscaling.minReplicas=2 \
          --set autoscaling.maxReplicas=10 \
          --wait --timeout=600s
          
    - name: Run integration tests
      run: |
        kubectl wait --for=condition=ready pod -l app=mister-smith-orchestrator \
          -n mister-smith-staging --timeout=300s
        # Add integration test commands here

  deploy-production:
    runs-on: ubuntu-latest
    needs: [build, deploy-staging]
    if: startsWith(github.ref, 'refs/tags/v')
    environment: production
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Configure kubectl
      uses: azure/k8s-set-context@v3
      with:
        method: kubeconfig
        kubeconfig: ${{ secrets.KUBE_CONFIG_PROD }}
        
    - name: Blue-Green Production Deployment
      run: |
        export NEW_VERSION=${{ github.sha }}
        export NAMESPACE=mister-smith-production
        ./scripts/blue-green-deploy.sh
        
    - name: Notify deployment success
      uses: 8398a7/action-slack@v3
      with:
        status: success
        channel: '#deployments'
        text: 'Production deployment completed successfully!'
      env:
        SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

#### 21.2 Helm Chart Implementation

```yaml
# helm/mister-smith/Chart.yaml
apiVersion: v2
name: mister-smith
description: A Helm chart for Mister Smith AI Agent Framework
type: application
version: 0.1.0
appVersion: "1.0.0"

dependencies:
- name: postgresql
  version: 12.12.10
  repository: https://charts.bitnami.com/bitnami
  condition: postgresql.enabled
- name: redis
  version: 18.1.5
  repository: https://charts.bitnami.com/bitnami
  condition: redis.enabled
- name: nats
  version: 1.1.5
  repository: https://nats-io.github.io/k8s/helm/charts/
  condition: nats.enabled
```

```yaml
# helm/mister-smith/values.yaml
# Default values for mister-smith
global:
  environment: development
  imageRegistry: ghcr.io
  imagePullSecrets: []

image:
  registry: ghcr.io
  repository: mister-smith
  tag: latest
  pullPolicy: IfNotPresent

orchestrator:
  enabled: true
  replicaCount: 3
  image:
    repository: mister-smith/orchestrator
    tag: ""
  service:
    type: ClusterIP
    port: 8080
    grpcPort: 9090
  resources:
    requests:
      memory: "512Mi"
      cpu: "500m"
    limits:
      memory: "1Gi"
      cpu: "1000m"

worker:
  enabled: true
  replicaCount: 2
  image:
    repository: mister-smith/worker
    tag: ""
  resources:
    requests:
      memory: "256Mi"
      cpu: "250m"
    limits:
      memory: "512Mi"
      cpu: "500m"
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 50
    targetCPUUtilizationPercentage: 70
    targetMemoryUtilizationPercentage: 80

serviceAccount:
  create: true
  annotations: {}
  name: ""

podAnnotations: {}

podSecurityContext:
  runAsNonRoot: true
  runAsUser: 1001
  runAsGroup: 1001
  fsGroup: 1001

securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true

ingress:
  enabled: false
  className: ""
  annotations: {}
  hosts:
  - host: api.mister-smith.local
    paths:
    - path: /
      pathType: Prefix
  tls: []

# External dependencies
postgresql:
  enabled: true
  auth:
    username: mister_smith
    database: mister_smith
    existingSecret: mister-smith-postgres-secret

redis:
  enabled: true
  auth:
    enabled: true
    existingSecret: mister-smith-redis-secret

nats:
  enabled: true
  nats:
    jetstream:
      enabled: true
    limits:
      maxPayload: 8MB
  cluster:
    enabled: true
    replicas: 3

monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
    namespace: mister-smith-monitoring
  grafana:
    dashboards:
      enabled: true

istio:
  enabled: false
  gateway:
    enabled: false
  virtualService:
    enabled: false

nodeSelector: {}

tolerations: []

affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchExpressions:
          - key: app.kubernetes.io/name
            operator: In
            values:
            - mister-smith
        topologyKey: kubernetes.io/hostname
```

### 22. Infrastructure as Code with Terraform

#### 22.1 AWS EKS Cluster Provisioning

```hcl
# terraform/main.tf - AWS EKS cluster for Mister Smith
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
  }
  
  backend "s3" {
    bucket         = "mister-smith-terraform-state"
    key            = "eks/terraform.tfstate"
    region         = "us-west-2"
    dynamodb_table = "mister-smith-terraform-locks"
    encrypt        = true
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "mister-smith"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

data "aws_availability_zones" "available" {
  filter {
    name   = "opt-in-status"
    values = ["opt-in-not-required"]
  }
}

# VPC Configuration
module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "${var.cluster_name}-vpc"
  cidr = var.vpc_cidr

  azs             = slice(data.aws_availability_zones.available.names, 0, 3)
  private_subnets = var.private_subnets
  public_subnets  = var.public_subnets

  enable_nat_gateway   = true
  single_nat_gateway   = false
  enable_dns_hostnames = true
  enable_dns_support   = true

  enable_flow_log                      = true
  create_flow_log_cloudwatch_iam_role  = true
  create_flow_log_cloudwatch_log_group = true

  public_subnet_tags = {
    "kubernetes.io/role/elb" = 1
  }

  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = 1
  }
}

# EKS Cluster
module "eks" {
  source = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  cluster_name    = var.cluster_name
  cluster_version = var.kubernetes_version

  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  cluster_endpoint_public_access = true
  cluster_endpoint_private_access = true

  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }

  # Node groups
  eks_managed_node_groups = {
    system = {
      name = "system-nodes"
      
      instance_types = ["m5.large"]
      capacity_type  = "ON_DEMAND"
      
      min_size     = 2
      max_size     = 4
      desired_size = 3
      
      labels = {
        role = "system"
      }
      
      taints = {
        system = {
          key    = "node-role.kubernetes.io/system"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
    }
    
    workers = {
      name = "worker-nodes"
      
      instance_types = ["m5.xlarge", "m5.2xlarge"]
      capacity_type  = "SPOT"
      
      min_size     = 2
      max_size     = 50
      desired_size = 4
      
      labels = {
        role = "worker"
      }
      
      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size           = 100
            volume_type           = "gp3"
            iops                  = 3000
            throughput            = 150
            encrypted             = true
            delete_on_termination = true
          }
        }
      }
    }
    
    data = {
      name = "data-nodes"
      
      instance_types = ["r5.large", "r5.xlarge"]
      capacity_type  = "ON_DEMAND"
      
      min_size     = 1
      max_size     = 5
      desired_size = 2
      
      labels = {
        role = "data"
      }
      
      taints = {
        data = {
          key    = "node-role.kubernetes.io/data"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
    }
  }

  # aws-auth configmap
  manage_aws_auth_configmap = true

  aws_auth_roles = [
    {
      rolearn  = module.eks_admins_iam_role.iam_role_arn
      username = "cluster-admin"
      groups   = ["system:masters"]
    },
  ]

  aws_auth_users = var.map_users
  aws_auth_accounts = var.map_accounts

  cluster_security_group_additional_rules = {
    ingress_nodes_ephemeral_ports_tcp = {
      description                = "Node groups to cluster API"
      protocol                   = "tcp"
      from_port                  = 1025
      to_port                    = 65535
      type                       = "ingress"
      source_node_security_group = true
    }
  }

  node_security_group_additional_rules = {
    ingress_self_all = {
      description = "Node to node all ports/protocols"
      protocol    = "-1"
      from_port   = 0
      to_port     = 0
      type        = "ingress"
      self        = true
    }
    
    egress_all = {
      description      = "Node all egress"
      protocol         = "-1"
      from_port        = 0
      to_port          = 0
      type             = "egress"
      cidr_blocks      = ["0.0.0.0/0"]
      ipv6_cidr_blocks = ["::/0"]
    }
  }
}

# IAM role for EKS admins
module "eks_admins_iam_role" {
  source = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name             = "${var.cluster_name}-eks-admins"
  attach_admin_policy   = true
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:cluster-admin"]
    }
  }
}

# AWS Load Balancer Controller
module "load_balancer_controller_irsa_role" {
  source = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name = "${var.cluster_name}-load-balancer-controller"

  attach_load_balancer_controller_policy = true

  oidc_providers = {
    ex = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:aws-load-balancer-controller"]
    }
  }
}

# EBS CSI Driver
module "ebs_csi_irsa_role" {
  source = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name = "${var.cluster_name}-ebs-csi"

  attach_ebs_csi_policy = true

  oidc_providers = {
    ex = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
}

# Cluster Autoscaler
module "cluster_autoscaler_irsa_role" {
  source = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name = "${var.cluster_name}-cluster-autoscaler"

  attach_cluster_autoscaler_policy = true
  cluster_autoscaler_cluster_names = [module.eks.cluster_name]

  oidc_providers = {
    ex = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:cluster-autoscaler"]
    }
  }
}

# RDS PostgreSQL for persistent storage
module "db" {
  source = "terraform-aws-modules/rds/aws"
  version = "~> 6.0"

  identifier = "${var.cluster_name}-postgres"

  engine            = "postgres"
  engine_version    = "15.4"
  instance_class    = var.db_instance_class
  allocated_storage = var.db_allocated_storage
  storage_encrypted = true

  db_name  = "mister_smith"
  username = "postgres"
  port     = "5432"

  manage_master_user_password = true

  vpc_security_group_ids = [module.db_security_group.security_group_id]

  maintenance_window = "Mon:00:00-Mon:03:00"
  backup_window      = "03:00-06:00"

  monitoring_interval    = "60"
  monitoring_role_name   = "${var.cluster_name}-rds-monitoring-role"
  create_monitoring_role = true

  tags = {
    Name = "${var.cluster_name}-postgres"
  }

  subnet_group_name   = module.db_subnet_group.db_subnet_group_id
  family              = "postgres15"
  major_engine_version = "15"

  deletion_protection = var.environment == "production"

  parameters = [
    {
      name  = "log_connections"
      value = 1
    }
  ]
}

module "db_subnet_group" {
  source = "terraform-aws-modules/rds/aws//modules/db_subnet_group"
  version = "~> 6.0"

  name       = "${var.cluster_name}-postgres"
  subnet_ids = module.vpc.private_subnets
}

module "db_security_group" {
  source = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "${var.cluster_name}-postgres"
  description = "Security group for RDS PostgreSQL"
  vpc_id      = module.vpc.vpc_id

  ingress_with_source_security_group_id = [
    {
      from_port                = 5432
      to_port                  = 5432
      protocol                 = "tcp"
      description              = "PostgreSQL access from EKS cluster"
      source_security_group_id = module.eks.cluster_security_group_id
    },
  ]
}

# ElastiCache Redis for caching
module "redis" {
  source = "terraform-aws-modules/elasticache/aws"
  version = "~> 1.0"

  cluster_id               = "${var.cluster_name}-redis"
  description              = "Redis cluster for Mister Smith"

  engine_version           = "7.0"
  node_type               = var.redis_node_type
  port                    = 6379
  parameter_group_name    = "default.redis7"

  num_cache_nodes         = 1

  subnet_group_name       = "${var.cluster_name}-redis"
  subnet_ids              = module.vpc.private_subnets
  security_group_ids      = [module.redis_security_group.security_group_id]

  apply_immediately       = true
  auto_minor_version_upgrade = false
  maintenance_window         = "sun:05:00-sun:06:00"
  
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                = var.redis_auth_token
}

module "redis_security_group" {
  source = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "${var.cluster_name}-redis"
  description = "Security group for ElastiCache Redis"
  vpc_id      = module.vpc.vpc_id

  ingress_with_source_security_group_id = [
    {
      from_port                = 6379
      to_port                  = 6379
      protocol                 = "tcp"
      description              = "Redis access from EKS cluster"
      source_security_group_id = module.eks.cluster_security_group_id
    },
  ]
}
```

#### 22.2 Terraform Variables

```hcl
# terraform/variables.tf
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "development"
}

variable "cluster_name" {
  description = "Name of the EKS cluster"
  type        = string
  default     = "mister-smith"
}

variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "private_subnets" {
  description = "Private subnet CIDR blocks"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnets" {
  description = "Public subnet CIDR blocks"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

variable "db_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_allocated_storage" {
  description = "The allocated storage in gigabytes"
  type        = number
  default     = 20
}

variable "redis_node_type" {
  description = "ElastiCache node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_auth_token" {
  description = "Auth token for Redis"
  type        = string
  sensitive   = true
}

variable "map_users" {
  description = "Additional IAM users to add to the aws-auth configmap"
  type = list(object({
    userarn  = string
    username = string
    groups   = list(string)
  }))
  default = []
}

variable "map_accounts" {
  description = "Additional AWS account numbers to add to the aws-auth configmap"
  type        = list(string)
  default     = []
}
```

### 23. Auto-scaling Implementation

#### 23.1 Vertical Pod Autoscaler Configuration

```yaml
# vpa-config.yaml - Vertical Pod Autoscaler for optimal resource allocation
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: mister-smith-orchestrator-vpa
  namespace: mister-smith-system
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mister-smith-orchestrator
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: orchestrator
      minAllowed:
        cpu: 100m
        memory: 128Mi
      maxAllowed:
        cpu: 2000m
        memory: 4Gi
      controlledResources: ["cpu", "memory"]
      controlledValues: RequestsAndLimits
    - containerName: metrics-sidecar
      mode: "Off"  # Don't auto-scale sidecar
---
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: mister-smith-worker-vpa
  namespace: mister-smith-workload
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mister-smith-worker
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: worker
      minAllowed:
        cpu: 50m
        memory: 64Mi
      maxAllowed:
        cpu: 1000m
        memory: 2Gi
      controlledResources: ["cpu", "memory"]
      controlledValues: RequestsAndLimits
```

#### 23.2 Custom Metrics for HPA

```yaml
# custom-metrics-hpa.yaml - HPA with custom metrics
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mister-smith-worker-custom-hpa
  namespace: mister-smith-workload
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mister-smith-worker
  minReplicas: 2
  maxReplicas: 100
  metrics:
  # Standard resource metrics
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  # Custom application metrics
  - type: Pods
    pods:
      metric:
        name: pending_tasks_per_pod
      target:
        type: AverageValue
        averageValue: "50"
  - type: Pods
    pods:
      metric:
        name: task_processing_rate_per_pod
      target:
        type: AverageValue
        averageValue: "10"
  # External metrics (from NATS)
  - type: External
    external:
      metric:
        name: nats_pending_messages
        selector:
          matchLabels:
            queue: "mister-smith.tasks"
      target:
        type: Value
        value: "1000"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 5
        periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 25
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Min
```

#### 23.3 Cluster Autoscaler Configuration

```yaml
# cluster-autoscaler.yaml - Cluster-level auto-scaling
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
  labels:
    app: cluster-autoscaler
spec:
  selector:
    matchLabels:
      app: cluster-autoscaler
  template:
    metadata:
      labels:
        app: cluster-autoscaler
      annotations:
        prometheus.io/scrape: 'true'
        prometheus.io/port: '8085'
    spec:
      priorityClassName: system-cluster-critical
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
        fsGroup: 65534
      serviceAccountName: cluster-autoscaler
      containers:
      - image: registry.k8s.io/autoscaling/cluster-autoscaler:v1.28.2
        name: cluster-autoscaler
        resources:
          limits:
            cpu: 100m
            memory: 600Mi
          requests:
            cpu: 100m
            memory: 600Mi
        command:
        - ./cluster-autoscaler
        - --v=4
        - --stderrthreshold=info
        - --cloud-provider=aws
        - --skip-nodes-with-local-storage=false
        - --expander=least-waste
        - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/mister-smith
        - --balance-similar-node-groups
        - --scale-down-enabled=true
        - --scale-down-delay-after-add=10m
        - --scale-down-unneeded-time=10m
        - --scale-down-utilization-threshold=0.5
        - --scale-down-non-empty-candidates-count=30
        - --max-node-provision-time=15m
        - --scan-interval=10s
        - --skip-nodes-with-system-pods=false
        env:
        - name: AWS_REGION
          value: us-west-2
        volumeMounts:
        - name: ssl-certs
          mountPath: /etc/ssl/certs/ca-certificates.crt
          readOnly: true
        imagePullPolicy: "Always"
      volumes:
      - name: ssl-certs
        hostPath:
          path: "/etc/ssl/certs/ca-bundle.crt"
---
apiVersion: v1
kind: ServiceAccount
metadata:
  labels:
    k8s-addon: cluster-autoscaler.addons.k8s.io
    k8s-app: cluster-autoscaler
  name: cluster-autoscaler
  namespace: kube-system
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::ACCOUNT_ID:role/mister-smith-cluster-autoscaler
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: cluster-autoscaler
  labels:
    k8s-addon: cluster-autoscaler.addons.k8s.io
    k8s-app: cluster-autoscaler
rules:
- apiGroups: [""]
  resources: ["events", "endpoints"]
  verbs: ["create", "patch"]
- apiGroups: [""]
  resources: ["pods/eviction"]
  verbs: ["create"]
- apiGroups: [""]
  resources: ["pods/status"]
  verbs: ["update"]
- apiGroups: [""]
  resources: ["endpoints"]
  resourceNames: ["cluster-autoscaler"]
  verbs: ["get", "update"]
- apiGroups: [""]
  resources: ["nodes"]
  verbs: ["watch", "list", "get", "update"]
- apiGroups: [""]
  resources: ["namespaces", "pods", "services", "replicationcontrollers", "persistentvolumeclaims", "persistentvolumes"]
  verbs: ["watch", "list", "get"]
- apiGroups: ["extensions"]
  resources: ["replicasets", "daemonsets"]
  verbs: ["watch", "list", "get"]
- apiGroups: ["policy"]
  resources: ["poddisruptionbudgets"]
  verbs: ["watch", "list"]
- apiGroups: ["apps"]
  resources: ["statefulsets", "replicasets", "daemonsets"]
  verbs: ["watch", "list", "get"]
- apiGroups: ["storage.k8s.io"]
  resources: ["storageclasses", "csinodes", "csidrivers", "csistoragecapacities"]
  verbs: ["watch", "list", "get"]
- apiGroups: ["batch", "extensions"]
  resources: ["jobs"]
  verbs: ["get", "list", "watch", "patch"]
- apiGroups: ["coordination.k8s.io"]
  resources: ["leases"]
  verbs: ["create"]
- apiGroups: ["coordination.k8s.io"]
  resourceNames: ["cluster-autoscaler"]
  resources: ["leases"]
  verbs: ["get", "update"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: cluster-autoscaler
  labels:
    k8s-addon: cluster-autoscaler.addons.k8s.io
    k8s-app: cluster-autoscaler
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: cluster-autoscaler
subjects:
- kind: ServiceAccount
  name: cluster-autoscaler
  namespace: kube-system
```

---

This deployment architecture specification provides comprehensive patterns and concrete implementation templates for deploying multi-agent systems at scale,
including Docker best practices, Kubernetes manifests, service mesh configuration, blue-green deployments, automated CI/CD pipelines, infrastructure as code,
and advanced auto-scaling strategies suitable for production deployment of the Mister Smith AI Agent Framework.
