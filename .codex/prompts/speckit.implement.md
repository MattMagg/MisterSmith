---
description: Execute the active Mister Smith packet by processing the tasks defined in tasks.md and closing cleanly.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Pre-Execution Checks

**Check for extension hooks (before implementation)**:

- Check if `.specify/extensions.yml` exists in the project root.
- If it exists, read it and look for entries under the `hooks.before_implement` key
- If the YAML cannot be parsed or is invalid, skip hook checking silently and continue normally
- Filter out hooks where `enabled` is explicitly `false`. Treat hooks without an `enabled` field as enabled by default.
- For each remaining hook, do **not** attempt to interpret or evaluate hook `condition` expressions:
  - If the hook has no `condition` field, or it is null/empty, treat the hook as executable
  - If the hook defines a non-empty `condition`, skip the hook and leave condition evaluation to the HookExecutor implementation
- For each executable hook, output the following based on its `optional` flag:
  - **Optional hook** (`optional: true`):
    ```text
    ## Extension Hooks

    **Optional Pre-Hook**: {extension}
    Command: `/{command}`
    Description: {description}

    Prompt: {prompt}
    To execute: `/{command}`
    ```
  - **Mandatory hook** (`optional: false`):
    ```text
    ## Extension Hooks

    **Automatic Pre-Hook**: {extension}
    Executing: `/{command}`
    EXECUTE_COMMAND: {command}

    Wait for the result of the hook command before proceeding to the Outline.
    ```
- If no hooks are registered or `.specify/extensions.yml` does not exist, skip silently

## Outline

1. Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` from
   repo root and parse `FEATURE_DIR` and `AVAILABLE_DOCS`. All paths must be absolute.

2. Read the implementation authority in this order:
   - `AGENTS.md`
   - `docs/current-state.md`
   - `docs/ms_recent_context.md`
   - the active packet files under `FEATURE_DIR`
   - `.specify/memory/constitution.md`

3. For Mister Smith packet work, perform the Smith-first preflight before task execution:
   - route the request with `route_workflow_request`
   - pull current repo or issue state with `get_control_plane_snapshot` or
     `get_issue_execution_snapshot`
   - reconcile the single `## Codex Workpad`
   - if lifecycle or queue posture matters, use `resolve_issue_lifecycle`, `plan_queue_stage`,
     `apply_queue_stage`, or the other Smith workflow-family tools before edits
   - only proceed once the active slice is confirmed honest and runnable

4. Inspect local repo state before edits:
   - `git status --short --branch`
   - `git rev-parse --short HEAD`
   - If the repo is already dirty, review that state before continuing
   - Do not create or rely on git worktrees; this repo forbids them

5. Check checklist status if `FEATURE_DIR/checklists/` exists:
   - Summarize total/completed/incomplete items
   - Treat incomplete checklists as risk input, not silent noise
   - If the user has not explicitly authorized proceeding past incomplete critical checklists,
     stop and surface the risk

6. Execute the task pack:
   - Follow the blocking freeze before any `[P]` lane begins
   - Respect task ordering and single-owner choke points
   - Mark completed tasks as `[x]` in `tasks.md`
   - Keep deterministic checks and live-proof steps explicitly separate
   - Update packet docs or proof notes when the task pack requires them

7. Validation and closure:
   - Run the narrowest meaningful validation for the changed behavior
   - Escalate to broader validation only when shared contracts or CI-critical surfaces moved
   - Run `git diff --check`
   - Before declaring completion, run
     `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
   - Do not stop with task-owned dirty repo state

8. Report:
   - completed tasks
   - validation run
   - proof boundaries
   - remaining blockers or deferred work

9. **Check for extension hooks**: After completion validation, check if
   `.specify/extensions.yml` exists in the project root.
   - If it exists, read it and look for entries under the `hooks.after_implement` key
   - If the YAML cannot be parsed or is invalid, skip hook checking silently and continue normally
   - Filter out hooks where `enabled` is explicitly `false`. Treat hooks without an `enabled`
     field as enabled by default.
   - For each remaining hook, do **not** attempt to interpret or evaluate hook `condition`
     expressions:
     - If the hook has no `condition` field, or it is null/empty, treat the hook as executable
     - If the hook defines a non-empty `condition`, skip the hook and leave condition evaluation to
       the HookExecutor implementation
   - For each executable hook, output the following based on its `optional` flag:
     - **Optional hook** (`optional: true`):
       ```text
       ## Extension Hooks

       **Optional Hook**: {extension}
       Command: `/{command}`
       Description: {description}

       Prompt: {prompt}
       To execute: `/{command}`
       ```
     - **Mandatory hook** (`optional: false`):
       ```text
       ## Extension Hooks

       **Automatic Hook**: {extension}
       Executing: `/{command}`
       EXECUTE_COMMAND: {command}
       ```
   - If no hooks are registered or `.specify/extensions.yml` does not exist, skip silently

## Key Rules

- Work from the packet and current repo truth, not from stale assumptions
- For Mister Smith packet work, the execution path is hybrid: Smith-first control-plane preflight,
  then this `speckit.implement` task-pack execution
- Keep diffs bounded to the packet scope
- No fake completion: validation and closure are part of implementation
- No task-owned leftovers
