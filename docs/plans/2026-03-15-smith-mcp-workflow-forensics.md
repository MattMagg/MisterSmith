# Smith MCP Workflow Forensics

**Date**: 2026-03-15
**Status**: Investigated
**Purpose**: Reconstruct the historical Mister Smith workflow model from repo documentation, plans,
skills, git history, and GitHub PR evidence so the `smith` MCP can be rebuilt against the real
operating pattern rather than a guessed compatibility surface.

---

## Objective

Recover the workflow system that previously connected:

- Mister Smith repo policy and specs
- Linear queue and issue state
- Symphony unattended execution
- GitHub PR, review, CI, and merge signals

This note is evidence-first. It distinguishes:

- documented contract
- repeated operational pattern
- informed inference for the missing `smith` MCP behavior

## Scope

- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- repo-local skills under `.codex/skills/`
- implementation plan docs under `docs/plans/`
- GitHub workflow docs and review automation
- git history for workflow-related files
- merged and open GitHub PR history

## Method

1. Search the repo corpus for workflow markers such as `Codex Workpad`, `Human Review`,
   `Merging`, `Rework`, `Execution Queue`, `Validated Backlog`, and Smith control-plane tool names.
2. Read the canonical workflow docs and the repo-local skills they refer to.
3. Read representative ticket implementation plans to extract repeated execution habits.
4. Inspect git history for when the workflow files, queue model, and control-plane docs changed.
5. Inspect representative merged GitHub PRs to confirm how work was actually described, reviewed,
   validated, and merged.

## High-Confidence Source Anchors

| Source | What it proves |
| --- | --- |
| `WORKFLOW.md` | The Symphony execution contract, queue state machine, workpad discipline, and issue lifecycle |
| `docs/linear/LINEAR.md` | The Linear taxonomy, watched queue boundary, validated backlog model, and status semantics |
| `.codex/skills/linear/SKILL.md` | Direct Linear GraphQL operations were a first-class part of the unattended workflow |
| `.codex/skills/pull/SKILL.md`, `.codex/skills/push/SKILL.md`, `.codex/skills/land/SKILL.md` | Branch sync, PR publication, and merge handling were codified as discrete workflow units |
| `.codex/skills/symphony-mister-smith-review-dispatch/SKILL.md` | Review/merge dispatch and watched-queue refill were already treated as one control-plane loop |
| `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` | Legitimacy and anti-drift decisions were supposed to serve supervised autonomy, not generic cleanup |
| Merged PRs `#160`, `#162`, `#165`, `#170` | Repeated Problem/Solution/Validation PR structure, Linear linkback, automated review signals, and narrow validation discipline |

## Recovered Timeline

### March 7, 2026: Review automation becomes explicit

Evidence:

- commit `c696a35`: `feat: Introduce vet skill with history loaders for OpenCode, Claude Code, and Codex sessions.`

Meaning:

- The workflow already expected session-aware automated review, not only tests and human reading.
- Review was tied to the current coding session and git diff, which is consistent with a long-running
  autonomous control plane.

### March 8, 2026: The core Symphony issue workflow lands

Evidence:

- commit `81daf2c`: adds `WORKFLOW.md` and the `linear`, `pull`, `push`, and `land` skills

Meaning:

- The repo already had a coherent unattended workflow by this point.
- The workflow was issue-centric, stateful, and explicitly resumable.
- The durable execution breadcrumb was the `## Codex Workpad` comment, not hidden conversation state.

### March 8 to March 11, 2026: Linear becomes the formal control-plane tracker

Evidence:

- commit `31d4e9b`: adds `docs/linear/LINEAR.md`
- commits `37551c8`, `a0188f9`, `806eb83`, `71ebb89`, `16b5907`, `93839e7`, `2bc9549`
  repeatedly refine status definitions, project slug, queue semantics, and frontier guidance

Meaning:

- The workflow matured from "Symphony runs on Linear issues" into a clearer queue model:
  watched execution queue, validated backlog, and docs hub.
- This was not incidental documentation drift. It was operational refinement.

### March 10 and March 11, 2026: Frontier policy and control-plane shims are added

Evidence:

- commit `2bc9549`: `docs(workflow): wire frontier mandate into Symphony`
- commit `b757c51`: `chore(control-plane): bootstrap repo-local smith workflow`
- commit `65211a1`: `chore(control-plane): standardize smith bootstrap`

Meaning:

