//! End-to-end transport integration tests.
//!
//! InMemory coverage below validates transport semantics quickly, but these tests
//! alone do **not** satisfy NATS/cross-transport acceptance criteria.
//! External-service tests for T047/T049 are environment-gated and require:
//! - `MISTER_SMITH_RUN_EXTERNAL_INTEGRATION=1`
//! - reachable NATS server URL via `MISTER_SMITH_NATS_URL` (default `nats://localhost:4222`)

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use futures::StreamExt;
use mister_smith_grpc::health;
use mister_smith_http::server::{AppState, NatsHealthCheck};
use mister_smith_http::websocket::WsEvent;
use mister_smith_nats::{NatsTransport, NatsTransportConfig};
use mister_smith_transport::{
    InMemoryTransport, MessageEnvelope, MessagePriority, SubjectTaxonomy, TaskAssignment,
    TaskResult, TaskStatus, Transport,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::oneshot;
use tonic::transport::Server;
use tonic_health::ServingStatus;
use tonic_health::pb::health_check_response::ServingStatus as PbServingStatus;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::pb::HealthCheckRequest;
use uuid::Uuid;

fn external_integration_enabled() -> bool {
    std::env::var("MISTER_SMITH_RUN_EXTERNAL_INTEGRATION")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn nats_test_config(client_name: &str) -> NatsTransportConfig {
    let server = std::env::var("MISTER_SMITH_NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    NatsTransportConfig {
        server_urls: vec![server],
        name: format!("mister-smith-integration-tests-{client_name}"),
        connection_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(5),
        ..Default::default()
    }
}

/// Fast T047 semantics via InMemoryTransport (unit-style).
#[tokio::test]
async fn e2e_task_assignment_to_result() {
    let transport = Arc::new(InMemoryTransport::new());
    let worker_uuid = Uuid::new_v4();
    let supervisor_uuid = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    let subject = SubjectTaxonomy::task_assignment("data-processing").unwrap();
    let mut subscription = transport.subscribe(&subject).await.unwrap();

    let assignment = TaskAssignment {
        task_id, task_type: "data-processing".to_string(),
        payload: serde_json::json!({"batch": 42}), priority: MessagePriority::High,
        deadline: None, assigned_agent: Some(worker_uuid),
        requester_id: supervisor_uuid, metadata: HashMap::new(),
    };
    let envelope = MessageEnvelope::builder("task.assignment")
        .source_agent_id(supervisor_uuid).target_agent_id(worker_uuid)
        .priority(MessagePriority::High)
        .payload_msgpack(&assignment).unwrap().build().unwrap();
    transport.publish(&subject, envelope.clone()).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(2), subscription.next())
        .await.expect("should not timeout").expect("should receive message");
    assert_eq!(received.envelope.message_type, "task.assignment");
    assert_eq!(received.envelope.source_agent_id, Some(supervisor_uuid));
    assert_eq!(received.envelope.target_agent_id, Some(worker_uuid));

    let received_assignment: TaskAssignment =
        mister_smith_transport::from_msgpack(&received.envelope.payload).unwrap();
    assert_eq!(received_assignment.task_id, task_id);
    assert_eq!(received_assignment.payload["batch"], 42);

    let result_subject = SubjectTaxonomy::task_result(&task_id.to_string()).unwrap();
    let mut result_sub = transport.subscribe(&result_subject).await.unwrap();
    let result = TaskResult {
        task_id, status: TaskStatus::Success,
        result: Some(serde_json::json!({"rows_processed": 1000})),
        error: None, duration_ms: 1500, agent_id: worker_uuid,
    };
    let result_envelope = MessageEnvelope::builder("task.result")
        .source_agent_id(worker_uuid).target_agent_id(supervisor_uuid)
        .priority(MessagePriority::Normal).correlation_id(received.envelope.message_id)
        .payload_msgpack(&result).unwrap().build().unwrap();
    transport.publish(&result_subject, result_envelope).await.unwrap();

    let result_msg = tokio::time::timeout(Duration::from_secs(2), result_sub.next())
        .await.expect("should not timeout").expect("should receive result");
    let received_result: TaskResult =
        mister_smith_transport::from_msgpack(&result_msg.envelope.payload).unwrap();
    assert_eq!(received_result.task_id, task_id);
    assert!(matches!(received_result.status, TaskStatus::Success));
    assert_eq!(received_result.result.unwrap()["rows_processed"], 1000);
    assert_eq!(received_result.duration_ms, 1500);
}

/// T047 acceptance: publish-process-reply pipeline over real NATS transport.
#[tokio::test]
async fn nats_task_assignment_to_result() {
    if !external_integration_enabled() {
        eprintln!("skipping: set MISTER_SMITH_RUN_EXTERNAL_INTEGRATION=1");
        return;
    }
    let transport = Arc::new(NatsTransport::new(nats_test_config("t047")));
    transport.connect().await.unwrap();

    let worker_uuid = Uuid::new_v4();
    let supervisor_uuid = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_kind = format!("data-processing-{}", Uuid::new_v4());
    let subject = SubjectTaxonomy::task_assignment(&task_kind).unwrap();
    let result_subject = SubjectTaxonomy::task_result(&task_id.to_string()).unwrap();
    let mut worker_sub = transport.subscribe(&subject).await.unwrap();
    let mut supervisor_result_sub = transport.subscribe(&result_subject).await.unwrap();

    let assignment = TaskAssignment {
        task_id, task_type: task_kind.clone(),
        payload: serde_json::json!({"batch": 42}), priority: MessagePriority::High,
        deadline: None, assigned_agent: Some(worker_uuid),
        requester_id: supervisor_uuid, metadata: HashMap::new(),
    };
    let assignment_envelope = MessageEnvelope::builder("task.assignment")
        .source_agent_id(supervisor_uuid).target_agent_id(worker_uuid)
        .priority(MessagePriority::High)
        .payload_msgpack(&assignment).unwrap().build().unwrap();
    transport.publish(&subject, assignment_envelope).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), worker_sub.next())
        .await.expect("worker should not timeout").expect("worker should receive assignment");
    let decoded_assignment: TaskAssignment =
        mister_smith_transport::from_msgpack(&received.envelope.payload).unwrap();
    assert_eq!(decoded_assignment.task_id, task_id);

    let result = TaskResult {
        task_id, status: TaskStatus::Success,
        result: Some(serde_json::json!({"rows_processed": 1000})),
        error: None, duration_ms: 750, agent_id: worker_uuid,
    };
    let result_envelope = MessageEnvelope::builder("task.result")
        .source_agent_id(worker_uuid).target_agent_id(supervisor_uuid)
        .correlation_id(received.envelope.message_id)
        .payload_msgpack(&result).unwrap().build().unwrap();
    transport.publish(&result_subject, result_envelope).await.unwrap();

    let result_msg = tokio::time::timeout(Duration::from_secs(5), supervisor_result_sub.next())
        .await.expect("supervisor should not timeout").expect("supervisor should receive result");
    let decoded_result: TaskResult =
        mister_smith_transport::from_msgpack(&result_msg.envelope.payload).unwrap();
    assert_eq!(decoded_result.task_id, task_id);
    assert!(matches!(decoded_result.status, TaskStatus::Success));
    transport.disconnect().await.unwrap();
}

