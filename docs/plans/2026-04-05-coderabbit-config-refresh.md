# CodeRabbit Config Refresh

## Objective

Replace the minimal repository CodeRabbit config with a repo-specific `.coderabbit.yaml` that fits
Mister Smith's Rust-first workspace, docs-heavy packet flow, local-validation posture, and
low-noise review expectations.

## Scope

- `/Users/macmain/MisterSmith/.coderabbit.yaml`
- lightweight planning note for this config refresh

## Assumptions

- The installed CodeRabbit app is already connected to the repository.
- Repo-local guidance should be the primary review context, with Linear used only as linked-issue
  read context.
- Automatic issue planning and chat-driven tracker mutations would duplicate the existing
  Smith/SpecKit control-plane workflow.

## Constraints

- Keep the config schema-valid for CodeRabbit v2.
- Favor plain-English, actionable review output over decorative walkthrough content.
- Reduce duplicate or irrelevant tool noise where the repo already has a clear local stack.

## Non-Goals

- No GitHub branch, PR, or Linear state changes
- No new review tooling files outside `.coderabbit.yaml`
- No changes to repo validation scripts or workflow docs beyond this note

## Milestones

1. Capture the repo-specific review contract.
   Validation: confirm current repo guidance from `AGENTS.md`, `docs/current-state.md`,
   `WORKFLOW.md`, `.github/workflows/README.md`, and the existing `.coderabbit.yaml`.
2. Rewrite `.coderabbit.yaml` around actual repo surfaces.
   Validation: config includes scoped review guidance for Rust crates, operator-console,
   scripts, docs/specs, deploy files, knowledge-base usage, and pre-merge checks.
3. Validate syntax and diff quality.
   Validation: parse the YAML successfully and inspect the final diff for scope and readability.

## Stop Conditions

- Stop if CodeRabbit docs or schema requirements conflict in a way that cannot be resolved from
  the published reference.
- Stop if the repo already relies on organization-level CodeRabbit settings that make a root-level
  override unsafe.
