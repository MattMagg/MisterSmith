# Dependency And SDK Drift Report

## Summary

Real repo-local drift was found, but it is documentation/reference drift rather than a live build or runtime break.

- Confirmed drift: `VERSION_REFERENCE.md` and `CLAUDE.md` no longer match the current workspace dependency state for `thiserror`, `rmcp`, `tokio`, and `reqwest`.
- No drift: the runtime-proof provider/model baseline is still consistently documented as `openai_chatgpt` with `gpt-5.4`, and the operator-console JavaScript/Tauri manifests are coherent with their lockfiles.
- Open question: `deploy/docker-compose.yml` still uses floating `latest` tags for Grafana and the OpenTelemetry collector, but the repo does not declare a pinned support baseline for those images, so repo-local drift cannot be proved from this run alone.

## Repo Evidence

- Workspace dependencies pin `thiserror = "2.0.18"`, `tokio = "1.49.0"`, and `reqwest = "0.12"` in [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml#L32C1).
- Workspace crates consume `thiserror 2.0.18`; the lockfile also contains transitive `thiserror 1.0.69` from external dependencies in [`Cargo.lock`](/Users/macmain/MisterSmith/Cargo.lock#L4120C1).
- `mister-smith-core` inherits `thiserror` from the workspace in [`crates/mister-smith-core/Cargo.toml`](/Users/macmain/MisterSmith/crates/mister-smith-core/Cargo.toml#L9C1).
- `mister-smith-mcp` inherits workspace `thiserror` and `reqwest`, and pins `rmcp = "1.3.0"` in [`crates/mister-smith-mcp/Cargo.toml`](/Users/macmain/MisterSmith/crates/mister-smith-mcp/Cargo.toml#L9C1).
- `VERSION_REFERENCE.md` still says the `thiserror` decision is pending, recommends staying on `1.0.69`, and presents `reqwest 0.13.2` / `tokio 1.49.0` as the implementation matrix in [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md#L40C1) and [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md#L290C1).
- `CLAUDE.md` still lists `Tokio 1.49.0`, `rmcp 1.1.0`, and `thiserror 1.x` in the repo technology table in [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md#L209C1).
- Runtime-proof support claims remain aligned across [`README.md`](/Users/macmain/MisterSmith/README.md#L10C1), [`docs/current-state.md`](/Users/macmain/MisterSmith/docs/current-state.md#L89C1), [`docs/ms_recent_context.md`](/Users/macmain/MisterSmith/docs/ms_recent_context.md#L12C1), and [`scripts/live_runtime_proof_smoke.py`](/Users/macmain/MisterSmith/scripts/live_runtime_proof_smoke.py#L4C1).
- The deploy stack still uses floating `latest` tags for collector/dashboard images in [`deploy/docker-compose.yml`](/Users/macmain/MisterSmith/deploy/docker-compose.yml#L35C1).

## Confirmed Drift

### 1. Version-reference docs still describe `thiserror` as undecided / 1.x, but the workspace is already on 2.0.18

- [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml#L34C1) pins `thiserror = "2.0.18"`.
- Workspace crates such as [`crates/mister-smith-core/Cargo.toml`](/Users/macmain/MisterSmith/crates/mister-smith-core/Cargo.toml#L9C1) inherit that workspace dependency, and the workspace lockfile shows those crates consuming `thiserror 2.0.18` in [`Cargo.lock`](/Users/macmain/MisterSmith/Cargo.lock#L2268C1) and [`Cargo.lock`](/Users/macmain/MisterSmith/Cargo.lock#L4130C1).
- [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md#L170C1) still says the decision is pending and recommends staying on `1.0.69`, while [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md#L222C1) still says `thiserror 1.x`.

### 2. Reference docs no longer match the current repo truth for `rmcp`, `reqwest`, and the resolved Tokio version

- [`crates/mister-smith-mcp/Cargo.toml`](/Users/macmain/MisterSmith/crates/mister-smith-mcp/Cargo.toml#L25C1) pins `rmcp = "1.3.0"`, but [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md#L216C1) still says `1.1.0`.
- [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml#L43C1) keeps `tokio = "1.49.0"` as the semver lower bound, but the current lockfile resolves `tokio 1.50.0`; the docs still present `1.49.0` as the current runtime version in [`CLAUDE.md`](/Users/macmain/MisterSmith/CLAUDE.md#L212C1) and [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md#L297C1).
- Workspace crates currently consume `reqwest 0.12.28`, while [`VERSION_REFERENCE.md`](/Users/macmain/MisterSmith/VERSION_REFERENCE.md#L333C1) presents `reqwest 0.13.2` as the implementation-time workspace dependency even though that upgrade has not landed in [`Cargo.toml`](/Users/macmain/MisterSmith/Cargo.toml#L45C1).

## Unclear Targets Or Open Questions

- [`deploy/docker-compose.yml`](/Users/macmain/MisterSmith/deploy/docker-compose.yml#L35C1) uses `otel/opentelemetry-collector-contrib:latest` and `grafana/grafana:latest`. That is a drift risk, but the repo does not declare a pinned supported version for either image, so no repo-local target can be proved from this run.
- `VERSION_REFERENCE.md` mixes current-state reference material with proposed upgrade guidance. The repo should decide whether that file is meant to describe current workspace truth, future upgrade candidates, or both, then label the sections accordingly.

## Minimal Alignment Plan

1. Refresh `VERSION_REFERENCE.md` so current workspace truth is explicit: `thiserror 2.0.18`, `rmcp 1.3.0`, current `tokio` lock resolution, and current `reqwest 0.12.x` usage.
2. Keep future upgrade ideas, if retained, under clearly marked proposal text rather than the current-state matrix.
3. Refresh the dependency/version table in `CLAUDE.md` to match current workspace reality.
4. Leave Grafana/OpenTelemetry image pinning for a separate decision unless the repo first defines the supported baseline.

## Issues Opened

- GitHub: [#328 Refresh dependency reference docs to match workspace pins](https://github.com/MattMagg/MisterSmith/issues/328)
- Linear: [MS-125 Refresh dependency reference docs to match workspace pins](https://linear.app/agentic-ops/issue/MS-125)

## Validation Limits

- This run only re-read repo manifests, lockfiles, deploy files, script constants, and reference docs.
- No dependency upgrades were performed.
- No external registry lookups were used, so floating-tag risk was noted only where the repo lacked its own target version.
