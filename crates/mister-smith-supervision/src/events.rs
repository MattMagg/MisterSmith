//! Lifecycle event emission to the EventBus.
//!
//! Emits supervision events with correlation and causation IDs,
//! linking failure → restart event chains for observability.

use mister_smith_core::AgentId;
use mister_smith_events::{AgentEventType, EventBuilder, EventBus, EventType};
use uuid::Uuid;

/// Emit an event when an actor failure is detected by the supervision system.
///
/// Returns the event ID, which should be used as the `causation_id` for
/// subsequent restart events in the same failure chain.
pub async fn emit_failure_event(
    bus: &EventBus,
    actor_id: &AgentId,
    error: &str,
    correlation_id: Uuid,
) -> Uuid {
    let event = EventBuilder::new("supervision", EventType::Agent(AgentEventType::Failed))
        .with_payload(&serde_json::json!({
            "actor_id": actor_id.to_string(),
            "error": error,
        }))
        .with_correlation_id(correlation_id)
        .build();
    let event_id = event.id;
    let _ = bus.publish(event).await;
    event_id
}

/// Emit an event when a supervisor restarts an actor.
pub async fn emit_restart_event(
    bus: &EventBus,
    actor_id: &AgentId,
    supervisor_id: &AgentId,
    correlation_id: Uuid,
    causation_id: Uuid,
) {
    let event = EventBuilder::new("supervision", EventType::Agent(AgentEventType::Started))
        .with_payload(&serde_json::json!({
            "actor_id": actor_id.to_string(),
            "supervisor_id": supervisor_id.to_string(),
            "action": "restart",
        }))
        .with_correlation_id(correlation_id)
        .with_causation_id(causation_id)
        .build();
    let _ = bus.publish(event).await;
}

/// Emit an event when a failure escalates to a parent supervisor.
pub async fn emit_escalation_event(
    bus: &EventBus,
    supervisor_id: &AgentId,
    correlation_id: Uuid,
    causation_id: Uuid,
) {
    let event = EventBuilder::new(
        "supervision",
        EventType::Custom("supervision.escalation".into()),
    )
    .with_payload(&serde_json::json!({
        "supervisor_id": supervisor_id.to_string(),
    }))
    .with_correlation_id(correlation_id)
    .with_causation_id(causation_id)
    .build();
    let _ = bus.publish(event).await;
}

/// Emit an event when a supervisor exhausts its restart budget.
pub async fn emit_budget_exhausted_event(
    bus: &EventBus,
    supervisor_id: &AgentId,
    correlation_id: Uuid,
) {
    let event = EventBuilder::new(
        "supervision",
        EventType::Custom("supervision.budget_exhausted".into()),
    )
    .with_payload(&serde_json::json!({
        "supervisor_id": supervisor_id.to_string(),
    }))
    .with_correlation_id(correlation_id)
    .build();
    let _ = bus.publish(event).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_events::Event;

    fn collect_broadcast(bus: &EventBus) -> tokio::sync::broadcast::Receiver<Event> {
        bus.subscribe_broadcast()
    }

    #[tokio::test]
    async fn failure_event_has_correct_type_and_payload() {
        let bus = EventBus::default();
        let mut rx = collect_broadcast(&bus);

        let actor_id = AgentId::new();
        let correlation_id = Uuid::new_v4();

        let event_id =
            emit_failure_event(&bus, &actor_id, "test error", correlation_id).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(event.id, event_id);
        assert_eq!(
            event.event_type,
            EventType::Agent(AgentEventType::Failed)
        );
        assert_eq!(event.correlation_id, Some(correlation_id));
        assert_eq!(event.payload["error"], "test error");
    }

    #[tokio::test]
    async fn restart_event_links_correlation_and_causation() {
        let bus = EventBus::default();
        let mut rx = collect_broadcast(&bus);

        let actor_id = AgentId::new();
        let supervisor_id = AgentId::new();
        let correlation_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();

        emit_restart_event(
            &bus,
            &actor_id,
            &supervisor_id,
            correlation_id,
            causation_id,
        )
        .await;

        let event = rx.recv().await.unwrap();
        assert_eq!(
            event.event_type,
            EventType::Agent(AgentEventType::Started)
        );
        assert_eq!(event.correlation_id, Some(correlation_id));
        assert_eq!(event.causation_id, Some(causation_id));
        assert_eq!(event.payload["action"], "restart");
    }

    #[tokio::test]
    async fn escalation_event_emitted() {
        let bus = EventBus::default();
        let mut rx = collect_broadcast(&bus);

        let sup_id = AgentId::new();
        let cid = Uuid::new_v4();
        let cause = Uuid::new_v4();

        emit_escalation_event(&bus, &sup_id, cid, cause).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(
            event.event_type,
            EventType::Custom("supervision.escalation".into())
        );
        assert_eq!(event.correlation_id, Some(cid));
        assert_eq!(event.causation_id, Some(cause));
    }


    #[tokio::test]
    async fn failure_then_restart_chain_preserves_ids() {
        let bus = EventBus::default();
        let mut rx = collect_broadcast(&bus);

        let actor_id = AgentId::new();
        let supervisor_id = AgentId::new();
        let correlation_id = Uuid::new_v4();

        let failure_event_id =
            emit_failure_event(&bus, &actor_id, "boom", correlation_id).await;
        emit_restart_event(
            &bus,
            &actor_id,
            &supervisor_id,
            correlation_id,
            failure_event_id,
        )
        .await;

        let failure_event = rx.recv().await.unwrap();
        let restart_event = rx.recv().await.unwrap();

        assert_eq!(failure_event.id, failure_event_id);
        assert_eq!(failure_event.payload["actor_id"], actor_id.to_string());
        assert_eq!(
            failure_event.event_type,
            EventType::Agent(AgentEventType::Failed)
        );

        assert_eq!(restart_event.payload["actor_id"], actor_id.to_string());
        assert_eq!(
            restart_event.payload["supervisor_id"],
            supervisor_id.to_string()
        );
        assert_eq!(restart_event.correlation_id, Some(correlation_id));
        assert_eq!(restart_event.causation_id, Some(failure_event_id));
        assert_eq!(
            restart_event.event_type,
            EventType::Agent(AgentEventType::Started)
        );
    }

    #[tokio::test]
    async fn budget_exhausted_event_emitted() {
        let bus = EventBus::default();
        let mut rx = collect_broadcast(&bus);

        let sup_id = AgentId::new();
        let cid = Uuid::new_v4();

        emit_budget_exhausted_event(&bus, &sup_id, cid).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(
            event.event_type,
            EventType::Custom("supervision.budget_exhausted".into())
        );
        assert_eq!(event.correlation_id, Some(cid));
    }
}
