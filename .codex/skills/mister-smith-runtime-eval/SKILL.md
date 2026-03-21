---
name: mister-smith-runtime-eval
description: Use when the user wants a complete Mister Smith live runtime session or workflow evaluation with real execution, durable evidence capture, and honest task/session/autonomy proof instead of mock-only validation.
---

# Mister Smith Runtime Eval

Use this skill when the ask is to prove what the current Mister Smith runtime actually does on a
live run.

This is a runtime-evaluation skill, not an implementation skill and not a planning-only skill.

## When To Use

Trigger this skill when the user wants any of the following:

- a real runtime-backed session or task evaluation
- proof that task, session, and autonomy surfaces agree
- a fresh rerun of an earlier proof note on current `main`
- artifact capture for runtime logs, payloads, and result surfaces
- confirmation of success, collapse, or failure-visible behavior on the supported live path
- an honest comparison between deterministic checks and real runtime behavior

## Do Not Use

Do not use this skill when:

- the user only wants unit or integration tests
- the task is to write or revise a SpecKit packet
- the task is to implement code changes
- the task is only about Linear, Symphony, or watched-queue workflow state
- mock-only coverage is enough for the request

## Required Inputs

- the evaluation target:
  - one-shot task path
  - session path
  - or both
- the proof scope:
  - one targeted outcome
  - or the full three-case proof matrix

Default runtime assumption unless the user explicitly overrides it:

- provider: `openai_chatgpt`
- model: `gpt-5.4`

For direct runtime-proof asks, prefer the product/runtime path first. Do not require Linear or
Symphony unless the user explicitly asks for development-workflow tracking alongside the runtime
evaluation.

## Start Sequence

Read, in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. the active checkpoint or current evaluation note relevant to the ask
4. any packet or closure note directly named by the user

If the user is asking for a rerun of a known proof lane, also read the closest existing note under
`docs/plans/` before starting the live run.

Use `docs/current-state.md` to confirm the currently supported live surfaces. Today the supported
autonomy HTTP route is:

- `GET /api/v1/autonomy/status/{workflow_id}`

Do not rely on older session-turn autonomy routes unless current repo truth explicitly restores
them.

## Workflow

### 1. Freeze the evaluation scope

Decide whether the run is:

- a targeted rerun for one known gap
- a bounded two-case evaluation
- or the full three-case proof matrix

Freeze the proof-outcome taxonomy exactly:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

Do not invent new proof classes during the evaluation session.

### 2. Prepare the durable note and artifact lane

Before the live run, choose the note and artifact destination:

- `docs/plans/YYYY-MM-DD-<slug>.md`
- `docs/plans/artifacts/YYYY-MM-DD-<slug>/`

Use the reference playbook at
`references/runtime-evaluation-playbook.md` for the standard artifact set and note structure.

### 3. Run deterministic preflight

Run the narrowest honest deterministic bundle for the requested scope before starting the live
runtime. Typical examples:

- `cargo build --workspace`
- `cargo test -p mister-smith-app`
- packet-specific crate tests when the request is tied to a known packet lane
- `git diff --check`

Keep deterministic validation separate from live-proof claims.

### 4. Launch the isolated live runtime

Start a local runtime with isolated state:

- temporary database
- explicit HTTP port
- runtime log capture
- explicit provider and model

Verify readiness before sending traffic.

Use the actual app path, for example:

- `target/debug/mister-smith run`
- or `cargo run -q -p mister-smith-app -- run`

Choose the path that matches current repo practice and the user request.

### 5. Execute the live proof

For task-based proof:

- submit a real request to `POST /api/v1/tasks`
- capture `workflow_id`
- inspect task status
- inspect autonomy status by `workflow_id`
- cross-check with CLI autonomy status when useful

For session-based proof:

- create or reuse a real session
- send the live turns required by the evaluation
- capture `session_id`, turn count, retained assistant result, and the linked workflow ids

Always collect:

- request payloads
- task/session/autonomy responses
- runtime log excerpts
- the runtime execution mode fields
- the exact provider/model used

### 6. Evaluate honestly

State what happened on the supported live path, not what the packet or docs hoped would happen.

Required distinctions:

- deterministic validation passed or failed
- live runtime proof passed or failed
- success, collapse, and failure-visible cases reached or not reached
- route or contract drift discovered during the run

If a formerly reliable prompt no longer reproduces an outcome, say that explicitly instead of
forcing a false comparison.

### 7. Clean up

At the end of the run:

- stop the runtime
- drop temporary databases created for the evaluation
- leave the note and artifact bundle in a cold-start-replayable state

If the user did not ask for git closure, stop after honest cleanup of runtime state and a clear
report of any remaining worktree delta created by the evaluation artifacts.

## Guardrails

- Do not claim runtime proof from mocks, handler-only tests, or deterministic checks.
- Keep provider and model explicit in the note and final report.
- Use the current workflow-id autonomy route unless repo truth says otherwise.
- Keep the three proof-outcome labels frozen.
- Do not widen into implementation or backlog work unless the user explicitly asks.
- Redact secrets from captured env files, tokens, or delegated-request artifacts before keeping
  them.
- If the request is actually about the development workflow rather than the runtime product, switch
  to the appropriate Smith control-plane skill instead of forcing a runtime-eval flow.

## Related Skills

- [$mister-smith-control-plane-router](/Users/macmain/MisterSmith/.codex/skills/mister-smith-control-plane-router/SKILL.md)
  for repo workflow and control-plane routing
- [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md)
  when the user explicitly wants the evaluation artifacts landed and the repo cleaned end to end

## Reference

Read `references/runtime-evaluation-playbook.md` when you need:

- the standard note layout
- the default artifact bundle checklist
- the proof matrix checklist
- route reminders
- cleanup expectations
