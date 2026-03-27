# Quickstart: Profile-Aware Predictive Runtime Supervision

## Targeted deterministic validation

Run the narrowest honest checks for the packet implementation:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-agents
cargo test -p mister-smith-events
cargo test -p mister-smith-app
cargo clippy -p mister-smith-core -- -D warnings
cargo clippy -p mister-smith-agents -- -D warnings
cargo clippy -p mister-smith-events -- -D warnings
cargo clippy -p mister-smith-app -- -D warnings
npm --prefix apps/operator-console run build
git diff --check
```

## Proof expectation

This packet earns proof by showing one bounded supported-ingress run where supervisory evidence is
first-class runtime output:

- the task path records non-empty profile, guard, or intervention evidence
- any fingerprint influence is explicitly identified as advisory context
- packet `020` repair lineage and packet `021` supervisory lineage remain coherent in the result
  view

## Live-proof boundary

Deterministic validation can prove the supervision contract, fingerprint plumbing, and operator
surface rendering before a new live runtime rerun exists. If a live rerun is later captured, it
must stay explicitly bounded to the supported provider path that was actually exercised and must
not imply CKM, topology-search, or benchmark-wide claims that this packet does not implement.
