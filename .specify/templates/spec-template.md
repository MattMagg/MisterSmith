# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`
**Created**: [DATE]
**Status**: Draft
**Input**: [repo-grounded inputs, notes, code paths, and user directive]

## Current Truth & Scope

Document the current repo truth before proposing new work.

Current repo truth already includes:

- [landed capability or artifact]
- [landed capability or artifact]

The remaining gap is narrower than a broad new program:

- [unfinished gap or bounded deficiency]
- [unfinished gap or bounded deficiency]

This packet or feature therefore freezes one bounded slice:

1. [bounded deliverable]
2. [bounded deliverable]
3. [bounded deliverable]

This is not:

- [explicit non-goal]
- [explicit non-goal]
- [explicit scope exclusion]

## User Scenarios & Testing

Use independently testable stories. For Mister Smith packets, prefer a small number of bounded
stories over a long backlog of loosely related asks.

### User Story 1 - [Brief Title] (Priority: P1)

[Describe the primary operator or developer journey in plain language]

**Independent Test**: [Describe how this story can be validated on its own]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

### User Story 2 - [Brief Title] (Priority: P1 or P2)

[Describe the next bounded journey]

**Independent Test**: [Describe how this story can be validated on its own]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

### User Story 3 - [Brief Title] (Priority: P2 or P3)

[Describe the proof, inspection, or follow-on journey if needed]

**Independent Test**: [Describe how this story can be validated on its own]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

## Edge Cases

- [boundary condition]
- [failure mode]
- [proof or validation limitation that must stay explicit]

## Requirements

### Functional Requirements

- **FR-001**: System MUST [bounded capability]
- **FR-002**: System MUST [bounded capability]
- **FR-003**: System MUST preserve [existing truth or invariant]
- **FR-004**: System MUST fail explicitly when [invalid configuration or boundary breach]
- **FR-005**: System MUST keep the write set bounded to [precise surface]
- **FR-006**: System MUST NOT widen into [explicitly deferred program]
- **FR-007**: System MUST record deterministic versus live-proof boundaries honestly when both
  exist

### Key Entities

- **[Entity 1]**: [what it represents and why it matters]
- **[Entity 2]**: [what it represents and why it matters]

## Success Criteria

- **SC-001**: [bounded behavior works while preserving current fallback or invariant]
- **SC-002**: [new path or surface is observable without raw log archaeology]
- **SC-003**: [targeted validation proves the scoped behavior]
- **SC-004**: [documentation or proof claims remain honest and bounded]
