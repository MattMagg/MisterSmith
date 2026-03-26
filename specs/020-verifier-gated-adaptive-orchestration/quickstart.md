# Quickstart: Verifier-Gated Adaptive Orchestration

## Targeted deterministic validation

Run the narrowest honest checks for the packet implementation:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-app
cargo clippy -p mister-smith-core -- -D warnings
cargo clippy -p mister-smith-app -- -D warnings
git diff --check
```

## Proof expectation

This packet does not earn a benchmark claim by itself. If implementation lands, proof should focus
on one bounded runtime transcript showing verifier, clarification, retry, or re-plan behavior on
the shipped baseline path without overstating broader benchmark or provider claims.

## Live-proof boundary

Deterministic validation can prove the verifier and repair control loop semantics before any live
runtime proof exists. If a live proof is later captured, it must stay explicitly bounded to the
actual exercised path and must not imply a new leaderboard result or a broader orchestration claim
than the evidence supports.
