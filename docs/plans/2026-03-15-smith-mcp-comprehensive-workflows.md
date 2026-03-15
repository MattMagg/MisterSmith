# Smith MCP Comprehensive Workflows

**Date**: 2026-03-15
**Status**: Proposed
**Scope**: Mister Smith control plane, Linear, Symphony, GitHub, and long-running autonomous workflow orchestration

---

## Objective

Define the workflow-first contract for the `smith` MCP so it acts as the long-running control-plane
bridge between:

- repo truth in Mister Smith
- issue and queue truth in Linear
- unattended execution truth in Symphony
- branch, PR, check, and merge truth in GitHub

The MCP should not primarily be a config checker. It should be the workflow router, reconciler, and
policy engine that keeps those systems aligned across long-running autonomous work.

Read this design alongside `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`, which captures
the repo, git, and GitHub evidence used to reconstruct these workflows.

## Scope

- Bootstrap and readiness workflows
- Legitimacy and intake workflows
- Queue curation and staging workflows
- Long-running execution workflows
- Autonomous review and merge workflows
- Rework and recovery workflows
- Phase planning and backlog progression workflows
- Tool chaining expectations for the existing `smith` MCP compatibility surface

## Assumptions

- `WORKFLOW.md` is the authoritative Symphony runtime contract.
- `docs/linear/LINEAR.md` is the authoritative Linear operating model.
- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` is the authoritative frontier
  mandate for legitimacy and anti-drift decisions.
- The current Linear workflow keeps the states `Todo`, `In Progress`, `Human Review`, `Rework`,
  `Merging`, and `Done`.
- Symphony currently watches one `project_slug`, not an arbitrary set of projects.
- The `smith` MCP should remain deterministic and repo-grounded. It can orchestrate autonomous
  workflows without becoming a separate opaque agent runtime.

## Constraints

- Keep the existing `smith` MCP tool names unless there is a compelling compatibility break reason.
- Preserve the watched-queue contract from `WORKFLOW.md`.
- Preserve the Linear taxonomy from `docs/linear/LINEAR.md`.
- Do not require a human for routine progress when repo state, CI state, review state, and policy
  state are sufficient to continue safely.
- Escalate to a human only on policy triggers, ambiguity, or high-impact risk.

## Non-Goals

- Replacing Symphony as the execution engine
- Replacing Linear as the durable workflow tracker
- Replacing GitHub as the PR and merge source of truth
- Inventing a second status taxonomy outside the documented Linear states

## Design Thesis

The `smith` MCP should be a **workflow control plane**, not a bag of disconnected admin tools.

It should:

- route requests into the correct long-running workflow
- keep repo, Linear, Symphony, and GitHub state reconciled
- make legitimacy and staging decisions using the frontier mandate
- keep unattended work resumable through stable snapshots and workpad continuity
- treat `Human Review` as an **autonomous review gate** by default, with human escalation only
  when policy says autonomy is insufficient

## System Roles

### Mister Smith repository

Source of truth for:

- workflow contract in `WORKFLOW.md`
- architecture and phase intent in `spec/`, `specs/`, `ROADMAP.md`, and `docs/plans/`
- validation rules
- crate boundaries and runtime invariants

### Linear

Source of truth for:

- issue identity
- queue and backlog placement
- execution states
- blocker chains
- project role taxonomy
- durable workpad and operator handoff

### Symphony

Source of truth for:

- active unattended execution against the watched queue
- retry, resume, and issue-session lifecycle
- per-issue workspace lifecycle

### GitHub

Source of truth for:

- branch state
- PR state
- review comments
- CI/check state
- mergeability

### Smith MCP

Source of truth for:

- cross-system reconciliation
- routing to the correct workflow
- legitimacy, staging, and anti-drift policy decisions
- autonomous review and merge decisions
- resumable control-plane snapshots

It should derive, not invent, core state.

## Canonical Workflow Graph

```text
Bootstrap / Readiness
  -> Control-plane Snapshot
  -> Intake / Legitimacy
  -> Backlog Classification
  -> Queue Staging
  -> Symphony Dispatch
  -> Active Execution
  -> Autonomous Review Gate
  -> Rework or Merging
  -> Done

