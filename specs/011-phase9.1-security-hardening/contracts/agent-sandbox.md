# Contract: Agent Sandbox

## Overview

The AgentSandbox contract defines persistent/ephemeral agent separation using NATS account-level
isolation, I/O firewall enforcement, and quarantine actors for cross-boundary data transfers.
This architecture reduces attack success rate from 58.8% to 4.34% (13x improvement).

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | AgentSandbox pattern, COWPOX defense, infectious jailbreak vectors |
| `spec/data-management/agent-orchestration.md` | Agent lifecycle and orchestration boundaries |
| `spec/security/` | NATS authorization patterns, JWT management |

## Agent Classification

```rust
#[non_exhaustive]
pub enum AgentClass {
    /// Stable identity, durable state, long-lived credentials, full subject access
    Persistent,
    /// Short-lived, isolated, restricted permissions, auto-cleanup on completion
    Ephemeral,
}
```

Classification is determined at agent spawn time based on:
- Agent type (Orchestrator, Planner → Persistent; Worker, Executor → Ephemeral by default)
- Configuration override (any agent can be forced to either class)
- Task characteristics (long-running → Persistent; one-shot → Ephemeral)

## NATS Account Isolation

```text
NATS Server
├── persistent-account
│   ├── persistent agents' subjects
│   ├── durable JetStream consumers
│   └── state storage subjects
├── ephemeral-account
│   ├── ephemeral agents' subjects
│   ├── temporary JetStream consumers
│   └── (no durable state)
└── system-account
    ├── Auth Callout subjects ($SYS.REQ.USER.AUTH)
    └── monitoring subjects
```

**Key property**: Agents in different NATS accounts cannot see each other's subjects by default.
Cross-account communication requires explicit import/export configuration.

## I/O Firewall

```rust
pub struct IOFirewall {
    pub persistent_account: String,
    pub ephemeral_account: String,
    pub allowed_crossings: Vec<CrossingRule>,
}

pub struct CrossingRule {
    pub source_account: String,
    pub target_account: String,
    pub subject_pattern: String,
    pub requires_quarantine: bool,
}

impl IOFirewall {
    /// Check if a cross-boundary communication is permitted
    pub fn check_crossing(
        &self,
        source: &str,
        target: &str,
        subject: &str,
    ) -> Result<CrossingDecision, SecurityError>;
}
```

### Default Crossing Rules

| Source | Target | Subjects | Quarantine |
| ------ | ------ | -------- | ---------- |
| Persistent | Ephemeral | `task.assign.*` | Yes |
| Ephemeral | Persistent | `task.result.*` | Yes |
| Persistent | Persistent | `*` (within account) | No |
| Ephemeral | Ephemeral | `*` (within account) | No |
| Any | System | `health.*`, `metrics.*` | No |

## Quarantine Actor

```rust
pub struct QuarantineActor {
    validator: Arc<dyn StateValidator>,
    audit_logger: Arc<AuditLogger>,
}

impl QuarantineActor {
    /// Inspect data crossing a boundary, return quarantine decision
    pub async fn inspect(
        &self,
        data: &serde_json::Value,
        crossing: &CrossingRule,
    ) -> Result<QuarantineAction, SecurityError>;
}
```

### Quarantine Decision Flow

1. Receive cross-boundary data transfer.
2. Apply `StateValidator` (size check, schema validation, pattern check).
3. Based on validation result:
   - `Clean` → `QuarantineAction::Pass` — forward unchanged.
   - `Sanitized` → `QuarantineAction::Sanitize` — forward modified data.
   - `Suspicious` → `QuarantineAction::Pass` with monitoring flag.
   - `Rejected` → `QuarantineAction::Reject` — block transfer, return error.
4. Log all quarantine decisions to audit.

### Performance Requirements

- Sub-millisecond overhead for clean data (Rust schema validation at 645x speed).
- Quarantine actors are supervised (OTP one-for-one restart on failure).
- Quarantine actors are separate processes/actors from the agents they protect.
- No single point of failure — each boundary crossing has its own quarantine actor.

## Credential Lifecycle

### Persistent Agent

```text
1. Agent spawned → Classify as Persistent
2. Generate SandboxCredentials with persistent-account, long-lived JWT
3. Agent operates with full authorized permissions
4. On restart: rehydrate state, reissue credentials
5. On decommission: revoke credentials, preserve audit trail
```

### Ephemeral Agent

```text
1. Agent spawned → Classify as Ephemeral
2. Generate SandboxCredentials with ephemeral-account, short-lived JWT
3. Agent operates with restricted permissions
4. On completion: auto-cleanup credentials + state + NATS subjects
5. On timeout: force cleanup with audit event
6. On crash: supervision system triggers cleanup
```

## Behavioral Requirements

1. Ephemeral agents MUST NOT access persistent agents' NATS subjects directly.
2. All cross-boundary data MUST route through quarantine actors when `requires_quarantine = true`.
3. Ephemeral agent cleanup MUST be automatic — no orphaned credentials or subjects.
4. Quarantine actors MUST be separate from the agents they protect (blast radius isolation).
5. The I/O firewall MUST be enforced at the NATS level (account isolation), not in application
   code alone.
6. Crossing rules MUST be explicitly configured — deny-by-default for cross-account traffic.

## Validation Requirements

- Ephemeral agent cannot subscribe to persistent agent subjects (NATS account enforcement).
- Cross-boundary data transfer through quarantine actor works for clean data.
- Cross-boundary data transfer is rejected for malicious data.
- Ephemeral agent credentials are cleaned up on completion, timeout, and crash.
- I/O firewall blocks unauthorized crossings.
- Quarantine actor restart (OTP supervision) does not lose in-flight inspections.
