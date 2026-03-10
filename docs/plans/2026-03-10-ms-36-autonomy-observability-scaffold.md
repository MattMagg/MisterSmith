# MS-36 Autonomy Observability Scaffold Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add standalone Phase 10 autonomy dashboard and alert scaffolding plus deploy mapping notes without pulling blocked runtime work into scope.

**Architecture:** Keep the existing overview dashboard and phase-8 alert file untouched. Add one new dashboard JSON and one new alert rules YAML that use `mistersmith_` placeholder metric names mapped directly to the `AutonomyStatusView` contract, then document the intended runtime wiring in `deploy/README.md`.

**Tech Stack:** Grafana dashboard JSON, Prometheus alert rules YAML, Markdown documentation, narrow CLI syntax validation (`python3`, `ruby` with `Psych`, optional `promtool` when installed)

---

### Task 1: Stand up the autonomy dashboard scaffold

**Files:**
- Create: `deploy/dashboards/mister-smith-autonomy.json`
- Reference: `deploy/dashboards/mister-smith-overview.json`
- Reference: `specs/012-phase10-frontier-autonomy/contracts/autonomy-observability.md`

**Step 1: Reuse the existing dashboard conventions**

Read the current dashboard scaffold and copy its top-level structure:

- `deploy/dashboards/mister-smith-overview.json`
- Match `schemaVersion`, `tags`, `time`, `editable`, and `gridPos` style.

**Step 2: Add only the Phase 10 operator surface**

Create panels for:

- topology choice and rationale
- branch health
- checkpoint freshness
- context pressure
- intervention history
- delegation/provenance rejections

Use placeholder `mistersmith_autonomy_*` PromQL queries and panel descriptions that explicitly say `MS-33` must wire the runtime emitters.

**Step 3: Validate the dashboard syntax**

Run:

```bash
python3 scripts/validate_deploy_assets.py deploy/dashboards
```

Expected: every dashboard JSON parses cleanly.

### Task 2: Add autonomy alert scaffolding

**Files:**
- Create: `deploy/alerts/mister-smith-autonomy-rules.yml`
- Reference: `deploy/alerts/mister-smith-rules.yml`
- Reference: `specs/012-phase10-frontier-autonomy/tasks.md:221`

**Step 1: Match the existing alert rule shape**

Reuse the current `groups -> rules -> labels/annotations` YAML structure and severity conventions from `deploy/alerts/mister-smith-rules.yml`.

**Step 2: Add the four requested autonomy alerts**

Create rules for:

- checkpoint staleness
- intervention spikes
- context pressure
- provenance rejection visibility

Use placeholder `mistersmith_autonomy_*` metrics and annotations that describe the intended operator meaning without claiming runtime support exists yet.

**Step 3: Validate the YAML syntax**

Run:

```bash
python3 scripts/validate_deploy_assets.py deploy/alerts
```

Expected: every alert file parses cleanly.

### Task 3: Document placeholder metric mappings and validation

**Files:**
- Modify: `deploy/README.md`
- Reference: `specs/012-phase10-frontier-autonomy/contracts/autonomy-observability.md`
- Reference: `crates/mister-smith-monitoring/src/prometheus.rs`

**Step 1: Add a Phase 10.5 scaffold section**

Document:

- the new dashboard and alert files
- the mapping from `AutonomyStatusView` fields to placeholder metric names
- the label expectations each placeholder query assumes

**Step 2: Record the unresolved runtime wiring points**

Call out the specific query assumptions `MS-33` must finalize, especially:

- whether topology rationale is a safe Prometheus label
- whether branch state emits one gauge per branch or aggregated counters
- whether context pressure is normalized as a ratio
- whether delegation rejection alerts should exclude general auth failures

**Step 3: Run final narrow validation and review**

Run:

```bash
python3 scripts/validate_deploy_assets.py
```

Then run `vet` against the final diff and update the Linear workpad with the validation evidence and remaining placeholders.
