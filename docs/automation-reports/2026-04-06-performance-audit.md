# 2026-04-06 Performance Audit

## Summary

No confirmed performance regression was grounded in this audit.

The strongest repo-native evidence points to one plausible core-runtime concern that still needs
measurement, not a proven slowdown: successful live runs on the default runtime path continue to
spend `15-27s` between `queued` and `completed`, while bootstrap itself stayed under `1s` in the
same artifact lanes. Current metrics, dashboards, and alerts do not expose enough phase-level
timing to attribute that wall-clock time cleanly.

Current operator-console build and test performance does not look regressed:

- `npm --prefix apps/operator-console run build`: `1.19s`
- `npm --prefix apps/operator-console test`: `1.83s`
- production bundle from the same build: `236.31 kB` JS / `71.27 kB` gzip

Because the runtime-latency concern is evidence-backed but not yet attributable, this audit opened
one bounded instrumentation follow-up instead of filing a regression bug:

- GitHub: `#329` <https://github.com/MattMagg/MisterSmith/issues/329>
- Linear: `MS-126` <https://linear.app/agentic-ops/issue/MS-126>

## Audit Window

- local date: `2026-04-06`
- last automation run: `2026-04-05T04:49:01.144Z`
- repo state inspected for evidence gathering: clean `codex/031-chat-first-cli-loop` checkout plus
  a clean detached `origin/main` worktree at `b7779a35fd14913db618b61222fc75a5228ba5bd`
- recent history and artifact focus:
  - recent hot-path commits from `2026-03-26` through `2026-04-06`
  - live runtime proof artifacts from `2026-03-26` and `2026-04-05`
  - current monitoring, dashboard, alert, and operator-console surfaces on `origin/main`

## Evidence Reviewed

- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
- `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md`
- `docs/current-state.md`
- `scripts/live_runtime_proof_smoke.py`
- `docs/plans/artifacts/live-runtime-proof-smoke/20260326T174047Z/`
- `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/`
- `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/repaired-baseline-ready/20260405T180349Z/`
- `deploy/dashboards/mister-smith-overview.json`
- `deploy/dashboards/mister-smith-autonomy.json`
- `deploy/alerts/mister-smith-rules.yml`
- `deploy/alerts/mister-smith-autonomy-rules.yml`
- recent code history under:
  - `crates/mister-smith-runtime`
  - `crates/mister-smith-monitoring`
  - `crates/mister-smith-events`
  - `crates/mister-smith-nats`
  - `crates/mister-smith-agents`
  - `crates/mister-smith-app`
  - `crates/mister-smith-persistence`
  - `apps/operator-console/src/`
  - `apps/operator-console/src-tauri/`
- current operator-console timing sample:
  - `npm --prefix apps/operator-console run build`
  - `npm --prefix apps/operator-console test`

## Confirmed Regressions

None grounded in current evidence.

What the audit could confirm:

- runtime bootstrap on the reviewed live lanes was fast rather than degraded:
  - `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/runtime.log` recorded
    `Mister Smith ready` in `866ms`
  - `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/repaired-baseline-ready/20260405T180349Z/runtime.log`
    recorded `Mister Smith ready` in `649ms`
- operator-console build/test performance on current code is quick and stable enough that no build
  or Vitest slowdown is evident from this run
- the most obvious recent runtime-liveness failure in the April 5 live evaluation note was already
  repaired before this audit, so it is not a current open regression on `main`

## Plausible Concerns Needing Measurement

### Unattributed wall-clock time on the core task path

Successful live runs still spend materially longer in task execution than bootstrap alone would
suggest:

- `docs/plans/artifacts/live-runtime-proof-smoke/20260326T190228Z/task-poll.log`
  - `queued -> completed`: `27.203s`
  - `running -> completed`: `26.196s`
- `docs/plans/artifacts/2026-04-05-live-runtime-eval-specs-022-026/repaired-baseline-ready/20260405T180349Z/task-poll.log`
  - `queued -> completed`: `16.158s`
  - `running -> completed`: `15.155s`

That is a real measurement, but not yet a regression claim. The audit could not attribute the
time between provider work, orchestration overhead, NATS/PostgreSQL effects, or slow inspect/read
paths because the current observability surfaces are too coarse:

- `crates/mister-smith-monitoring/src/prometheus.rs` only documents standard histograms for
  `mistersmith_task_duration_seconds`, `mistersmith_message_latency_seconds`, and
  `mistersmith_health_check_duration_seconds`
- `crates/mister-smith-app/src/bootstrap.rs` records only one aggregate `Mister Smith ready`
  duration at the end of bootstrap
- `deploy/dashboards/mister-smith-overview.json` and
  `deploy/alerts/mister-smith-rules.yml` visualize or alert on queue depth, task duration,
  message latency, and health-check latency, but not task lifecycle phase timings or inspect/read
  path latency

### Operator-console selected-run rendering remains unbenchmarked in-browser

Recent packets `021`, `023`, `025`, and `026` added more selected-run detail rendering in
`apps/operator-console/src/views/RunsView.tsx`, but this audit did not capture browser-side render
timings. Current build/test timings are fast, so there is no grounded frontend regression, but
render-cost measurement is still missing if the selected-run surface becomes sluggish under larger
payloads.

## Highest-Leverage Fix Directions

The highest-leverage bounded follow-up is instrumentation, not broad optimization:

1. Add runtime lifecycle metrics for `submit -> running` and `running -> completed`.
2. Add duration metrics for `GET /api/v1/tasks/{task_id}`,
   `GET /api/v1/autonomy/status/{workflow_id}`, and session inspect/read surfaces.
3. Split bootstrap timing into subphases at least for NATS connect, runtime task service
   bootstrap, and total ready-state completion.
4. Surface those metrics in the existing overview/autonomy dashboards and alerts so future audits
   can distinguish provider cost from runtime overhead.
5. Optionally emit the same timings into smoke-harness artifacts so proof bundles stay auditable
   without a live Prometheus scrape.

This direction is tracked in:

- GitHub `#329`: <https://github.com/MattMagg/MisterSmith/issues/329>
- Linear `MS-126`: <https://linear.app/agentic-ops/issue/MS-126>

## Issues Opened

- GitHub `#329` <https://github.com/MattMagg/MisterSmith/issues/329>
  - type: feature request
  - labels: `enhancement`, `codex`, `rust`
  - scope: phase-level latency metrics for runtime lifecycle and inspect paths
- Linear `MS-126` <https://linear.app/agentic-ops/issue/MS-126>
  - project: `MisterSmith Validated Backlog`
  - state: `Backlog`
  - priority: `3`
  - labels: `Performance`, `Validated`, `source:spec-validation`, `crate:monitoring`,
    `Symphony Candidate`

## Validation Limits

- This audit did not run a fresh live runtime proof. It relied on the latest repo-native artifact
  lanes from `2026-03-26` and `2026-04-05`, plus one current operator-console timing sample.
- The task-path timing concern is real at the wall-clock level, but it is not yet attributable and
  therefore is not reported here as a confirmed regression.
- No in-browser selected-run render benchmark was captured during this run.
- No load test was run against NATS, PostgreSQL, or the autonomy/task inspect endpoints.
- Report-path validation for this automation run is limited to markdown verification and
  `git diff --check`.
