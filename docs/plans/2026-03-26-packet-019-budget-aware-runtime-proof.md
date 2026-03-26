# 2026-03-26 Packet 019 Budget-Aware Runtime Proof

## Status

Completed on `main` as of 2026-03-26

## Scope Closed

This note closes the remaining packet-019 proof gap for the config-gated budget-aware runtime path.

The proof is intentionally bounded:

- accepted provider-backed tier remains `openai_chatgpt` with `gpt-5.4`
- the configured runtime profile additionally registers one `mock` fallback tier
- the seeded budget root uses `soft_cap` so the live run can surface budget-aware routing without
  claiming alternate live-provider proof

## Repeatable Live Command

```bash
python3 scripts/live_runtime_proof_smoke.py --profile budget_softcap_openai_mock
```

## Prerequisites

- Docker available for `postgres` and `nats`
- local `openai_chatgpt` auth already established:

  ```bash
  cargo run -q -p mister-smith-app -- auth openai-chatgpt status
  ```

- free local HTTP port `8080`

## Live Artifact Bundle

- artifact root:
  `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/`
- primary summary:
  `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/smoke-summary.json`
- runtime profile used:
  `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/runtime-config.toml`

## Observed Live Evidence

- `task-result-summary.json` records:
  - `provider_kind=openai_chatgpt`
  - `model_id=gpt-5.4`
  - `routing_policy=cascade`
  - `registered_provider_count=2`
  - `budget_root=runtime.task_path`
- `autonomy-status.json` records the latest step-routing decision as:
  - `tier=primary`
  - `action=downgrade`
  - `triggered_checkpoints=["budget_policy"]`
- `budget-state-before.json` seeds `runtime.task_path` at `used_tokens=0` with `policy=soft_cap`
- `budget-state-after.json` shows the same budget root reconciled to `used_tokens=66029`
- `smoke-summary.json` records the completed workflow `dc6e05d8-bdd8-46f5-92ea-d01fedebbe48`
  with `graph_state=Completed` and `topology_kind=Hybrid`

## Deterministic Validation Versus Live Proof

Deterministic validation still carries the broader control-loop semantics:

- router cascade escalation/fallback and hard-cap rejection semantics
- JetStream-backed budget-store round-trip and missing-root bootstrap failure
- harness unit coverage for the new budget-aware smoke profile

This live packet-019 proof adds one narrower runtime claim:

- current `main` can boot the config-gated `cascade` profile with two registered providers,
  enforce a seeded `soft_cap` budget root on the runtime task path, and surface the resulting
  budget-aware `downgrade` decision through task/autonomy proof outputs

## Explicit Non-Claims

- this does not make the budget-aware profile the unqualified default when no routing profile is
  configured
- this does not claim live runtime proof for `claude_subscription`
- this does not claim that the fallback tier executed live in this artifact bundle
- this does not broaden the existing provider-backed baseline beyond
  `openai_chatgpt` / `gpt-5.4`
