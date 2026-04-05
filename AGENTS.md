# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust workspace implementing the Mister Smith orchestration operating system.
It contains 20 crates across 10 implemented phases, with the operating-system substrate now
validated through Phase 10 plus the landed frontier packets through `024`.

- `crates/`: Rust workspace — 18 library crates + 1 binary + 1 integration test crate
- `spec/`: Canonical architecture specifications (the system contract)
- `specs/`: SpecKit-generated per-phase implementation artifacts (build instructions)
- `plans/`: Implementation plans and batch trackers
- `docs/`: Research output, code reviews, session analysis
- `archive/`: Historical validation/research artifacts; avoid editing unless explicitly needed
- `deploy/`: Deployment artifacts — Dockerfile, K8s manifests, Grafana dashboards, Prometheus alerts
- `nats.rs/`: Vendored upstream Rust NATS workspace used as API reference
- `scripts/`: Utility scripts for control-plane bootstrap, validation, and local runtime support

Use `docs/current-state.md` as the repo-wide current-state overview and document router.
Use `docs/direction.md` when you need the merged strategic direction and next-build priority.
Use `README.md`, `ROADMAP.md`, and `CLAUDE.md` as supporting orientation entry points.
Treat `AGENTS.md`, `docs/current-state.md`,
`docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md`, and the active direct-execution
sections of `docs/linear/LINEAR.md` as the current Smith control-plane contract. Treat
`WORKFLOW.md` as legacy Symphony background unless a task explicitly targets that historical
automation layer.
Treat `docs/current-state.md`, `scripts/live_runtime_proof_smoke.py`, and
`docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` as the current bounded
runtime-proof baseline. Use `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` as
historical broader live-run context.
Treat `docs/current-state.md` as the current forward-direction router. Use
`specs/023-runtime-truth-and-run-trace/` and
`specs/024-agent-boundary-security-hardening/` as the latest landed packet authorities,
`specs/025-step-level-intelligence-v2/` as the latest landed step-policy packet authority,
`specs/026-first-real-coordinator-subagent-runtime/` as the next implementation-ready packet,
`specs/022-durable-workflow-core/` for packet-022 ownership, and
`docs/plans/2026-03-27-runtime-planning-simplification.md` plus
`docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` for the current live-proof and
repair-provenance context.

## Product Boundary

Treat the Mister Smith OS and the repo-development workflow as different layers.

- Mister Smith OS: the Rust workspace, runtime process, task/session/autonomy operator surfaces,
  NATS and persistence integration, the shipped `mister-smith-mcp` crate, security model, deploy
  assets, and the architecture captured in `spec/` and implemented in `crates/`.
- Repository development workflow around Mister Smith: Linear, Symphony, Ralph, SpecKit, repo
  workpads, PR flow, and watched-queue orchestration.
- Smith MCP is a repo-owned crate and control-plane surface in this workspace. Do not collapse it
  into the same bucket as external workflow services.
- Linear and Symphony are not part of the Mister Smith operating system. They are external
  development tools and workflow machinery used to plan, stage, execute, review, and land changes
  to Mister Smith.
- Do not describe the product as if Linear state, Symphony queue execution, or other repo workflow
  control-plane helpers are runtime subsystems of the shipped OS.
- When a task is about the product, prioritize repo/runtime truth first. When a task is about the
  development workflow, use the control-plane docs and tools deliberately without collapsing that
  workflow into the OS architecture.

## Smith-First Workflow Posture

For Mister Smith development work, treat Smith MCP as the default control-plane entrypoint. This
section is about how the repository is developed, not about what the Mister Smith OS contains at
runtime.

- Start broad workflow requests with `route_workflow_request`.
- Use the router's `response_only`, `plan_only`, and `tracked_execution` classes to keep analysis,
  planning, and execution distinct without bouncing out for extra operator review.
- Pull current state with `get_control_plane_snapshot` or `get_issue_execution_snapshot` before
  mutating Linear or review state.
- Use Smith workflow-family tools before raw Linear or ad hoc repo glue:
  - `save_linear_issue`, `save_issue_workpad`
  - `prepare_direct_execution`, `materialize_backlog_slices`
  - `resolve_issue_lifecycle`, `review_merge_status`
  - `prepare_ralph_packet`, `record_ralph_outcome`
  - `prepare_speckit_context`, `translate_speckit_tasks`
- For frozen packet implementation work, use a hybrid flow:
  - Smith-first routing and state reconciliation decide the direct execution plan and what
    control-plane state must be refreshed first.
  - After that preflight, explicitly execute the repo-local `speckit.implement` surface through
    `.codex/commands/implement.md` and `.codex/prompts/speckit.implement.md` before code changes.
  - Do not jump from packet discovery straight into ad hoc implementation against `specs/` without
    running the repo-local implement flow.
- When a task explicitly calls for Ralph, use `./scripts/ralph` instead of bare `ralph`; rerun
  `./scripts/ralph prompt --packet <packet.json>` before each `./scripts/ralph run`.
- Treat `docs/current-state.md` as the current repo-wide router,
  `specs/023-runtime-truth-and-run-trace/` and
  `specs/024-agent-boundary-security-hardening/` as the latest landed packet authorities,
  `specs/025-step-level-intelligence-v2/` as the latest landed step-policy packet authority,
  `specs/026-first-real-coordinator-subagent-runtime/` as the next implementation-ready packet,
  `docs/plans/2026-03-16-smith-first-development-system.md` as historical control-plane background,
  and `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md` as the current direct
  execution control-plane note.
