import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# Removing the whole #[cfg(test)] mod benches block which we appended earlier
# Since we appended it, we can just split at #[cfg(test)]\nmod benches {
if "#[cfg(test)]\nmod benches {" in content:
    content = content.split("#[cfg(test)]\nmod benches {")[0]

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
