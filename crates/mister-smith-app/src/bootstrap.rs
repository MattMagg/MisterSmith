//! Deterministic process bootstrap sequence.
//!
//! Initializes all framework subsystems in a fixed order with timeout
//! enforcement and fail-fast behavior for external services.
//!
//! Startup sequence (per contracts/process-lifecycle.md):
//! 1. Config validation (done before bootstrap)
//! 2. Observability init (done before bootstrap)
//! 3. EventBus + monitoring infrastructure
//! 4. NATS connection (with timeout, fail-fast)
//! 5. Supervision tree
//! 6. Agent registry
//! 7. Start background monitors
//! 8. Start HTTP server (health probes)
//! 9. Set state to Ready

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mister_smith_agents::AgentRegistry;
use mister_smith_config::FrameworkConfig;
use mister_smith_core::ProcessLifecycle;
use mister_smith_events::EventBus;
use mister_smith_monitoring::{HealthMonitor, MetricsCollector};
use mister_smith_nats::{NatsTransport, NatsTransportConfig};
use mister_smith_supervision::SupervisedSystem;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::autonomy;
use crate::conversation::ConversationRuntimeService;
use crate::execution::RuntimeTaskService;
use crate::observability;
use crate::observability::ObservabilityGuard;
use crate::ProcessStateTracker;

/// JSON response for Kubernetes health probes.
#[derive(serde::Serialize)]
struct HealthProbeResponse {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// Holds references to all initialized subsystems.
///
/// Passed to the shutdown sequence for orderly teardown in reverse order.
#[allow(dead_code)]
pub struct BootstrapContext {
    pub event_bus: Arc<EventBus>,
    pub health_monitor: Arc<HealthMonitor>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub supervised_system: Arc<SupervisedSystem>,
    pub supervision_handle: Option<tokio::task::JoinHandle<()>>,
    pub agent_registry: Arc<AgentRegistry>,
    pub nats_transport: Option<Arc<NatsTransport>>,
    /// Broadcast sender — dropping or sending signals HTTP server shutdown.
    pub shutdown_tx: broadcast::Sender<()>,
    pub http_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown flag for cooperative cancellation of monitoring loops.
    pub shutdown_flag: Arc<AtomicBool>,
    pub monitor_handle: Option<tokio::task::JoinHandle<()>>,
    pub metrics_handle: Option<tokio::task::JoinHandle<()>>,
}

struct HttpServerServices {
    task_service: Arc<RuntimeTaskService>,
    conversation_service: Arc<ConversationRuntimeService>,
}

/// Run the full bootstrap sequence with startup timeout enforcement.
///
/// Returns [`BootstrapContext`] on success, or an error if:
/// - External services are unreachable (NATS, PostgreSQL)
/// - Startup timeout expires (configured via `observability.startup_timeout_secs`)
pub async fn bootstrap(
    config: &FrameworkConfig,
    state_tracker: &ProcessStateTracker,
    otel_guard: &ObservabilityGuard,
) -> Result<BootstrapContext, Box<dyn std::error::Error + Send + Sync>> {
    let startup_timeout = Duration::from_secs(config.observability.startup_timeout_secs);

    match tokio::time::timeout(
        startup_timeout,
        bootstrap_inner(config, state_tracker, otel_guard),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            error!(
                timeout_secs = config.observability.startup_timeout_secs,
                "Startup timed out"
            );
            state_tracker.set(ProcessLifecycle::Failed);
            Err("startup timeout exceeded".into())
        }
    }
}