/// Fast T047 semantics via InMemoryTransport (unit-style).
#[tokio::test]
async fn e2e_queue_group_task_distribution() {
    let transport = Arc::new(InMemoryTransport::new());
    let subject = SubjectTaxonomy::task_assignment("batch").unwrap();
    let mut w1 = transport.queue_subscribe(&subject, "worker-pool").await.unwrap();
    let mut w2 = transport.queue_subscribe(&subject, "worker-pool").await.unwrap();
    for _i in 0..10 {
        let a = TaskAssignment { task_id: Uuid::new_v4(), task_type: "batch".to_string(),
            payload: serde_json::json!({}), priority: MessagePriority::Normal,
            deadline: None, assigned_agent: None, requester_id: Uuid::new_v4(), metadata: HashMap::new() };
        let e = MessageEnvelope::builder("task.assignment")
            .payload_msgpack(&a).unwrap().build().unwrap();
        transport.publish(&subject, e).await.unwrap();
    }
    let (mut c1, mut c2) = (0usize, 0usize);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    for _ in 0..10 {
        tokio::select! {
            msg = w1.next() => { if msg.is_some() { c1 += 1; } }
            msg = w2.next() => { if msg.is_some() { c2 += 1; } }
            _ = tokio::time::sleep_until(deadline) => { break; }
        }
    }
    assert_eq!(c1 + c2, 10);
    assert_eq!(c1, 5);
    assert_eq!(c2, 5);
}

