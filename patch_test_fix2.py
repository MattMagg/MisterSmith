import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# Let's restore the original convert_event struct init
old_struct = """            AuditEntry {
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
            }"""

new_struct = """            AuditEntry {
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
            }"""

if old_struct in content:
    content = content.replace(old_struct, new_struct)
else:
    print("Not found struct")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
