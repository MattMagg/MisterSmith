---
title: Observability & Monitoring Framework
type: technical-specification
permalink: operations/observability-monitoring-framework
---

# Observability & Monitoring Framework

## Technical Specifications for Multi-Agent Systems

This document provides implementation patterns and technical specifications for observability and monitoring in the MisterSmith framework.

### Technical Scope

**Implementation Patterns**:

- OpenTelemetry instrumentation patterns for multi-agent tracing
- Metrics collection and aggregation strategies
- Structured logging with correlation IDs
- Distributed tracing across agent boundaries
- Performance monitoring and alerting

**Technology Stack**:

- **Telemetry**: OpenTelemetry (traces, metrics, logs)
- **Metrics Storage**: Prometheus with custom exporters
- **Trace Storage**: Jaeger for distributed tracing
- **Log Aggregation**: ELK Stack (Elasticsearch, Logstash, Kibana)
- **Visualization**: Grafana dashboards and alerts
- **Transport**: OTLP (OpenTelemetry Protocol) over gRPC/HTTP

### Cross-References

**Core Architecture**:

- **[System Integration](../core-architecture/system-integration.md)** - Component integration patterns
- **[Supervision Trees](../core-architecture/supervision-trees.md)** - Agent supervision and error handling
- **[Tokio Runtime](../core-architecture/tokio-runtime.md)** - Async runtime configuration
- **[Component Architecture](../core-architecture/component-architecture.md)** - Agent component structure

**Operations**:

- **[Deployment Configuration](../operations/deployment-configuration.md)** - Infrastructure setup and deployment patterns
- **[Process Management](../operations/process-management.md)** - Agent lifecycle and supervision patterns
- **[Container Orchestration](../operations/container-orchestration.md)** - Docker and Kubernetes deployment

**Data Management**:

- **[Agent Communication](../data-management/agent-communication.md)** - Message patterns and protocols
- **[Message Schemas](../data-management/message-schemas.md)** - Data structure specifications
- **[Agent Lifecycle](../data-management/agent-lifecycle.md)** - State management patterns

**Transport & Security**:

- **[NATS Integration](../transport/nats-integration.md)** - Message transport configuration
- **[Authentication](../security/authentication.md)** - Agent authentication patterns

### 1. Architecture Overview

#### 1.1 Core Observability Components Pattern

```rust
PATTERN ObservabilityArchitecture:
    COMPONENTS:
        instrumentation_layer
        collection_pipeline
        storage_backend
        analysis_engine
        visualization_layer
        alerting_system
```

#### 1.2 Data Flow Pattern

```rust
PATTERN ObservabilityPipeline:
    STAGES:
        agent_instrumentation
        local_buffering
        centralized_collection
        processing_enrichment
        persistent_storage
        real_time_analysis
        user_presentation
```

### 2. Instrumentation Patterns

#### 2.1 Agent Instrumentation Pattern

```rust
PATTERN AgentInstrumentation:
    initialize_telemetry_context()
    configure_data_exporters()
    set_resource_attributes()
    enable_context_propagation()
    configure_sampling_strategy()
```

#### 2.2 OpenTelemetry SDK Initialization

```rust
// Rust implementation example
use opentelemetry::{global, sdk::{
    export::trace::stdout,
    propagation::TraceContextPropagator,
    resource::{EnvResourceDetector, SdkProvidedResourceDetector},
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
}};
use opentelemetry_otlp::{Protocol, WithExportConfig};

PATTERN OTLPInitialization:
    pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
        // Set global propagator
        global::set_text_map_propagator(TraceContextPropagator::new());
        
        // Configure resource
        let resource = Resource::from_detectors(
            std::time::Duration::from_secs(3),
            vec![
                Box::new(EnvResourceDetector::new()),
                Box::new(SdkProvidedResourceDetector),
            ],
        )
        .merge(&Resource::new(vec![
            KeyValue::new("service.name", "mister-smith-agent"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("deployment.environment", std::env::var("ENVIRONMENT").unwrap_or("development".into())),
        ]));
        
        // Configure OTLP exporter
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://localhost:4317")
            .with_protocol(Protocol::Grpc)
            .with_timeout(std::time::Duration::from_secs(3));
            
        // Build trace provider
        let trace_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(32)
                    .with_max_links_per_span(16)
                    .with_resource(resource),
            )
            .install_batch(opentelemetry::runtime::Tokio)?;
            
        global::set_tracer_provider(trace_provider);
        Ok(())
    }
```

#### 2.3 Custom Metrics Collection Implementation

```rust
PATTERN CustomMetricsImplementation:
    use opentelemetry::{
        metrics::{Counter, Histogram, ObservableGauge, Unit},
        KeyValue,
    };
    use std::sync::Arc;
    
    pub struct AgentMetrics {
        // Counters
        pub task_completions: Counter<u64>,
        pub task_errors: Counter<u64>,
        pub messages_sent: Counter<u64>,
        pub messages_received: Counter<u64>,
        
        // Histograms
        pub task_duration: Histogram<f64>,
        pub message_latency: Histogram<f64>,
        pub processing_time: Histogram<f64>,
        
        // Gauges
        pub active_agents: Arc<AtomicI64>,
        pub queue_depth: Arc<AtomicI64>,
        pub memory_usage: Arc<AtomicI64>,
    }
    
    impl AgentMetrics {
        pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
            let active_agents = Arc::new(AtomicI64::new(0));
            let queue_depth = Arc::new(AtomicI64::new(0));
            let memory_usage = Arc::new(AtomicI64::new(0));
            
            // Register observable gauges
            let active_agents_clone = active_agents.clone();
            meter
                .i64_observable_gauge("agent_active_count")
                .with_description("Current number of active agents")
                .with_unit(Unit::new("{agents}"))
                .with_callback(move |observer| {
                    observer.observe(
                        active_agents_clone.load(Ordering::Relaxed),
                        &[KeyValue::new("state", "active")],
                    );
                })
                .init();
                
            Self {
                task_completions: meter
                    .u64_counter("task_completions_total")
                    .with_description("Total number of completed tasks")
                    .with_unit(Unit::new("{tasks}"))
                    .init(),
                    
                task_errors: meter
                    .u64_counter("task_errors_total")
                    .with_description("Total number of failed tasks")
                    .with_unit(Unit::new("{errors}"))
                    .init(),
                    
                task_duration: meter
                    .f64_histogram("task_duration_seconds")
                    .with_description("Task execution duration distribution")
                    .with_unit(Unit::new("s"))
                    .init(),
                    
                message_latency: meter
                    .f64_histogram("message_latency_seconds")
                    .with_description("Message delivery latency")
                    .with_unit(Unit::new("s"))
                    .init(),
                    
                messages_sent: meter
                    .u64_counter("messages_sent_total")
                    .with_description("Total messages sent")
                    .init(),
                    
                messages_received: meter
                    .u64_counter("messages_received_total")
                    .with_description("Total messages received")
                    .init(),
                    
                processing_time: meter
                    .f64_histogram("processing_time_seconds")
                    .with_description("Processing time per operation")
                    .with_unit(Unit::new("s"))
                    .init(),
                    
                active_agents,
                queue_depth,
                memory_usage,
            }
        }
        
        // Helper methods for metric updates
        pub fn record_task_completion(&self, task_type: &str, duration: f64, success: bool) {
            let labels = &[KeyValue::new("task_type", task_type.to_string())];
            
            if success {
                self.task_completions.add(1, labels);
            } else {
                self.task_errors.add(1, labels);
            }
            
            self.task_duration.record(duration, labels);
        }
        
        pub fn record_message_sent(&self, msg_type: &str, latency: f64) {
            let labels = &[
                KeyValue::new("message_type", msg_type.to_string()),
                KeyValue::new("direction", "outbound"),
            ];
            
            self.messages_sent.add(1, labels);
            self.message_latency.record(latency, labels);
        }
    }
```

#### 2.4 Context Propagation Pattern

```rust
PATTERN ContextPropagationImplementation:
    use opentelemetry::{
        global,
        propagation::{Injector, Extractor, TextMapPropagator},
        Context,
    };
    use std::collections::HashMap;
    
    // Complete context propagation implementation
    pub struct ContextPropagator {
        propagator: Box<dyn TextMapPropagator + Send + Sync>,
    }
    
    impl ContextPropagator {
        pub fn new() -> Self {
            Self {
                propagator: Box::new(global::text_map_propagator()),
            }
        }
        
        // Inject context into message headers
        pub fn inject_context(&self, context: &Context, headers: &mut HashMap<String, String>) {
            let mut injector = MessageHeaderInjector::new(headers);
            self.propagator.inject_context(context, &mut injector);
        }
        
        // Extract context from message headers
        pub fn extract_context(&self, headers: &HashMap<String, String>) -> Context {
            let extractor = MessageHeaderExtractor::new(headers);
            self.propagator.extract(&extractor)
        }
        
        // Maintain correlation across agent boundaries
        pub fn create_child_context(&self, parent: &Context, span_name: &str) -> Context {
            let tracer = global::tracer("ms-framework");
            let span = tracer
                .span_builder(span_name)
                .with_parent_context(parent.clone())
                .start(&tracer);
            Context::current_with_span(span)
        }
        
        // Preserve causality information
        pub fn add_baggage(&self, context: &Context, key: &str, value: &str) -> Context {
            context.with_baggage(vec![(key.to_string(), value.to_string())])
        }
    }
    
    // Message header injector implementation
    struct MessageHeaderInjector<'a> {
        headers: &'a mut HashMap<String, String>,
    }
    
    impl<'a> MessageHeaderInjector<'a> {
        fn new(headers: &'a mut HashMap<String, String>) -> Self {
            Self { headers }
        }
    }
    
    impl<'a> Injector for MessageHeaderInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            self.headers.insert(key.to_string(), value);
        }
    }
    
    // Message header extractor implementation
    struct MessageHeaderExtractor<'a> {
        headers: &'a HashMap<String, String>,
    }
    
    impl<'a> MessageHeaderExtractor<'a> {
        fn new(headers: &'a HashMap<String, String>) -> Self {
            Self { headers }
        }
    }
    
    impl<'a> Extractor for MessageHeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.headers.get(key).map(|v| v.as_str())
        }
        
        fn keys(&self) -> Vec<&str> {
            self.headers.keys().map(|k| k.as_str()).collect()
        }
    }
    
    // Agent-specific context management
    pub struct AgentContextManager {
        agent_id: String,
        agent_type: String,
        propagator: ContextPropagator,
    }
    
    impl AgentContextManager {
        pub fn new(agent_id: String, agent_type: String) -> Self {
            Self {
                agent_id,
                agent_type,
                propagator: ContextPropagator::new(),
            }
        }
        
        // Create context for outgoing communication
        pub fn create_outbound_context(&self, parent_context: &Context, operation: &str) -> Context {
            let context = self.propagator.create_child_context(
                parent_context,
                &format!("agent.{}.{}", self.agent_type, operation)
            );
            
            // Add agent-specific baggage
            self.propagator.add_baggage(&context, "agent.id", &self.agent_id)
                .add_baggage("agent.type", &self.agent_type)
                .add_baggage("operation", operation)
        }
        
        // Process incoming context
        pub fn process_inbound_context(&self, headers: &HashMap<String, String>) -> Context {
            let context = self.propagator.extract_context(headers);
            
            // Validate context and add local attributes
            let local_context = self.propagator.create_child_context(
                &context,
                &format!("agent.{}.process", self.agent_type)
            );
            
            self.propagator.add_baggage(&local_context, "processing.agent_id", &self.agent_id)
        }
    }
```

#### 2.5 Distributed Tracing Implementation

