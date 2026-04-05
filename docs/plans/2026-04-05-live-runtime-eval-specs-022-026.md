# 2026-04-05 Live Runtime Evaluation Against Specs 022-026

## Summary

This note captures a fresh live runtime evaluation on current `main` against packet specs `022`,
`023`, `024`, `025`, and `026`, including the repair pass after the initial live timeout failure
was debugged.

The live matrix is frozen to three packet-015 proof labels only:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

The prompts for this evaluation are intentionally domain-neutral and do not ask the runtime to work
on Mister Smith, the repo, or internal development workflow tasks.

Primary artifact lane:

- `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/`

## Baseline

- Repo state at start: clean `main`
- Provider/model under test: `openai_chatgpt` / `gpt-5.4`
- Supported live surfaces under test:
  - `POST /api/v1/sessions`
  - `POST /api/v1/sessions/{session_id}/turns`
  - `GET /api/v1/sessions/{session_id}`
  - `GET /api/v1/tasks/{task_id}`
  - `GET /api/v1/autonomy/status/{workflow_id}`
  - `mister-smith autonomy status --workflow-id <id>`
- Frozen prompt set:
  - success probe: fictional city festival coordination memo with three independent tracks plus one
    final synthesis step
  - collapse probe: `Reply with exactly READY.`
  - delegation probe: fictional museum reopening memo with three parallel specialist tracks plus
    one final joiner step

## Deterministic Validation

All deterministic checks passed before the live attempts:

