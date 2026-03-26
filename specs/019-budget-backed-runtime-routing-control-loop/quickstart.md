# Quickstart: Budget-Backed Runtime Routing Control Loop

## Keep the current fallback path

Run exactly as today with no routing profile configured:

```bash
cargo run -p mister-smith-app -- run
```

## Targeted validation

```bash
cargo test -p mister-smith-config
cargo test -p mister-smith-llm router_tests budget_tests
cargo test -p mister-smith-app
cargo clippy --workspace -- -D warnings
cargo build --workspace
```

## Repeatable budget-aware proof

```bash
python3 scripts/live_runtime_proof_smoke.py --profile budget_softcap_openai_mock
```

This profile keeps the provider-backed acceptance path on `openai_chatgpt` / `gpt-5.4`, adds one
registered `mock` fallback tier, seeds `runtime.task_path` with `soft_cap`, and proves the live
`cascade` + budget-aware `downgrade` path without claiming broader alternate-provider proof.

See `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` for the committed artifact
bundle and explicit proof boundaries.

## Proof boundary

Deterministic validation still carries the broader fallback and hard-cap semantics. The committed
live proof is intentionally narrower: it covers the configured `budget_softcap_openai_mock`
profile only and does not expand the provider-backed live-proof claim beyond the accepted
`openai_chatgpt` / `gpt-5.4` baseline.
