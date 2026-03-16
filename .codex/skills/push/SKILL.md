---
name: push
description: |
  Push the current branch to `origin` and create or update the matching GitHub
  PR for this repository.
---

# Push

## Goals

- Publish the current branch safely.
- Create a PR if one does not exist.
- Refresh the PR title and body so they reflect the real scope of the branch.

## Prerequisites

- `gh` is installed and authenticated.
- Local validation for the current scope has been rerun successfully.

## Validation guidance

Follow `AGENTS.md` rather than blindly running the full workspace suite.

- Baseline cross-crate proof: `cargo build --workspace`
- Affected-crate proof: `cargo test -p <crate>`
- Escalate to `cargo test --workspace` only when touching `mister-smith-core`, shared contracts, CI/workflows, or when the issue explicitly requires it.

## Steps

1. Identify the current branch.
2. Confirm the working tree is fully committed before push; do not leave local leftovers behind.
3. Push to `origin`:
   - `git push -u origin HEAD`
4. If push is rejected because the remote moved, run the `pull` skill, revalidate, and push again.
5. If push fails because of auth or permissions, stop and surface the exact error. Do not rewrite remotes as a workaround.
6. Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`.
7. Ensure a PR exists:
   - create one if missing
   - otherwise update the existing PR
8. Write a concise PR title and body that cover:
   - problem
   - solution
   - validation commands run
9. Reply with the PR URL and attach it to the Linear issue.

## Example commands

```sh
branch=$(git branch --show-current)
git push -u origin HEAD
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync

pr_state=$(gh pr view --json state -q .state 2>/dev/null || true)
if [ -z "$pr_state" ]; then
  gh pr create --title "<clear title>" --body-file /tmp/pr_body.md
else
  gh pr edit --title "<clear title>" --body-file /tmp/pr_body.md
fi

gh pr view --json url -q .url
```
