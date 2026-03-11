import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# Let's remove the benchmark test from the main file since it's just for verification
bench_module = r"#\[cfg\(test\)\]\nmod benches \{.*?\}\n"
content = re.sub(bench_module, "", content, flags=re.DOTALL)

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.write(content)
