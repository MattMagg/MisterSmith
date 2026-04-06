# Contributing to Mister Smith

Thanks for taking the time to contribute.

Mister Smith is a Rust-based multi-agent orchestration operating system. Useful contributions
include runtime fixes, operator-surface improvements, documentation updates, validation hardening,
and repo-cleanup work that keeps the shipped surfaces honest.

## Before You Start

- Read [README.md](README.md) for the public repo overview.
- Read [docs/current-state.md](docs/current-state.md) for the current truth on `main`.
- Read [SECURITY.md](SECURITY.md) before reporting or discussing a security-sensitive issue.
- Search existing issues and pull requests before opening a new one.

## Development Setup

Prerequisites:

- Rust 1.88.0 or later
- Docker and Docker Compose
- `jq` is helpful for local inspection scripts

Bootstrap:

```bash
git clone https://github.com/MattMagg/MisterSmith.git
cd MisterSmith
cargo build --workspace
docker compose -f deploy/docker-compose.yml up -d postgres nats
```

## Recommended Workflow

1. Create a focused branch from `main`.
2. Make the smallest reviewable change that proves the requested behavior.
3. Update docs when the public behavior, operator surface, or setup story changes.
4. Run narrow validation first, then broader validation only when the affected surface warrants it.
5. Open a pull request with a concise summary and exact validation evidence.

## Validation

Use the narrowest checks that materially prove the change:

```bash
cargo build --workspace
cargo test -p <crate-name>
cargo clippy --workspace -- -D warnings
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
python3 -m py_compile scripts/live_runtime_proof_smoke.py
npx markdownlint-cli2 "spec/**/*.md" "*.md" --config .markdownlint.json
```

Guidance:

- Run crate-scoped tests while iterating.
- Run workspace-wide validation when touching shared contracts, core types, or cross-crate seams.
- If you change `scripts/live_runtime_proof_smoke.py`, run the smoke-harness tests listed above.
- If you change docs, run targeted markdown linting on the edited files.

## Pull Requests

Pull requests should include:

- the problem being solved
- the high-level implementation approach
- exact validation commands run
- any remaining risks, gaps, or follow-up work

Keep pull requests tightly scoped. Large mixed diffs are much harder to review correctly in this
repo.

## Commit Style

Conventional commits are preferred:

- `feat(<scope>): ...`
- `fix(<scope>): ...`
- `docs: ...`
- `chore: ...`

Examples:

- `feat(app): add session resume shell command`
- `fix(http): preserve runtime truth in session inspect response`
- `docs: refresh public README and contribution guidance`

## Design Expectations

When contributing, prefer:

- supervised and observable behavior over hidden magic
- explicit proof wording over overstated success claims
- bounded, reviewable changes over broad rewrites
- current repo truth over historical assumptions

## Questions and Support

For general usage questions or contribution-routing questions, start with [SUPPORT.md](SUPPORT.md).
