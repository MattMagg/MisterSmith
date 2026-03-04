# Repository Guidelines

## Project Structure & Module Organization
This repository is primarily a specification and planning workspace for Mister Smith.

- `spec/`: Canonical architecture, transport, security, data, and testing specs (main contribution area).
- `plans/`: Implementation planning docs and batch trackers.
- `archive/`: Historical validation/research artifacts; avoid editing unless explicitly needed.
- `docs/`, `logs/`: Supporting documentation and run logs.
- `nats.rs/`: Vendored upstream Rust NATS workspace used as API reference and optional implementation/test surface.

Use `README.md`, `ROADMAP.md`, and `VALIDATION_REPORT.md` as orientation entry points before editing specs.

## Build, Test, and Development Commands
Run from repository root unless noted.

- `npx markdownlint-cli2 "spec/**/*.md" "*.md" --config .markdownlint.json`: lint Markdown against project rules.
- `git grep -nE "TODO|TBD|FIXME" spec/`: catch unfinished spec language before PR.
- `cd nats.rs/async-nats && cargo test --features slow_tests,websockets -- --nocapture`: full async-nats test path used by CI.
- `cd nats.rs && cargo +nightly fmt -- --check && cargo clippy --benches --tests --examples --all-features -- --deny clippy::all`: formatting and lint gates for Rust changes.

If running `nats.rs` tests locally, install server first: `go install github.com/nats-io/nats-server/v2@main`.

## Coding Style & Naming Conventions
- Markdown: ATX headings, 2-space list indentation, and 200-char max line length (see `.markdownlint.json`).
- Spec docs: prefer lowercase kebab-case filenames (for example, `spec/core-architecture/system-architecture.md`).
- Keep terminology consistent with existing domain docs; update cross-references when renaming files.
- Rust (only when editing `nats.rs/`): follow rustfmt defaults in `nats.rs/.rustfmt.toml` (`max_width = 100`), no clippy warnings.

## Testing Guidelines
- Documentation changes must at minimum pass markdown linting and basic broken-link sanity checks in edited files.
- Rust changes in `nats.rs/` should run targeted crate tests plus relevant feature/TLS variants when applicable.
- Test files in `nats.rs` follow `*_tests.rs` / domain-specific naming (`kv_tests.rs`, `service_tests.rs`); mirror that pattern.

## Commit & Pull Request Guidelines
- Follow observed commit style: `docs: ...`, `fix(spec): ...`, `refactor(spec): ...`, `feat: ...`, `chore: ...`.
- Keep commits atomic and scoped to one concern.
- PRs should include:
  - concise problem/solution summary,
  - touched directories/files,
  - validation commands run and outcomes,
  - linked issue (if available).
- Include screenshots only for workflow/UI-facing changes; not required for spec-only text edits.

## Security & Configuration Tips
- Never commit secrets; use GitHub Actions secrets (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) for workflow credentials.
- Treat `mistersmith-api.json` and workflow files as integration surfaces: validate references and paths after edits.

## Active Technologies
- Markdown specifications + Rust contract references (MSRV 1.88 context) + Existing canonical docs in `spec/core-architecture/`, `spec/operations/`; repo checks via `rg`, `cargo`, `markdownlint` (001-phase1-foundation)
- N/A (documentation artifacts only) (001-phase1-foundation)
- Markdown specifications + Rust-oriented contract references (Tokio 1.49.0 baseline context) + Canonical docs in `spec/core-architecture/`, `spec/data-management/`, `spec/operations/`; repository checks via `rg` and `markdownlint` (002-phase2-runtime-async)

## Recent Changes
- 001-phase1-foundation: Added Markdown specifications + Rust contract references (MSRV 1.88 context) + Existing canonical docs in `spec/core-architecture/`, `spec/operations/`; repo checks via `rg`, `cargo`, `markdownlint`