Recovery / Reconciliation can re-enter from any stage.
Phase Planning feeds Queue Staging.
```

## Workflow Bundles

Each workflow bundle is a chained control-plane behavior, not a single tool.

### 1. Bootstrap And Readiness

**Purpose**: verify that the control-plane can reason correctly before unattended work begins.

**Primary tools**:

- `audit_workflow_readiness`
- `get_server_runtime_info`
- `plan_workspace_adjustments`
- `get_control_plane_snapshot`

**Source anchors**:

- `WORKFLOW.md`
- repo-local Smith skills

**Decision rules**:

- Treat missing auth, missing required tools, missing repo workflow files, or missing watched queue
  metadata as blockers.
- Treat optional tooling and advisory integrations as warnings.
- Do not overfit readiness to one local checkout layout when the repo contract does not require it.

**Output**:

- structured readiness result
- stable runtime metadata
- safe local adjustment plan
- first full control-plane snapshot

### 2. Request Routing

**Purpose**: convert an operator request into the correct workflow bundle.

**Primary tools**:

- `route_workflow_request`
- `get_control_plane_snapshot`

**Routing result should include**:

- bundle name
- why that bundle applies
- next tool chain
- whether the request is repo-local, queue-local, review-local, or legitimacy-local

**Required route families**:

- bootstrap and repair
- queue reconciliation
- phase staging
- issue execution inspection
- review and merge dispatch
- legitimacy and follow-up classification

### 3. Intake And Legitimacy

**Purpose**: decide whether work is real, frontier-aligned, and ready for backlog or queue handling.

**Primary tools**:

- `evaluate_issue_legitimacy`
- `classify_follow_up_work`

**Source anchors**:

- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`
- `docs/linear/LINEAR.md`

**Decision rule**:

Use the frontier mandate directly:

1. Does the work strengthen supervised autonomy?
2. Does it improve long-term leverage in coordination, supervision, execution, memory, streaming,
   routing, reliability, observability, state, or distributed behavior?

If both are no, it is likely drift.

**Placement rule**:

- `Triage`: raw or unvalidated intake
- `MisterSmith Validated Backlog`: validated real work that should not dispatch yet
- `MisterSmith Workspace Docs`: cross-cutting documentation or operating model work
- `MisterSmith Execution Queue`: only after explicit staging and only when runnable now

### 4. Queue Reconciliation

**Purpose**: ensure Linear, Symphony, GitHub, and repo truth agree on what is runnable now.

**Primary tools**:

- `sync_linear_with_runtime`
- `get_control_plane_snapshot`
- `get_issue_execution_snapshot`

**Source anchors**:

- `WORKFLOW.md`
- `docs/linear/LINEAR.md`

**Required checks**:

- watched project slug matches `WORKFLOW.md`
- only watched-project issues in active states are considered live dispatch candidates
- `Todo` means runnable now, not merely important next work
- empty `Todo` means there is nothing runnable right now
- validated backlog is not silently treated as queue

**Output**:

- queue occupancy
- discrepancies
- concrete refill or cleanup actions

### 5. Phase Planning And Honest Staging

**Purpose**: convert repo-grounded phase work into honest Linear staging actions.

**Primary tools**:

- `plan_phase_execution`
- `apply_phase_execution_plan`

**Source anchors**:

- `specs/<phase>/spec.md`
- `specs/<phase>/plan.md`
- `specs/<phase>/tasks.md`
- `docs/linear/LINEAR.md`

**Required behavior**:

- distinguish completed work, runnable slices, blocked slices, and prep slices
- preserve blocker chains
- stage only unblocked runnable work into the watched queue
- keep future validated work in `MisterSmith Validated Backlog`

**Prohibition**:

- do not move blocked work into the execution queue just to keep Symphony busy

### 6. Active Execution

**Purpose**: govern the unattended work loop once an issue is in the watched queue.

**Primary tools**:

- `get_issue_execution_snapshot`
- `get_control_plane_snapshot`

