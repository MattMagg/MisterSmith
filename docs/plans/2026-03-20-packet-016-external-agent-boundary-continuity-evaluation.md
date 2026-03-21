# Packet 016 External-Agent Boundary Continuity Evaluation

Date: March 20, 2026
Status: Completed
Issues: `MS-99` live proof, `MS-100` final validation

## Objective

Capture one accepted delegated `POST /api/v1/tasks` proof run, verify the returned workflow
identifier on `GET /api/v1/autonomy/status/{workflow_id}` and CLI parity, and record whether a
workflow-backed live reject surface exists on the current runtime path.

## Scope And Repo Truth

- bounded to packet `016` task-ingress continuity and proof
- no widening into new delegated routes, provider work, queue work, or contract expansion
- the current runtime returns the root workflow identifier in the `task_id` field from
  `TaskSubmissionResponse`; `RuntimeTaskService::submit_task` sets `task_id: workflow_id` in
  `crates/mister-smith-app/src/execution.rs`
- the active workflow-level operator surfaces remain:
  - `GET /api/v1/autonomy/workflows`
  - `GET /api/v1/autonomy/status/{workflow_id}`
  - `mister-smith autonomy status --workflow-id ...`

## Environment Used

- repo workspace: `/Users/macmain/.local/share/symphony-workspaces/MS-99`
- branch: `codex/ms-99-accepted-ingress-proof`
- runtime base URL: `http://127.0.0.1:63140`
- database: `mistersmith_ms99_packet016_20260320`
- NATS URL: `nats://127.0.0.1:4222`
- security runtime config:
  - `MISTER_SMITH_SECURITY__ENABLED=true`
  - `MISTER_SMITH_SECURITY__AUTH_ENABLED=true`
  - `MISTER_SMITH_SECURITY__AUTH__ALGORITHM=HS256`
  - `MISTER_SMITH_SECURITY__AUTH__ISSUER=mister-smith-ms99-proof`
  - `MISTER_SMITH_SECURITY__AUTH__AUDIENCE=ms99-proof`
  - `MISTER_SMITH_SECURITY__AUTH__HMAC_SECRET=[redacted]`
- provider auth proof:
  - `cargo run -q -p mister-smith-app -- auth openai-chatgpt status`
  - artifact: `docs/plans/artifacts/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation/openai-chatgpt-auth-status.txt`

## Code Changes Required For Honest Proof

- `crates/mister-smith-app/src/bootstrap.rs`
  - attach `SecurityLayer` to `AppState` when runtime security is enabled so delegated HTTP proof
    uses the real auth boundary
- `crates/mister-smith-security/src/middleware/mod.rs`
  - map runtime security config into JWT, RBAC, audit, and TLS middleware config instead of
    silently falling back to defaults
- `crates/mister-smith-config/src/loader.rs`
  - honor env overlays for security auth algorithm, TTLs, issuer, audience, and HMAC secret
- `crates/mister-smith-http/src/server.rs`
  - extend deterministic rejection coverage with an explicit mismatched-scope delegated token case

## Commands Run

```bash
docker compose -f deploy/docker-compose.yml up -d postgres nats
docker exec deploy-postgres-1 pg_isready -U mistersmith -h 127.0.0.1 -p 5432
docker exec deploy-postgres-1 psql -U mistersmith -d postgres \
  -c "CREATE DATABASE mistersmith_ms99_packet016_20260320;"
cargo run -q -p mister-smith-app -- auth openai-chatgpt status
env \
  DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_ms99_packet016_20260320' \
  MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4222' \
  MISTER_SMITH_TRANSPORT__HTTP_PORT='63140' \
  MISTER_SMITH_SECURITY__ENABLED='true' \
  MISTER_SMITH_SECURITY__AUTH_ENABLED='true' \
  MISTER_SMITH_SECURITY__AUTH__ALGORITHM='HS256' \
  MISTER_SMITH_SECURITY__AUTH__ISSUER='mister-smith-ms99-proof' \
  MISTER_SMITH_SECURITY__AUTH__AUDIENCE='ms99-proof' \
  MISTER_SMITH_SECURITY__AUTH__HMAC_SECRET='[redacted]' \
  cargo run -q -p mister-smith-app -- run
curl -sS http://127.0.0.1:63140/health/ready
curl -sS -H 'content-type: application/json' \
  --data @docs/plans/artifacts/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation/task-request.json \
  -D docs/plans/artifacts/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation/task-submit.headers \
  http://127.0.0.1:63140/api/v1/tasks
curl -sS http://127.0.0.1:63140/api/v1/autonomy/workflows
curl -sS http://127.0.0.1:63140/api/v1/autonomy/status/45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b
cargo run -q -p mister-smith-app -- autonomy status \
  --workflow-id 45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b \
  --base-url http://127.0.0.1:63140
```

## Accepted Ingress Proof

Artifacts are under:

`docs/plans/artifacts/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation/`

### Accepted delegated submission

- `task-submit.headers` recorded `HTTP/1.1 202 Accepted`
- `task-submit-response.json` recorded:
  - `task_id = 45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b`
  - `assigned_agent_id = 7b6ecf71-09c8-41bc-bf82-e6cf71f7b2d6`
  - `status = queued`
