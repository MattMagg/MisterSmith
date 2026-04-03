# Branch 025 Merge Triage

## Objective

Decide whether `origin/025-step-level-intelligence-v2` contains branch-only work that should be
landed after PR #266 merged packet 025 to `main`.

## Scope

- Compare branch-only commits against current `main`
- Land only valid, still-missing changes
- Keep packet-025 scope limited to step-policy and evidence behavior

## Assumptions

- PR #266 already landed the packet-025 base implementation on `main`
- The remote branch may still contain valid follow-up commits after the squash merge

## Constraints

- Do not merge the stale branch wholesale if it replays already-landed history
- Validate the real behavior affected by any selected follow-up commit
- Leave `main` clean and synced after push

## Non-goals

- No runtime redesign
- No new packet scope beyond step-policy/evidence follow-up fixes

## Milestones

1. Triage branch-only commits
   - Validation: compare commit content against `main` and classify as already-landed or missing
2. Land selected missing fixes
   - Validation: targeted Rust tests, operator-console tests, markdown lint for `AGENTS.md`
3. Close on clean `main`
   - Validation: `git diff --check` and `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Stop Conditions

- Stop if the branch-only fix conflicts with current packet-025 behavior or repo guidance
- Stop if validation shows the follow-up commit is no longer correct on current `main`
