# Packet Authoring Checklist

Status: Pre-spec helper for future SpecKit packet writing

Use this before freezing any packet from `docs/packet-prep/`.

## 1. Reconfirm Current Truth

- reread `docs/direction.md`
- reread `docs/current-state.md`
- confirm the packet itself is still `planned-only`
- confirm no newer post-packet-021 bounded phase already makes the dossier stale

## 2. Reconfirm Dependency Gates

- `022`: confirm no later packet already froze durable workflow semantics elsewhere
- `023`: confirm `022` still owns lifecycle and durable-identifier language
- `024`: confirm boundary hardening is still separate from later interop protocol design
- `025`: confirm `023` still owns proof-boundary and run-trace semantics
- `026`: confirm packets `022` through `025` are frozen enough to reuse
- `027`: confirm packets `022`, `023`, and `024` are frozen enough to reuse
- `028`: confirm earlier packets, especially `027`, proved a stable seam worth coordinating

## 3. Reconfirm Proof Status

- list what is `live-default`
- list what is `landed-not-default`
- list what is `deterministic-only`
- list what is still only direction or backlog
- cite the exact closure notes, tests, or artifact paths behind every non-live claim

## 4. Reconfirm Official Sources

- open the exact official docs linked in the dossier
- keep stable and version-pinned protocol sources consistent inside one packet
- do not mix stable A2A pages with `dev` pages
- if packet `027` still owns the interop freeze, keep MCP on one pinned revision and keep A2A on
  the dossier's pinned `v0.3.0` baseline unless the packet explicitly records an upgrade
- do not replace Responses API sources with older OpenAI API families
- treat comparator framework docs as comparator input, not as the Mister Smith contract

## 5. Lock Repo Anchors

- name the concrete files, structs, functions, tests, proof notes, and artifact directories the
  packet will rely on
- prefer exact runtime seams over broad crate-level references
- if a claim matters and the dossier does not point to the exact repo seam, add that seam before
  packet writing starts

## 6. Freeze Boundaries Before Spec Writing

- restate the packet scope in one sentence
- restate the packet non-goals in one short list
- note which open questions are still real and which are already constrained
- if a truth-status or dependency claim changed during this pass, update the dossier before writing
  the packet spec
