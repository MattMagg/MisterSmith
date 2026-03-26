# Feature Specification: Live Runtime Proof Smoke Harness

## Summary

Add one repo-owned smoke harness for the current live runtime proof path so operators can rerun the
default provider-backed proof without rebuilding the manual March 19 workflow by hand.

## Problem

The repo has a documented manual live proof path, but no repeatable script that:

1. verifies the local stack honestly
2. runs the current runtime proof path end to end
3. captures proof artifacts consistently

That leaves the current proof baseline harder to refresh and easier to run inconsistently.

## Goals

- codify the current live proof path into a repo-owned smoke harness
- assert the key runtime and autonomy proof markers from the current default path
- make NATS/JetStream verification honest without relying on a flaky `/healthz` probe

## Non-Goals

- alternate-provider proof
- session-route proof expansion
- new runtime features
- budget-backed control-loop work

## Acceptance Criteria

1. A repo-owned script can run the default live proof path end to end from a clean local repo
   checkout with the expected prerequisites.
2. The harness captures task and autonomy artifacts and asserts the expected `runtime_execution_mode`
   and `execution_boundary` markers.
3. The NATS/JetStream verification step uses a truthful surface supported by the local stack.
4. Script tests pass, and any touched Rust code still builds/lints/tests honestly.
