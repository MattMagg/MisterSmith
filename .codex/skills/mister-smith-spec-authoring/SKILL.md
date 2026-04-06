---
name: mister-smith-spec-authoring
description: "Use when Mister Smith needs spec-only frontier packet work: turning current repo truth or a pre-SpecKit primer into a new product-side SpecKit packet, or running the full local SpecKit authoring chain for an already chosen packet. Trigger for requests to create or extend a Mister Smith packet spec, pre-SpecKit primer, specify-init prompt, or full spec bundle without implementation."
---

# Mister Smith Spec Authoring

Use this skill for new Mister Smith packet authoring and other spec-only frontier planning work.

Coordinate the repo-specific workflow around Smith legitimacy checks, product-boundary scoping,
pre-SpecKit primer validation, and the local SpecKit artifact chain. Do not use this skill for
implementation, post-packet staging, runtime proof runs, or general repo-doc sync.

## Required inputs

- the planning target:
  - a new bounded packet or feature slice
  - an existing pre-SpecKit primer
  - or an existing packet spec that must be extended
- the primary source note or prompt, when one exists
- explicit scope if the user wants less than the full local SpecKit chain

## Core rules

- Start with Smith MCP authoring prep, legitimacy, and scope routing:
  - `prepare_spec_authoring`
  - `evaluate_issue_legitimacy`
  - `classify_follow_up_work`
- Keep the work product-side unless the user explicitly asks for repo-workflow surfaces.
- Treat `docs/current-state.md` as shipped truth and `docs/direction.md` as strategic priority.
- Do not auto-promote later packet material just because `027`, `028`, or `029+` scaffolds exist.
- Default to the full local SpecKit chain once the packet boundary is chosen. Stop earlier only
  when the user explicitly narrows scope.
- Keep deterministic spec validation separate from implementation or runtime-proof claims.

## Start sequence

Read, in order:

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/direction.md`
4. `docs/ms_recent_context.md`
5. the latest landed packet authorities that bound the new slice
6. the user-provided primer, proposal, or existing packet files

If the ask is frontier-planning only, also use
[$mister-smith-frontier-mandate](../mister-smith-frontier-mandate/SKILL.md) to keep the packet
legitimate and bounded.

Use `prepare_spec_authoring` before running the local SpecKit chain so the authoring mode, entry
surface, stop stage, and validation posture are explicit.

If the user already provided a pre-SpecKit primer or packet-prep dossier, verify that it:

- states the product problem and bounded future packet shape
- separates current repo truth from future direction
- avoids implementation or task-pack claims
- names the exact follow-up questions the later packet must settle

Tighten the primer first if those boundaries are fuzzy.

## Workflow

### 1. Freeze the packet boundary

Choose one honest bounded slice before running SpecKit.

Confirm:

- what already landed on `main`
- what remains draft or pre-spec
- whether the new slice is product-side or workflow-side
- whether the packet should stop at `spec.md` or continue through the full local chain

Do not let the packet drift into Linear, Symphony, Ralph, or generic admin-console glue unless the
user explicitly wants that.

### 2. Choose the correct entry surface

Use the source that matches the task:

- pre-SpecKit primer or packet-prep dossier when the packet shape still needs freezing
- existing `spec.md` when the packet already exists and needs extension
- a new bounded feature prompt only when no primer exists and repo truth is already sufficient

When the task starts from a new idea, produce or refine the specify-init prompt so the next
SpecKit step is grounded in repo truth and the intended packet boundary.

### 3. Run the local SpecKit chain

Default sequence for new packet authoring:

1. `speckit.specify`
2. `speckit.clarify`
3. generate the custom packet checklist when the base requirements checklist is too generic
4. `speckit.plan`
5. `speckit.tasks`
6. `speckit.analyze`

If the user explicitly says `spec-only`, stop after `speckit.specify` or the narrower stage they
requested.

When running the full chain:

- treat `.specify` script output as the source of truth for branch and file paths
- keep the packet numbering and short name explicit
- review any `.specify/extensions.yml` hooks before assuming they are safe to run
- keep repo-wide side effects explicit, especially agent-context updaters that touch `AGENTS.md`

### 4. Keep the packet grounded while authoring

While writing the spec bundle:

- reuse current CLI, HTTP, session, autonomy, or operator seams instead of speculating
- keep the packet honest about default-path truth versus future work
- make the bounded slice obvious in `spec.md`, `plan.md`, and `tasks.md`
- keep implementation details out of the spec unless they are required to define the contract

### 5. Validate the packet docs

Run the narrowest honest validation for the artifact set you produced.

Typical checks:

```sh
.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
npx markdownlint-cli2 "specs/<packet>/**/*.md" --config .markdownlint.json
git diff --check
```

If the task stopped before `tasks.md`, skip `--require-tasks` and validate only the stages that
exist.

### 6. Land doc-only packet work only when asked

If the user asks to land the packet docs:

- keep the commit scoped to the packet artifacts and any required repo-router updates
- commit directly to `main`
- if the current checkout is dirty or on the wrong branch, move only the task-owned files into a
  clean `origin/main` lane before pushing
- run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` after push

Do not widen doc-only packet work into implementation or backlog mutations unless the user asks.

## Related surfaces

- [$mister-smith-frontier-mandate](../mister-smith-frontier-mandate/SKILL.md) for legitimacy and
  frontier-scope judgment
- [$mister-smith-control-plane-router](../mister-smith-control-plane-router/SKILL.md) when Smith
  workflow routing is needed before or alongside the packet work
- `.codex/commands/specify.md`, `.codex/commands/clarify.md`, `.codex/commands/plan.md`,
  `.codex/commands/tasks.md`, and `.codex/commands/analyze.md` for the concrete local SpecKit
  steps this skill coordinates
