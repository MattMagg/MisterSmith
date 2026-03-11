import re

with open("crates/mister-smith-persistence/src/audit_persister.rs", "r") as f:
    content = f.read()

# I see the problem. The regex `r"    #\[test\]\n    fn bench_convert_event\(\) \{.*?\n    \}"` in `patch_bench.py` matched everything down to the end of the first test `bench_convert_event` but it seems I removed the #[test] and fn signature when doing `content = re.sub(bench_module, "", content, flags=re.DOTALL)`.
# Actually `bench_module = r"#\[cfg\(test\)\]\nmod benches \{.*?\}\n"` failed to match, because I accidentally replaced it in `tests` module or something? Let's just restore from git and re-apply our changes.