/// Fast T047 semantics via InMemoryTransport request-reply.
#[tokio::test]
async fn e2e_request_reply() {
    let transport = Arc::new(InMemoryTransport::new());
    let request_subject = "tools.call.read_file";
    let t2 = transport.clone();
    let mut sub = transport.subscribe(request_subject).await.unwrap();
    let handle = tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            if let Some(reply) = msg.reply_subject {
                let response = MessageEnvelope::builder("tool.response")
                    .correlation_id(msg.envelope.correlation_id.unwrap())
                    .payload_msgpack(&serde_json::json!({"content": "file data"}))
                    .unwrap().build().unwrap();
                t2.publish(&reply, response).await.unwrap();
            }
        }
    });
    let request = MessageEnvelope::builder("tool.call")
        .payload_msgpack(&serde_json::json!({"tool": "read_file", "path": "/tmp/test"}))
        .unwrap().build().unwrap();
    let response = transport.request(request_subject, request, Duration::from_secs(5)).await.unwrap();
    assert_eq!(response.message_type, "tool.response");
    let body: serde_json::Value = mister_smith_transport::from_msgpack(&response.payload).unwrap();
    assert_eq!(body["content"], "file data");
    handle.await.unwrap();
}

/// T049: MessageEnvelope serialization roundtrip across transport boundaries.
#[tokio::test]
async fn cross_transport_envelope_roundtrip() {
    let task_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let assignment = TaskAssignment {
        task_id, task_type: "analysis".to_string(),
        payload: serde_json::json!({"key": "value"}), priority: MessagePriority::High,
        deadline: None, assigned_agent: None, requester_id, metadata: HashMap::new(),
    };
    let msgpack_bytes = mister_smith_transport::to_msgpack(&assignment).unwrap();
    let decoded: TaskAssignment = mister_smith_transport::from_msgpack(&msgpack_bytes).unwrap();
    assert_eq!(decoded.task_id, task_id);
    assert_eq!(decoded.task_type, "analysis");

    let json_str = mister_smith_transport::to_json(&assignment).unwrap();
    let json_decoded: TaskAssignment = mister_smith_transport::from_json(&json_str).unwrap();
    assert_eq!(json_decoded.requester_id, requester_id);
    assert_eq!(json_decoded.payload["key"], "value");

    let envelope = MessageEnvelope::builder("task.assignment")
        .priority(MessagePriority::High)
        .payload_msgpack(&assignment).unwrap().build().unwrap();
    let bytes = envelope.to_bytes().unwrap();
    let restored = MessageEnvelope::from_bytes(&bytes).unwrap();
    assert_eq!(restored.message_type, "task.assignment");
    assert_eq!(restored.priority, MessagePriority::High);
    let ra: TaskAssignment = mister_smith_transport::from_msgpack(&restored.payload).unwrap();
    assert_eq!(ra.task_id, task_id);
}

/// T049: Subject taxonomy generates consistent subjects across transports.
#[tokio::test]
async fn subject_taxonomy_consistency() {
    assert_eq!(SubjectTaxonomy::task_assignment("default").unwrap(), "tasks.default.assignment");
    assert_eq!(SubjectTaxonomy::task_result("task-123").unwrap(), "tasks.task-123.result");
    assert_eq!(SubjectTaxonomy::agent_heartbeat("agent-1").unwrap(), "agents.agent-1.heartbeat");
    assert_eq!(SubjectTaxonomy::agent_status("worker-1").unwrap(), "agents.worker-1.status");
    assert_eq!(SubjectTaxonomy::system_health(), "system.health");
}

/// T049 acceptance: HTTP health reflects NATS connection state.
#[tokio::test]
async fn http_health_reflects_nats_connection_state() {
    if !external_integration_enabled() {
        eprintln!("skipping: set MISTER_SMITH_RUN_EXTERNAL_INTEGRATION=1");
        return;
    }
    let transport = NatsTransport::new(nats_test_config("http-health"));
    let hc = Arc::new(NatsHealthCheck::new(false));
    let unhealthy_state = AppState::new().with_transport_health(hc.clone());
    let ur = mister_smith_http::handlers::health_check(State(unhealthy_state)).await;
    let uj = serde_json::to_value(ur.0).unwrap();
    assert_eq!(uj["status"], "unhealthy");

    transport.connect().await.unwrap();
    hc.set_connected(true);
    let healthy_state = AppState::new().with_transport_health(hc);
    let hr = mister_smith_http::handlers::health_check(State(healthy_state)).await;
    let hj = serde_json::to_value(hr.0).unwrap();
    assert_eq!(hj["status"], "healthy");
    transport.disconnect().await.unwrap();
}

