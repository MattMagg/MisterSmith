# Ralph Upstream Bootstrap And Resolution

Date: March 20, 2026
Status: Complete

## Objective

Update Mister Smith to use an upstream `ralph-orchestrator` install from
`https://github.com/mikeyobrien/ralph-orchestrator` and stop depending on whichever `ralph`
binary happens to be first on `PATH`.

## Scope

- add a repo-owned Ralph bootstrap script
- add a repo-owned Ralph wrapper entrypoint
- update live workflow/docs surfaces to use the wrapper
- run the bootstrap flow and record the active Ralph commit/path evidence

## Assumptions

- the requested update target is upstream `main`, not the latest bottled Homebrew stable release
- the existing fork checkout at `~/ralph-orchestrator` is not the install source of truth for
  Mister Smith

## Constraints

- do not overwrite the dirty local fork checkout at `~/ralph-orchestrator`
- keep the change scoped to Ralph bootstrap/usage surfaces
- preserve Ralph as a loop runner only; do not widen repo workflow scope

## Non-Goals

- modifying upstream Ralph source code
- changing Smith MCP routing or SpecKit behavior
- introducing a repo-local vendored Ralph copy

## Milestones

### Milestone 1: Managed Ralph Bootstrap

- add `scripts/bootstrap_ralph.sh`
- install/update a managed Ralph checkout under `~/.local/share/mister-smith/ralph-orchestrator`

Validation:

- `./scripts/bootstrap_ralph.sh`
- verify managed checkout commit and installed binary path

### Milestone 2: Deterministic Repo Entry Point

- add `./scripts/ralph`
- update `WORKFLOW.md`, `README.md`, `CLAUDE.md`, `PROMPT.md`, and `ralph.yml` to use the wrapper

Validation:

- `./scripts/ralph --version`
- confirm live contract docs no longer require bare `ralph run`

## Stop Conditions

- a managed upstream Ralph install exists and records its source commit
- Mister Smith live workflow surfaces point to `./scripts/ralph`
- validation proves the wrapper resolves the updated managed install

## Result

- added `scripts/bootstrap_ralph.sh` to clone/update upstream Ralph into
  `~/.local/share/mister-smith/ralph-orchestrator/source`
- added `./scripts/ralph` as the only supported Mister Smith Ralph entrypoint
- updated live contract docs and workflow surfaces to call `./scripts/ralph`

## Evidence

- managed upstream source commit: `91ebd137f5c8795348e13cfcef1bd75f1cdc4ed7`
- managed install path:
  `~/.local/share/mister-smith/ralph-orchestrator/install/bin/ralph`
- wrapper proof:
  `bash -x ./scripts/ralph --version` executed
  `~/.local/share/mister-smith/ralph-orchestrator/install/bin/ralph --version`