```rust
PATTERN DistributedTracingImplementation:
    use opentelemetry::{trace::{Span, Tracer, TracerProvider}, Context};
    use opentelemetry::propagation::{Injector, Extractor, TextMapPropagator};
    
    // Message header injection/extraction
    pub struct MessageHeaders(HashMap<String, String>);
    
    impl Injector for MessageHeaders {
        fn set(&mut self, key: &str, value: String) {
            self.0.insert(key.to_string(), value);
        }
    }
    
    impl Extractor for MessageHeaders {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).map(|v| v.as_str())
        }
        
        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|k| k.as_str()).collect()
        }
    }
    
    // Tracing wrapper for agent operations
    pub struct TracedAgent {
        tracer: Box<dyn Tracer + Send + Sync>,
        propagator: Box<dyn TextMapPropagator + Send + Sync>,
    }
    
    impl TracedAgent {
        pub async fn execute_task<F, T>(
            &self,
            task_name: &str,
            task_type: &str,
            task_fn: F,
        ) -> Result<T, Box<dyn std::error::Error>>
        where
            F: FnOnce(Context) -> Future<Output = Result<T, Box<dyn std::error::Error>>>,
        {
            // Create span for task execution
            let mut span = self.tracer
                .span_builder(format!("agent.task.{}", task_name))
                .with_kind(SpanKind::Internal)
                .with_attributes(vec![
                    KeyValue::new("task.name", task_name.to_string()),
                    KeyValue::new("task.type", task_type.to_string()),
                    KeyValue::new("agent.id", self.agent_id.clone()),
                    KeyValue::new("agent.type", self.agent_type.clone()),
                ])
                .start(&self.tracer);
                
            let cx = Context::current_with_span(span);
            
            // Execute task with tracing context
            let start_time = Instant::now();
            match task_fn(cx.clone()).await {
                Ok(result) => {
                    span.set_status(Status::ok());
                    span.set_attribute(KeyValue::new("task.duration_ms", start_time.elapsed().as_millis() as i64));
                    Ok(result)
                }
                Err(e) => {
                    span.record_error(&e);
                    span.set_status(Status::error(e.to_string()));
                    Err(e)
                }
            }
        }
        
        pub async fn send_message(
            &self,
            message: &Message,
            destination: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let span = self.tracer
                .span_builder("agent.message.send")
                .with_kind(SpanKind::Producer)
                .with_attributes(vec![
                    KeyValue::new("messaging.system", "nats"),
                    KeyValue::new("messaging.destination", destination.to_string()),
                    KeyValue::new("messaging.message_id", message.id.clone()),
                    KeyValue::new("messaging.message_type", message.msg_type.clone()),
                ])
                .start(&self.tracer);
                
            let cx = Context::current_with_span(span);
            
            // Inject trace context into message headers
            let mut headers = MessageHeaders(HashMap::new());
            self.propagator.inject_context(&cx, &mut headers);
            
            // Add headers to message
            let mut msg_with_context = message.clone();
            msg_with_context.headers = headers.0;
            
            // Send message
            self.transport.send(destination, &msg_with_context).await
        }
        
        pub async fn receive_message(
            &self,
            message: Message,
        ) -> Result<Context, Box<dyn std::error::Error>> {
            // Extract trace context from message headers
            let headers = MessageHeaders(message.headers.clone());
            let parent_cx = self.propagator.extract(&headers);
            
            // Create span for message processing
            let span = self.tracer
                .span_builder("agent.message.receive")
                .with_kind(SpanKind::Consumer)
                .with_attributes(vec![
                    KeyValue::new("messaging.system", "nats"),
                    KeyValue::new("messaging.message_id", message.id.clone()),
                    KeyValue::new("messaging.message_type", message.msg_type.clone()),
                ])
                .with_parent_context(parent_cx)
                .start(&self.tracer);
                
            Ok(Context::current_with_span(span))
        }
    }
```

### 3. Distributed Tracing Patterns

#### 3.1 Trace Structure Pattern

```rust
PATTERN TraceStructure:
    COMPONENTS:
        trace_identifier
        span_collection
        timing_information
        status_indicators
        attribute_metadata
        event_markers
        relationship_links
```

#### 3.2 Sampling Strategy Pattern

```rust
PATTERN AdaptiveSampling:
    DECISION_FACTORS:
        operation_criticality
        system_load_level
        error_presence
        latency_threshold
        trace_priority
    
    SAMPLING_LOGIC:
        IF critical_operation: ALWAYS_SAMPLE
        ELIF error_detected: ALWAYS_SAMPLE
        ELIF high_load: MINIMAL_SAMPLING
        ELSE: ADAPTIVE_RATE
```

#### 3.3 Cross-Agent Tracing Pattern

```rust
PATTERN CrossAgentTracing:
    create_client_span(source_agent)
    inject_context_headers(message)
    transmit_to_target(message)
    extract_context_on_receive()
    create_server_span(target_agent)
    link_span_hierarchy()
```

#### 3.4 Trace Layers Pattern

```rust
PATTERN TraceLayers:
    CHILD_PROCESS_TRACING:
        spawn_agent_with_piped_output()
        create_tracing_span(agent_id)
        capture_stdout_to_tracing()
        capture_stderr_to_tracing()
        correlate_process_lifecycle()

    CONTEXT_PROPAGATION:
        inject_trace_headers(outgoing_message)
        extract_trace_headers(incoming_message)
        maintain_span_relationships()
        preserve_baggage_items()
```

#### 3.5 Claude-CLI Parallel Agent Tracing Pattern

```rust
PATTERN ClaudeCLIParallelTracing:
    PARALLEL_AGENT_INSTRUMENTATION:
        # Handle multiple internal agents sharing one PID
        create_root_span(claude_cli_process_id)

        FOR each_internal_agent IN parallel_agents:
            create_child_span(internal_agent_id)
            add_span_attribute("agent_internal_id", internal_agent_id)
            add_span_attribute("agent_type", "task_patch_agent")
            add_span_attribute("parallel_group", parallel_execution_id)

        # Parse and correlate output lines
        parse_output_line("Task(Patch Agent <n> ...)")
        correlate_to_span(agent_internal_id)

    LOG_DISAMBIGUATION:
        # Multiple internal agents share PID but need separate traces
        span_attributes:
            agent_internal_id: "unique_identifier_per_internal_agent"
            process_id: "shared_claude_cli_pid"
            parallel_execution_id: "batch_identifier"
            agent_role: "task_agent" | "patch_agent"

    TRACE_CORRELATION:
        # Link parallel agent spans to parent orchestration
        parent_span: "orchestrator_task_span"
        child_spans: ["agent_0_span", "agent_1_span", ..., "agent_n_span"]
        correlation_key: "parallel_execution_batch_id"
```

#### 3.6 Claude-CLI Hook System Tracing Pattern

```rust
PATTERN HookSystemTracing:
    HOOK_EXECUTION_SPANS:
        # Create spans for each hook execution
        create_hook_span(hook_name, hook_type)
        add_span_attribute("hook_name", hook_name)
        add_span_attribute("hook_type", hook_type)  # startup, pre_task, post_task, on_error, on_file_change
        add_span_attribute("hook_outcome", outcome)  # success, failure, timeout
        add_span_attribute("hook_latency_ms", execution_time)
        add_span_attribute("hook_script_path", script_path)

    HOOK_LIFECYCLE_TRACING:
        # Trace complete hook lifecycle
        start_span("hook_execution")
        record_event("hook_started", {"hook_type": hook_type})

        # Execute hook and capture metrics
        execution_start = current_timestamp()
        hook_result = execute_hook_script(hook_payload)
        execution_end = current_timestamp()

        # Record outcome and timing
        add_span_attribute("execution_duration_ms", execution_end - execution_start)
        add_span_attribute("hook_exit_code", hook_result.exit_code)
        add_span_attribute("payload_size_bytes", payload_size)
        add_span_attribute("response_size_bytes", response_size)

        IF hook_result.success:
            record_event("hook_completed", {"mutations": hook_result.mutations})
        ELSE:
            record_event("hook_failed", {"error": hook_result.error})
            add_span_attribute("error_type", hook_result.error_type)

        end_span()

    HOOK_CORRELATION_PATTERN:
        # Link hook spans to triggering task/agent spans
        parent_span: "task_execution_span" | "agent_lifecycle_span"
        hook_span: "hook_execution_span"
        correlation_attributes:
            task_id: "triggering_task_identifier"
            agent_id: "executing_agent_identifier"
            hook_trigger: "pre_task" | "post_task" | "on_error" | "on_file_change"
```

#### 3.5 OTLP Integration Pattern

```rust
PATTERN OTLPConfiguration:
    EXPORTER_SETUP:
        protocol: binary_protobuf | grpc | http_json
        endpoint: collector_address
        compression: gzip | none
        retry_policy: exponential_backoff
    
    COLLECTOR_PIPELINE:
        receivers: [otlp_grpc, otlp_http]
        processors: [batch, memory_limiter]
        exporters: [backend_storage]
    
    PERFORMANCE_CONSIDERATIONS:
        binary_encoding: 2-3x_cpu_efficiency
        batch_processing: reduce_network_calls
        async_export: prevent_blocking
```

### 4. Metrics Collection Patterns

#### 4.1 Metric Types Pattern

```rust
PATTERN MetricTypes:
    CATEGORIES:
        counters: cumulative_values
        gauges: point_in_time_values
        histograms: distribution_values
        summaries: statistical_aggregates
```

#### 4.2 Agent-Specific Metrics Pattern

```rust
PATTERN AgentMetrics:
    COMMON_METRICS:
        operation_count
        operation_duration
        error_rate
        resource_utilization
        queue_depth
    
    SPECIALIZED_METRICS:
        researcher_agent: [sources_searched, relevance_scores]
        coder_agent: [lines_generated, complexity_scores]
        coordinator_agent: [agents_managed, coordination_latency]
```

#### 4.3 System-Wide Metrics Pattern

```rust
PATTERN SystemMetrics:
    CATEGORIES:
        swarm_coordination_metrics
        resource_utilization_metrics
        agent_lifecycle_metrics
        task_completion_metrics
        communication_overhead_metrics
```

#### 4.4 Messaging Backplane Metrics Pattern

```rust
PATTERN BackplaneMetrics:
    CONSUMER_HEALTH:
        consumer_lag_ms
        pending_message_count
        ack_rate_per_second
        reconnection_count
    
    BROKER_PERFORMANCE:
        fsync_latency_p99
        disk_write_queue_depth
        memory_buffer_utilization
        active_connection_count
    
    MESSAGE_FLOW:
        publish_rate_per_topic
        fan_out_multiplier
        message_size_p95
        throughput_mb_per_second
    
    SYSTEM_HEALTH:
        broker_cpu_percentage
        jvm_gc_pause_ms
        network_retransmits
        partition_leader_changes
```

#### 4.5 Critical Threshold Monitoring Pattern

```rust
PATTERN CriticalThresholds:
    MESSAGING_THRESHOLDS:
        jetstream_fsync_lag: 50M_msgs_per_day
        rabbitmq_queue_depth: 10000_messages
        redis_output_buffer: approaching_limit
        kafka_consumer_lag: 1000_msgs_or_30sec
    
    RESPONSE_ACTIONS:
        emit_threshold_alert()
        trigger_backpressure()
        scale_consumer_pool()
        initiate_flow_control()
```

### 5. Logging Patterns

#### 5.0 Log Aggregation Pipeline Implementation

