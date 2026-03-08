# Requirements Checklist: Phase 9.1 — Security Hardening

## Audit Findings Status

| ID | Finding | Severity | Status | Task IDs | Notes |
| -- | ------- | -------- | ------ | -------- | ----- |
| F1 | Inter-agent message authentication absent (97% ASR) | CRITICAL | PLANNED | S001-S007 | HMAC-SHA256 signing + nonce replay prevention |
| F2 | No persistent/ephemeral agent separation (58.8% ASR) | CRITICAL | PLANNED | S017-S021 | NATS account isolation + I/O firewall |
| F3 | No data sanitization between persistence and agents | CRITICAL | PLANNED | S008-S011 | StateValidator trait + schema validation |
| F4 | No NATS Auth Callout for dynamic capability scoping | HIGH | PLANNED | S012-S016 | Auth Callout service + trust profiles |
| F5 | Infectious jailbreak defense absent (exponential propagation) | HIGH | PLANNED | S022-S025 | Quarantine actors + COWPOX pattern |
| F6 | CVE-2025-30215: NATS server version not pinned | HIGH | PLANNED | S026-S027 | Pin >= v2.11.1 + wildcard audit |
| F7 | `AgentClaims.delegation_chain` is dead code | MODERATE | PLANNED | S028-S030 | Validate + propagate (or remove) |

## Research Evidence

| Finding | Evidence Level | Key Citation |
| ------- | -------------- | ------------ |
| F1 | HIGH | Trail of Bits (2025), COLM 2025 — 97% ASR inter-agent hijacking |
| F2 | HIGH | AgentSandbox research — 58.8% to 4.34% ASR (13x improvement) |
| F3 | HIGH | MINJA (Dong 2025, 18 citations) — memory injection without direct access |
| F4 | HIGH | NATS Auth Callout protocol (NATS 2.10+) — dynamic per-request scoping |
| F5 | MODERATE | Agent Smith vulnerability — exponential propagation; COWPOX (ICML 2025 poster) |
| F6 | HIGH | CVE-2025-30215 — JetStream Admin API flaw, cross-account destruction |
| F7 | LOW | Crate audit — `delegation_chain` at `jwt/claims.rs:46` never validated |

## Acceptance Criteria

- [ ] All inter-agent messages carry verifiable HMAC-SHA256 signatures
- [ ] Replay attacks using previously-valid nonces are rejected
- [ ] No raw persistence retrieval reaches agent context without `StateValidator`
- [ ] Auth Callout service dynamically scopes agent permissions
- [ ] Ephemeral agents cannot access persistent agent NATS subjects
- [ ] Cross-boundary data transfers pass through quarantine actors
- [ ] Deploy artifacts enforce NATS >= v2.11.1
- [ ] No wildcard `>` or `$JS.>` permissions in agent configurations
- [ ] `delegation_chain` is validated or removed
- [ ] All security events produce audit log entries

## Crates Affected

| Crate | Changes |
| ----- | ------- |
| `mister-smith-transport` | MessageEnvelope security fields (signature, nonce, capability_token) |
| `mister-smith-security` | MessageSigner, AuthCalloutHandler, StateValidator, sandbox, quarantine |
| `mister-smith-persistence` | StateValidator integration in AgentRepository |
| `mister-smith-agents` | AgentSandbox integration, quarantine actors |
| Deploy artifacts | NATS version pinning, permission audit |
