# Quickstart: Durable Workflow Core

## Purpose

This quickstart is for the later session that turns packet `022` from scaffold into an
implementation-ready packet.

## Step 1: Refresh Before Coding

Run this refresh pass first:

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

If those seams changed materially, refresh this packet before implementation.

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

## Step 4: Use The Tasks Only After Refresh

- Review `tasks.md`
- Review `design.md`
- Mark any refresh-dependent items before coding
- Only then move to implementation planning or `/speckit.implement`

## Expected Outcome

After this quickstart, the packet should either:

- remain valid with only light wording updates, or
- get one bounded refresh before implementation starts
