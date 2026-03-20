# Data Model: External-Agent Boundary Continuity And Runtime Proof

**Date**: 2026-03-20  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `crates/mister-smith-http/src/handlers.rs` | Defines accepted delegated HTTP task-ingress forwarding. |
| `crates/mister-smith-app/src/execution.rs` | Defines persisted raw `external_delegation` context and the no-fabrication invariant. |
| `crates/mister-smith-app/src/autonomy.rs` | Defines workflow-level autonomy inspection and CLI parity expectations. |
| `crates/mister-smith-app/src/conversation.rs` | Defines retained session continuity rules that this packet must preserve. |
| `crates/mister-smith-agents/src/tool_bus.rs` | Defines the current operator-visible decision summary surface. |
| `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md` | Defines the already-landed bounded MCP baseline. |
| `docs/plans/2026-03-20-ms-95-post-merge-re-evaluation.md` | Freezes the active workflow-id status route. |

## Contract Mapping

The packet freezes these roles before implementation:

| Existing form | Contract role | Notes |
| ------------- | ------------- | ----- |
| request `delegation` on `POST /api/v1/tasks` | Accepted delegated ingress input | Existing transport-auth boundary input |
| metadata `external_delegation` | Persisted raw delegated ingress context | Baseline truth; not itself a first-class operator decision |
| `external_capability_decisions` | Preferred operator-visible decision surface | Reuse if possible for accepted ingress continuity |
| workflow-level autonomy status | Active operator inspection contract | `GET /api/v1/autonomy/status/{workflow_id}` |
| CLI autonomy status | Human-readable parity view | `mister-smith autonomy status --workflow-id ...` |
| retained session continuity | Compatibility rule | Must not fabricate ingress decisions from raw metadata alone |

## Entities

### PersistedExternalDelegationContext

Already-landed raw delegated ingress context stored in workflow metadata.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| capability | `ExternalDelegationEnvelope.capability` | Required when ingress is delegated | Bounded capability carried across the HTTP boundary |
| provenance | `ExternalDelegationEnvelope.provenance` | Required when ingress is delegated | Provenance chain for the delegated capability |
| action | `ExternalDelegationEnvelope.action` | Optional in the raw envelope | Typed delegated action at the boundary |

**Invariant**: persisted raw delegation context is evidence, not by itself an allowed or rejected
operator-visible boundary decision.

---

### ExternalCapabilityDecisionSummary

Preferred existing operator-visible summary surface.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| outcome | existing enum | Required | Allowed or rejected boundary decision |
| capability_descriptor_id | existing field | Optional | Descriptor seen at the boundary |
| action_descriptor_id | existing field | Optional | Requested action descriptor |
| action_id | existing field | Optional | Stable action id |
| required_scope | existing field | Optional | Required scope at the boundary |
| policy fields | existing fields | Optional | Policy binding carried by the boundary action |
| rationale | existing field | Required | Human-readable continuity explanation |

**Preferred invariant**: accepted ingress continuity reuses this summary shape if it can express
the accepted task-ingress boundary without ambiguity.

**Conditional invariant**: add a backward-compatible discriminator or shape extension only if
implementation research proves the existing summary cannot distinguish accepted ingress decisions
from outbound ToolBus decisions without ambiguity.

---

### WorkflowBoundaryContinuityView

The workflow-level operator-visible projection for one accepted delegated task-ingress workflow.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Returned identifier from accepted task ingress |
| persisted_ingress_context | derived | Required | Derived from stored raw delegated ingress metadata |
| operator_visible_decision | `ExternalCapabilityDecisionSummary` or compatible view | Required | First-class accepted boundary decision |
| provenance_lines | `Vec<String>` | Required | Compact explanation of provenance and policy continuity |

**Invariant**: this view is inspectable through workflow-level autonomy status and CLI parity.

---

### RetainedSessionContinuity Rule

Compatibility rule for session-facing continuity.

| Rule | Description |
| ---- | ----------- |
| No fabrication | Raw metadata-only delegated ingress context must not fabricate an accepted or rejected decision |
| No relabeling | Session continuity remains session continuity; it is not described as an autonomy-status surface |
| Compatibility only | Packet `016` may preserve compatibility with retained session continuity logic without widening live proof to delegated session ingress |

---

### IngressBoundaryProofRun

Durable proof record for one accepted delegated live task-ingress run.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| run_id | `Uuid` | Required | Stable proof identifier |
| request_surface | `String` | Required | Must be `POST /api/v1/tasks` |
| workflow_id | `TaskId` | Required | Returned workflow identifier |
| status_route | `String` | Required | Must be `GET /api/v1/autonomy/status/{workflow_id}` |
| cli_command | `String` | Required | `mister-smith autonomy status --workflow-id ...` |
| artifact_path | `String` | Required | Durable proof note path under `docs/plans/` |

**Invariant**: one proof run ties the accepted delegated request to both HTTP and CLI inspection.

## Relationships

```text
Accepted delegated POST /api/v1/tasks
  -> PersistedExternalDelegationContext
  -> WorkflowBoundaryContinuityView
  -> workflow-level autonomy status
  -> CLI autonomy status

PersistedExternalDelegationContext
  != operator-visible decision by itself

ExternalCapabilityDecisionSummary
  -> preferred operator-visible decision carrier
  -> may gain a backward-compatible discriminator only if ambiguity is proven
```

## Lifecycle Rules

### Accepted ingress continuity lifecycle

`delegated HTTP task request` -> `TaskSubmissionRequest.delegation` ->
persisted metadata `external_delegation` -> accepted workflow `workflow_id` ->
workflow-level autonomy projection -> CLI parity

Notes:

- the accepted workflow is the proof anchor
- the workflow-level route is the authoritative operator surface
- retained session continuity rules stay compatible but are not the primary live-proof path

### Rejection validation lifecycle

`missing or wrong or revoked or mismatched authority` -> deterministic transport/runtime rejection
tests -> packet proof record

Notes:

- deterministic rejection tests stay in scope
- live rejection proof stays out of scope unless implementation proves a workflow-backed reject
  surface already exists

## Identifier Guarantees

- `workflow_id` remains the canonical identifier for accepted live proof
- the same `workflow_id` must correlate:
  - the accepted delegated task-ingress response
  - workflow-level autonomy status
  - CLI autonomy status
  - the durable proof artifact