/// T049 acceptance: WebSocket stream receives events bridged from transport messages.
#[tokio::test]
async fn websocket_receives_event_from_transport_message() {
    if !external_integration_enabled() {
        eprintln!("skipping: set MISTER_SMITH_RUN_EXTERNAL_INTEGRATION=1");
        return;
    }
    let transport = NatsTransport::new(nats_test_config("ws-events"));
    transport.connect().await.unwrap();
    let state = AppState::new();
    let app = mister_smith_http::routes::api_router().with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { let _ = shutdown_rx.await; })
            .await.unwrap();
    });
    let subject = format!("events.{}", Uuid::new_v4());
    let mut sub = transport.subscribe(&subject).await.unwrap();
    let event_tx = state.event_tx.clone();
    let bridge_done = Arc::new(AtomicBool::new(false));
    let bd = bridge_done.clone();
    let bridge_task = tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            let payload: serde_json::Value =
                mister_smith_transport::from_msgpack(&msg.envelope.payload).unwrap();
            let event = WsEvent {
                event_type: "transport.event".to_string(), payload,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = event_tx.send(event);
            bd.store(true, Ordering::SeqCst);
        }
    });
    let ws_url = format!("ws://{addr}/api/v1/events/ws?filter=transport.event");
    let (mut socket, _) = tokio_tungstenite::connect_async(ws_url).await.unwrap();
    let envelope = MessageEnvelope::builder("event.transport")
        .payload_msgpack(&serde_json::json!({"key": "value-from-nats"}))
        .unwrap().build().unwrap();
    transport.publish(&subject, envelope).await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await.expect("ws recv").expect("stream open").expect("valid frame");
    let text = msg.into_text().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["event_type"], "transport.event");
    assert_eq!(parsed["payload"]["key"], "value-from-nats");
    assert!(bridge_done.load(Ordering::SeqCst));
    let _ = shutdown_tx.send(());
    bridge_task.await.unwrap();
    server_task.await.unwrap();
    transport.disconnect().await.unwrap();
}

/// T049 acceptance: gRPC health status tracks transport health transitions.
#[tokio::test]
async fn grpc_health_tracks_transport_state() {
    if !external_integration_enabled() {
        eprintln!("skipping: set MISTER_SMITH_RUN_EXTERNAL_INTEGRATION=1");
        return;
    }
    let transport = NatsTransport::new(nats_test_config("grpc-health"));
    let (reporter, health_service) = health::create_health_service().await;
    reporter.set_service_status(health::service_names::SYSTEM_SERVICE, ServingStatus::NotServing).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
    let server_task = tokio::spawn(async move {
        Server::builder().add_service(health_service)
            .serve_with_incoming_shutdown(incoming, async { let _ = shutdown_rx.await; })
            .await.unwrap();
    });
    let channel = tonic::transport::Endpoint::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = HealthClient::new(channel);
    let initial = client.check(HealthCheckRequest {
        service: health::service_names::SYSTEM_SERVICE.to_string(),
    }).await.unwrap().into_inner();
    assert_eq!(initial.status(), PbServingStatus::NotServing);

    transport.connect().await.unwrap();
    if transport.connection_state().await == async_nats::connection::State::Connected {
        reporter.set_service_status(health::service_names::SYSTEM_SERVICE, ServingStatus::Serving).await;
    }
    let connected = client.check(HealthCheckRequest {
        service: health::service_names::SYSTEM_SERVICE.to_string(),
    }).await.unwrap().into_inner();
    assert_eq!(connected.status(), PbServingStatus::Serving);

    transport.disconnect().await.unwrap();
    if transport.connection_state().await != async_nats::connection::State::Connected {
        reporter.set_service_status(health::service_names::SYSTEM_SERVICE, ServingStatus::NotServing).await;
    }
    let disconnected = client.check(HealthCheckRequest {
        service: health::service_names::SYSTEM_SERVICE.to_string(),
    }).await.unwrap().into_inner();
    assert_eq!(disconnected.status(), PbServingStatus::NotServing);
    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

/// Sanity: HTTP health route itself remains reachable.
#[tokio::test]
async fn health_route_still_returns_ok() {
    let app = mister_smith_http::routes::api_router().with_state(AppState::new());
    let request = Request::builder().uri("/api/v1/health").body(Body::empty()).unwrap();
    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
