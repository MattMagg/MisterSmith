# Dependency And SDK Drift Audit

Date: 2026-04-09
Repo HEAD: `e51c3bdd959448d5d27e3ac4bc71d40d5cca2a68` (`e51c3bd`)

## Summary

Real drift was found, but it is narrow.

- Confirmed drift exists in repo-state references and operator-console version metadata.
- The Rust workspace baseline, MCP baseline, and live runtime-proof provider/model claims are internally consistent.
- The only issue pair opened from this run is for floating observability image tags in
  `deploy/docker-compose.yml`, because that is a justified runtime-version alignment proposal with a
  bounded repo-owned fix.

## Repo Evidence

- Workspace baseline:
  - `Cargo.toml` pins `rust-version = "1.88"`, `tokio = "1.49.0"`, `thiserror = "2.0.18"`,
    `async-nats = "0.46.0"`, and `reqwest = "0.12"`.
  - `Cargo.lock` resolves `tokio 1.50.0`, `thiserror 2.0.18`, `async-nats 0.46.0`,
    `reqwest 0.12.28`, and `rmcp 1.3.0`.
- Reference docs:
  - `VERSION_REFERENCE.md` current workspace baseline still matches the checked-in manifests and
    lockfile for the repo-truth section refreshed on `2026-04-06`.
  - `CLAUDE.md` technology-stack notes match the current workspace truth for Rust 1.88, Tokio
    1.49.x workspace baseline with a 1.50.0 lockfile resolution, `rmcp 1.3.0`, and `thiserror 2.x`.
- Runtime-proof support claims:
  - `README.md`, `docs/current-state.md`, `docs/ms_recent_context.md`, and
    `scripts/live_runtime_proof_smoke.py` all still align on the bounded live-proof baseline:
    `openai_chatgpt` with `gpt-5.4`.
- Operator console:
  - `apps/operator-console/package.json` and `apps/operator-console/package-lock.json` root version
    are `0.0.0`.
  - `apps/operator-console/src-tauri/Cargo.toml` and
    `apps/operator-console/src-tauri/tauri.conf.json` both declare `0.1.0`.
- Repo-state trackers:
  - `docs/current-state.md` says `main` is synced at
    `de338ee68ec8a1dd55209f130eab423560d52412`.
  - `docs/ms_recent_context.md` says `main` is synced at `de338ee`.
  - `git rev-parse HEAD` reports `e51c3bdd959448d5d27e3ac4bc71d40d5cca2a68`.
- Deploy stack:
  - `deploy/docker-compose.yml` pins `nats:2.12.4-alpine` and `postgres:15-alpine`.
  - The same file still uses `otel/opentelemetry-collector-contrib:latest` and
    `grafana/grafana:latest`.

## Confirmed Drift

1. Stale repo HEAD references in current-state docs
   - `docs/current-state.md` and `docs/ms_recent_context.md` still point at `de338ee...`, while the
     repo is now at `e51c3bd`.
   - This is confirmed internal drift in repo-state reference material.

2. Mixed operator-console version metadata on the same shipped surface
   - The web package root stays at `0.0.0`, but the Tauri app manifest and config are both `0.1.0`.
   - This is a real version-reference split inside one repo-owned product surface even though it is
     not currently proven to break builds.

## Unclear Targets Or Open Questions

1. Observability image version targets are not declared
   - `deploy/docker-compose.yml` uses floating `latest` tags for Grafana and the OpenTelemetry
     Collector.
   - This is a justified alignment proposal, but the exact target versions should be chosen
     deliberately because the repo does not currently declare a supported baseline for those two
     images.

2. Operator-console version intent should be clarified
   - If `0.0.0` is an intentional unpublished placeholder for the npm package root, document that
     explicitly.
   - If not, align the package root and lockfile root to `0.1.0` so the JS and Tauri surfaces stop
     disagreeing.

## Minimal Alignment Plan

1. Refresh the stale `main` SHA references in `docs/current-state.md` and `docs/ms_recent_context.md`.
2. Decide whether the operator-console package root should advertise `0.1.0` or remain a documented
   placeholder; then align `package.json` and `package-lock.json` root metadata accordingly.
3. Choose explicit supported versions for Grafana and the OpenTelemetry Collector, pin them in
   `deploy/docker-compose.yml`, and add any needed version-reference note.

## Issues Opened

- GitHub: [#333](https://github.com/MattMagg/MisterSmith/issues/333) -
  `Pin explicit observability image versions in docker-compose`
- Linear: [MS-128](https://linear.app/agentic-ops/issue/MS-128) -
  `Pin explicit Grafana and OTel Collector versions in deploy/docker-compose.yml`

No issue was opened for the stale SHA references or the operator-console version split because both
are low-risk, repo-local alignment items with straightforward fixes once someone chooses to touch
those files.

## Validation Limits

- This run was intentionally repo-grounded only. No external registry lookup or image-tag research
  was used to guess target versions.
- Validation was limited to re-reading the named manifests, lockfiles, and support docs plus
  confirming the current repo SHA and issue state.
- No code, manifest, lockfile, or deploy configuration was changed in this run.
