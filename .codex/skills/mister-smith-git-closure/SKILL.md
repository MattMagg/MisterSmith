---
name: mister-smith-git-closure
description: |
  Finish Mister Smith git work cleanly end to end. Use when a task should end
  with no local leftovers for that task: commit and push small `main` changes,
  create or update PRs from worktrees or branch lanes, wait for review-agent
  activity, address review comments or review-agent commits, merge when ready,
  and restore the local checkout to a clean synced state while only documenting
  unrelated worktrees, PRs, or branches outside the current task scope.
---

# Mister Smith Git Closure

## Overview

Use this skill to close a Mister Smith task without leaving git debris behind.
The goal is honest closure for the current task scope, not broad repo janitorial work on unrelated
branches, worktrees, or PRs.

## Closure Principles

- Audit the current scope before mutating git state.
- Distinguish current-task state from unrelated repo state.
- Leave the current task either:
  - committed, pushed, and synced on `main`, or
  - committed, pushed, reviewed, merged, and cleaned up from its branch/worktree lane.
- Do not leave task-owned uncommitted changes behind.
- Do not force-push unless the user explicitly approves it.
- Do not close or rewrite unrelated PRs, worktrees, or branches; document them only.

## Workflow

### 1. Audit scope first

Run:

```sh
git status --short --branch
git worktree list
gh pr list --state open --limit 30
```

Then determine:

- is this task on `main` or on a branch/worktree?
- which changes belong to the current task?
- are there unrelated open PRs, worktrees, or branches?

If unrelated state exists, note it in the response and leave it untouched unless the user
explicitly broadens scope.

### 2. Review task-owned changes

Before committing:

- inspect the diff for task-owned files
- separate unrelated local changes from the current task if needed
- keep commits scoped by concern

Use separate commits when there are clearly separate units, for example:

- repo docs or implementation changes
- workflow or skill additions

### 3. Main-branch closure

Use this path when the current task is intentionally landing directly on `main`.

1. commit the task-owned changes
1. push `main`
1. run the closure gate:

```sh
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

1. stop only when the local `main` checkout is clean and synced

### 4. Branch or worktree closure

Use this path when the current task is on a feature branch or in a worktree.

1. commit the task-owned changes
1. push the branch
1. create or update the PR
1. wait for review-agent activity and checks
1. address review output
1. merge only when the PR is actually ready
1. return local state to a clean synced `main`

Useful commands:

```sh
git push -u origin HEAD
gh pr view --json number,url,state,mergeable,reviewDecision
gh pr checks --watch
gh pr view --comments
```

### 5. Review-agent waiting loop

After the PR is open, wait a reasonable interval for cloud review agents and checks.

Default posture:

- wait long enough for the first automated review/check cycle to complete
- then re-check PR comments, reviews, commits, and check results

Recommended checks:

```sh
gh pr checks
gh pr view --comments
gh pr view --json reviews,latestReviews,reviewDecision,commits,mergeable
```

Treat both of these as actionable review input:

- suggested changes in review comments
- direct commits pushed to the PR branch by review agents

If review agents pushed commits directly:

- inspect the new commits
- verify the changes are correct
- rerun the right validation for the final branch state
- continue toward merge only if the branch is still good

If review comments suggest changes:

- decide whether the suggestion is correct
- if yes, apply it cleanly, commit, push, and re-check
- if no, reply with a concise technical rationale and keep the branch honest

### 6. Merge rule

Merge only when all of the following are true:

- the PR is mergeable
- actionable review comments are resolved
- required checks are green
- the branch reflects the final desired task scope

Preferred sequence:

1. pull or merge `origin/main` if needed
2. rerun validation
3. push the final branch state
4. merge
5. switch local checkout back to clean synced `main`

Post-merge cleanup:

```sh
git fetch origin --prune
git switch -C main origin/main
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

Delete the task branch or worktree only if it belongs to the just-completed task and is no longer
needed.

### 7. If review is still pending

If you have already pushed the branch and waited a reasonable amount of time but review agents or
checks are still not done, stop there and end the response with a clear hold statement.

Use this exact style:

`Review is still pending for the current PR. Come back and tell me to check again.`

Do not pretend closure is complete while reviews are still outstanding.

## Validation

Use the narrowest honest validation for the task scope, then always run the closure gate when the
git flow is supposed to be complete:

```sh
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

For Mister Smith repo work, reuse the repo's normal scope-aware validation before final push or
merge, for example:

- `cargo build --workspace`
- targeted `cargo test -p <crate>`
- markdown lint or other task-specific checks

## Related skills

- `commit`: create a scoped conventional commit
- `pull`: merge `origin/main` safely into the current branch
- `push`: publish the branch and create or update the PR
- `land`: finish the PR merge loop and clean local state

Use this skill as the end-to-end closure coordinator when a task needs the whole git lifecycle,
not just one of those sub-steps.
