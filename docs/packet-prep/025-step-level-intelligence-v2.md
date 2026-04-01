# Packet 025: Step-Level Intelligence v2

## Packet Name

Step-level intelligence v2

## Why This Packet Exists

Packet `020` landed a real verifier-gated control loop. Packet `021` landed bounded supervision
evidence. That was the first honest slice. This packet exists to deepen those surfaces into a more
capable step-level control system without pretending the runtime already has fully grounded step
execution.

## Why This Stage Is Correct

`docs/direction.md` places stronger step-level intelligence in the `Next` band, after the current
substrate hardening work. That matches the repo truth:

- verifier and repair lineage now exist
- runtime routing profiles now exist
- supervision evidence now exists
- the next sensible extension is smarter step selection, verification, and escalation

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - packet-020-style verifier and repair lineage can appear on the supported result path
  - runtime metadata already carries `routing_policy`, `registered_provider_count`, and
    `budget_root`
  - step evaluations, clarification requests, and repair directives are first-class value objects
- Landed but not yet a full step-intelligence packet:
  - injected verifier-policy handling and repair-state transitions exist in the runtime
  - step-proof outcomes are projected into canonical result views
- Deterministic-only today:
  - packet-021 supervision evidence is landed and deterministically validated, but it is not yet a
    fresh new live default-path baseline
- Missing for this packet:
  - a narrow first-class step scoring and escalation contract
  - one honest default rule for when to keep, retry, clarify, downgrade, or escalate at step level
  - one stable split between placeholder step completion and grounded step proof

## Current Repo Grounding

### Live on the default runtime path now

- typed step evaluation records, clarification requests, and orchestration-quality views
- verifier-gated step decisions and local repair lineage
- runtime routing policy, provider count, and budget root on provenance surfaces
- predictive-supervision evidence is carried on task, autonomy, and operator views, but the newer
  packet-021 proof for that surface is still deterministic-only rather than a fresh live rerun

### Landed in repo but not yet one frozen step-intelligence packet

- step intelligence exists at workflow and handoff granularity, not full token or reasoning-step
  granularity
- there is a real ToolBus step boundary, but the current `workflow.execute_step` runtime tool still
  stamps placeholder completion metadata rather than grounded repo work
- verifier and repair surfaces are real substrate hooks, but they still sit above that placeholder
  step-completion boundary today

### Deterministically grounded but not yet a fresh live baseline

- packet-021 supervision evidence and proof-boundary projection are landed and deterministically
  validated
- the live supported-path proof surface still lags the deterministic packet-021 closure note

### Missing pieces

- richer step difficulty assessment
- explicit model escalation policy at step level
- token budgeting and early-abort guidance at step level
- narrower proof criteria for when a step is substrate-complete versus task-grounded

## High-Signal Repo Anchors

- `crates/mister-smith-core/src/autonomy.rs`
  - `StepEvaluationRecord`
  - `HandoffClarificationRequest`
  - `TaskResultView`
  - This is the current type contract for step evidence and projection.
- `crates/mister-smith-app/src/execution.rs`
  - `WorkflowStepTool`
  - `impl Tool for WorkflowStepTool`
  - `runtime_supervision_plan_for_repair_evaluation`
  - `runtime_repair_step_evaluation`
  - `gate_step_execution_result`
  - `build_step_evaluation_record`
  - `repair_state_from_evaluation`
  - `repair_budget_exhausted`
  - `verifier_gate_accepts_enabled_policy_and_records_step_evaluation`
  - `verifier_gate_rejects_enabled_policy_and_records_repair_directive`
  - `verifier_gate_clarifies_handoff_before_accepting_follow_up`
  - `verifier_gate_replans_from_checkpoint_with_preserved_failure_context`
  - `description_only_repair_step_records_runtime_generated_step_evaluation`
  - `runtime_supervision_plan_targets_node_for_description_only_repair_step`
  - This is the current runtime seam for step verification and repair.
- `crates/mister-smith-app/src/autonomy.rs`
  - `build_canonical_result_envelope`
  - `build_task_result_view`
  - `classify_proof_outcome`
  - This is the current result-surface seam that packet `025` must preserve.
- `crates/mister-smith-events/src/autonomy.rs`
  - `StepRoutingDecisionSummary`
  - `merge_operator_result_preview`
  - This is the current operator-facing step-routing projection seam.