**Execution contract from `WORKFLOW.md`**:

- `Todo -> In Progress`: Symphony picks up the issue
- find or create a single `## Codex Workpad`
- reconcile that workpad at the start of every execution pass
- keep acceptance criteria, validation checklist, and blockers current
- final execution output reports completed actions and blockers only

**Smith MCP role**:

- provide issue-centric state snapshots
- detect divergence between issue state, PR state, and repo state
- keep the control-plane aware of retries, stale PRs, and missing handoff data

### 7. Autonomous Review Gate

**Purpose**: make `Human Review` autonomous by default while preserving the existing Linear state
name for compatibility.

`Human Review` should no longer mean "wait passively for a person." It should mean:

> the issue is in a review gate where the control plane gathers review evidence, applies merge
> policy, and either advances to `Merging`, sends the issue to `Rework`, or escalates to a human
> only when policy requires it.

**Primary tools**:

- `review_merge_dispatch_cycle`
- `get_issue_execution_snapshot`
- `get_control_plane_snapshot`

**Evidence sources**:

- GitHub PR comments and review summaries
- required checks and CI status
- repo validation evidence recorded in the workpad
- advisory review automation such as Claude review workflow results
- policy triggers from repo guidance

**Autonomous review sub-stages**:

1. Collect review state
2. Classify findings as actionable, advisory, stale, or resolved
3. Reconcile PR comments against repo state and latest commits
4. Apply merge posture rules
5. Decide:
   - `Merging` when required validation is green and no unresolved blocking feedback remains
   - `Rework` when actionable feedback or failed substantive checks remain
   - human escalation only when policy triggers fire

**Human escalation triggers**:

- reviewer disagreement that the MCP cannot resolve deterministically
- security-sensitive or destructive changes with unclear safety
- unclear legitimacy or scope drift near merge time
- repeated rework loops without convergence
- merge policy ambiguity

### 8. Rework Loop

**Purpose**: restart execution as a deliberate fresh attempt, not an incremental thrash loop.

**Primary tools**:

- `get_issue_execution_snapshot`
- `route_workflow_request`
- `get_control_plane_snapshot`

**Required behavior from `WORKFLOW.md`**:

- treat `Rework` as a fresh attempt
- re-read feedback
- identify what changes this attempt
- refresh the workpad plan
- reopen execution from current truth, not stale assumptions

**Smith MCP role**:

- summarize unresolved review findings
- detect whether branch or PR reuse is still valid
- reconnect the issue to the correct execution bundle

### 9. Merge And Landing

**Purpose**: finish approved work without creating manual gaps between review and merge.

**Primary tools**:

- `review_merge_dispatch_cycle`
- `get_issue_execution_snapshot`

**Delegated skill / operator flow**:

- `land`

**Required behavior**:

- ensure checks are green
- ensure feedback is resolved
- ensure branch is current with `main`
- merge only when the substantive merge gate is satisfied
- move `Merging -> Done` after successful merge

### 10. Recovery And Reconciliation

**Purpose**: recover from crashed sessions, stale issue state, stale PRs, or partially applied work
without losing long-running continuity.

**Primary tools**:

- `get_control_plane_snapshot`
- `sync_linear_with_runtime`
- `get_issue_execution_snapshot`
- `reload_server`

**Recovery rules**:

- repo, Linear, GitHub, and Symphony can drift; the MCP must detect and report that drift
- the workpad is the durable execution breadcrumb
- if issue state and PR state disagree, prefer explicit reconciliation over blind advancement
- if the queue is empty, confirm whether work is actually absent or merely already claimed

## Tool-Chaining Contract

The existing tool surface can support the workflow model if the semantics are tightened.

