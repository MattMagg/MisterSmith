# Remove Vet From Active Workflow

Date: March 16, 2026
Status: Completed

## Objective

Remove `vet` from the active Mister Smith development workflow so the repository no longer treats it as a required or expected review step.

## Scope

- remove the active GitHub Actions `vet` workflow
- remove repo-local `vet` config, wrapper, and skill surfaces
- update current repo guidance so active workflow docs no longer instruct agents to use `vet`

## Assumptions

- historical docs may keep old `vet` references as execution history
- compile, test, clippy, and markdown validation remain the active deterministic validation paths

## Constraints

- preserve unrelated local drift by doing this work in a clean worktree
- do not rewrite historical plan artifacts unless required for current workflow clarity

## Non-Goals

- redesign the full validation stack beyond removing `vet`
- clean or rewrite unrelated uncommitted docs in the original checkout

## Milestones

1. Remove active `vet` implementation surfaces.
   Validation: targeted search shows no active workflow/config/script/skill files remain.
2. Reconcile active docs.
   Validation: touched docs lint clean and describe the new active workflow accurately.

## Result

- active `vet` workflow/config/script/skill surfaces removed
- active repo guidance no longer routes operators or agents through `vet`
- stale `WinYear` references replaced with `frontier-direction` in active guidance docs
- historical plan artifacts keep older `vet` references as execution history

## Stop Conditions

- another active workflow depends on repo-local `vet` files in a way that would break current CI or queue execution
- removal leaves contradictory active guidance that cannot be resolved from repo authority files