- For repo development workflow only, keep Linear as the durable source of truth, Smith MCP as the
  direct Codex control plane, Ralph as the loop runner, and SpecKit as the upstream spec/task-pack
  scaffold.
- Keep risky or merge-sensitive checks internal to the Smith workflow by using the tool-reported
  gate state and review status instead of adding extra human interruption just to decide whether the
  next autonomous step is allowed.

## Subagent Orchestration

This repo ships a project-scoped Codex agent roster under `.codex/agents/`.

- Use explicit subagent delegation aggressively for bounded, parallel work. The repo default is
  `24` threads and depth `4`; use the `smith-burst` profile when the task is naturally wide or
  nested enough to justify `32 / 6`.
- Use `smith_repo_grounder` plus `smith_control_plane_auditor` for kickoff and recovery. Add
  `smith_docs_researcher` when external docs or tool behavior matter.
- Use `smith_frontier_guard` plus `smith_slice_planner` for backlog legitimacy, bounded slicing,
  and direct execution readiness analysis.
- Use one `smith_crate_worker` per disjoint write scope, pair it with `smith_validator`, and run
  `smith_reviewer` before parent-controlled finalization.
- Use `smith_ralph_packet_builder` for Ralph-assisted flows and `smith_speckit_router` plus
  `smith_slice_planner` for SpecKit entry and task-pack translation.
- Use `spawn_agents_on_csv` for repeated audits or review sweeps across many similar files, issues,
  or services.
- Keep durable control-plane mutations in the parent thread:
  `save_linear_issue`, `save_issue_workpad`, PR merge/push/land, and final issue state transitions
  stay parent-owned unless a one-off exception is explicitly delegated.

## Build, Test, and Development Commands

Run from repository root unless noted.

```bash
cargo build --workspace                    # Build all crates
cargo test --workspace                     # Run all tests (1115+)
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
cargo test -p <crate-name>                 # Test a single crate
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
                                          # Validate the repo-owned smoke harness tests
python3 -m py_compile scripts/live_runtime_proof_smoke.py
                                          # Fast syntax check for smoke-harness edits
./scripts/ralph --version                  # Check the repo-managed Ralph wrapper
./scripts/run-symphony.sh                  # Supported local Symphony launcher for this repo
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
                                           # Verify clean synced closure state
```

For markdown linting:

- `npx markdownlint-cli2 "spec/**/*.md" "*.md" --config .markdownlint.json`
- `git grep -nE "TODO|TBD|FIXME" spec/`: catch unfinished spec language before PR

## Coding Style & Naming Conventions

- **Rust**: Follow existing workspace conventions — `rustfmt` defaults, zero clippy warnings
- **Error pattern**: Domain errors defined in `mister-smith-core`, re-exported from domain crates (SecurityError, PersistenceError, LlmError)
- **Feature flags**: Used for optional integrations (`security`, `sqlx`, `llm`, provider features)
- **Markdown**: ATX headings, 2-space list indentation, 200-char max line length (see `.markdownlint.json`)
- **Spec docs**: lowercase kebab-case filenames (e.g., `spec/core-architecture/system-architecture.md`)
- Keep terminology consistent with existing domain docs; update cross-references when renaming files

## Testing Guidelines

- Run `cargo test -p <crate>` for the affected crate during development
- Full workspace tests only when touching `mister-smith-core` types or when explicitly asked
- `cargo build --workspace` is a fast (~8s) check for cross-crate compatibility
- When touching `scripts/live_runtime_proof_smoke.py` or its proof-contract behavior, run
  `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke` and
  `python3 -m py_compile scripts/live_runtime_proof_smoke.py`
- Use `python3 scripts/live_runtime_proof_smoke.py --profile budget_softcap_openai_mock` only for
  honest packet-019 bounded live-proof work when the documented Docker/auth prerequisites are
  already satisfied
- Env-gated integration tests: `#[ignore]` by default, require `DATABASE_URL` / `NATS_URL`

## Commit & Pull Request Guidelines

- Conventional commits with scope: `feat(llm):`, `fix(agents):`, `docs:`, `chore:`, `style:`
- Keep commits atomic and scoped to one concern
- Do not end a workflow or handoff with uncommitted or untracked repository changes; review
  leftovers immediately and either land them on a branch/PR or drop them only after verifying they
  are already landed or stale
- Opening a PR is not closure. For task-owned branch/worktree lanes, closure is complete only
  after the PR is merged, the task-owned branch/worktree is removed locally, and the primary
  `/Users/macmain/MisterSmith` checkout is back on a clean synced `main`
- Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` after push,
  before `Human Review`, and again after merge
- PRs should include: concise problem/solution summary, touched files, validation commands run
- PR references use `(#NNN)` suffix
- GitHub Actions are intentionally disabled in this repository; use local validation plus
  CodeRabbit and operator review as the review posture

## Security & Configuration Tips

- Never commit secrets; use environment variables or another external secret store for credentials
- Provider API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
- OAuth credentials: Claude subscription uses Keychain/file-based credential sources

## Active Technologies

- Rust 1.88.0 plus the existing TypeScript desktop surface in `apps/operator-console/`
  (029-session-first-user-shell)
- Existing retained-session seams in `mister-smith-app`, `mister-smith-http`, and
  `apps/operator-console/` are the planning focus for `029-session-first-user-shell`

## Recent Changes

- 029-session-first-user-shell: Added packet planning artifacts for the session-first shared
  CLI and GUI shell
