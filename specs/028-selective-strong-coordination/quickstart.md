# Quickstart: Selective Strong Coordination

## Step 1: Revalidate before any code work

Before any implementation starts, reread the current repo truth and upstream packet state:

```bash
sed -n '1,260p' docs/direction.md
sed -n '1,320p' docs/current-state.md
sed -n '1,260p' docs/packet-prep/027-capability-discovery-and-interoperability.md
sed -n '1,260p' docs/packet-prep/028-selective-strong-coordination.md
sed -n '1,260p' specs/022-durable-workflow-core/spec.md
sed -n '1,260p' specs/023-runtime-truth-and-run-trace/spec.md
```

Then confirm the real state of packets `024` and `027` from the current repo before touching code.

## Step 2: Refresh the scaffold if repo truth moved

If upstream packet wording, dependency state, or live-proof boundaries changed, refresh:

- `spec.md`
- `plan.md`
- `research.md`
- `data-model.md`
- `contracts/selective-strong-coordination-contract.md`
- `tasks.md`
- `analyze.md`

## Step 3: Validate the scaffold packet itself

Run the narrowest honest checks for the scaffold packet:

```bash
git diff --check
npx markdownlint-cli2 \"specs/028-selective-strong-coordination/**/*.md\" --config .markdownlint.json
```

## Step 4: Re-run the planning loop if needed

If the upstream dependency state changed enough to move the packet boundary, rerun:

```bash
SPECIFY_FEATURE=028-selective-strong-coordination ./.specify/scripts/bash/check-prerequisites.sh --json --paths-only
SPECIFY_FEATURE=028-selective-strong-coordination ./.specify/scripts/bash/setup-plan.sh --json
```

Then refresh the packet artifacts before starting implementation work.

## Proof expectation

This scaffold packet earns packet-proof only, not runtime proof.

It proves that:

- the packet scope is bounded
- the taxonomy is explicit
- the coordination choice rule is explicit
- the first reusable primitive is explicit
- the revalidation gate is explicit

It does **not** prove that:

- stronger coordination is already live on the default runtime path
- packet `027` already froze a stable protocol seam
- packet `028` is implementation-ready without revision
