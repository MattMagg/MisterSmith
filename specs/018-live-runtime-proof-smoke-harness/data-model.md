# Data Model: Live Runtime Proof Smoke Harness

## Inputs

- repo root and output artifact directory
- local Docker-backed `postgres` and `nats` prerequisites
- runtime base URL and proof task payload
- terminal task result and autonomy-status JSON responses

## Invariants

- the harness must prove current shipped behavior, not a synthetic mock flow
- artifact output must include enough identifiers and JSON evidence to audit the run later
- NATS/JetStream verification must use a truthful surface supported by the local stack

## Outputs

- runtime health and startup evidence
- task submission and terminal task-status/task-result evidence
- autonomy-status evidence for the completed workflow
- a small summary artifact describing what was directly proved
