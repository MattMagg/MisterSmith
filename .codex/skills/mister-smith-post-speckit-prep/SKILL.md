---
name: mister-smith-post-speckit-prep
description: Use after a Mister Smith SpecKit packet has been created and needs post-spec closure: packet verification, git closure, Linear materialization, honest execution-lane preparation, and follow-on execution readiness without implementation unless explicitly requested.
---

# Mister Smith Post-SpecKit Prep

Use this skill when a SpecKit packet already exists and must be turned into honest executable
development work.

This is a post-packet workflow skill, not a packet-writing skill and not an implementation skill.

## When To Use

Trigger this skill when the user wants any of the following after a packet already exists:

- land the packet on `main`
- verify the packet still matches repo truth
- materialize the packet into Linear
- preserve blocker chains from `tasks.md`
- stage only the honest runnable slices into the active execution lane
- reconcile status-bearing docs, notes, logs, readmes, and artifact indexes so they match the landed packet
- stop at “ready for Symphony”
- optionally launch Symphony only if the user explicitly asks

## Do Not Use

Do not use this skill when:

- the packet has not been written yet
- the request is to revise packet scope or write packet files
- the request is to implement packet tasks
- the request is to do unrelated repo cleanup, provider work, or queue filling

## Required Inputs

- packet path, for example:
  - `specs/016-external-agent-boundary-continuity-and-runtime-proof/`
- current repo authority docs:
  - `AGENTS.md`
  - `docs/current-state.md`
  - forward checkpoint note currently in force
  - `WORKFLOW.md`
  - `docs/linear/LINEAR.md`

Optional:

- the parent Linear issue if it already exists
- explicit instruction to launch Symphony after staging

## Core Rules

- Treat the packet as the source of truth unless current repo truth proves it stale or wrong.
- Verify first, mutate second.
- Use [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md) for git closure.
- Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md) plus Smith MCP tools for packet-to-Linear translation and execution-lane prep.
- Check and refresh status-bearing repo docs, notes, logs, readmes, and artifact indexes before calling the packet closed.
- Do not widen into implementation.
- Do not stage blocked slices just to keep Symphony busy.
- This repo forbids new git worktrees for packet closure. If unrelated legacy worktrees exist,
  document them and leave them alone.
- Do not touch unrelated PRs or branches.
- Stop and report if the packet is stale, materially wrong, or no slice is honestly runnable.

## Workflow

### 1. Start sequence

Read, in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. the active scope-freeze or closure note currently named by `docs/current-state.md`
4. `WORKFLOW.md`
5. `docs/linear/LINEAR.md`
6. `<packet>/spec.md`
7. `<packet>/plan.md`
8. `<packet>/tasks.md`
9. `<packet>/analyze.md`

Also inspect the current status-bearing repo surfaces that may need reconciliation when the packet
lands:

- `README.md`
- `ROADMAP.md`
- `CLAUDE.md`
- `docs/current-state.md`
- `docs/ms_recent_context.md`
- the active forward checkpoint note
- relevant packet evaluation or closure notes under `docs/plans/`
- relevant `docs/plans/artifacts/.../README.md` files for proof bundles
- `WORKFLOW.md` and `docs/linear/LINEAR.md` only when the packet changed workflow contract or
  queue semantics

Then audit local state:

```sh
git status --short --branch
gh pr list --state open --limit 30
```

### 2. Packet verification

Before touching git or Linear, verify:

- required packet files exist
- packet scope still matches current repo truth
- `tasks.md` blocker language is internally consistent
- packet-only markdown lint passes

Run narrow validation for the packet only.

If current repo truth shows the packet is stale or materially wrong:

- stop
- report the mismatch
- do not force closure or Linear materialization

### 2a. Status-surface audit

Before declaring the packet closed, identify all status-bearing docs, notes, logs, readmes, and
artifact indexes that mention the packet, its issue lineage, or “current” repo state.

Minimum audit targets:

- repo entry points:
  - `README.md`
  - `ROADMAP.md`
  - `CLAUDE.md`
- current-state routers:
  - `docs/current-state.md`
  - `docs/ms_recent_context.md`
  - the active forward checkpoint note
- packet proof or closure notes:
  - packet evaluation or closure notes under `docs/plans/`
  - any follow-up note that became current authority
- artifact indexes:
  - `docs/plans/artifacts/.../README.md` for logs, JSON captures, screenshots, or proof bundles
- workflow docs:
  - `WORKFLOW.md`
  - `docs/linear/LINEAR.md`
  - only when the packet changed development workflow contract

What to verify:

- no status-bearing doc still describes the packet as upcoming after it landed
- no doc claims a surface is still missing when it was landed by the packet
- no doc claims runtime proof or implementation happened when the packet stopped at planning-only or
  deterministic-only proof
- packet numbers, issue identifiers, and closure state are consistent
- artifact bundles have an index README instead of leaving raw files unframed

