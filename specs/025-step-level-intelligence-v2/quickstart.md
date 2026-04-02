# Quickstart: Step-Level Intelligence v2

## Packet-readiness validation

Run the narrowest honest checks for the packet bundle itself:

```bash
npx markdownlint-cli2 "specs/025-step-level-intelligence-v2/**/*.md" --config .markdownlint.json
git diff --check
```

## Targeted implementation validation

Run the narrowest honest checks for the future packet implementation:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
npm --prefix apps/operator-console run build
npm --prefix apps/operator-console test
git diff --check
```

## Proof expectation

Packet `025` earns deterministic proof by showing one bounded packet-owned step-policy layer on
top of current runtime seams:

- at least one `keep` and one non-`keep` decision can be derived from deterministic inputs
- task inspect, autonomy status, and operator selected-run detail show the same step-policy fields
- explicit placeholder-versus-grounded wording remains visible through packet-023 proof wording

## Live-proof boundary

Packet `025` is implementation-ready, not live-proof-complete.

Deterministic validation can prove the packet-owned contract, runtime assembly, summary
projection, and proof-honesty behavior. Any later live runtime-proof claim must be captured in a
separate artifact and must not infer grounded task proof from `workflow.execute_step` placeholder
completion alone.
