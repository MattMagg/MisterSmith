# Quickstart: External-Agent Boundary Continuity And Runtime Proof

## Prerequisites

- active feature directory:
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/`
- packet `015`, `MS-95`, and `MS-77` are treated as current baseline truth
- local Rust toolchain available for targeted crate tests
- a runtime environment that can accept one delegated `POST /api/v1/tasks` request when live proof
  is required
- no watched-queue staging is required to validate the packet structure itself

## Required Validation Bundle

```bash
# 1. Narrow automated checks for delegated ingress continuity and inspection
cargo test -p mister-smith-http
cargo test -p mister-smith-app
cargo test -p mister-smith-events

# 2. Cross-crate compile safety
cargo build --workspace
```

## Scenario 1: Confirm The Current Baseline

1. Verify the repo still shows:
   - bounded MCP discovery and enforcement from `MS-77`
   - persisted raw `external_delegation` in workflow metadata
   - workflow-level autonomy inspection via `GET /api/v1/autonomy/status/{workflow_id}`
   - CLI parity via `mister-smith autonomy status --workflow-id ...`
   - packet `015` plus `MS-95` closure for the earlier failure-visible autonomy gap
2. Confirm packet `016` does not reopen packet `015` or widen into provider, router, budget,
   JetStream KV, or broad external-agent work.

## Scenario 2: Prove Accepted Delegated Task Ingress

1. Submit one accepted delegated `POST /api/v1/tasks` request.
2. Capture:
   - the accepted response
   - the returned `workflow_id`
   - persisted workflow metadata continuity for the accepted request
3. Verify the workflow record preserves the accepted boundary context without fabricating a
   decision from raw metadata alone.

## Scenario 3: Inspect Workflow-Level Autonomy Status

1. Use the returned `workflow_id` to call:
   - `GET /api/v1/autonomy/status/{workflow_id}`
2. Verify the response includes:
   - one first-class operator-visible accepted boundary decision
   - preserved provenance and policy continuity
   - no ambiguity about whether the decision came from task ingress or an outbound ToolBus action
     unless the packet explicitly records that a minimal discriminator is required

## Scenario 4: Verify CLI Parity

1. Run:

```bash
mister-smith autonomy status --workflow-id <workflow_id> --base-url http://127.0.0.1:<port>
```

1. Verify the CLI shows the same accepted boundary decision and provenance summary as the workflow
   status route.

## Scenario 5: Preserve Retained Session Continuity Rules

1. Confirm the packet does not relabel task or session views as autonomy-status surfaces.
2. Confirm raw metadata-only delegated ingress still does not fabricate an accepted or rejected
   operator-visible boundary decision in recovered or retained continuity paths.

## Scenario 6: Keep Rejection Proof Deterministic

1. Run deterministic coverage for:
   - missing delegated authority
   - wrong-route delegated authority
   - revoked delegated authority
   - mismatched delegated authority
2. Verify the packet records deterministic rejection proof as in scope.
3. Verify live rejection proof remains out of scope unless a workflow-backed reject surface is
   already proven to exist.

## Expected Proof Artifacts

- one accepted delegated `POST /api/v1/tasks` proof run
- the returned `workflow_id`
- one workflow-level autonomy status capture for that `workflow_id`
- one CLI parity capture for that `workflow_id`
- deterministic rejection coverage evidence
- one durable evaluation note under `docs/plans/`
