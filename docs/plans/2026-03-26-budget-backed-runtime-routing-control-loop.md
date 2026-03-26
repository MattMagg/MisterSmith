# 2026-03-26 Budget-Backed Runtime Routing Control Loop

## Status

Completed on `main` as of 2026-03-26

## Landed On Main So Far

- typed `llm.runtime_routing_profile` config for one bounded shipped-provider routing profile
- config-gated multi-provider runtime bootstrap with preserved single-provider fallback when no
  profile is configured
- one JetStream-backed runtime `BudgetStore` adapter plus runtime `BudgetEnforcer` bootstrap
  preflight against the configured budget root
- task/autonomy provenance that now surfaces runtime routing policy, budget root, and latest
  accepted step tier/checkpoint evidence

## Remaining Bounded Gap

- none within packet `019` scope

## Objective

Freeze one bounded next-phase packet that turns the current runtime router from a single-provider
`RoundRobin` placeholder into a real budget-aware control loop backed by shipped multi-provider
runtime configuration and JetStream-backed budget state.

## Repo-Grounded Current Truth

- `docs/current-state.md` still records two remaining runtime gaps after packet `017`:
  - the live proof baseline is only `openai_chatgpt` / `gpt-5.4`
  - the config-gated budget-aware path is not yet the unqualified live-proof baseline
- `crates/mister-smith-app/src/execution.rs` now derives a `RuntimeBootstrapPlan`, preserves the
  single-provider fallback when no routing profile exists, and can boot a bounded registered
  provider set plus `BudgetEnforcer` wiring when `llm.runtime_routing_profile` is configured.
- `crates/mister-smith-llm/src/router.rs` already contains:
  - `RoutingPolicy::Cascade`
  - budget reservation and reconciliation hooks
  - routing checkpoints for budget pressure, confidence, and provider fallback
  - tests proving cascade plus budget behavior in isolation
- `crates/mister-smith-llm/src/budget.rs` contains the bounded budget abstraction:
  `BudgetStore`, `BudgetEnforcer`, `BudgetPolicy`, and CAS-based reconciliation semantics.
- `crates/mister-smith-config/src/types.rs` now exposes typed runtime routing profile config for
  shipped-provider tiers plus one configured budget root.
- packet `018` is currently the smoke-harness lane in review, so the next packet number reserved in
  the main checkout for new scope is `019`.

## Proof Closure

- repo-owned harness support now includes one bounded budget-aware proof profile:
  `python3 scripts/live_runtime_proof_smoke.py --profile budget_softcap_openai_mock`
- committed live artifact bundle:
  `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/`
- committed proof note:
  `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
- live proof remains explicitly bounded to:
  - accepted provider-backed tier `openai_chatgpt` with `gpt-5.4`
  - one registered `mock` fallback tier
  - one seeded `soft_cap` budget root at `runtime.task_path`
- this closes the packet only because the proof note and artifact bundle keep deterministic router
  semantics separate from the narrower live runtime claim

## Scope

- add typed runtime routing configuration for one bounded multi-provider runtime profile using only
  shipped providers
- let the runtime-backed task path boot more than one provider into the router when that profile is
  configured
- add one JetStream-backed runtime budget-store implementation that satisfies the existing
  `BudgetStore` contract
- activate one bounded budget-aware cascade routing policy on the runtime task path
- preserve current task, session, and autonomy provenance while extending routing metadata with
  tier and budget-aware reasoning
- keep today's single-provider path available as the explicit fallback/default until proof expands

## Assumptions

- the current app binary still ships only `openai_chatgpt`, `claude_subscription`, and `mock`
- the existing `ModelRouter`, `CascadePolicy`, and `BudgetEnforcer` abstractions are the canonical
  runtime routing substrate
- packet `018` or an equivalent repeatable proof harness will exist before this lane makes a new
  live runtime-proof claim
- one runtime-local budget root and one cascade profile are sufficient for the first activation

## Constraints

- no new provider implementations
- no external-agent or workflow-contract expansion
- no broad operator-console redesign; extend existing operator/task/autonomy surfaces only where
  the runtime decision data already belongs
- no multi-tenant budget governance in this packet
- no queue staging or new worktree/PR choreography as part of this scope-freeze pass

## Non-Goals

- no requirement that the budget-backed path become the unqualified default on day one
- no live proof claim for alternate providers unless the harness or equivalent evidence is updated
  honestly
- no dynamic runtime policy mutation from external control-plane writes after boot
- no reopening packet `017` or packet `018` beyond the interfaces this packet depends on
- no wider session/conversation routing program beyond the runtime-backed task path

## Milestones

### Milestone 1: Freeze packet and runtime-profile boundary

Deliverables:

- this planning note
- packet `019` under `specs/`
- state-bearing docs updated to point at this packet as the current next phase

Validation:

- note and packet cite current repo truth, explicit constraints, and the single-provider bootstrap
  gap in `execution.rs`

### Milestone 2: Add bounded multi-provider runtime bootstrap

Deliverables:

- typed config for a runtime routing profile
- runtime bootstrap that can build and register a bounded provider set instead of exactly one
  provider
- explicit fallback behavior that preserves today's single-provider path when the new profile is
  not configured

Validation:

- targeted config and app tests for default behavior, profile parsing, and shipped-provider
  validation

### Milestone 3: Wire budget store and cascade control loop

Deliverables:

- one JetStream-backed `BudgetStore` implementation
- runtime bootstrap wiring for `BudgetEnforcer`
- one bounded cascade policy exercised on the runtime task path

Validation:

- `mister-smith-llm` router and budget tests expanded for the production store adapter
- targeted runtime tests proving budget-aware escalation, downgrade, or rejection semantics

### Milestone 4: Extend proof/evidence and refresh state docs

Deliverables:

- repeatable proof guidance or harness integration for the budget-aware runtime path
- `docs/current-state.md` and related router docs updated only where shipped truth changes

Validation:

- deterministic validation for the new routing path
- live smoke/evidence only if the environment can prove the bounded path honestly

## Stop Conditions

- the packet would require providers the current app binary does not ship
- the packet cannot preserve today's single-provider path as a bounded fallback
- the implementation requires a broader distributed-control program than one runtime-local budget
  root and one cascade profile
- proof would depend on unlanded smoke-harness behavior without an honest substitute