```yaml
PATTERN LogAggregationPipeline:
    # Fluent Bit Configuration for Log Collection
    fluent_bit_config:
        [SERVICE]
            flush        1
            daemon       Off
            log_level    info
            parsers_file parsers.conf
            
        [INPUT]
            Name              tail
            Path              /var/log/mister-smith/agents/*.log
            Parser            json
            Tag               agents.*
            Refresh_Interval  5
            
        [INPUT]
            Name   systemd
            Tag    host.*
            Read_From_Tail On
            
        [FILTER]
            Name         parser
            Match        agents.*
            Key_Name     log
            Parser       json
            Reserve_Data On
            
        [FILTER]
            Name         nest
            Match        agents.*
            Operation    lift
            Nested_under attributes
            
        [FILTER]
            Name         modify
            Match        agents.*
            Add          environment ${ENVIRONMENT}
            Add          cluster ${CLUSTER_NAME}
            
        [OUTPUT]
            Name   otlp
            Match  *
            Host   ${OTLP_ENDPOINT}
            Port   4317
            Logs_uri            /v1/logs
            Log_response_payload True
            Tls                 Off
            Tls_verify          Off
            
        [OUTPUT]
            Name   forward
            Match  agents.*
            Host   ${LOG_AGGREGATOR}
            Port   24224
            Shared_Key ${SHARED_KEY}
            
    # Vector Configuration for Advanced Processing
    vector_config:
        sources:
          agent_logs:
            type: file
            include:
              - /var/log/mister-smith/agents/*.log
            file_key: file
            hostname_key: host
            
        transforms:
          parse_json:
            type: remap
            inputs:
              - agent_logs
            source: |
              . = parse_json!(.message)
              .timestamp = parse_timestamp!(.timestamp, "%+")
              
          enrich_logs:
            type: remap
            inputs:
              - parse_json
            source: |
              .environment = get_env_var!("ENVIRONMENT")
              .cluster = get_env_var!("CLUSTER_NAME")
              .severity_number = to_int(get(., ["severity_number"]) ?? 9)
              
          multiline_aggregation:
            type: reduce
            inputs:
              - enrich_logs
            group_by:
              - trace_id
              - span_id
            merge_strategies:
              message: concat_newline
            expire_after_ms: 5000
            
          filter_errors:
            type: filter
            inputs:
              - multiline_aggregation
            condition: '.level == "error" || .level == "critical"'
            
        sinks:
          otlp:
            type: opentelemetry_logs
            inputs:
              - enrich_logs
            endpoint: "http://${OTLP_ENDPOINT}:4317"
            compression: gzip
            
          elasticsearch:
            type: elasticsearch
            inputs:
              - multiline_aggregation
            endpoint: "https://${ES_ENDPOINT}:9200"
            index: "mister-smith-logs-%Y.%m.%d"
            
          error_alerts:
            type: prometheus_remote_write
            inputs:
              - filter_errors
            endpoint: "http://${PROMETHEUS_ENDPOINT}/api/v1/write"
```

#### 5.1 Structured Logging Pattern

```rust
PATTERN StructuredLogging:
    LOG_ENTRY_STRUCTURE:
        timestamp
        severity_level
        message_content
        trace_correlation
        agent_context
        operation_context
        environment_metadata
```

#### 5.2 Log Level Classification Pattern

```rust
PATTERN LogLevels:
    DEBUG: detailed_diagnostic_info
    INFO: general_operational_events
    WARNING: potential_issue_indicators
    ERROR: failure_notifications
    CRITICAL: system_critical_events
```

#### 5.3 Log Aggregation Pattern

```rust
PATTERN LogAggregation:
    PIPELINE_STAGES:
        collection_from_sources
        parsing_and_structuring
        enrichment_with_context
        correlation_by_identifiers
        aggregation_by_patterns
        storage_and_indexing
```

#### 5.4 Multiline Log Handling Pattern

```rust
PATTERN MultilineLogHandling:
    DETECTION_STRATEGIES:
        prefix_pattern_matching
        continuation_character_detection
        timeout_based_grouping
        structured_format_parsing
    
    PROCESSING_OPTIONS:
        buffer_until_complete()
        apply_max_wait_timeout()
        preserve_original_formatting()
        add_correlation_metadata()
    
    OUTPUT_FORMATS:
        single_log_entry: complete_text_block
        json_lines: structured_per_line
        otlp_format: single_log_record
```

### 6. Multi-Agent Observability Patterns

#### 6.1 Agent State Tracking Pattern

```rust
PATTERN AgentStateObservability:
    STATES:
        initializing
        idle
        processing
        waiting
        error
        terminating
    
    STATE_TRANSITIONS:
        capture_state_change()
        measure_state_duration()
        detect_anomalous_transitions()
        maintain_state_history()
```

#### 6.2 Swarm Coordination Observability Pattern

```rust
PATTERN SwarmObservability:
    track_agent_assignments()
    monitor_task_distribution()
    measure_coordination_latency()
    analyze_communication_patterns()
    detect_coordination_anomalies()
```

#### 6.3 Communication Pattern Analysis

```rust
PATTERN CommunicationAnalysis:
    build_communication_graph()
    track_message_flow()
    measure_communication_latency()
    identify_bottlenecks()
    detect_anomalous_patterns()
```

### 7. Alerting and Anomaly Detection Patterns

#### 7.1 Alert Rule Pattern

```rust
PATTERN AlertRules:
    RULE_COMPONENTS:
        condition_expression
        severity_level
        notification_channels
        remediation_actions
        suppression_windows
```

#### 7.2 Anomaly Detection Pattern

```rust
PATTERN AnomalyDetection:
    DETECTION_METHODS:
        statistical_analysis
        pattern_recognition
        machine_learning_models
        threshold_monitoring
        behavioral_analysis
    
    RESPONSE_ACTIONS:
        emit_alert()
        trigger_investigation()
        initiate_auto_remediation()
        update_baselines()
```

#### 7.3 Prometheus Alert Pattern

```rust
PATTERN PrometheusAlerts:
    ALERT_DEFINITION:
        name: descriptive_alert_name
        expression: promql_query
        duration: evaluation_period
        severity: critical | warning | info
        annotations: detailed_descriptions
    
    COMMON_ALERTS:
        agent_crash_loop:
            expr: rate(restarts_total[5m]) > 1/min
            severity: critical
        
        high_consumer_lag:
            expr: consumer_lag > 1000
            severity: warning
        
        resource_exhaustion:
            expr: memory_percent > 90
            severity: critical
    
    ALERT_ROUTING:
        group_by: [alertname, cluster, agent]
        group_wait: 30s
        group_interval: 5m
        repeat_interval: 12h
```

### 8. Visualization Patterns

#### 8.1 Dashboard Organization Pattern

```rust
PATTERN DashboardStructure:
    DASHBOARD_TYPES:
        system_overview
        agent_performance
        swarm_coordination
        error_analysis
        resource_utilization
    
    VISUALIZATION_TYPES:
        time_series_graphs
        distribution_histograms
        relationship_matrices
        status_indicators
        flow_diagrams
```

#### 8.2 Real-Time Monitoring Pattern

```rust
PATTERN RealTimeMonitoring:
    COMPONENTS:
        live_trace_visualization
        metric_sparklines
        log_streaming
        topology_mapping
        alert_notifications
```

### 9. Data Management Patterns

#### 9.1 Retention Policy Pattern

```rust
PATTERN DataRetention:
    RETENTION_TIERS:
        hot_storage: recent_full_fidelity
        warm_storage: medium_term_sampled
        cold_storage: long_term_compressed
        archive: historical_metadata
    
    TRANSITION_RULES:
        age_based_migration()
        importance_based_retention()
        compliance_driven_archival()
```

#### 9.2 Data Lifecycle Pattern

```rust
PATTERN DataLifecycle:
    ingest_and_validate()
    process_and_enrich()
    store_with_indexing()
    age_and_compress()
    archive_or_delete()
```

### 10. Performance Optimization Patterns

#### 10.0 Performance Profiling Integration

```rust
PATTERN PerformanceProfilingIntegration:
    use pprof::{protos::Message, ProfilerGuard, Report};
    use std::fs::File;
    use tokio::time::{interval, Duration};
    
    pub struct ProfilingManager {
        continuous_profiler: Option<ProfilerGuard<'static>>,
        profile_interval: Duration,
        output_dir: PathBuf,
    }
    
    impl ProfilingManager {
        pub fn new(output_dir: PathBuf) -> Self {
            Self {
                continuous_profiler: None,
                profile_interval: Duration::from_secs(60),
                output_dir,
            }
        }
        
        // Continuous CPU profiling
        pub fn start_continuous_profiling(&mut self) -> Result<(), Box<dyn Error>> {
            let guard = pprof::ProfilerGuardBuilder::default()
                .frequency(1000) // 1000 Hz sampling
                .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                .build()?;
                
            self.continuous_profiler = Some(guard);
            
            // Spawn periodic profile dumps
            let output_dir = self.output_dir.clone();
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(300)); // 5 minutes
                
                loop {
                    interval.tick().await;
                    if let Err(e) = Self::dump_profile(&output_dir).await {
                        error!("Failed to dump profile: {}", e);
                    }
                }
            });
            
            Ok(())
        }
        
        // Memory profiling with jemalloc
        pub fn configure_memory_profiling() {
            // Enable jemalloc profiling
            std::env::set_var("MALLOC_CONF", "prof:true,prof_prefix:jeprof.out,lg_prof_interval:30");
        }
        
        // Trace-based profiling integration
        pub fn create_profiling_span(tracer: &dyn Tracer, operation: &str) -> Span {
            let mut span = tracer
                .span_builder(format!("profile.{}", operation))
                .with_kind(SpanKind::Internal)
                .start(tracer);
                
            // Add profiling metadata
            span.set_attributes(vec![
                KeyValue::new("profile.type", "performance"),
                KeyValue::new("profile.operation", operation.to_string()),
                KeyValue::new("profile.thread_id", format!("{:?}", std::thread::current().id())),
            ]);
            
            span
        }
        
        // Async runtime metrics
        pub fn export_tokio_metrics(meter: &Meter) {
            let runtime_handle = tokio::runtime::Handle::current();
            let runtime_metrics = runtime_handle.metrics();
            
            // Worker thread metrics
            meter
                .u64_observable_gauge("tokio_workers_count")
                .with_description("Number of worker threads")
                .with_callback(move |observer| {
                    observer.observe(runtime_metrics.num_workers() as u64, &[]);
                })
                .init();
                
            // Task metrics
            meter
                .u64_observable_gauge("tokio_active_tasks_count")
                .with_description("Number of active tasks")
                .with_callback(move |observer| {
                    observer.observe(runtime_metrics.active_tasks_count() as u64, &[]);
                })
                .init();
                
            // Queue metrics
            meter
                .u64_observable_gauge("tokio_injection_queue_depth")
                .with_description("Injection queue depth")
                .with_callback(move |observer| {
                    observer.observe(runtime_metrics.injection_queue_depth() as u64, &[]);
                })
                .init();
        }
    }
    
    // Flame graph generation
    pub async fn generate_flame_graph(
        profile_data: Vec<u8>,
        output_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let report = Report::from_protobuf(&profile_data)?;
        
        let mut flame_graph = Vec::new();
        report.flamegraph(&mut flame_graph)?;
        
        std::fs::write(output_path, flame_graph)?;
        Ok(())
    }
```

#### 10.1 Overhead Management Pattern

```rust
PATTERN OverheadOptimization:
    STRATEGIES:
        adaptive_sampling
        batch_processing
        asynchronous_export
        intelligent_filtering
        resource_pooling
```

#### 10.2 Scalability Pattern

```rust
PATTERN ObservabilityScaling:
    HORIZONTAL_SCALING:
        distribute_collection_load()
        shard_storage_backend()
        parallelize_processing()
    
    VERTICAL_SCALING:
        optimize_resource_allocation()
        implement_tiered_storage()
        cache_frequent_queries()
```

### 11. Integration Patterns

#### 11.1 External System Integration Pattern

```rust
PATTERN ExternalIntegration:
    INTEGRATION_POINTS:
        incident_management_systems
        analytics_platforms
        machine_learning_pipelines
        notification_services
        automation_tools
```

#### 11.2 Data Export Pattern

```rust
PATTERN DataExport:
    configure_export_format()
    set_export_schedule()
    filter_export_data()
    transform_for_destination()
    verify_export_completion()
```

### 12. Security and Compliance Patterns

#### 12.1 Observability Security Pattern

