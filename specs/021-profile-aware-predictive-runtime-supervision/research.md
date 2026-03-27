# Research Notes: Profile-Aware Predictive Runtime Supervision

## Current repo truth

- packet `020` already landed verifier-gated repair lineage and operator-visible
  orchestration-quality evidence on the runtime-backed task path
- the March 27 runtime-planning simplification pass proved the smallest-workflow baseline and
  moved supported description-only repair telemetry onto an explicit runtime-owned record
- the agents layer already contains `ProfileAssessment`, `Guard`, `InterventionEngine`, stream
  monitoring, and operator-visible autonomy events for profile snapshots, guard decisions, and
  interventions
- the supported runtime ingress still seeds supervision with `GuardTarget::Provider(...)` and the
  operator console run detail does not yet treat supervisory evidence as a first-class summary

## Research signals that matter here

### Predictive supervision and profile fingerprints are the strongest next frontier seam

- `docs/research-output/consolidated/00-MASTER-FINDINGS.md` ranks predictive supervision as one
  of the top category-defining advantages beyond current agent frameworks
- `docs/research-output/consolidated/03-supervision-and-resilience.md` converges on a layered
  Guard/Advisor architecture: live telemetry, bounded intervention budgets, fingerprint-backed
  advice, and local recovery before global restart
- `docs/research-output/research/targeted-predictive-supervision-R6.md` gives the most concrete
  blueprint for Mister Smith: AWorld-style performance fingerprints, MetaOrch-style targeted
  intervention selection, and bounded JetStream KV storage for profile state

### Step and stream signals are already good enough to justify a bounded packet

- `docs/research-output/consolidated/06-streaming-architecture.md` shows that typed stream events
  and streaming content monitors can detect degradation early enough to guide local intervention
- the repo already ships `StreamMonitor`, `SemanticSignal`, and packet-020 repair lineage, so the
  next gap is not theoretical viability; it is supported-ingress integration

### Why not the other frontier candidates yet

- packet `019` default-runtime activation is real follow-up work, but it is less frontier and
  less strategically differentiating than predictive supervision
- `docs/plans/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze.md` explicitly says adaptive
  topology remains dormant until new live evidence appears, so reopening topology now would be
  dishonest
- `docs/research-output/consolidated/05-coordination-and-state.md` keeps CRDT coordination,
  MPST, and event-triggered consensus as important later frontier work, but they require a wider
  coordination packet than this runtime-bound slice
- `docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md` reinforces the
  strategic aim: exceed framework norms through runtime leverage, not by copying market-default
  abstractions

## Bounded conclusion

The next legitimate packet is not another routing tweak and not a speculative coordination rewrite.
It is profile-aware predictive runtime supervision on the supported ingress: convert the already
landed Guard/Profile/Intervention substrate into a first-class runtime contract, add bounded
performance fingerprints grounded in replayable evidence, and make the resulting supervisory
evidence operator-visible.
