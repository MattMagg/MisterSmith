# MS-33 Dispatch and Phase 10 Next-Queue Prep

## Objective

- Trigger the watched execution queue for `MS-33` and confirm that Symphony is actively processing it.
- While `MS-33` is running, inspect `MS-34` and `MS-35` and leave a durable, dependency-aware prep packet for the next refill pass.

## Scope

- Repo-owned workflow contract in `WORKFLOW.md` and `docs/linear/LINEAR.md`
- Smith control-plane state for Mister Smith, Linear, GitHub, and Symphony
- Live Symphony runtime launched from `./scripts/run-symphony.sh`
- Linear issues `MS-33`, `MS-34`, and `MS-35`

## Assumptions

- An issue in the watched project with state `Todo` is sufficient to trigger Symphony dispatch.
- Blocked work should remain in `MisterSmith Validated Backlog` until the blocking issue is complete.
- Read-only inspection inside the active Symphony workspace is safe as long as it does not modify the worker copy.

## Constraints

- Follow the repo-owned sequence: Linear first, Symphony second.
- Do not move blocked future work into the watched project just to keep Symphony busy.
- Do not interfere with the live `MS-33` worker beyond read-only status checks.

## Non-goals

- Manually implement `MS-33`, `MS-34`, or `MS-35` from the main repo checkout during this pass
- Override Symphony's lifecycle mutations unless the runtime proves unable to proceed
- Re-slice the validated backlog beyond the current `MS-33` -> `MS-34` -> `MS-35` dependency chain

## Milestones

### 1. Confirm queue trigger inputs and runtime launch

- Verify `MS-33` is still in the watched project and in `Todo`
- Launch the supported repo wrapper `./scripts/run-symphony.sh`
- Confirm a live Symphony dashboard, worker PID, workspace clone, and Codex session

**Validation**

- `smith.get_control_plane_snapshot`
- `smith.sync_linear_with_runtime`
- `smith.get_issue_execution_snapshot` for `MS-33`
- `./scripts/run-symphony.sh`
- Read-only process and workspace inspection

### 2. Monitor `MS-33` startup without interfering

- Confirm the worker has moved past idle queue polling and into a live Codex session
- Check the workspace and runtime logs for startup activity or obvious blockers

**Validation**

- Symphony TTY status output
- `ps` against the spawned Codex PID
- Workspace existence under `~/.local/share/symphony-workspaces/MS-33`
- `logs/mcp-tools.log` tail in the active workspace

### 3. Prep `MS-34` and `MS-35`

- Reconfirm their current Linear state and blocking posture
- Compare issue descriptions against current repo reality
- Record missing surfaces, existing scaffolds, and validation bundles for the next refill pass

**Validation**

- `linear.get_issue` for `MS-34` and `MS-35`
- `smith.get_issue_execution_snapshot` for `MS-34` and `MS-35`
- Targeted repo file inventory for the paths named in the issue descriptions

## Sources Inspected

- `/Users/macmain/MisterSmith/WORKFLOW.md`
- `/Users/macmain/MisterSmith/docs/linear/LINEAR.md`
- `/Users/macmain/MisterSmith/scripts/run-symphony.sh`
- `/Users/macmain/MisterSmith/specs/012-phase10-frontier-autonomy/tasks.md`
- `/Users/macmain/MisterSmith/specs/012-phase10-frontier-autonomy/quickstart.md`
- `/Users/macmain/MisterSmith/docs/plans/2026-03-15-mister-smith-state-audit-and-recovery.md`
- Smith MCP: route, snapshot, sync, review cycle, issue execution snapshots, phase execution plan
- Linear issue records for `MS-33`, `MS-34`, and `MS-35`
- Active Symphony workspace and runtime process state

## Live Dispatch Evidence

- Repo state before launch: clean `main` at `bf85b10`
- `MS-33` remained in project `MisterSmith Execution Queue` (`slugId 320a0741920c`) and state `Todo`
- `./scripts/run-symphony.sh` launched successfully and exposed the dashboard at `http://127.0.0.1:4000/`
- Symphony spawned a live Codex worker for `MS-33` with PID `88945`
- Symphony created a workspace at `/Users/macmain/.local/share/symphony-workspaces/MS-33`
- Symphony assigned a live session id beginning `019c...30e1cf`
- Runtime output advanced from `no codex` to `mcp start`, `turn start`, `user message`, `item start`, and `item complete`
- Token counters reached `124,155` input and `1,930` output during the first observed turn, confirming real execution rather than idle polling

