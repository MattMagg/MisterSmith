# Data Model: Complex Multi-Agent Proof and Unified Result Surfaces

**Date**: 2026-03-19  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `crates/mister-smith-app/src/execution.rs` | Defines the existing `final_result`, nested `aggregated_result`, and task-result persistence shape. |
| `crates/mister-smith-app/src/conversation.rs` | Defines retained `assistant_result`, `last_assistant_result`, and transcript summary storage. |
| `crates/mister-smith-app/src/autonomy.rs` | Defines the current operator surface that needs a compact result preview and provenance block. |
| `crates/mister-smith-core/src/autonomy.rs` | Natural choke point for shared result and proof-outcome value objects. |
| `crates/mister-smith-events/src/autonomy.rs` | Natural choke point for typed operator-visible result summaries. |
| `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md` | Shows the need for inspectable final-result evidence after successful graph execution. |
| `docs/plans/2026-03-19-framework-comparison-stress-test.md` | Provides the three proof outcome classes the packet must encode. |

## Contract Mapping

The packet freezes the existing result forms into one explicit contract:

| Existing form | Contract role | Notes |
| ------------- | ------------- | ----- |
| metadata `final_result` | Canonical persisted runtime result object | Primary source of truth for terminal workflow result content and provenance |
| metadata `aggregated_result` | Execution-produced payload nested inside the canonical result object | Not a competing top-level result shape |
| `task.result` | Task-facing result envelope | Mirrors or derives directly from the canonical runtime result object |
| session `assistant_result` / `last_assistant_result` | Session-facing retained result projection | Derived from the canonical runtime result object |
| operator preview/provenance output | Compact operator-facing projection | Derived from the canonical runtime result object without full payload dump by default |

## Entities

### UnifiedResultEnvelope

Canonical runtime result object rooted at metadata `final_result`.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow that produced the result |
| provider_kind | `String` | Required | Live provider path used for the run |
| model_id | `String` | Required | Model used for the run |
| description | `String` | Required | Workflow request summary or description |
| runtime_execution_mode | `Value` | Required | Existing execution markers such as supervised actor and `tool_bus` |
| planner_output | `Value` | Required | Existing planner output captured by the runtime |
| execution_plan | `Value` | Required | Existing normalized execution plan captured by the runtime |
| step_results | `Vec<Value>` | Required | Existing per-step results captured by the runtime |
| aggregated_result | `Value` | Required | Execution-produced payload nested inside the canonical result object |
| proof_outcome | `ProofOutcomeClassification` | Required | Outcome class for proof and evaluation surfaces |

**Invariant**: `aggregated_result` is always nested inside `UnifiedResultEnvelope`.

**Invariant**: `UnifiedResultEnvelope` is the single authoritative source for task, session, and
operator result projections.

---

### TaskResultView

Task-facing result envelope exposed through `task.result`.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow identifier shown on the task surface |
| status | `String` | Required | Terminal workflow status |
| result | `UnifiedResultEnvelope` or equivalent derived view | Required for terminal states | Canonical result envelope exposed by the task surface |
| proof_outcome | `ProofOutcomeClassification` | Required | Task-facing outcome classification |

**Invariant**: `TaskResultView.result` must mirror or derive directly from the canonical
`UnifiedResultEnvelope` without dropping the nested `aggregated_result`.

---

### SessionRetainedResultView

Retained session-facing projection stored as `assistant_result`.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow linked to the retained turn |
| turn_index | `u32` | Required | Session turn that owns this projection |
| status | `String` | Required | Turn or workflow status |
| assistant_result | `Value` | Required | Session-facing projection derived from the canonical result object |
| preview | `Option<String>` | Optional | Compact preview extracted from the canonical result object |
| provenance | `ResultProvenanceSummary` | Required | Bounded provenance summary for retained context |

**Invariant**: `assistant_result` is derived from `UnifiedResultEnvelope`, not a separately
invented result shape.

---

### OperatorResultPreview

Compact operator-facing result preview and provenance block rendered alongside autonomy status.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow being inspected |
| proof_outcome | `ProofOutcomeClassification` | Required | Outcome class visible to operators |
| preview_text | `Option<String>` | Optional | Bounded result preview, omitted when not safe or available |
| payload_location | `String` | Required | Where the full result comes from, for example `task.result` or metadata `final_result` |
| provenance_lines | `Vec<String>` | Required | Compact explanation of how the result was produced and classified |

**Invariant**: `OperatorResultPreview` never requires dumping the full payload by default.

---

### ResultProvenanceSummary

Shared provenance block reused by session and operator projections.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| runtime_execution_mode | `Value` | Required | Execution-mode summary from the canonical result object |
| graph_state | `Option<String>` | Optional | Graph state if a graph was formed |
| graph_id | `Option<String>` | Optional | Graph identifier when available |
| source_fields | `Vec<String>` | Required | Canonical fields used to derive this projection |

**Invariant**: `source_fields` explicitly names the canonical result fields used to build the
projection.

---

### ProofOutcomeClassification

Explicit outcome taxonomy used across runtime, status, and evaluation surfaces.

| Value | Meaning |
| ----- | ------- |
| `graph_formed_and_completed` | The planner formed a real graph and the workflow reached terminal completion |
| `collapsed_to_sequential` | The workflow completed, but the planner compressed it to a trivial sequential path |
| `failed_before_graph` | The workflow failed before usable graph formation |

---

### EvaluationHarnessRun

Durable evidence record for one proof-matrix run.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| run_id | `Uuid` | Required | Stable comparison identifier |
| workload_class | `String` | Required | Human-readable workload class |
| proof_outcome | `ProofOutcomeClassification` | Required | Outcome class for the run |
| graph_formed | `bool` | Required | Whether a real graph formed |
| branch_count | `Option<usize>` | Optional | Branch count when a graph exists |
| result_preview | `Option<String>` | Optional | Bounded preview captured for later proof review |
| evidence_note_path | `String` | Required | Repo path for the durable evidence note under `docs/plans/` |

**Invariant**: every evaluation run maps to exactly one proof outcome class.

## Relationships

```text
UnifiedResultEnvelope 1 --- 1 TaskResultView
UnifiedResultEnvelope 1 --- N SessionRetainedResultView
UnifiedResultEnvelope 1 --- 1 OperatorResultPreview
UnifiedResultEnvelope 1 --- 1 ProofOutcomeClassification
EvaluationHarnessRun records workload_class + ProofOutcomeClassification + evidence note path
```

## Lifecycle Rules

### Result lifecycle

`step_results` -> `aggregated_result` -> `UnifiedResultEnvelope` -> task view ->
session retained view -> operator preview

Notes:

- `aggregated_result` is part of the canonical envelope, not a separate final-result contract
- projections may omit or compact payload details, but they must preserve enough provenance to map
  back to the canonical envelope

### Proof classification lifecycle

`submit workload` -> `planner outcome observed` -> `graph formed?` -> `terminal state observed` ->
`proof outcome classified` -> `evaluation artifact written`

Notes:

- a completed workflow may still classify as `collapsed_to_sequential`
- a failed workflow may classify as `failed_before_graph` even when task-level failure data exists

## Identifier Guarantees

- `workflow_id` remains the canonical identifier across task, session, operator, and evaluation
  surfaces
- `UnifiedResultEnvelope.workflow_id` must match the owning task, session projection, operator
  preview, and evaluation record
- the proof outcome class must be stable enough to appear in stored evidence and later review
