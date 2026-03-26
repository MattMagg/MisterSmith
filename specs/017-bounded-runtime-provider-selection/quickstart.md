# Quickstart: Bounded Runtime Provider Selection

## Defaults

Run exactly as today and keep the default runtime path:

```bash
cargo run -p mister-smith-app -- run
```

## Select the mock provider

```bash
MISTER_SMITH_LLM__PROVIDER_KIND=mock \
MISTER_SMITH_LLM__MODEL_ID=mock-ops \
cargo run -p mister-smith-app -- run
```

## Select Claude subscription

```bash
MISTER_SMITH_LLM__PROVIDER_KIND=claude_subscription \
MISTER_SMITH_LLM__MODEL_ID=claude-sonnet-4-5 \
cargo run -p mister-smith-app -- run
```

## Validation

```bash
cargo test -p mister-smith-config
cargo test -p mister-smith-app
cargo build --workspace
```