## Current Runtime Caveat

- As of this note, Linear still reports `MS-33` in `Todo` even though Symphony has a live worker session and active token use.
- Treat that mismatch as startup lag until the runtime either writes the tracker mutation or produces a concrete tracker error.
- No fatal startup error was visible in the observed logs; the workspace-local `mcp-tools.log` only showed advisory persistence warnings unrelated to the repo's Linear trigger path.

## `MS-34` Prep Packet

**Current state**

- Linear state: `Backlog`
- Project: `MisterSmith Validated Backlog`
- Explicit blocker: `MS-33`

**Repo reality check**

- Missing today:
  - `crates/mister-smith-security/src/delegation.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- Already present and likely to be extended rather than created from scratch:
  - `crates/mister-smith-security/tests/delegation_tests.rs`
  - `crates/mister-smith-security/src/jwt/claims.rs`
  - `crates/mister-smith-security/src/auth_callout.rs`
  - `crates/mister-smith-security/src/audit/events.rs`
  - `crates/mister-smith-agents/tests/gate10_tests.rs`
  - `crates/mister-smith-app/src/observability.rs`
  - `crates/mister-smith-events/src/autonomy.rs`

**Execution guidance**

- Keep `MS-34` in validated backlog until `MS-33` is done because the issue description and Phase 10 dependency model both require operator-visible provenance first.
- Treat `T032` as the contract-defining first cut.
- After `T032`, `T033`, `T034`, and `T035` are the best parallelizable cluster.
- Finish with security, agents, and app validation:
  - `cargo test -p mister-smith-security`
  - `cargo test -p mister-smith-agents`
  - `cargo test -p mister-smith-app`

## `MS-35` Prep Packet

**Current state**

- Linear state: `Backlog`
- Project: `MisterSmith Validated Backlog`
- Explicit blockers: `MS-28`, `MS-29`, `MS-30`, `MS-31`, `MS-32`, `MS-33`, and `MS-34`

**Execution guidance**

- `MS-35` should remain outside the watched queue until both `MS-33` and `MS-34` are complete.
- The gate is validation-and-docs only. It should not reopen already-landed design decisions unless the validation bundle finds a real defect.
- Expected validation bundle:
  - `cargo test -p mister-smith-agents`
  - `cargo test -p mister-smith-persistence`
  - `cargo test -p mister-smith-security`
  - `cargo test -p mister-smith-llm`
  - `cargo test -p mister-smith-core`
  - `cargo test -p mister-smith-app`
  - `cargo build --workspace`
- Expected documentation touch set:
  - `ROADMAP.md`
  - `CLAUDE.md`
  - `README.md`
- Quickstart dependency:
  - Scenario 4 validates operator inspection of autonomy state
  - Scenario 5 validates delegation rejection after bounded delegation lands

## Planning Discrepancy To Carry Forward

- `smith.plan_phase_execution` currently warns that `specs/012-phase10-frontier-autonomy/tasks.md` is stale relative to already-landed Phase 10.0-10.4 work.
- That same planner recommended staging `MS-36` because it infers the current
  operator-view slice from task text, but live Linear and the watched queue
  already identify `MS-33` as the active Phase 10.5 issue.
- For the next refill pass, prefer current Linear issue inventory and repo file existence over stale task-pack naming.

## Next Action

- Keep Symphony running and let the `MS-33` worker continue its first execution turn.
- Recheck the issue state and PR surface after the worker reaches a natural handoff or writes its first tracker mutation.
- Refill with `MS-34` only after `MS-33` is complete; keep `MS-35` behind both.

## Stop Conditions

- `MS-33` is confirmed to have a live Symphony worker or a concrete startup blocker is documented
- `MS-34` and `MS-35` both have durable prep guidance captured from current repo evidence
- The repo remains clean on `main`