- `cargo build --workspace`
- `cargo test -p mister-smith-app --test autonomy_status_tests --test effect_boundary_projection_tests`
- `cargo test -p mister-smith-events --test autonomy_event_tests`
- `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- `git diff --check`
- `cargo test -p mister-smith-llm app_server_run_completion_handles_notifications_before_turn_start_response -- --nocapture`
- `cargo test -p mister-smith-agents supervise_allows_provider_target_before_graph_registration -- --nocapture`
- `cargo test -p mister-smith-llm --features openai-chatgpt openai_chatgpt_provider_completes_against_live_codex_app_server -- --ignored --nocapture`

## Debugging Result

The initial live failure was not a bad route and it was not a planner-only problem.

The repaired root cause is now clear:

- the supported path reached live `turn/start` acceptance
- the `CodexAppServerClient` request loop could re-queue unrelated notifications and then consume
  the same notification again instead of continuing to read stdout
- when the Codex app-server emitted startup or MCP notifications ahead of the matching response,
  the Rust client could livelock inside the buffer loop

The repair that unlocked the live path:

- `crates/mister-smith-llm/src/app_server.rs`
  - fixed matching-response lookup so request handling scans buffered messages for the right
    response id and otherwise keeps reading stdout
  - added trace markers for `thread/start`, `turn/start`, and `turn/completed`
- `crates/mister-smith-llm/tests/app_server_tests.rs`
  - added a regression test that emits `mcpServer/startupStatus/updated` before the
    `turn/start` response and proves `run_completion()` does not hang
- `crates/mister-smith-agents/src/orchestrator.rs`
  - preserved provider-target supervision before graph registration so provider-only retries do
    not fail with `No execution graph found`
- `crates/mister-smith-llm/tests/integration/openai_tests.rs`
  - aligned the ignored live Codex-app-server test with the supported runtime path by using
    `gpt-5.4` and a `120_000ms` timeout budget

## Live Runs

### Discarded Early Manual Lane

- Artifact: `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/20260405T105940Z/`
- Status: discarded from final scoring
- Reason: this first manual lane used the wrong assumption about active read surfaces and cleaned
  up immediately after the first accepted turn

### Manual Session Lane

- Artifact: `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/20260405T110634Z/`
- Prompt: neutral city-festival coordination task
- Observed facts:
  - runtime reached ready state
  - `POST /api/v1/sessions` accepted the turn and returned
    `session_id=f8c63a97-7086-478d-ac95-12da008c83e9`
  - turn 1 `workflow_id=e4883f3f-4c7e-4fa2-9c6d-57d6a32fdaec`
  - `GET /api/v1/sessions/{session_id}` timed out while the workflow was active
  - runtime log showed startup plus actor start, but no later terminal proof markers

### DB-Polled Session Lane

- Artifact: `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/20260405T111004Z/`
- Prompt: same neutral city-festival coordination task
- Observed facts:
  - runtime reached ready state
  - session turn accepted with
    `session_id=e0d1e63a-387e-4453-8e10-50c15a691e17`
  - turn 1 `workflow_id=3ee063f0-fce8-45b4-a12e-e7d85e3a060f`
  - root workflow row reached `running`
  - after a bounded wait, the root row still had no persisted `autonomy_status` and no persisted
    durable workflow history snapshot
  - no child tasks appeared under `tasks.records`

This lane was manually stopped so the repo was not left with a hanging live runtime.

### Repo-Owned Smoke Harness Lane

- Artifact:
  `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/harness-baseline-ready/20260405T111439Z/`
- Prompt: `Reply with exactly READY.`
- Status: repo-owned live harness failed
- Observed facts:
  - runtime reached ready state
  - `POST /api/v1/tasks` accepted the task and returned
    `task_id=68b48c0d-630e-4022-80d5-52ded2eacceb`
  - `task-poll.log` recorded `queued -> running`
  - after the task entered `running`, `GET /api/v1/tasks/{task_id}` timed out inside the
    repo-owned harness
  - direct Postgres inspection after the harness failure still showed:
    `status=running`, `started_at=true`, `completed_at=false`,
    `has_autonomy_status=false`, `has_durable_workflow_history=false`

This harness failure is the cleanest repo-owned live proof artifact from the session.

### Repaired Manual Traced Lane

- Runtime port: `63189`
- Prompt: `Reply with exactly READY.`
- Task ids:
  - `1a0ad381-33cd-464e-9f95-ae0273d01549`
  - `c6d150cb-8329-488f-a86b-53b5cc30caa2`
- Observed facts:
  - traced logs showed the planner path reaching:
    `openai_chatgpt provider starting completion` ->
    `codex app-server thread ready` ->
    `codex app-server turn accepted` ->
    `codex app-server turn completed`
  - both repeated live tasks reached terminal `completed`
  - the task surface returned bounded terminal results again on the supported live path

### Repaired Repo-Owned Smoke Harness Lane

- Artifact:
  `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/repaired-baseline-ready/20260405T180349Z/`
- Prompt: `Reply with exactly READY.`
- Status: repo-owned live harness passed
- Observed facts:
  - `POST /api/v1/tasks` accepted
    `task_id=f08ffacc-f7b8-429d-9c39-cf25e6f7818f`
  - `task-poll.log` recorded `queued -> running -> completed`
  - `autonomy-status.json` captured:
    - `graph.state=Completed`
    - `topology_kind=Sequential`
    - `branch_count=1`
    - `node_count=1`
    - one `step_routing_history` record with `tier=direct` and `action=continue`
  - `task-status-latest.json` captured:
    - terminal `result`
    - packet-023 `runtime_truth`
    - packet-026 proof boundary with honest sequential-collapse wording
  - direct Postgres inspection after completion showed:
    - `status=completed`
    - `started_at=true`
    - `completed_at=true`
    - `metadata.autonomy_status=true`
    - `metadata.final_result=true`
    - no `workflow_history` key was observed on this baseline lane

## Evaluation Result

Bottom line: the repaired live path now proves packet `023` on a bounded sequential baseline run.
Packets `022`, `024`, `025`, and `026` still do not have a full fresh live pass in this session.

| Packet | Live result | Reason |
| --- | --- | --- |
| `022` | not proven live | the repaired lane reached terminal completion and persisted `autonomy_status` plus `final_result`, but this session still did not live-prove packet-022 replay recovery, lifecycle verbs, effect replay safety, or durable history semantics |
| `023` | live passed on the repaired baseline lane | the repaired task and autonomy payloads exposed packet-023 `runtime_truth`, placeholder-proof-boundary wording, and bounded `run_trace` relationships on the supported `openai_chatgpt` / `gpt-5.4` path |
| `024` | not exercised live | the neutral prompts did not cross an external capability boundary, so this session produced no meaningful live least-privilege or quarantine evidence |
| `025` | not proven live | the repaired lane exposed `step_routing_history`, but packet-025-owned `step_policy` was not present on the task or autonomy payloads captured in this run |
| `026` | not proven live | the repaired lane honestly collapsed to sequential and explicitly said `packet 026 real coordinator-subagent runtime not satisfied`; no live delegated child runtime was exercised |

Additional packet-026 note:

- current code does already carry packet-026 contract surfaces and deterministic coverage across
  core, events, app, and operator-console seams
- `docs/current-state.md` still describes packet `026` as the "next implementation-ready packet"
  rather than as landed code on `main`

That is a doc-truth gap, but not a live-proof win.

## Remaining Limits

What remains after the repair:

- the supported live path is healthy again for the bounded sequential baseline task
- packet `023` now has fresh live evidence on this repaired path
- packet `022` still needs a replay, lifecycle, or effect-boundary live rerun if it is going to
  earn a true packet-level live pass
- packet `024` still needs a safe boundary-crossing prompt or controlled capability lane
- packet `025` still needs a live run that actually projects packet-owned `step_policy`
- packet `026` still needs a real fan-out lane with visible coordinator-owned delegation and child
  state that survives to the read surfaces

This evaluation can now honestly claim:

- deterministic packet surfaces for `022` through `026` are present and locally validated
- packet `023` achieved a fresh bounded live pass on the repaired `openai_chatgpt` / `gpt-5.4`
  baseline lane
- packets `022`, `024`, `025`, and `026` did **not** achieve a full fresh live pass in this
  session

## Cleanup

- temporary runtimes on ports `63182`, `63183`, `63189`, and `63190` were not left listening
  after the evaluation and repair pass
- repo worktree now carries the repair code, test updates, and this evaluation note