- `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
  - This is the packet-020 closure note for verifier-gated orchestration.
- `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`
  - This is the packet-021 closure note for supervision evidence and proof boundaries.
- `docs/plans/2026-03-29-packet-021-live-evaluation.md`
  - This is the clearest note showing the remaining live-gap between deterministic closure and fresh
    supported-path proof.
- `docs/2026-03-28-session-context-report.md`
  - This is the clearest cold-start note for why placeholder `workflow.execute_step` completion is
    not enough to claim grounded task proof.
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - This is the strongest existing deterministic test anchor for the projected step-evidence
    surfaces.
- `scripts/tests/test_live_runtime_proof_smoke.py`
  - This is the strongest repo-owned smoke-harness guard for preserving supported-path proof
    markers while step-policy work gets sharper.

## Official Docs / Primary Sources

- [OpenAI Responses streaming events](https://developers.openai.com/api/reference/resources/responses/streaming-events/)  
  Why it matters: this is the canonical event taxonomy for item-added, delta, done, and
  completed events. Use it when step-event names or payload fields matter.
- [OpenAI streaming API responses](https://developers.openai.com/api/docs/guides/streaming-responses/)  
  Why it matters: official guide for typed SSE event flow on the Responses API.
- [OpenAI function calling guide: Streaming](https://developers.openai.com/api/docs/guides/function-calling/#streaming)  
  Why it matters: official guidance for tool-call streaming, argument deltas, and completion
  semantics on top of the Responses event stream.
- [OpenAI Responses create reference](https://developers.openai.com/api/reference/resources/responses/methods/create/)  
  Why it matters: official request contract for `stream`, tool use, and bounded per-response
  settings.
- [OpenTelemetry traces](https://opentelemetry.io/docs/concepts/signals/traces/)  
  Why it matters: step-level intelligence needs step-level observability, not only decision logic.

Treat the Responses API pages above as the official baseline for packet `025`. Do not backslide to
older Chat Completions streaming docs when freezing the first step-event contract.

## Research Findings That Matter

- The model-routing and streaming corpora both say the strong next gain is step-level control, not
  only task-level routing.
- BiPRM and related work support stronger step verification than the current coarse verifier seam.
- RSD-style escalation is the most relevant start-cheap-escalate pattern in the repo research.
- TALE and related token-budget work matter because overthinking and uncontrolled compute are part
  of the same step-quality problem.
- The papers should shape policy ideas, but the first packet slice should still stay heuristic and
  deterministic before it becomes judge-heavy or training-heavy.

## Best-Practice Guidance

- Let packet `023` own run-trace taxonomy and proof-boundary schema. Packet `025` should only own
  step scoring, escalation, retry, clarify, and downgrade policy on top of that truth surface.
- Keep the first slice narrow: step difficulty assessment, verifier escalation, and budget-aware
  step routing.
- Treat the Responses streaming-events reference as the event-name authority. Treat the create
  reference and function-calling guide as request-shape and tool-call semantics layered on top of
  that event surface.
- Treat the OpenAI Responses docs as event-shape and request-contract guidance, not as proof that
  the current Smith step boundary is already grounded.
- Separate step intelligence from PRM training programs. Start with policy and runtime hooks.
- Make each escalation decision observable and auditable.
- Keep repair lineage and clarification history tied to step IDs.
- Do not claim semantic step proof if the executed boundary is still placeholder-only.

## Likely Architecture Shape

- step-scoring layer on top of existing verifier and routing surfaces
- escalation policy that can keep, retry, clarify, downgrade, or escalate models per step
- token-budget hints or step-budget contracts carried alongside step evaluation
- streamed step events and proof-boundary projection into result views

## Risks / Constraints / Non-Goals

- Do not widen this into a training pipeline packet.
- Do not hide placeholder execution behind smarter routing language.
- Do not pull in full coordinator-runtime or interoperability scope here.
- Do not make packet `025` depend on benchmark claims before packet `023` style truth surfaces exist.

## Open Questions Before Spec Writing

- What is the smallest useful step-scoring contract?
- What is the first escalation policy: heuristic, judge-based, or small verifier model?
- How should token budgets be surfaced in runtime metadata?
- Which step states are safe to early-abort?
- How should grounded versus placeholder step evidence be displayed?

## Fixed Constraints Before Spec Writing

- Let packet `023` keep ownership of run-trace taxonomy and proof-boundary schema. Packet `025`
  should only add step scoring, escalation, retry, clarify, and downgrade policy on top.
- Keep the first slice heuristic and deterministic on top of existing verifier and repair surfaces
  before introducing training-heavy or judge-heavy designs.
- Keep placeholder step completion separate from grounded step proof in all result surfaces.
- Do not widen packet `025` into coordinator-runtime, benchmark, or interoperability work.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> packet `023` truth boundary -> packet `020` and `021`
closure notes -> current verifier/repair seams -> official Responses docs.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/packet-prep/023-runtime-truth-and-run-trace.md`
  - use to preserve the already-scoped run-trace and proof-boundary ownership split
- `docs/research-output/consolidated/01-model-routing-and-cost-optimization.md`
- `docs/research-output/consolidated/06-streaming-architecture.md`
- `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
- `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`
- `docs/plans/2026-03-29-packet-021-live-evaluation.md`
- `docs/2026-03-28-session-context-report.md`
- `crates/mister-smith-core/src/autonomy.rs`
  - start from `StepEvaluationRecord`, `HandoffClarificationRequest`, and `TaskResultView`
- `crates/mister-smith-app/src/execution.rs`
  - start from `runtime_supervision_plan_for_repair_evaluation`,
    `runtime_repair_step_evaluation`, `gate_step_execution_result`,
    `build_step_evaluation_record`, `repair_state_from_evaluation`, and the verifier/repair tests
- `crates/mister-smith-app/src/autonomy.rs`
  - start from `build_canonical_result_envelope`, `build_task_result_view`, and
    `classify_proof_outcome`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `scripts/tests/test_live_runtime_proof_smoke.py`
  - use to keep supported-path proof markers honest while the packet sharpens step policy
- `crates/mister-smith-app/src/execution.rs`
  - specifically read `WorkflowStepTool`, `impl Tool for WorkflowStepTool`, and the tests
    `verifier_gate_accepts_enabled_policy_and_records_step_evaluation`,
    `verifier_gate_rejects_enabled_policy_and_records_repair_directive`,
    `verifier_gate_clarifies_handoff_before_accepting_follow_up`,
    `verifier_gate_replans_from_checkpoint_with_preserved_failure_context`,
    `description_only_repair_step_records_runtime_generated_step_evaluation`, and
    `runtime_supervision_plan_targets_node_for_description_only_repair_step`
- use the packet-021 proof note and live-evaluation note as bounded supervision and repair proof,
  not as a fresh packet-025 live baseline
- only after the repo-local proof boundary and placeholder-step limits are clear, re-confirm the
  official docs and primary sources linked earlier
