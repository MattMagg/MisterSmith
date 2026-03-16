# First Live Multi-Agent Runtime Proof

Date: March 15, 2026

## Objective

Define and deliver the first honest end-to-end runtime proof for Mister Smith: boot the required
local infrastructure, start the framework runtime, submit one simple multi-agent workflow through a
real runtime-backed surface, and verify operator-visible autonomy state plus terminal completion
evidence.

This proof must be split into two explicit tiers:

- Tier 1, required: model-agnostic runtime proof using the existing deterministic `MockProvider`
  path so the runtime, task submission, autonomy projection, and agent coordination are validated
  independently of vendor credentials or model behavior.
- Tier 2, optional until chosen: provider-backed runtime proof using an explicitly selected
  provider/model pair and the corresponding auth/config surface. Tier 2 must not be claimed or
  counted complete until the operator chooses that provider/model.

## Why This Is Next

Phase 10 closed the framework gate with targeted crate tests, deploy-asset validation, and
workspace build proof. The repo now has a runnable binary entry point and autonomy inspection
surface, but it does not yet have a checked-in live run path that proves the system can execute a
real workflow from a cold start.

Current evidence:

- `crates/mister-smith-app/src/main.rs` exposes `mister-smith run` and autonomy inspection
  commands.
- `crates/mister-smith-http/src/routes.rs` exposes `POST /api/v1/tasks` plus task and autonomy
  inspection routes.
- `crates/mister-smith-http/src/handlers.rs` still states that handlers use placeholder/mock data
  and `create_task` currently returns synthetic IDs without runtime-backed dispatch.
- `specs/012-phase10-frontier-autonomy/quickstart.md` validates gate scenarios through tests and
  sketches, not a live runtime smoke path.
- Phase 9 and the LLM crate are provider-neutral by design and already include a deterministic
  `MockProvider`, so model choice must be explicit before any provider-backed runtime claim.

## Scope

- Runtime bootstrap using the existing `mister-smith-app` binary
- Real task or workflow submission path for a minimal happy-path agent run
- Local infra prerequisites for that run: NATS/JetStream and PostgreSQL
- Operator/autonomy inspection proof using existing status surfaces
- Durable runbook and repeatable smoke automation for future cold starts
- Explicit provider/model selection gate for any non-mock runtime proof

## Candidate Surfaces

Primary path to prefer:

- wire the existing `POST /api/v1/tasks` surface to real runtime-backed task submission
- verify result retrieval via `GET /api/v1/tasks/{task_id}`
- verify autonomy projection via `mister-smith autonomy list` and `mister-smith autonomy status`

Fallback path only if the HTTP task surface is too incomplete:

- add a dedicated runtime smoke harness that submits directly into the scheduler/orchestrator while
  preserving operator-visible autonomy evidence

## Constraints

- Do not claim broad production readiness from this item; prove one deterministic happy-path run
  first.
- Extend existing runtime and HTTP surfaces before inventing a parallel control path.
- Keep the issue bounded to runtime proof, task submission, and operator visibility.
- Keep Tier 1 and Tier 2 separate in both implementation and reporting.
- Do not silently pick a real provider/model on behalf of the operator.
- Do not absorb deferred Phase 11+ items such as learned routing, speculative decoding, or local
  inference.

## Non-Goals

- Full production soak, chaos, or load testing
- New frontier routing policies or inference backends
- Queue-governance or smith MCP throughput changes unrelated to the runtime proof

## Milestones

### 1. Replace the placeholder submission gap

- Trace the current `POST /api/v1/tasks` path through the runtime.
- Wire it to a real scheduler/orchestrator-backed submission path, or document and implement the
  smallest honest direct smoke harness if the HTTP route cannot yet carry the workflow.

Validation:

- `cargo test -p mister-smith-http`
- `cargo test -p mister-smith-app`
- targeted assertions that task submission no longer returns mock-only state

### 2. Prove one live workflow run

- Start NATS/JetStream and PostgreSQL locally.
- Start the Mister Smith runtime.
- Submit one simple workflow that exercises at least planner/coordinator plus one worker path.
- Complete this first with the deterministic `MockProvider` path before attempting any real
  provider-backed run.
- Verify the workflow reaches a terminal state with stable task/workflow identifiers.

Validation:

- `cargo build --workspace`
- a repeatable smoke command or script added by this issue
- captured runtime output or structured logs showing task acceptance and completion

### 3. Prove operator-visible runtime state

- Verify the run appears in the autonomy projection surface.
- Confirm topology, branch state, and terminal completion can be inspected without diving into raw
  internals.
- Write a short runbook for future operators.

Validation:

- `cargo run -p mister-smith-app -- autonomy list`
- `cargo run -p mister-smith-app -- autonomy status --workflow-id <id>`
- docs validation for the new runbook/smoke instructions

### 4. Optional provider-backed proof after explicit selection

- Record the chosen provider and model explicitly in the runbook and validation output.
- Configure the matching auth path and runtime config.
- Re-run the smoke proof against the selected real provider/model.

Validation:

- provider-specific auth/config verification
- the same smoke procedure with the selected provider/model recorded in evidence

## Acceptance Criteria

- Starting the runtime with required local infra produces healthy process and transport state.
- A simple submitted task or workflow is tracked by the real runtime rather than a placeholder
  handler.
- The run produces a stable workflow or task ID that can be queried to observe progress and
  completion.
- The operator-facing autonomy surface reports the live run and exposes enough state to confirm the
  run happened.
- The repo gains a repeatable smoke procedure that a cold-start operator can follow.
- Tier 1 acceptance uses `MockProvider` or another deterministic provider-neutral path and is valid
  without choosing a vendor model.
- Any Tier 2 real-model claim names the exact provider/model pair and corresponding auth path.

## Initial Validation Target

- `cargo build --workspace`
- `cargo test -p mister-smith-http`
- `cargo test -p mister-smith-app`
- `cargo test -p mister-smith-agents`
- smoke procedure or script introduced by this work

## Stop Conditions

- Stop and narrow scope if the current HTTP route cannot honestly map to the runtime without a much
  larger API design change; in that case, land a dedicated direct smoke harness instead.
- Stop before attempting Tier 2 unless the operator has chosen the provider/model pair explicitly.
- Stop before expanding into new model-routing or inference features.
- Stop before claiming success without a real run against local NATS/JetStream and PostgreSQL.