```rust
PATTERN ObservabilitySecurity:
    SECURITY_MEASURES:
        encrypt_data_transmission()
        authenticate_access()
        authorize_operations()
        audit_all_access()
        detect_sensitive_data()
        apply_data_masking()
```

#### 12.2 Compliance Pattern

```rust
PATTERN ComplianceMonitoring:
    track_data_access()
    enforce_retention_policies()
    maintain_audit_trails()
    generate_compliance_reports()
    automate_policy_enforcement()
```

### 13. Implementation Considerations

#### 13.1 Framework Selection Criteria

- Language-agnostic instrumentation capabilities
- Support for distributed tracing standards
- Flexible metric collection options
- Structured logging support
- Extensible architecture

#### 13.2 Protocol Performance Considerations

```rust
PATTERN ProtocolSelection:
    ENCODING_EFFICIENCY:
        binary_protobuf: 15_microsec_per_log
        json_encoding: 45_microsec_per_log
        size_ratio: 1:2_binary_vs_json
    
    TRANSPORT_OPTIONS:
        grpc_tonic: minimal_async_overhead
        http_binary: wide_compatibility
        direct_stdout: essentially_free
    
    OPTIMIZATION_STRATEGIES:
        use_non_blocking_appenders()
        batch_telemetry_exports()
        implement_circuit_breakers()
```

#### 13.3 Deployment Considerations

- Minimize instrumentation overhead
- Ensure high availability of monitoring
- Plan for data volume growth
- Implement proper access controls
- Enable disaster recovery
- Consider protocol efficiency for scale

### 14. Best Practices Summary

#### 14.1 Instrumentation Best Practices

- Instrument critical code paths
- Use consistent naming conventions
- Implement proper context propagation
- Apply appropriate sampling rates
- Minimize performance impact

#### 14.2 Data Management Best Practices

- Define clear retention policies
- Implement data compression strategies
- Use appropriate storage tiers
- Regular backup and archival
- Monitor storage growth trends

#### 14.3 Operational Best Practices

- Establish baseline metrics
- Define meaningful alerts
- Regular review of dashboards
- Continuous optimization
- Document monitoring patterns

---

## 15. Concrete Specifications

### 15.1 Prometheus Metrics Schema

#### 15.1.1 Agent Lifecycle Metrics

```prometheus
# HELP agent_spawns_total Number of agent instances spawned
# TYPE agent_spawns_total counter
agent_spawns_total{agent_type="task_agent",spawn_method="subprocess"} 125

# HELP agent_active_count Current number of active agents
# TYPE agent_active_count gauge
agent_active_count{agent_type="task_agent",state="processing"} 8

# HELP agent_duration_seconds Time agents spend in each state
# TYPE agent_duration_seconds histogram
agent_duration_seconds_bucket{agent_type="patch_agent",state="processing",le="1.0"} 45
agent_duration_seconds_bucket{agent_type="patch_agent",state="processing",le="5.0"} 89
agent_duration_seconds_bucket{agent_type="patch_agent",state="processing",le="10.0"} 112
agent_duration_seconds_bucket{agent_type="patch_agent",state="processing",le="+Inf"} 125
agent_duration_seconds_sum{agent_type="patch_agent",state="processing"} 387.5
agent_duration_seconds_count{agent_type="patch_agent",state="processing"} 125

# HELP agent_memory_bytes Memory usage per agent
# TYPE agent_memory_bytes gauge
agent_memory_bytes{agent_id="agent-42",agent_type="analysis_agent"} 524288000

# HELP agent_cpu_percent CPU utilization per agent
# TYPE agent_cpu_percent gauge
agent_cpu_percent{agent_id="agent-42",agent_type="analysis_agent"} 23.5
```

#### 15.1.2 Task Execution Metrics

```prometheus
# HELP task_completions_total Number of completed tasks
# TYPE task_completions_total counter
task_completions_total{task_type="analysis",status="success",agent_type="analysis_agent"} 1247

# HELP task_errors_total Number of failed tasks
# TYPE task_errors_total counter
task_errors_total{task_type="code_generation",error_type="timeout",agent_type="coder_agent"} 23

# HELP task_duration_seconds Task execution duration distribution
# TYPE task_duration_seconds histogram
task_duration_seconds_bucket{task_type="analysis",le="5.0"} 234
task_duration_seconds_bucket{task_type="analysis",le="10.0"} 567
task_duration_seconds_bucket{task_type="analysis",le="30.0"} 890
task_duration_seconds_bucket{task_type="analysis",le="+Inf"} 923
task_duration_seconds_sum{task_type="analysis"} 8934.2
task_duration_seconds_count{task_type="analysis"} 923

# HELP task_queue_depth Current number of pending tasks
# TYPE task_queue_depth gauge
task_queue_depth{queue_type="priority",agent_type="coordinator_agent"} 15
```

#### 15.1.3 Communication Metrics

```prometheus
# HELP messages_sent_total Number of messages sent between agents
# TYPE messages_sent_total counter
messages_sent_total{from_agent_type="coordinator",to_agent_type="worker",message_type="task_assignment"} 1542

# HELP message_latency_seconds Message delivery latency distribution
# TYPE message_latency_seconds histogram
message_latency_seconds_bucket{message_type="task_result",transport="nats",le="0.001"} 1234
message_latency_seconds_bucket{message_type="task_result",transport="nats",le="0.005"} 2345
message_latency_seconds_bucket{message_type="task_result",transport="nats",le="0.01"} 2456
message_latency_seconds_bucket{message_type="task_result",transport="nats",le="+Inf"} 2467
message_latency_seconds_sum{message_type="task_result",transport="nats"} 12.34
message_latency_seconds_count{message_type="task_result",transport="nats"} 2467

# HELP message_queue_depth Number of messages in transport queues
# TYPE message_queue_depth gauge
message_queue_depth{queue_name="task_assignments",broker="nats"} 8

# HELP message_delivery_failures_total Failed message deliveries
# TYPE message_delivery_failures_total counter
message_delivery_failures_total{failure_reason="timeout",transport="nats"} 12
```

#### 15.1.4 Claude-CLI Integration Metrics

```prometheus
# HELP claude_cli_processes_total Number of Claude CLI processes spawned
# TYPE claude_cli_processes_total counter
claude_cli_processes_total{command_type="parallel_agents",batch_size="5"} 45

# HELP claude_cli_parallel_agents_active Current number of parallel internal agents
# TYPE claude_cli_parallel_agents_active gauge
claude_cli_parallel_agents_active{batch_id="batch-123",process_id="12345"} 5

# HELP claude_cli_hook_executions_total Number of hook executions
# TYPE claude_cli_hook_executions_total counter
claude_cli_hook_executions_total{hook_type="post_task",hook_name="validation",status="success"} 234

# HELP claude_cli_hook_duration_seconds Hook execution duration
# TYPE claude_cli_hook_duration_seconds histogram
claude_cli_hook_duration_seconds_bucket{hook_type="pre_task",hook_name="setup",le="0.1"} 45
claude_cli_hook_duration_seconds_bucket{hook_type="pre_task",hook_name="setup",le="0.5"} 67
claude_cli_hook_duration_seconds_bucket{hook_type="pre_task",hook_name="setup",le="1.0"} 78
claude_cli_hook_duration_seconds_bucket{hook_type="pre_task",hook_name="setup",le="+Inf"} 80
claude_cli_hook_duration_seconds_sum{hook_type="pre_task",hook_name="setup"} 23.45
claude_cli_hook_duration_seconds_count{hook_type="pre_task",hook_name="setup"} 80
```

### 15.2 OpenTelemetry Tracing Schema

#### 15.2.1 Agent Execution Trace Structure

```json
{
  "traceID": "4bf92f3577b34da6a3ce929d0e0e4736",
  "spanID": "00f067aa0ba902b7",
  "parentSpanID": "00f067aa0ba90000",
  "operationName": "agent.task_execution",
  "startTime": 1609459200000000,
  "duration": 5000000,
  "tags": {
    "agent.id": "agent-42",
    "agent.type": "analysis_agent",
    "agent.version": "1.0.0",
    "task.id": "task-12345",
    "task.type": "code_analysis",
    "task.priority": "high",
    "component": "mister-smith-agent",
    "span.kind": "internal"
  },
  "process": {
    "serviceName": "mister-smith-framework",
    "tags": {
      "hostname": "agent-node-01",
      "process.pid": "12345",
      "jaeger.version": "1.29.0"
    }
  },
  "logs": [
    {
      "timestamp": 1609459201000000,
      "fields": [
        {"key": "event", "value": "task_started"},
        {"key": "agent.state", "value": "processing"}
      ]
    },
    {
      "timestamp": 1609459205000000,
      "fields": [
        {"key": "event", "value": "task_completed"},
        {"key": "result.status", "value": "success"},
        {"key": "result.items_processed", "value": 47}
      ]
    }
  ]
}
```

#### 15.2.2 Cross-Agent Correlation Pattern

```json
{
  "orchestrator_span": {
    "traceID": "4bf92f3577b34da6a3ce929d0e0e4736",
    "spanID": "00f067aa0ba90000",
    "operationName": "orchestrator.task_distribution",
    "tags": {
      "component": "task_orchestrator",
      "task.batch_id": "batch-123",
      "parallel.agent_count": 5
    }
  },
  "child_spans": [
    {
      "traceID": "4bf92f3577b34da6a3ce929d0e0e4736",
      "spanID": "00f067aa0ba902b7",
      "parentSpanID": "00f067aa0ba90000",
      "operationName": "agent.parallel_execution",
      "tags": {
        "agent.internal_id": "agent-0",
        "parallel.execution_id": "batch-123",
        "agent.role": "task_agent"
      }
    }
  ]
}
```

#### 15.2.3 Hook System Tracing Pattern

```json
{
  "traceID": "4bf92f3577b34da6a3ce929d0e0e4736",
  "spanID": "00f067aa0ba90abc",
  "parentSpanID": "00f067aa0ba902b7",
  "operationName": "hook.execution",
  "tags": {
    "hook.name": "validation_hook",
    "hook.type": "post_task",
    "hook.script_path": "/hooks/validate.sh",
    "hook.trigger": "task_completion"
  },
  "events": [
    {
      "timestamp": 1609459205100000,
      "name": "hook_started",
      "attributes": {
        "payload.size_bytes": 1024
      }
    },
    {
      "timestamp": 1609459205300000,
      "name": "hook_completed",
      "attributes": {
        "hook.exit_code": 0,
        "response.size_bytes": 256,
        "mutations.count": 3
      }
    }
  ]
}
```

### 15.3 Structured Logging Schema

#### 15.3.1 Agent Lifecycle Log Format

```json
{
  "timestamp": "2024-01-01T12:00:00.000Z",
  "level": "INFO",
  "msg": "Agent spawned successfully",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "resource": {
    "service.name": "mister-smith-agent",
    "service.version": "1.0.0",
    "service.instance.id": "agent-node-01"
  },
  "attributes": {
    "agent.id": "agent-42",
    "agent.type": "analysis_agent",
    "agent.state": "initializing",
    "spawn.method": "subprocess",
    "parent.agent_id": "coordinator-01",
    "memory.allocated_mb": 256,
    "cpu.cores_allocated": 2
  }
}
```

#### 15.3.2 Task Execution Log Format

```json
{
  "timestamp": "2024-01-01T12:01:30.500Z",
  "level": "ERROR",
  "msg": "Task execution failed due to timeout",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "resource": {
    "service.name": "mister-smith-agent",
    "service.version": "1.0.0"
  },
  "attributes": {
    "agent.id": "agent-42",
    "task.id": "task-12345",
    "task.type": "code_analysis",
    "task.priority": "high",
    "task.timeout_seconds": 30,
    "task.duration_seconds": 30.1,
    "error.type": "timeout",
    "error.category": "execution",
    "retry.attempt": 2,
    "retry.max_attempts": 3
  },
  "exception": {
    "type": "TaskTimeoutException",
    "message": "Task execution exceeded 30 second timeout",
    "stacktrace": "TaskTimeoutException: Task execution exceeded...\n  at execute_task:142\n  at agent_loop:89"
  }
}
```

