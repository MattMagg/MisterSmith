---
description: Generate an actionable, dependency-ordered Mister Smith tasks.md for the active packet.
handoffs:
  - label: Analyze For Consistency
    agent: speckit.analyze
    prompt: Run a project analysis for consistency
    send: true
  - label: Implement Project
    agent: speckit.implement
    prompt: Start the implementation in phases
    send: true
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Pre-Execution Checks

**Check for extension hooks (before tasks generation)**:

- Check if `.specify/extensions.yml` exists in the project root.
- If it exists, read it and look for entries under the `hooks.before_tasks` key
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

1. **Setup**: Run `.specify/scripts/bash/check-prerequisites.sh --json` from repo root and parse
   `FEATURE_DIR` and `AVAILABLE_DOCS`. All paths must be absolute. For single quotes in args like
   "I'm Groot", use escape syntax: e.g. `'I'\''m Groot'` (or double-quote if possible:
   `"I'm Groot"`).

2. **Load packet context**: Read from `FEATURE_DIR`:
   - **Required**: `plan.md`, `spec.md`
   - **Optional**: `data-model.md`, `contracts/`, `research.md`, `quickstart.md`, `analyze.md`
   - Also read one recent landed `specs/*/tasks.md` packet example from this repo so the output
     matches Mister Smith task-pack style.

3. **Generate the task pack**:
   - Start with status reconciliation and preserved baseline truth
   - Add one blocking freeze section if the packet needs a scope or design checkpoint
   - Generate one bounded section per user story or lane
   - Within each story or lane, include:
     - goal
     - independent test
     - targeted validation tasks
     - implementation tasks
     - checkpoint outcome
   - Add a final validation and evidence section
   - Add a parallel directive with allowed lanes and choke points

4. **Task generation rules**:
   - Use `.specify/templates/tasks-template.md` as the structure
   - Keep every task specific enough that an LLM can execute it without extra context
   - Use exact file paths in every implementation or documentation task
   - Use `[P]` only when the write set is disjoint and every blocking checkpoint in the current
     section is already complete
   - Keep deterministic checks and live-proof steps explicitly separate when both exist
   - Include explicit final closure work such as `git diff --check` and
     `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

5. **Report**: Output the path to `tasks.md` plus:
   - total task count
   - task count per user story or lane
   - blocking freeze tasks identified
   - parallel opportunities identified
   - final validation and evidence tasks identified
   - format validation confirming all tasks use checkbox, ID, labels, and file paths

6. **Check for extension hooks**: After `tasks.md` is generated, check if
   `.specify/extensions.yml` exists in the project root.
   - If it exists, read it and look for entries under the `hooks.after_tasks` key
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

## Task Generation Rules

**CRITICAL**: Mister Smith task packs are bounded packet execution guides, not generic sprint
backlogs.

- Start with a blocking freeze when the packet needs one
- Keep user stories or lanes independently understandable and testable
- Include final validation and evidence tasks
- Do not create fake parallelism; `[P]` requires a disjoint write set and completed blockers
- Do not widen scope just to fill the task list

### Checklist Format (REQUIRED)

Every task MUST strictly follow this format:

```text
- [ ] [TaskID] [P?] [Story?] Description with file path
```

**Format Components**:

1. **Checkbox**: ALWAYS start with `- [ ]`
2. **Task ID**: Sequential number (`T001`, `T002`, `T003`, ...)
3. **[P] marker**: Include ONLY when the task can run in parallel under the bounded packet rules
4. **[Story] label**: Use `[US1]`, `[US2]`, `[US3]`, etc. for story-bound work
5. **Description**: Clear action with exact file path

**Examples**:

- `- [ ] T001 [US1] Freeze scope in specs/020-example/spec.md`
- `- [ ] T004 [P] [US1] Add targeted runtime test in crates/mister-smith-app/tests/example.rs`
- `- [ ] T010 [US2] Capture durable proof note in docs/plans/2026-03-26-example.md`

### Section Guidance

- **Blocking freeze**: required when packet scope, contract choice, or shared invariants must be
  frozen first
- **Story sections**: tests -> implementation -> checkpoint
- **Final validation**: include targeted tests, broader build or lint only when justified, and
  closure checks
- **Parallel directive**: list allowed lanes and shared choke points explicitly

## Context

$ARGUMENTS
