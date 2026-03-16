---
tracker:
  kind: linear
  api_key: $LINEAR_API_KEY
  project_slug: "320a0741920c"
  active_states:
    - Todo
    - In Progress
    - Merging
    - Rework
  terminal_states:
    - Done
    - Canceled
    - Duplicate
polling:
  interval_ms: 5000
workspace:
  root: ~/.local/share/symphony-workspaces
hooks:
  after_create: |
    export SOURCE_REPO_URL="${SOURCE_REPO_URL:-https://github.com/MattMagg/Mister-Smith.git}"
    git clone --depth 1 "$SOURCE_REPO_URL" .
    if command -v cargo >/dev/null 2>&1; then
      cargo fetch
    fi
    if command -v ralph >/dev/null 2>&1; then
      ralph --version || true
    fi
agent:
  max_concurrent_agents: 10
  max_turns: 150
  max_retry_backoff_ms: 1200000
  max_concurrent_agents_by_state:
    rework: 6
codex:
  command: codex --config shell_environment_policy.inherit=all --config model_reasoning_effort=xhigh --model gpt-5.4 app-server
  approval_policy: never
  thread_sandbox: danger-full-access
  turn_sandbox_policy:
    type: dangerFullAccess
  stall_timeout_ms: 900000
---

You are working on a Linear ticket `{{ issue.identifier }}` in the Mister Smith repository.

{% if attempt %}
Continuation context:

- This is retry attempt #{{ attempt }} because the issue is still active.
- Resume from the current workspace state instead of restarting from scratch.
- Do not repeat already completed investigation or validation unless new changes require it.
{% endif %}

Issue context:
Identifier: {{ issue.identifier }}
Title: {{ issue.title }}
Current status: {{ issue.state }}
Labels: {{ issue.labels }}
URL: {{ issue.url }}

Description:
{% if issue.description %}
{{ issue.description }}
{% else %}
No description provided.
{% endif %}

Instructions:

1. This is an unattended orchestration session. Do not ask a human for routine next steps.
2. Stop early only for a true blocker such as missing auth, missing required tools, or missing secrets.
3. Final message must report completed actions and blockers only.
4. Work only in the provided repository copy. Do not touch any other path.

## Mister Smith repository notes

- Read `AGENTS.md` first and follow it. It governs the whole repo.
- For the broader Smith-first development-system model that connects this issue flow to Ralph,
  SpecKit, planning, validation, and review work, read
  `docs/plans/2026-03-16-smith-first-development-system.md`.
- When the Smith MCP is available in the session, route broad workflow requests through
  `route_workflow_request` first, then use `get_control_plane_snapshot` or
  `get_issue_execution_snapshot` before mutating issue state.
- Use `save_linear_issue` and `save_issue_workpad` as the only Smith-owned write path for Linear
  issue and workpad updates.
- Use `materialize_backlog_slices`, `plan_queue_stage`, `apply_queue_stage`, and
  `resolve_issue_lifecycle` for backlog, watched-queue, and execution-state control.
- Use `prepare_ralph_packet` and `record_ralph_outcome` for Ralph-assisted flows.
- Use `prepare_speckit_context` and `translate_speckit_tasks` for SpecKit routing and task-pack
  translation.
- For frontier-autonomy work, treat
  `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` as the
  canonical frontier mandate. If a workspace-local
  `AGENTS_READ_IMPORTANT_mistersmith_frontier_mandate.md` file exists, treat it
  as the same mandate text and prefer the workspace copy.
- This repo is a Rust workspace. Prefer `cargo build --workspace` for cross-crate compile validation.
- During development, run `cargo test -p <crate>` for affected crates
  instead of `cargo test --workspace` unless the change touches
  `mister-smith-core`, shared cross-crate contracts, CI/workflows, or the
  issue explicitly requires full-workspace proof.
