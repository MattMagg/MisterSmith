# Post-Research Analysis Brief: Durable Workflows and Resume Semantics

**Imported report:** `docs/research-output/inbox/deep-research-report (3).md`

## Executive Assessment

The imported research does not change Mister Smith's core direction. It sharpens it.
The strongest takeaway is that Mister Smith should treat durable execution as a first-class
semantic contract built on its own NATS/JetStream and OTP-style substrate, not as a reason to
imitate Temporal or Durable Functions structurally.

The report's most decision-relevant contribution is the explicit split between exactly-once
workflow state transitions and at-least-once side effects. That distinction is stronger and more
actionable than some existing repo-local language around JetStream deduplication and replay. By
contrast, event-sourced replay, checkpoints, and saga-style compensation mostly confirm directions
already present in Mister Smith research and bounded-runtime lineage.

Decision horizon used here:

- Whole-system future direction: define the durable workflow core that can support long-running,
  replayable, resumable execution across Mister Smith.
- Current bounded work: keep the active runtime packet honest without absorbing durable workflow
  scope prematurely.

## Findings That Merit Consideration

### 1. Durable execution should be modeled as a deterministic workflow state machine with event history as the semantic source of truth

**Imported evidence:** Strong. The report draws this from the strongest, most repeated evidence
in the imported set: Temporal, Durable Functions, Netherite, and related formal work all converge
on event history plus deterministic replay as the practical durable execution baseline.

**Repo context:** This largely aligns with existing Mister Smith thinking. Current research
already centers JetStream-backed event sourcing, replay, checkpoints, and resume boundaries, and
recent bounded runtime work already relies on replayable evidence and last-stable-checkpoint
lineage.

**Inference:** This should influence architecture now, but as a semantic baseline for the next
durable-execution seam, not as a reason to reorient the current bounded packet. The right transfer is
"Mister Smith should expose Temporal-grade semantics on NATS/JetStream," not "Mister Smith should
become a clone of Temporal."

**Decision:** Ready to influence architecture now. Do not fold it into the current bounded packet;
use it to frame the next dedicated durable workflow seam.

### 2. JetStream dedup is not enough; exactly-once outcomes require idempotent activities plus outbox/inbox bridging

**Imported evidence:** Strong. The report is explicit that broker-level deduplication reduces
duplicates but does not solve dual-write or side-effect correctness. Its strongest transferable
mechanism is idempotent activity identity plus durable intent tracking.

**Repo context:** Existing research already values `Nats-Msg-Id`, JetStream replay, and
crash-safe side effects, but some repo-local wording still compresses "exactly-once" too
aggressively around broker behavior. The imported report sharpens the boundary: JetStream can
support the transport, but effect correctness still depends on idempotency and durable bridging.

**Inference:** This is the most important genuinely decision-shaping refinement in the imported
report. It should harden Mister Smith's architecture language immediately so future seams do not
overclaim what JetStream alone guarantees.

**Decision:** Ready to influence decisions now. Prototype next. This is work for the next
dedicated durable workflow seam, but it is current architectural guidance.

### 3. Cancellation, termination, pause/resume, and reset/rewind should become explicit runtime lifecycle semantics

**Imported evidence:** Strong on cancel vs terminate and pause/resume; moderate on reset/rewind
as a production requirement rather than a debugging convenience.

**Repo context:** The gathered repo and current-state context already expose checkpoints, repair
lineage, and replayable evidence, but they do not yet present a crisp repo-wide lifecycle
contract for graceful cancel, forceful terminate, resumable pause, and replay-safe reset.

**Inference:** Mister Smith should define these operations as part of its operating-system
contract before making broader claims about durable long-running workflows. This is especially
important if task, session, and autonomy surfaces are meant to converge.

**Decision:** Better suited for design exploration next. It should shape the next architecture
packet, but it does not justify expanding the current bounded packet.

### 4. History growth, replay cost, workflow versioning, and replay-regression gates are mandatory once Mister Smith broadens durable workflow support

**Imported evidence:** Strong. The imported report repeatedly ties durable execution viability to
bounded history growth, snapshot or compaction strategy, and version-safe replay.

**Repo context:** Existing research already values replay and checkpoints, but the imported
report makes the operational consequence sharper: if Mister Smith expands long-lived workflow
semantics without a "continue-as-new" or equivalent compaction/versioning story, replay
correctness will become an unbounded operational liability.

**Inference:** This is not a concern for the current bounded packet, but it is an architectural
requirement for the first serious durable workflow seam.

**Decision:** Better suited for design exploration next, with selective prototyping soon after the
semantic core is frozen.