async fn bootstrap_inner(
    config: &FrameworkConfig,
    state_tracker: &ProcessStateTracker,
    otel_guard: &ObservabilityGuard,
) -> Result<BootstrapContext, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    info!("Bootstrap sequence starting");

    // Shutdown coordination
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // Step 1: Initialize EventBus
    let event_bus = Arc::new(EventBus::new(1024));
    observability::spawn_autonomy_metrics_observer(event_bus.clone(), shutdown_flag.clone());
    info!("EventBus initialized");

    // Step 2: Initialize monitoring infrastructure
    let health_monitor = Arc::new(HealthMonitor::new(
        config.agent.monitoring.health_check_interval,
    ));
    let metrics_collector = Arc::new(MetricsCollector::new(
        config.agent.monitoring.metrics_export_interval,
    ));
    info!("Monitoring initialized");

    // Step 3: Connect to NATS (if configured)
    let nats_transport = connect_nats(config).await?;

    // Step 4: Initialize supervision tree
    let actor_config = mister_smith_actor::ActorSystemConfig::default();
    let supervised_system = Arc::new(SupervisedSystem::with_event_bus(
        actor_config,
        event_bus.clone(),
    ));
    let supervision_handle = Some(supervised_system.start_supervision());
    info!("Supervision tree initialized");

    // Step 5: Initialize runtime-backed task execution
    let task_service = RuntimeTaskService::bootstrap(
        event_bus.clone(),
        nats_transport.clone(),
        supervised_system.clone(),
    )
    .await
    .map_err(|error| format!("runtime task service bootstrap failed: {error}"))?;
    let conversation_service = ConversationRuntimeService::new(task_service.clone());
    info!("Runtime task service initialized");

    // Step 6: Initialize agent registry
    let agent_registry = Arc::new(AgentRegistry::new());
    info!("Agent registry initialized");

    // Step 7: Start background monitoring loops
    let monitor_handle = {
        let monitor = health_monitor.clone();
        let flag = shutdown_flag.clone();
        Some(tokio::spawn(async move {
            monitor.run(flag).await;
        }))
    };

    let metrics_handle = {
        let metrics = metrics_collector.clone();
        let flag = shutdown_flag.clone();
        Some(tokio::spawn(async move {
            metrics.run(flag).await;
        }))
    };
    info!("Background monitors started");

    // Step 8: Start HTTP server (with /metrics endpoint if prometheus enabled)
    let http_handle = start_http_server(
        config,
        &shutdown_tx,
        otel_guard,
        state_tracker,
        event_bus.clone(),
        nats_transport.clone(),
        HttpServerServices {
            task_service,
            conversation_service,
        },
    )
    .await?;

    // Step 9: Mark ready
    state_tracker.set(ProcessLifecycle::Ready);
    let startup_duration = start.elapsed();
    info!(
        duration_ms = startup_duration.as_millis() as u64,
        "Mister Smith ready"
    );

    Ok(BootstrapContext {
        event_bus,
        health_monitor,
        metrics_collector,
        supervised_system,
        supervision_handle,
        agent_registry,
        nats_transport,
        shutdown_tx,
        http_handle,
        shutdown_flag,
        monitor_handle,
        metrics_handle,
    })
}

/// Connect to NATS with timeout. Returns `None` if NATS is not configured.
async fn connect_nats(
    config: &FrameworkConfig,
) -> Result<Option<Arc<NatsTransport>>, Box<dyn std::error::Error + Send + Sync>> {
    let nats_url = match &config.transport.nats_url {
        Some(url) => url.clone(),
        None => {
            warn!("NATS URL not configured — running without messaging transport");
            return Ok(None);
        }
    };

    info!(url = %nats_url, "Connecting to NATS");
    let nats_config = NatsTransportConfig {
        server_urls: vec![nats_url.clone()],
        ..Default::default()
    };
    let transport = NatsTransport::new(nats_config);

    match tokio::time::timeout(Duration::from_secs(10), transport.connect()).await {
        Ok(Ok(())) => {
            info!("NATS connected");
            Ok(Some(Arc::new(transport)))
        }
        Ok(Err(e)) => {
            error!(error = %e, "Failed to connect to NATS");
            Err(format!("NATS connection failed: {e}").into())
        }
        Err(_) => {
            error!("NATS connection timed out (10s)");
            Err("NATS connection timed out".into())
        }
    }
}

