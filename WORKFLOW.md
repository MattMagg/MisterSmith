---
tracker:
  kind: linear
  api_key: $LINEAR_API_KEY
  project_slug: "phase-91-security-hardening-e439446ddfb9"
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
  root: ~/code/mister-smith-symphony-workspaces
hooks:
  after_create: |
    export SOURCE_REPO_URL="${SOURCE_REPO_URL:-https://github.com/MattMagg/Mister-Smith.git}"
    git clone --depth 1 "$SOURCE_REPO_URL" .
    if command -v cargo >/dev/null 2>&1; then
      cargo fetch
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
- This repo is a Rust workspace. Prefer `cargo build --workspace` for cross-crate compile validation.
- During development, run `cargo test -p <crate>` for affected crates
  instead of `cargo test --workspace` unless the change touches
  `mister-smith-core`, shared cross-crate contracts, CI/workflows, or the
  issue explicitly requires full-workspace proof.
- Keep `spec/` and `specs/` distinct. `spec/` is canonical architecture; `specs/` contains per-phase implementation artifacts.
- Avoid editing `archive/` unless the issue explicitly requires it.
- This repository forbids git worktrees. For this unattended Symphony session
  only, you are explicitly authorized to create, switch, and push
  issue-specific branches inside this isolated workspace when the Linear/PR
  workflow requires it.

## Symphony prerequisites

- A Linear connection is available either through Symphony's tracker integration or the `linear_graphql` tool.
- The Linear team workflow must define the non-standard states `Rework`, `Human Review`, and `Merging`.
- GitHub CLI auth may be required for PR workflows.
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
- Spend extra effort on investigation and verification design before implementation.
- Reproduce the current issue signal before changing code when the task is a bug or regression.
- Keep ticket metadata current.
- Treat any `Validation`, `Test Plan`, or `Testing` section in the issue as mandatory acceptance input.
- When you discover meaningful out-of-scope work, create a follow-up issue instead of silently expanding scope.
- This workflow is scoped to the single `tracker.project_slug` in this file; issues outside that watched project will not dispatch.
- An empty `Todo` queue means there is no runnable issue in the watched project right now; it does not mean the `Todo` state is missing.
- Do not move blocked or future work into the watched project just to keep Symphony busy.

## Status map

- `Backlog` -> out of scope for this workflow. Do not modify.
- `Todo` -> immediately move to `In Progress`, then start execution.
- `In Progress` -> active implementation.
- `Human Review` -> wait for reviewer input and poll for updates.
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
   - sync with latest `origin/main` using the `pull` skill if needed
   - capture a concrete reproduction signal for bugs and regressions

## Step 2: Execute

1. Implement against the workpad checklist and keep it current after every meaningful milestone.
2. Run validation that matches the scope.
   - For Rust code changes, use `cargo build --workspace` plus affected-crate tests.
   - Escalate to broader validation when shared contracts or CI-critical surfaces move.
   - For docs or workflow-only changes, run the narrowest proof that directly validates the edit.
3. Before every push, rerun the validation required for the current scope.
4. Attach the PR URL to the issue once a PR exists.
5. Update the workpad with final checklist state, validation evidence, and any remaining confusion points.
6. Before moving to `Human Review`:
   - resolve all actionable PR comments or push back explicitly
   - confirm checks are green after the latest push
   - confirm every required validation item is complete

## Step 3: Human review and merge handling

1. In `Human Review`, do not code. Poll for updates.
2. If review feedback requires changes, move the issue to `Rework`.
3. If approved, the issue moves to `Merging`.
4. In `Merging`, follow the `land` skill until the PR is merged.
5. After merge is complete, move the issue to `Done`.

## Step 4: Rework handling

1. Treat `Rework` as a fresh attempt, not a tiny patch pass.
2. Re-read the issue and review comments and explicitly identify what will change this attempt.
3. If the existing branch or PR is no longer the right vehicle, close it and create a fresh branch from `origin/main`.
4. Create or refresh the `## Codex Workpad` comment and restart the normal execution flow.
