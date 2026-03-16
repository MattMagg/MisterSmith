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

Use `README.md`, `ROADMAP.md`, and `CLAUDE.md` as orientation entry points.
Treat `WORKFLOW.md` and `docs/linear/LINEAR.md` as the live control-plane contract.
Treat `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` as the current runtime-proof
direction when the task is about proving real end-to-end execution rather than adding another
implementation phase.
Treat `docs/plans/2026-03-16-winyear-frontier-direction.md` as the current forward-direction note
when the task is about what should happen next after the March 16 recovery landings.

## Build, Test, and Development Commands

Run from repository root unless noted.

```bash
cargo build --workspace                    # Build all crates
cargo test --workspace                     # Run all tests (1115+)
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
cargo test -p <crate-name>                 # Test a single crate
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
- PRs should include: concise problem/solution summary, touched files, validation commands run
- PR references use `(#NNN)` suffix

## Security & Configuration Tips

- Never commit secrets; use environment variables or GitHub Actions secrets for credentials
- Provider API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
- OAuth credentials: Claude subscription uses Keychain/file-based credential sources
