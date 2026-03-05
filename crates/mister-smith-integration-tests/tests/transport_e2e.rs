//! End-to-end transport integration tests.
//!
//! T047: Send TaskAssignment through transport, process it, receive TaskResult back.
//! T049: Cross-transport integration tests.

use mister_smith_transport::{
    InMemoryTransport, MessageEnvelope, MessagePriority, SubjectTaxonomy, TaskAssignment,
    TaskResult, TaskStatus, Transport,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// T047: Full publish-process-reply pipeline via InMemoryTransport.
#[tokio::test]
async fn e2e_task_assignment_to_result() {
    let transport = Arc::new(InMemoryTransport::new());

    let worker_uuid = Uuid::new_v4();
    let supervisor_uuid = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    // Worker subscribes for task assignments.
    let subject = SubjectTaxonomy::task_assignment("data-processing").unwrap();
    let mut subscription = transport.subscribe(&subject).await.unwrap();

    // Supervisor publishes a TaskAssignment.
    let assignment = TaskAssignment {
        task_id,
        task_type: "data-processing".to_string(),
        payload: serde_json::json!({"batch": 42}),
        priority: MessagePriority::High,
        deadline: None,
        assigned_agent: Some(worker_uuid),
        requester_id: supervisor_uuid,
        metadata: HashMap::new(),
    };

    let envelope = MessageEnvelope::builder("task.assignment")
        .source_agent_id(supervisor_uuid)
        .target_agent_id(worker_uuid)
        .priority(MessagePriority::High)
        .payload_msgpack(&assignment)
        .unwrap()
        .build()
        .unwrap();

    transport.publish(&subject, envelope.clone()).await.unwrap();

    // Worker receives and processes the message.
    let received = tokio::time::timeout(Duration::from_secs(2), subscription.next())
        .await
        .expect("should not timeout")
        .expect("should receive message");

    // Verify envelope fields.
    assert_eq!(received.envelope.message_type, "task.assignment");
    assert_eq!(received.envelope.source_agent_id, Some(supervisor_uuid));
    assert_eq!(received.envelope.target_agent_id, Some(worker_uuid));

    // Deserialize the assignment.
    let received_assignment: TaskAssignment =
        mister_smith_transport::from_msgpack(&received.envelope.payload).unwrap();
    assert_eq!(received_assignment.task_id, task_id);
    assert_eq!(received_assignment.payload["batch"], 42);

    // Supervisor subscribes for result.
    let result_subject = SubjectTaxonomy::task_result(&task_id.to_string()).unwrap();
    let mut result_sub = transport.subscribe(&result_subject).await.unwrap();

    // Worker publishes TaskResult.
    let result = TaskResult {
        task_id,
        status: TaskStatus::Success,
        result: Some(serde_json::json!({"rows_processed": 1000})),
        error: None,
        duration_ms: 1500,
        agent_id: worker_uuid,
    };

    let result_envelope = MessageEnvelope::builder("task.result")
        .source_agent_id(worker_uuid)
        .target_agent_id(supervisor_uuid)
        .priority(MessagePriority::Normal)
        .correlation_id(received.envelope.message_id)
        .payload_msgpack(&result)
        .unwrap()
        .build()
        .unwrap();

    transport
        .publish(&result_subject, result_envelope)
        .await
        .unwrap();

    // Supervisor receives the result.
    let result_msg = tokio::time::timeout(Duration::from_secs(2), result_sub.next())
        .await
        .expect("should not timeout")
        .expect("should receive result");

    let received_result: TaskResult =
        mister_smith_transport::from_msgpack(&result_msg.envelope.payload).unwrap();
    assert_eq!(received_result.task_id, task_id);
    assert!(matches!(received_result.status, TaskStatus::Success));
    assert_eq!(received_result.result.unwrap()["rows_processed"], 1000);
    assert_eq!(received_result.duration_ms, 1500);
}

/// T047: Queue group delivery distributes work across multiple workers.
#[tokio::test]
async fn e2e_queue_group_task_distribution() {
    let transport = Arc::new(InMemoryTransport::new());

    let subject = SubjectTaxonomy::task_assignment("batch").unwrap();

    // Two workers in the same queue group.
    let mut worker1_sub = transport
        .queue_subscribe(&subject, "worker-pool")
        .await
        .unwrap();
    let mut worker2_sub = transport
        .queue_subscribe(&subject, "worker-pool")
        .await
        .unwrap();

    // Send 10 tasks.
    for _i in 0..10 {
        let assignment = TaskAssignment {
            task_id: Uuid::new_v4(),
            task_type: "batch".to_string(),
            payload: serde_json::json!({}),
            priority: MessagePriority::Normal,
            deadline: None,
            assigned_agent: None,
            requester_id: Uuid::new_v4(),
            metadata: HashMap::new(),
        };

        let envelope = MessageEnvelope::builder("task.assignment")
            .payload_msgpack(&assignment)
            .unwrap()
            .build()
            .unwrap();

        transport.publish(&subject, envelope).await.unwrap();
    }

    // Collect messages received by each worker.
    let mut worker1_count = 0usize;
    let mut worker2_count = 0usize;

    // Use a timeout to avoid hanging if messages don't arrive.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    for _ in 0..10 {
        tokio::select! {
            msg = worker1_sub.next() => {
                if msg.is_some() { worker1_count += 1; }
            }
            msg = worker2_sub.next() => {
                if msg.is_some() { worker2_count += 1; }
            }
            _ = tokio::time::sleep_until(deadline) => {
                break;
            }
        }
    }

    // Both workers should have received some messages (round-robin).
    assert_eq!(worker1_count + worker2_count, 10);
    assert_eq!(worker1_count, 5);
    assert_eq!(worker2_count, 5);
}