### 5. Compensation needs more formal structure for parallel branches than "saga exists"

**Imported evidence:** Moderate. The imported report's strongest support here comes from ExoFlow
and Petri-net-style saga work, which are useful but not yet equivalent to production-proven
mainstream workflow practice.

**Repo context:** Mister Smith already has saga-style compensation in its research baseline,
including compensation-aware supervision thinking. What appears missing is a sharper contract for
rollbackable versus irreversible steps, and a deterministic compensation ordering rule once
branches run in parallel.

**Inference:** This matters for Mister Smith because the architecture is explicitly chasing
adaptive, branch-capable orchestration. A compensation story that only works for linear flows is
insufficient.

**Decision:** Prototype or design exploration later. Important, but not ready to drive current
implementation beyond contract definition.

### 6. Transactional epochs, decentralized orchestration, and system-level process snapshots are frontier ideas, not near-term commitments

**Imported evidence:** Mixed. Styx is strategically interesting but high-complexity; Unum and
Pheromone are useful thought experiments for pushing orchestration toward the work; CRIU-style
checkpointing is operationally powerful but weak as a semantic foundation.

**Repo context:** Mister Smith's current supported runtime path prioritizes profile-aware
predictive supervision, not a distributed transaction engine or opaque process snapshot system.
Existing research already gives the repo stronger immediate frontiers than these options.

**Inference:** These ideas are worth monitoring, but they should not pull Mister Smith off the
current path until the semantic durable core is defined and proven.

**Decision:** Worth monitoring, not yet actionable. CRIU-style checkpointing is not worth
pursuing for Mister Smith at this time as a primary durability mechanism.

## Novelty Relative To Mister Smith

Mostly already aligned:

- event-sourced or replayable workflow state
- JetStream-backed audit and checkpoint surfaces
- OTP-style supervision layered over durable runtime state
- saga-style compensation as part of graceful degradation
- replayable evidence as a first-class runtime and proof surface

Genuinely new or materially sharper than the current baseline:

- the explicit separation between broker deduplication and effect correctness
- outbox/inbox plus activity identity as non-optional if Mister Smith wants effectively-once
  outcomes
- a clearer lifecycle contract for cancel, terminate, pause/resume, and reset/rewind
- the operational necessity of history-bounding, workflow versioning, and replay-regression gates
- the need to formalize compensation ordering and rollbackability for parallel execution, not just
  sequential sagas

Direction-changing judgment:

- The imported report does not replace Mister Smith's frontier direction. It narrows and sharpens
  the substrate requirements underneath that direction.
- The report's recommendation to build a "Temporal/Durable-Functions-like core" is correct at the
  semantic level and incomplete at the architectural level. Mister Smith should copy the durable
  semantics, not the product shape.

## Further Research Needed

1. JetStream-native outbox/inbox design for Mister Smith's persistence seams.
The imported report proves the need, but it does not answer how Mister Smith should compose PostgreSQL, JetStream, and current crate boundaries into one crash-safe effect pipeline.

2. Runtime lifecycle contract across task, session, and autonomy surfaces.
The report identifies the right semantics, but it does not map them onto Mister Smith's current HTTP, CLI, and operator-console surfaces.

3. History compaction strategy for JetStream-backed replay.
The report says Mister Smith needs a "continue-as-new" or snapshot analogue, but it does not tell us whether the right transfer is rollup events, snapshot streams, KV pointers, or a hybrid.

4. Compensation taxonomy for Mister Smith tool classes.
The current report is too generic to define which Smith-side effects are retryable, compensable, rollbackable, or irreversible across file, network, provider, and operator-visible actions.

5. Replay-safe versioning and regression testing.
The report establishes the need for replay-regression gates, but not the repo-specific fixture model, version markers, or rollout policy Mister Smith should adopt.

6. Whether any Styx-like transactional guarantees are worth the cost on the Smith substrate.
This should be a targeted follow-up only after the semantic durable core exists; the current report is not enough to justify implementation.

## Bottom Line

Mister Smith should take this imported research seriously as a substrate-clarification report,
not as a reason to change strategic direction. The actionable takeaway is to define a dedicated
durable workflow seam around event-history semantics, idempotent activities, outbox/inbox
bridging, explicit lifecycle operations, and replay/versioning boundaries.

What should not happen is just as important: the current bounded packet should not be widened into
a general durable-engine rewrite, and Mister Smith should not mistake JetStream deduplication for
complete exactly-once semantics. The imported report mostly confirms the frontier path Mister Smith is
already on, but it meaningfully raises the bar for how rigorously the durable execution layer
needs to be specified.
