# MisterSmith GitHub Workflows

GitHub Actions workflows are intentionally disabled in this repository.

The prior Actions-based CI, Claude-assist, documentation, labeling, and release
automation paths were removed so repo progress does not depend on GitHub
Actions billing.

Repository-level AI review configuration now lives in `.coderabbit.yaml`, which
is consumed by the installed CodeRabbit app rather than GitHub Actions.

## Current Posture

- Validation is performed locally and recorded in commits, workpads, and issue
  notes.
- Review can be provided by the installed CodeRabbit app plus human or
  delegated operator review.
- Documentation linting and packet markdown checks remain local or issue-scoped
  validation, not hosted automation.

### Archived workflow

- `mistersmith-ci.yml`: Archived under `.github/deactived-workflows/` as a
  historical documentation-phase workflow. It is no longer active.
