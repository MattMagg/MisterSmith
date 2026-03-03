# MisterSmith CLAUDE.md Design Document

## Executive Summary

This document outlines the domain-driven design for MisterSmith-specific system instructions that complement global anti-fabrication directives while addressing the unique challenges of building a multi-agent LLM orchestration framework incrementally.

## 1. Architecture Overview

### 1.1 Design Principles

- **Inheritance Model**: Project instructions extend (never override) global instructions
- **Verification-First**: Every instruction must be verifiable through execution
- **Incremental Enforcement**: Instructions evolve with project maturity
- **Anti-Complexity**: Actively prevent over-engineering at instruction level

### 1.2 Instruction System Architecture

```
┌─────────────────────────────────────┐
│   Global System Instructions        │
│  (Foundation-First Philosophy)      │
└────────────┬────────────────────────┘
             │ extends
┌────────────▼────────────────────────┐
│   MisterSmith CLAUDE.md             │
│  (Project-Specific Constraints)     │
├─────────────────────────────────────┤
│ • Project Context                   │
│ • Development Phases                │
│ • Verification Requirements         │
│ • Domain-Specific Patterns          │
└─────────────────────────────────────┘
```

## 2. Domain-Driven Design

### 2.1 Bounded Contexts

#### Context 1: Project Reality Management

**Purpose**: Maintain accurate project state awareness
**Core Entities**:

- ProjectPhase (Foundation → Single Agent → Multi-Agent → Distributed)
- ImplementationStatus (Specification vs Reality)
- VerificationEvidence (Working Code vs Documentation)

**Invariants**:

- No phase advancement without verified functionality
- Documentation reflects actual implementation, not aspirations

#### Context 2: Anti-Pattern Prevention

**Purpose**: Actively prevent common LLM/AI project failures
**Core Entities**:

- ForbiddenPattern (Premature Distribution, Complex Abstractions)
- RealityCheck (Questions before implementation)
- ComplexityTrap (Distributed systems, AI orchestration, Performance)

**Invariants**:

- Every implementation must pass simplicity test first
- No distributed features before local execution works

#### Context 3: Incremental Development Control

**Purpose**: Enforce strict phase-based development
**Core Entities**:

- DevelopmentPhase (0: Hello World → 3: Basic Supervision)
- PhaseGate (Verification requirements per phase)
- SuccessMetric (Executable proof, not theoretical compliance)

**Invariants**:

- Phase N+1 forbidden until Phase N verification complete
- Each phase has explicit, executable success criteria

#### Context 4: LLM Integration Guidance

**Purpose**: Prevent AI system over-engineering
**Core Entities**:

- IntegrationLevel (Subprocess → Session → Orchestration)
- VerificationMethod (Command output, Process state, Message logs)
- ComplexityBudget (Start simple, expand only with proof)

**Invariants**:

- No session management before subprocess execution works
- No multi-agent coordination before single agent reliable

### 2.2 Aggregate Design

#### Instruction Aggregate Root

```
Instruction {
    id: SectionIdentifier
    purpose: VerifiablePurpose
    constraints: List<ExecutableConstraint>
    verification: RequiredProof
    antiPatterns: List<ForbiddenBehavior>
    examples: ConcreteImplementation
}
```

#### Verification Aggregate

```
VerificationRequirement {
    phase: DevelopmentPhase
    requiredCommands: List<ExecutableCommand>
    expectedOutput: ConcreteEvidence
    forbiddenClaims: List<TheoreticalAssertion>
}
```

### 2.3 Domain Events

- PhaseCompletionVerified
- AntiPatternDetected
- ComplexityThresholdExceeded
- VerificationFailed

## 3. Content Architecture

### 3.1 Section Hierarchy

```
1. PROJECT CONTEXT
   └─ System Reality Statement
   └─ Current Implementation Status
   └─ Goal vs Reality Mapping

2. CRITICAL PROJECT CONSTRAINTS
   ├─ ANTI-PATTERN PREVENTION
   │  └─ Forbidden Behaviors
   │  └─ Complexity Traps
   └─ MISTERSMITH-SPECIFIC REALITY CHECKS
      └─ Pre-implementation Questions
      └─ Verification Requirements

3. INCREMENTAL DEVELOPMENT PATH
   ├─ PHASE 0: Foundation
   ├─ PHASE 1: Single Agent  
   ├─ PHASE 2: Local Communication
   └─ PHASE 3: Basic Orchestration

4. LLM/AI SYSTEM GUIDELINES
   ├─ Claude Integration Reality
   └─ Multi-Agent AI Workflows

5. VERIFICATION REQUIREMENTS
   ├─ Universal Component Tests
   └─ MisterSmith-Specific Tests

6. DOCUMENTATION REALITY
   ├─ Specification vs Implementation
   └─ Documentation Update Protocol

7. PROJECT-SPECIFIC GOTCHAS
   ├─ Distributed Systems Trap
   ├─ AI/LLM Integration Trap
   └─ Performance Optimization Trap

8. SUCCESS METRICS
   ├─ Phase Success Criteria
   └─ Non-Success Indicators

9. DEVELOPMENT COMMANDS
   └─ Ordered Verification Sequence
```

