# Deployment Notes

## NATS Version Requirement

The repo-managed deploy artifacts pin `nats-server` to `v2.12.4-alpine`.
This satisfies the Phase 9.1 security floor of `>= v2.11.1` required to
mitigate CVE-2025-30215.

## Version Checks

Verify the pinned local-development image:

```bash
docker compose -f deploy/docker-compose.yml config | rg "nats:2.12.4-alpine"
docker run --rm nats:2.12.4-alpine --version
```

Verify the Kubernetes image after applying the manifests:

```bash
kubectl -n mister-smith get deployment nats -o jsonpath='{.spec.template.spec.containers[0].image}'
kubectl -n mister-smith exec deploy/nats -- nats-server --version
```

## Permission Audit

Before shipping NATS auth changes, scan repo-managed config files for forbidden
wildcard permissions:

```bash
python3 scripts/audit_nats_permissions.py deploy
```

The audit fails on wildcard `>` and `$JS.>` permissions. Documentation and spec
Markdown are excluded on purpose so example snippets do not produce false
positives.

## Phase 10.5 autonomy observability scaffold

This prep slice adds deploy-only operator assets for the blocked Phase 10.5
autonomy view work:

- `deploy/dashboards/mister-smith-autonomy.json`
- `deploy/alerts/mister-smith-autonomy-rules.yml`

These assets are intentionally standalone. They do not implement
`AutonomyStatusView`, event aggregation, or app wiring. `MS-33` remains the
implementation issue that must emit the runtime signals and finalize the
placeholder queries below.

### Placeholder metric mapping

The new dashboard and alert rules assume typed autonomy state is exported as
Prometheus metrics with the existing `mistersmith_` prefix. The mapping target
is the Phase 10 `AutonomyStatusView` contract, not raw logs.

| `AutonomyStatusView` field | Placeholder metric(s) | Expected labels | Used by |
| --- | --- | --- | --- |
| `graph` | `mistersmith_autonomy_workflows_active` | `workflow_id`, `status` | Active autonomy workflow stat |
| `topology` | `mistersmith_autonomy_topology_info`, `mistersmith_autonomy_topology_selections_total` | `workflow_id`, `topology`, `rationale` | Topology table and selection trends |
| `branches` | `mistersmith_autonomy_branches`, `mistersmith_autonomy_branch_checkpoint_age_seconds` | `workflow_id`, `branch_id`, `state`, `recovery_state` | Branch health panels and checkpoint staleness alert |
| `memory_pressure` | `mistersmith_autonomy_context_pressure_ratio` | `workflow_id`, `branch_id`, `pressure_level` | Context pressure panels and critical alert |
| `interventions` | `mistersmith_autonomy_interventions_total` | `workflow_id`, `branch_id`, `intervention`, `decision_source` | Intervention history panel and spike alert |
| `delegation_alerts` | `mistersmith_autonomy_delegation_rejections_total` | `workflow_id`, `reason`, `scope`, `issuer`, `recipient` | Provenance rejection panel and visibility alert |

### Placeholder finalization notes for `MS-33`

The scaffold is intentionally opinionated but not final. `MS-33` must confirm
or revise these assumptions when typed runtime state exists:

- `mistersmith_autonomy_workflows_active` should represent operator-visible
  active workflow count. If runtime state prefers a per-workflow info metric
  instead, update the stat query but keep the panel.
- `mistersmith_autonomy_topology_info` currently assumes topology rationale can
  be emitted as a label. If the rationale text is too high-cardinality, replace
  it with a stable rationale code and keep the human-readable text in the app
  surface.
- `mistersmith_autonomy_branches` assumes one gauge per branch/state. If the
  runtime emits counters or separate health metrics instead, preserve the panel
  intent and change only the PromQL.
- `mistersmith_autonomy_branch_checkpoint_age_seconds` assumes "seconds since
  last durable checkpoint" semantics. Use a freshness metric with the same
  meaning if the implementation prefers a different name.
- `mistersmith_autonomy_context_pressure_ratio` assumes normalized `0..1`
  pressure semantics. The dashboard thresholds and critical alert depend on that
  contract.
- `mistersmith_autonomy_interventions_total` should count targeted Guard or
  Advisor decisions only. Do not mix it with generic restart or failure metrics.
- `mistersmith_autonomy_delegation_rejections_total` is reserved for
  operator-visible provenance or delegation chain failures. Do not point it at
  the existing `mistersmith_unauthorized_operations_total` counter, which is
  broader than the Phase 10 autonomy surface.

### Syntax validation commands

Validate all repo-managed dashboard and alert assets with the checked-in script:

```bash
python3 scripts/validate_deploy_assets.py
```

This validator requires:

- `python3` for dashboard JSON parsing
- `ruby` with `Psych` available on `PATH` for alert rule YAML parsing

The script validates:

- every `deploy/dashboards/*.json` file via Python's `json` module
- every `deploy/alerts/*.yml` or `*.yaml` file via Ruby `Psych`
