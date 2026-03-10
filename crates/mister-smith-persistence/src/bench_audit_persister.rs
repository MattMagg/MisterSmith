use std::collections::HashSet;

pub struct SecurityAuditEvent {
    pub event_id: String,
    pub data: String,
}

pub fn baseline(events: Vec<SecurityAuditEvent>, persisted: &mut HashSet<String>, batch_size: usize) {
    let new_events: Vec<&SecurityAuditEvent> = events
        .iter()
        .filter(|e| !persisted.contains(&e.event_id))
        .collect();

    if new_events.is_empty() {
        return;
    }

    for event in &new_events {
        persisted.insert(event.event_id.clone());
    }

    if persisted.len() > batch_size * 2 {
        let new_ids: HashSet<String> =
            new_events.iter().map(|e| e.event_id.clone()).collect();
        *persisted = new_ids;
    }
}

pub fn optimized(events: Vec<SecurityAuditEvent>, persisted: &mut HashSet<String>, batch_size: usize) {
    let new_events: Vec<SecurityAuditEvent> = events
        .into_iter()
        .filter(|e| !persisted.contains(&e.event_id))
        .collect();

    if new_events.is_empty() {
        return;
    }

    if persisted.len() + new_events.len() > batch_size * 2 {
        let new_ids: HashSet<String> =
            new_events.into_iter().map(|e| e.event_id).collect();
        *persisted = new_ids;
    } else {
        for event in new_events {
            persisted.insert(event.event_id);
        }
    }
}
