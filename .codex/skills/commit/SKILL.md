---
name: commit
description: |
  Create a conventional git commit from the current changes using the repo's
  session context, scope, and validation evidence.
---

# Commit

## Goals

- Produce a commit that matches the actual staged changes.
- Follow this repo's conventional commit style with a scope when it is clear.
- Record both what changed and how it was validated.

## Inputs

- Session history for intent and rationale.
- `git status`, `git diff`, and `git diff --staged` for actual scope.
- Repo commit guidance from `AGENTS.md`.

## Steps

1. Read the current session context and inspect the working tree.
2. If unexpected leftovers are present, decide whether they belong in this commit, a dedicated
   follow-up branch/PR, or should be dropped because they are already landed or stale.
3. Stage only the intended files.
4. Sanity-check the index for unrelated files, generated junk, or secrets.
5. Write a conventional subject such as `feat(scope): ...`, `fix(scope): ...`, `docs: ...`, or `chore: ...`.
6. Keep the subject imperative and under 72 characters.
7. Write a body with:
   - Summary: what changed
   - Rationale: why it changed
   - Validation: exact commands run, or `not run (reason)`
8. Wrap body lines at 72 characters.
9. Create the commit with `git commit -F <file>`.

## Template

```text
<type>(<scope>): <short summary>

Summary:
- <what changed>

Rationale:
- <why>

Validation:
- <command>
```
