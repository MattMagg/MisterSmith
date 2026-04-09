# Dependency And SDK Drift Audit

Date: 2026-04-09
Repo HEAD: `aae62e5f932a99b96efce5686e8c1d9c0a4635a1` (`aae62e5`)

## Summary

Real drift was found, but it is still narrow and mostly repo-local.

- The Rust workspace manifests, `Cargo.lock`, operator-console JS lockfile, Tauri lockfile, and
  runtime-proof provider/model claims are internally consistent.
- Confirmed drift exists in one deploy/runtime healthcheck path and one repo-owned technology-stack
  feature note.
- An earlier issue pair from today remains valid for floating observability image tags, but those
  exact target versions are still a deliberate selection rather than a repo-provable current bug.

## Repo Evidence

- Workspace baseline:
  - [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml) pins `rust-version = "1.88"`,
    `tokio = "1.49.0"`, `async-nats = "0.46.0"`, `thiserror = "2.0.18"`, `reqwest = "0.12"`,
    `opentelemetry = "0.31.0"`, and `tracing-opentelemetry = "0.32.1"`.
  - [`Cargo.lock`](/Users/macmain/MisterSmith/Cargo.lock) resolves `tokio 1.50.0`,
    `async-nats 0.46.0`, `reqwest 0.12.28`, `rmcp 1.3.0`, and the expected observability stack.
- Version reference:
  - [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md) still matches the
    current repo-truth baseline sections for Rust 1.88, async-nats 0.46.0, reqwest 0.12.x, and the
    current workspace dependency table.
- Operator console:
  - [`apps/operator-console/package.json`](/Users/macmain/MisterSmith/apps/operator-console/package.json)
    and
    [`apps/operator-console/package-lock.json`](/Users/macmain/MisterSmith/apps/operator-console/package-lock.json)
    agree on the JS root dependency ranges and lockfile root metadata.
  - [`apps/operator-console/src-tauri/Cargo.toml`](/Users/macmain/MisterSmith/apps/operator-console/src-tauri/Cargo.toml)
    and
    [`apps/operator-console/src-tauri/Cargo.lock`](/Users/macmain/MisterSmith/apps/operator-console/src-tauri/Cargo.lock)
    agree on the current Tauri 2.x surface.
- Runtime-proof support claims:
  - [`README.md`](/Users/macmain/MisterSmith/README.md),
    [`docs/current-state.md`](/Users/macmain/MisterSmith/docs/current-state.md),
    [`docs/ms_recent_context.md`](/Users/macmain/MisterSmith/docs/ms_recent_context.md), and
    [`scripts/live_runtime_proof_smoke.py`](/Users/macmain/MisterSmith/scripts/live_runtime_proof_smoke.py)
    all still align on the bounded live-proof baseline: `openai_chatgpt` with `gpt-5.4`.
- Deploy/runtime definitions:
  - [`deploy/Dockerfile`](/Users/macmain/MisterSmith/deploy/Dockerfile) installs only
    `ca-certificates` in the runtime stage, then defines `HEALTHCHECK ... CMD curl -f
    http://localhost:8080/health/live`.
  - [`deploy/docker-compose.yml`](/Users/macmain/MisterSmith/deploy/docker-compose.yml) uses the
    same built image for the `mister-smith` service and also runs `curl` inside the container for
    the service healthcheck.
- Repo-owned tech-stack note:
  - [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md) says the async-nats baseline includes
    `jetstream, kv, object-store, service features`, while
    [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml) enables only `jetstream`, `kv`, and
    `service`.

## Confirmed Drift

1. Runtime image healthcheck dependency mismatch
   - [`deploy/Dockerfile`](/Users/macmain/MisterSmith/deploy/Dockerfile) runtime stage does not
     install `curl`, but both the image `HEALTHCHECK` and the compose-level `mister-smith`
     healthcheck invoke `curl` inside the container.
   - This is a real repo-owned runtime/deploy bug because the health signal depends on a binary the
     runtime image does not provide.

2. async-nats feature note drift in repo documentation
   - [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md) still describes the current async-nats
     baseline as including `object-store`, but the workspace dependency in
     [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml) does not enable that feature.
   - This is confirmed documentation drift on a repo-owned version/features reference surface.

## Unclear Targets Or Open Questions

1. Floating observability image tags still need explicit targets
   - [`deploy/docker-compose.yml`](/Users/macmain/MisterSmith/deploy/docker-compose.yml) still uses
     `otel/opentelemetry-collector-contrib:latest` and `grafana/grafana:latest`.
   - This remains a justified alignment proposal, but the repo still does not declare the exact
     supported Grafana and collector versions to pin.
   - Existing tracking already covers this: GitHub [#333](https://github.com/MattMagg/MisterSmith/issues/333)
     and Linear [MS-128](https://linear.app/agentic-ops/issue/MS-128).

2. Operator-console root version intent is still ambiguous
   - The JS root stays at `0.0.0`, while the Tauri app surface is `0.1.0`.
   - That may be intentional placeholder metadata for a private frontend package, or it may be an
     alignment task. The repo does not currently declare the intended target clearly enough to call
     it confirmed drift by itself.

3. Current-state SHA references may already be in flight
   - [`docs/current-state.md`](/Users/macmain/MisterSmith/docs/current-state.md) and
     [`docs/ms_recent_context.md`](/Users/macmain/MisterSmith/docs/ms_recent_context.md) still
     mention older `main` SHAs in their current file contents, but both files are already locally
     modified in the worktree during this run.
   - Because those documents are already in flight, this audit records the mismatch but does not
     open new tracking for it.

## Minimal Alignment Plan

1. Fix the deploy/runtime healthcheck mismatch by either installing `curl` in the runtime image or
   replacing both probe commands with a probe that the image already guarantees.
2. Correct the async-nats feature note in [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md) so it
   matches the actual workspace feature set.
3. When the in-flight current-state docs are next touched, refresh any stale `main` SHA references
   in the same pass.
4. Keep the separate observability pinning follow-up under [#333](https://github.com/MattMagg/MisterSmith/issues/333)
   and [MS-128](https://linear.app/agentic-ops/issue/MS-128) until explicit target versions are
   chosen.

## Issues Opened

- Existing from the earlier 2026-04-09 run:
  - GitHub [#333](https://github.com/MattMagg/MisterSmith/issues/333) -
    `Pin explicit observability image versions in docker-compose`
  - Linear [MS-128](https://linear.app/agentic-ops/issue/MS-128) -
    `Pin explicit Grafana and OTel Collector versions in deploy/docker-compose.yml`
- Opened in this revision:
  - GitHub [#334](https://github.com/MattMagg/MisterSmith/issues/334) -
    `Fix runtime image healthcheck dependency mismatch`
  - Linear [MS-129](https://linear.app/agentic-ops/issue/MS-129) -
    `Fix runtime image healthcheck dependency mismatch`

No separate issue was opened for the async-nats feature-note drift because it is a small
documentation-only correction. No new issue was opened for the stale SHA references because those
two files are already locally modified and may already be in the middle of being refreshed.

## Validation Limits

- This run stayed repo-grounded. No external registry or package-index lookup was used to guess
  target versions.
- Validation was limited to re-reading the named manifests, lockfiles, deploy/runtime files, and
  repo-owned support docs, plus confirming current GitHub and Linear issue state.
- No source, manifest, lockfile, or deploy config was changed in this run beyond this audit report.
