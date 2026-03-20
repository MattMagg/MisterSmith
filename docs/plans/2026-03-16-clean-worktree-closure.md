# Clean Worktree Closure

Date: March 16, 2026
Status: Completed on `main`; clean-and-synced closure is now a repo workflow requirement

## Objective

Make clean, reviewable, pushed repository state a hard closure requirement for Mister Smith
workflows so unattended sessions do not leave behind local modifications, untracked files, or
unpushed commits.

## Scope

- add one repo-local verification script for clean-and-synced closure checks
- update repo workflow contracts to require clean closure before review, merge, and done
- update repo skills so `commit`, `push`, and `land` enforce the same rule
- reconcile the current dirty primary checkout after the contract change is landed

## Assumptions

- Symphony issue workspaces start from a clean clone, so the main failure mode is incomplete
  closure rather than dirty startup state
- stale or superseded leftovers should be explicitly reviewed and dropped instead of preserved
  indefinitely

## Constraints

- preserve the existing `Human Review` and `Merging` state names
- do not introduce a second workflow engine outside repo contracts and skills
- keep the closure check shell-only and host-compatible

## Non-Goals

- redesign the full Linear/Symphony state model
- enforce cleanliness through GitHub Actions alone
- preserve obviously stale local leftovers just because they exist

## Milestones

1. Add a deterministic closure-check script and wire it into the active workflow contracts.
   Validation: script passes in a clean repo and fails with clear output on dirty state.
2. Update commit/push/land guidance to require closure before handoff and after merge.
   Validation: targeted doc lint and readback of the touched workflow surfaces.
3. Reconcile the current dirty primary checkout against `origin/main`.
   Validation: the primary checkout ends the run on a clean local branch with no leftover changes.

## Stop Conditions

- enforcing clean closure would contradict an existing repo-owned workflow contract
- the current dirty checkout contains ambiguous user work that cannot be safely classify as valid,
  stale, or already landed
