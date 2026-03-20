# 2026-03-20 MS-96 External-Agent Pre-Spec Decision

Issue: `MS-96`  
Status: planning-only; packet `016` initialized

## Summary

This note freezes the bounded post-`MS-77` follow-on after packet `015` and `MS-95` completed on
`main`.

Decision:

- reuse `MS-96`
- keep the lane planning-only
- create exactly one new packet:
  `016-external-agent-boundary-continuity-and-runtime-proof`
- do not reopen packet `015`
- do not widen into provider, router, JetStream KV, A2A, mesh, CRDT, MPST, or queue work

## What `MS-48` Already Landed

`MS-48` is closed and should be treated as landed history.

Repo truth already includes:

- bounded delegation capability issuance and validation in
  `crates/mister-smith-security/src/delegation.rs`
- revocation-aware action validation and provenance-chain enforcement
- operator-visible delegation and external capability decision plumbing through the ToolBus and
  autonomy projection layers
- workflow metadata persistence for delegated ingress context

`MS-48` is not open scope for packet `016`.

## What `MS-77` Already Landed

`MS-77` is also closed and remains baseline truth on `main`.

It already landed:

- one bounded external-agent surface on the existing MCP boundary
- capability discovery metadata for that MCP surface
- exact delegated boundary-action enforcement at `tools/call`
- bounded operator inspection via `describe_external_capabilities`

Packet `016` is not the creation of the first bounded external-agent surface from scratch.

## Exact Residual Gap On `main`

The remaining gap is narrower than generic external-agent ingress:

- accepted delegated HTTP task ingress is not yet carried through persisted workflow metadata and
  projected onto workflow-level autonomy status as a first-class operator-visible boundary decision
  with preserved provenance and policy continuity

Current repo truth that constrains that statement:

- raw `external_delegation` is already persisted in workflow metadata
- `external_capability_decisions` already exists as an operator-visible decision surface for the
  bounded MCP and ToolBus boundary
- `GET /api/v1/autonomy/status/{workflow_id}` is the supported workflow-level autonomy route
- packet `015` plus `MS-95` already closed the failure-visible result-surface gap
- metadata-only delegation context must not fabricate an allowed or rejected operator-visible
  boundary decision

## Explicitly Out Of Scope

- reopening packet `015`
- widening the packet to all delegated HTTP ingress routes
- generic A2A or mesh protocol work
- CRDT, MPST, or distributed-memory expansion
- provider, router, budget, or JetStream KV follow-on work
- unrelated cleanup or framework-parity work
- watched-queue staging or implementation in this lane

## Packet Decision

Packet `016` is justified because the repo already has the bounded MCP discovery and enforcement
surface, but still lacks one equally bounded continuity-and-proof lane for delegated HTTP task
ingress through the active operator inspection path.

Freeze packet `016` around:

- delegated HTTP task ingress via `POST /api/v1/tasks`
- workflow metadata continuity for the accepted delegated request
- workflow-level inspection via `GET /api/v1/autonomy/status/{workflow_id}`
- CLI parity via `mister-smith autonomy status --workflow-id ...`
- deterministic rejection coverage for missing, wrong-route, revoked, or mismatched delegated
  authority

Live rejection proof stays out of scope unless a real workflow-backed reject surface already exists
on `main`.

## Required Artifact Set

- refresh:
  - `docs/plans/2026-03-19-central-development-checkpoint.md`
  - `docs/current-state.md`
  - `docs/ms_recent_context.md`
- create:
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/spec.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/data-model.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/research.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/plan.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/tasks.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/quickstart.md`
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/analyze.md`

Add `contracts/` only if packet research proves the current operator-visible JSON contract cannot
carry the ingress decision without ambiguity.
