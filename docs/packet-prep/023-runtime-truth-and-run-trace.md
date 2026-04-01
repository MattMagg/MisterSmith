# Packet 023: Runtime Truth and Run Trace

## Packet Name

Runtime truth and run trace  
Sharper framing: honest execution truth, proof boundaries, and operator-visible traceability

## Why This Packet Exists

The repo already exposes runtime provenance, result previews, orchestration quality, and
supervision evidence. But the March 28 session report shows the main truth gap clearly:

- graph completion is visible
- routing and provenance are visible
- the live step boundary is still `workflow.execute_step`
- a run can look successful at the orchestration-substrate layer without proving grounded task work

This packet exists to make the system honest about what really happened in a run.

## Why This Stage Is Correct

This belongs right after durable workflow work because durable history and lifecycle IDs give the
trace packet something canonical to project. It also belongs before broader autonomy claims,
because coordinator-runtime and interoperability work will only get harder to reason about if the
run-truth surface stays blurry.

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - workflow submission, graph completion, and terminal result projection
  - packet-020-style `orchestration_quality` on the supported result path
  - stable `workflow_id`, `session_id`, and `coordinator_agent_id` continuity
- Landed but not yet a full run-trace standard:
  - `MessageEnvelope` trace and correlation fields
  - transport trace-context helpers
  - event-bus synthesis of supervision and status views
- Deterministic-only today:
  - packet-021 `supervision_evidence` and proof-boundary projection are landed and
    deterministically validated, but the last fresh live default-path baseline is still the earlier
    packet-019 and packet-020 proof set
- Missing for this packet:
  - one canonical end-to-end run-trace taxonomy
  - one stable proof-boundary contract that cleanly separates substrate completion from grounded
    task completion

## Current Repo Grounding

### Live on the default runtime path now

- `workflow_id`, `session_id`, and `coordinator_agent_id` are durable runtime identifiers
- task and autonomy views already project `orchestration_quality`, and the same surfaces now carry
  packet-021 `supervision_evidence` even though the freshest proof for that newer surface is still
  deterministic-only
- operator console run detail already consumes bounded runtime proof surfaces
- the supported-path smoke harness already checks runtime markers and tool-bus completion markers,
  but that still does not turn placeholder step completion into grounded task proof
- packet-019 and packet-020 remain the last fresh live-proof baseline for the supported path

### Landed in repo but not yet one frozen run-trace contract

- `MessageEnvelope` already carries `message_id`, `correlation_id`, `trace_id`, and header-level
  trace-context helpers
- packet-015, packet-020, and packet-021 already created proof-boundary language and result views
- provenance exists, but not yet as one complete end-to-end run trace standard
- trace IDs exist in transport, but there is no repo-wide trace/span contract for graph, step,
  tool, handoff, and repair events
- operator truth and semantic task truth still diverge when placeholder step execution is used

### Deterministically validated, but not yet a fresh live baseline

- packet-021 supervision-evidence and proof-boundary projection are frozen in
  `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`
- the live evaluation note still records that one clean successful default-path run proving the full
  packet-021 surface was not yet established in the earlier March 29 evaluation pass

### Missing pieces

- one canonical run-trace model across task, session, autonomy, and operator surfaces
- explicit proof-boundary fields that distinguish substrate proof from grounded task proof
- first-class span or event-link semantics for fan-out, join, repair, and escalation
- retention and artifact rules for replayable run traces

### High-Signal Repo Anchors

- `crates/mister-smith-transport/src/envelope.rs`
  - `MessageEnvelope`
  - `inject_trace_context`
  - `extract_trace_context`
  - `extract_tracestate`
  - This is the current transport-level trace seam.
- `crates/mister-smith-core/src/autonomy.rs`
  - `ProofOutcomeClassification`
  - `UnifiedResultEnvelope`
  - `StepEvaluationRecord`
  - `OrchestrationQualityView`
  - `TaskResultView`
  - This is the current truth/projection contract surface.
