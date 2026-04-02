# Repo Doc Sync After Git Sync

Date: April 2, 2026
Status: Current

## Objective

Sync repo-wide current-state, router, and instruction docs after stashing local work and
fast-forwarding `main` so the docs match the landed git history.

## Repo Truth At Sync Time

- `main` is clean and synced at `4d1d044b0879a284a07dbe1e549fb9e7c387f0d8`
- stash `pre-sync-doc-refresh-2026-04-02` preserves the pre-sync local work
- packet `023` is landed on `main` via `4d1d044`
- packet `024` is landed on `main` via `59e4ca2`
- packet `025` remains the next draft scaffold under `specs/025-step-level-intelligence-v2/`

## Scope

- repo routers and contributor entry points
- current-state and tracking docs
- broken references to the missing April 1 packet-022 closure note

## Non-Goals

- code changes
- queue-policy or workflow-contract changes
- claiming fresh live runtime proof for packets `021` through `024`
- reopening landed packet implementations

## Milestones

### Milestone 1: Audit Synced Truth

Validation:

- clean synced `main`
- stash preserved
- recent landed packet commits identified

### Milestone 2: Patch Current Routers

Validation:

- `README.md`, `CLAUDE.md`, `AGENTS.md`, `docs/current-state.md`, `docs/direction.md`,
  `docs/linear/LINEAR.md`, and `docs/ms_recent_context.md` all point at landed packet `023`,
  landed packet `024`, and next scaffold packet `025`

### Milestone 3: Repair Broken Packet References

Validation:

- no touched current-facing doc still points at
  `docs/plans/2026-04-01-packet-022-durable-workflow-core.md`

### Milestone 4: Validate The Doc Pass

Validation:

- `git diff --check`
- targeted markdownlint on touched state-tracking docs

## Stop Conditions

- no current-facing doc still says that no post-packet-020 packet is frozen
- no current-facing doc still describes packet `023` as active work
- no touched doc points at the missing April 1 packet-022 closure note
- landed deterministic validation stays clearly separated from the older live-proof baseline