/// Start the HTTP server for health probes, API endpoints, and /metrics.
///
/// Uses `build_router` from the HTTP crate with a broadcast-based shutdown
/// signal for coordinated process shutdown. Adds `/metrics` Prometheus endpoint
/// when the prometheus recorder is enabled.
async fn start_http_server(
    config: &FrameworkConfig,
    shutdown_tx: &broadcast::Sender<()>,
    otel_guard: &ObservabilityGuard,
    state_tracker: &ProcessStateTracker,
    event_bus: Arc<EventBus>,
    nats_transport: Option<Arc<NatsTransport>>,
    services: HttpServerServices,
) -> Result<Option<tokio::task::JoinHandle<()>>, Box<dyn std::error::Error + Send + Sync>> {
    let port = config.transport.http_port.unwrap_or(8080);
    let bind_address = format!("0.0.0.0:{port}");

    let http_config = mister_smith_http::HttpTransportConfig {
        bind_address: bind_address.clone(),
        ..Default::default()
    };
    let autonomy_pool = services.task_service.pool();
    let autonomy_task_service = services.task_service.clone();
    let app_state = mister_smith_http::AppState::new()
        .with_transport_health(Arc::new(mister_smith_http::server::NatsHealthCheck::new(
            nats_transport.is_some(),
        )))
        .with_task_service(services.task_service)
        .with_conversation_service(services.conversation_service);
    let mut app = mister_smith_http::server::build_router(&http_config, app_state);

    // Add Kubernetes health probe endpoints
    // GET /health/live — liveness probe (always 200 if server is responding)
    app = app.route(
        "/health/live",
        axum::routing::get(|| async {
            axum::Json(HealthProbeResponse {
                status: "alive",
                reason: None,
            })
        }),
    );

    // GET /health/ready — readiness probe (200 only when Ready, 503 otherwise)
    let tracker = state_tracker.clone();
    app = app.route(
        "/health/ready",
        axum::routing::get(move || {
            let t = tracker.clone();
            async move {
                if t.is_ready() {
                    (
                        axum::http::StatusCode::OK,
                        axum::Json(HealthProbeResponse {
                            status: "ready",
                            reason: None,
                        }),
                    )
                } else {
                    let state = format!("{:?}", t.get());
                    (
                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        axum::Json(HealthProbeResponse {
                            status: "not_ready",
                            reason: Some(format!("Process state: {state}")),
                        }),
                    )
                }
            }
        }),
    );
    info!("Health probe endpoints registered (/health/live, /health/ready)");

    let autonomy_bus = event_bus.clone();
    let autonomy_task_service_for_workflows = autonomy_task_service.clone();
    app = app.route(
        "/api/v1/autonomy/workflows",
        axum::routing::get(move || {
            let event_bus = autonomy_bus.clone();
            let task_service = autonomy_task_service_for_workflows.clone();
            async move {
                let mut workflows = autonomy::workflows_from_bus(event_bus).await.workflows;
                workflows.extend(
                    task_service
                        .autonomy_workflows()
                        .into_iter()
                        .map(|workflow_id| workflow_id.to_string()),
                );
                workflows.extend(
                    task_service
                        .persisted_autonomy_workflows()
                        .await
                        .into_iter()
                        .map(|workflow_id| workflow_id.to_string()),
                );
                workflows.sort();
                workflows.dedup();
                axum::Json(autonomy::AutonomyWorkflowList { workflows })
            }
        }),
    );

    let autonomy_bus = event_bus.clone();
    let autonomy_task_service_for_status = autonomy_task_service.clone();
    app = app.route(
        "/api/v1/autonomy/status/{workflow_id}",
        axum::routing::get(
            move |axum::extract::Path(workflow_id): axum::extract::Path<String>| {
                let event_bus = autonomy_bus.clone();
                let pool = autonomy_pool.clone();
                let task_service = autonomy_task_service_for_status.clone();
                async move {
                    match autonomy::status_from_bus_with_metadata_continuity(
                        event_bus,
                        pool,
                        &workflow_id,
                    )
                    .await
                    {
                        Ok(view) => Ok(axum::Json(view)),
                        Err(autonomy::AutonomyStatusError::NotFound(workflow_id)) => task_service
                            .autonomy_status(workflow_id)
                            .await
                            .map(axum::Json)
                            .ok_or(autonomy::AutonomyStatusError::NotFound(workflow_id)),
                        Err(error) => Err(error),
                    }
                }
            },
        ),
    );
    info!("Autonomy inspection endpoints registered (/api/v1/autonomy/...)");

    // Add /metrics endpoint if Prometheus is enabled
    if let Some(ref handle) = otel_guard.prometheus_handle {
        let handle = handle.clone();
        app = app.route(
            "/metrics",
            axum::routing::get(move || {
                let h = handle.clone();
                async move { h.render() }
            }),
        );
        info!("Prometheus /metrics endpoint enabled");
    }

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    info!(port = port, "HTTP server listening");

    let mut shutdown_rx = shutdown_tx.subscribe();
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        {
            error!(error = %e, "HTTP server error");
        }
    });

    Ok(Some(handle))
}
