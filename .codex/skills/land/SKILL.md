---
name: land
description: |
  Land the current PR by resolving merge conflicts, handling feedback, waiting
  for checks, and squash-merging when the branch is ready.
---

# Land

Use this skill when a Symphony issue moves to `Merging`.

## Goals

- Keep the PR conflict-free with `origin/main`.
- Make sure feedback is addressed.
- Wait for checks to complete and only merge when green.
- Leave the issue workspace on a clean `origin/main` checkpoint after merge.

## Preconditions

- `gh` is installed and authenticated.
- You are on the PR branch.
- The working tree is clean or intentionally committed.

## Workflow

1. Identify the PR for the current branch.
2. Rerun the current scope's validation before any final push.
3. Check mergeability:
   - `gh pr view --json number,title,body,mergeable,url`
4. If the PR conflicts with `main`, run the `pull` skill, resolve conflicts, revalidate, and push.
5. Review open PR feedback:
   - top-level comments
   - inline review comments
   - review summaries
6. Treat every actionable review comment as blocking until it is addressed in code or explicitly pushed back on with rationale.
7. Wait for checks:
   - `gh pr checks --watch`
8. If checks fail:
   - inspect failing runs with `gh pr checks` and `gh run view --log`
   - fix the issue
   - commit, push, and re-run the watch
9. When all checks are green and feedback is resolved, squash-merge:
   - `gh pr merge --squash --subject "$pr_title" --body "$pr_body"`
10. After merge, run:

    ```sh
    git fetch origin --prune
    branch=$(git branch --show-current)
    git switch -C main origin/main
    if [ "$branch" != "main" ]; then git branch -D "$branch"; fi
    scripts/verify_worktree_closure.sh
    ```

## Notes

- Do not use auto-merge.
- Do not merge while actionable review feedback is still open.
- Keep PR title and body aligned with the actual final scope.
- `Done` is not honest until the post-merge workspace cleanup is complete.
