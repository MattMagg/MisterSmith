import re

with open("crates/mister-smith-integration-tests/tests/persistence_integration.rs", "r") as f:
    content = f.read()

content = content.replace("AuditPersister::convert_event(&event)", "AuditPersister::convert_event(event)")
content = content.replace(".map(|e| AuditPersister::convert_event(e))", ".map(|e| AuditPersister::convert_event(e.clone()))")
content = content.replace("let entry = AuditPersister::convert_event(&event);", "let entry = AuditPersister::convert_event(event);")

with open("crates/mister-smith-integration-tests/tests/persistence_integration.rs", "w") as f:
    f.write(content)
