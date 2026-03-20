---
name: mister-smith-post-speckit-prep
description: Use after a Mister Smith SpecKit packet has been created and needs post-spec closure: packet verification, git closure, Linear materialization, honest queue staging, and Symphony readiness without implementation unless explicitly requested.
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
- stage only the honest runnable slices for the watched queue
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
- Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md) plus Smith MCP tools for packet-to-Linear translation and queue staging.
- Do not widen into implementation.
- Do not stage blocked slices just to keep Symphony busy.
- Do not touch unrelated worktrees, PRs, or branches.
- Stop and report if the packet is stale, materially wrong, or no slice is honestly runnable.

## Workflow

### 1. Start sequence

Read, in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. the active forward checkpoint note
4. `WORKFLOW.md`
5. `docs/linear/LINEAR.md`
6. `<packet>/spec.md`
7. `<packet>/plan.md`
8. `<packet>/tasks.md`
9. `<packet>/analyze.md`

Then audit local state:

```sh
git status --short --branch
git worktree list
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

Do not continue to Linear sync until the closure gate passes.

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
- do not put everything into the watched queue

### 5. Honest queue staging

Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md).

Required flow:

1. `plan_phase_execution`
2. review runnable, blocked, and prep-only slices
3. `apply_phase_execution_plan` only for the honest runnable slices

Rules:

- preserve blocker order
- do not stage blocked slices
- do not move backlog work into `Todo` just to manufacture parallelism
- real parallelism should come from multiple genuinely unblocked slices

If there are no runnable slices yet:

- stop
- report that clearly

### 6. Symphony readiness

Default stop point:

- packet landed on `main`
- parent issue created or updated
- child slices created
- honest runnable slices staged
- repo clean and synced
- ready for Symphony dispatch

Do not launch Symphony automatically unless the user explicitly asks.

If the user explicitly asks to launch Symphony:

- launch it against the watched queue
- confirm which staged slices were picked up
- report only completed actions and blockers

## Final Response Requirements

Always include:

- packet path
- landed commit hash
- closure proof that `main` is clean and synced
- parent Linear issue identifier and URL
- child slice identifiers created
- blocker-chain summary
- which slices were staged into the watched queue
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
6. staged only the honest runnable slices into the watched queue for Symphony
7. stopped with the repo clean and with a clear report of what is now ready for Symphony

## Core Rules

- Follow `AGENTS.md` first.
- Treat `docs/current-state.md` and `<FORWARD_CHECKPOINT_PATH>` as current authority.
- Treat `WORKFLOW.md` and `docs/linear/LINEAR.md` as the control-plane contract.
- Use the existing packet as the source of truth. Do not rewrite scope unless current repo truth proves the packet is wrong.
- Use [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md) for git closure.
- Use [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md) plus Smith control-plane tools for translating and staging work.
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
- do not put everything into the watched queue

## Phase 4: Honest Queue Staging

Once the slices exist in Linear:

1. use `plan_phase_execution`
2. review runnable slices, blocked slices, and prep-only slices
3. use `apply_phase_execution_plan` to stage only the honest runnable slices

Rules:

- preserve blocker order
- do not stage blocked slices
- do not move backlog work into `Todo` just to create parallelism
- parallel Symphony execution should come from genuinely unblocked slices only

If there are no runnable slices yet, stop and report that clearly.

## Phase 5: Symphony Readiness

Do not start implementation work automatically unless explicitly instructed to do so.

Default stop point:

- packet landed on `main`
- parent Linear issue created or updated
- child slices created
- runnable slices staged honestly
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
- which slices were staged into the watched queue
- whether Symphony was launched or intentionally not launched
- any blockers that prevent the next execution stage

Do not claim implementation progress on the packet tasks unless implementation was explicitly requested.
```

## Related Skills

- [$mister-smith-git-closure](/Users/macmain/MisterSmith/.codex/skills/mister-smith-git-closure/SKILL.md)
- [$stage-mister-smith-phase](/Users/macmain/MisterSmith/.codex/skills/stage-mister-smith-phase/SKILL.md)
- `mister-smith-control-plane-router`
- `symphony-linear-mister-smith`
