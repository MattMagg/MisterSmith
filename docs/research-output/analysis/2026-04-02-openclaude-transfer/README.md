# OpenClaude To Mister Smith Transfer Analysis

Date: April 3, 2026
Status: Second-pass review
Source repo: `/Users/macmain/openclaude`
Judged against:

- `/Users/macmain/MisterSmith/docs/current-state.md`
- `/Users/macmain/MisterSmith/docs/direction.md`
- `/Users/macmain/MisterSmith/specs/023-runtime-truth-and-run-trace/`
- `/Users/macmain/MisterSmith/specs/024-agent-boundary-security-hardening/`
- `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/`
- `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/`

## Purpose

This bundle keeps only the OpenClaude ideas that look useful for Mister Smith after a second,
frontier-focused pass.

It does not treat OpenClaude as a product template. It treats it as a source of specific patterns
that may improve Mister Smith's coordination runtime, capability boundary, operator proof, and
execution safety.

## Bottom Line

The strongest OpenClaude transfers for Mister Smith are now clearer than they were in the first
pass:

1. subordinate-runtime event intake inside the main turn loop
2. stable delegated work units with inspectable state and follow-up messaging
3. deterministic parallel tool batches with clear cancellation rules
4. long-lived MCP lifecycle handling with delta refresh, auth-state surfacing, and large-result
   offload
5. stricter discovery-versus-execute separation for local, MCP, and remote capability surfaces
6. surface-specific command and permission gates

The first pass gave too much weight to framework convenience items like general command palettes,
plugin-style capability UX, provider-specific search brokers, and other parity features. Those are
not the best near-term Smith transfers.

## Reading Order

1. `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/01-runtime-and-tooling.md`
2. `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/02-operator-and-ux.md`
3. `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/03-remote-and-delegated-execution.md`
4. `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/04-priority-backlog.md`

## Fit Labels

- `KEEP as-is`: still correct and still bounded well
- `KEEP with update`: keep the idea, but narrow or reframe it
- `SPLIT or DEFER`: useful, but later or in a different packet
- `REMOVE as misfit`: not a strong Smith transfer right now
