# Data Model: Profile-Aware Predictive Runtime Supervision

## Supervisory profile entities

### `ProfileFingerprint`

- `fingerprint_id`: stable identifier for the persisted fingerprint
- `target_kind`: supported runtime scope the fingerprint describes, such as `planner`,
  `coordinator`, `executor`, or `provider`
- `target_selector`: stable selector for the profiled runtime target or target class
- `source_refs`: replayable run, checkpoint, or fixture references used to build the fingerprint
- `summary_payload`: structured supervisory summary only; no duplicated raw transcript bodies
- `dominant_failure_modes`: ordered list of recurring failure tendencies or MAST-aligned labels
- `preferred_interventions`: ordered list of existing `InterventionType` values the Guard should
  prefer when live evidence matches
- `confidence`: bounded confidence score for using the fingerprint as advisory context
- `expires_at`: timestamp after which the fingerprint is no longer trusted automatically
- `updated_at`: last refresh timestamp

### `ProfileSnapshot`

- existing runtime-facing health snapshot derived from live stream or step signals
- may include `fingerprint_ref` when a current fingerprint contributed to interpretation
- remains the live evidence surface; fingerprints reinforce it but do not replace it

### `GuardDecision`

- existing typed supervisory decision
- evidence must be able to explain whether the decision came from `live_signals_only`,
  `fingerprint_reinforced`, or `conservative_fallback`
- target scope should prefer branch or node context when available

### `InterventionRecord`

- existing applied recovery record
- must stay aligned with the `GuardDecision` that caused it
- before/after state should remain local to the chosen runtime scope whenever recovery is local

## Operator evidence projection

### `SupervisionEvidenceView`

- `latest_fingerprint_ref`: optional current fingerprint identifier
- `latest_profile`: latest `ProfileSnapshot`
- `latest_guard_decision`: latest `GuardDecision`
- `latest_intervention`: latest `InterventionRecord`
- `decision_basis`: human-readable explanation of how live signals and fingerprint context were
  combined
- `proof_boundary`: note clarifying whether the evidence is deterministic-only or backed by a live
  runtime rerun

## Invariants

- fingerprints are advisory context, not the sole authority for intervention
- fingerprints store structured summaries and references only; raw transcripts remain in the
  existing audit and replay lanes
- live runtime signals override stale or contradicted fingerprints
- provider-scoped supervision is fallback only when no branch or node target exists yet
- packet `020` verifier and repair lineage must remain consistent with packet `021` supervisory
  lineage in operator-facing projections
- local intervention remains the default before graph-wide restart when evidence supports it