Useful search pattern:

```sh
rg -n "Status:|Current State|Current Direction|What Is Planned Next|packet 0|MS-" \
  README.md ROADMAP.md CLAUDE.md docs specs
```

If the landed packet changed current repo truth, update the affected status surfaces in the same
closure session unless the user explicitly narrows scope.

### 3. Git closure

Use [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md).

Required flow:

1. review the packet-owned diff
2. keep only packet work in scope
3. commit the packet cleanly
4. push the correct branch or `main`
5. complete review/merge if the lane is branch-based
6. run:

```sh
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

Do not continue to Linear sync until the closure gate passes and the primary repo checkout is back
on a clean synced `main`.

### 4. Linear materialization

After the packet is landed on `main`, create or update its Linear representation.

Parent issue:

- one parent issue in `MisterSmith Validated Backlog`
- include:
  - packet path
  - epic summary
  - in-scope work
  - deferred work
  - validation or proof shape
  - explicit note that the issue comes from the landed packet

Child slices:

- prefer `translate_speckit_tasks` when it maps honestly from `tasks.md`
- otherwise use `materialize_backlog_slices`

For every child:

- preserve the packet task identifier when present
- preserve blocker chains exactly
- keep the slice bounded and runnable
- keep blocked or future work in `MisterSmith Validated Backlog`
- do not move everything into active execution

### 5. Honest execution-lane prep

Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md).

Required flow:

1. `translate_speckit_tasks` when `tasks.md` is honest enough
2. otherwise `materialize_backlog_slices` with explicitly bounded slices
3. move only the truly runnable slices into the active Linear execution lane

Rules:

- preserve blocker order
- do not stage blocked slices
- do not move backlog work into `Todo` just to manufacture parallelism
- real parallelism should come from multiple genuinely unblocked slices

If there are no runnable slices yet:

- stop
- report that clearly

### 6. Repo-status reconciliation

Before stopping, update the status-bearing docs and artifact indexes identified in the status audit
when the landed packet changed what is true on `main`.

Rules:

- update only the surfaces actually affected by the packet
- keep historical notes honest instead of silently rewriting history
- if a packet produced live proof artifacts, ensure the artifact directory has a readable
  `README.md` explaining what the files are and how they relate to the packet
- if a note became authority, point the router docs to it

Validation:

- narrow `markdownlint` on every changed doc or skill file
- `git diff --check`

### 7. Symphony readiness

Default stop point:

- packet landed on `main`
- parent issue created or updated
- child slices created
- honest runnable slices moved into the active execution lane
- repo clean and synced
- ready for Symphony dispatch

Do not launch Symphony automatically unless the user explicitly asks.

If the user explicitly asks to launch Symphony:

- launch it only after confirming the intended runnable slices are already in the active execution lane
- confirm which runnable slices were picked up
- report only completed actions and blockers

## Final Response Requirements

Always include:

- packet path
- landed commit hash
- closure proof that `main` is clean and synced
- parent Linear issue identifier and URL
- child slice identifiers created
- blocker-chain summary
- which slices were moved into the active execution lane
- which status-bearing docs or artifact indexes were updated
- whether Symphony was launched or intentionally not launched
- any blockers preventing the next execution stage

Do not claim implementation progress unless implementation was explicitly requested.

## Reusable Prompt Template

Use this as the operator prompt after a packet exists. Replace the placeholders before sending it
to another agent.

```text
You are working in the Mister Smith repository at `/Users/macmain/MisterSmith`.

Your mission is to take the completed SpecKit packet at:

`<PACKET_PATH>`

and finish the post-SpecKit workflow cleanly and honestly so the packet becomes executable
development work.

This is a packet closure + Linear materialization + honest queue staging session.

It is not a packet-writing session and it is not an implementation session unless explicitly
instructed later to launch execution.

## Objective

By the end of this session, you must have:

1. verified the packet is complete and still matches current repo truth
2. closed the git state cleanly for the packet work
3. committed and pushed the packet to `main`
4. synchronized the packet into Linear as one parent epic issue plus child slices
5. preserved blocker chains and placed slices in the correct project and state
6. moved only the honest runnable slices into the active execution lane for Symphony
7. reconciled status-bearing docs, notes, logs, readmes, and artifact indexes so they reflect the current state honestly
8. stopped with the repo clean and with a clear report of what is now ready for follow-on execution

## Core Rules

- Follow `AGENTS.md` first.
- Treat `docs/current-state.md` and `<FORWARD_CHECKPOINT_PATH>` as current authority.
- Treat `WORKFLOW.md` and `docs/linear/LINEAR.md` as the control-plane contract.
- Use the existing packet as the source of truth. Do not rewrite scope unless current repo truth proves the packet is wrong.
- Use [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md) for git closure.
- Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md) plus Smith control-plane tools for translating and staging work.
- identify and refresh all status-bearing docs, notes, logs, readmes, and artifact indexes touched by the packet so they match the landed repo truth
- Do not stage blocked slices just to keep Symphony busy.
- Do not widen into implementation, provider work, JetStream KV work, or unrelated cleanup.
- Do not touch unrelated worktrees, PRs, or branches.

