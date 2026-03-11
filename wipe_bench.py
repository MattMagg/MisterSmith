import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    lines = f.readlines()

new_lines = []
in_bench = False
for line in lines:
    if line.strip() == "fn bench_convert_event() {":
        in_bench = True
        # remove the previous #[test] as well
        if new_lines and new_lines[-1].strip() == "#[test]":
            new_lines.pop()
    if in_bench:
        if line.strip() == "println!(\"Time taken for 1000 convert_event calls with 1000 details (owned): {:?}\", start.elapsed());":
            in_bench = False
        continue
    new_lines.append(line)

with open("crates/mister-smith-persistence/src/audit_persister.rs", "w") as f:
    f.writelines(new_lines)
