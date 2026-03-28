# Mister Smith MS-116 Fresh-Session Initialization Handoff

You are Codex in a fresh session working in:

- <repo_root>`/Users/macmain/MisterSmith`</repo_root>

Your mission is to initialize the next honest packet-021 implementation issue in Linear without
starting implementation work:

- <linear_issue>`MS-116`</linear_issue>
- Title: `T3: Add bounded profile fingerprints`
- Parent packet: <linear_parent_issue>`MS-113`</linear_parent_issue>
- Current known state at handoff:
  - `MS-114` is landed on `main`
  - `MS-115` is landed on `main`
  - `MS-116` is currently `Backlog`
  - `MS-117` is currently `Backlog`
  - `MS-118` is still blocked by `MS-116` and `MS-117`
  - `MS-116` does not yet have a `## Codex Workpad`
- Issue URL:
  <linear_issue_url>`https://linear.app/agentic-ops/issue/MS-116`</linear_issue_url>
- Packet source:
  <packet_source>`/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/`</packet_source>

This is an issue-initialization session, not an implementation session.

## Objective

By the end of this session, you must:

1. verify that `MS-116` is still the next honest packet-021 child to initialize
2. reconcile the current repo and Linear state before mutating anything
3. create or update exactly one durable `## Codex Workpad` comment on `MS-116`
4. leave `MS-116` in `Backlog`
5. stop before code edits, branch creation, queue staging, or PR work

## Start Sequence

Before mutating Linear state, read these files in order:

1. `/Users/macmain/MisterSmith/AGENTS.md`
2. `/Users/macmain/MisterSmith/WORKFLOW.md`
3. `/Users/macmain/MisterSmith/docs/linear/LINEAR.md`
4. `/Users/macmain/MisterSmith/docs/current-state.md`
5. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/spec.md`
6. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/plan.md`
7. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/tasks.md`
8. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/contracts/supervision-evidence-contract.md`

Then verify control-plane truth before taking any action:

1. start with Smith MCP first-hop routing
2. fetch the current snapshots for `MS-116`, `MS-117`, `MS-118`, and parent `MS-113`
3. confirm `MS-118` is still blocked and should not be initialized first
4. confirm the primary checkout is on clean synced `main`

## Session Scope

Initialize `MS-116` only.

What that means:

- confirm the issue is still the next honest packet-021 child to prepare
- create or reconcile the single durable workpad comment
- record scope, surfaces, constraints, validation plan, and next action for a later implementation
  session
- keep the issue resumable for a cold-start agent

What that does not mean:

- do not start implementation
- do not create a branch
- do not move the issue to `In Progress`
- do not open a PR
- do not initialize `MS-117` or `MS-118` in the same session unless the user explicitly asks
- do not widen into packet-019 defaultization, operator-console work, or packet closure

## MS-116 Scope To Capture In The Workpad

Record the bounded packet tasks for `MS-116`:

- `T013` add fingerprint serialization and guard-evidence tests
- `T014` add JetStream KV fingerprint coverage
- `T015` add fingerprint storage helpers
- `T016` extend profile and Guard decision logic to consume fingerprints
- `T017` wire fingerprint loading and save/update flow into runtime execution

Record the primary surfaces:

- `/Users/macmain/MisterSmith/crates/mister-smith-persistence/src/kv/`
- `/Users/macmain/MisterSmith/crates/mister-smith-persistence/tests/kv_tests.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-agents/src/profile.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-agents/src/guard.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/execution.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/tests/trait_compilation_tests.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-agents/tests/`

Record the non-goals:

- no `MS-117`
- no `MS-118`
- no operator-console rendering
- no packet-019 defaultization
- no broader learned control-plane work
- no raw transcript duplication outside existing audit or replay surfaces

## Workpad Requirements

Create or update exactly one `## Codex Workpad` comment containing:

- environment stamp as `host:cwd@sha`
- issue title and URL
- parent issue reference `MS-113`
- current Linear state and whether the issue has a workpad yet
- concise scope summary
- milestone checklist for `T013` through `T017`
- validation checklist for the future implementation lane
- blockers
- assumptions
- explicit non-goals
- exact next action for the future implementation session

If an existing workpad is already present, update it instead of creating a second one.

## Validation Plan To Record

Capture the narrowest honest future validation set for this slice:

- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-persistence`
- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-app`
- `cargo clippy -p mister-smith-core -- -D warnings`
- `cargo clippy -p mister-smith-persistence -- -D warnings`
- `cargo clippy -p mister-smith-agents -- -D warnings`
- `cargo clippy -p mister-smith-app -- -D warnings`
- `cargo build --workspace`
- `git diff --check`
- `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

Do not run those commands unless you actually make repo changes during this session.

## Decision Rules

- If `MS-116` is still `Backlog` and uninitialized, initialize it and stop.
- If `MS-116` already has a good workpad, reconcile and tighten it rather than replacing it.
- If control-plane evidence shows `MS-117` must be prepared first, stop and report the exact
  evidence instead of silently changing direction.
- If the repo is not on clean synced `main`, report that before mutating Linear state.

## Final Response Requirements

At the end of the session, report only:

- whether `MS-116` was confirmed as the next honest issue
- whether the workpad was created or updated
- the exact next implementation action recorded
- blockers or risks, if any
- whether any repo files were edited

Do not claim implementation progress. This session is complete only when `MS-116` is cleanly
initialized in Linear for a later implementation session.
