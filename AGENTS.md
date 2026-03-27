# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust workspace implementing the Mister Smith orchestration operating system.
It contains 20 crates across 10 implemented phases, with the operating-system substrate now
validated through Phase 10 plus the March 16 runtime and session recovery slices.

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
Use `README.md`, `ROADMAP.md`, and `CLAUDE.md` as supporting orientation entry points.
Treat `WORKFLOW.md` and `docs/linear/LINEAR.md` as the live control-plane contract.
Treat `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` as the current runtime-proof
direction when the task is about proving real end-to-end execution rather than adding another
implementation phase.
Treat `docs/current-state.md` as the current forward-direction router. Use
`docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md` and
`docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` as the latest landed frontier
closure notes. No newer post-packet-020 bounded phase is frozen yet.

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
- Pull current state with `get_control_plane_snapshot` or `get_issue_execution_snapshot` before
  mutating Linear, queue, or review state.
- Use Smith workflow-family tools before raw Linear or ad hoc repo glue:
  - `save_linear_issue`, `save_issue_workpad`
  - `materialize_backlog_slices`, `plan_queue_stage`, `apply_queue_stage`
  - `resolve_issue_lifecycle`
  - `prepare_ralph_packet`, `record_ralph_outcome`
  - `prepare_speckit_context`, `translate_speckit_tasks`
- When a task explicitly calls for Ralph, use `./scripts/ralph` instead of bare `ralph`; rerun
  `./scripts/ralph prompt --packet <packet.json>` before each `./scripts/ralph run`.
- Treat `docs/current-state.md` as the current repo-wide router,
  `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md` as the most recent landed
  frontier closure note, `docs/plans/2026-03-16-smith-first-development-system.md` as historical
  control-plane background, and
  `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the current implemented
  workflow-family surface.
- For repo development workflow only, keep Linear as the durable source of truth, Symphony as the
  watched-queue executor, Ralph as the loop runner, and SpecKit as the upstream spec/task-pack
  scaffold.

## Subagent Orchestration

This repo ships a project-scoped Codex agent roster under `.codex/agents/`.

- Use explicit subagent delegation aggressively for bounded, parallel work. The repo default is
  `24` threads and depth `4`; use the `smith-burst` profile when the task is naturally wide or
  nested enough to justify `32 / 6`.
- Use `smith_repo_grounder` plus `smith_control_plane_auditor` for kickoff and recovery. Add
  `smith_docs_researcher` when external docs or tool behavior matter.
- Use `smith_frontier_guard` plus `smith_slice_planner` for backlog legitimacy, bounded slicing,
  and queue-readiness analysis.
- Use one `smith_crate_worker` per disjoint write scope, pair it with `smith_validator`, and run
  `smith_reviewer` before parent-controlled finalization.
- Use `smith_ralph_packet_builder` for Ralph-assisted flows and `smith_speckit_router` plus
  `smith_slice_planner` for SpecKit entry and task-pack translation.
- Use `spawn_agents_on_csv` for repeated audits or review sweeps across many similar files, issues,
  or services.
- Keep durable control-plane mutations in the parent thread:
  `save_linear_issue`, `save_issue_workpad`, `apply_queue_stage`, PR merge/push/land, and final
  issue state transitions stay parent-owned unless a one-off exception is explicitly delegated.

## Build, Test, and Development Commands

Run from repository root unless noted.

```bash
cargo build --workspace                    # Build all crates
cargo test --workspace                     # Run all tests (1115+)
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
cargo test -p <crate-name>                 # Test a single crate
./scripts/ralph --version                  # Check the repo-managed Ralph wrapper
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

## Security & Configuration Tips

- Never commit secrets; use environment variables or GitHub Actions secrets for credentials
- Provider API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
- OAuth credentials: Claude subscription uses Keychain/file-based credential sources