#### 15.3.3 Communication Log Format

```json
{
  "timestamp": "2024-01-01T12:02:15.250Z",
  "level": "WARN",
  "msg": "Message delivery retry required",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba90xyz",
  "resource": {
    "service.name": "mister-smith-messaging",
    "service.version": "1.0.0"
  },
  "attributes": {
    "message.id": "msg-67890",
    "message.type": "task_assignment",
    "message.size_bytes": 1024,
    "sender.agent_id": "coordinator-01",
    "receiver.agent_id": "worker-05",
    "transport.broker": "nats",
    "transport.subject": "agents.tasks.assign",
    "delivery.attempt": 2,
    "delivery.latency_ms": 150,
    "failure.reason": "connection_timeout"
  }
}
```

### 15.4 Health Check Endpoints

#### 15.4.0 Health Check Implementation Pattern

```rust
PATTERN HealthCheckImplementation:
    use serde::{Deserialize, Serialize};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    use tokio::time::timeout;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum HealthStatus {
        Healthy,
        Degraded,
        Unhealthy,
        Unknown,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HealthCheckResult {
        pub status: HealthStatus,
        pub timestamp: u64,
        pub uptime_seconds: u64,
        pub version: String,
        pub checks: HashMap<String, ComponentHealth>,
        pub metrics: HealthMetrics,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ComponentHealth {
        pub status: HealthStatus,
        pub last_check: u64,
        pub latency_ms: Option<u64>,
        pub details: Option<String>,
        pub dependencies: Vec<String>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HealthMetrics {
        pub cpu_usage_percent: f64,
        pub memory_usage_mb: u64,
        pub memory_available_mb: u64,
        pub active_agents: u32,
        pub pending_tasks: u32,
        pub message_queue_depth: u32,
    }
    
    pub struct HealthCheckManager {
        start_time: Instant,
        system_start: SystemTime,
        components: HashMap<String, Box<dyn HealthChecker>>,
    }
    
    #[async_trait]
    pub trait HealthChecker: Send + Sync {
        async fn check_health(&self) -> ComponentHealth;
        fn component_name(&self) -> &str;
        fn dependencies(&self) -> Vec<String>;
    }
    
    impl HealthCheckManager {
        pub fn new() -> Self {
            Self {
                start_time: Instant::now(),
                system_start: SystemTime::now(),
                components: HashMap::new(),
            }
        }
        
        pub fn register_checker(&mut self, checker: Box<dyn HealthChecker>) {
            self.components.insert(checker.component_name().to_string(), checker);
        }
        
        pub async fn perform_health_check(&self) -> HealthCheckResult {
            let mut checks = HashMap::new();
            let mut overall_status = HealthStatus::Healthy;
            
            // Check all registered components
            for (name, checker) in &self.components {
                let component_health = timeout(
                    Duration::from_secs(5),
                    checker.check_health()
                ).await.unwrap_or_else(|_| ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    last_check: current_timestamp(),
                    latency_ms: Some(5000),
                    details: Some("Health check timeout".to_string()),
                    dependencies: checker.dependencies(),
                });
                
                // Update overall status based on component status
                match component_health.status {
                    HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                    HealthStatus::Degraded if matches!(overall_status, HealthStatus::Healthy) => {
                        overall_status = HealthStatus::Degraded
                    },
                    _ => {}
                }
                
                checks.insert(name.clone(), component_health);
            }
            
            HealthCheckResult {
                status: overall_status,
                timestamp: current_timestamp(),
                uptime_seconds: self.start_time.elapsed().as_secs(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                checks,
                metrics: self.collect_system_metrics().await,
            }
        }
        
        async fn collect_system_metrics(&self) -> HealthMetrics {
            HealthMetrics {
                cpu_usage_percent: get_cpu_usage().await.unwrap_or(0.0),
                memory_usage_mb: get_memory_usage().await.unwrap_or(0),
                memory_available_mb: get_available_memory().await.unwrap_or(0),
                active_agents: get_active_agent_count().await.unwrap_or(0),
                pending_tasks: get_pending_task_count().await.unwrap_or(0),
                message_queue_depth: get_message_queue_depth().await.unwrap_or(0),
            }
        }
    }
    
    // Component-specific health checkers
    pub struct MessagingHealthChecker {
        nats_client: Arc<async_nats::Client>,
    }
    
    #[async_trait]
    impl HealthChecker for MessagingHealthChecker {
        async fn check_health(&self) -> ComponentHealth {
            let start = Instant::now();
            
            match self.nats_client.connection_state() {
                async_nats::connection::State::Connected => {
                    // Test message round-trip
                    let test_subject = format!("health_check.{}", uuid::Uuid::new_v4());
                    let result = timeout(
                        Duration::from_millis(100),
                        self.nats_client.request(test_subject, "ping".into())
                    ).await;
                    
                    match result {
                        Ok(Ok(_)) => ComponentHealth {
                            status: HealthStatus::Healthy,
                            last_check: current_timestamp(),
                            latency_ms: Some(start.elapsed().as_millis() as u64),
                            details: Some("NATS connection active and responsive".to_string()),
                            dependencies: vec!["nats-server".to_string()],
                        },
                        _ => ComponentHealth {
                            status: HealthStatus::Degraded,
                            last_check: current_timestamp(),
                            latency_ms: Some(start.elapsed().as_millis() as u64),
                            details: Some("NATS connected but slow response".to_string()),
                            dependencies: vec!["nats-server".to_string()],
                        }
                    }
                },
                _ => ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    last_check: current_timestamp(),
                    latency_ms: None,
                    details: Some("NATS connection lost".to_string()),
                    dependencies: vec!["nats-server".to_string()],
                }
            }
        }
        
        fn component_name(&self) -> &str {
            "messaging"
        }
        
        fn dependencies(&self) -> Vec<String> {
            vec!["nats-server".to_string()]
        }
    }
    
    pub struct DatabaseHealthChecker {
        pool: Arc<sqlx::PgPool>,
    }
    
    #[async_trait]
    impl HealthChecker for DatabaseHealthChecker {
        async fn check_health(&self) -> ComponentHealth {
            let start = Instant::now();
            
            match timeout(
                Duration::from_secs(2),
                sqlx::query("SELECT 1").fetch_one(self.pool.as_ref())
            ).await {
                Ok(Ok(_)) => ComponentHealth {
                    status: HealthStatus::Healthy,
                    last_check: current_timestamp(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    details: Some("Database connection active".to_string()),
                    dependencies: vec!["postgresql".to_string()],
                },
                Ok(Err(e)) => ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    last_check: current_timestamp(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    details: Some(format!("Database error: {}", e)),
                    dependencies: vec!["postgresql".to_string()],
                },
                Err(_) => ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    last_check: current_timestamp(),
                    latency_ms: Some(2000),
                    details: Some("Database connection timeout".to_string()),
                    dependencies: vec!["postgresql".to_string()],
                }
            }
        }
        
        fn component_name(&self) -> &str {
            "database"
        }
        
        fn dependencies(&self) -> Vec<String> {
            vec!["postgresql".to_string()]
        }
    }
    
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
```

#### 15.4.1 Basic Health Check

```http
GET /health
Content-Type: application/json

{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00.000Z",
  "uptime_seconds": 3600,
  "version": "1.0.0",
  "checks": {
    "messaging": {
      "status": "healthy",
      "last_check": 1704110400,
      "latency_ms": 15,
      "details": "NATS connection active and responsive",
      "dependencies": ["nats-server"]
    },
    "database": {
      "status": "healthy", 
      "last_check": 1704110400,
      "latency_ms": 8,
      "details": "Database connection active",
      "dependencies": ["postgresql"]
    },
    "agents": {
      "status": "healthy",
      "last_check": 1704110400,
      "latency_ms": 3,
      "details": "All agents responsive",
      "dependencies": []
    }
  },
  "metrics": {
    "cpu_usage_percent": 23.5,
    "memory_usage_mb": 512,
    "memory_available_mb": 2048,
    "active_agents": 8,
    "pending_tasks": 5,
    "message_queue_depth": 12
  }
}
```

#### 15.4.2 Agent-Specific Health Check

```http
GET /health/agent/agent-42
Content-Type: application/json

{
  "agent_id": "agent-42",
  "status": "healthy",
  "state": "processing",
  "uptime_seconds": 1800,
  "current_task": {
    "task_id": "task-12345",
    "task_type": "analysis",
    "started_at": "2024-01-01T11:45:00.000Z",
    "duration_seconds": 900
  },
  "resources": {
    "memory_usage_mb": 245,
    "memory_limit_mb": 512,
    "cpu_usage_percent": 23.5
  },
  "last_heartbeat": "2024-01-01T12:00:00.000Z"
}
```

#### 15.4.3 Readiness Check

```http
GET /health/ready
Content-Type: application/json

{
  "status": "ready",
  "timestamp": "2024-01-01T12:00:00.000Z",
  "ready_for_tasks": true,
  "available_agents": 8,
  "queue_depth": 5,
  "resource_availability": {
    "memory_available_mb": 2048,
    "cpu_available_percent": 65
  }
}
```

### 15.5 Alert Rule Definitions

#### 15.5.0 MS Framework Specific Alert Patterns

