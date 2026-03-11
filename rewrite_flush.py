import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# Fix the specific lines
content = content.replace("""            // Convert to AuditEntry
            let entries: Vec<AuditEntry> =
                new_events.into_iter().map(|e| Self::convert_event(e)).collect();

            // Batch insert
            let count = self.repository.append_batch(&entries).await?;

            let new_ids: HashSet<String> = new_events
                .into.into_iter()
                .map(|event| event.event_id.clone())
                .collect();""", """            let new_ids: HashSet<String> = new_events
                .iter()
                .map(|event| event.event_id.clone())
                .collect();

            // Convert to AuditEntry
            let entries: Vec<AuditEntry> =
                new_events.into_iter().map(Self::convert_event).collect();

            // Batch insert
            let count = self.repository.append_batch(&entries).await?;""")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