- The missing `smith` MCP was not just a launcher or config alias.
- It was intended to route workflow requests, bootstrap the repo-local skill pack, and expose the
  control-plane tool chain.

### March 11, 2026: Review posture is formalized

Evidence:

- commit `af02335` / PR `#165`: stabilize `Claude Code Review` workflow and document merge posture

Meaning:

- Review signals were already multi-source:
  - substantive CI
  - vet
  - Claude review
  - Codex review
  - PR comments and review feedback
- `Human Review` was documented as passive waiting, but the automation layer was already active.

## Recovered Core Workflow Model

### 1. Queue topology was explicit and intentionally small

Evidence:

- `docs/linear/LINEAR.md`
- `WORKFLOW.md`

Recovered model:

- `MisterSmith Execution Queue` is the single watched dispatch boundary.
- `MisterSmith Validated Backlog` is for real repo-validated work that is not yet scheduled.
- `MisterSmith Workspace Docs` is a separate docs hub, not part of the live queue.
- `Todo` means runnable now in the watched queue, not just "important next work."
- An empty `Todo` queue is acceptable and does not justify stuffing future work into the queue.

This is a high-confidence conclusion.

### 2. Each issue had a persistent execution ledger

Evidence:

- `WORKFLOW.md`
- multiple implementation plans under `docs/plans/`
- `.codex/skills/linear/SKILL.md`

Recovered model:

- Every run starts by finding or creating one `## Codex Workpad` comment.
- The workpad stores:
  - environment stamp
  - plan
  - acceptance criteria
  - validation checklist
  - blockers
- The workpad is reconciled at the start of every attempt and updated at each milestone.

This is a high-confidence conclusion.

### 3. Execution was narrow, spec-driven, and validation-first

Evidence:

- `WORKFLOW.md`
- `docs/plans/2026-03-09-message-signing-hmac-sha256.md`
- `docs/plans/2026-03-10-ms-29-execution-graph-topology-compiler.md`
- `docs/plans/2026-03-10-ms-31-managed-memory-and-context-snapshots.md`
- PRs `#160`, `#162`, `#170`

Recovered model:

- The implementation plans consistently used:
  - goal
  - architecture
  - task-by-task file scope
  - explicit commands to prove each step
- Validation was generally:
  - affected crate tests
  - `cargo build --workspace`
  - broader checks only when scope justified it
- The repo preferred narrow proof with explicit evidence over giant blanket test runs.

This is a high-confidence conclusion.

### 4. Linear issues were structured as executable mandates

Evidence:

- `docs/linear/LINEAR.md`
- PR linkback comments on `#160`, `#165`, `#170`

Recovered model:

- Linear issues contained:
  - spec paths
  - tasks IDs
  - file locations
  - workflow expectations
  - acceptance criteria
  - blocker chains
- The PR linkback comments show that the issue body itself carried enough context to drive unattended
  execution.

This is a high-confidence conclusion.

### 5. The branch/PR workflow was standardized

Evidence:

- `docs/linear/LINEAR.md`
- `.codex/skills/pull/SKILL.md`
- `.codex/skills/push/SKILL.md`
- `.codex/skills/land/SKILL.md`
- merged PR branch names from GitHub history

Recovered model:

- Branches were Linear-derived, usually `matthewtmaggio/ms-<number>-<slug>`.
- `push` published the branch and created or refreshed the PR.
- `pull` merged `origin/main` into the current issue branch with validation afterward.
- `land` handled the final merge only after review feedback and checks were resolved.

This is a high-confidence conclusion.

### 6. Review was already partially autonomous

Evidence:

- `vet` skill
- `vet.yml` documentation in `.github/workflows/README.md`
- `claude-code-review.yml`
- PR review activity on `#160`, `#165`, `#170`
- `docs/linear/LINEAR.md` review state machine

Recovered model:

- The documented state machine says:
  - `In Progress -> Human Review -> Merging -> Done`
  - `Human Review -> Rework` on requested changes
- In practice, the review lane already involved automated signals:
  - vet posted findings or explicit zero-issue results
  - Codex review posted review passes/comments
  - Claude review ran automatically and had documented merge posture
  - GitHub checks provided substantive gate signals

**Inference**:

The original workflow was already moving toward autonomous review, even though the documented
`Human Review` state still described passive waiting. The control plane likely intended `Human Review`
to be a gate that collects and reconciles review evidence, with human input reserved for ambiguous or
high-risk cases.

This is a medium-confidence inference supported by strong surrounding evidence.

### 7. Rework was a first-class loop, not an afterthought

