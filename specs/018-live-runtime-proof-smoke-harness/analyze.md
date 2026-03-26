# Analysis: Live Runtime Proof Smoke Harness

## Authority

- `docs/plans/2026-03-19-live-run-trace-evaluation.md`
- `docs/current-state.md`
- `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md`

## Key Gap

- the repo has a manual live proof note but no repeatable smoke harness
- the old proof path relied on a monitor probe that was not truthful in that environment

## Bounded Approach

- start in `scripts/` with a harness that mirrors the March 19 manual flow closely
- prefer helper functions that can be unit-tested without a live runtime
- keep the slice about proof repeatability, not new runtime behavior
