import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

content = content.replace("total_events = new_events.len(),", "total_events = new_ids.len(),")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
