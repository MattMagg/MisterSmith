# 2026-03-22 Runtime Planner/Compiler Evaluation

## Summary

This is a bounded live evaluation of the current Mister Smith runtime planning path on `main`.
Scope was limited to real `POST /api/v1/tasks` runs plus the minimum code inspection needed to
explain planner/runtime/compiler behavior. No implementation changes were made.

Bottom line:

- the current runtime still accepts explicit planner graphs too early
- the current explicit-graph normalization preserves semantically bad join metadata instead of
  coercing or rejecting it
- I did not reproduce a fresh `unsupported planner role 'join'` or `'joiner'` failure in these four
  runs, but the earlier failure remains credible because unsupported roles still pass through
  normalization until `TopologyCompiler::compile`
- the current head can distinguish simple sequential tasks from branchable tasks
- I do not have evidence from this run set that simple prompts are currently being forced into
  parallel graphs

Artifact lane:

- `docs/plans/artifacts/2026-03-22-runtime-planner-compiler-evaluation/`

## Baseline

- repo cwd: `/Users/macmain/MisterSmith`
- provider/model: `openai_chatgpt` / `gpt-5.4`
- runtime path: `cargo run -q -p mister-smith-app -- run`
- temporary database: `mistersmith_runtime_planner_eval_20260322`
- HTTP port: `63140`
- infra: `deploy-postgres-1` and `deploy-nats-1` already healthy on `5432` and `4222`

## Deterministic Validation

Preflight completed before live runs:

- `cargo build -p mister-smith-app`
- `cargo test -p mister-smith-app normalize_runtime_plan_ -- --nocapture`
- `cargo test -p mister-smith-app synthesize_failed_before_graph_status_preserves_hybrid_fanout_join_width -- --nocapture`
- `cargo run -q -p mister-smith-app -- auth openai-chatgpt status`

Results:

- build passed
- targeted runtime-plan and failed-before-graph tests passed
- ChatGPT auth was valid

Artifacts:

- `test-normalize-runtime-plan.txt`
- `test-failed-before-graph.txt`
- `manual-env.txt`

## Findings

### P1. Explicit graphs are accepted and published before topology compilation

Severity: high

Why this matters:

- the runtime persists `planner_output` and `execution_plan`
- then emits `workflow.planned`
- only after that does it call `TopologyCompiler::compile`

That ordering explains the operator-visible failure mode from the earlier session: a workflow can
look planned even when its normalized graph is still invalid and has never compiled.

Code anchor:

- `crates/mister-smith-app/src/execution.rs:543-576`

Evidence:

- code path persists and publishes the planned event before compile
- earlier session failure: `WORKFLOW.PLANNED` followed by `execution graph compile failed:
  Unsupported topology contract: unsupported planner role 'join'`

### P1. Explicit join-shaped graphs keep semantically wrong join metadata

Severity: high

Why this matters:

- `normalize_explicit_runtime_steps` only fills in `role` and `branch` when they are missing
- if the planner emits a join step with `role: "worker"` or with a non-join branch label, the
  runtime keeps it unchanged
- the compiler accepts that graph because `worker` is a supported role, so the malformed plan is
  treated as valid even though it encodes join semantics inconsistently

Code anchors:

- `crates/mister-smith-app/src/execution.rs:1732-1822`
- `crates/mister-smith-agents/src/topology.rs:445-464`

Run evidence:

- Run 2 accepted a five-step fanout/join plan where the synthesize step had
  `branch: "join"`, `depends_on` on three branches, but `role: "worker"`
- Run 4 accepted a three-step fanout/join plan where the merge step had two dependencies but kept
  `branch: "branch-a"` and `role: "worker"`
- both runs completed as hybrid graphs instead of being normalized or rejected earlier

Artifact anchors:

- `run2-task-status.json`
- `run2-autonomy-status.json`
- `run4-task-status.json`
- `run4-autonomy-status.json`

### P2. Unsupported synthetic roles are still only caught at compile time

Severity: medium

Why this matters:

- `TopologyCompiler::parse_agent_type` rejects unknown roles such as `join`
- the explicit-graph normalization path does not sanitize existing role strings
- that leaves the system vulnerable to the same late compile failure when the planner emits a
  synthetic role instead of a supported agent type

Code anchors:

- `crates/mister-smith-app/src/execution.rs:1761-1819`
- `crates/mister-smith-agents/src/topology.rs:445-464`

Evidence:

- current live runs did not re-emit `join`/`joiner`
- prior session already produced a real runtime failure with `unsupported planner role 'join'`
- repo memory and existing tests still encode the same failure family around unsupported join-style
  roles

### P3. The runtime can currently distinguish simple sequential tasks from branchable tasks

Severity: low

Why this matters:

- this reduces the scope of the patch plan
- the evidence does not justify a broader planner redesign or a general anti-decomposition patch

Run evidence:

- Run 1, explicit sequential repo analysis: four-step strict chain, sequential topology
- Run 3, trivial prompt `Reply with exactly READY.`: one-step sequential plan,
  `proof_outcome=collapsed_to_sequential`
- Runs 2 and 4, branchable prompts: hybrid fanout/join plans with width `3` and `2`

Code note:

- the hard-coded `execution_contract` in the planning context still asks for parallel workers and a
  join step for every task, but this run set did not prove that it currently forces parallelization
  on obviously sequential prompts

Code anchor:

- `crates/mister-smith-app/src/execution.rs:512-531`

## Run-By-Run Evidence

### Run 1: Sequential repo-analysis prompt

- task id: `9e5362f4-1576-4818-bf85-5002a648b049`
- request: sequential inspection of `/Users/macmain/hi-pro-ops-dash`, explicitly saying not to
  split into branches
- result:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - planner steps: `4`
  - execution steps: `4`
  - topology: `Sequential`
  - shape: `strict-chain`
- interpretation:
  - no branch/join over-decomposition
  - planner produced a chain of focused steps rather than a single step

Artifacts:

- `run1-task-request.json`
- `run1-submit-response.json`
- `run1-task-status.json`
- `run1-autonomy-status.json`

### Run 2: Branchable repo-review prompt

- task id: `6eb824c4-4719-40e2-9ad4-0765eb82a108`
- request: split review into package scripts, deployment config, and API/runtime tracks, then
  synthesize
- result:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - planner steps: `5`
  - topology: `Hybrid`
  - width: `3`
  - shape: `fanout-join`
- malformed detail:
  - join step `synthesize-operator-note` kept `role: "worker"` despite three dependencies and
    `branch: "join"`
- interpretation:
  - branchable workload behavior is live and functional
  - explicit join semantics are normalized too loosely

Artifacts:

- `run2-task-request.json`
- `run2-submit-response.json`
- `run2-task-status.json`
- `run2-autonomy-status.json`

### Run 3: Obvious single-threaded prompt

- task id: `e614b86a-14a6-4354-811e-f41e09aafbc9`
- request: `Reply with exactly READY.`
- result:
  - `status=completed`
  - `proof_outcome=collapsed_to_sequential`
  - planner steps: `1`
  - topology: `Sequential`
  - width: `1`
  - shape: `strict-chain`
- interpretation:
  - the current head still has an honest sequential collapse path
  - this run does not support a patch that broadly suppresses planner decomposition

Artifacts:

- `run3-task-request.json`
- `run3-submit-response.json`
- `run3-task-status.json`
- `run3-autonomy-status.json`

### Run 4: Coding-workflow style prompt with explicit final join

- task id: `6087ec95-b593-408c-ad9e-1c5c36f37c76`
- request: two parallel workers plus one final join-style patch recommendation
- result:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - planner steps: `3`
  - topology: `Hybrid`
  - width: `2`
  - shape: `fanout-join`
- malformed detail:
  - merge step `join-patch-recommendation` had two dependencies but kept `role: "worker"` and
    `branch: "branch-a"` instead of a join branch
- interpretation:
  - this is fresh proof that malformed branch/join metadata can survive normalization and still
    compile

Artifacts:

- `run4-task-request.json`
- `run4-submit-response.json`
- `run4-task-status.json`
- `run4-autonomy-status.json`

## Root-Cause Assessment

Primary root cause:

- the current explicit-graph path treats any planner plan with branch labels or dependencies as
  already-usable runtime graph input
- that path only fills missing fields and does not enforce join-step invariants

Secondary root cause:

- the runtime announces `workflow.planned` before compile-time contract validation succeeds

Specific code-level assessment:

1. `normalize_runtime_plan` preserves explicit graphs whenever any step already includes `branch`
   or `depends_on`.
2. `normalize_explicit_runtime_steps` does not coerce existing `role` or existing `branch` even
   when `depends_on.len() > 1` clearly indicates join semantics.
3. `TopologyCompiler::parse_agent_type` catches only unsupported role strings, not join-step
   semantic mismatches such as `role=worker` on a multi-dependency merge node.
