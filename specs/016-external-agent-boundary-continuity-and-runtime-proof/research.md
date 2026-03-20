# Research: External-Agent Boundary Continuity And Runtime Proof

**Date**: 2026-03-20  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

The next honest follow-on after packet `015` is not generic external-agent work. The repo already
has the bounded MCP discovery and enforcement surface from `MS-77`, plus operator-visible decision
surfaces for that boundary. The remaining gap is narrower:

- accepted delegated HTTP task ingress via `POST /api/v1/tasks` is not yet carried through
  persisted workflow metadata and projected onto workflow-level autonomy status as a first-class
  operator-visible boundary decision with preserved provenance and policy continuity

The strongest repo-grounded conclusion is therefore:

- treat `MS-48`, `MS-77`, packet `015`, and `MS-95` as landed baseline truth
- freeze packet `016` around delegated HTTP task ingress continuity and workflow-level runtime
  proof
- prefer reusing `external_capability_decisions`
- add a new `contracts/` doc only if research during implementation proves the existing summary
  cannot distinguish ingress decisions from outbound ToolBus decisions without ambiguity

## Current Repo Findings That Shape The Design

### Already Exists In Code

#### R1: `MS-77` already closed the bounded MCP discovery and enforcement surface

**Sources**:

- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
- `crates/mister-smith-mcp/src/server.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`

**Evidence**:

- MCP tools publish bounded capability descriptors and exact boundary actions
- delegated `Discover` and `Execute` actions are enforced at the MCP boundary
- `describe_external_capabilities` already exposes the bounded discovery surface and requires a
  delegated `Discover` envelope

**Decision**: packet `016` must not claim the first bounded external-agent surface is still
missing.

#### R2: Delegated HTTP task ingress is already forwarded and persisted as raw context

**Sources**:

- `crates/mister-smith-http/src/handlers.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/conversation.rs`

**Evidence**:

- `POST /api/v1/tasks` forwards `ExternalDelegationBoundary` into `TaskSubmissionRequest`
- workflow payload and metadata already persist `external_delegation`
- session creation and continuation also accept delegation, but the bounded current proof for this
  packet is clearest on `POST /api/v1/tasks`

**Decision**: freeze packet `016` around delegated HTTP task ingress via `POST /api/v1/tasks`
rather than widening silently to all delegated HTTP ingress.

#### R3: The active operator inspection contract is workflow-id based

**Sources**:

- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `docs/plans/2026-03-20-ms-95-post-merge-re-evaluation.md`

**Evidence**:

- the active HTTP route is `GET /api/v1/autonomy/status/{workflow_id}`
- the CLI uses `mister-smith autonomy status --workflow-id ...`
- `MS-95` closed the earlier failure-visible autonomy-status parity gap on that route

**Decision**: workflow-level autonomy status plus CLI parity is the correct proof surface for
packet `016`.

#### R4: Operator-visible decision plumbing already exists for the bounded MCP and ToolBus boundary

**Sources**:

- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `docs/plans/2026-03-19-ms-75-capability-decision-visibility.md`

**Evidence**:

- the ToolBus publishes `DelegationDecisionRecorded` operator-visible events
- autonomy status renders `external capability decisions:`
- persisted autonomy snapshots preserve allowed and rejected decision summaries

**Decision**: prefer reusing `external_capability_decisions` instead of inventing a second
operator decision surface.

### Not Yet First-Class Or Not Yet Proven End To End

#### R5: Metadata-only delegation context intentionally does not fabricate boundary decisions

**Sources**:

- `crates/mister-smith-app/src/execution.rs`

**Evidence**:

- persisted raw `external_delegation` survives in workflow metadata
- recovery tests explicitly assert that metadata-only delegation must not fabricate allowed or
  rejected operator-visible boundary decisions

**Decision**: packet `016` must preserve the no-fabrication rule and prove a real accepted ingress
path instead of inferring one from stored raw context.

#### R6: Accepted delegated HTTP task ingress does not yet have workflow-level decision continuity proof

**Sources**:

- `crates/mister-smith-http/src/handlers.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`

**Evidence**:

- accepted delegated HTTP task requests are forwarded and persisted
- workflow-level autonomy inspection already exists
- the current repo proof set does not yet show one accepted delegated HTTP task-ingress workflow
  surfacing a first-class accepted boundary decision on the supported workflow-level autonomy route

**Decision**: this is the core packet-`016` gap.

#### R7: Rejection proof is real, but live rejection proof may not be

**Sources**:

- `crates/mister-smith-http/src/server.rs`
- `crates/mister-smith-http/src/handlers.rs`

**Evidence**:

- deterministic rejection coverage already exists for missing, wrong-route, revoked, and mismatched
  delegated authority
- current repo grounding does not yet prove that rejected HTTP delegation creates a workflow-backed
  runtime artifact suitable for the same live proof shape as accepted ingress

**Decision**: keep deterministic rejection tests in scope and keep live rejection proof out of
scope unless the implementation work proves a workflow-backed reject surface already exists.

### Open Design Question

#### R8: `external_capability_decisions` may need a minimal discriminator, but that is not proven yet

**Sources**:

- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-agents/src/tool_bus.rs`

**Evidence**:

- current decision summaries are already rich enough for bounded MCP and ToolBus decisions
- the repo does not yet prove whether accepted ingress decisions can be distinguished from outbound
  ToolBus decisions without ambiguity using the existing summary shape alone

**Decision**: do not freeze a new JSON contract up front. Prefer the current summary and add a
backward-compatible discriminator or shape extension only if implementation research proves it is
necessary.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/plans/2026-03-20-ms-96-external-agent-pre-spec-decision.md` | Freezes the bounded packet-016 gap and out-of-scope list. |
| `docs/plans/2026-03-19-central-development-checkpoint.md` | Establishes packet `016` as the post-`MS-77` follow-on rather than packet-015 reopening. |
| `docs/current-state.md` | Distinguishes landed bounded external surfaces from the narrower remaining gap. |
| `docs/ms_recent_context.md` | Captures the repo-wide next-step framing after packet `015`. |
| `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md` | Defines the already-landed bounded MCP discovery/enforcement surface. |
| `docs/plans/2026-03-20-ms-95-post-merge-re-evaluation.md` | Confirms the active workflow-level autonomy route and packet-015 closure. |
| `crates/mister-smith-http/src/handlers.rs` | Shows delegated task-ingress forwarding at `POST /api/v1/tasks`. |
| `crates/mister-smith-app/src/execution.rs` | Shows raw `external_delegation` persistence and the no-fabrication invariant. |
| `crates/mister-smith-app/src/autonomy.rs` | Shows the active workflow-level inspection surface. |
| `crates/mister-smith-agents/src/tool_bus.rs` | Shows the existing operator-visible boundary decision surface. |

## Explicitly Deferred Questions

- whether session creation or continuation ingress should ever join this packet after the narrower
  task-ingress path is proven
- whether live rejection proof becomes appropriate if a real workflow-backed reject surface is
  discovered during implementation
- whether the current operator-visible summary needs a minimal discriminator for ingress decisions
- whether any broader external-agent transport work is warranted after this bounded continuity lane