Evidence:

- `WORKFLOW.md`
- `docs/linear/LINEAR.md`

Recovered model:

- `Rework` restarts the execution flow as a fresh attempt.
- The workflow explicitly rejects "tiny patch pass" handling.
- The system expects the workpad and branch/PR vehicle to be reevaluated at rework time.

This is a high-confidence conclusion.

### 8. The missing `smith` MCP sat above the workflow, not beside it

Evidence:

- commit `b757c51`
- `.codex/skills/mister-smith-control-plane-router/SKILL.md`
- `.codex/skills/mister-smith-control-plane-bootstrap/SKILL.md`
- `.codex/skills/stage-mister-smith-phase/SKILL.md`
- `.codex/skills/symphony-mister-smith-review-dispatch/SKILL.md`

Recovered model:

- The `smith` MCP was supposed to:
  - bootstrap the control-plane environment
  - route workflow requests
  - inspect current repo/queue/PR/runtime state
  - stage honest phase work into the queue
  - run the review/merge/refill loop
  - make legitimacy and follow-up classification decisions

It was a workflow orchestrator and reconciler, not merely an MCP server discovery endpoint.

This is a high-confidence conclusion.

## Reconstructed Smith MCP Workflow Bundles

These bundles are the strongest reconstruction of the intended control-plane behavior.

### Bundle A: Bootstrap And Readiness

- read `WORKFLOW.md`
- verify the watched queue contract
- verify auth/tooling availability
- return a structured readiness result
- expose runtime metadata after MCP edits

### Bundle B: Issue Intake And Legitimacy

- classify incoming work as triage, validated backlog, docs hub, or queue candidate
- use frontier mandate rules, not only title keywords

### Bundle C: Queue Reconciliation

- compare watched project, active states, and current issue distribution
- identify stale `Human Review`, `Merging`, and empty-queue conditions
- return refill actions only when staging is honest

### Bundle D: Phase Planning And Staging

- read spec/task artifacts
- preserve blocker chains
- emit runnable slices, blocked slices, and prep slices
- stage only truly runnable work into the watched queue

### Bundle E: Execution Inspection

- resolve issue state, current PR state, and repo state together
- treat the workpad as part of durable execution context

### Bundle F: Autonomous Review And Merge

- collect review evidence from GitHub comments, review summaries, vet, Claude review, and required checks
- classify the current issue into:
  - ready for `Merging`
  - needs `Rework`
  - requires human escalation
- prioritize clearing `Merging` and `Human Review` pressure before queue refill

### Bundle G: Recovery And Reconciliation

- reload stale MCP runtime state
- recover from drift between issue state, PR state, and repo state
- resume from durable artifacts instead of hidden session memory

## What The Current Rebuild Still Misses

Compared with the evidence above, the rebuilt compatibility server still under-models several areas:

1. `route_workflow_request` is still too keyword-driven.
2. `sync_linear_with_runtime` is still too snapshot-like and not enough of a queue-policy evaluator.
3. `review_merge_dispatch_cycle` is still not a true autonomous review gate.
4. `evaluate_issue_legitimacy` and `classify_follow_up_work` still need to use the frontier mandate
   more directly.
5. `plan_phase_execution` should rely more on spec/task artifacts and less on title heuristics.

## Most Likely Intended Bigger Picture

The repo evidence points to this larger control-plane architecture:

- Linear is the durable work planner and state machine.
- Symphony is the unattended issue executor against one watched project.
- GitHub is the code review, CI, and merge surface.
- The `smith` MCP is the missing cross-system brain that:
  - understands the repo mandate
  - reconciles queue truth
  - stages real work
  - drives autonomous review and merge loops
  - keeps long-running work resumable and policy-aligned

**Inference**:

The logical endpoint of this design is exactly what the user described:
an MCP tailored to Mister Smith, Linear, and Symphony for long-running autonomous workflows where
human review becomes policy-driven autonomous review by default, not a permanent manual bottleneck.

This is a medium-confidence inference, but it is strongly supported by the direction of the docs,
skills, workflow states, and PR automation.

## Next Implementation Targets

1. Make `review_merge_dispatch_cycle` the real autonomous review controller.
2. Tighten `sync_linear_with_runtime` into a queue-policy evaluator.
3. Tighten `route_workflow_request` into workflow-bundle routing.
4. Move legitimacy and follow-up classification onto frontier-mandate rules.
5. Make `plan_phase_execution` explicitly spec/task driven for staging honesty.
