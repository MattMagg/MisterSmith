import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# We need to change bench_convert_event to pass the event by value (which means we can't do it 1000 times with the same event,
# or we clone it before converting). Wait, we can clone the event in the loop for the benchmark, but that includes clone time.
# Let's clone first, then benchmark the conversion itself.
new_bench = """    #[test]
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
            resource: Some("user".to_string()),
            details,
            outcome: AuditOutcome::Success,
            previous_hash: Some("abcd".to_string()),
        };

        let mut events = Vec::new();
        for _ in 0..1000 {
            events.push(event.clone());
        }

        let start = Instant::now();
        for e in events.into_iter() {
            let _ = AuditPersister::convert_event(e);
        }
        println!("Time taken for 1000 convert_event calls with 1000 details (owned): {:?}", start.elapsed());
    }"""

old_bench_regex = r"    #\[test\]\n    fn bench_convert_event\(\) \{.*?\n    \}"

content = re.sub(old_bench_regex, new_bench, content, flags=re.DOTALL)

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
