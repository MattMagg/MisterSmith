# Research Notes: Session-First User Shell

## Decision: `session` is the canonical user-facing noun

- **Decision**: The product shell uses `session` as the first-level noun for start, resume,
  browse, and live steering.
- **Rationale**: The repo already has durable session identity, retained turn history, restart and
  resume lineage, and session list or inspect surfaces. Reusing `session` keeps the product aligned
  with current repo truth while avoiding internal vocabulary drift.
- **Alternatives considered**:
  - keep `conversation` as the top-level noun: rejected because it reflects an internal seam more
    than the intended product posture
  - keep `autonomy` or runtime terms first-level: rejected because that makes the product feel
    runtime-first again

## Decision: Startup is recent-first and intentionally narrow

- **Decision**: The startup home shows recent sessions, resume last, start new, warnings, and open
  config. It does not expand into a broad dashboard in this slice.
- **Rationale**: The April 5 primer explicitly centers opening the shell, starting or resuming
  work, browsing recent sessions, and steering the session in place. A recent-first home supports
  that path without widening into admin-console or generic dashboard scope.
- **Alternatives considered**:
  - a broader dashboard with pinned sessions, recent workspaces, and quick-start prompts: deferred
    because it expands the feature boundary before the main shell path is settled
  - a minimal blank composer with no home state: rejected because it weakens resume and recent
    session discoverability

## Decision: CLI and GUI must share one session system and one protocol

- **Decision**: The desktop app and terminal shell use one shared session identity model, one
  shared transcript model, one shared session summary model, and one shared app protocol.
- **Rationale**: The primer makes cross-surface continuity a core user flow. The repo already has
  a stable session record and a current desktop surface, so the next correct move is to define how
  both front ends share the same truth instead of letting them diverge into two products.
- **Alternatives considered**:
  - a desktop-only session cache layered over the CLI: rejected because it creates split-brain
    state
  - CLI-first behavior now and GUI-specific behavior later: rejected because it weakens the core
    promise that a live session can move between front ends without losing state

## Decision: Core live-session controls need cross-surface parity

- **Decision**: Model, permissions, config, status, and MCP controls must be available in both the
  CLI and GUI through in-session surfaces.
- **Rationale**: The requested feature centers on steering a live session in place. The product
  should not force users to leave the session or switch to a maintenance-first surface to change
  core live-session settings.
- **Alternatives considered**:
  - CLI-only slash control with later GUI parity: rejected because it makes the GUI a weaker
    product surface
  - a shared model with intentionally different control coverage: rejected because it invites
    product drift and session confusion

## Decision: Support surfaces stay available but secondary

- **Decision**: Runtime, doctor, auth, proof, config, and MCP administration remain in the
  product, but they sit beside the main session path rather than defining the shell.
- **Rationale**: The requested packet is product-side, not admin-console-side. The shell must stay
  centered on starting, resuming, browsing, and steering work while still exposing support state
  honestly.
- **Alternatives considered**:
  - keep runtime or maintenance commands as the default product identity: rejected because it
    preserves the current upside-down experience
  - hide degraded support state completely: rejected because the user still needs honest warnings

## Bounded conclusion

The correct packet for this lane is not a generic shell rewrite. It is a bounded product packet
that reorders the experience around one shared session system with two front ends, preserves the
current durable session seams, and keeps runtime or admin concerns as support features rather than
the main user path.
