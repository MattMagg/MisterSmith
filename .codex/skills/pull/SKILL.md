---
name: pull
description: |
  Pull the latest `origin/main` into the current branch with a merge, resolve
  conflicts carefully, and rerun the right validation for this Rust workspace.
---

# Pull

Use this skill when the current branch needs to catch up with `origin/main`.

## Workflow

1. Confirm the current branch is the one that should receive the merge.
2. Inspect `git status` before starting.
3. Enable rerere locally:
   - `git config rerere.enabled true`
   - `git config rerere.autoupdate true`
4. Fetch latest refs:
   - `git fetch origin`
5. Sync the remote copy of the current branch first:
   - `git pull --ff-only origin $(git branch --show-current)`
6. Merge `origin/main` with conflict context:
   - `git -c merge.conflictstyle=zdiff3 merge origin/main`
7. If conflicts appear:
   - inspect the intent on both sides before editing
   - resolve one file at a time
   - run `git diff --check` before finalizing
8. After the merge, rerun validation that matches the scope:
   - baseline: `cargo build --workspace`
   - affected crates: `cargo test -p <crate>`
   - broader checks only when the change crosses shared contracts or CI-critical surfaces
9. Record the merge result and any notable conflicts in the Linear workpad.

## Ask only when necessary

Proceed unless the conflict changes user-visible or external behavior in a way that cannot be inferred from code, tests, docs, or the Linear issue.
