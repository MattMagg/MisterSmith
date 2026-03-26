# Feature Specification: Budget-Backed Runtime Routing Control Loop

**Feature Branch**: `019-budget-backed-runtime-routing-control-loop`
**Created**: 2026-03-26
**Status**: Draft
**Input**: `docs/current-state.md`,
`docs/plans/2026-03-21-post-packet-016-development-checkpoint.md`,
`docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md`, current runtime bootstrap in
`crates/mister-smith-app/src/execution.rs`, and router/budget code in
`crates/mister-smith-llm/src/router.rs` and `crates/mister-smith-llm/src/budget.rs`

## Current Truth & Scope

Current repo truth already includes:

- typed runtime provider/model selection for shipped providers through packet `017`
- a `ModelRouter` with `RoundRobin`, `CostOptimized`, `CapabilityMatched`, and `Cascade` policies
- budget reservation/reconciliation primitives with explicit `HardCap`, `SoftCap`, and
  `Conditioned` policies
- routing checkpoints and tests for budget-aware downgrade/escalation behavior

The unfinished runtime gap is narrower than a new provider program:

- the runtime-backed task path still boots exactly one provider into a `RoundRobin` router
- there is no typed runtime routing profile for multi-provider boot
- there is no production runtime budget-store wiring even though the router and budget abstractions
  already exist

This packet therefore freezes one bounded epic:

1. add a typed runtime routing profile for a bounded shipped-provider set
2. let the runtime task path boot a multi-provider router profile instead of exactly one provider
3. wire one JetStream-backed budget store and one bounded cascade policy into that runtime path
4. preserve current operator-visible provenance while extending routing evidence with tier and
   budget-aware reasoning

This is not a new provider-implementation packet, not a new external-agent packet, not a broad
control-plane mutation program, and not an operator-console redesign packet.

## User Scenarios & Testing

### User Story 1 - Boot a bounded multi-provider runtime profile (Priority: P1)

An operator configures a bounded runtime routing profile and the runtime boots more than one
shipped provider into the router without breaking today's single-provider fallback behavior.

**Independent Test**: load a runtime config with a bounded cascade profile and confirm the runtime
registers the declared shipped providers while the old config path still boots the current
single-provider default.

**Acceptance Scenarios**:

1. **Given** no runtime routing profile is configured, **When** the runtime boots, **Then** it
   preserves today's single-provider `openai_chatgpt` / `gpt-5.4` path.
2. **Given** a valid bounded runtime routing profile using shipped providers, **When** the runtime
   boots, **Then** it registers each configured provider tier and exposes the selected routing
   policy in runtime metadata.
3. **Given** a routing profile references a provider the current binary does not ship, **When**
   the runtime boots, **Then** startup fails explicitly instead of silently coercing the profile.

### User Story 2 - Enforce budget-aware routing decisions on the runtime task path (Priority: P1)

An operator runs a workflow through the runtime-backed task path and the router can downgrade,
escalate, or reject requests based on the configured budget policy instead of always behaving like
plain `RoundRobin`.

**Independent Test**: run targeted runtime tests that exercise hard-cap rejection plus one
budget-aware cascade or downgrade path and confirm the task path preserves supervision and
provenance.

**Acceptance Scenarios**:

1. **Given** a hard-cap budget would be exceeded by a fallback attempt, **When** the request runs,
   **Then** the runtime rejects that attempt and preserves the already-consumed first-tier usage
   honestly.
2. **Given** a soft-cap or conditioned budget profile, **When** the first tier remains viable but
   budget pressure is triggered, **Then** the routing signal records downgrade/escalation intent
   and the runtime stays coherent.
3. **Given** the first provider attempt fails, **When** a later tier succeeds, **Then** routing
   evidence preserves the provider-failure checkpoint, accepted tier, and final decision reason.

### User Story 3 - Inspect and prove budget-aware routing decisions (Priority: P2)

An operator can inspect budget-aware routing decisions through the existing task/autonomy surfaces,
and a developer can capture one repeatable proof or evaluation artifact for the new runtime path.

**Independent Test**: inspect task or autonomy output for a workflow that used the budget-aware
router and confirm the result includes tier, budget-policy, and fallback rationale; if the proof
environment is available, capture one durable artifact bundle.

**Acceptance Scenarios**:

1. **Given** a workflow uses the budget-aware runtime path, **When** the operator inspects task or
   autonomy status, **Then** the result includes routing policy, accepted tier, and budget-related
   checkpoints without requiring raw log archaeology.
2. **Given** a repeatable proof harness is available, **When** the budget-aware path is exercised,
   **Then** it produces durable artifacts that distinguish deterministic validation from live proof.
3. **Given** live proof is not available for the full path, **When** the packet closes, **Then**
   the state docs record exactly what remains deterministic-only instead of overstating the claim.

### Edge Cases

- a routing profile lists only one provider tier
- one provider tier lacks valid credentials at boot
- budget reservation succeeds for the first tier but a fallback reservation exceeds a hard cap
- JetStream budget state sees CAS contention during reconciliation
- the configured preferred tier is unavailable and the router must continue on remaining healthy
  tiers
- a repeatable proof harness for the new runtime path is not yet landed on `main`

## Requirements

### Functional Requirements

- **FR-001**: System MUST add a typed runtime routing profile to framework configuration.
- **FR-002**: System MUST preserve today's single-provider path when that profile is omitted.
- **FR-003**: System MUST validate that runtime routing profiles use only providers the current app
  binary actually ships.
- **FR-004**: System MUST allow the runtime-backed task path to boot more than one provider into
  `ModelRouter` when the new profile is configured.
- **FR-005**: System MUST wire one runtime `BudgetStore` implementation backed by JetStream state.
- **FR-006**: System MUST use the existing `BudgetEnforcer` contract rather than inventing a
  parallel budget path in the app layer.
- **FR-007**: System MUST activate one bounded cascade or downgrade-capable routing policy on the
  runtime task path.
- **FR-008**: System MUST preserve task, session, and autonomy provenance while extending routing
  evidence with policy, tier, and budget checkpoints.
- **FR-009**: System MUST fail explicitly when runtime routing configuration is invalid or cannot
  be satisfied by the shipped provider set.
- **FR-010**: System MUST keep the write set bounded to config, app runtime bootstrap, router and
  budget integration, targeted proof support, and state-bearing docs.
- **FR-011**: System MUST NOT widen into new provider implementations, new external-agent
  contracts, or a broad multi-tenant budget-governance program.
- **FR-012**: System MUST record validation boundaries honestly when deterministic tests land before
  equivalent live proof.

### Key Entities

- **RuntimeRoutingProfile**: typed framework configuration describing the runtime routing policy,
  declared provider tiers, and budget root for the task path
- **RuntimeProviderTier**: one shipped provider configuration registered into the runtime router for
  a bounded tier label
- **RuntimeBudgetState**: the JetStream-backed budget node consumed by `BudgetEnforcer` during
  runtime routing
- **RuntimeRoutingDecisionView**: operator-visible projection joining policy, accepted tier,
  provider/model selection, and budget/fallback checkpoints

## Success Criteria

- **SC-001**: the runtime preserves today's single-provider boot path when no routing profile is
  configured
- **SC-002**: a valid bounded routing profile boots multiple shipped providers into the runtime
  router
- **SC-003**: budget-aware routing behavior on the runtime task path is covered by targeted tests
  proving rejection, fallback, or downgrade semantics
- **SC-004**: task or autonomy surfaces expose tier plus budget-aware routing rationale without
  raw log inspection
- **SC-005**: the packet keeps live-proof claims honest by separating deterministic validation from
  any not-yet-proven runtime path
