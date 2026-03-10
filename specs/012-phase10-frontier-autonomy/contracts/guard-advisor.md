# Contract: Guard / Advisor

## Overview

The Guard / Advisor contract defines the predictive supervision layer that augments existing OTP
restart semantics. It classifies failures, consumes step-level degradation signals, and applies
targeted interventions before full graph restarts.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | Failure taxonomy, Guard/Advisor layer, checkpoint-aware recovery |
| `docs/research-output/consolidated/06-streaming-architecture.md` | Step-boundary and stream-monitor signals |
| `spec/core-architecture/supervision-trees.md` | Restart policies and failure-isolation boundaries |

## Public API

```rust
pub trait GuardAdvisor: Send + Sync {
    async fn observe(&self, target: GuardTarget) -> Result<ProfileSnapshot, GuardError>;
    async fn evaluate(
        &self,
        profile: &ProfileSnapshot,
        stream_signals: &[SemanticSignal],
        checkpoints: &[BranchCheckpoint],
    ) -> Result<GuardDecision, GuardError>;
    async fn apply(&self, decision: &GuardDecision) -> Result<InterventionRecord, GuardError>;
}
```

## Failure Classes

```text
Transient   — retries/failover may help
Structural  — invalid request/auth/config; fail fast
Streaming   — stream dropped or stalled; resume or failover
Semantic    — low-quality, repetitive, or degraded reasoning; targeted intervention
```

## Intervention Types

```text
Retry
Failover
ContextRefresh
BranchIsolation
Reassignment
Escalation
Abort
```

## Behavioral Requirements

1. The Guard layer MUST classify failures before choosing an intervention.
2. Step-level or stream-level degradation signals MUST be usable as Guard evidence.
3. Targeted interventions MUST prefer branch-local action before graph-wide restart.
4. Every applied intervention MUST produce an operator-visible `InterventionRecord`.
5. Hard process crashes still defer to existing OTP supervision behavior.

## Validation Requirements

- Transient failure does not trigger full restart by default.
- Structural failure is not retried indefinitely.
- Semantic degradation can trigger context refresh or isolation.
- Applied intervention is visible through the operator state surface.
