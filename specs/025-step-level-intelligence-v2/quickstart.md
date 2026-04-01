# Quickstart: Step-Level Intelligence v2

## Targeted deterministic validation

Run the narrowest honest checks for the future packet implementation:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-app
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
npm --prefix apps/operator-console run build
git diff --check
```

## Proof expectation

This packet earns proof by showing one bounded deterministic step-policy surface on top of current
runtime seams:

- at least one `keep` and one non-`keep` decision can be derived from deterministic inputs
- task and autonomy summaries can show score, chosen action, and budget-aware summary fields
- explicit placeholder-versus-grounded wording remains visible on the current inspect path

## Live-proof boundary

Deterministic validation can prove the step-policy contract, summary projection, and proof-honesty
behavior before any later live rerun exists. A later live rerun must stay explicitly bounded to
the supported provider path that was actually exercised and must not imply grounded task proof
from `workflow.execute_step` placeholder completion alone.
