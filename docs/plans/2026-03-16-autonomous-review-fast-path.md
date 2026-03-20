# Autonomous Review Fast Path

Date: March 16, 2026
Status: Implemented in current workflow contracts; use `WORKFLOW.md` and `docs/linear/LINEAR.md` for the live review posture

## Objective

Preserve Symphony's native `Human Review` state name while removing the extra human handoff when
the operator has already delegated review and merge authority to Codex, and reduce avoidable PR
latency from redundant or mis-scoped workflow automation.

## Scope

- `WORKFLOW.md` and `docs/linear/LINEAR.md` contract updates
- Smith lifecycle hint updates in `crates/mister-smith-mcp/src/compatibility.rs`
- GitHub Actions trigger, concurrency, and advisory-review scoping updates
- workflow documentation refresh under `.github/workflows/README.md`
- re-evaluation of the remaining open PRs under the updated contract

## Assumptions

- Symphony's native state machine keeps the `Human Review` and `Merging` state names.
- Explicit operator delegation in the active Codex session is sufficient authority for the agent to
  perform the review step and land a clean PR.
- automated Claude review remains an advisory layer on top of substantive repository validation.

## Constraints

- Do not rename or remove Symphony-native workflow states.
- Do not weaken the substantive merge gate for code changes.
- Prefer narrower workflow triggers and canceled duplicate runs over adding more required checks.

## Non-goals

- Replacing Symphony's state machine
- Rebuilding the full documentation-validation workflow
- Forcing merges that still fail the substantive repository gate

## Milestones

1. Update workflow contracts so `Human Review` can be satisfied by delegated-agent review.
   Validation: targeted markdown readback.
2. Update Smith lifecycle hints to recommend review/merge instead of passive waiting.
   Validation: targeted `mister-smith-mcp` test/build proof.
3. Scope PR automation to the changes that actually need it and cancel superseded runs.
   Validation: YAML syntax checks and workflow diff review.
4. Reassess the open PR set and execute the merge/close actions that are now honest.
   Validation: live GitHub and Smith control-plane snapshots.

## Stop Conditions

- A required workflow name, state name, or external integration contract would need to change.
- The narrower CI trigger would skip a code-changing surface that still needs the substantive gate.
- Smith compatibility changes introduce test regressions or contradict the repo-owned workflow docs.