```yaml
groups:
- name: ms_framework_critical_alerts
  rules:
  # Agent lifecycle and spawning alerts
  - alert: AgentCrashLoop
    expr: rate(agent_spawns_total[5m]) > 1
    for: 2m
    labels:
      severity: critical
      component: agent_lifecycle
    annotations:
      summary: "Agent {{ $labels.agent_type }} crash loop detected"
      description: "Agent {{ $labels.agent_type }} is restarting more than once per 5 minutes. This indicates a systematic failure in agent initialization or execution."
      runbook_url: "https://docs.ms-framework.org/runbooks/agent-crash-loop"
      dashboard_url: "https://grafana.ms-framework.org/d/agents/agent-performance?var-agent_type={{ $labels.agent_type }}"

  # Task execution monitoring
  - alert: HighTaskErrorRate
    expr: rate(task_errors_total[5m]) / rate(task_completions_total[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
      component: task_execution
    annotations:
      summary: "High task error rate for {{ $labels.task_type }}"
      description: "Task error rate is {{ $value | humanizePercentage }} over the last 5 minutes. Expected error rate < 1%."
      runbook_url: "https://docs.ms-framework.org/runbooks/high-error-rate"

  - alert: TaskExecutionStalled
    expr: rate(task_completions_total[10m]) == 0 and task_queue_depth > 0
    for: 10m
    labels:
      severity: critical
      component: task_execution
    annotations:
      summary: "Task execution completely stalled for {{ $labels.agent_type }}"
      description: "No tasks have completed in 10 minutes despite {{ $value }} tasks in queue"

  # Resource exhaustion
  - alert: AgentMemoryExhaustion
    expr: agent_memory_bytes / (1024*1024*1024) > 1.5
    for: 1m
    labels:
      severity: critical
      component: resource_management
    annotations:
      summary: "Agent {{ $labels.agent_id }} memory exhaustion"
      description: "Agent {{ $labels.agent_id }} is using {{ $value }}GB memory, approaching system limits"
      runbook_url: "https://docs.ms-framework.org/runbooks/memory-exhaustion"

  - alert: SystemMemoryPressure
    expr: (sum(agent_memory_bytes) / (1024*1024*1024)) / (system_memory_total_gb) > 0.85
    for: 5m
    labels:
      severity: critical
      component: system_resources
    annotations:
      summary: "System memory pressure detected"
      description: "Agent memory usage is {{ $value | humanizePercentage }} of total system memory"

  # Communication and messaging
  - alert: MessageQueueBackup
    expr: message_queue_depth > 1000
    for: 5m
    labels:
      severity: critical
      component: messaging
    annotations:
      summary: "Message queue {{ $labels.queue_name }} backup"
      description: "Queue depth is {{ $value }} messages, indicating processing bottleneck"
      runbook_url: "https://docs.ms-framework.org/runbooks/queue-backup"

  - alert: HighMessageLatency
    expr: histogram_quantile(0.95, rate(message_latency_seconds_bucket[5m])) > 5
    for: 5m
    labels:
      severity: critical
      component: messaging
    annotations:
      summary: "High message latency detected"
      description: "95th percentile message latency is {{ $value }}s, expected < 1s"

  # Agent coordination failures
  - alert: AgentCoordinationFailure
    expr: rate(agent_coordination_failures_total[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
      component: coordination
    annotations:
      summary: "Agent coordination failures detected"
      description: "{{ $value }} coordination failures per second, indicating swarm instability"

  - alert: SwarmSplitBrain
    expr: count(group by (cluster_id) (agent_active_count)) > 1
    for: 30s
    labels:
      severity: critical
      component: coordination
    annotations:
      summary: "Multiple active clusters detected (split brain)"
      description: "Detected {{ $value }} separate cluster formations, indicating split brain scenario"

  # Claude CLI integration alerts
  - alert: ClaudeCLIProcessFailures
    expr: rate(claude_cli_processes_failed_total[5m]) > 0.1
    for: 3m
    labels:
      severity: critical
      component: claude_cli
    annotations:
      summary: "Claude CLI process failures detected"
      description: "{{ $value }} Claude CLI process failures per second"

  - alert: HookExecutionFailures
    expr: rate(claude_cli_hook_executions_total{status="failure"}[5m]) > 0.05
    for: 5m
    labels:
      severity: warning
      component: claude_cli
    annotations:
      summary: "High hook execution failure rate"
      description: "Hook failure rate: {{ $value }} failures per second"

  # Performance degradation alerts
  - alert: HighTaskLatencyP99
    expr: histogram_quantile(0.99, rate(task_duration_seconds_bucket[10m])) > 60
    for: 10m
    labels:
      severity: warning
      component: performance
    annotations:
      summary: "High task latency detected"
      description: "99th percentile task latency is {{ $value }}s for {{ $labels.task_type }}"

  - alert: AgentSaturation
    expr: agent_cpu_percent > 90
    for: 10m
    labels:
      severity: warning
      component: performance
    annotations:
      summary: "Agent {{ $labels.agent_id }} CPU saturation"
      description: "Agent CPU usage is {{ $value }}% for 10+ minutes"

#### 15.5.1 Critical Alerts (Prometheus AlertManager)
```yaml
groups:
- name: agent_critical_alerts
  rules:
  - alert: AgentCrashLoop
    expr: rate(agent_spawns_total[5m]) > 1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Agent {{ $labels.agent_type }} crash loop detected"
      description: "Agent {{ $labels.agent_type }} is restarting more than once per 5 minutes"

  - alert: HighTaskErrorRate
    expr: rate(task_errors_total[5m]) / rate(task_completions_total[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High task error rate for {{ $labels.task_type }}"
      description: "Task error rate is {{ $value | humanizePercentage }} over the last 5 minutes"

  - alert: AgentMemoryExhaustion
    expr: agent_memory_bytes / (1024*1024*1024) > 1.5
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Agent {{ $labels.agent_id }} memory exhaustion"
      description: "Agent {{ $labels.agent_id }} is using {{ $value }}GB memory"

  - alert: MessageQueueBackup
    expr: message_queue_depth > 1000
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Message queue {{ $labels.queue_name }} backup"
      description: "Queue depth is {{ $value }} messages"
```

#### 15.5.2 Warning Alerts

```yaml
- name: agent_warning_alerts
  rules:
  - alert: HighTaskLatency
    expr: histogram_quantile(0.95, task_duration_seconds) > 30
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High task latency for {{ $labels.task_type }}"
      description: "95th percentile latency is {{ $value }}s"

  - alert: AgentSaturation
    expr: agent_cpu_percent > 80
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "Agent {{ $labels.agent_id }} high CPU usage"
      description: "CPU usage is {{ $value }}%"

  - alert: CommunicationDelay
    expr: histogram_quantile(0.99, message_latency_seconds) > 5
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High message latency for {{ $labels.message_type }}"
      description: "99th percentile message latency is {{ $value }}s"
```

### 15.6 Dashboard Configurations

#### 15.6.1 System Overview Dashboard (Grafana JSON)

```json
{
  "dashboard": {
    "title": "Mister Smith - System Overview",
    "panels": [
      {
        "title": "Active Agents",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(agent_active_count)",
            "legendFormat": "Total Active Agents"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {"mode": "thresholds"},
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 50},
                {"color": "red", "value": 100}
              ]
            }
          }
        }
      },
      {
        "title": "Task Throughput",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(task_completions_total[5m]) * 60",
            "legendFormat": "Tasks/minute - {{ task_type }}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(task_errors_total[5m]) / rate(task_completions_total[5m]) * 100",
            "legendFormat": "Error % - {{ task_type }}"
          }
        ]
      },
      {
        "title": "Resource Utilization",
        "type": "heatmap",
        "targets": [
          {
            "expr": "agent_cpu_percent",
            "legendFormat": "{{ agent_id }}"
          }
        ]
      }
    ]
  }
}
```

#### 15.6.2 Agent Performance Dashboard Configuration

```json
{
  "dashboard": {
    "title": "Mister Smith - Agent Performance",
    "panels": [
      {
        "title": "Agent Lifecycle Timeline",
        "type": "timeline",
        "targets": [
          {
            "expr": "agent_duration_seconds",
            "legendFormat": "{{ agent_id }} - {{ state }}"
          }
        ]
      },
      {
        "title": "Task Duration Distribution",
        "type": "histogram",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, task_duration_seconds)",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, task_duration_seconds)",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.99, task_duration_seconds)",
            "legendFormat": "99th percentile"
          }
        ]
      },
      {
        "title": "Agent State Distribution",
        "type": "piechart",
        "targets": [
          {
            "expr": "sum by (state) (agent_active_count)",
            "legendFormat": "{{ state }}"
          }
        ]
      }
    ]
  }
}
```

### 15.7 Performance Baselines

#### 15.7.1 Agent Performance Baselines

```yaml
agent_performance_baselines:
  task_completion:
    p95_duration_seconds: 10
    p99_duration_seconds: 30
    success_rate_percent: 99
  
  agent_spawn:
    p95_duration_seconds: 2
    p99_duration_seconds: 5
    success_rate_percent: 99.5
  
  resource_usage:
    memory_baseline_mb: 256
    memory_max_mb: 512
    cpu_baseline_percent: 15
    cpu_max_percent: 80

communication_baselines:
  message_delivery:
    p95_latency_ms: 50
    p99_latency_ms: 100
    delivery_success_rate: 99.9
  
  queue_processing:
    processing_time_ms: 25
    max_queue_depth: 100
  
  context_propagation:
    overhead_ms: 1
    success_rate: 99.99

system_baselines:
  throughput:
    tasks_per_minute: 100
    burst_capacity: 500
  
  utilization:
    optimal_range_percent: [70, 85]
    max_sustainable_percent: 90
  
  error_rates:
    baseline_error_rate: 0.01
    alert_threshold: 0.05
    critical_threshold: 0.10
```

### 15.8 OTLP Configuration

#### 15.8.0 Prometheus Exporter Implementation

```rust
PATTERN PrometheusExporterImplementation:
    use prometheus::{{
        Encoder, TextEncoder, Registry,
        core::{Collector, Desc, Opts},
        proto::MetricFamily,
    }};
    use hyper::{Body, Request, Response, Server, StatusCode};
    use std::sync::Arc;
    
    pub struct MetricsExporter {
        registry: Registry,
        agent_collector: AgentMetricsCollector,
        system_collector: SystemMetricsCollector,
    }
    
    impl MetricsExporter {
        pub fn new() -> Result<Self, Box<dyn Error>> {
            let registry = Registry::new();
            
            // Register custom collectors
            let agent_collector = AgentMetricsCollector::new()?;
            let system_collector = SystemMetricsCollector::new()?;
            
            registry.register(Box::new(agent_collector.clone()))?;
            registry.register(Box::new(system_collector.clone()))?;
            
            Ok(Self {
                registry,
                agent_collector,
                system_collector,
            })
        }
        
        // HTTP endpoint handler
        pub async fn serve_metrics(self: Arc<Self>) -> Result<(), Box<dyn Error>> {
            let addr = ([0, 0, 0, 0], 9090).into();
            
            let make_service = hyper::service::make_service_fn(move |_| {
                let exporter = self.clone();
                async move {
                    Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                        let exporter = exporter.clone();
                        async move { exporter.handle_request(req).await }
                    }))
                }
            });
            
            let server = Server::bind(&addr).serve(make_service);
            info!("Prometheus metrics available at http://{}/metrics", addr);
            
            server.await?;
            Ok(())
        }
        
        async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
            match req.uri().path() {
                "/metrics" => {
                    let encoder = TextEncoder::new();
                    let metric_families = self.registry.gather();
                    let mut buffer = Vec::new();
                    
                    encoder.encode(&metric_families, &mut buffer).unwrap();
                    
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", encoder.format_type())
                        .body(Body::from(buffer))
                        .unwrap())
                }
                "/health" => {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("{\"status\":\"healthy\"}\n"))
                        .unwrap())
                }
                _ => {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("404 Not Found\n"))
                        .unwrap())
                }
            }
        }
    }
    
    // Custom collector for agent-specific metrics
    #[derive(Clone)]
    struct AgentMetricsCollector {
        agent_states: Arc<RwLock<HashMap<String, AgentState>>>,
        descs: Vec<Desc>,
    }
    
    impl Collector for AgentMetricsCollector {
        fn desc(&self) -> Vec<&Desc> {
            self.descs.iter().collect()
        }
        
        fn collect(&self) -> Vec<MetricFamily> {
            let mut families = Vec::new();
            
            // Collect agent state metrics
            let states = self.agent_states.read().unwrap();
            let mut state_counts: HashMap<String, i64> = HashMap::new();
            
            for (_, state) in states.iter() {
                *state_counts.entry(state.to_string()).or_insert(0) += 1;
            }
            
            // Create gauge metric family
            let mut agent_state_family = MetricFamily::new();
            agent_state_family.set_name("agent_state_count".to_string());
            agent_state_family.set_help("Number of agents in each state".to_string());
            agent_state_family.set_field_type(prometheus::proto::MetricType::GAUGE);
            
            for (state, count) in state_counts {
                let mut metric = prometheus::proto::Metric::new();
                let mut gauge = prometheus::proto::Gauge::new();
                gauge.set_value(count as f64);
                metric.set_gauge(gauge);
                
                let mut label = prometheus::proto::LabelPair::new();
                label.set_name("state".to_string());
                label.set_value(state);
                metric.set_label(vec![label].into());
                
                agent_state_family.mut_metric().push(metric);
            }
            
            families.push(agent_state_family);
            families
        }
    }
```

#### 15.8.1 OpenTelemetry Collector Configuration

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    send_batch_size: 1024
    timeout: 1s
  
  memory_limiter:
    limit_mib: 512
  
  resource:
    attributes:
      - key: service.name
        value: mister-smith-framework
        action: upsert

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
    namespace: mister_smith
  
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true
  
  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [jaeger, logging]
    
    metrics:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [prometheus, logging]
    
    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [logging]
```

### 16. Agent-Specific Instrumentation Patterns

#### 16.1 Research Agent Instrumentation

