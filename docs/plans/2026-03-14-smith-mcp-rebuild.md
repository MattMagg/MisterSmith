# Smith MCP Rebuild

## Objective

Rebuild the missing `smith` MCP as a local Codex-launched stdio server that restores the repo's
control-plane contract using only repo-visible consumers and deterministic local/external sources
of truth.

## Scope

- Add a real `smith-mcp` executable inside `crates/mister-smith-mcp`
- Implement the compatibility tool surface expected by repo-local skills and bootstrap flow
- Return structured JSON responses with explicit degraded/blocker states
- Integrate the rebuilt server with bootstrap/docs/tests so the repo can treat it as canonical

## Assumptions

- The original MCP source is unavailable
- The repo-local skills, `scripts/bootstrap_control_plane.py`, `WORKFLOW.md`, and
  `docs/linear/LINEAR.md` define the replacement contract
- Stdio is the only required runtime form for this first pass
- Mutation-capable tools must default to dry-run and require explicit `apply: true`

## Constraints

- Keep the rebuilt server deterministic and repo-grounded
- Avoid an LLM dependency for control-plane reasoning
- Reuse the existing `mister-smith-mcp` crate instead of introducing a second MCP runtime
- Do not regress the existing MCP client/session/bridge APIs already used elsewhere in the workspace

## Non-goals

- Full generic MCP platform parity beyond the repo compatibility contract
- HTTP server exposure in this pass
- Rebuilding the lost original internals exactly

## Milestones

### M1: Runtime and bootstrap tools

- Add stdio `smith-mcp` binary
- Finish rmcp server binding in `mister-smith-mcp`
- Implement `audit_workflow_readiness`, `get_server_runtime_info`, `reload_server`,
  `route_workflow_request`, and `get_control_plane_snapshot`
- Validation: local server lists tools and serves these calls end-to-end

### M2: Symphony and workspace tools

- Implement `get_symphony_checkout_snapshot`, `plan_workspace_adjustments`, `refresh_symphony`,
  and `sync_symphony_main`
- Validation: structured degraded response when Symphony prerequisites are missing; full snapshot
  when present

### M3: Linear and queue orchestration tools

- Implement `sync_linear_with_runtime`, `get_issue_execution_snapshot`,
  `review_merge_dispatch_cycle`, `plan_phase_execution`, and `apply_phase_execution_plan`
- Validation: read-only calls against live Linear when auth exists; dry-run mutation previews

### M4: Legitimacy and repo integration

- Implement `evaluate_issue_legitimacy` and `classify_follow_up_work`
- Reconcile bootstrap/tests/docs with the rebuilt server
- Validation: repo-local skills can make their first required `smith` tool call cleanly

## Validation

- Unit tests for handler behavior and routing
- MCP integration tests over a local rmcp server transport
- Bootstrap script tests
- Smoke test for core stdio launch path

## Repo Evidence Sources

- `WORKFLOW.md`: authoritative Symphony runtime contract, watched `project_slug`, state machine,
  queue scope, and unattended execution posture
- `docs/linear/LINEAR.md`: authoritative Linear taxonomy, admission rules, queue-vs-backlog
  semantics, status transitions, and GitHub/Symphony integration expectations
- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`: authoritative frontier mandate
  for legitimacy and drift decisions
- `specs/012-phase10-frontier-autonomy/spec.md` and `plan.md`: phase-level source for operator
  autonomy, bounded delegation, and why those surfaces belong in the control plane
- Repo-local Smith skill pack under `.codex/skills/`: tool-by-tool operator workflow expectations
- `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`: canonical workflow-first design
  for chaining the Smith MCP across Mister Smith, Linear, Symphony, GitHub, and long-running
  autonomous review/merge loops

## Fidelity Gaps To Correct

- `route_workflow_request` is currently keyword-routed; it should be refined against the
  repo-documented Symphony and Linear state machine instead of only string matching.
- `sync_linear_with_runtime` and `review_merge_dispatch_cycle` only partially encode the queue
  contract from `docs/linear/LINEAR.md`; they should reason explicitly about watched queue,
  validated backlog, docs hub, and Human Review/Merging/Rework posture.
- `plan_phase_execution` is currently Phase-10-biased and title-heuristic-heavy; it should derive
  more of its logic from spec/task artifacts plus documented backlog staging rules.
- `evaluate_issue_legitimacy` and `classify_follow_up_work` currently use lightweight keyword
  heuristics; they should be tightened to the frontier mandate decision rule from the March 9
  design note.
- Symphony path and workspace assumptions should be reduced to repo-authoritative config and
  explicit environment input where the repo does not actually define a single canonical checkout
  path.

## Stop Conditions

- Codex can launch `smith` as an MCP stdio server
- `audit_workflow_readiness` and `get_server_runtime_info` succeed locally
- Repo-local skills that currently require `smith` can complete their first tool hop

## Status

- 2026-03-14: Implemented the stdio `smith-mcp` binary, the 16-tool compatibility surface, repo bootstrap integration, and Codex config registration.
- 2026-03-14: Live Codex restart verified that `get_server_runtime_info` works against the rebuilt `smith` server.
- 2026-03-14: Found one post-restart regression in `audit_workflow_readiness`: the live server
  reports a false `smith_mcp_config` blocker even though `/Users/macmain/.codex/config.toml`
  contains `[mcp_servers.smith]` and the server is loaded.
- Current milestone: harden Codex config inspection inside the MCP and revalidate the readiness path so the remaining blockers are only real environment gaps.
