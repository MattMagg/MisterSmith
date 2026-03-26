# 2026-03-26 Bounded Runtime Provider Selection

## Status

Implemented and locally validated on 2026-03-26

## Objective

Remove one documented limitation from the default Mister Smith runtime path by making the
runtime-backed task path select `provider_kind` and `model_id` from framework configuration while
preserving today's `openai_chatgpt` / `gpt-5.4` behavior as the default.

## Repo-Grounded Current Truth

- `docs/current-state.md` still records the live runtime path as fixed to `openai_chatgpt` with
  `gpt-5.4`, even though the repo already contains a provider-neutral LLM substrate and a
  deterministic `MockProvider`.
- `crates/mister-smith-app/src/execution.rs` hardcodes the runtime provider/model path and builds a
  single-provider `ModelRouter`.
- The current app binary depends on `mister-smith-llm` with `openai-chatgpt` and
  `claude-subscription` features enabled, and always has the deterministic `MockProvider`
  available.
- Smith frontier-mandate assessment classified this slice as legitimate product work and placed it
  in validated backlog posture, not queue staging.

## Scope

- add a typed `llm` configuration section to `FrameworkConfig`
- support bounded runtime selection for the providers the current binary actually ships:
  `openai_chatgpt`, `claude_subscription`, and `mock`
- preserve current default behavior and supervision/provenance metadata
- fail explicitly when configuration asks for a provider kind this binary does not ship

## Assumptions

- config-file plus `MISTER_SMITH_*` env overlay is the authoritative runtime configuration path
- this slice does not need multi-provider fan-out or live policy switching
- current operator-visible task/session/autonomy surfaces should continue to expose provider/model
  metadata, but now from selected runtime configuration rather than fixed constants

## Constraints

- no widening into budget-control, JetStream KV control loops, or new router policy programs
- no new external-agent or workflow-contract surfaces
- no queue staging as part of this slice
- keep the write set bounded to `mister-smith-config`, `mister-smith-app`, and state-bearing docs

## Non-Goals

- no support for providers that the current app binary does not compile in
- no new auth subcommands beyond what the current binary already exposes
- no change to the default routing policy
- no live runtime-proof rerun in this planning slice unless deterministic validation reveals a
  behavior gap that requires it

## Milestones

### Milestone 1: Freeze bounded packet and config shape

Deliverables:

- this planning note
- packet `017` under `specs/`
- one validated backlog issue with packet framing

Validation:

- packet and note cite current repo truth and explicit non-goals

### Milestone 2: Implement runtime provider/model selection

Deliverables:

- typed `llm` config with env overlay
- runtime bootstrap that builds the selected provider for the supported provider set
- conversation/task/autonomy metadata fed from runtime selection instead of fixed constants

Validation:

- targeted config and app tests for defaults, env overlay, and provider selection

### Milestone 3: Update state-bearing docs and verify build

Deliverables:

- `docs/current-state.md` updated only where shipped truth changed
- any repo orientation docs updated only if they make stale claims

Validation:

- `cargo test -p mister-smith-config`
- `cargo test -p mister-smith-app`
- `cargo build --workspace`

## Stop Conditions

- the slice would require adding providers not compiled into the current app binary
- the change would force a workflow-contract or queue-state mutation beyond validated backlog
- deterministic validation shows the change weakens supervision, provenance, or task/session
  continuity

## Implementation Result

- `mister-smith-config` now owns a typed `llm` section with defaults, validation, and env overlay
  support for `MISTER_SMITH_LLM__PROVIDER_KIND` and `MISTER_SMITH_LLM__MODEL_ID`
- `mister-smith-app` runtime bootstrap now resolves provider/model from framework config and builds
  only the shipped provider set: `openai_chatgpt`, `claude_subscription`, and `mock`
- task, session, and autonomy metadata now surface the selected runtime provider/model instead of
  fixed constants while preserving the existing default path
- unsupported provider kinds (`openai`, `anthropic`) now fail explicitly in the current app
  binary with bounded messaging instead of silent fallback

## Validation Evidence

- `cargo test -p mister-smith-config`
- `cargo test -p mister-smith-app`
- `cargo build --workspace`

## Remaining Boundary

- the live runtime-proof baseline is still the previously proven `openai_chatgpt` / `gpt-5.4`
  path
- alternate supported selections are now implemented and deterministically tested, but they do not
  yet carry equivalent live runtime-proof claims
