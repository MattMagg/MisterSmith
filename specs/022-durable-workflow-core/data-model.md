# Data Model: Durable Workflow Core

## Notes

- This model is the active data-model authority for packet `022`.
- Entity names and required relationships are intentionally more stable than exact field lists.
- Packet `022` now freezes the first-slice event shape, lifecycle verbs, effect-boundary
  placement, and compaction posture.

## Entities

### WorkflowHistoryEvent

**Purpose**: One accepted durable record of workflow progress or lifecycle change.

**Required attributes**:

- workflow identity
- event identity
- replay position
- event type
- ordering or causality reference
- recorded time
- actor or source reference when relevant
- payload with the minimum durable state needed for replay
- optional parent or lineage reference
- optional effect-boundary reference
- optional lifecycle-decision reference
- optional compaction reference

**Relationships**:

- many `WorkflowHistoryEvent` records build one `WorkflowProjection`
- one event may refer to one `EffectBoundaryRecord`
- one event may record one `LifecycleDecision`

### WorkflowProjection

**Purpose**: The current durable state rebuilt from accepted history.

**Required attributes**:

- workflow identity
- current durable lifecycle state
- latest accepted history position
- current branch and node summary
- current session continuity link when present
- current compaction or rollup reference when present

**Relationships**:

- derived from many `WorkflowHistoryEvent` records
- exposes state to task, session, and autonomy surfaces

### EffectBoundaryRecord

**Purpose**: Durable tracking for an external side effect.

**Required attributes**:

- workflow identity
- effect boundary identity
- intent state
- outcome state
- idempotency or dedup reference
- operator-visible notes or reason when outcome is unknown
- recorded time

**Relationships**:

- may be referenced by one or more `WorkflowHistoryEvent` records
- contributes to one `WorkflowProjection`

### LifecycleCommand

**Purpose**: One request to change durable workflow lifecycle.

**Required attributes**:

- workflow identity
- command identity
- verb
- requested by
- requested time
- optional reason or operator note

**Supported verbs in packet `022`**:

- `pause`
- `resume`
- `cancel`
- `terminate`

**Deferred verbs**:

- `reset/rewind`

### LifecycleDecision

**Purpose**: Durable accepted result of a lifecycle command.

**Required attributes**:

- command identity
- accepted outcome
- resulting durable lifecycle state
- decision time
- optional no-op or deferred reason

### HistoryCompactionRecord

**Purpose**: Durable lineage that lets replay stay bounded while keeping recovery explainable.

**Required attributes**:

- workflow identity
- compaction identity
- source history range
- replacement or rollup reference
- resulting replay start point
- preserved lineage note

## Candidate State Groups

These are state groups the packet must freeze. Exact enum names can be finalized in the first
implementation slice without reopening packet scope.

### Workflow lifecycle group

- active or running
- paused or suspended
- cancelling
- cancelled
- terminated
- completed
- failed
- reset or rewind posture is deferred in the first slice

### Effect-boundary group

- intent recorded
- completion unknown
- completed
- failed
- compensated or intentionally not retried remains out of first-slice scope

### Compaction group

- not compacted
- compacted with replay pointer
- compacted with preserved lineage note

## Invariants

- Replaying the same accepted workflow history must rebuild the same projection.
- Repeating the same lifecycle command must not create contradictory durable states.
- Repeating a completed effect boundary must not create a duplicate operator-visible outcome.
- Compaction must not erase the lineage needed to explain the current durable state.
- Session continuity links must survive any durable workflow change packet `022` introduces.
