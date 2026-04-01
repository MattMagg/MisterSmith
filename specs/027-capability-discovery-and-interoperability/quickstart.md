# Quickstart: Packet 027 Scaffold Refresh And Use

## Purpose

Use this packet as a scaffold for future implementation planning. Do not treat it as immediate
implementation authorization.

## Steps

1. Re-read `docs/direction.md` and `docs/current-state.md`.
2. Re-read the completed packet outputs for `022`, `023`, and `024` once they exist.
3. Refresh packet `027` artifacts against those final upstream outputs before writing code.
4. Confirm protocol pins still match:
   - MCP `2025-11-25`
   - A2A `v0.3.0`
5. Confirm the packet still freezes only:
   - normalized capability discovery
   - one A2A lifecycle bridge
   - operator-visible provenance for remote capability use
6. Confirm the packet still defers:
   - generic federation
   - remote execution authorization expansion
   - extra protocols
   - live multi-remote runtime proof
7. Use `tasks.md` only after the refresh gate has been completed.

## Packet Validation Commands

```bash
git diff --check
npx markdownlint-cli2 "specs/027-capability-discovery-and-interoperability/**/*.md" --config .markdownlint.json
```

## Implementation Reopen Checklist

- upstream packets `022`, `023`, and `024` are complete
- packet `027` artifacts were refreshed after those packet completions
- version pins remain unchanged or are explicitly updated everywhere
- the packet still separates discovery metadata from execution permission
- packet `016` is still referenced only for continuity and provenance boundaries