4. The live planning context still globally asks for two parallel workers and a join step, which
   may bias the planner, but this run set does not justify a broader behavioral redesign.

## Patch Plan

Only evidence-backed patches are included.

### Patch 1: Coerce explicit join-shaped steps to valid runtime join semantics

Goal:

- normalize explicit planner graphs so multi-dependency merge steps cannot retain incompatible
  role/branch metadata

Likely files:

- `crates/mister-smith-app/src/execution.rs`

Patch shape:

- in `normalize_explicit_runtime_steps`, when a step has `depends_on.len() > 1`:
  - force `role` to `coordinator`
  - force `branch` to `join` or `join-N`
- optionally treat an existing `branch` starting with `join` as a join hint even when the planner
  left the role as `worker`
- keep this bounded to current runtime semantics; do not redesign topology selection

Why justified:

- Run 2 and Run 4 both preserved semantically bad join metadata

### Patch 2: Reject unsupported planner roles before publishing `workflow.planned`

Goal:

- stop surfacing a workflow as planned when the runtime graph has not passed contract validation

Likely files:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-agents/src/topology.rs`

Patch shape:

- move topology compilation or a smaller explicit validation pass ahead of the `workflow.planned`
  event emission
- if compile/validation fails, surface a planning-validation failure directly instead of first
  publishing a seemingly valid planned graph

Why justified:

- the earlier session failure happened after `WORKFLOW.PLANNED`
- the current code path still preserves that ordering defect

### Patch 3: Add explicit topology-contract validation for join semantics

Goal:

- distinguish malformed branch/join plans from valid branchable plans even when every role string is
  technically supported

Likely files:

- `crates/mister-smith-agents/src/topology.rs`
- possibly `crates/mister-smith-app/src/execution.rs` if validation is kept at the runtime
  normalization boundary

Patch shape:

- add a bounded check that a node with multiple dependencies must compile as a coordinator/join
  node under current runtime expectations
- reject or coerce inconsistent explicit graphs instead of silently accepting them as normal worker
  steps

Why justified:

- Run 2 and Run 4 were malformed enough to be misleading but still compiled

### Patch 4: Clarify operator-facing failure reason when planning succeeded but compile failed

Goal:

- make the operator-visible error say exactly which stage failed and why

Likely files:

- `crates/mister-smith-app/src/execution.rs`
- any typed result/provenance helper touched by failed-before-graph status assembly

Patch shape:

- keep the current `failed_before_graph` proof outcome
- make the surfaced error distinguish:
  - planner returned output
  - runtime normalization produced an execution plan
  - topology compile rejected the plan due to unsupported role or invalid join semantics

Why justified:

- earlier failure wording was technically accurate but operationally incomplete
- this patch is bounded to current error clarity, not observability expansion

## Validation Plan

### For Patch 1

- add normalization tests covering:
  - explicit join step with `role: "worker"` and `branch: "join"`
  - explicit multi-dependency step with `role: "worker"` and non-join branch
- rerun:
  - `cargo test -p mister-smith-app normalize_runtime_plan_ -- --nocapture`
  - the current Run 2 and Run 4 style prompts against live runtime

Expected proof:

- normalized execution plans show `role=coordinator` and `branch=join*` on merge nodes

### For Patch 2

- add a test around the task execution path proving invalid explicit roles do not emit
  `workflow.planned` before rejection
- rerun a live prompt known to trigger `join` or `joiner` role emission if available

Expected proof:

- invalid planner output fails before planned-state publication, with explicit stage labeling

### For Patch 3

- add compiler or runtime-validation tests for multi-dependency nodes with inconsistent branch/role
  combinations
- rerun the Run 4 style prompt live

Expected proof:

- malformed join graphs are either coerced into the current valid form or rejected explicitly

### For Patch 4

- add focused failed-before-graph tests for compile rejection wording
- rerun an invalid-role live case once Patch 2 is in place

Expected proof:

- task/autonomy failure text names planning success plus topology compile failure distinctly

## Non-Goals

- no planner architecture rewrite
- no new planning capabilities
- no future-state roadmap or frontier design
- no Langfuse, sidecars, or broader observability work
- no frontend/operator-console redesign
- no Linear or Symphony workflow changes
- no attempt to redesign how many steps the planner should use for a successful sequential task

## Cleanup

- runtime and task artifacts were kept under
  `docs/plans/artifacts/2026-03-22-runtime-planner-compiler-evaluation/`
- runtime process and temporary database should be removed after this note is finalized
