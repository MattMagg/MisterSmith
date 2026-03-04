# Quickstart: Transport & Messaging Integration Scenarios

## Scenario 1: In-Memory Transport Round-Trip

Validates MessageEnvelope construction, serialization, and Transport trait compliance without external dependencies.

```rust
use mister_smith_transport::{
    MessageEnvelope, MessagePriority, InMemoryTransport, Transport,
};

#[tokio::test]
async fn envelope_round_trip() {
    let transport = InMemoryTransport::new();

    // Subscribe to a subject
    let mut sub = transport.subscribe("agents.agent-001.commands.task").await.unwrap();

    // Build and publish a message
    let envelope = MessageEnvelope::builder()
        .message_type("TaskAssignment")
        .source_agent_id("agent-000".into())
        .target_agent_id("agent-001".into())
        .priority(MessagePriority::Normal)
        .payload_msgpack(&TaskAssignment {
            task_id: Uuid::new_v4(),
            task_type: "analysis".into(),
            // ...
        })
        .unwrap()
        .build();

    transport.publish("agents.agent-001.commands.task", envelope.clone()).await.unwrap();

    // Receive and verify
    let received = sub.next().await.unwrap();
    assert_eq!(received.message_id, envelope.message_id);
    assert_eq!(received.message_type, "TaskAssignment");
    assert!(received.correlation_id.is_none());
}
```

## Scenario 2: NATS Publish/Subscribe

Validates NATS transport connecting to a live NATS server and delivering messages between agents.

```rust
use mister_smith_nats::{NatsTransport, NatsTransportConfig};
use mister_smith_transport::{Transport, MessageEnvelope};

#[tokio::test]
async fn nats_pub_sub() {
    let config = NatsTransportConfig {
        server_urls: vec!["nats://localhost:4222".into()],
        ..Default::default()
    };
    let transport = NatsTransport::connect(config).await.unwrap();

    let mut sub = transport.subscribe("agents.agent-001.commands.task").await.unwrap();

    let envelope = MessageEnvelope::builder()
        .message_type("TaskAssignment")
        .payload_msgpack(&task)
        .unwrap()
        .build();

    transport.publish("agents.agent-001.commands.task", envelope).await.unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(msg.message_type, "TaskAssignment");
}
```

## Scenario 3: NATS Request-Reply with Timeout

Validates the request-reply pattern with correlation ID matching and timeout behavior.

```rust
#[tokio::test]
async fn nats_request_reply() {
    let transport = NatsTransport::connect(config).await.unwrap();

    // Responder subscribes and replies
    let mut sub = transport.subscribe("tasks.analysis.assignment").await.unwrap();
    tokio::spawn(async move {
        let request = sub.next().await.unwrap();
        let reply = MessageEnvelope::builder()
            .message_type("TaskResult")
            .correlation_id(request.message_id) // echo correlation
            .payload_msgpack(&TaskResult { status: TaskStatus::Success, .. })
            .unwrap()
            .build();
        transport_clone.reply(request.reply_subject().unwrap(), reply).await.unwrap();
    });

    // Requester sends request with timeout
    let response = transport
        .request("tasks.analysis.assignment", request_envelope, Duration::from_secs(5))
        .await
        .unwrap();

    assert_eq!(response.message_type, "TaskResult");
    assert_eq!(response.correlation_id, Some(request_envelope.message_id));
}
```

## Scenario 4: JetStream Durable Messaging

Validates message persistence across consumer restarts.

```rust
#[tokio::test]
async fn jetstream_durability() {
    let transport = NatsTransport::connect(config).await.unwrap();
    let js = transport.jetstream();

    // Create stream for task subjects
    js.create_stream(StreamConfig {
        name: "TASKS".into(),
        subjects: vec!["tasks.>".into()],
        retention: RetentionPolicy::WorkQueue,
        ..Default::default()
    }).await.unwrap();

    // Publish a durable message
    let ack = js.publish("tasks.analysis.assignment", envelope_bytes).await.unwrap();
    ack.await.unwrap(); // double-await for PublishAckFuture

    // Create a pull consumer
    let consumer = js.create_consumer("TASKS", ConsumerConfig {
        durable_name: Some("worker-1".into()),
        ..Default::default()
    }).await.unwrap();

    // Consume and explicitly ack
    let mut messages = consumer.messages().await.unwrap();
    let msg = messages.next().await.unwrap().unwrap();
    msg.ack().await.unwrap();

    // After restart: acknowledged messages are not redelivered
}
```

## Scenario 5: HTTP API + WebSocket Events

Validates REST endpoint responses and WebSocket event streaming.

```rust
#[tokio::test]
async fn http_api_and_websocket() {
    let app = HttpTransport::router(app_state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, app));

    // REST: health check
    let resp = reqwest::get(format!("http://{addr}/api/v1/health")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(["healthy", "degraded", "unhealthy"].contains(&body["status"].as_str().unwrap()));

    // REST: list agents
    let resp = reqwest::get(format!("http://{addr}/api/v1/agents")).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().contains_key("x-request-id"));

    // WebSocket: event streaming
    let (mut ws, _) = tokio_tungstenite::connect_async(
        format!("ws://{addr}/api/v1/events/ws?filter=agent_status")
    ).await.unwrap();

    // Trigger an agent status change...
    // Verify WebSocket receives the event
    let msg = ws.next().await.unwrap().unwrap();
    let event: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    assert_eq!(event["type"], "agent_status");
}
```

## Scenario 6: End-to-End TaskAssignment → TaskResult Pipeline

The critical integration test from Gate 4: a full round-trip through NATS.

```rust
#[tokio::test]
async fn task_pipeline_e2e() {
    // Setup: NATS transport with two agents
    let transport = NatsTransport::connect(config).await.unwrap();

    // Worker agent subscribes to task assignments via queue group
    let mut worker_sub = transport
        .queue_subscribe("tasks.analysis.assignment", "workers")
        .await
        .unwrap();

    // Coordinator publishes a TaskAssignment
    let assignment = TaskAssignment {
        task_id: Uuid::new_v4(),
        task_type: "analysis".into(),
        payload: serde_json::json!({"input": "data"}),
        priority: MessagePriority::Normal,
        requester_id: "coordinator".into(),
        ..Default::default()
    };

    let request_envelope = MessageEnvelope::builder()
        .message_type("TaskAssignment")
        .source_agent_id("coordinator".into())
        .payload_msgpack(&assignment)
        .unwrap()
        .build();

    // Use request-reply for synchronous task execution
    let response = transport
        .request("tasks.analysis.assignment", request_envelope, Duration::from_secs(30))
        .await
        .unwrap();

    // Verify the response
    assert_eq!(response.message_type, "TaskResult");
    let result: TaskResult = response.payload_msgpack().unwrap();
    assert_eq!(result.task_id, assignment.task_id);
    assert_eq!(result.status, TaskStatus::Success);
}
```

## Scenario 7: Transport Health Integration

Validates transport health status reporting through the existing HealthMonitor infrastructure.

```rust
#[tokio::test]
async fn transport_health_reporting() {
    let transport = NatsTransport::connect(config).await.unwrap();
    let health_monitor = HealthMonitor::new();

    // Register transport health check
    health_monitor.register("nats_transport", transport.health_check()).await;

    // Verify healthy when connected
    let report = health_monitor.check_all().await;
    assert_eq!(report.get("nats_transport").unwrap().status, HealthStatus::Healthy);

    // Simulate disconnection...
    // Verify degraded/unhealthy status
}
```
