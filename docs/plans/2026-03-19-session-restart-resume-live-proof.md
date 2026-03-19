# Session Restart-Resume Live Proof

Date: March 19, 2026
Status: Completed

## Objective

Run one real restart-resume session proof on the current live runtime path and capture durable
evidence that:

- a session turn is accepted through the live HTTP surface
- the runtime is restarted after the first workflow reaches `running`
- the restarted runtime repairs the interrupted session state
- the same `session_id` and `coordinator_agent_id` continue into a second turn
- resumed lineage is exposed through the session inspect and autonomy surfaces

## Environment Used

- repo: `/Users/macmain/MisterSmith`
- branch at start: `main...origin/main` with a clean tracked worktree
- runtime binary: `target/debug/mister-smith`
- local infra used:
  - Docker `deploy-postgres-1` on host `5432`
  - Docker `deploy-nats-1` on host `4222`
- authenticated provider path:
  - `provider_kind = openai_chatgpt`
  - `model_id = gpt-5.4`
- manual live proof runtime:
  - base URL: `http://127.0.0.1:62695`
  - gRPC port: `62696`
  - temp database:
    `postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_manual_rr_20260319`

## Files Read For Grounding

- `AGENTS.md`
- `docs/current-state.md`
- `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`
- `docs/plans/2026-03-17-ms-67-restart-resume-proof.md`
- `crates/mister-smith-integration-tests/tests/conversation_restart_resume.rs`

## Commands Run

```bash
git status --short --branch
docker exec deploy-postgres-1 pg_isready -U mistersmith -h 127.0.0.1 -p 5432
docker exec deploy-nats-1 sh -lc 'nc -z 127.0.0.1 4222 && echo ok || echo fail'
cargo run -q -p mister-smith-app -- auth openai-chatgpt status

cargo test -p mister-smith-integration-tests \
  live_restart_resume_http_roundtrip_recovers_idle_session_and_resumed_lineage \
  -- --ignored --exact --nocapture

docker exec deploy-postgres-1 psql -U mistersmith -d postgres \
  -c "CREATE DATABASE mistersmith_manual_rr_20260319;"

env DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_manual_rr_20260319' \
  MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4222' \
  MISTER_SMITH_TRANSPORT__HTTP_PORT='62695' \
  MISTER_SMITH_TRANSPORT__GRPC_PORT='62696' \
  MISTER_SMITH_OBSERVABILITY__OTLP_ENDPOINT='' \
  target/debug/mister-smith run

curl -sS http://127.0.0.1:62695/health/ready
curl -sS -D docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/manual-session-create.headers \
  -H 'content-type: application/json' \
  --data @docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/manual-session-create-request.json \
  http://127.0.0.1:62695/api/v1/sessions
curl -sS http://127.0.0.1:62695/api/v1/tasks/cd1ee8b1-8508-4ded-9cd9-78aebb4d707a

# restart after first workflow reaches running

curl -sS http://127.0.0.1:62695/api/v1/sessions/e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a
curl -sS -D docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/manual-session-continue.headers \
  -H 'content-type: application/json' \
  --data @docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/manual-session-continue-request.json \
  http://127.0.0.1:62695/api/v1/sessions/e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a/turns
curl -sS http://127.0.0.1:62695/api/v1/sessions/e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a
curl -sS \
  http://127.0.0.1:62695/api/v1/autonomy/status/4da9de30-a603-4c7c-b9c6-84b49c20b611

cargo run -q -p mister-smith-app -- conversation inspect \
  --base-url http://127.0.0.1:62695 \
  --session-id e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a
cargo run -q -p mister-smith-app -- autonomy status \
  --base-url http://127.0.0.1:62695 \
  --workflow-id 4da9de30-a603-4c7c-b9c6-84b49c20b611

docker exec deploy-postgres-1 psql -U mistersmith -d postgres \
  -c "DROP DATABASE IF EXISTS mistersmith_manual_rr_20260319 WITH (FORCE);"
```

## Identifiers Produced

- session id: `e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a`
- coordinator agent id: `935325e0-946e-4e01-9b07-598bd786f884`
- first workflow id: `cd1ee8b1-8508-4ded-9cd9-78aebb4d707a`
- resumed workflow id: `4da9de30-a603-4c7c-b9c6-84b49c20b611`

## Artifacts Captured

All supporting evidence is under:

`docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/`

Key files:

- `binary-check.txt`
- `postgres-health.txt`
- `nats-health.txt`
- `openai-chatgpt-auth-status.txt`
- `test-output.txt`
- `first-runtime.log`
- `second-runtime.log`
- `manual-env.txt`
- `manual-first-runtime.log`
- `manual-second-runtime.log`
- `manual-session-create-request.json`
- `manual-session-create.headers`
- `manual-session-create-response.json`
- `manual-turn1-task-poll.log`
- `manual-turn1-task-latest.json`
- `manual-session-after-restart.json`
- `manual-session-inspect-after-restart-poll.log`
- `manual-session-continue-request.json`
- `manual-session-continue.headers`
- `manual-session-continue-response.json`
- `manual-session-final-poll.log`
- `manual-session-final.json`
- `manual-session-inspect-cli.txt`
- `manual-turn2-autonomy-status.json`
- `manual-turn2-autonomy-cli.txt`

