#[cfg(test)]
mod benches {
    use super::*;
    use mister_smith_security::audit::{SecurityAuditEvent, AuditEventType, AuditEventOutcome};
    use std::collections::HashMap;
    use std::time::Instant;

    #[test]
    fn bench_convert_event() {
        let mut details = HashMap::new();
        for i in 0..1000 {
            details.insert(format!("key{}", i), format!("value{}", i));
        }

        let event = SecurityAuditEvent {
            event_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authentication,
            action: Some("login".to_string()),
            principal: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
            source_ip: Some("192.168.1.1".to_string()),
            resource_type: Some("user".to_string()),
            resource_id: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
            details,
            outcome: AuditEventOutcome::Success,
            previous_hash: Some("abcd".to_string()),
        };

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = AuditPersister::convert_event(&event);
        }
        println!("Time taken for 1000 convert_event calls with 1000 details: {:?}", start.elapsed());
    }
}
