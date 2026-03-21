# Ralph Prompt Freshness Enforcement

Date: March 20, 2026
Status: Completed on March 21, 2026 (`Milestones 1-3` implemented and verified)

## Objective

Make prompt regeneration from the active issue, workpad, or plan packet explicit and verifiable before
`./scripts/ralph run` uses the repo-default `PROMPT.md`.

## Scope

- add a repo-owned prompt freshness assertion step for default Ralph runs
- update the wrapper so default `run` invocations fail fast when `PROMPT.md` is stale or still at the
  checked-in contract form
- add a repo-owned `./scripts/ralph prompt --packet <packet.json>` bridge so Smith packet output can
  regenerate `PROMPT.md` without pretending upstream Ralph ships that subcommand
- document the required prompt metadata block in live workflow surfaces

## Assumptions

- the checked-in `PROMPT.md` remains a contract template, not a committed live prompt
- advanced callers may intentionally use `--prompt` or a non-default `--prompt-file`

## Constraints

- keep Ralph as a loop runner only
- keep the change repo-local; do not depend on upstream Ralph modifications
- avoid auto-generating active prompts inside the wrapper because the live issue/workpad context stays
  outside the wrapper

## Non-Goals

- building a full prompt packet generator
- changing Smith MCP, SpecKit, or issue routing behavior
- persisting extra prompt state outside the prompt file itself

## Milestones

### Milestone 1: Prompt Preparation Helper

- add a repo-owned helper that writes `PROMPT.md` from a prepared prompt input plus explicit
  source-of-record paths
- make the helper emit machine-checkable metadata for `generated-at` and `source` lines
- replace any existing helper-managed metadata block without mutating the prompt body itself

Validation:

- targeted script tests cover metadata rendering, repo-relative source formatting, and metadata
  replacement
- one real helper invocation writes the expected metadata block and prompt body layout

Execution status:

- completed on March 21, 2026 via `scripts/prepare_ralph_prompt.py` and the repo-owned
  `./scripts/ralph prompt --packet <packet.json>` bridge
- verified with `python3 -m unittest scripts.tests.test_prepare_ralph_prompt`
- verified with one real render proof and one stdin failure-path proof for missing `--source`

### Milestone 2: Freshness Contract

- define the active-prompt metadata block (`generated-at` plus one or more source paths)
- document the required structure in `PROMPT.md` and the workflow docs

Validation:

- docs and prompt contract clearly show the required metadata block and required prompt sections

### Milestone 3: Wrapper Enforcement

- add a repo-owned validator for the metadata block and source-file mtimes
- run the validator automatically before default `./scripts/ralph run` invocations

Validation:

- `./scripts/ralph run --dry-run` passes with a freshly generated active prompt
- `./scripts/ralph run --dry-run` fails fast when the prompt lacks the metadata block or a listed
  source becomes newer than the recorded generation time

Execution status:

- completed on March 21, 2026 via `scripts/validate_ralph_prompt.py` plus `scripts/ralph`
- verified with `python3 -m unittest scripts.tests.test_validate_ralph_prompt`
- verified with one real `./scripts/ralph run --dry-run` pass after `scripts/prepare_ralph_prompt.py`
  regenerated `PROMPT.md`
- verified with one stale-source failure path after advancing a listed source file timestamp

## Stop Conditions

- the default repo Ralph entrypoint refuses stale `PROMPT.md`
- the expected active-prompt metadata is documented in live repo guidance
- validation covers one passing dry run and one realistic failure path