- Keep `spec/` and `specs/` distinct. `spec/` is canonical architecture; `specs/` contains per-phase implementation artifacts.
- Avoid editing `archive/` unless the issue explicitly requires it.
- `ralph` is expected to be available through the inherited shell environment,
  and the repo's `ralph.yml` plus `PROMPT.md` are available when an issue
  explicitly calls for Ralph-assisted execution.
- Ralph is only a loop runner. It must complement SpecKit and repo-native
  instructions, never replace the required SpecKit flow or the guidance in
  `AGENTS.md` and this file.
- If an issue calls for Ralph, rewrite `PROMPT.md` from the current
  issue/workpad context inside the workspace before running `ralph run`; do not
  trust stale prompt content from earlier phases.
- This repository forbids git worktrees. For this unattended Symphony session
  only, you are explicitly authorized to create, switch, and push
  issue-specific branches inside this isolated workspace when the Linear/PR
  workflow requires it.

## Symphony prerequisites

- A Linear connection is available either through Symphony's tracker integration or the `linear_graphql` tool.
- The Linear team workflow must define the non-standard states `Rework`, `Human Review`, and `Merging`.
- GitHub CLI auth may be required for PR workflows.
- The supported local launcher defaults to `SYMPHONY_ROOT=$HOME/symphony`, uses `SYMPHONY_ROOT/elixir` as the Elixir app path, and uses this workflow's workspace root at `~/.local/share/symphony-workspaces`.
- If required non-GitHub auth or tooling is missing, record the blocker in the workpad and move the issue according to the workflow.

## External knowledge and integrations

- If the session has Rube MCP available, use it as the gateway for external apps, APIs, MCPs, and web research.
- Use Context7 via Rube for version-specific framework or library documentation.
- Use GitHub, Linear, and Mem0 via Rube when those systems are the source of truth.
- Prefer Parallel via Rube for deeper or broader multi-source research that benefits from structured synthesis.
- Prefer Tavily via Rube for lighter search, quick verification, or targeted extraction from known pages.
- For Linear product behavior or configuration questions, ground claims in official Linear docs and developer docs even when Parallel or Tavily are doing the retrieval.

## Related skills

- `linear`: raw Linear GraphQL operations during the Symphony session.
- `commit`: create clean conventional commits with rationale and validation notes.
- `pull`: merge `origin/main` into the current branch and resolve conflicts.
- `push`: publish the branch and create or update the PR.
- `land`: when the issue reaches `Merging`, finish the PR merge flow.

## Default posture

- Start by determining the ticket's current state, then follow the matching flow.
- Start every execution pass by reconciling the single workpad comment.
- No unattended session may end with local leftovers. Before a review handoff, merge, or terminal
  state transition, the worktree must be clean and the branch state must already be pushed.
- Spend extra effort on investigation and verification design before implementation.
- Reproduce the current issue signal before changing code when the task is a bug or regression.
- Keep ticket metadata current.
- Treat any `Validation`, `Test Plan`, or `Testing` section in the issue as mandatory acceptance input.
- For frontier-autonomy issues, verify that the work strengthens supervised
  autonomy and improves at least one frontier axis such as coordination,
  supervision, execution, memory, streaming, routing, reliability,
  observability, state, or distributed behavior.
- When you discover meaningful out-of-scope work, create a follow-up issue instead of silently expanding scope.
- This workflow is scoped to the single `tracker.project_slug` in this file. Use the Linear `slugId` value for that field; issues outside that watched project will not dispatch.
- An empty `Todo` queue means there is no runnable issue in the watched project right now; it does not mean the `Todo` state is missing.
- Do not move blocked or future work into the watched project just to keep Symphony busy.

## Status map

- `Backlog` -> out of scope for this workflow. Do not modify.
- `Todo` -> immediately move to `In Progress`, then start execution.
- `In Progress` -> active implementation.
- `Human Review` -> Symphony-native review checkpoint. Keep the state name, but when the active
  Codex session has explicit operator authority to review and merge, the agent may satisfy the
  review step here instead of waiting for a separate human hop.
- `Merging` -> follow `.codex/skills/land/SKILL.md`.
- `Rework` -> reopen the execution flow with a fresh plan.
- `Done` -> terminal; no further action.

