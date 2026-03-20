# MisterSmith GitHub Workflows

This directory contains the active GitHub Actions workflows for the Rust
workspace and the remaining on-demand Claude assistant path.

Repository-level AI review configuration now lives in `.coderabbit.yaml`, which
is consumed by the installed CodeRabbit app rather than GitHub Actions.

## Active Workflows

### Core automation

- `ci.yml`: Required Rust CI for code-changing pushes and pull requests
  targeting `main`. It remains the only substantive repository merge gate and
  now runs for workflow-file changes as well.
- `claude.yml`: On-demand Claude assistant for `@claude` issue and pull request
  interactions using `anthropics/claude-code-action@v1`. It is not part of the
  default merge gate.

### Archived workflow

- `mistersmith-ci.yml`: Archived under `.github/deactived-workflows/` as a
  historical documentation-phase workflow. It is no longer active.

## Required Secrets

Configure only the secrets used by the workflows that remain active:

- `CLAUDE_CODE_OAUTH_TOKEN`: Required by `claude.yml`.

The legacy `grll/claude-code-action` OAuth bootstrap path has been removed, so
the old `CLAUDE_ACCESS_TOKEN`, `CLAUDE_REFRESH_TOKEN`, `CLAUDE_EXPIRES_AT`, and
`SECRETS_ADMIN_PAT` workflow path is no longer part of this repository.

## Notes

- The Rust CI is the repository's primary enforcement workflow.
- `Human Review` remains the native Symphony state name, but delegated-agent
  review may satisfy that checkpoint when the operator has already granted
  review-and-merge authority in the active session.
- Documentation linting and packet markdown checks remain local or issue-scoped
  validation, not scheduled GitHub automation.
- Claude automation now uses a single authentication approach based on
  `CLAUDE_CODE_OAUTH_TOKEN` for explicit `@claude` requests only.
