# MisterSmith GitHub Workflows

This directory contains the active GitHub Actions workflows for the Rust
workspace, Claude-assisted review flows, and the remaining documentation update
jobs.

## Active Workflows

### Core automation

- `ci.yml`: Required Rust CI for code-changing pushes and pull requests
  targeting `main`. It is intentionally scoped away from docs-only and
  release-note-only changes, and it cancels superseded runs for the same PR.
- `vet.yml`: Pull-request review workflow that runs `imbue-ai/vet` with the
  repository's `.vet/configs.toml` CI profile and posts findings back to the
  PR. It is scoped to code-changing PRs, skips Dependabot, and skips cleanly
  when `OPENAI_API_KEY` is unavailable.
- `claude.yml`: On-demand Claude assistant for `@claude` issue and pull request
  interactions using `anthropics/claude-code-action@v1`.
- `claude-code-review.yml`: Automatic Claude review on code-changing pull
  request updates using the same OAuth-backed action. It skips Dependabot and
  cancels superseded runs for the same PR.
- `pr-labeler.yml`: Automatic pull request labeling based on changed files and
  branch patterns.
- `release-drafter.yml`: Maintains a draft release on `main` so release notes
  are ready when tags and published releases start being used.

### Documentation workflows

- `docs-async-runtime.yml`
- `docs-transport-messaging.yml`
- `docs-data-persistence.yml`
- `docs-security-crypto.yml`
- `documentation-validation.yml`
- `markdown-lint-fixer.yml`

### Archived workflow

- `mistersmith-ci.yml`: Archived under `.github/deactived-workflows/` as a
  historical documentation-phase workflow. It is no longer active.

## Required Secrets

Configure only the secrets used by the workflows that remain active:

- `CLAUDE_CODE_OAUTH_TOKEN`: Required by `claude.yml` and
  `claude-code-review.yml`.
- `ANTHROPIC_API_KEY`: Required by the documentation workflows that call
  Anthropic models directly.
- `OPENAI_API_KEY`: Required by the documentation workflows that call OpenAI
  models directly and by `vet.yml`, which uses the repo-local `gpt-5.2`
  profile from `.vet/configs.toml`.

The legacy `grll/claude-code-action` OAuth bootstrap path has been removed, so
the old `CLAUDE_ACCESS_TOKEN`, `CLAUDE_REFRESH_TOKEN`, `CLAUDE_EXPIRES_AT`, and
`SECRETS_ADMIN_PAT` workflow path is no longer part of this repository.

## Notes

- The Rust CI is the repository's primary enforcement workflow.
- Vet adds LLM-based PR review on top of CI instead of replacing compile/test
  validation.
- `Human Review` remains the native Symphony state name, but delegated-agent
  review may satisfy that checkpoint when the operator has already granted
  review-and-merge authority in the active session.
- The documentation workflows are still scoped to the `spec/` corpus.
- Claude automation now uses a single authentication approach based on
  `CLAUDE_CODE_OAUTH_TOKEN` rather than maintaining two competing action
  families.
- Release drafting prepares notes for future tags and published releases, but it
  does not publish releases automatically.