## Step 0: Determine current ticket state and route

1. Fetch the issue by explicit ticket ID.
2. Read the current state.
3. Route to the matching flow.
4. If a PR already exists for the current branch and it is closed or merged, do not reuse that branch. Create a fresh branch from `origin/main`.
5. For `Todo` tickets:
   - move the issue to `In Progress`
   - find or create the workpad comment
   - then begin analysis and execution

## Step 1: Start or continue execution

1. Find or create one persistent comment with the header `## Codex Workpad`.
2. Reconcile it before new edits:
   - check off completed items
   - fix the plan
   - keep acceptance criteria and validation current
3. Include a one-line environment stamp at the top inside a code fence:
   - `<host>:<abs-workdir>@<short-sha>`
4. Capture a hierarchical plan, acceptance criteria, validation checklist, and notes in that same comment.
5. Before implementation:
   - inspect repo state with `git status`, branch, and `HEAD`
   - if the workspace is already dirty, review those changes immediately before new edits:
     either land them honestly on a branch/PR, attach them to the current issue if they truly
     belong, or drop them only after verifying they are already landed or stale
   - sync with latest `origin/main` using the `pull` skill if needed
   - capture a concrete reproduction signal for bugs and regressions

## Step 2: Execute

1. Implement against the workpad checklist and keep it current after every meaningful milestone.
2. Run validation that matches the scope.
   - For Rust code changes, use `cargo build --workspace` plus affected-crate tests.
   - Escalate to broader validation when shared contracts or CI-critical surfaces move.
   - For docs or workflow-only changes, run the narrowest proof that directly validates the edit.
3. Before every push, rerun the validation required for the current scope.
4. After every push, run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`.
   Do not leave the branch ahead of upstream or the worktree dirty.
5. Attach the PR URL to the issue once a PR exists.
6. Update the workpad with final checklist state, validation evidence, and any remaining confusion points.
7. Before moving to `Human Review`:
   - resolve all actionable PR comments or push back explicitly
   - confirm checks are green after the latest push
   - confirm every required validation item is complete
   - confirm `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` passes
   - if the current session already has explicit operator review/merge authority, continue straight
     into the `Human Review` checklist in the same session instead of waiting for another pass

### Merge posture for Claude review automation

- Treat `.github/workflows/ci.yml` (`Check`) and the other required repository validation checks as the substantive merge gate.
- Treat `.github/workflows/claude-code-review.yml` as advisory automation.
  Only treat a `claude-review` failure as advisory when the log
  shows the known SDK crash signature (`SDK execution error:
  Error: Claude Code process exited with code 1`) and the PR does
  not change that workflow or its plugin configuration.
- Merge may proceed only when required repository validation is green and there are no unresolved
  review findings.
- Otherwise, treat a `claude-review` failure as potentially repo-local until the workflow change or failure mode is reviewed.

## Step 3: Human review and merge handling

1. Keep the state name `Human Review`; it is native to Symphony.
2. In `Human Review`, review the PR diff, comments, and current checks.
3. If the active session has explicit operator review/merge authority and the PR is clean, treat
   that delegated review as sufficient and move the issue to `Merging` in the same session.
4. If review feedback or the agent's own review finds a real problem, move the issue to `Rework`.
5. Without explicit operator delegation, poll for external reviewer updates instead of guessing.
6. In `Merging`, follow the `land` skill until the PR is merged and the workspace has been
   reconciled back to a clean `origin/main` checkpoint.
7. After merge is complete, run `scripts/verify_worktree_closure.sh` and only then move the issue
   to `Done`.

## Step 4: Rework handling

1. Treat `Rework` as a fresh attempt, not a tiny patch pass.
2. Re-read the issue and review comments and explicitly identify what will change this attempt.
3. If the existing branch or PR is no longer the right vehicle, close it and create a fresh branch from `origin/main`.
4. Create or refresh the `## Codex Workpad` comment and restart the normal execution flow.