- `crates/mister-smith-events/src/bus.rs`
  - `repair_lineage_ref_from_checkpoint_lineage`
  - `supported_task_path_proof_boundary`
  - supervision-evidence synthesis and merge logic
  - This is the current event-to-status truth reducer.
- `crates/mister-smith-app/src/execution.rs`
  - `WorkflowStepTool`
  - `impl Tool for WorkflowStepTool`
  - `runtime_execution_mode_with_context`
  - `capture_autonomy_status_metadata`
  - `current_supervision_evidence`
  - `workflow_step_tool_marks_payload_as_tool_bus_completed`
  - This is the current boundary where substrate completion can still outrun grounded task truth.
- `crates/mister-smith-events/src/autonomy.rs`
  - `StepRoutingDecisionSummary`
  - `merge_operator_result_preview`
  - This is the current operator-facing projection seam for run truth.
- `crates/mister-smith-app/src/autonomy.rs`
  - `build_canonical_result_envelope`
  - `build_task_result_view`
  - `classify_proof_outcome`
  - This is the current proof-outcome and result-envelope projection seam.
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - projection and rendering assertions for orchestration and supervision surfaces
- `scripts/tests/test_live_runtime_proof_smoke.py`
  - `test_summarize_task_status_extracts_runtime_markers_and_step_summaries`
  - `test_assert_task_summary_rejects_missing_tool_bus_marker`
  - These are the strongest smoke-harness guards for the supported-path truth boundary.
- `crates/mister-smith-events/tests/autonomy_event_tests.rs`
  - event-bus truth-synthesis assertions, including proof-boundary preservation
- `docs/plans/2026-03-19-live-run-trace-evaluation.md`
  - the clearest earlier proof note for the `workflow.execute_step` truth gap
- `docs/plans/2026-03-27-runtime-planning-simplification.md`
  - repair-probe artifact lane and runtime-owned repair record follow-up
  - This is the clearest live proof that packet-020-style repair provenance is now projected from
    runtime-owned state instead of planner output alone.
- `docs/plans/2026-03-29-packet-021-live-evaluation.md`
  - the clearest newer note showing the difference between deterministic packet closure and fresh
    live-path proof
- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/baseline/20260330T005745Z/`
  - `task-result-summary.json`
  - `autonomy-status.json`
  - These are the strongest concrete live artifacts for the current packet-021 proof gap.

## Official Docs / Primary Sources

### Primary Trace Sources

- [OpenTelemetry traces](https://opentelemetry.io/docs/concepts/signals/traces/)  
  Why it matters: the cleanest official model for spans, causal structure, links, and events.
- [OpenTelemetry context propagation](https://opentelemetry.io/docs/concepts/context-propagation/)  
  Why it matters: shows how trace IDs move across process and network boundaries.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)  
  Why it matters: the standard wire format for trace correlation across service boundaries.

## Research Findings That Matter

- The March 28 session report says the current graph is an execution graph, not a visible
  agent-to-agent dialogue graph.
- The same report makes the key honesty rule explicit: completed substrate work is not the same as
  grounded task completion.
- The streaming architecture corpus says streaming must be treated as a typed event pipeline, not
  raw token chunks.
- Packet-020 and packet-021 already separated orchestration quality from supervision evidence. This
  packet should extend that separation into run truth.

## Best-Practice Guidance

- Let packet `023` own the run-trace taxonomy and proof-boundary schema. Later packets should
  consume that contract instead of redefining it.
- Let packet `022` own durable event-history semantics. Packet `023` should project those durable
  identifiers honestly, not replace them with UI-only trace language.
- Keep packet `019` and packet `020` live-proof evidence separate from packet `021`
  deterministic-only closure evidence. A future spec should preserve that split explicitly.
- Keep one canonical run identifier and one trace root per workflow execution.
- Use parent-child spans and span links for fan-out, join, retry, repair, and restart edges.
- Treat OpenTelemetry and W3C tracing docs as taxonomy guidance for the repo contract, not as
  proof that Mister Smith already emits one complete span model today.
- Keep proof-boundary text first-class in result surfaces. Never bury it in logs.
- Separate semantic event logs from lossy UI streaming.
- Treat placeholder execution, simulated execution, and grounded external action as different
  proof states.
- Make the operator surface show what was proven, not just what was attempted.

## Likely Architecture Shape

- canonical run record rooted at `workflow_id`
- trace/span projection for graph nodes, branches, tool calls, handoffs, repairs, and supervision
- explicit proof-boundary block on task/session/operator views
- typed semantic event log that can be replayed without depending on the UI stream
- retained artifact bundle for cold replay of one run

## Risks / Constraints / Non-Goals

- Do not treat UI polish as runtime truth.
- Do not over-collect baggage or sensitive payloads just because tracing is added.
- Do not claim grounded execution until the step boundary is no longer placeholder-only.
- Do not require a full external observability platform before shipping the repo contract.

## Open Questions Before Spec Writing

- What is the minimum stable span/event taxonomy Mister Smith should freeze?
- How should branch fan-out and join be represented: parent-child spans, links, or both?
- Which fields belong in proof-boundary text versus machine-checkable trace metadata?
- What retention model should be used for replayable traces and artifacts?
- How should the operator surface distinguish substrate proof, grounded tool proof, and human proof?

## Fixed Constraints Before Spec Writing

- Keep packet `023` responsible for run-trace taxonomy and proof-boundary language. Later packets
  should consume that contract instead of redefining it.
- Keep substrate completion separate from grounded task proof. Do not let UI or graph completion
  language blur that line.
- Treat packet-021 supervision projection as landed on the default path but still only
  deterministically proven until a fresh live rerun exists.
- Do not make external observability tooling a prerequisite for freezing the repo-native truth
  contract.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> proof-boundary notes -> live-gap evidence -> code seams ->
official tracing standards.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/packet-prep/022-durable-workflow-core.md`
  - use to inherit durable identifiers and lifecycle assumptions before freezing trace contracts
- `docs/2026-03-28-session-context-report.md`
- `docs/plans/2026-03-27-runtime-planning-simplification.md`
- `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`
- `docs/plans/2026-03-19-live-run-trace-evaluation.md`
- `docs/plans/2026-03-29-packet-021-live-evaluation.md`
- `crates/mister-smith-transport/src/envelope.rs`
  - start from `MessageEnvelope`, `inject_trace_context`, and `extract_trace_context`
- `crates/mister-smith-core/src/autonomy.rs`
  - start from `UnifiedResultEnvelope`, `ProofOutcomeClassification`, `TaskResultView`, and
    `OrchestrationQualityView`
- `crates/mister-smith-events/src/bus.rs`
  - start from `repair_lineage_ref_from_checkpoint_lineage`,
    `supported_task_path_proof_boundary`, and preserved-evidence merge paths
- `crates/mister-smith-app/src/execution.rs`
  - start from `WorkflowStepTool`, `impl Tool for WorkflowStepTool`,
    `runtime_execution_mode_with_context`,
    `capture_autonomy_status_metadata`, and the tool-bus completion test
- `crates/mister-smith-events/src/autonomy.rs`
  - start from `StepRoutingDecisionSummary` and `merge_operator_result_preview`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- `scripts/live_runtime_proof_smoke.py`
  - start from the supported-path assertion helpers that separate runtime markers from grounded
    task proof
- `scripts/tests/test_live_runtime_proof_smoke.py`
  - start from the runtime-marker summary assertions before widening the supported-path claims
- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/baseline/20260330T005745Z/`
  - start from `task-result-summary.json` and `autonomy-status.json`
- `apps/operator-console/`
  - use only after the runtime truth contract is frozen, not as the primary source of truth
- only after the repo-local proof-boundary split is clear, re-confirm the official docs and
  primary sources linked earlier
