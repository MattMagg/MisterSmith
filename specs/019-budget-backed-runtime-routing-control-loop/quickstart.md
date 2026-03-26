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

## Proof boundary

Do not claim the budget-backed runtime path is live-proven until one repeatable proof or evidence
flow exists on `main` for that path. Deterministic validation alone is not enough for that claim.
