with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# I messed up git restore, maybe it didn't clear the bench code that I appended manually way back!
if "#[cfg(test)]\nmod benches {" in content:
    content = content.split("#[cfg(test)]\nmod benches {")[0]

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
