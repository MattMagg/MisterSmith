# MisterSmith CLAUDE.md Bounded Context Map

## Context Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     Global Instructions Context                  │
│                   (Upstream - Published Language)                │
└───────┬─────────────────────────────────────────────┬───────────┘
        │                                             │
        │ Conforms                                    │ Customer/Supplier
        ▼                                             ▼
┌──────────────────────┐                    ┌────────────────────┐
│  Project Reality     │◄───────────────────│ Anti-Pattern       │
│  Management Context  │   Shared Kernel    │ Prevention Context │
└──────────────────────┘   (Reality State)  └────────────────────┘
        │                                             │
        │ Partnership                                 │ Partnership
        ▼                                             ▼
┌──────────────────────┐                    ┌────────────────────┐
│ Incremental Dev      │───────────────────►│ LLM Integration    │
│ Control Context      │    Conformist      │ Guidance Context   │
└──────────────────────┘  (Phase Gates)     └────────────────────┘
```

## Detailed Context Definitions

### 1. Project Reality Management Context

**Purpose**: Maintain accurate awareness of actual vs specified state

**Core Domain Language**:

```typescript
type ProjectReality = {
  currentState: {
    hasImplementation: boolean
    workingComponents: string[]
    verifiedFunctionality: string[]
  }
  specifiedState: {
    documentedComponents: string[]
    plannedArchitecture: string
    estimatedTimeline: string
  }
  gap: SpecificationRealityGap
}

type SpecificationRealityGap = {
  componentsNotImplemented: string[]
  documentationDebt: string[]
  complexityRatio: number // actual/specified
}
```

**Ubiquitous Language**:

- "Reality State": What actually exists and works
- "Specification Debt": Documented but not implemented
- "Verification Evidence": Executable proof of functionality
- "Implementation Status": Concrete working state

**Context Boundaries**:

- OWNS: Current state tracking, reality checks
- USES: Phase definitions from Incremental Dev Context
- PROVIDES: Reality state to all other contexts

### 2. Anti-Pattern Prevention Context

**Purpose**: Actively prevent known failure modes in LLM/AI projects

**Core Domain Language**:

```typescript
type AntiPattern = {
  id: string
  name: string
  detection: DetectionRule
  prevention: PreventionStrategy
  consequences: FailureMode[]
}

type DetectionRule = {
  trigger: CodePattern | BehaviorPattern
  warningLevel: 'CRITICAL' | 'HIGH' | 'MEDIUM'
  verificationRequired: VerificationCommand[]
}

type PreventionStrategy = {
  forbiddenAction: string
  requiredAlternative: string
  exampleCorrection: CodeExample
}
```

**Ubiquitous Language**:

- "Complexity Trap": Premature optimization/abstraction
- "Fabrication Pattern": Claims without verification
- "Distribution Trap": Building distributed before local works
- "AI Orchestration Trap": Complex AI before simple subprocess

**Context Boundaries**:

- OWNS: Anti-pattern definitions, detection rules
- CONSUMES: Reality state for detection
- ENFORCES: Constraints on all implementation

### 3. Incremental Development Control Context

**Purpose**: Enforce phase-based development with verification gates

**Core Domain Language**:

```typescript
type DevelopmentPhase = {
  id: number
  name: string
  entryRequirements: VerificationGate
  allowedFeatures: Feature[]
  forbiddenFeatures: Feature[]
  successCriteria: ExecutableTest[]
  transitionRules: PhaseTransition
}

type VerificationGate = {
  requiredTests: string[]
  requiredOutput: string[]
  minimumCodeExamples: number
  prohibitedClaims: string[]
}