## Observed Live Evidence

### Existing live harness still passes

- `test-output.txt` ends with the ignored live integration target completing successfully:
  `live_restart_resume_http_roundtrip_recovers_idle_session_and_resumed_lineage ... ok`
- `first-runtime.log` and `second-runtime.log` preserve the harness runtime logs used by that
  test.

### First live turn was accepted and reached `running`

- `manual-session-create.headers` shows `HTTP/1.1 202 Accepted`
- `manual-session-create-response.json` returned:
  - `session_id = e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a`
  - `task_id = cd1ee8b1-8508-4ded-9cd9-78aebb4d707a`
  - `assigned_agent_id = 935325e0-946e-4e01-9b07-598bd786f884`
- `manual-turn1-task-poll.log` shows the first workflow reaching `running` before restart
- `manual-turn1-task-latest.json` captured that live task state

### Restart repaired the interrupted session state

- after runtime shutdown and restart, `manual-session-after-restart.json` shows:
  - same `session_id`
  - same `coordinator_agent_id`
  - `last_completed_workflow_id = cd1ee8b1-8508-4ded-9cd9-78aebb4d707a`
  - turn 1 `status = failed`
  - `resume_provenance.recovered_after_restart = true`
  - `recovery_reason = workflow interrupted by runtime restart before session sync`
- `manual-session-inspect-after-restart-poll.log` captured the poll loop until that repaired state
  became visible

### Second turn continued on the same session lineage

- `manual-session-continue.headers` shows `HTTP/1.1 202 Accepted`
- `manual-session-continue-response.json` returned:
  - same `session_id = e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a`
  - same `assigned_agent_id = 935325e0-946e-4e01-9b07-598bd786f884`
  - new `task_id = 4da9de30-a603-4c7c-b9c6-84b49c20b611`
- `manual-session-final.json` shows:
  - `active_workflow_id = null`
  - `last_completed_workflow_id = 4da9de30-a603-4c7c-b9c6-84b49c20b611`
  - `turn_count = 2`
  - turn 2 `status = completed`
  - `resume_provenance.resumed_after_restart = true`
  - `resume_provenance.resumed_from_turn_index = 1`
  - `resume_provenance.resumed_from_workflow_id = cd1ee8b1-8508-4ded-9cd9-78aebb4d707a`
- `manual-session-inspect-cli.txt` renders the same recovery and resumed lineage in the operator
  CLI surface

### Autonomy surface preserved resumed lineage and completed graph state

- `manual-turn2-autonomy-status.json` shows:
  - `session_id = e5fcb6db-6b9c-491d-a9e4-4abbe6cebb7a`
  - `turn_index = 2`
  - same `coordinator_agent_id = 935325e0-946e-4e01-9b07-598bd786f884`
  - `resume_provenance.resumed_after_restart = true`
  - `resume_provenance.resumed_from_workflow_id = cd1ee8b1-8508-4ded-9cd9-78aebb4d707a`
  - `graph.workflow_id = 4da9de30-a603-4c7c-b9c6-84b49c20b611`
  - `graph.state = Completed`
  - `graph.active_topology = Hybrid`
  - `topology.parallelism_width = 2`
  - `topology.task_shape.kind = fanout-join`
- `manual-turn2-autonomy-cli.txt` renders the same workflow, session, resume provenance, and
  topology details on the CLI path

### Restarted runtime executed the resumed workflow on the same fixed provider path

- `manual-second-runtime.log` shows:
  - runtime execution service ready with `provider_kind = openai_chatgpt` and
    `model_id = gpt-5.4`
  - three step executions for workflow `4da9de30-a603-4c7c-b9c6-84b49c20b611`
  - terminal `Workflow completed` for that same workflow id

## What Was Proved

- The stronger run was not just prompt submission and a single reply. It proved live
  session-backed continuity across a runtime restart.
- The current `main` path can:
  - accept a real session turn through `POST /api/v1/sessions`
  - survive a runtime restart after the first workflow has begun running
  - repair the interrupted turn into explicit recovered state
  - continue the same session through a second turn
  - preserve the same `session_id` and `coordinator_agent_id`
  - expose resumed lineage through both session inspect and autonomy status surfaces
- The resumed workflow completed on the fixed provider/model path `openai_chatgpt` / `gpt-5.4`.
- The resumed workflow still exposed a completed hybrid fanout-join topology through the autonomy
  surface after restart.

## What Was Not Proved

- cross-host or distributed restart recovery
- process crash classes beyond the explicit local shutdown/restart exercised here
- provider failover or provider-neutral session continuity
- any new runtime feature beyond the already-implemented restart-resume contract

## Cleanup

- the manual second runtime was shut down after evidence capture
- the temporary database `mistersmith_manual_rr_20260319` was dropped
- the proof artifacts remain under
  `docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/`