| Workflow Bundle | Primary Entry Tool | Typical Chain |
| --- | --- | --- |
| Bootstrap | `audit_workflow_readiness` | `audit_workflow_readiness -> get_server_runtime_info -> plan_workspace_adjustments -> get_control_plane_snapshot` |
| Request routing | `route_workflow_request` | `route_workflow_request -> bundle-specific next tools` |
| Intake and legitimacy | `evaluate_issue_legitimacy` | `evaluate_issue_legitimacy -> classify_follow_up_work -> sync_linear_with_runtime` |
| Queue reconciliation | `sync_linear_with_runtime` | `sync_linear_with_runtime -> get_issue_execution_snapshot -> review_merge_dispatch_cycle` |
| Phase staging | `plan_phase_execution` | `plan_phase_execution -> apply_phase_execution_plan -> sync_linear_with_runtime` |
| Execution inspection | `get_issue_execution_snapshot` | `get_issue_execution_snapshot -> get_control_plane_snapshot` |
| Autonomous review gate | `review_merge_dispatch_cycle` | `review_merge_dispatch_cycle -> get_issue_execution_snapshot -> get_control_plane_snapshot` |
| Recovery | `get_control_plane_snapshot` | `get_control_plane_snapshot -> sync_linear_with_runtime -> reload_server` |

## Required Semantic Upgrades To The Current Rebuild

### `route_workflow_request`

Upgrade from keyword routing to workflow-bundle routing grounded in:

- watched queue rules
- issue lifecycle state
- legitimacy policy
- phase staging intent
- review and merge posture

### `sync_linear_with_runtime`

Upgrade from a queue count snapshot to a real queue contract evaluator that can answer:

- is the watched queue healthy
- is work incorrectly sitting in backlog versus queue
- is `Human Review` accumulating unattended work
- is `Merging` blocked on mergeability or stale checks

### `review_merge_dispatch_cycle`

Upgrade from queue-summary helper to the core autonomous review and dispatch loop.

It should:

- inspect Human Review items first
- classify each into merge, rework, escalate, or wait
- prioritize Merging completion before refill
- refill the queue only from valid staged candidates after review pressure is cleared

### `plan_phase_execution`

Upgrade from title heuristics to spec- and task-grounded planning with:

- runnable slices
- blocked slices
- prep slices
- explicit staging rationale

### `evaluate_issue_legitimacy` and `classify_follow_up_work`

Upgrade from keyword heuristics to mandate-aware policy based on:

- frontier leverage
- supervised autonomy
- anti-drift rule
- queue versus backlog placement rules

## Long-Running Autonomy Rules

The MCP should be tailored to long-running workflows explicitly.

### Resumability

- every workflow must be restart-safe
- every decision should be reproducible from durable system state
- the workpad is part of the execution checkpoint, not commentary

### Bounded autonomy

- autonomy is the default
- revocation and escalation remain available
- human intervention is exceptional, not routine

### Observable decisions

- every queue move, review decision, staging action, and escalation should be attributable
- review and merge decisions should carry enough evidence to explain why they happened

### Minimal hidden state

- the MCP may cache for speed
- it should not depend on hidden mutable state for correctness

## Milestones

### M1: Workflow contract capture

- write this workflow-first design
- align the rebuild plan to it

**Validation**

- cross-check against `WORKFLOW.md`, `docs/linear/LINEAR.md`, and the frontier mandate

### M2: Tool semantic tightening

- refine the existing `smith` MCP handlers to match the workflow contract

**Validation**

- handler tests reflect queue taxonomy, lifecycle routing, and review-gate behavior

### M3: Autonomous review gate

- implement the real review/rework/merge controller behavior behind `review_merge_dispatch_cycle`

**Validation**

- repo scenarios cover merge, rework, and human-escalation paths

### M4: Phase staging and legitimacy hardening

- tighten phase planning and legitimacy decisions to repo policy sources

**Validation**

- phase and follow-up scenarios route correctly between queue, backlog, docs hub, and triage

## Stop Conditions

- the `smith` MCP can explain and chain every major Mister Smith workflow from bootstrap through
  merge and recovery
- `Human Review` is modeled as an autonomous review gate rather than a passive wait state
- queue staging, phase staging, legitimacy, and recovery decisions are grounded in repo docs rather
  than ad hoc heuristics
- the MCP remains tailored to Mister Smith, Linear, and Symphony without trying to replace any of
  them
