# Research: Session-First CLI Shell

## Decision 1: The packet is CLI-only

- **Decision**: Narrow the packet to the terminal shell and explicitly defer GUI parity and
  cross-surface continuity.
- **Rationale**: The user said the GUI mattered only by accident and the current need is the CLI.
  That keeps the packet smaller, reduces parity overhead, and still improves the main product
  entry.
- **Alternatives considered**:
  - keep the shared CLI and GUI packet together
  - split the CLI-only shell into a smaller command-only cleanup packet

## Decision 2: The default CLI entry becomes recent-first

- **Decision**: Treat `mister-smith` with no arguments as the main CLI shell entry, centered on
  recent sessions, resume-last, start-new, warnings, and config.
- **Rationale**: The current runtime-first default is the main product-shape problem. A recent-first
  home makes the terminal flow match the session-first product thesis without requiring broader
  platform changes.
- **Alternatives considered**:
  - keep `run` as the practical default and add a second shell command
  - open directly into a blank composer without recent-session context

## Decision 3: Resume and recent-session browsing stay distinct

- **Decision**: Keep quick resume and broader recent-session browsing as separate CLI behaviors.
- **Rationale**: Resume-last should stay fast, while browsing history should provide enough session
  context to support deliberate reopening decisions.
- **Alternatives considered**:
  - collapse all resume flows into one picker
  - treat resume as only a session-id based command

## Decision 4: Core live controls stay in the CLI session

- **Decision**: Keep model, permissions, config, status, and MCP controls inside the live CLI
  shell through slash commands or another clearly in-session command flow.
- **Rationale**: The session-first shell loses its value if users have to leave the live session
  for separate admin commands whenever they need to steer the session.
- **Alternatives considered**:
  - keep support commands as the only place to adjust these controls
  - move all control changes into config files or pre-launch flags

## Decision 5: Support surfaces stay secondary

- **Decision**: Runtime, doctor, auth, proof, config, and MCP administration remain available, but
  they stay beside the main CLI session flow instead of defining it.
- **Rationale**: These are still real product capabilities, but this packet is about reshaping the
  terminal experience around sessions rather than maintenance-first navigation.
- **Alternatives considered**:
  - pull support surfaces into the startup home as equal first-class paths
  - exclude support surfaces from the product story entirely