### 3.2 Content Patterns

#### Pattern 1: Forbidden/Required Pairs

```
FORBIDDEN: Complex behavior before simple works
REQUIRED: Simple working example with verification
```

#### Pattern 2: Code Reality Examples

```
// CORRECT: Start simple
[minimal working code]

// INCORRECT: Complex abstraction
[overengineered example]
```

#### Pattern 3: Verification Commands

```bash
# MUST provide:
[specific executable commands]
[expected concrete output]

# FORBIDDEN:
[theoretical claims without proof]
```

## 4. API Design

### 4.1 Instruction Interface Contract

```yaml
GlobalInstructions:
  provides:
    - FoundationFirstPhilosophy
    - VerificationProtocol
    - AutonomyBoundaries
    - QualityEnforcement

ProjectInstructions:
  requires:
    - GlobalInstructions.all
  extends:
    - ProjectSpecificConstraints
    - DomainVerification
    - PhaseEnforcement
  never:
    - Overrides global rules
    - Weakens verification
    - Allows theoretical solutions
```

### 4.2 Phase Transition API

```rust
trait DevelopmentPhase {
    fn current_phase() -> Phase;
    fn verification_requirements() -> Vec<VerificationCommand>;
    fn can_advance() -> Result<bool, BlockingIssue>;
    fn advance_phase() -> Result<Phase, VerificationFailure>;
}
```

### 4.3 Verification API

```rust
trait ProjectVerification {
    fn verify_component(component: &Component) -> VerificationResult;
    fn required_evidence() -> Vec<ExecutableTest>;
    fn forbidden_claims() -> Vec<TheoreticalAssertion>;
}
```

## 5. Evolution Strategy

### 5.1 Instruction Lifecycle

1. **Initial State**: Maximum constraints, minimal scope
2. **Verified Growth**: Constraints relax as verification accumulates
3. **Maturity Indicators**: Working code density > documentation density
4. **Maintenance Mode**: Focus shifts to regression prevention

### 5.2 Update Triggers

- Phase completion verified → Update allowed phases
- Anti-pattern detected → Add specific prevention
- New complexity trap discovered → Document gotcha
- Verification method proven → Add to requirements

### 5.3 Deprecation Path

Instructions become deprecated when:

- Implementation proves them unnecessary
- Better verification method discovered
- Complexity never materialized
- Feature proven impossible/impractical

## 6. Measurement & Monitoring

### 6.1 Instruction Effectiveness Metrics

- Anti-pattern prevention rate
- Phase advancement velocity
- Verification command usage
- Reality check trigger frequency

### 6.2 Health Indicators

- ✅ High ratio of executed commands to theoretical discussion
- ✅ Incremental progress with working examples
- ❌ Attempting phase N+2 features in phase N
- ❌ Theoretical architecture without executable proof

## 7. Implementation Plan

### 7.1 Content Creation Order

1. Write PROJECT CONTEXT with current reality
2. Define CRITICAL CONSTRAINTS based on anti-patterns
3. Create INCREMENTAL PHASES with concrete examples
4. Add VERIFICATION for each component type
5. Document GOTCHAS from actual experience

### 7.2 Validation Checklist

- [ ] Every constraint has executable verification
- [ ] No instruction enables theoretical solutions
- [ ] Examples show minimal working code first
- [ ] Phase progression requires concrete evidence
- [ ] Anti-patterns explicitly forbidden with rationale

### 7.3 Integration Testing

Test the instructions by:

1. Attempting to skip phases → Should fail
2. Claiming success without verification → Should fail
3. Building minimal working component → Should succeed
4. Following incremental path → Should progress smoothly

## 8. Risk Mitigation

### 8.1 Instruction Risks

- **Risk**: Instructions become too restrictive
  - **Mitigation**: Phases allow gradual relaxation
  
- **Risk**: Developers bypass verification
  - **Mitigation**: No phase advancement without proof

- **Risk**: Complexity creep in instructions
  - **Mitigation**: Regular simplification reviews

### 8.2 Anti-Pattern Insurance

Each major anti-pattern has:

- Explicit prohibition
- Reality check questions
- Verification requirement
- Simpler alternative provided

## 9. Summary

The MisterSmith CLAUDE.md design creates a verification-first instruction system that:

- Extends global anti-fabrication philosophy
- Enforces incremental development through phases
- Prevents LLM/AI system over-engineering
- Requires executable proof at every step
- Evolves based on actual implementation experience

Success metric: Developer builds working single agent before reading about distributed systems.