type PhaseTransition = {
  from: Phase
  to: Phase
  requires: VerificationEvidence[]
  automaticChecks: Command[]
}
```

**Ubiquitous Language**:

- "Phase Gate": Verification requirement between phases
- "Feature Budget": What's allowed in current phase
- "Transition Criteria": Concrete evidence for advancement
- "Phase Lock": Cannot skip phases

**Context Boundaries**:

- OWNS: Phase definitions, transition rules
- ENFORCES: Feature limitations per phase
- REQUIRES: Reality state for phase verification

### 4. LLM Integration Guidance Context

**Purpose**: Guide incremental AI/LLM integration without over-engineering

**Core Domain Language**:

```typescript
type IntegrationLevel = {
  level: 'SUBPROCESS' | 'SESSION' | 'MANAGED' | 'ORCHESTRATED'
  requirements: TechnicalRequirement[]
  verificationMethod: VerificationApproach
  complexityScore: number
  allowedInPhase: DevelopmentPhase
}

type LLMIntegrationPattern = {
  name: string
  complexity: IntegrationLevel
  implementation: CodePattern
  verification: TestPattern
  antiPatterns: string[]
}
```

**Ubiquitous Language**:

- "Integration Level": Complexity of LLM interaction
- "Subprocess Pattern": Simple command execution
- "Session Management": Stateful LLM interaction
- "Orchestration Pattern": Multi-agent coordination

**Context Boundaries**:

- OWNS: LLM integration patterns, complexity rules
- CONFORMS TO: Phase restrictions from Incremental Context
- USES: Anti-patterns for guidance

## Inter-Context Communication

### 1. Shared Kernel: Reality State

All contexts share understanding of:

```typescript
interface RealityState {
  getCurrentPhase(): DevelopmentPhase
  getWorkingComponents(): Component[]
  getVerificationStatus(): VerificationResult[]
}
```

### 2. Context Events

```typescript
// Project Reality → Other Contexts
event RealityUpdated {
  component: string
  status: 'WORKING' | 'FAILED' | 'THEORETICAL'
  evidence: VerificationCommand[]
}

// Anti-Pattern → Development Control
event AntiPatternDetected {
  pattern: AntiPattern
  location: CodeLocation
  severity: WarningLevel
  remediation: PreventionStrategy
}

// Development Control → All Contexts
event PhaseTransitioned {
  from: Phase
  to: Phase
  unlockedFeatures: Feature[]
  newConstraints: Constraint[]
}

// LLM Guidance → Anti-Pattern
event ComplexityThresholdExceeded {
  integration: LLMIntegrationPattern
  currentComplexity: number
  maxAllowed: number
  simplification: AlternativePattern
}
```

### 3. Translation Maps

**Global Instructions → Project Contexts**:

```yaml
GlobalTerm: "Foundation-First Development"
ProjectTranslation: "Phase-Locked Incremental Development"

GlobalTerm: "Verification Protocol"  
ProjectTranslation: "Phase Gate Requirements"

GlobalTerm: "Anti-Patterns"
ProjectTranslation: "MisterSmith-Specific Traps"
```

**Between Project Contexts**:

```yaml
RealityTerm: "No Implementation"
DevelopmentTerm: "Phase 0 - Foundation Required"
AntiPatternTerm: "Specification-Only State"
LLMTerm: "Pre-Integration Phase"
```

## Context Evolution Rules

### 1. Context Boundaries Can Shift When

- New anti-patterns discovered → Anti-Pattern Context expands
- Phase proven unnecessary → Development Control simplifies
- Integration pattern validated → LLM Guidance promotes pattern
- Reality consistently matches spec → Reality Management scope reduces

### 2. Context Merging Conditions

- Two contexts share >80% of operations → Consider merging
- Contexts never communicate → Question separation value
- Single team owns both → Natural merge candidate

### 3. Context Splitting Triggers

- Single context has >10 aggregates → Consider splitting
- Multiple teams need different parts → Split along team lines
- Different change velocities → Separate stable from volatile

## Implementation Priority

1. **First**: Project Reality Management (ground truth)
2. **Second**: Anti-Pattern Prevention (protect from failure)
3. **Third**: Incremental Development Control (enforce progress)
4. **Fourth**: LLM Integration Guidance (domain-specific patterns)
