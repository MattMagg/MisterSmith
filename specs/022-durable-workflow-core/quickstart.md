# Quickstart: Durable Workflow Core

## Purpose

This quickstart is for starting packet `022` implementation on current `main`.

## Step 1: Kickoff Check

Run this quick check first:

```bash
git status --short --branch
cat docs/current-state.md
cat docs/direction.md
cat docs/research-output/analysis/2026-03-28-durable-workflows-transfer-brief.md
cat docs/plans/2026-03-19-session-restart-resume-live-proof.md
```

Then re-read these touched seams:

```bash
cat crates/mister-smith-agents/src/branch_checkpoint.rs
cat crates/mister-smith-persistence/src/kv/state.rs
cat crates/mister-smith-persistence/src/hybrid/manager.rs
cat crates/mister-smith-persistence/src/repository/task.rs
cat crates/mister-smith-app/src/conversation.rs
cat docs/plans/2026-03-19-session-restart-resume-live-proof.md
```

If those seams materially contradict packet `022`, fix the packet docs first, then continue.

## Step 2: Revalidate The Packet Boundaries

Confirm these boundaries still hold:

- packet `022` owns durable workflow semantics, lifecycle verbs, effect boundaries, and bounded
  compaction plus replay governance
- packet `022` does not absorb coordinator runtime, interoperability, or strong coordination
- current session continuity and restart-resume proof remain preserved baseline behavior

## Step 3: Run Packet Doc Validation

```bash
npx markdownlint-cli2 "specs/022-durable-workflow-core/**/*.md" --config .markdownlint.json
```

## Step 4: Start The Task Pack

- Review `tasks.md`
- Review `design.md`
- Use the frozen first-slice decisions from the packet docs:
  - canonical workflow history stored on the SQL-backed workflow record
  - supported lifecycle verbs: `pause`, `resume`, `cancel`, `terminate`
  - deferred lifecycle posture: `reset/rewind`
  - lifecycle decisions are recorded durably with `applied`, `noop`, or `deferred` outcomes
  - live runner pause, resume, cancel, or terminate control is not claimed by packet `022`
  - persistence-owned effect intent and outcome records
  - minimal lineage-preserving compaction record with replay start pointer
- Then move straight into `/speckit.implement`

## Expected Outcome

After this quickstart, the packet should:

- remain aligned with current `main`
- have any real contradiction fixed
- be ready for immediate packet `022` implementation
