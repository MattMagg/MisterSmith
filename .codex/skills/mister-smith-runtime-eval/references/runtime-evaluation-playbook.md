# Runtime Evaluation Playbook

Use this playbook when running a full Mister Smith live runtime evaluation.

## Grounding Order

Read, in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` when the task is about proving
   real end-to-end execution
4. the active checkpoint or the closest current evaluation note
5. the specific packet, issue, or closure note named by the user

For direct runtime asks, stay on the product/runtime side unless the user explicitly asks for
Linear, Symphony, or other development-workflow tracking.

## Current Supported Live Surfaces

Verify current repo truth before the run. On the current live path, the important operator surfaces
are:

- task submission: `POST /api/v1/tasks`
- bounded session handling: `POST /api/v1/sessions` and related session routes
- autonomy inspection: `GET /api/v1/autonomy/status/{workflow_id}`
- CLI cross-check:
  - `mister-smith autonomy list --base-url http://127.0.0.1:<port>`
  - `mister-smith autonomy status --workflow-id <workflow_id> --base-url http://127.0.0.1:<port>`

If any older route appears in historical notes, verify it live before treating it as current.

## Proof Matrix

Freeze the evaluation taxonomy to these three labels:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

Use the full matrix when the user wants a complete runtime evaluation. Use a targeted subset only
when the user narrows scope explicitly.

## Standard Artifact Bundle

Create:

- `docs/plans/YYYY-MM-DD-<slug>.md`
- `docs/plans/artifacts/YYYY-MM-DD-<slug>/`

Recommended artifact files:

- `manual-env.txt`
- `runtime.log`
- request payloads such as `task-request.json` or `session-turn-*.json`
- `task-status.json`
- `session-inspect.json`
- `autonomy-status.json`
- CLI output captures when they materially confirm parity
- a short `README.md` if the bundle needs indexing for cold-start replay

Redact secrets before keeping any artifact.

## Runtime Setup Checklist

- choose a unique temporary database name
- choose a free HTTP port
- use explicit provider/model env settings
- capture stdout or structured logs to `runtime.log`
- verify runtime readiness before the first request

Typical runtime paths:

- `target/debug/mister-smith run`
- `cargo run -q -p mister-smith-app -- run`

Use the narrowest honest path that matches current repo practice.

## Deterministic Preflight

Run the smallest deterministic bundle that materially supports the evaluation. Common examples:

- `cargo build --workspace`
- `cargo test -p mister-smith-app`
- packet-specific crate tests
- `git diff --check`

Never blur this into the live-proof claim.

## What To Capture Per Live Run

Record:

- provider and model
- runtime execution mode fields
- `session_id` when present
- `workflow_id`
- task result and proof outcome
- retained session result and proof outcome
- autonomy result preview or failure
- runtime log lines that mark execution, collapse, or failure

For failure-visible runs, capture the exact error string from task, session, and autonomy surfaces
and note whether they match.

## Note Structure

Keep the durable note short and factual.

Recommended sections:

- `Summary`
- `Baseline`
- `Deterministic Validation`
- `Important Route Clarification` when needed
- `Live Runs`
- `Evaluation Result`
- `Remaining Limits`
- `Cleanup`

## Evaluation Rules

- State what happened on the current head, not what older notes expected.
- Note when an old prompt no longer reproduces collapse or failure.
- Record live-path improvements as improvements, not as missing proof.
- When the live path regresses, identify exactly which surface disagreed:
  - task
  - session
  - autonomy
  - CLI

## Cleanup

At the end:

- stop the runtime process
- drop the temporary database
- keep the artifact bundle replayable
- report any remaining worktree delta caused by the new note or artifacts

If the user also wants repo closure, hand off to
[`mister-smith-git-closure`](../../mister-smith-git-closure/SKILL.md).
