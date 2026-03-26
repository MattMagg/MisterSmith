# Quickstart: Live Runtime Proof Smoke Harness

## Deterministic Validation

```bash
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
```

## Live Run

```bash
python3 scripts/live_runtime_proof_smoke.py
ls -1dt docs/plans/artifacts/live-runtime-proof-smoke/* | head -n 1
```

## Targeted Areas

- `scripts/live_runtime_proof_smoke.py`
- `scripts/tests/test_live_runtime_proof_smoke.py`
- `docs/plans/artifacts/live-runtime-proof-smoke/`
- state docs only if the harness changes repo-truth claims