- `delegated-token.json` recorded the delegated capability used for the accepted ingress:
  - `capability_id = 08f18e36-0737-42a4-a493-dcc1c5d1a124`
  - `descriptor_id = http:post:/api/v1/tasks`
  - `revocation_key = http:post:/api/v1/tasks#execute`

### Workflow-level HTTP inspection

- `autonomy-workflows.json` listed exactly one workflow:
  - `45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b`
- `autonomy-status.json` for the same identifier returned `200 OK`
- the status payload shows:
  - `graph.workflow_id = 45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b`
  - `graph.state = Completed`
  - `graph.active_topology = Hybrid`
  - one operator-visible `external_capability_decisions` entry with:
    - `boundary_surface = task_ingress`
    - `outcome = Allowed`
    - `capability_descriptor_id = http:post:/api/v1/tasks`
    - `action_descriptor_id = http:post:/api/v1/tasks`
    - `scope = InvokeTool`
    - rationale confirming continuity projected from `accepted_task_ingress` sourced from
      `external_delegation`

### CLI parity

- `autonomy-status-cli.txt` renders the same workflow id and terminal state:
  - `workflow: 45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b`
  - `graph: 45cb5f0c-4b13-41bc-9fb5-c8e8207ddc3b Completed`
- the CLI output also renders the same operator-visible accepted-ingress decision:
  - `surface=task_ingress`
  - `outcome=allowed`
  - `capability_descriptor=http:post:/api/v1/tasks`
  - `action_descriptor=http:post:/api/v1/tasks`

## Rejection Scope Decision

Live rejection proof remains out of scope.

Current code truth does not expose a workflow-backed reject surface for delegated HTTP task
ingress:

- rejected delegated HTTP requests are intercepted by the auth middleware in
  `crates/mister-smith-security/src/middleware/axum_mw.rs`
- the delegated boundary action is built from the matched HTTP route, and invalid, revoked, or
  mismatched delegated envelopes return `401 Unauthorized` before the request reaches the task
  handler
- workflow creation only begins after the handler calls `TaskExecutionService::submit_task` in
  `crates/mister-smith-http/src/handlers.rs` and `crates/mister-smith-app/src/execution.rs`

Because rejected requests do not create a workflow record or workflow id on the current path,
there is no honest live workflow-level reject proof to capture for packet `016`.

Deterministic reject coverage remains the correct proof surface and now includes:

- missing auth header
- wrong-route descriptor
- mismatched delegation scope
- revoked capability
- revoked action

## Validation

```bash
cargo test -p mister-smith-config --test loader_tests
cargo test -p mister-smith-security middleware
cargo test -p mister-smith-http
cargo test -p mister-smith-app --test autonomy_status_tests
cargo build --workspace
```

## Final Validation Refresh (`MS-100`)

Date: March 20, 2026 at 22:21:40 EDT

- repo workspace: `/Users/macmain/.local/share/symphony-workspaces/MS-100`
- branch: `ms-100-packet-016-validation`
- validation base: fresh local branch from `origin/main` after recovering the provided workspace
  from an invalid `HEAD`
- packet note basis: packet `016` landed on `main` at `21f46f3`; this refresh revalidates the
  bounded HTTP, app, and event surfaces on `51a9ba1`

### Refresh Commands Run

```bash
cargo test -p mister-smith-http
cargo test -p mister-smith-app --test autonomy_status_tests
cargo test -p mister-smith-events --test autonomy_event_tests
cargo build --workspace
npx markdownlint-cli2 docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md --config .markdownlint.json
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

### Refresh Results

- `cargo test -p mister-smith-http`
  - passed: `59` unit tests, `0` failures
  - includes the delegated-ingress forwarding path plus the deterministic route, scope, and
    revocation rejection coverage added for packet `016`
- `cargo test -p mister-smith-app --test autonomy_status_tests`
  - passed: `17` tests, `0` failures
  - confirms accepted task-ingress continuity, operator-visible rationale/history, and CLI-facing
    autonomy projection remain green
- `cargo test -p mister-smith-events --test autonomy_event_tests`
  - passed: `15` tests, `0` failures
  - confirms event-bus autonomy projection, accepted-ingress decision continuity, and frozen proof
    outcome summaries remain green
- `cargo build --workspace`
  - succeeded with no compile errors across the full Rust workspace
- `npx markdownlint-cli2 docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md --config .markdownlint.json`
  - passed with no markdownlint violations in the refreshed packet-016 proof note
- `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
  - passed after the refreshed validation-evidence branch was pushed, confirming the task-owned
    worktree stayed synced and reviewable

### Final Slice Outcome

- `T017`: complete; the targeted HTTP, app, and event validation set is current and green
- `T018`: complete; `cargo build --workspace` succeeded on the refreshed packet branch
- `T019`: complete; this note now carries both the original accepted-ingress proof and the
  final-validation refresh needed for review without widening packet `016`

## Conclusion

- `T013`: completed by the added deterministic mismatched-scope rejection test in
  `crates/mister-smith-http/src/server.rs`
- `T014`: already satisfied on `main`; the accepted-ingress continuity coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs` remained valid under targeted test
- `T015`: completed by this note plus the artifact set under `docs/plans/artifacts/...`
- `T016`: completed; there is no workflow-backed live reject surface on the current runtime path,
  so live rejection proof stays out of scope