## Start Sequence

Read these in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. `<FORWARD_CHECKPOINT_PATH>`
4. `WORKFLOW.md`
5. `docs/linear/LINEAR.md`
6. `<PACKET_PATH>/spec.md`
7. `<PACKET_PATH>/plan.md`
8. `<PACKET_PATH>/tasks.md`
9. `<PACKET_PATH>/analyze.md`

Also inspect the status-bearing repo surfaces that may need updates:

- `README.md`
- `ROADMAP.md`
- `CLAUDE.md`
- `docs/current-state.md`
- `docs/ms_recent_context.md`
- `<FORWARD_CHECKPOINT_PATH>`
- relevant packet closure or evaluation notes under `docs/plans/`
- relevant `docs/plans/artifacts/.../README.md` files
- `WORKFLOW.md` and `docs/linear/LINEAR.md` when workflow contract changed

Then audit current local state:

- `git status --short --branch`
- `git worktree list`
- `gh pr list --state open --limit 30`

## Phase 1: Packet Verification

Before touching git or Linear, verify:

- the packet files all exist
- packet scope is still aligned with current repo truth
- task and blocker language is internally consistent
- packet-only markdown lint passes

Run the narrow validation for this packet only.

If current repo truth shows the packet is stale or materially wrong, stop and report that instead
of forcing closure.

## Phase 1a: Status Audit

Before closure, audit all status-bearing docs, readmes, instructions, notes, logs, and artifact
indexes that mention the packet, issue lineage, or current repo direction.

Verify:

- they do not describe landed packet work as still upcoming
- they do not omit newly landed proof or closure notes
- they do not claim proof or implementation happened when it did not
- packet numbers, issue identifiers, and closure state are consistent
- raw logs or proof bundles are indexed by a readable artifact `README.md`

If the packet changed current repo truth, update those surfaces in the same session.

## Phase 2: Git Closure

Use [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md) and finish the packet work cleanly.

Required flow:

1. review the packet-owned diff
2. keep only the packet work in scope
3. commit the packet cleanly
4. push
5. run:

`scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

Do not continue to Linear sync until local closure is clean and synced.

## Phase 3: Linear Materialization

After the packet is landed on `main`, create or update the Linear representation for this packet.

Parent issue:

- one parent issue in `MisterSmith Validated Backlog`
- include packet path, epic summary, in-scope work, deferred work, validation or proof shape, and
  an explicit note that this issue comes from the landed packet

Child slices:

- prefer `translate_speckit_tasks`
- if translation is not honest enough, use `materialize_backlog_slices`

For every child:

- preserve packet task identifiers when present
- preserve blocker chains exactly
- keep slices bounded and runnable
- keep blocked or future work in `MisterSmith Validated Backlog`
- do not move everything into active execution

## Phase 4: Honest Execution-Lane Prep

Once the slices exist in Linear:

1. use `translate_speckit_tasks` when the packet maps cleanly from `tasks.md`
2. otherwise use `materialize_backlog_slices`
3. move only the honest runnable slices into the active execution lane

Rules:

- preserve blocker order
- do not stage blocked slices
- do not move backlog work into `Todo` just to create parallelism
- parallel Symphony execution should come from genuinely unblocked slices only

If there are no runnable slices yet, stop and report that clearly.

## Phase 5: Repo-Status Reconciliation

Before stopping, update the docs, notes, logs, readmes, and artifact indexes identified in the
status audit.

Use narrow validation:

- `markdownlint-cli2` on every changed doc
- `git diff --check`

## Phase 6: Symphony Readiness

Do not start implementation work automatically unless explicitly instructed to do so.

Default stop point:

- packet landed on `main`
- parent Linear issue created or updated
- child slices created
- runnable slices moved honestly into the active execution lane
- repo clean and synced
- ready for Symphony dispatch

If the user explicitly asks to start Symphony now:

- launch Symphony attached
- confirm it is watching the expected queue
- confirm which staged slices were picked up
- report only completed actions and blockers

## Final Response Requirements

Your final response must include:

- packet path
- commit hash that landed the packet
- closure proof that `main` is clean and synced
- parent Linear issue identifier and URL
- child slice identifiers created
- blocker-chain summary
- which slices were moved into the active execution lane
- which status-bearing docs or artifact indexes were updated
- whether Symphony was launched or intentionally not launched
- any blockers that prevent the next execution stage

Do not claim implementation progress on the packet tasks unless implementation was explicitly requested.
```

## Related Skills

- [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md)
- [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md)
- `mister-smith-control-plane-router`
- `symphony-linear-mister-smith`
