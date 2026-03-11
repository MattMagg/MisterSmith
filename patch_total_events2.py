import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

content = content.replace("total_events = new_ids.len(),", "total_events = count,")

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
