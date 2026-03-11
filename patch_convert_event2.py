import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

old_fn = """        pub fn convert_event(event: &SecurityAuditEvent) -> AuditEntry {
            let id = Uuid::parse_str(&event.event_id).unwrap_or_else(|_| Uuid::new_v4());

            // Map event_type enum to string
            let event_type = format!("{:?}", event.event_type);

            // Action: use the event's action field or derive from event_type
            let action = event.action.clone().unwrap_or_else(|| event_type.clone());

            // Build metadata from details + source_ip + outcome
            let mut meta = serde_json::Map::new();
            for (k, v) in &event.details {
                meta.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            if let Some(ref ip) = event.source_ip {
                meta.insert(
                    "source_ip".to_string(),
                    serde_json::Value::String(ip.clone()),
                );
            }
            meta.insert(
                "outcome".to_string(),
                serde_json::Value::String(format!("{:?}", event.outcome)),
            );
            if let Some(ref hash) = event.previous_hash {
                meta.insert(
                    "previous_hash".to_string(),
                    serde_json::Value::String(hash.clone()),
                );
            }

            // Try to parse principal as UUID for agent_id
            let agent_id = event
                .principal
                .as_ref()
                .and_then(|p| Uuid::parse_str(p).ok());

            AuditEntry {
                id,
                event_type,
                agent_id,
                resource_type: Some("security".to_string()),
                resource_id: None,
                action,
                old_values: None,
                new_values: None,
                metadata: serde_json::Value::Object(meta),
                correlation_id: None,
                created_at: event.timestamp,
            }
        }"""

new_fn = """        pub fn convert_event(event: SecurityAuditEvent) -> AuditEntry {
            let id = Uuid::parse_str(&event.event_id).unwrap_or_else(|_| Uuid::new_v4());

            // Map event_type enum to string
            let event_type = format!("{:?}", event.event_type);

            // Action: use the event's action field or derive from event_type
            let action = event.action.unwrap_or_else(|| event_type.clone());

            // Build metadata from details + source_ip + outcome
            let mut meta = serde_json::Map::new();
            for (k, v) in event.details.into_iter() {
                meta.insert(k, serde_json::Value::String(v));
            }
            if let Some(ip) = event.source_ip {
                meta.insert(
                    "source_ip".to_string(),
                    serde_json::Value::String(ip),
                );
            }
            meta.insert(
                "outcome".to_string(),
                serde_json::Value::String(format!("{:?}", event.outcome)),
            );
            if let Some(hash) = event.previous_hash {
                meta.insert(
                    "previous_hash".to_string(),
                    serde_json::Value::String(hash),
                );
            }

            // Try to parse principal as UUID for agent_id
            let agent_id = event
                .principal
                .as_ref()
                .and_then(|p| Uuid::parse_str(p).ok());

            let resource_id = event
                .resource
                .as_ref()
                .and_then(|r| Uuid::parse_str(r).ok());

            AuditEntry {
                id,
                event_type,
                agent_id,
                resource_type: Some("security".to_string()),
                resource_id,
                action,
                old_values: None,
                new_values: None,
                metadata: serde_json::Value::Object(meta),
                correlation_id: None,
                created_at: event.timestamp,
            }
        }"""

if old_fn in content:
    content = content.replace(old_fn, new_fn)
else:
    print("Not found exactly")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
