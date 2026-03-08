# Research: Phase 9.1 — Security Hardening

**Date**: 2026-03-07
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

Phase 9.1 is entirely driven by the research corpus. The 7-round research phase (2,000+ papers)
identified critical security gaps that the 5-crate audit confirmed exist in the current
implementation. The authoritative security research synthesis is at
`docs/research-output/consolidated/04-security-and-trust.md` (120+ papers synthesized).

## Key Research Findings

### R1: LLMs Cannot Enforce Security (Master Finding #6)

**Evidence**: GPT-4.1 achieves F1=0.27 on hard RBAC tests. Nearly all frontier agents violate
policies within 10-100 queries (ART benchmark). Security must be infrastructure-level,
deterministic, and Rust-enforced.

**Action**: All Phase 9.1 defenses operate on different substrate than attackers — Rust code
enforcing rules, not LLM prompts requesting compliance.

### R2: Inter-Agent Hijacking at 97% ASR (Master Finding #6)

**Source**: Trail of Bits (2025), COLM 2025 paper.

**Evidence**: A GPT-4-powered orchestrator ran arbitrary malicious code 97% of the time when
given a malicious prompt file. Current agent frameworks lack innate mutual authentication.

**Action**: `MessageSigner` with HMAC-SHA256 per-message signing and nonce-based replay prevention.

### R3: AgentSandbox Reduces ASR 13x (Master Finding #6)

**Source**: Persistent/ephemeral separation research.

**Evidence**: ASR drops from 58.8% to 4.34% with persistent/ephemeral agent separation and I/O
firewall at agent boundaries.

**Action**: NATS account-level isolation between persistent and ephemeral agents. I/O firewall
with quarantine actors at boundaries.

### R4: Infectious Jailbreaks Propagate Exponentially (Master Finding #7)

**Source**: Agent Smith vulnerability, ResearchGate publication/380897242.

**Evidence**: A single adversarial input in shared memory exponentially compromises the entire
agent network. Each infected agent becomes a vector for infecting others.

**Action**: Quarantine actors for all cross-boundary data transfers. Never pass raw persistence
retrievals into agent context without sanitization.

### R5: NATS Auth Callouts for Dynamic Capability Scoping

**Source**: NATS 2.10+ Auth Callout protocol.

**Evidence**: Static JWTs cannot adapt to changing trust conditions. Auth Callouts enable
per-connection dynamic capability scoping.

**Action**: Auth Callout service generating dynamic JWTs from behavioral trust profiles.

### R6: CVE-2025-30215 — JetStream Admin API Flaw

**Source**: NATS security advisory.

**Evidence**: Wildcard `>` and `$JS.>` permissions enable cross-account JetStream stream
destruction. Fixed in NATS v2.11.1.

**Action**: Pin NATS server >= v2.11.1 in deploy artifacts. Audit all agent permissions for
wildcards.

### R7: Schema Validation at Rust Speed

**Source**: consolidated/04-security-and-trust.md.

**Evidence**: Rust `jsonschema` validates at 645x speed of legacy validators. Sub-millisecond
validation overhead is negligible.

**Action**: `StateValidator` trait with pluggable schema validation at persistence boundaries.

## Design Decisions

### D1: HMAC-SHA256 (Symmetric) for Internal Messages

**Decision**: Use symmetric HMAC-SHA256, not RSA/EdDSA (asymmetric).

**Rationale**: Inter-agent messages are high-frequency, low-latency. HMAC-SHA256 is orders of
magnitude faster than asymmetric signing. All agents share the same trust domain (the framework),
so symmetric keys are appropriate. Key distribution uses the existing NATS credential system.

### D2: Auth Callout as Separate Service

**Decision**: Run the Auth Callout handler as a separate Rust service, not embedded in the main
binary.

**Rationale**: Operational independence — the auth service can be updated, restarted, or scaled
independently. Limits blast radius if the main service is compromised. Follows NATS Auth Callout
protocol which expects a service listening on a NATS subject.

### D3: StateValidator as Pluggable Trait

**Decision**: Define `StateValidator` as a trait, not a concrete implementation.

**Rationale**: Schema implementations will evolve — JSON Schema today, potentially ML-based
detection later (Phase 12+). The trait interface is stable; implementations are replaceable.

### D4: NATS Accounts for Sandbox Isolation

**Decision**: Use NATS accounts (not just subject ACLs) for persistent/ephemeral agent isolation.

**Rationale**: NATS accounts provide true credential-level isolation. Agents in different accounts
cannot see each other's subjects by default. Subject ACLs within a single account are insufficient
— a compromised agent could potentially bypass ACLs.

### D5: Delegation Chain Validated, Not Removed

**Decision**: Validate and propagate `AgentClaims.delegation_chain` rather than removing it.

**Rationale**: The delegation chain maps to Macaroon attenuation patterns planned for Phase 10.
Removing it now would require re-adding it later. Instead, wire up validation (non-empty entries,
max depth, no cycles) and propagation across agent boundaries.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | Primary security research synthesis (120+ papers) |
| `docs/research-output/consolidated/00-MASTER-FINDINGS.md` | Strategic ranking — findings #6 and #7 |
| `docs/research-output/research/targeted-inter-agent-security-R6.md` | Deep dive on inter-agent attacks and defenses |
| `docs/research-output/research/targeted-capability-security-sandboxing-R4.md` | Capability security and sandboxing patterns |
| `spec/data-management/agent-orchestration.md` | CRITICAL GAP flag on MessageEnvelope security |
| `spec/security/` | Existing security architecture to extend |
| `crates/mister-smith-security/src/` | Current implementation to audit and extend |

## Explicitly Deferred

| Finding | Phase | Reason |
| ------- | ----- | ------ |
| Macaroon capability delegation | 10 | Extends delegation_chain foundation built in 9.1 |
| eBPF agent observability (AgentSight) | 12+ | Independent infrastructure |
| Progent DSL for capability policies | 12+ | Depends on mature capability model |
| ML-based behavioral anomaly detection | 12+ | Requires runtime data collection |
| Distributed backdoor correlation monitoring | 14+ | Requires cross-agent behavioral analysis |
