# Mister Smith Pre-Packet Rube Tavily Research Handoff

You are working in the Mister Smith repository at `<repo_root>`.

Your mission in this session is to use **Rube MCP with the Tavily app** to do a full external-doc
 and protocol refresh for the pre-spec packet-prep dossier set under `<packet_prep_root>`.

You are not writing specs. You are not implementing code. You are not brainstorming new packets.

You are validating and improving the **documentation layer that should exist before future SpecKit
packet authoring**.

## Objective

By the end of this session, you must have:

1. verified that the Tavily path through Rube MCP is active and used it for the external research
2. audited `docs/packet-prep/README.md` and all seven packet dossiers `022` through `028`
3. built a source inventory for every external protocol, vendor doc family, standard, or
   comparator surface those docs mention or clearly rely on
4. determined the most current stable versions or revisions of those families from official sources
   where possible
5. separated true open protocols from vendor APIs, comparator docs, and product-specific feature
   pages
6. updated the packet-prep docs if the research justifies corrections or stronger grounding
7. stopped before spec writing, code implementation, or architecture expansion beyond the
   documentation layer

## Research Boundary

This is a research-and-doc-grounding session only.

You must not:

- create or edit anything under `specs/`
- implement runtime or application code
- widen packet scope because a protocol family is interesting
- treat product feature pages as protocol truth unless the official docs clearly define them as
  such
- rewrite `docs/direction.md` or `docs/current-state.md` unless a direct contradiction forces a
  narrow correction

## Authority Order

Use this exact authority stack:

1. `<direction_doc>`
2. `<current_state_doc>`
3. `docs/packet-prep/README.md` and packet dossiers `022` through `028`
4. `docs/research-output/consolidated/`
5. `docs/research-output/analysis/`
6. official docs and primary sources discovered through Rube MCP plus Tavily
7. broader outside research only if the official sources still do not answer the question

Rules:

- `docs/current-state.md` wins for what is actually live or landed now
- `docs/direction.md` wins for overall direction and staging
- packet-prep docs are pre-spec dossiers, not packet-implementation truth
- external docs should sharpen or correct the packet-prep docs, not override repo truth

## Start Sequence

Before you do external research, read these local sources in order:

1. `<direction_doc>`
2. `<current_state_doc>`
3. `<packet_prep_root>/README.md`
4. all seven packet dossiers under `<packet_prep_root>/`
5. the supporting research-output docs already cited by those dossiers where needed

During this pass, build an inventory of every external family the packet-prep docs mention or
clearly depend on. That inventory will likely include families such as:

- MCP
- A2A
- OpenAI Responses, streaming, and function calling
- OpenTelemetry and W3C Trace Context
- Temporal and Azure Durable Functions
- NATS and JetStream
- SPIFFE
- JSON Schema
- comparator framework docs already used in the dossiers

Do not assume every family in that inventory is an open protocol. Some may be:

- open protocols
- official vendor APIs or specifications
- comparator framework docs
- product-specific feature or config models

Your job is to classify them honestly.

## Required Rube Plus Tavily Workflow

Use **Rube MCP** as the external research path.

1. Discover Tavily tools with `RUBE_SEARCH_TOOLS`.
2. If `tavily_mcp` is not active, use `RUBE_MANAGE_CONNECTIONS` and wait for an active
   connection before executing Tavily tools.
3. Use Tavily search for targeted discovery.
4. Use Tavily extract on official pages when you need exact version, revision, protocol, or
   wording details.
5. Use Tavily research only when a topic needs cross-source synthesis beyond direct extraction.

Tool discipline:

- validate `response.data.status` and tool-specific errors, not only top-level success flags
- prefer official domains and official protocol sites
- use targeted protocol- or vendor-specific queries
- do **not** waste time on generic one-word searches

This means:

- do not search for `OpenAI`
- do search for the exact family or question you are validating
- do extract the official page when a version, revision, or protocol claim matters

## Research Tasks

For the packet-prep doc set, determine:

1. what external families are currently cited
2. whether the cited sources are official, stable, and still current
3. whether the versions or revisions used in the docs are:
   - current
   - stale
   - mixed
   - or intentionally conservative
4. whether each family is functioning in the dossier as:
   - protocol truth
   - vendor/API truth
   - comparator guidance
   - or inspiration-only research
5. whether the packet-prep docs currently imply:
   - compatibility
   - hardening
   - normalization
   - comparator-only use
6. what corrections or strengthening the docs need

Be especially careful about:

- `latest` versus pinned versions
- mixed-version protocol references
- protocol claims that are really product-feature claims
- compatibility claims that are implied too loosely

For Anthropic-adjacent surfaces, be explicit:

- if something is MCP, say that
- if something is a Claude Code subagent/config feature, say that
- if there is no official open protocol for a claimed category such as "skills" or "plugins",
  say that plainly instead of forcing it into the same bucket as MCP or A2A

## Edit Scope

If the research justifies corrections, update only the packet-prep layer.

Allowed edits:

- `<packet_prep_root>/README.md`
- `<packet_prep_root>/022-028`
- one small supporting source map under `<packet_prep_root>/` only if it materially improves
  cold-start usability

Do not add broad new documentation outside this scope unless a direct contradiction forces it.

## Expected Deliverable

Leave the repo with:

- packet-prep docs updated only where the evidence justifies it
- stronger official-doc grounding
- corrected version or revision references
- clearer protocol versus feature classification
- no spec-writing or implementation drift

## Output

End the session with a concise markdown report containing:

1. **Executive Assessment**
   - what changed at a high level
2. **Source And Version Matrix**
   - each important external family
   - official source
   - most current stable version or revision
   - how the packet-prep docs should treat it
3. **Packet-Prep Corrections**
   - which dossier docs changed and why
4. **Protocol Reality Check**
   - what is truly protocol-level versus vendor-feature-level
5. **Unresolved Questions**
   - what still could not be firmly established
6. **Validation**
   - commands and checks run

## Anti-Patterns

Do not:

- use generic searches and call the results sufficient
- treat unofficial summaries as stronger than official docs
- confuse comparator framework docs with normative Mister Smith inputs
- silently mix stale and current protocol versions
- invent protocol categories because a vendor has a feature with a similar name
- drift into spec authoring, packet freezing, or implementation

## Verification Checklist

Before finishing, verify that you:

- used Rube MCP with Tavily for the external research path
- inspected the full packet-prep doc set
- built a source inventory of the external families the docs rely on
- validated current versions or revisions from official sources where possible
- explicitly distinguished protocol, vendor/API, comparator doc, and product feature
- updated docs only where the evidence justified it
- kept edits bounded to the packet-prep layer