```rust
PATTERN ResearchAgentInstrumentation:
    use opentelemetry::{
        metrics::{Counter, Histogram, Gauge},
        trace::{Tracer, Span},
        KeyValue,
    };
    
    pub struct ResearchAgentMetrics {
        // Research-specific counters
        pub sources_searched: Counter<u64>,
        pub documents_processed: Counter<u64>,
        pub citations_found: Counter<u64>,
        pub queries_executed: Counter<u64>,
        
        // Quality metrics
        pub relevance_scores: Histogram<f64>,
        pub confidence_scores: Histogram<f64>,
        pub search_depth_levels: Histogram<u64>,
        
        // Performance metrics
        pub search_duration: Histogram<f64>,
        pub processing_time_per_document: Histogram<f64>,
        pub memory_usage_per_search: Gauge<f64>,
    }
    
    impl ResearchAgentMetrics {
        pub fn record_search_operation(
            &self,
            query: &str,
            sources_count: u64,
            duration: f64,
            relevance_score: f64,
        ) {
            let labels = &[
                KeyValue::new("query_type", classify_query(query)),
                KeyValue::new("search_method", "semantic"),
            ];
            
            self.sources_searched.add(sources_count, labels);
            self.search_duration.record(duration, labels);
            self.relevance_scores.record(relevance_score, labels);
        }
        
        pub fn record_document_processing(
            &self,
            document_type: &str,
            processing_time: f64,
            extracted_citations: u64,
        ) {
            let labels = &[
                KeyValue::new("document_type", document_type),
                KeyValue::new("processing_method", "nlp_extraction"),
            ];
            
            self.documents_processed.add(1, labels);
            self.processing_time_per_document.record(processing_time, labels);
            self.citations_found.add(extracted_citations, labels);
        }
    }
```

#### 16.2 Coder Agent Instrumentation

```rust
PATTERN CoderAgentInstrumentation:
    pub struct CoderAgentMetrics {
        // Code generation metrics
        pub lines_generated: Counter<u64>,
        pub functions_created: Counter<u64>,
        pub tests_written: Counter<u64>,
        pub bugs_fixed: Counter<u64>,
        
        // Quality metrics
        pub code_complexity_scores: Histogram<f64>,
        pub test_coverage_percentage: Histogram<f64>,
        pub compilation_success_rate: Histogram<f64>,
        
        // Performance metrics
        pub generation_time_per_function: Histogram<f64>,
        pub refactoring_time: Histogram<f64>,
        pub context_processing_time: Histogram<f64>,
    }
    
    impl CoderAgentMetrics {
        pub fn record_code_generation(
            &self,
            language: &str,
            lines_count: u64,
            complexity_score: f64,
            generation_time: f64,
        ) {
            let labels = &[
                KeyValue::new("language", language),
                KeyValue::new("generation_type", "function"),
            ];
            
            self.lines_generated.add(lines_count, labels);
            self.functions_created.add(1, labels);
            self.code_complexity_scores.record(complexity_score, labels);
            self.generation_time_per_function.record(generation_time, labels);
        }
        
        pub fn record_test_creation(
            &self,
            test_type: &str,
            coverage_percentage: f64,
            creation_time: f64,
        ) {
            let labels = &[
                KeyValue::new("test_type", test_type),
                KeyValue::new("framework", "unknown"),
            ];
            
            self.tests_written.add(1, labels);
            self.test_coverage_percentage.record(coverage_percentage, labels);
        }
    }
```

#### 16.3 Coordinator Agent Instrumentation

```rust
PATTERN CoordinatorAgentInstrumentation:
    pub struct CoordinatorAgentMetrics {
        // Coordination metrics
        pub agents_managed: Gauge<u64>,
        pub tasks_distributed: Counter<u64>,
        pub coordination_messages: Counter<u64>,
        pub load_balancing_decisions: Counter<u64>,
        
        // Performance metrics
        pub coordination_latency: Histogram<f64>,
        pub task_assignment_time: Histogram<f64>,
        pub agent_response_time: Histogram<f64>,
        
        // Efficiency metrics
        pub resource_utilization_efficiency: Histogram<f64>,
        pub task_completion_rate: Histogram<f64>,
        pub agent_idle_time: Histogram<f64>,
    }
    
    impl CoordinatorAgentMetrics {
        pub fn record_task_distribution(
            &self,
            task_type: &str,
            agent_count: u64,
            distribution_time: f64,
        ) {
            let labels = &[
                KeyValue::new("task_type", task_type),
                KeyValue::new("distribution_strategy", "load_balanced"),
            ];
            
            self.tasks_distributed.add(1, labels);
            self.task_assignment_time.record(distribution_time, labels);
            self.coordination_messages.add(agent_count, labels);
        }
        
        pub fn record_coordination_decision(
            &self,
            decision_type: &str,
            latency: f64,
            efficiency_score: f64,
        ) {
            let labels = &[
                KeyValue::new("decision_type", decision_type),
                KeyValue::new("coordination_algorithm", "consensus"),
            ];
            
            self.coordination_latency.record(latency, labels);
            self.resource_utilization_efficiency.record(efficiency_score, labels);
            self.load_balancing_decisions.add(1, labels);
        }
    }
```

#### 16.4 Comprehensive Agent Performance Monitoring

```rust
PATTERN ComprehensiveAgentMonitoring:
    use tokio::time::{interval, Duration};
    use sysinfo::{System, SystemExt, ProcessExt};
    
    pub struct AgentPerformanceMonitor {
        agent_id: String,
        agent_type: String,
        system: System,
        metrics: AgentMetrics,
        tracer: Box<dyn Tracer + Send + Sync>,
    }
    
    impl AgentPerformanceMonitor {
        pub fn new(agent_id: String, agent_type: String) -> Self {
            Self {
                agent_id,
                agent_type,
                system: System::new_all(),
                metrics: AgentMetrics::new(),
                tracer: global::tracer("agent-performance"),
            }
        }
        
        // Continuous performance monitoring
        pub async fn start_monitoring(&mut self) {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                self.collect_performance_metrics().await;
            }
        }
        
        async fn collect_performance_metrics(&mut self) {
            let span = self.tracer
                .span_builder("performance.collection")
                .with_attributes(vec![
                    KeyValue::new("agent.id", self.agent_id.clone()),
                    KeyValue::new("agent.type", self.agent_type.clone()),
                ])
                .start(&self.tracer);
                
            let _guard = span.enter();
            
            // Update system information
            self.system.refresh_all();
            
            // CPU metrics
            let cpu_usage = self.get_process_cpu_usage();
            self.metrics.cpu_usage_percent.record(cpu_usage, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
            
            // Memory metrics
            let memory_usage = self.get_process_memory_usage();
            self.metrics.memory_usage_bytes.record(memory_usage as f64, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
            
            // Network metrics
            let network_stats = self.collect_network_metrics().await;
            self.record_network_metrics(network_stats);
            
            // Agent-specific metrics
            self.collect_agent_specific_metrics().await;
            
            span.add_event("performance_metrics_collected", vec![
                KeyValue::new("cpu_usage", cpu_usage),
                KeyValue::new("memory_mb", memory_usage / (1024 * 1024)),
            ]);
        }
        
        fn get_process_cpu_usage(&self) -> f64 {
            let pid = std::process::id();
            self.system
                .process(sysinfo::Pid::from(pid as usize))
                .map(|process| process.cpu_usage() as f64)
                .unwrap_or(0.0)
        }
        
        fn get_process_memory_usage(&self) -> u64 {
            let pid = std::process::id();
            self.system
                .process(sysinfo::Pid::from(pid as usize))
                .map(|process| process.memory())
                .unwrap_or(0)
        }
        
        fn get_network_bytes_sent() -> Result<u64, std::io::Error> {
            // Read network statistics from /proc/net/dev
            let content = std::fs::read_to_string("/proc/net/dev")?;
            let mut total_bytes = 0;
            
            for line in content.lines().skip(2) { // Skip header lines
                if let Some(interface_stats) = line.split_whitespace().nth(9) {
                    if let Ok(bytes) = interface_stats.parse::<u64>() {
                        total_bytes += bytes;
                    }
                }
            }
            Ok(total_bytes)
        }
        
        fn get_network_bytes_received() -> Result<u64, std::io::Error> {
            let content = std::fs::read_to_string("/proc/net/dev")?;
            let mut total_bytes = 0;
            
            for line in content.lines().skip(2) {
                if let Some(interface_stats) = line.split_whitespace().nth(1) {
                    if let Ok(bytes) = interface_stats.parse::<u64>() {
                        total_bytes += bytes;
                    }
                }
            }
            Ok(total_bytes)
        }
        
        fn get_network_packets_sent() -> Result<u64, std::io::Error> {
            let content = std::fs::read_to_string("/proc/net/dev")?;
            let mut total_packets = 0;
            
            for line in content.lines().skip(2) {
                if let Some(interface_stats) = line.split_whitespace().nth(10) {
                    if let Ok(packets) = interface_stats.parse::<u64>() {
                        total_packets += packets;
                    }
                }
            }
            Ok(total_packets)
        }
        
        fn get_network_packets_received() -> Result<u64, std::io::Error> {
            let content = std::fs::read_to_string("/proc/net/dev")?;
            let mut total_packets = 0;
            
            for line in content.lines().skip(2) {
                if let Some(interface_stats) = line.split_whitespace().nth(2) {
                    if let Ok(packets) = interface_stats.parse::<u64>() {
                        total_packets += packets;
                    }
                }
            }
            Ok(total_packets)
        }
        
        fn get_active_connections() -> Result<u64, std::io::Error> {
            // Count active TCP connections from /proc/net/tcp
            let content = std::fs::read_to_string("/proc/net/tcp")?;
            let connection_count = content.lines()
                .skip(1) // Skip header
                .filter(|line| {
                    // Filter for established connections (state 01)
                    line.split_whitespace().nth(3).map_or(false, |state| state == "01")
                })
                .count();
            Ok(connection_count as u64)
        }
        
        async fn collect_network_metrics(&self) -> NetworkStats {
            // Collect network statistics specific to agent communication
            NetworkStats {
                bytes_sent: Self::get_network_bytes_sent().unwrap_or(0),
                bytes_received: Self::get_network_bytes_received().unwrap_or(0),
                packets_sent: Self::get_network_packets_sent().unwrap_or(0),
                packets_received: Self::get_network_packets_received().unwrap_or(0),
                connection_count: Self::get_active_connections().unwrap_or(0),
            }
        }
        
        async fn collect_agent_specific_metrics(&self) {
            match self.agent_type.as_str() {
                "research_agent" => self.collect_research_metrics().await,
                "coder_agent" => self.collect_coding_metrics().await,
                "coordinator_agent" => self.collect_coordination_metrics().await,
                _ => self.collect_generic_metrics().await,
            }
        }
        
        async fn collect_research_metrics(&self) {
            // Research agent specific performance metrics
            let active_searches = self.count_active_searches().await;
            self.metrics.active_searches.record(active_searches as f64, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
        }
        
        async fn collect_coding_metrics(&self) {
            // Coding agent specific performance metrics
            let active_compilations = self.count_active_compilations().await;
            self.metrics.active_compilations.record(active_compilations as f64, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
        }
        
        async fn collect_coordination_metrics(&self) {
            // Coordinator agent specific performance metrics
            let managed_agents = self.count_managed_agents().await;
            self.metrics.managed_agents.record(managed_agents as f64, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
        }
        
        async fn collect_generic_metrics(&self) {
            // Generic agent performance metrics
            let queue_depth = self.get_task_queue_depth().await;
            self.metrics.task_queue_depth.record(queue_depth as f64, &[
                KeyValue::new("agent.id", self.agent_id.clone())
            ]);
        }
    }
    
    #[derive(Debug)]
    struct NetworkStats {
        bytes_sent: u64,
        bytes_received: u64,
        packets_sent: u64,
        packets_received: u64,
        connection_count: u32,
    }
```

### 17. Deployment Configuration Specifications

#### 17.1 Complete Docker Compose Observability Stack

