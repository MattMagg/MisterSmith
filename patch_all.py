import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# 1. Update flush method
content = content.replace("""            // Filter out already-persisted events
            let new_events: Vec<&SecurityAuditEvent> = events
                .iter()
                .filter(|e| !persisted.contains(&e.event_id))
                .collect();

            if new_events.is_empty() {
                return Ok(0);
            }

            // Convert to AuditEntry
            let entries: Vec<AuditEntry> =
                new_events.iter().map(|e| Self::convert_event(e)).collect();

            // Batch insert
            let count = self.repository.append_batch(&entries).await?;

            let new_ids: HashSet<String> = new_events
                .into_iter()
                .map(|event| event.event_id.clone())
                .collect();""", """            // Filter out already-persisted events
            let new_events: Vec<SecurityAuditEvent> = events
                .into_iter()
                .filter(|e| !persisted.contains(&e.event_id))
                .collect();

            if new_events.is_empty() {
                return Ok(0);
            }

            let new_ids: HashSet<String> = new_events
                .iter()
                .map(|event| event.event_id.clone())
                .collect();

            // Convert to AuditEntry
            let entries: Vec<AuditEntry> =
                new_events.into_iter().map(Self::convert_event).collect();

            // Batch insert
            let count = self.repository.append_batch(&entries).await?;""")

# Update total_events in debug macro
content = content.replace("total_events = events.len(),", "total_events = count,")

# 2. Update convert_event method
old_convert = """        pub fn convert_event(event: &SecurityAuditEvent) -> AuditEntry {
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

new_convert = """        pub fn convert_event(event: SecurityAuditEvent) -> AuditEntry {
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
                resource_type: event.resource,
                resource_id,
                action,
                old_values: None,
                new_values: None,
                metadata: serde_json::Value::Object(meta),
                correlation_id: None,
                created_at: event.timestamp,
            }
        }"""

content = content.replace(old_convert, new_convert)

# 3. Fix the 3 test calls to convert_event(&event) to convert_event(event) in mod tests { ... }
content = content.replace("AuditPersister::convert_event(&event)", "AuditPersister::convert_event(event)")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
