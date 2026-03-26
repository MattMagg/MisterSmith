# Research: Live Runtime Proof Smoke Harness

## Source Notes

- the March 19 live run trace note already provides the exact manual command flow and artifact set
- `deploy/docker-compose.yml` shows the local stack and supported health surfaces
- no external documentation is required for the first implementation pass

## Implication

The next implementation step should prefer a script that lifts the proven manual flow directly,
with helper functions and assertions that can be tested deterministically.