```yaml
PATTERN ObservabilityStack:
    version: '3.8'
    
    services:
      # OpenTelemetry Collector
      otel-collector:
        image: otel/opentelemetry-collector-contrib:0.89.0
        container_name: ms-framework-otel-collector
        command: ["--config=/etc/otel-collector-config.yaml"]
        volumes:
          - ./config/otel-collector-config.yaml:/etc/otel-collector-config.yaml
        ports:
          - "4317:4317"   # OTLP gRPC receiver
          - "4318:4318"   # OTLP HTTP receiver
          - "8889:8889"   # Prometheus metrics
        depends_on:
          - prometheus
          - jaeger
          - elasticsearch
        environment:
          - PROMETHEUS_ENDPOINT=prometheus:9090
          - JAEGER_ENDPOINT=jaeger:14250
          - ELASTICSEARCH_ENDPOINT=elasticsearch:9200
        networks:
          - ms-framework
          
      # Prometheus for metrics storage
      prometheus:
        image: prom/prometheus:v2.47.0
        container_name: ms-framework-prometheus
        command:
          - '--config.file=/etc/prometheus/prometheus.yml'
          - '--storage.tsdb.path=/prometheus'
          - '--web.console.libraries=/etc/prometheus/console_libraries'
          - '--web.console.templates=/etc/prometheus/consoles'
          - '--storage.tsdb.retention.time=30d'
          - '--web.enable-lifecycle'
        volumes:
          - ./config/prometheus.yml:/etc/prometheus/prometheus.yml
          - ./rules:/etc/prometheus/rules
          - prometheus_data:/prometheus
        ports:
          - "9090:9090"
        networks:
          - ms-framework
          
      # Grafana for visualization
      grafana:
        image: grafana/grafana:10.2.0
        container_name: ms-framework-grafana
        environment:
          - GF_SECURITY_ADMIN_PASSWORD=admin
          - GF_USERS_ALLOW_SIGN_UP=false
        volumes:
          - grafana_data:/var/lib/grafana
          - ./config/grafana/dashboards:/etc/grafana/provisioning/dashboards
          - ./config/grafana/datasources:/etc/grafana/provisioning/datasources
        ports:
          - "3000:3000"
        networks:
          - ms-framework
          
      # Jaeger for distributed tracing
      jaeger:
        image: jaegertracing/all-in-one:1.50
        container_name: ms-framework-jaeger
        environment:
          - COLLECTOR_OTLP_ENABLED=true
        ports:
          - "16686:16686"  # Jaeger UI
          - "14250:14250"  # OTLP gRPC
        networks:
          - ms-framework
          
      # Elasticsearch for logs
      elasticsearch:
        image: elasticsearch:8.11.0
        container_name: ms-framework-elasticsearch
        environment:
          - discovery.type=single-node
          - xpack.security.enabled=false
          - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
        volumes:
          - elasticsearch_data:/usr/share/elasticsearch/data
        ports:
          - "9200:9200"
        networks:
          - ms-framework
          
      # Kibana for log visualization
      kibana:
        image: kibana:8.11.0
        container_name: ms-framework-kibana
        environment:
          - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
        ports:
          - "5601:5601"
        depends_on:
          - elasticsearch
        networks:
          - ms-framework
          
      # AlertManager for alert handling
      alertmanager:
        image: prom/alertmanager:v0.26.0
        container_name: ms-framework-alertmanager
        volumes:
          - ./config/alertmanager.yml:/etc/alertmanager/alertmanager.yml
        ports:
          - "9093:9093"
        networks:
          - ms-framework
    
    volumes:
      prometheus_data:
      grafana_data:
      elasticsearch_data:
    
    networks:
      ms-framework:
        driver: bridge
```

#### 17.2 Monitoring Implementation Guidelines

```yaml
PATTERN MonitoringGuidelines:
    monitoring_principles:
      - instrument_critical_paths_only
      - use_consistent_naming_conventions
      - implement_proper_sampling_strategies
      - minimize_performance_overhead
      - ensure_high_availability_monitoring
      
    metric_naming_conventions:
      counters: "{component}_{action}_total"
      gauges: "{component}_{resource}_current"
      histograms: "{component}_{operation}_duration_seconds"
      
    label_conventions:
      required_labels:
        - service_name
        - service_version
        - environment
        - cluster
      optional_labels:
        - agent_type
        - task_type
        - region
        
    retention_policies:
      raw_metrics: "7d"
      downsampled_5m: "30d"
      downsampled_1h: "1y"
      
    alerting_guidelines:
      critical_alerts:
        - response_time: "immediate"
        - escalation: "15_minutes"
        - channels: ["pagerduty", "slack_critical"]
      warning_alerts:
        - response_time: "1_hour"
        - escalation: "24_hours"
        - channels: ["slack_warnings", "email"]
```

### 18. Security Configuration Specifications

#### 18.1 mTLS Configuration for Production

```yaml
# mtls-security.yaml - Production security configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: observability-mtls-config
  namespace: monitoring
data:
  prometheus-tls.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
    
    # mTLS configuration for secure scraping
    scrape_configs:
      - job_name: 'mister-smith-agents-secure'
        scheme: https
        tls_config:
          ca_file: /etc/prometheus/certs/ca.crt
          cert_file: /etc/prometheus/certs/client.crt
          key_file: /etc/prometheus/certs/client.key
          insecure_skip_verify: false
        basic_auth:
          username_file: /etc/prometheus/secrets/username
          password_file: /etc/prometheus/secrets/password
  
  otel-collector-mtls.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
            tls:
              cert_file: /etc/otel/certs/server.crt
              key_file: /etc/otel/certs/server.key
              ca_file: /etc/otel/certs/ca.crt
              client_ca_file: /etc/otel/certs/ca.crt
              min_version: "1.3"
              ciphers:
                - TLS_AES_128_GCM_SHA256
                - TLS_AES_256_GCM_SHA384
                - TLS_CHACHA20_POLY1305_SHA256
```

#### 18.2 Data Retention Compliance Patterns

```yaml
# compliance-retention.yaml - GDPR/SOX compliant retention
PATTERN ComplianceRetention:
    gdpr_compliance:
      personal_data_retention:
        logs_with_pii: "30d"  # Minimum necessary
        anonymized_logs: "2y"  # After PII removal
        deletion_policy: "automated_purge"
        
      pii_detection_rules:
        - pattern: "email_addresses"
          action: "hash_or_remove"
        - pattern: "ip_addresses"
          action: "anonymize_last_octet"
        - pattern: "user_identifiers"
          action: "pseudonymize"
    
    sox_compliance:
      financial_transaction_logs:
        retention_period: "7y"
        immutability: "write_once_read_many"
        audit_trail: "cryptographic_hashing"
        
    retention_implementation:
      elasticsearch:
        ilm_policy: |
          {
            "policy": {
              "phases": {
                "hot": {
                  "actions": {
                    "rollover": {
                      "max_age": "7d",
                      "max_size": "50GB"
                    }
                  }
                },
                "warm": {
                  "min_age": "7d",
                  "actions": {
                    "shrink": {
                      "number_of_shards": 1
                    },
                    "forcemerge": {
                      "max_num_segments": 1
                    }
                  }
                },
                "cold": {
                  "min_age": "30d",
                  "actions": {
                    "allocate": {
                      "require": {
                        "box_type": "cold"
                      }
                    }
                  }
                },
                "delete": {
                  "min_age": "90d",
                  "actions": {
                    "delete": {}
                  }
                }
              }
            }
          }
```

#### 18.3 Cost Monitoring and Optimization

```yaml
# cost-optimization.yaml - Monitor and reduce observability costs
PATTERN CostOptimization:
    metrics_optimization:
      cardinality_control:
        - enforce_label_limits: 10_per_metric
        - drop_high_cardinality_metrics: true
        - aggregate_before_storage: true
        
      sampling_strategies:
        traces:
          error_traces: "100%"  # Keep all errors
          success_traces: "1%"  # Sample successful operations
          latency_based: "p99"  # Keep slow traces
          
        logs:
          error_level: "100%"
          info_level: "10%"
          debug_level: "0.1%"
          
    storage_optimization:
      prometheus:
        retention_by_importance:
          critical_metrics: "90d"
          standard_metrics: "30d"
          debug_metrics: "7d"
          
      elasticsearch:
        compression: "best_compression"
        replica_count: 
          hot_data: 2
          warm_data: 1
          cold_data: 0
          
    cost_monitoring_queries:
      prometheus_cardinality: |
        topk(10, count by (__name__)({__name__=~".+"}))
        
      storage_growth_rate: |
        rate(prometheus_tsdb_symbol_table_size_bytes[1d])
```

#### 18.4 Disaster Recovery Procedures

```yaml
# disaster-recovery.yaml - Backup and recovery for observability
PATTERN DisasterRecovery:
    backup_strategy:
      prometheus:
        snapshot_schedule: "0 2 * * *"  # Daily at 2 AM
        retention_count: 7
        offsite_storage: "s3://backup-bucket/prometheus/"
        
      elasticsearch:
        snapshot_repository:
          type: "s3"
          settings:
            bucket: "observability-backups"
            region: "us-west-2"
            compress: true
            
        snapshot_policy:
          schedule: "0 30 1 * * ?"  # 1:30 AM daily
          retention:
            expire_after: "30d"
            min_count: 7
            max_count: 30
            
      grafana:
        database_backup: "pg_dump"
        dashboard_export: "api_based"
        configuration_backup: "git_versioned"
        
    recovery_procedures:
      prometheus_recovery:
        - stop_prometheus_instance()
        - restore_from_snapshot()
        - verify_data_integrity()
        - restart_with_validation()
        
      elasticsearch_recovery:
        - close_indices()
        - restore_snapshot()
        - open_indices()
        - verify_cluster_health()
        
      rto_targets:
        critical_metrics: "15_minutes"
        standard_metrics: "1_hour"
        historical_data: "4_hours"
        
    validation_procedures:
      data_integrity_checks:
        - verify_metric_continuity()
        - validate_trace_completeness()
        - check_log_sequence_gaps()
        
      functional_validation:
        - test_query_performance()
        - verify_alert_firing()
        - validate_dashboard_loading()
```

---

## Implementation Summary

This framework provides technical patterns for observability and monitoring in multi-agent systems. Implementation follows these key principles:

**Performance Considerations**:

- Minimal overhead instrumentation (< 5% performance impact)
- Efficient sampling strategies for high-throughput scenarios
- Async export patterns to prevent blocking operations
- Resource-aware metric collection

**Integration Points**:

- Agent lifecycle hooks for automatic instrumentation
- Message transport integration for distributed tracing
- Supervision tree integration for error correlation
- Configuration management for dynamic observability settings

**Deployment Requirements**:

- OpenTelemetry Collector for centralized processing
- Prometheus for metrics storage and querying
- Jaeger for trace storage and analysis
- Grafana for visualization and alerting

The patterns in this document enable comprehensive observability without compromising system performance or complexity.

## Quick Reference

### Key Patterns

- **AgentInstrumentation**: Initialize telemetry context and configure exporters
- **ObservabilityPipeline**: Data flow from agent to visualization
- **DistributedTracing**: Cross-agent trace correlation and context propagation
- **MetricsCollection**: Agent-specific and system-wide metrics
- **StructuredLogging**: Correlated logs with trace context

### Essential Integrations

- **OpenTelemetry SDK**: `opentelemetry = "0.20"` for Rust agents
- **OTLP Export**: gRPC endpoint on port 4317 for trace/metrics export
- **Prometheus**: `/metrics` endpoint for scraping agent metrics
- **Context Propagation**: HTTP/NATS headers for distributed tracing

### Configuration Files

- `otel-collector-config.yaml`: Central collector configuration
- `prometheus.yml`: Metrics scraping configuration
- `grafana/dashboards/`: Dashboard definitions for visualization
- `alertmanager.yml`: Alert routing and notification rules
