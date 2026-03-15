# Mister Smith State Audit And Recovery

Date: March 15, 2026

## Objective

Verify the current Mister Smith control plane as a live system spanning the local repo, smith MCP, Symphony, GitHub, and Linear; repair the highest-risk truth-source drift; define a clean checkpoint and recovery point; and leave a durable agenda for the next development stage.

## Scope

- Local repository state and workflow artifacts
- smith MCP behavior and runtime assumptions
- Symphony launcher and local checkout contract
- Remote GitHub pull request and CI posture
- Live Linear queue, backlog, docs hub, and next runnable work
- Documentation and workflow drift affecting cold-start operators

## Sources Inspected

Repository guidance and plans:

- `AGENTS.md`
- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`
- `docs/plans/2026-03-14-smith-mcp-rebuild.md`
- `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`
- `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`
- `README.md`
- `ROADMAP.md`
- `CLAUDE.md`

Local live evidence:

- `git status --short --branch`
- `git rev-parse HEAD`
- `cargo test -p mister-smith-mcp`
- `cargo build --workspace`
- `cargo fmt --all --check`
- `bash -n scripts/run-symphony.sh`
- local direct stdio validation of `scripts/run-smith-mcp.sh`
- local filesystem inspection of `/Users/macmain/symphony` and `~/.local/share/symphony-workspaces`

Remote/live evidence:

- `gh pr list` / `gh pr view` snapshots for open and merged PRs
- Rube Linear session `gift`
- smith MCP snapshots before and after repair, validated first through a direct local stdio session and then again through the restored desktop-bound smith transport after restart

## Current State By Area

### Local Repo State

- Current branch: `codex/mister-smith-state-recovery-20260315`
- Base HEAD: `2add12b6fd1e06d72c6011b6e8c624eefe4f792f`
- Upstream branch before branch creation: `origin/main`
- Working tree is intentionally dirty; this is still a recovery branch, not a clean checkpoint.
- Modified tracked files still present from the active smith rebuild:
  - `Cargo.lock`
  - `crates/mister-smith-mcp/Cargo.toml`
  - `crates/mister-smith-mcp/src/lib.rs`
  - `crates/mister-smith-mcp/src/server.rs`
  - `scripts/bootstrap_control_plane.py`
- Untracked control-plane/workflow artifacts still present:
  - `crates/mister-smith-mcp/src/bin/`
  - `crates/mister-smith-mcp/src/compatibility.rs`
  - `docs/plans/2026-03-14-smith-mcp-rebuild.md`
  - `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`
  - `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`
  - `scripts/run-smith-mcp.sh`
- Duplicate `.agents/skills/*` imports were present locally during the first recovery pass, but those mirrored already tracked `.claude/skills/*` content and were reclassified as discard noise before landing.
- Session-created adjacent backups with timestamp `20260315-010538` were used during the repair pass and removed during follow-up cleanup.

### smith MCP / Control Plane

Confirmed repairs applied in this session:

- Fixed Linear GraphQL auth to use the raw `Authorization` header instead of `Bearer <LINEAR_API_KEY>`.
- Fixed GitHub PR deserialization for `gh pr list --json ...` camelCase fields.
- Normalized project-slug matching so smith accepts both the live Linear `slugId` and the legacy full slug during recovery.
- Switched the watched project in `WORKFLOW.md` to the live Linear `slugId` `320a0741920c`.
- Updated smith’s default Symphony checkout to `/Users/macmain/symphony`.
- Updated the workflow workspace root to `~/.local/share/symphony-workspaces` and created that directory locally.

Validated post-repair smith state through direct stdio execution of `scripts/run-smith-mcp.sh`:

- `get_control_plane_snapshot` status: `ok`
- `sync_linear_with_runtime` status: `ok`
- `review_merge_dispatch_cycle` status: `ok`
- smith now reports:
  - `symphony_checkout`: `/Users/macmain/symphony`
  - `workflow.project_slug`: `320a0741920c`
  - `workflow.workspace_root`: `~/.local/share/symphony-workspaces`
  - `github.open_pull_requests`: 16
  - `linear.issues_by_state` after queue refill: `Done: 16`, `Todo: 1`

Desktop transport status:

- The desktop app’s `mcp__smith__*` transport now matches the rebuilt server after restart.
- `mcp__smith__get_control_plane_snapshot` and `mcp__smith__sync_linear_with_runtime` both return `status: ok` in the restarted session.

### Symphony Runtime Model

- Canonical local checkout for this machine: `/Users/macmain/symphony`
- Local checkout branch and SHA: `main` at `ff65c7c729c03d4daa550bd30290fc5291f60c67`
- Canonical Elixir app path for launching: `/Users/macmain/symphony/elixir`
- Canonical workspace root: `/Users/macmain/.local/share/symphony-workspaces`
- Repo launcher now matches that model:
  - `scripts/run-symphony.sh` defaults `SYMPHONY_ROOT=$HOME/symphony`
  - `scripts/run-symphony.sh` derives `SYMPHONY_ELIXIR_ROOT=$SYMPHONY_ROOT/elixir`
- `WORKFLOW.md` now uses the live watched `slugId` and canonical workspace root.

### Remote GitHub State

- Open PR count: 16
- Open PR numbers: `#171` through `#186`
- Current open PR lane is dominated by external automation work, including `Sentinel`, `Bolt`, and `perf/*` branches.
- Classification for recovery purposes:
  - security automation lane: `#171`, `#182`, `#184`, `#186`
  - performance automation lane: `#173` through `#181`, plus `#183` and `#185`
  - documentation lane: `#178`
  - feature lane: `#172`
- All 16 open PRs currently look like out-of-band automation work rather than live `Human Review` or `Merging` items from the Symphony-governed queue. Treat them as a separate governance queue until a human decides which ones should merge.
- Representative open PRs still present:
  - `#186` `🛡️ Sentinel: [CRITICAL] Fix hardcoded JWT secret`
  - `#185` `⚡ Bolt: Optimize background deadline monitor allocation`
  - `#184` `🛡️ Sentinel: [CRITICAL] Fix hardcoded JWT secret in default configuration`
- Historical governed flow still exists in merged PRs:
  - `#160`
  - `#162`
  - `#165`
  - `#170`
- Current CI/review automation surface remains active:
  - `Check`
  - `Claude Code Review`
  - `Vet`
  - `Pull Request Labeler`
  - periodic documentation automation on some PRs

### Linear State

Projects:

- `MisterSmith Execution Queue`
  - project id: `b323e5e9-7199-4298-9d60-9e8b53df58c2`
  - watched `slugId`: `320a0741920c`
- `MisterSmith Validated Backlog`
  - project id: `49f87cfe-2bcc-40a8-8d3e-6d7256d1a91c`
- `MisterSmith Workspace Docs`
  - project id: `de035af2-ec7b-4132-afb0-ca15a0f26213`

Team state ids:

- `Todo`: `938f99f9-3518-4ce3-b943-1cea1ebc8b76`
- `In Progress`: `d13b1692-92d8-4a7c-8cdd-259bc8f66519`
- `Human Review`: `e91e7f35-94da-4eea-868d-db7188e51455`
- `Merging`: `d27a2dd3-88e9-4cff-83a3-46f5b3a19a49`
- `Rework`: `7807c63f-6809-4f0f-be1c-2015ea43424e`
- `Done`: `496039df-6666-4ab0-b734-6dc46e7cf800`
- `Backlog`: `39b1e874-3b76-4e27-8289-cb879c1a71ff`

Queue state after this session’s reconciliation:

- smith reports `queue_issue_count: 17`
- smith reports queue states: `Done: 16`, `Todo: 1`
- `MS-33` was moved live from `MisterSmith Validated Backlog` to `MisterSmith Execution Queue` and set to `Todo`
  - issue id: `ea790fde-4647-4dcb-a813-72624f4a9e32`
  - comment id: `c19b5c44-cca0-4bc0-b02a-7751af8305b1`
- `MS-34` remains in `MisterSmith Validated Backlog`
  - issue id: `72742727-27c9-4c54-a001-95abf191e974`
- `MS-35` remains in `MisterSmith Validated Backlog`
  - issue id: `f2f5c77c-5afe-4e84-87c3-553f06836074`

### Docs / Spec / Plan Drift

Updated in this session:

- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `README.md`
- `ROADMAP.md`
- `CLAUDE.md`

Current front-door doc posture after update:

- `WORKFLOW.md` and `docs/linear/LINEAR.md` now match the live runtime contract.
- `README.md`, `ROADMAP.md`, and `CLAUDE.md` now explicitly point cold operators at the active Phase 10 control-plane lane instead of stopping at a Phase 9 or 9.1 worldview.
- Historical dated plans remain intentionally historical and should not be “updated” to erase the audit trail.

## Confirmed Facts Vs Inference

### Confirmed Facts

- smith’s Linear auth bug was real and is repaired in the repo source.
- smith’s GitHub PR parsing bug was real and is repaired in the repo source.
- The live watched Linear identifier for the execution queue is `320a0741920c`.
- The live queue now has one runnable item: `MS-33`.
- The local usable Symphony checkout is `/Users/macmain/symphony` at `ff65c7c729c03d4daa550bd30290fc5291f60c67`.
- The local workspace root exists at `/Users/macmain/.local/share/symphony-workspaces`.
- The GitHub open PR inventory is 16 items, numbers `#171` through `#186`.

### Inference

- The external automation PR lane should be treated as a separate governance lane from the Symphony/Linear queue until explicitly folded into the governed workflow.
- `MS-34` and `MS-35` should remain out of the execution queue until `MS-33` completes because their descriptions still make that dependency explicit even after earlier blockers were resolved.

## Risks And Drift

- The repo is still not at a clean checkpoint because the branch contains preexisting smith rebuild edits plus the current recovery changes that have not yet been committed.
- Open PR governance is unresolved. The current remote PR lane is noisy and not obviously tied to the Linear execution queue.
- The active recovery branch still has a large intentional keep set that needs commit scoping.

## Artifact Classification

Keep on the recovery branch:

- `Cargo.lock`
- `crates/mister-smith-mcp/Cargo.toml`
- `crates/mister-smith-mcp/src/lib.rs`
- `crates/mister-smith-mcp/src/server.rs`
- `scripts/bootstrap_control_plane.py`
- `crates/mister-smith-mcp/src/bin/`
- `crates/mister-smith-mcp/src/compatibility.rs`
- `scripts/run-smith-mcp.sh`
- `docs/plans/2026-03-14-smith-mcp-rebuild.md`
- `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`
- `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`
- The tracked docs and launcher updates from this session

Discarded during follow-up cleanup:

- `.agents/skills/code-review-cycle/`
- `.agents/skills/crawl/`
- `.agents/skills/extract/`
- `.agents/skills/prompt-improver/`
- `.agents/skills/research/`
- `.agents/skills/search/`
- `.agents/skills/tavily-best-practices/`
- `CLAUDE.md.bak-20260315-010538`
- `README.md.bak-20260315-010538`
- `ROADMAP.md.bak-20260315-010538`
- `WORKFLOW.md.bak-20260315-010538`
- `crates/mister-smith-mcp/src/compatibility.rs.bak-20260315-010538`
- `docs/linear/LINEAR.md.bak-20260315-010538`
- `scripts/run-symphony.sh.bak-20260315-010538`
- `scripts/tests/__pycache__/`

## Cleanup And Repair Plan

Completed in this session:

1. Repaired smith truth-source bugs and validated them against live GitHub and live Linear.
2. Canonicalized the local Symphony checkout and workspace root contract.
3. Updated the repo workflow and front-door docs to point at the live Phase 10 control plane.
4. Created the scoped recovery branch `codex/mister-smith-state-recovery-20260315`.
5. Refilled the live execution queue with the only currently runnable backlog issue: `MS-33`.
6. Restarted the desktop session, revalidated smith through the in-app transport, and removed the discard-classified backup/cache artifacts.

Still required:

1. Decide whether the external automation PR lane is merge-worthy, close-worthy, or permanently out of the Symphony/Linear governed lane.
2. Commit the recovery branch once the intended keep set is confirmed.

## Checkpoint And Recovery-Point Definition

Clean checkpoint definition:

- Branch has a name and purpose: `codex/mister-smith-state-recovery-20260315`
- `git status` is clean except for intentional tracked recovery work about to be committed
- smith snapshots agree with live GitHub and live Linear
- Canonical Symphony checkout and workspace root are configured and present
- The watched Linear `project_slug` equals the live `slugId` `320a0741920c`
- The execution queue has only real runnable work, not speculative backlog

Recovery-point definition:

- This note
- The recovery branch name
- The exact base HEAD `2add12b6fd1e06d72c6011b6e8c624eefe4f792f`
- The canonical Symphony SHA `ff65c7c729c03d4daa550bd30290fc5291f60c67`
- The live Linear project ids and staged issue ids above
- The current open PR inventory `#171` through `#186`

## Next-Stage Readiness Assessment

Status: ready for the next execution slice, with governance cleanup still pending

Ready:

- smith repo code is trustworthy again when launched directly
- Linear and workflow identifiers are aligned
- Symphony runtime paths are aligned locally
- The execution queue is no longer empty

Not yet ready:

- PR governance is unresolved

## Ordered Proposed Next Actions

1. Reconnect or restart the desktop session so the in-app `mcp__smith__*` tools use the rebuilt server again.
2. Delete all `.bak-20260315-010538` files and `scripts/tests/__pycache__/` after confirming the backups are no longer needed.
3. Review the keep-classified rebuild artifacts and decide whether they all belong in the next commit.
4. Triage open PRs `#171` through `#186` into one of three lanes: merge candidate, close candidate, or external automation noise to ignore.
5. Launch or dispatch execution for `MS-33` from the now-correct queue.
6. Keep `MS-34` and `MS-35` in `MisterSmith Validated Backlog` until `MS-33` is complete, then repeat the same evidence-backed refill pass.
