# MisterSmith GitHub Workflows

This directory contains the active GitHub Actions workflows for the Rust
workspace, Claude-assisted review flows, and the remaining documentation update
jobs.

## Active Workflows

### Core automation

- `ci.yml`: Required Rust CI for pushes and pull requests targeting `main`.
- `claude.yml`: On-demand Claude assistant for `@claude` issue and pull request
  interactions using `anthropics/claude-code-action@v1`.
- `claude-code-review.yml`: Automatic Claude review on pull request updates using
  the same OAuth-backed action.

### Documentation workflows

- `docs-async-runtime.yml`
- `docs-transport-messaging.yml`
- `docs-data-persistence.yml`
- `docs-security-crypto.yml`
- `documentation-validation.yml`
- `markdown-lint-fixer.yml`

### Legacy workflow

- `mistersmith-ci.yml`: Retained as a manual historical workflow for the old
  documentation-only phase. It is not part of the current implementation CI
  path.

## Required Secrets

Configure only the secrets used by the workflows that remain active:

- `CLAUDE_CODE_OAUTH_TOKEN`: Required by `claude.yml` and
  `claude-code-review.yml`.
- `ANTHROPIC_API_KEY`: Required by the documentation workflows that call
  Anthropic models directly.
- `OPENAI_API_KEY`: Required by the documentation workflows that call OpenAI
  models directly.

The legacy `grll/claude-code-action` OAuth bootstrap path has been removed, so
the old `CLAUDE_ACCESS_TOKEN`, `CLAUDE_REFRESH_TOKEN`, `CLAUDE_EXPIRES_AT`, and
`SECRETS_ADMIN_PAT` workflow path is no longer part of this repository.

## Notes

- The Rust CI is the repository's primary enforcement workflow.
- The documentation workflows are still scoped to the `spec/` corpus.
- Claude automation now uses a single authentication approach based on
  `CLAUDE_CODE_OAUTH_TOKEN` rather than maintaining two competing action
  families.