/// T047: Request-reply pattern for synchronous tool calls.
#[tokio::test]
async fn e2e_request_reply() {
    let transport = Arc::new(InMemoryTransport::new());

    let request_subject = "tools.call.read_file";

    // Subscribe BEFORE spawning to avoid race with request().
    let t2 = transport.clone();
    let mut sub = transport.subscribe(request_subject).await.unwrap();
    let handle = tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            if let Some(reply) = msg.reply_subject {
                // Must set correlation_id from the incoming message for reply matching.
                let response = MessageEnvelope::builder("tool.response")
                    .correlation_id(msg.envelope.correlation_id.unwrap())
                    .payload_msgpack(&serde_json::json!({"content": "file data"}))
                    .unwrap()
                    .build()
                    .unwrap();
                t2.publish(&reply, response).await.unwrap();
            }
        }
    });

    // Client sends request.
    let request = MessageEnvelope::builder("tool.call")
        .payload_msgpack(&serde_json::json!({"tool": "read_file", "path": "/tmp/test"}))
        .unwrap()
        .build()
        .unwrap();

    let response = transport
        .request(request_subject, request, Duration::from_secs(5))
        .await
        .unwrap();

    assert_eq!(response.message_type, "tool.response");
    let body: serde_json::Value =
        mister_smith_transport::from_msgpack(&response.payload).unwrap();
    assert_eq!(body["content"], "file data");

    handle.await.unwrap();
}

/// T049: MessageEnvelope serialization roundtrip across transport boundaries.
#[tokio::test]
async fn cross_transport_envelope_roundtrip() {
    let task_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let assignment = TaskAssignment {
        task_id,
        task_type: "analysis".to_string(),
        payload: serde_json::json!({"key": "value"}),
        priority: MessagePriority::High,
        deadline: None,
        assigned_agent: None,
        requester_id,
        metadata: HashMap::new(),
    };

    // Serialize to MessagePack (what NATS transport would send).
    let msgpack_bytes = mister_smith_transport::to_msgpack(&assignment).unwrap();

    // Deserialize (what HTTP or gRPC receiver would do).
    let decoded: TaskAssignment = mister_smith_transport::from_msgpack(&msgpack_bytes).unwrap();
    assert_eq!(decoded.task_id, task_id);
    assert_eq!(decoded.task_type, "analysis");

    // JSON roundtrip (HTTP transport path).
    let json_str = mister_smith_transport::to_json(&assignment).unwrap();
    let json_decoded: TaskAssignment = mister_smith_transport::from_json(&json_str).unwrap();
    assert_eq!(json_decoded.requester_id, requester_id);
    assert_eq!(json_decoded.payload["key"], "value");

    // Envelope bytes roundtrip.
    let envelope = MessageEnvelope::builder("task.assignment")
        .priority(MessagePriority::High)
        .payload_msgpack(&assignment)
        .unwrap()
        .build()
        .unwrap();

    let bytes = envelope.to_bytes().unwrap();
    let restored = MessageEnvelope::from_bytes(&bytes).unwrap();
    assert_eq!(restored.message_type, "task.assignment");
    assert_eq!(restored.priority, MessagePriority::High);

    let restored_assignment: TaskAssignment =
        mister_smith_transport::from_msgpack(&restored.payload).unwrap();
    assert_eq!(restored_assignment.task_id, task_id);
}

/// T049: Subject taxonomy generates consistent subjects across transports.
#[tokio::test]
async fn subject_taxonomy_consistency() {
    let assign_subject = SubjectTaxonomy::task_assignment("default").unwrap();
    assert_eq!(assign_subject, "tasks.default.assignment");

    let result_subject = SubjectTaxonomy::task_result("task-123").unwrap();
    assert_eq!(result_subject, "tasks.task-123.result");

    let heartbeat_subject = SubjectTaxonomy::agent_heartbeat("agent-1").unwrap();
    assert_eq!(heartbeat_subject, "agents.agent-1.heartbeat");

    let status_subject = SubjectTaxonomy::agent_status("worker-1").unwrap();
    assert_eq!(status_subject, "agents.worker-1.status");

    let system_health = SubjectTaxonomy::system_health();
    assert_eq!(system_health, "system.health");
}
