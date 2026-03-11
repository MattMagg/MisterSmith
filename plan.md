1. **Understand:** The issue states "Clone inside loop" in `crates/mister-smith-persistence/src/audit_persister.rs:144`.
   The code looks like this:
   ```rust
            let mut meta = serde_json::Map::new();
            for (k, v) in &event.details {
                meta.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
   ```
   `event.details` is a `HashMap<String, String>` (or similar) from `SecurityAuditEvent`. `meta` is a `serde_json::Map<String, serde_json::Value>`.

   Wait, let's look at `event_type` and `action` as well:
   ```rust
            // Action: use the event's action field or derive from event_type
            let action = event.action.clone().unwrap_or_else(|| event_type.clone());
   ```

   Since the function signature is `pub fn convert_event(event: &SecurityAuditEvent) -> AuditEntry`, we only have a reference to `event`. So if we want to extract strings from it to build `AuditEntry`, we normally have to clone them.
   Wait, is `event.details` iterating and cloning `k` and `v`?
   Can we avoid cloning `k.clone()`? `serde_json::Map::insert` takes `String` for the key. If we only have a reference, we must allocate a new string.
   Wait, what if `AuditPersister` can consume the `SecurityAuditEvent` instead of taking a reference?

   Let's check `flush`:
   ```rust
            let events = self.logger.recent_events(self.batch_size); // returns Vec<SecurityAuditEvent>
            ...
            let new_events: Vec<&SecurityAuditEvent> = events
                .iter()
                .filter(|e| !persisted.contains(&e.event_id))
                .collect();
            ...
            let entries: Vec<AuditEntry> =
                new_events.iter().map(|e| Self::convert_event(e)).collect();
   ```
   Here `events` is an owned `Vec<SecurityAuditEvent>`. `new_events` is a vector of references to `events`.
   If we change `new_events` to take ownership of the filtered elements, we can pass `SecurityAuditEvent` by value to `convert_event(event: SecurityAuditEvent)` and avoid all the `.clone()` calls inside `convert_event`!

2. **Measure:** We have created `bench_convert_event` in `audit_persister.rs` and it takes ~1.2s for 1000 calls with 1000 details when taking a reference. We will modify `convert_event` to take ownership and measure again.

3. **Implement:**
   Modify `convert_event(event: SecurityAuditEvent) -> AuditEntry`.
   Update `flush` to use `.into_iter()` on `events` (after filtering) and pass owned items to `convert_event`.
   Remove all the `clone()` calls inside `convert_event`.

4. **Verify:** Run tests `cargo test -p mister-smith-persistence`.
